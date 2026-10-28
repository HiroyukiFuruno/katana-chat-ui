from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
from unittest.mock import patch


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import dispatch
import issue_gate


BASE_MANIFEST = b'''[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nalpha = "1"\n'''
BASE_LOCK = b'''version = 4\n\n[[package]]\nname = "sample"\nversion = "0.1.0"\ndependencies = ["alpha 1.0.0"]\n\n[[package]]\nname = "alpha"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n\n[[package]]\nname = "transitive"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n'''
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def git(repo: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(["git", "-C", str(repo), *args], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if check and result.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {result.stderr}")
    return result


def write(repo: Path, relative: str, content: bytes) -> None:
    path = repo / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(content)


def commit(repo: Path, message: str) -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "-m", message)
    return git(repo, "rev-parse", "HEAD").stdout.strip()


def repository_fixture(root: Path) -> tuple[Path, str]:
    remote = root / "remote.git"
    repo = root / "repo"
    git(root, "init", "--bare", "--initial-branch=main", str(remote))
    git(root, "init", "--initial-branch=main", str(repo))
    git(repo, "config", "user.name", "Issue gate test")
    git(repo, "config", "user.email", "issue-gate@example.invalid")
    write(repo, "Cargo.toml", BASE_MANIFEST)
    write(repo, "Cargo.lock", BASE_LOCK)
    write(repo, "Justfile", b'COVERAGE_MIN_LINES := env_var_or_default("COVERAGE_MIN_LINES", "77")\n')
    base = commit(repo, "chore: initial fixture")
    git(repo, "remote", "add", "origin", str(remote))
    git(repo, "push", "-u", "origin", "main")
    git(repo, "fetch", "origin")
    git(repo, "remote", "set-head", "origin", "main")
    return repo, base


def lock_with_versions(alpha: str, transitive: str) -> bytes:
    return f'''version = 4

[[package]]
name = "sample"
version = "0.1.0"
dependencies = ["alpha {alpha}"]

[[package]]
name = "alpha"
version = "{alpha}"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "transitive"
version = "{transitive}"
source = "registry+https://github.com/rust-lang/crates.io-index"
'''.encode()


def evidence_for(
    repo: Path,
    changed_files: list[str],
    lock: bytes,
    *,
    alpha: str,
    transitive: str,
    local_sample_version: str | None = None,
) -> bytes:
    manifest_hash = hashlib.sha256((repo / "Cargo.toml").read_bytes()).hexdigest()
    lock_hash = hashlib.sha256(lock).hexdigest()
    direct = []
    transitive_changes = []
    if alpha != "1.0.0":
        direct.append({"name": "alpha", "source": REGISTRY_SOURCE, "from": "1.0.0", "to": alpha})
    if transitive != "1.0.0":
        transitive_changes.append({"name": "transitive", "source": REGISTRY_SOURCE, "from": "1.0.0", "to": transitive})
    if local_sample_version is not None:
        transitive_changes.append({"name": "sample", "source": "local", "from": "0.1.0", "to": local_sample_version})
    value = {
        "upstream_release": {"url": "https://example.invalid/releases/v2", "version": "2.0.0"},
        "api_migration_note": "No migration required for this fixture.",
        "scopes": [{
            "manifest": "Cargo.toml",
            "manifest_sha256": manifest_hash,
            "lockfile": "Cargo.lock",
            "lock_sha256": lock_hash,
            "changed_files": sorted(changed_files),
            "direct_changes": direct,
            "transitive_changes": transitive_changes,
        }],
        "verification": {"commands": ["just unit-test"], "strict_quality_gate": "just release-verify"},
    }
    return (json.dumps(value, indent=2) + "\n").encode()


def evidence_for_scope(
    repo: Path,
    manifest_path: str,
    lockfile_path: str,
    changed_files: list[str],
    old_manifest: bytes,
    old_lock: bytes,
) -> bytes:
    manifest = (repo / manifest_path).read_bytes()
    lock = (repo / lockfile_path).read_bytes()
    direct_names = issue_gate._direct_dependency_names(old_manifest, manifest_path) | issue_gate._direct_dependency_names(manifest, manifest_path)
    direct, transitive = issue_gate._package_changes(
        issue_gate._cargo_packages(old_lock, lockfile_path),
        issue_gate._cargo_packages(lock, lockfile_path),
        direct_names,
    )
    value = {
        "upstream_release": {"url": "https://example.invalid/releases/v2", "version": "2.0.0"},
        "api_migration_note": "No migration required for this fixture.",
        "scopes": [{
            "manifest": manifest_path,
            "manifest_sha256": hashlib.sha256(manifest).hexdigest(),
            "lockfile": lockfile_path,
            "lock_sha256": hashlib.sha256(lock).hexdigest(),
            "changed_files": sorted(changed_files),
            "direct_changes": direct,
            "transitive_changes": transitive,
        }],
        "verification": {"commands": ["just unit-test"], "strict_quality_gate": "just release-verify"},
    }
    return (json.dumps(value, indent=2) + "\n").encode()


class IssueGateTests(unittest.TestCase):
    def test_issue_reference_requires_positive_number(self) -> None:
        self.assertEqual(issue_gate.issue_numbers("fix: use API; Issue #23"), {23})
        self.assertEqual(issue_gate.issue_numbers("fix: update #0"), set())
        self.assertEqual(issue_gate.issue_numbers("issue-0"), set())
        self.assertEqual(issue_gate.issue_numbers("other/repo#123"), set())
        self.assertEqual(issue_gate.issue_numbers("other/issue#123"), set())
        self.assertEqual(issue_gate.issue_numbers("see https://github.com/other/repo/issues/123#45"), set())
        self.assertEqual(issue_gate.issue_numbers("fix: local #123"), {123})
        self.assertEqual(issue_gate.issue_numbers("fix: local Issue #123"), {123})

    def test_commit_message_gate_requires_issue_on_release_branch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, _ = repository_fixture(Path(directory))
            git(repo, "switch", "-c", "release/v1.2.3")
            with self.assertRaisesRegex(issue_gate.GateError, "positive Issue reference"):
                issue_gate.check_commit_message(repo, "fix: no reference")
            with patch.dict("os.environ", {"KATANA_HOOK_BRANCH_OVERRIDE": "main", "KATANA_HOOK_COMMIT_SUBJECT_OVERRIDE": "fix #1"}):
                with self.assertRaisesRegex(issue_gate.GateError, "positive Issue reference"):
                    issue_gate.check_commit_message(repo, "fix: no reference")
            for foreign_only in (
                "chore: external link other/repo#17",
                "chore: external link other/issue#17",
                "docs: see https://github.com/other/repo/issues/17#45",
            ):
                with self.subTest(message=foreign_only), self.assertRaisesRegex(issue_gate.GateError, "positive Issue reference"):
                    issue_gate.check_commit_message(repo, foreign_only)
            issue_gate.check_commit_message(repo, "fix: linked #17")
            git(repo, "switch", "main")
            issue_gate.check_commit_message(repo, "chore: default branch commit")

    def test_manifest_and_lock_changes_require_exact_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            git(repo, "switch", "-c", "release/issue-42")
            target_lock = lock_with_versions("2.0.0", "2.0.0")
            write(repo, "Cargo.toml", BASE_MANIFEST.replace(b'alpha = "1"', b'alpha = "2"'))
            write(repo, "Cargo.lock", target_lock)
            write(repo, "issues/42/dependency-update-evidence.json", evidence_for(repo, ["Cargo.lock", "Cargo.toml"], target_lock, alpha="2.0.0", transitive="2.0.0"))
            target = commit(repo, "deps: update alpha Issue #42")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            self.assertTrue(issue_gate.validate_dependency_updates(repo, base, target, commits, messages))

            evidence = json.loads((repo / "issues/42/dependency-update-evidence.json").read_text())
            evidence["scopes"][0]["direct_changes"][0]["to"] = "9.9.9"
            write(repo, "issues/42/dependency-update-evidence.json", (json.dumps(evidence) + "\n").encode())
            tampered = commit(repo, "docs: alter evidence Issue #42")
            tampered_commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{tampered}")).splitlines()
            tampered_messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in tampered_commits}
            with self.assertRaisesRegex(issue_gate.GateError, "direct_changes does not match"):
                issue_gate.validate_dependency_updates(repo, base, tampered, tampered_commits, tampered_messages)
            evidence_only = str(issue_gate.git(repo, "rev-list", "--reverse", f"{target}..{tampered}")).splitlines()
            evidence_only_messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in evidence_only}
            with self.assertRaisesRegex(issue_gate.GateError, "evidence must change in the same range"):
                issue_gate.validate_dependency_updates(repo, target, tampered, evidence_only, evidence_only_messages)

    def test_manifest_only_equivalent_requirement_change_allows_empty_lock_diff(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            manifest = BASE_MANIFEST.replace(b'alpha = "1"', b'alpha = "=1.0.0"')
            write(repo, "Cargo.toml", manifest)
            write(
                repo,
                "issues/47/dependency-update-evidence.json",
                evidence_for_scope(repo, "Cargo.toml", "Cargo.lock", ["Cargo.toml"], BASE_MANIFEST, BASE_LOCK),
            )
            target = commit(repo, "deps: normalize alpha requirement Issue #47")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            self.assertTrue(issue_gate.validate_dependency_updates(repo, base, target, commits, messages))

    def test_multi_issue_commit_maps_scopes_to_their_unique_evidence_owners(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            root_manifest = BASE_MANIFEST.replace(b'alpha = "1"', b'alpha = "2"')
            root_lock = lock_with_versions("2.0.0", "1.0.0")
            child_manifest = BASE_MANIFEST.replace(b'name = "sample"', b'name = "child"').replace(b'alpha = "1"', b'alpha = "3"')
            child_lock = lock_with_versions("3.0.0", "1.0.0").replace(b'name = "sample"', b'name = "child"')
            write(repo, "Cargo.toml", root_manifest)
            write(repo, "Cargo.lock", root_lock)
            write(repo, "child/Cargo.toml", child_manifest)
            write(repo, "child/Cargo.lock", child_lock)
            write(repo, "issues/42/dependency-update-evidence.json", evidence_for_scope(repo, "Cargo.toml", "Cargo.lock", ["Cargo.lock", "Cargo.toml"], BASE_MANIFEST, BASE_LOCK))
            write(repo, "issues/43/dependency-update-evidence.json", evidence_for_scope(repo, "child/Cargo.toml", "child/Cargo.lock", ["child/Cargo.lock", "child/Cargo.toml"], b"", b""))
            target = commit(repo, "deps: update workspace scopes Issue #42 Issue #43")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            self.assertTrue(issue_gate.validate_dependency_updates(repo, base, target, commits, messages))

    def test_owner_can_be_selected_from_union_across_two_commits(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            git(repo, "switch", "-c", "release/issue-3-union")
            write(repo, "Cargo.toml", BASE_MANIFEST + b"\n# preparation tracked by Issue #13\n")
            preparation = commit(repo, "chore: prepare dependency scope Issue #13")

            target_manifest = BASE_MANIFEST.replace(b'alpha = "1"', b'alpha = "2"')
            target_lock = lock_with_versions("2.0.0", "1.0.0")
            write(repo, "Cargo.toml", target_manifest)
            write(repo, "Cargo.lock", target_lock)
            write(
                repo,
                "issues/3/dependency-update-evidence.json",
                evidence_for_scope(repo, "Cargo.toml", "Cargo.lock", ["Cargo.lock", "Cargo.toml"], BASE_MANIFEST, BASE_LOCK),
            )
            target = commit(repo, "deps: update alpha Issue #3")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            self.assertEqual(commits, [preparation, target])
            self.assertTrue(issue_gate.validate_dependency_updates(repo, base, target, commits, messages))

    def test_multi_issue_scope_with_two_evidence_owners_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            target_lock = lock_with_versions("2.0.0", "1.0.0")
            write(repo, "Cargo.toml", BASE_MANIFEST.replace(b'alpha = "1"', b'alpha = "2"'))
            write(repo, "Cargo.lock", target_lock)
            for issue_id in (42, 43):
                write(
                    repo,
                    f"issues/{issue_id}/dependency-update-evidence.json",
                    evidence_for_scope(repo, "Cargo.toml", "Cargo.lock", ["Cargo.lock", "Cargo.toml"], BASE_MANIFEST, BASE_LOCK),
                )
            target = commit(repo, "deps: update alpha Issue #42 Issue #43")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            with self.assertRaisesRegex(issue_gate.GateError, "ambiguous dependency evidence owner"):
                issue_gate.validate_dependency_updates(repo, base, target, commits, messages)

    def test_multi_issue_scope_without_matching_evidence_owner_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            target_lock = lock_with_versions("2.0.0", "1.0.0")
            write(repo, "Cargo.toml", BASE_MANIFEST.replace(b'alpha = "1"', b'alpha = "2"'))
            write(repo, "Cargo.lock", target_lock)
            write(repo, "issues/42/dependency-update-evidence.json", evidence_for_scope(repo, "Cargo.toml", "Cargo.lock", ["Cargo.lock", "Cargo.toml"], BASE_MANIFEST, BASE_LOCK))
            target = commit(repo, "deps: update alpha Issue #41 Issue #43")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            with self.assertRaisesRegex(issue_gate.GateError, "no uniquely referenced dependency-update-evidence.json owner"):
                issue_gate.validate_dependency_updates(repo, base, target, commits, messages)

    def test_lock_only_format_change_without_package_delta_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            target_lock = BASE_LOCK + b"\n"
            write(repo, "Cargo.lock", target_lock)
            write(
                repo,
                "issues/48/dependency-update-evidence.json",
                evidence_for_scope(repo, "Cargo.toml", "Cargo.lock", ["Cargo.lock"], BASE_MANIFEST, BASE_LOCK),
            )
            target = commit(repo, "deps: change lock formatting Issue #48")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            with self.assertRaisesRegex(issue_gate.GateError, "neither a package change nor an actual manifest change"):
                issue_gate.validate_dependency_updates(repo, base, target, commits, messages)

    def test_lock_only_change_still_requires_same_range_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            git(repo, "switch", "-c", "release/issue-44")
            target_lock = lock_with_versions("1.0.0", "2.0.0")
            write(repo, "Cargo.lock", target_lock)
            write(repo, "issues/44/dependency-update-evidence.json", evidence_for(repo, ["Cargo.lock"], target_lock, alpha="1.0.0", transitive="2.0.0"))
            target = commit(repo, "deps: update transitive crate Issue #44")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            self.assertTrue(issue_gate.validate_dependency_updates(repo, base, target, commits, messages))

            git(repo, "switch", "-c", "release/issue-45", base)
            write(repo, "Cargo.lock", lock_with_versions("1.0.0", "3.0.0"))
            target_without = commit(repo, "deps: update transitive crate Issue #45")
            commits_without = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target_without}")).splitlines()
            messages_without = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits_without}
            with self.assertRaisesRegex(issue_gate.GateError, "dependency-update-evidence.json"):
                issue_gate.validate_dependency_updates(repo, base, target_without, commits_without, messages_without)

    def test_local_workspace_package_patch_bump_requires_exact_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            git(repo, "switch", "-c", "release/issue-46")
            manifest = BASE_MANIFEST.replace(b'version = "0.1.0"', b'version = "0.1.1"', 1)
            target_lock = BASE_LOCK.replace(
                b'name = "sample"\nversion = "0.1.0"',
                b'name = "sample"\nversion = "0.1.1"',
                1,
            )
            write(repo, "Cargo.toml", manifest)
            write(repo, "Cargo.lock", target_lock)
            write(
                repo,
                "issues/46/dependency-update-evidence.json",
                evidence_for(
                    repo,
                    ["Cargo.lock", "Cargo.toml"],
                    target_lock,
                    alpha="1.0.0",
                    transitive="1.0.0",
                    local_sample_version="0.1.1",
                ),
            )
            target = commit(repo, "release: patch local package Issue #46")
            commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{target}")).splitlines()
            messages = {item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in commits}
            self.assertTrue(issue_gate.validate_dependency_updates(repo, base, target, commits, messages))

            evidence = json.loads((repo / "issues/46/dependency-update-evidence.json").read_text())
            evidence["scopes"][0]["transitive_changes"][0]["to"] = "0.1.9"
            write(repo, "issues/46/dependency-update-evidence.json", (json.dumps(evidence) + "\n").encode())
            tampered = commit(repo, "docs: mismatch local package evidence Issue #46")
            tampered_commits = str(issue_gate.git(repo, "rev-list", "--reverse", f"{base}..{tampered}")).splitlines()
            tampered_messages = {
                item: str(issue_gate.git(repo, "show", "-s", "--format=%B", item)) for item in tampered_commits
            }
            with self.assertRaisesRegex(issue_gate.GateError, "transitive_changes does not match"):
                issue_gate.validate_dependency_updates(repo, base, tampered, tampered_commits, tampered_messages)

    def test_cargo_lock_rejects_invalid_source_fields(self) -> None:
        lock = b'[[package]]\nname = "sample"\nversion = "0.1.0"\nsource = 42\n'
        with self.assertRaisesRegex(issue_gate.GateError, "invalid source"):
            issue_gate._cargo_packages(lock, "Cargo.lock")

    def test_dispatch_stops_on_release_gate_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, _ = repository_fixture(Path(directory))
            real_run = subprocess.run
            calls: list[list[str]] = []

            def run(args, *positional, **kwargs):
                if args[:2] == ["just", "release-verify"]:
                    calls.append(args)
                    return subprocess.CompletedProcess(args, 23)
                return real_run(args, *positional, **kwargs)

            with patch.object(subprocess, "run", side_effect=run):
                result = dispatch.run_dispatch(repo, "")
            self.assertEqual(result, 23)
            self.assertEqual(calls, [["just", "release-verify"]])

    def test_dispatch_preserves_high_coverage_and_rejects_lower_after_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, _ = repository_fixture(Path(directory))
            real_run = subprocess.run
            forwarded: dict[str, str] = {}
            calls: list[list[str]] = []

            def run(args, *positional, **kwargs):
                if args[:2] == ["just", "release-verify"]:
                    calls.append(args)
                    forwarded.update(kwargs.get("env", {}))
                    return subprocess.CompletedProcess(args, 0)
                return real_run(args, *positional, **kwargs)

            bypasses = {
                "JUSTFILE": "/tmp/fake.just",
                "RTK": "true",
                "CARGO": "/tmp/fake-cargo",
                "VERSION": "0.0.0",
                "COVERAGE_MIN_LINES": "88",
                "RUSTFLAGS": "--cap-lints allow",
            }
            with patch.dict(os.environ, bypasses):
                with patch.object(subprocess, "run", side_effect=run):
                    refs = f"refs/heads/main {str(issue_gate.git(repo, 'rev-parse', 'HEAD')).strip()} refs/heads/main {'0' * 40}\n"
                    result = dispatch.run_dispatch(repo, refs)
            self.assertEqual(result, 0)
            self.assertEqual(forwarded.get("COVERAGE_MIN_LINES"), "88")
            self.assertNotIn("RTK", forwarded)
            self.assertNotIn("JUSTFILE", forwarded)
            self.assertNotIn("CARGO", forwarded)
            self.assertNotIn("RUSTFLAGS", forwarded)

            for requested in ("76", "invalid"):
                forwarded.clear()
                calls.clear()
                with patch.dict(os.environ, {"COVERAGE_MIN_LINES": requested}, clear=False):
                    with patch.object(subprocess, "run", side_effect=run):
                        result = dispatch.run_dispatch(repo, refs)
                self.assertEqual(result, 1)
                self.assertEqual(calls, [["just", "release-verify"]])
                self.assertEqual(forwarded.get("COVERAGE_MIN_LINES"), "77")

    def test_dispatch_fails_closed_on_empty_pre_push_input(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, _ = repository_fixture(Path(directory))
            real_run = subprocess.run

            def run(args, *positional, **kwargs):
                if args[:2] == ["just", "release-verify"]:
                    return subprocess.CompletedProcess(args, 0)
                return real_run(args, *positional, **kwargs)

            with patch.object(subprocess, "run", side_effect=run):
                result = dispatch.run_dispatch(repo, "")
            self.assertEqual(result, 1)

    def test_dispatch_rejects_source_drift_during_release_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, _ = repository_fixture(Path(directory))
            real_run = subprocess.run

            def run(args, *positional, **kwargs):
                if args[:2] == ["just", "release-verify"]:
                    (repo / "gate-created.txt").write_text("unexpected source change")
                    return subprocess.CompletedProcess(args, 0)
                return real_run(args, *positional, **kwargs)

            with patch.object(subprocess, "run", side_effect=run):
                result = dispatch.run_dispatch(repo, "")
            self.assertEqual(result, 1)

    def test_dispatch_rejects_dirty_source_cleaned_by_release_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, _ = repository_fixture(Path(directory))
            dirty_path = repo / "pre-existing-untracked.txt"
            dirty_path.write_text("must not be hidden by the gate\n")
            real_run = subprocess.run
            calls: list[list[str]] = []

            def run(args, *positional, **kwargs):
                if args[:2] == ["just", "release-verify"]:
                    calls.append(args)
                    dirty_path.unlink()
                    return subprocess.CompletedProcess(args, 0)
                return real_run(args, *positional, **kwargs)

            error_output = StringIO()
            with patch.object(subprocess, "run", side_effect=run), redirect_stderr(error_output):
                result = dispatch.run_dispatch(repo, "")
            self.assertEqual(result, 1)
            self.assertEqual(calls, [["just", "release-verify"]])
            self.assertIn("dirty before or after", error_output.getvalue())

    def test_dispatch_checks_only_new_non_default_commits(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo, base = repository_fixture(Path(directory))
            git(repo, "switch", "-c", "release/issue-51")
            write(repo, "README.md", b"linked work\n")
            target = commit(repo, "docs: record work Issue #51")
            refs = f"refs/heads/release/issue-51 {target} refs/heads/release/issue-51 {('0' * 40)}\n"
            real_run = subprocess.run

            def run(args, *positional, **kwargs):
                if args[:2] == ["just", "release-verify"]:
                    return subprocess.CompletedProcess(args, 0)
                return real_run(args, *positional, **kwargs)

            with patch.object(subprocess, "run", side_effect=run):
                result = dispatch.run_dispatch(repo, refs)
            self.assertEqual(result, 0)
            self.assertNotEqual(base, target)


if __name__ == "__main__":
    unittest.main()
