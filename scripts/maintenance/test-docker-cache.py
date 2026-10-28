#!/usr/bin/env python3
"""docker-cache.pyの単体検査と実Docker安全fixture。"""

from __future__ import annotations

import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import unittest
import uuid
from datetime import datetime, timezone
from typing import Any


sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/maintenance/docker-cache.py"
EVIDENCE_PARENT = ROOT / "tmp/verification/docker-cache-tests"
IMAGE_TAG = "kcu-screenshot-linux-arm64:local"
OWNER_LABEL = "io.katana-chat-ui.cache-owner"
OWNER_VALUE = "katana-chat-ui"
TEST_LABEL = "io.katana-chat-ui.docker-cache-test"
PURPOSE_LABEL = "io.katana-chat-ui.cache-purpose"
RETENTION_LABEL = "io.katana-chat-ui.retention"


def load_cache_module() -> Any:
    spec = importlib.util.spec_from_file_location("kcu_docker_cache", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


CACHE = load_cache_module()


class Receipt:
    def __init__(self, directory: Path, run_id: str, image_id: str | None) -> None:
        self.directory = directory
        self.path = directory / "receipt.json"
        self.data: dict[str, Any] = {
            "schema_version": 1,
            "run_id": run_id,
            "started_at": datetime.now(timezone.utc).isoformat(),
            "source_sha256": {
                "docker-cache.py": hashlib.sha256(SCRIPT.read_bytes()).hexdigest(),
                "test-docker-cache.py": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            },
            "image_tag_used_only_for_inspect": IMAGE_TAG,
            "immutable_image_id_used_for_all_containers": image_id,
            "pull_policy": "never",
            "network_policy": "none",
            "volumes": [],
            "containers": [],
            "checks": [],
            "cleanup": [],
            "actual_exit": None,
        }
        self.write()

    def write(self) -> None:
        temporary = self.path.with_suffix(".json.tmp")
        temporary.write_text(json.dumps(self.data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        os.replace(temporary, self.path)

    def event(self, collection: str, value: dict[str, Any]) -> None:
        self.data[collection].append(value)
        self.write()

    def set(self, key: str, value: Any) -> None:
        self.data[key] = value
        self.write()


class RecordingDocker(CACHE.DockerCli):
    """実Docker CLIを実行しながら呼出し結果を記録する。"""

    def __init__(self, command_log: str) -> None:
        self.command_log = command_log
        self.command_records: list[dict[str, Any]] = []

    def run(self, arguments: Any, *, check: bool = True) -> subprocess.CompletedProcess[str]:
        materialized = list(arguments)
        command = ["docker", *materialized]
        started_at = datetime.now(timezone.utc).isoformat()
        try:
            result = super().run(materialized, check=False)
        except CACHE.DockerCacheError as error:
            self._record(
                {
                    "command": command,
                    "started_at": started_at,
                    "finished_at": datetime.now(timezone.utc).isoformat(),
                    "returncode": None,
                    "exception": {"type": type(error).__name__, "message": str(error)},
                }
            )
            raise
        self._record(
            {
                "command": command,
                "started_at": started_at,
                "finished_at": datetime.now(timezone.utc).isoformat(),
                "returncode": result.returncode,
                "stdout_bytes": len(result.stdout.encode("utf-8")),
                "stdout_sha256": hashlib.sha256(result.stdout.encode("utf-8")).hexdigest(),
                "stderr_bytes": len(result.stderr.encode("utf-8")),
                "stderr_sha256": hashlib.sha256(result.stderr.encode("utf-8")).hexdigest(),
            }
        )
        if check and result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or "no Docker diagnostic"
            raise CACHE.DockerCacheError(
                f"docker {' '.join(materialized)} failed with exit {result.returncode}: {detail}"
            )
        return result

    def _record(self, record: dict[str, Any]) -> None:
        self.command_records.append(
            {"arguments": record["command"][1:], "returncode": record["returncode"]}
        )
        with open(self.command_log, "a", encoding="utf-8") as log:
            log.write(json.dumps(record, ensure_ascii=False) + "\n")
            log.flush()

    def commands_for(self, arguments: list[str]) -> int:
        return sum(1 for record in self.command_records if record["arguments"] == arguments)

    def command_log_records(self) -> list[dict[str, Any]]:
        path = Path(self.command_log or "")
        if not path.exists():
            return []
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]


class FixtureRegistryBase:
    """このtestが作成したrun token付きcontainerとvolumeだけを管理する。"""

    def __init__(self, run_id: str, image_id: str, directory: Path, receipt: Receipt) -> None:
        self.run_id = run_id
        self.prefix = f"kcu-issue14-test-{run_id}"
        self.image_id = image_id
        self.directory = directory
        self.receipt = receipt
        self.docker = CACHE.DockerCli()
        self.volume_labels = {
            OWNER_LABEL: OWNER_VALUE,
            TEST_LABEL: run_id,
        }
        self.container_labels = {OWNER_LABEL: OWNER_VALUE, TEST_LABEL: run_id}
        self.volume_names: set[str] = set()
        self.container_names: set[str] = set()
        self.volume_names_for_test: set[str] = set()
        self.cleaning = False
        self.cleanup_complete = False

    def _all_volume_names(self) -> set[str]:
        return CACHE._volume_names(self.docker)

    def _assert_volume_name_free(self, name: str) -> None:
        if name in self._all_volume_names():
            raise RuntimeError(f"fixture volume name collision; leaving it untouched: {name}")

    def _assert_container_name_free(self, name: str) -> None:
        result = self.docker.run(
            ["ps", "--all", "--quiet", "--no-trunc", "--filter", f"name=/{name}"]
        )
        for container_id in result.stdout.splitlines():
            if not container_id.strip():
                continue
            inspected = self.docker.run(["inspect", "--type", "container", container_id.strip()])
            data = json.loads(inspected.stdout)[0]
            if str(data.get("Name", "")).lstrip("/") == name:
                raise RuntimeError(f"fixture container name collision; leaving it untouched: {name}")

    def create_volume(self, suffix: str, *, purpose: str, retention: str | None = None) -> str:
        name = f"{self.prefix}-{suffix}"
        self._assert_volume_name_free(name)
        labels = {**self.volume_labels, PURPOSE_LABEL: purpose}
        if retention is not None:
            labels[RETENTION_LABEL] = retention
        generation = 1 + sum(1 for item in self.receipt.data["volumes"] if item["name"] == name)
        self.receipt.event("volumes", {
            "name": name,
            "generation": generation,
            "labels": labels,
            "state": "planned",
            "volume_id": None,
        })
        arguments = ["volume", "create"]
        for key, value in labels.items():
            arguments.extend(["--label", f"{key}={value}"])
        arguments.append(name)
        self.docker.run(arguments)
        volume = CACHE._inspect_volume(self.docker, name)
        observed_labels = volume.get("Labels") or {}
        if observed_labels != labels:
            self.receipt.event(
                "checks",
                {
                    "name": "fixture volume ownership confirmed",
                    "volume": name,
                    "passed": False,
                    "observed_labels": observed_labels,
                },
            )
            raise RuntimeError(f"fixture volume ownership did not match; preserving {name}")
        self.volume_names.add(name)
        entry = next(
            item for item in reversed(self.receipt.data["volumes"])
            if item["name"] == name and item["generation"] == generation
        )
        entry.update({
            "state": "created",
            "volume_id": volume.get("ID"),
            "identity": CACHE._identity(volume),
        })
        self.receipt.write()
        return name

    def create_container(
        self,
        suffix: str,
        volume_name: str,
        command: list[str],
    ) -> str:
        name = f"{self.prefix}-{suffix}"
        self._assert_container_name_free(name)
        self.container_names.add(name)
        self.receipt.event(
            "containers",
            {
                "name": name,
                "volume_name": volume_name,
                "state": "planned",
                "container_id": None,
                "image_id": self.image_id,
                "pull_policy": "never",
                "network": "none",
                "labels": self.container_labels,
            },
        )
        arguments = [
            "create",
            "--pull=never",
            "--platform=linux/arm64",
            "--network=none",
            "--name",
            name,
        ]
        for key, value in self.container_labels.items():
            arguments.extend(["--label", f"{key}={value}"])
        arguments.extend([
            "--mount",
            f"type=volume,src={volume_name},dst=/fixture",
            self.image_id,
            *command,
        ])
        result = self.docker.run(arguments)
        container_id = result.stdout.strip()
        if not re.fullmatch(r"[0-9a-f]{64}", container_id):
            raise RuntimeError(f"Docker did not return a complete container ID for {name}")
        inspected = self.docker.run(["inspect", "--type", "container", container_id])
        container = json.loads(inspected.stdout)[0]
        if (
            container.get("Id") != container_id
            or str(container.get("Name", "")).lstrip("/") != name
            or container.get("Config", {}).get("Labels", {}).get(OWNER_LABEL) != OWNER_VALUE
            or container.get("Config", {}).get("Labels", {}).get(TEST_LABEL) != self.run_id
            or not any(
                mount.get("Type") == "volume" and mount.get("Name") == volume_name
                for mount in container.get("Mounts", [])
            )
        ):
            raise RuntimeError(f"fixture container ownership/mount did not match for {name}")
        entry = next(item for item in self.receipt.data["containers"] if item["name"] == name)
        entry.update({"state": "created", "container_id": container_id})
        self.receipt.write()
        return container_id

    def start(self, container_id: str, *, attach: bool = False) -> subprocess.CompletedProcess[str]:
        arguments = ["start"]
        if attach:
            arguments.append("--attach")
        arguments.append(container_id)
        return self.docker.run(arguments)

    def stop(self, container_id: str) -> None:
        self.docker.run(["stop", "--timeout", "2", container_id])

    def record_check(self, name: str, passed: bool, **details: Any) -> None:
        self.receipt.event("checks", {"name": name, "passed": passed, **details})

    def cleanup(self) -> bool:
        if self.cleaning or self.cleanup_complete:
            return self.cleanup_complete
        self.cleaning = True
        succeeded = True
        try:
            found_ids = self.docker.run(
                ["ps", "--all", "--quiet", "--no-trunc", "--filter", f"label={TEST_LABEL}={self.run_id}"]
            ).stdout.splitlines()
            ids = {item["container_id"] for item in self.receipt.data["containers"] if item.get("container_id")}
            ids.update(item.strip() for item in found_ids if item.strip())
            for container_id in sorted(ids):
                try:
                    inspected = self.docker.run(["inspect", "--type", "container", container_id])
                    container = json.loads(inspected.stdout)[0]
                except Exception as error:  # Preserve uninspectable resources rather than guessing ownership.
                    self.receipt.event(
                        "cleanup",
                        {"resource": "container", "container_id": container_id, "status": "preserved_uninspectable", "error": str(error)},
                    )
                    succeeded = False
                    continue
                exact_name = str(container.get("Name", "")).lstrip("/")
                labels = container.get("Config", {}).get("Labels") or {}
                if (
                    exact_name not in self.container_names
                    or not exact_name.startswith(self.prefix + "-")
                    or labels.get(OWNER_LABEL) != OWNER_VALUE
                    or labels.get(TEST_LABEL) != self.run_id
                    or container.get("Id") != container_id
                ):
                    self.receipt.event(
                        "cleanup",
                        {"resource": "container", "container_id": container_id, "name": exact_name, "status": "preserved_identity_mismatch"},
                    )
                    succeeded = False
                    continue
                state = container.get("State", {}).get("Status")
                if state == "running":
                    stopped = self.docker.run(["stop", "--timeout", "2", container_id], check=False)
                    self.receipt.event(
                        "cleanup",
                        {"resource": "container", "container_id": container_id, "name": exact_name, "action": "stop", "exit": stopped.returncode},
                    )
                    if stopped.returncode != 0:
                        current = self.docker.run(["inspect", "--type", "container", container_id])
                        current_state = json.loads(current.stdout)[0].get("State", {}).get("Status")
                        if current_state == "running":
                            succeeded = False
                            continue
                removed = self.docker.run(["rm", container_id], check=False)
                all_ids = self.docker.run(["ps", "--all", "--quiet", "--no-trunc"])
                absence_verified = container_id not in {line.strip() for line in all_ids.stdout.splitlines()}
                self.receipt.event(
                    "cleanup",
                    {
                        "resource": "container",
                        "container_id": container_id,
                        "name": exact_name,
                        "action": "rm",
                        "exit": removed.returncode,
                        "absence_verified": absence_verified,
                        "stderr": removed.stderr.strip(),
                    },
                )
                if removed.returncode != 0 or not absence_verified:
                    succeeded = False

            existing = self._all_volume_names()
            planned_names = {
                item["name"]
                for item in self.receipt.data["volumes"]
                if item.get("name", "").startswith(self.prefix + "-")
            }
            for name in sorted(self.volume_names | planned_names):
                if name not in existing:
                    self.receipt.event("cleanup", {"resource": "volume", "name": name, "status": "already_absent"})
                    continue
                try:
                    volume = CACHE._inspect_volume(self.docker, name)
                    labels = volume.get("Labels") or {}
                except Exception as error:
                    self.receipt.event(
                        "cleanup",
                        {"resource": "volume", "name": name, "status": "preserved_uninspectable", "error": str(error)},
                    )
                    succeeded = False
                    continue
                if (
                    labels.get(OWNER_LABEL) != OWNER_VALUE
                    or labels.get(TEST_LABEL) != self.run_id
                    or not name.startswith(self.prefix + "-")
                ):
                    self.receipt.event(
                        "cleanup",
                        {"resource": "volume", "name": name, "status": "preserved_identity_mismatch", "labels": labels},
                    )
                    succeeded = False
                    continue
                references = CACHE._container_references(self.docker, name)
                owned_ids = {
                    item["container_id"]
                    for item in self.receipt.data["containers"]
                    if item.get("container_id")
                }
                if any(not reference["reference_verified"] or reference["container_id"] not in owned_ids for reference in references):
                    self.receipt.event(
                        "cleanup",
                        {"resource": "volume", "name": name, "status": "preserved_external_reference", "references": references},
                    )
                    succeeded = False
                    continue
                removed = self.docker.run(["volume", "rm", name], check=False)
                absent = name not in self._all_volume_names() if removed.returncode == 0 else False
                self.receipt.event(
                    "cleanup",
                    {"resource": "volume", "name": name, "action": "volume rm", "exit": removed.returncode, "absence_verified": absent, "stderr": removed.stderr.strip()},
                )
                if removed.returncode != 0 or not absent:
                    succeeded = False
        except Exception as error:
            self.receipt.event("cleanup", {"resource": "registry", "status": "cleanup_error", "error": str(error)})
            succeeded = False
        finally:
            self.cleaning = False
            self.cleanup_complete = succeeded
            self.receipt.set("cleanup_succeeded", succeeded)
        return succeeded


ACTIVE_REGISTRY: FixtureRegistry | None = None
PENDING_SIGNAL: int | None = None


class DockerCacheUnitTests(unittest.TestCase):
    def test_production_allowlist_is_exact(self) -> None:
        self.assertEqual(
            frozenset({"kcu-v010-linux-target", "kcu-v010-linux-registry"}),
            CACHE.PRODUCTION_POLICY.allowed_names,
        )

    def test_candidate_validation_rejects_unknown_wildcard_and_duplicate(self) -> None:
        policy = CACHE.PRODUCTION_POLICY
        with self.assertRaises(CACHE.CandidateRejected):
            CACHE.validate_candidates(["kcu-v010-linux-target*"], policy)
        with self.assertRaises(CACHE.CandidateRejected):
            CACHE.validate_candidates(["another-project-volume"], policy)
        with self.assertRaises(CACHE.CandidateRejected):
            CACHE.validate_candidates(["kcu-v010-linux-target", "kcu-v010-linux-target"], policy)

    def test_remove_command_failure_keeps_actual_exit_and_error(self) -> None:
        completed = subprocess.CompletedProcess(
            ["docker", "volume", "rm", "kcu-v010-linux-target"],
            17,
            "",
            "volume is in use",
        )
        result = CACHE.remove_result_from_exit("kcu-v010-linux-target", completed)
        self.assertEqual("rm_command_failed", result["status"])
        self.assertEqual(17, result["docker_exit_code"])
        self.assertEqual("volume is in use", result["stderr"])

    def test_empty_owner_policy_does_not_prove_ownership(self) -> None:
        policy = CACHE.VolumePolicy(allowed_names=frozenset({"fixture"}), expected_labels=())
        evidence = CACHE._owner_evidence("fixture", None, policy)
        self.assertFalse(evidence["accepted"])

    def test_evidence_retention_label_overrides_legacy_name(self) -> None:
        labels = {OWNER_LABEL: OWNER_VALUE, RETENTION_LABEL: "evidence"}
        evidence = CACHE._owner_evidence("kcu-v010-linux-target", labels, CACHE.PRODUCTION_POLICY)
        self.assertFalse(evidence["accepted"])
        self.assertEqual("explicit_evidence_retention_label", evidence["basis"])

    def test_non_default_volume_options_fail_closed(self) -> None:
        policy = CACHE.VolumePolicy(
            allowed_names=frozenset({"fixture"}),
            expected_labels=(),
            legacy_unlabeled_names=frozenset({"fixture"}),
        )
        for options in (None, {}):
            with self.subTest(options=options):
                volume = {
                    "Name": "fixture",
                    "Driver": "local",
                    "Scope": "local",
                    "Options": options,
                    "Labels": None,
                }
                record = CACHE._volume_record("fixture", volume, [], policy)
                self.assertTrue(record["eligible_for_removal"])

        volume["Options"] = {"type": "nfs", "o": "addr=192.0.2.10"}
        record = CACHE._volume_record("fixture", volume, [], policy)
        self.assertFalse(record["eligible_for_removal"])
        self.assertIn("volume has non-empty or unsupported mount options", record["reasons"])


class DockerCacheIntegrationTests(unittest.TestCase):
    def test_real_docker_cache_volume_contract(self) -> None:
        if ACTIVE_REGISTRY is None:
            self.fail("fixture registry was not initialized")
        registry = ACTIVE_REGISTRY
        eligible = registry.create_volume("unreferenced", purpose="target-cache")
        running_volume = registry.create_volume("running-reference", purpose="target-cache")
        stopped_volume = registry.create_volume("stopped-reference", purpose="target-cache")
        late_volume = registry.create_volume("late-reference", purpose="target-cache")
        wrong_purpose = registry.create_volume("wrong-purpose", purpose="verification-record", retention="evidence")
        unrelated = registry.create_volume("unrelated", purpose="unrelated-cache")
        evidence = registry.create_volume("evidence", purpose="verification-record", retention="evidence")
        registry.volume_names_for_test = {
            eligible,
            running_volume,
            stopped_volume,
            late_volume,
            wrong_purpose,
        }
        policy = CACHE.VolumePolicy(
            allowed_names=frozenset(registry.volume_names_for_test),
            expected_labels=(
                (OWNER_LABEL, OWNER_VALUE),
                (TEST_LABEL, registry.run_id),
                (PURPOSE_LABEL, "target-cache"),
            ),
            description=f"test-only explicit fixture policy {registry.run_id}",
        )

        running_id = registry.create_container(
            "holder-running",
            running_volume,
            ["/bin/bash", "-lc", "trap 'exit 0' TERM; while :; do sleep 5 & wait $!; done"],
        )
        registry.start(running_id)
        stopped_id = registry.create_container(
            "holder-stopped",
            stopped_volume,
            ["/bin/bash", "-lc", "trap 'exit 0' TERM; while :; do sleep 5 & wait $!; done"],
        )
        registry.start(stopped_id)
        registry.stop(stopped_id)

        first_preview = CACHE.preview_volumes(policy, registry.docker)
        registry.receipt.set("preview_before_attach", first_preview)
        by_name = {item["name"]: item for item in first_preview["volumes"]}
        self.assertEqual("present", by_name[eligible]["state"])
        self.assertTrue(by_name[eligible]["eligible_for_removal"])
        self.assertEqual([], by_name[eligible]["container_references"])
        self.assertEqual("running", by_name[running_volume]["container_references"][0]["state"])
        self.assertEqual("exited", by_name[stopped_volume]["container_references"][0]["state"])
        self.assertFalse(by_name[wrong_purpose]["owner_evidence"]["accepted"])
        self.assertIsNone(by_name[eligible]["volume_id"])
        self.assertEqual(64, len(by_name[eligible]["identity"]["fingerprint_sha256"]))
        self.assertNotIn(unrelated, by_name)
        self.assertNotIn(evidence, by_name)
        registry.record_check(
            "initial preview separates eligible, referenced and unowned candidates",
            True,
            eligible=eligible,
            running_reference=running_volume,
            stopped_reference=stopped_volume,
            excluded_unrelated=unrelated,
            excluded_evidence=evidence,
        )

        running_calls_before = registry.docker.commands_for(["volume", "rm", running_volume])
        stopped_calls_before = registry.docker.commands_for(["volume", "rm", stopped_volume])
        blocked = CACHE.remove_volumes([running_volume, stopped_volume, wrong_purpose], policy, registry.docker)
        registry.receipt.set("remove_referenced_and_wrong_owner_result", blocked)
        statuses = {item["name"]: item["status"] for item in blocked["volumes"]}
        self.assertFalse(blocked["success"])
        self.assertEqual("refused_container_reference", statuses[running_volume])
        self.assertEqual("refused_container_reference", statuses[stopped_volume])
        self.assertEqual("refused_owner_or_driver_mismatch", statuses[wrong_purpose])
        self.assertEqual(running_calls_before, registry.docker.commands_for(["volume", "rm", running_volume]))
        self.assertEqual(stopped_calls_before, registry.docker.commands_for(["volume", "rm", stopped_volume]))

        # preview後に参照を追加し、toolの再確認とdaemon自身のin-use拒否を別々に確認する。
        late_before = CACHE.preview_volumes(policy, registry.docker)
        late_record = next(item for item in late_before["volumes"] if item["name"] == late_volume)
        self.assertTrue(late_record["eligible_for_removal"])
        late_id = registry.create_container(
            "holder-late-stopped",
            late_volume,
            ["/bin/bash", "-lc", "trap 'exit 0' TERM; while :; do sleep 5 & wait $!; done"],
        )
        registry.start(late_id)
        registry.stop(late_id)
        daemon_rm = registry.docker.run(["volume", "rm", late_volume], check=False)
        registry.receipt.set(
            "daemon_in_use_rejection_after_unreferenced_scan",
            {
                "scanned_unreferenced": True,
                "attached_container_id": late_id,
                "docker_volume_rm_exit": daemon_rm.returncode,
                "stdout": daemon_rm.stdout.strip(),
                "stderr": daemon_rm.stderr.strip(),
                "force_used": False,
            },
        )
        self.assertNotEqual(0, daemon_rm.returncode)
        self.assertIn("in use", (daemon_rm.stdout + daemon_rm.stderr).lower())
        self.assertIn(late_volume, CACHE._volume_names(registry.docker))
        late_rm_calls_before = registry.docker.commands_for(["volume", "rm", late_volume])
        late_blocked = CACHE.remove_volumes([late_volume], policy, registry.docker)
        self.assertFalse(late_blocked["success"])
        self.assertEqual("refused_container_reference", late_blocked["volumes"][0]["status"])
        self.assertEqual(late_rm_calls_before, registry.docker.commands_for(["volume", "rm", late_volume]))

        removed = CACHE.remove_volumes([eligible], policy, registry.docker)
        registry.receipt.set("remove_unreferenced_result", removed)
        self.assertTrue(removed["success"])
        self.assertEqual("removed_and_absence_verified", removed["volumes"][0]["status"])
        after_delete = CACHE.preview_volumes(policy, registry.docker)
        absent = next(item for item in after_delete["volumes"] if item["name"] == eligible)
        self.assertEqual("absent", absent["state"])
        repeated_remove = CACHE.remove_volumes([eligible], policy, registry.docker)
        self.assertTrue(repeated_remove["success"])
        self.assertEqual("absent", repeated_remove["volumes"][0]["status"])

        # 削除済みfixtureを再作成し、新containerから生成したartifactが保持されることを確認する。
        recreated = registry.create_volume("unreferenced", purpose="target-cache")
        self.assertEqual(eligible, recreated)
        writer_id = registry.create_container(
            "regenerate-writer",
            recreated,
            ["/bin/bash", "-lc", "printf '%s\\n' 'cache-regenerated' > /fixture/cache-proof.txt"],
        )
        writer = registry.start(writer_id, attach=True)
        self.assertEqual(0, writer.returncode, writer.stderr)
        reader_id = registry.create_container(
            "regenerate-reader",
            recreated,
            ["/bin/bash", "-lc", "test \"$(cat /fixture/cache-proof.txt)\" = cache-regenerated"],
        )
        reader = registry.start(reader_id, attach=True)
        self.assertEqual(0, reader.returncode, reader.stderr)
        registry.record_check(
            "deleted fixture volume recreated and retained a regenerated artifact",
            True,
            volume_name=recreated,
            writer_container_id=writer_id,
            reader_container_id=reader_id,
            artifact="/fixture/cache-proof.txt",
        )

        names_after = CACHE._volume_names(registry.docker)
        for kept in (running_volume, stopped_volume, late_volume, wrong_purpose, unrelated, evidence):
            self.assertIn(kept, names_after)
        registry.record_check(
            "all referenced, wrong-purpose, unrelated and evidence fixtures remain",
            True,
            retained=[running_volume, stopped_volume, late_volume, wrong_purpose, unrelated, evidence],
        )

        before_unknown_cli = len(registry.docker.command_log_records())
        invalid_cli = subprocess.run(
            [sys.executable, str(SCRIPT), "--remove", "kcu-v010-linux-target*"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        registry.receipt.set(
            "production_cli_unknown_name_rejection",
            {"exit": invalid_cli.returncode, "stdout": invalid_cli.stdout.strip(), "stderr": invalid_cli.stderr.strip()},
        )
        self.assertEqual(2, invalid_cli.returncode)
        self.assertEqual(before_unknown_cli, len(registry.docker.command_log_records()))


class FixtureRegistry(FixtureRegistryBase):
    def __init__(self, run_id: str, image_id: str, directory: Path, receipt: Receipt) -> None:
        super().__init__(run_id, image_id, directory, receipt)
        self.docker = RecordingDocker(str(directory / "docker-commands.jsonl"))

    def commands_for(self, arguments: list[str]) -> int:
        assert isinstance(self.docker, RecordingDocker)
        return self.docker.commands_for(arguments)

    def command_log_records(self) -> list[dict[str, Any]]:
        assert isinstance(self.docker, RecordingDocker)
        return self.docker.command_log_records()


def _new_evidence_directory() -> tuple[str, Path]:
    run_id = uuid.uuid4().hex[:12]
    directory = EVIDENCE_PARENT / f"run-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}-{run_id}"
    directory.mkdir(parents=True, exist_ok=False)
    return run_id, directory


def _inspect_test_image(docker: CACHE.DockerCli) -> tuple[str, dict[str, Any]]:
    result = docker.run(["image", "inspect", IMAGE_TAG])
    values = json.loads(result.stdout)
    if not isinstance(values, list) or len(values) != 1:
        raise RuntimeError("local fixture image inspect returned an unexpected result")
    image = values[0]
    image_id = image.get("Id")
    if not isinstance(image_id, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", image_id):
        raise RuntimeError("fixture image did not resolve to a complete immutable image ID")
    if image.get("Os") != "linux" or image.get("Architecture") != "arm64":
        raise RuntimeError(f"fixture image has unexpected platform: {image.get('Os')}/{image.get('Architecture')}")
    return image_id, image


def _signal_handler(signum: int, _frame: Any) -> None:
    global PENDING_SIGNAL
    for handled_signal in (signal.SIGINT, signal.SIGTERM):
        signal.signal(handled_signal, signal.SIG_IGN)
    PENDING_SIGNAL = signum
    # 実行中のDocker CLIをsubprocess.runに回収させてからmain()のfinallyでfixtureを片付ける。
    raise SystemExit(128 + signum)


def main() -> int:
    global ACTIVE_REGISTRY, PENDING_SIGNAL
    EVIDENCE_PARENT.mkdir(parents=True, exist_ok=True)
    run_id, directory = _new_evidence_directory()
    preflight_docker = RecordingDocker(str(directory / "docker-commands.jsonl"))
    receipt: Receipt | None = Receipt(directory, run_id, None)
    registry: FixtureRegistry | None = None
    exit_code = 1
    output = io.StringIO()
    original_handlers = {sig: signal.signal(sig, _signal_handler) for sig in (signal.SIGINT, signal.SIGTERM)}
    try:
        preflight_docker.run(["info", "--format", "{{.ServerVersion}}|{{.OperatingSystem}}|{{.Containers}}"])
        image_id, image = _inspect_test_image(preflight_docker)
        receipt.set("immutable_image_id_used_for_all_containers", image_id)
        receipt.set(
            "image_inspect",
            {"id": image_id, "os": image.get("Os"), "architecture": image.get("Architecture")},
        )
        registry = FixtureRegistry(run_id, image_id, directory, receipt)
        ACTIVE_REGISTRY = registry
        suite = unittest.TestSuite()
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(DockerCacheUnitTests))
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(DockerCacheIntegrationTests))
        result = unittest.TextTestRunner(stream=output, verbosity=2).run(suite)
        exit_code = 0 if result.wasSuccessful() else 1
        receipt.set(
            "test_result",
            {
                "tests_run": result.testsRun,
                "failures": len(result.failures),
                "errors": len(result.errors),
                "was_successful": result.wasSuccessful(),
            },
        )
    except BaseException as error:
        exit_code = error.code if isinstance(error, SystemExit) and isinstance(error.code, int) else 1
        if receipt is not None:
            receipt.set("exception", {"type": type(error).__name__, "message": str(error)})
            if PENDING_SIGNAL is not None:
                receipt.event(
                    "checks",
                    {
                        "name": "termination signal received",
                        "signal": PENDING_SIGNAL,
                        "cleanup_started_after_current_docker_command_returns": True,
                    },
                )
        output.write(f"{type(error).__name__}: {error}\n")
    finally:
        if registry is not None:
            try:
                cleanup_succeeded = registry.cleanup()
            except Exception as error:
                cleanup_succeeded = False
                if receipt is not None:
                    receipt.set("cleanup_exception", {"type": type(error).__name__, "message": str(error)})
            if not cleanup_succeeded:
                exit_code = 1
        if receipt is not None:
            receipt.set("finished_at", datetime.now(timezone.utc).isoformat())
            receipt.set("actual_exit", exit_code)
        ACTIVE_REGISTRY = None
        PENDING_SIGNAL = None
        for sig, handler in original_handlers.items():
            signal.signal(sig, handler)
        report = output.getvalue()
        (directory / "test-output.log").write_text(report, encoding="utf-8")
        if receipt is not None:
            receipt.set("actual_exit", exit_code)
        manifest = []
        for path in sorted(directory.iterdir()):
            if path.is_file() and path.name != "sha256-manifest.txt":
                manifest.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}")
        (directory / "sha256-manifest.txt").write_text("\n".join(manifest) + "\n", encoding="utf-8")
        print(report, end="")
        print(f"docker cache fixture evidence: {directory}")
        print(f"actual_exit={exit_code}")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
