#!/usr/bin/env python3
"""KCUの登録済みDocker cache volumeをpreviewし、明示指定時だけ削除する。"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
import subprocess
import sys
from typing import Any, Iterable


OWNER_LABEL = "io.katana-chat-ui.cache-owner"
OWNER_VALUE = "katana-chat-ui"
RETENTION_LABEL = "io.katana-chat-ui.retention"
LEGACY_VOLUME_NAMES = (
    "kcu-v010-linux-target",
    "kcu-v010-linux-registry",
)
LEGACY_ALLOWLIST = frozenset(LEGACY_VOLUME_NAMES)


class DockerCacheError(RuntimeError):
    """Docker照会に失敗し、所有者や使用状態を確認できない。"""


class CandidateRejected(ValueError):
    """要求されたvolume名が指定policyの完全一致allowlistにない。"""


@dataclass(frozen=True)
class VolumePolicy:
    """CLI固定allowlistと隔離Docker fixtureの内部allowlist。"""

    allowed_names: frozenset[str]
    expected_labels: tuple[tuple[str, str], ...]
    legacy_unlabeled_names: frozenset[str] = frozenset()
    description: str = ""


PRODUCTION_POLICY = VolumePolicy(
    allowed_names=LEGACY_ALLOWLIST,
    expected_labels=((OWNER_LABEL, OWNER_VALUE), (RETENTION_LABEL, "cache")),
    legacy_unlabeled_names=LEGACY_ALLOWLIST,
    description="KCU fixed-name legacy cache allowlist",
)


class DockerCli:
    """shellを介さずDocker CLIを実行する。"""

    def run(self, arguments: Iterable[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
        command = ["docker", *arguments]
        try:
            result = subprocess.run(command, capture_output=True, text=True, check=False)
        except OSError as error:
            raise DockerCacheError(f"could not execute Docker CLI: {error}") from error
        if check and result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or "no Docker diagnostic"
            raise DockerCacheError(
                f"docker {' '.join(arguments)} failed with exit {result.returncode}: {detail}"
            )
        return result


def validate_candidates(names: Iterable[str], policy: VolumePolicy) -> tuple[str, ...]:
    candidates = tuple(names)
    if not candidates:
        raise CandidateRejected("at least one exact volume name is required")
    if len(set(candidates)) != len(candidates):
        raise CandidateRejected("duplicate volume names are not accepted")
    for name in candidates:
        if not name or any(character in name for character in "*?[]/\\"):
            raise CandidateRejected(f"invalid exact volume name: {name!r}")
        if name not in policy.allowed_names:
            raise CandidateRejected(f"volume is not in the fixed allowlist: {name}")
    return candidates


def _volume_names(docker: DockerCli) -> set[str]:
    result = docker.run(["volume", "ls", "--quiet"])
    return {line.strip() for line in result.stdout.splitlines() if line.strip()}


def _inspect_volume(docker: DockerCli, name: str) -> dict[str, Any]:
    result = docker.run(["volume", "inspect", name])
    try:
        data = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise DockerCacheError(f"docker volume inspect returned invalid JSON for {name}") from error
    if not isinstance(data, list) or len(data) != 1 or not isinstance(data[0], dict):
        raise DockerCacheError(f"docker volume inspect returned an unexpected result for {name}")
    if data[0].get("Name") != name:
        raise DockerCacheError(f"docker volume inspect identity did not match requested name {name}")
    return data[0]


def _container_references(docker: DockerCli, volume_name: str) -> list[dict[str, Any]]:
    listed = docker.run(
        ["ps", "--all", "--quiet", "--no-trunc", "--filter", f"volume={volume_name}"]
    )
    references: list[dict[str, Any]] = []
    for container_id in (line.strip() for line in listed.stdout.splitlines()):
        if not container_id:
            continue
        inspected = docker.run(["inspect", "--type", "container", container_id])
        try:
            containers = json.loads(inspected.stdout)
        except json.JSONDecodeError as error:
            raise DockerCacheError(f"docker inspect returned invalid JSON for container {container_id}") from error
        if not isinstance(containers, list) or len(containers) != 1 or not isinstance(containers[0], dict):
            raise DockerCacheError(f"docker inspect returned an unexpected container result for {container_id}")
        container = containers[0]
        matches = [
            mount
            for mount in container.get("Mounts", [])
            if mount.get("Type") == "volume" and mount.get("Name") == volume_name
        ]
        if not matches:
            # filterとinspectの結果が食い違う場合は、参照なしと断定せず削除を止める。
            references.append(
                {
                    "container_id": container.get("Id", container_id),
                    "name": str(container.get("Name", "")).lstrip("/"),
                    "state": container.get("State", {}).get("Status", "unknown"),
                    "mount_destination": None,
                    "read_write": None,
                    "reference_verified": False,
                }
            )
            continue
        for mount in matches:
            references.append(
                {
                    "container_id": container.get("Id", container_id),
                    "name": str(container.get("Name", "")).lstrip("/"),
                    "state": container.get("State", {}).get("Status", "unknown"),
                    "mount_destination": mount.get("Destination"),
                    "read_write": mount.get("RW"),
                    "reference_verified": True,
                }
            )
    return references


def _labels(volume: dict[str, Any]) -> dict[str, str] | None:
    labels = volume.get("Labels")
    if labels is None:
        return None
    if not isinstance(labels, dict) or not all(
        isinstance(key, str) and isinstance(value, str) for key, value in labels.items()
    ):
        raise DockerCacheError(f"volume {volume.get('Name')} has malformed labels")
    return labels


def _owner_evidence(name: str, labels: dict[str, str] | None, policy: VolumePolicy) -> dict[str, Any]:
    normalized = labels or {}
    required = dict(policy.expected_labels)
    labels_match = bool(required) and all(
        normalized.get(key) == value for key, value in required.items()
    )
    preserve_label = normalized.get(RETENTION_LABEL) == "evidence"
    legacy_name_match = name in policy.legacy_unlabeled_names
    legacy_unlabeled = labels is None or labels == {}
    if preserve_label:
        accepted = False
        basis = "explicit_evidence_retention_label"
    elif labels_match:
        accepted = True
        basis = "required_owner_labels"
    elif legacy_name_match and legacy_unlabeled:
        accepted = True
        basis = "exact_legacy_allowlist_name_without_labels"
    else:
        accepted = False
        basis = "owner_not_proven"
    return {
        "expected_labels": required,
        "observed_owner_label": normalized.get(OWNER_LABEL),
        "observed_retention_label": normalized.get(RETENTION_LABEL),
        "exact_allowlist_name_match": name in policy.allowed_names,
        "legacy_unlabeled_exception": legacy_name_match and legacy_unlabeled,
        "accepted": accepted,
        "basis": basis,
    }


def _identity(volume: dict[str, Any]) -> dict[str, Any]:
    attributes = {
        "name": volume.get("Name"),
        "created_at": volume.get("CreatedAt"),
        "driver": volume.get("Driver"),
        "scope": volume.get("Scope"),
        "options": volume.get("Options"),
        "labels": volume.get("Labels"),
    }
    encoded = json.dumps(attributes, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return {
        **attributes,
        "fingerprint_sha256": hashlib.sha256(encoded).hexdigest(),
        "fingerprint_scope": "inspect metadata only; Docker named volumes expose no immutable ID",
    }


def _volume_options_are_local_default(volume: dict[str, Any]) -> bool:
    return volume.get("Options") is None or volume.get("Options") == {}


def _volume_record(
    name: str,
    volume: dict[str, Any],
    references: list[dict[str, Any]],
    policy: VolumePolicy,
) -> dict[str, Any]:
    labels = _labels(volume)
    owner_evidence = _owner_evidence(name, labels, policy)
    reasons: list[str] = []
    if not owner_evidence["accepted"]:
        reasons.append("owner evidence did not match fixed policy")
    if references:
        reasons.append("one or more running, stopped, or unresolved container references exist")
    if volume.get("Driver") != "local":
        reasons.append("volume driver is not local")
    if volume.get("Scope") != "local":
        reasons.append("volume scope is not local")
    if not _volume_options_are_local_default(volume):
        reasons.append("volume has non-empty or unsupported mount options")
    return {
        "name": name,
        "volume_id": volume.get("ID"),
        "volume_id_source": (
            "docker volume inspect ID field" if volume.get("ID") else "not exposed by Docker; see identity fingerprint"
        ),
        "labels": labels,
        "identity": _identity(volume),
        "driver": volume.get("Driver"),
        "scope": volume.get("Scope"),
        "owner_evidence": owner_evidence,
        "container_references": references,
        "eligible_for_removal": not reasons,
        "reasons": reasons,
        "state": "present",
    }


def _absent_record(name: str, policy: VolumePolicy) -> dict[str, Any]:
    return {
        "name": name,
        "volume_id": None,
        "volume_id_source": "volume absent",
        "labels": None,
        "identity": None,
        "driver": None,
        "scope": None,
        "owner_evidence": {
            "expected_labels": dict(policy.expected_labels),
            "observed_owner_label": None,
            "observed_retention_label": None,
            "exact_allowlist_name_match": name in policy.allowed_names,
            "legacy_unlabeled_exception": False,
            "accepted": False,
            "basis": "volume_absent",
        },
        "container_references": [],
        "eligible_for_removal": False,
        "reasons": ["volume does not exist"],
        "state": "absent",
    }


def preview_volumes(policy: VolumePolicy, docker: DockerCli) -> dict[str, Any]:
    names = tuple(sorted(policy.allowed_names))
    existing = _volume_names(docker)
    records = []
    for name in names:
        if name not in existing:
            records.append(_absent_record(name, policy))
            continue
        volume = _inspect_volume(docker, name)
        references = _container_references(docker, name)
        records.append(_volume_record(name, volume, references, policy))
    return {
        "schema_version": 1,
        "operation": "preview",
        "policy": policy.description,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "volumes": records,
    }


def remove_result_from_exit(name: str, completed: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    """docker volume rmの実exitとdiagnosticを失わずresultへ写す。"""
    if completed.returncode == 0:
        return {
            "name": name,
            "status": "rm_command_succeeded",
            "docker_exit_code": completed.returncode,
            "stdout": completed.stdout.strip(),
            "stderr": completed.stderr.strip(),
        }
    return {
        "name": name,
        "status": "rm_command_failed",
        "docker_exit_code": completed.returncode,
        "stdout": completed.stdout.strip(),
        "stderr": completed.stderr.strip(),
    }


def remove_volumes(
    names: Iterable[str],
    policy: VolumePolicy,
    docker: DockerCli,
) -> dict[str, Any]:
    candidates = validate_candidates(names, policy)
    results: list[dict[str, Any]] = []
    overall_success = True
    for name in candidates:
        existing = _volume_names(docker)
        if name not in existing:
            results.append({"name": name, "status": "absent", "docker_exit_code": 0})
            continue
        volume = _inspect_volume(docker, name)
        references = _container_references(docker, name)
        record = _volume_record(name, volume, references, policy)
        if (
            volume.get("Driver") != "local"
            or volume.get("Scope") != "local"
            or not record["owner_evidence"]["accepted"]
        ):
            results.append({
                "name": name,
                "status": "refused_owner_or_driver_mismatch",
                "record": record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue
        if references:
            results.append({
                "name": name,
                "status": "refused_container_reference",
                "record": record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue
        if not _volume_options_are_local_default(volume):
            results.append({
                "name": name,
                "status": "refused_unsupported_volume_options",
                "record": record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue

        # 別processの利用開始やvolume置換を検知するため、削除直前にidentityと参照を再取得する。
        latest_names = _volume_names(docker)
        if name not in latest_names:
            results.append({"name": name, "status": "absent_before_rm", "docker_exit_code": 0})
            continue
        latest_volume = _inspect_volume(docker, name)
        latest_references = _container_references(docker, name)
        latest_record = _volume_record(name, latest_volume, latest_references, policy)
        if latest_record["identity"]["fingerprint_sha256"] != record["identity"]["fingerprint_sha256"]:
            results.append({
                "name": name,
                "status": "refused_identity_changed_before_rm",
                "record": latest_record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue
        if latest_references:
            results.append({
                "name": name,
                "status": "refused_container_reference_before_rm",
                "record": latest_record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue
        if (
            not latest_record["owner_evidence"]["accepted"]
            or latest_volume.get("Driver") != "local"
            or latest_volume.get("Scope") != "local"
        ):
            results.append({
                "name": name,
                "status": "refused_owner_or_driver_mismatch_before_rm",
                "record": latest_record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue
        if not _volume_options_are_local_default(latest_volume):
            results.append({
                "name": name,
                "status": "refused_unsupported_volume_options_before_rm",
                "record": latest_record,
                "docker_exit_code": None,
            })
            overall_success = False
            continue

        # 最終確認後の競合はDockerに委ね、forceを使わずdaemonの失敗をそのまま記録する。
        rm = docker.run(["volume", "rm", name], check=False)
        result = remove_result_from_exit(name, rm)
        if rm.returncode != 0:
            results.append(result)
            overall_success = False
            continue
        if name in _volume_names(docker):
            results.append({
                **result,
                "status": "rm_succeeded_but_volume_still_present",
            })
            overall_success = False
            continue
        results.append({**result, "status": "removed_and_absence_verified"})
    return {
        "schema_version": 1,
        "operation": "remove",
        "policy": policy.description,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "success": overall_success,
        "volumes": results,
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Preview KCU's fixed Docker cache volumes; removal requires each exact name."
    )
    parser.allow_abbrev = False
    parser.add_argument(
        "--remove",
        action="append",
        choices=sorted(PRODUCTION_POLICY.allowed_names),
        metavar="EXACT_VOLUME_NAME",
        help="remove one explicitly named, allowlisted cache volume; may be repeated",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _parser()
    args = parser.parse_args(argv)
    docker = DockerCli()
    try:
        if args.remove:
            result = remove_volumes(args.remove, PRODUCTION_POLICY, docker)
            print(json.dumps(result, ensure_ascii=False, sort_keys=True))
            return 0 if result["success"] else 1
        result = preview_volumes(PRODUCTION_POLICY, docker)
        print(json.dumps(result, ensure_ascii=False, sort_keys=True))
        return 0
    except (CandidateRejected, DockerCacheError) as error:
        print(
            json.dumps(
                {
                    "schema_version": 1,
                    "operation": "error",
                    "error": str(error),
                },
                ensure_ascii=False,
                sort_keys=True,
            )
        )
        return 2


if __name__ == "__main__":
    sys.exit(main())
