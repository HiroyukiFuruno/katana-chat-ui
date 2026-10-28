#!/usr/bin/env python3
"""Git hook 用の Issue 参照と依存更新証跡の検査。"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Iterable
from urllib.parse import urlparse


class GateError(RuntimeError):
    """リポジトリ変更が hook の契約を満たさない場合のエラー。"""


ISSUE_PATTERN = re.compile(r"(?<![\w./-])(?:issue[\s_/#-]*|#)(\d+)\b", re.IGNORECASE)


def git(repo: Path, *args: str, text: bool = True) -> str | bytes:
    result = git_process(repo, *args, text=text)
    if result.returncode != 0:
        detail = result.stderr.strip() if isinstance(result.stderr, str) else result.stderr.decode(errors="replace").strip()
        raise GateError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout


def git_environment() -> dict[str, str]:
    env = os.environ.copy()
    for name in tuple(env):
        if name in {
            "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR",
            "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_CONFIG", "GIT_CONFIG_COUNT", "GIT_CONFIG_PARAMETERS", "GIT_CEILING_DIRECTORIES",
        } or name.startswith(("GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_")):
            env.pop(name, None)
    return env


def git_process(repo: Path, *args: str, text: bool = True) -> subprocess.CompletedProcess[str] | subprocess.CompletedProcess[bytes]:
    try:
        return subprocess.run(
            ["git", "-C", str(repo), *args],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=text,
            env=git_environment(),
        )
    except OSError as error:
        raise GateError(f"cannot run git {' '.join(args)}: {error}") from error


def issue_numbers(message: str) -> set[int]:
    return {number for match in ISSUE_PATTERN.findall(message) if (number := int(match)) > 0}


def default_branch(repo: Path) -> str:
    symbolic = git_process(repo, "symbolic-ref", "--quiet", "--short", "refs/remotes/origin/HEAD")
    if symbolic.returncode == 0:
        value = symbolic.stdout.strip()
        if value.startswith("origin/") and value.removeprefix("origin/"):
            branch = value.removeprefix("origin/")
            exists = git_process(repo, "show-ref", "--verify", "--quiet", f"refs/remotes/origin/{branch}")
            if exists.returncode == 0:
                return branch
            if exists.returncode != 1:
                raise GateError(f"cannot inspect origin/{branch}: {exists.stderr.strip()}")
        raise GateError(f"origin/HEAD does not identify an existing origin branch: {value!r}")

    if symbolic.returncode != 1:
        raise GateError(f"cannot read origin/HEAD: {symbolic.stderr.strip()}")
    candidates = []
    for name in ("master", "main"):
        exists = git_process(repo, "show-ref", "--verify", "--quiet", f"refs/remotes/origin/{name}")
        if exists.returncode == 0:
            candidates.append(name)
        elif exists.returncode != 1:
            raise GateError(f"cannot inspect origin/{name}: {exists.stderr.strip()}")
    if len(candidates) == 1:
        return candidates[0]
    if not candidates:
        raise GateError("origin/HEAD is unset and neither origin/master nor origin/main exists")
    raise GateError("origin/HEAD is unset and both origin/master and origin/main exist; default is ambiguous")


def check_commit_message(repo: Path, message: str) -> None:
    branch = str(git(repo, "rev-parse", "--abbrev-ref", "HEAD")).strip()
    default = default_branch(repo)
    if branch == default:
        return
    if not issue_numbers(message):
        raise GateError(f"new commit on non-default branch {branch!r} must include a positive Issue reference")


def _show(repo: Path, revision: str, path: str) -> bytes:
    if not re.fullmatch(r"[0-9a-fA-F]{40,64}", revision):
        raise GateError(f"invalid commit object id: {revision!r}")
    result = git_process(repo, "show", f"{revision}:{path}", text=False)
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace") if isinstance(result.stderr, bytes) else result.stderr
        if "does not exist in" in detail or "exists on disk, but not in" in detail:
            raise GateError(f"{path} does not exist at {revision}")
        raise GateError(f"cannot inspect {path} at {revision}: {detail.strip()}")
    return result.stdout


def _show_optional(repo: Path, revision: str, path: str) -> bytes:
    if not re.fullmatch(r"[0-9a-fA-F]{40,64}", revision):
        raise GateError(f"invalid commit object id: {revision!r}")
    result = git_process(repo, "show", f"{revision}:{path}", text=False)
    if result.returncode == 0:
        return result.stdout
    detail = result.stderr.decode("utf-8", errors="replace") if isinstance(result.stderr, bytes) else result.stderr
    if "does not exist in" in detail or "exists on disk, but not in" in detail:
        return b""
    raise GateError(f"cannot inspect optional path {path} at {revision}: {detail.strip()}")


def _name_status(repo: Path, commit: str) -> list[tuple[str, str]]:
    parents = str(git(repo, "rev-list", "--parents", "-n", "1", commit)).split()
    if not parents:
        raise GateError(f"cannot resolve commit {commit}")
    if len(parents) > 1:
        diff_range = [parents[1], commit]
        root_option: list[str] = []
    else:
        diff_range = [commit]
        root_option = ["--root"]
    result = git_process(repo, "diff-tree", *root_option, "--no-renames", "--no-commit-id", "--name-status", "-r", "-z", *diff_range, text=False)
    if result.returncode != 0:
        raise GateError(f"cannot inspect changed paths for {commit}")
    tokens = result.stdout.decode("utf-8", errors="surrogateescape").split("\0")
    tokens = [token for token in tokens if token]
    changes: list[tuple[str, str]] = []
    index = 0
    while index < len(tokens):
        status = tokens[index]
        index += 1
        if index >= len(tokens):
            raise GateError(f"malformed git diff-tree output for {commit}")
        path = tokens[index]
        index += 1
        changes.append((status, path))
    return changes


def changed_paths(repo: Path, commits: Iterable[str]) -> dict[str, set[str]]:
    changed: dict[str, set[str]] = {}
    for commit in commits:
        for status, path in _name_status(repo, commit):
            changed.setdefault(path, set()).add(commit)
    return changed


def _is_supported_manifest(path: str) -> bool:
    return Path(path).name == "Cargo.toml"


def _is_supported_lock(path: str) -> bool:
    return Path(path).name == "Cargo.lock"


def _scope_for_path(path: str, target: str, repo: Path) -> tuple[str, str]:
    parts = Path(path).parts
    if path.startswith("/") or ".." in parts:
        raise GateError(f"unsafe dependency path: {path!r}")
    directories = [Path(*parts[:index]).as_posix() for index in range(len(parts))]
    if Path(path).name == "Cargo.toml":
        for directory in reversed(directories):
            lock = (Path(directory) / "Cargo.lock").as_posix() if directory != "." else "Cargo.lock"
            try:
                _show(repo, target, lock)
                return path, lock
            except GateError:
                continue
        raise GateError(f"Cargo.toml {path} has no ancestor Cargo.lock at pushed commit {target}")
    if Path(path).name == "Cargo.lock":
        for directory in reversed(directories):
            manifest = (Path(directory) / "Cargo.toml").as_posix() if directory != "." else "Cargo.toml"
            try:
                _show(repo, target, manifest)
                return manifest, path
            except GateError:
                continue
        raise GateError(f"Cargo.lock {path} has no ancestor Cargo.toml at pushed commit {target}")
    raise GateError(f"unsupported dependency manifest or lockfile: {path}; this gate currently validates Cargo scopes")


def _cargo_packages(lock_bytes: bytes, path: str) -> dict[tuple[str, str], list[str]]:
    try:
        document = tomllib.loads(lock_bytes.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise GateError(f"invalid Cargo.lock {path}: {error}") from error
    package_entries = document.get("package", [])
    if not isinstance(package_entries, list):
        raise GateError(f"invalid package list in Cargo.lock {path}")
    packages: dict[tuple[str, str], set[str]] = {}
    for index, package in enumerate(package_entries):
        if not isinstance(package, dict):
            raise GateError(f"invalid package entry {index} in Cargo.lock {path}")
        name = package.get("name")
        version = package.get("version")
        if not isinstance(name, str) or not name.strip() or not isinstance(version, str) or not version.strip():
            raise GateError(f"package entry {index} in Cargo.lock {path} requires non-empty name and version")
        source = package.get("source", "local")
        if not isinstance(source, str) or not source.strip():
            raise GateError(f"package {name} in Cargo.lock {path} has an invalid source")
        checksum = package.get("checksum")
        if checksum is not None and (not isinstance(checksum, str) or not checksum.strip()):
            raise GateError(f"package {name} in Cargo.lock {path} has an invalid checksum")
        revision = f"{version}#{checksum}" if isinstance(checksum, str) else version
        packages.setdefault((name, source), set()).add(revision)
    return {identity: sorted(versions) for identity, versions in packages.items()}


def _direct_dependency_names(manifest_bytes: bytes, path: str) -> set[str]:
    try:
        document = tomllib.loads(manifest_bytes.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise GateError(f"invalid Cargo.toml {path}: {error}") from error
    names: set[str] = set()

    def add_table(table: object) -> None:
        if not isinstance(table, dict):
            return
        for declared_name, spec in table.items():
            actual_name = spec.get("package", declared_name) if isinstance(spec, dict) else declared_name
            if isinstance(actual_name, str):
                names.add(actual_name)

    for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
        add_table(document.get(table_name))
    add_table(document.get("workspace", {}).get("dependencies") if isinstance(document.get("workspace"), dict) else None)
    targets = document.get("target")
    if isinstance(targets, dict):
        for target in targets.values():
            if isinstance(target, dict):
                for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
                    add_table(target.get(table_name))
    return names


def _package_changes(
    old: dict[tuple[str, str], list[str]],
    new: dict[tuple[str, str], list[str]],
    direct_names: set[str],
) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    direct: list[dict[str, str]] = []
    transitive: list[dict[str, str]] = []
    for name, source in sorted(set(old) | set(new)):
        before = old.get((name, source), [])
        after = new.get((name, source), [])
        if before == after:
            continue
        change = {
            "name": name,
            "source": source,
            "from": ",".join(before) if before else "absent",
            "to": ",".join(after) if after else "absent",
        }
        (direct if name in direct_names else transitive).append(change)
    return direct, transitive


def _validate_change_list(value: object, expected: list[dict[str, str]], field: str) -> None:
    if not isinstance(value, list):
        raise GateError(f"dependency evidence {field} must be a list")
    normalized: list[dict[str, str]] = []
    for item in value:
        if not isinstance(item, dict) or not all(isinstance(item.get(key), str) and item[key].strip() for key in ("name", "source", "from", "to")):
            raise GateError(f"dependency evidence {field} entries require non-empty name/source/from/to values")
        normalized.append({key: item[key] for key in ("name", "source", "from", "to")})
    if normalized != expected:
        raise GateError(f"dependency evidence {field} does not match the Cargo.lock diff; expected {expected!r}")


def _validate_dependency_evidence(
    repo: Path,
    base: str,
    target: str,
    issue_id: int,
    evidence_path: str,
    scope_paths: set[str],
    changes: dict[str, set[str]],
) -> None:
    paths_to_scopes: dict[str, tuple[str, str]] = {}
    for path in sorted(scope_paths):
        scope = _scope_for_path(path, target, repo)
        previous = paths_to_scopes.get(scope[0])
        if previous is not None and previous != scope:
            raise GateError(f"ambiguous Cargo dependency scope for {path}")
        paths_to_scopes[scope[0]] = scope

    evidence_bytes = _show(repo, target, evidence_path)
    try:
        evidence = json.loads(evidence_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise GateError(f"invalid dependency evidence JSON {evidence_path}: {error}") from error
    if not isinstance(evidence, dict):
        raise GateError("dependency evidence root must be a JSON object")

    upstream = evidence.get("upstream_release")
    upstream_url = upstream.get("url") if isinstance(upstream, dict) else None
    parsed_url = urlparse(upstream_url) if isinstance(upstream_url, str) else None
    upstream_version = upstream.get("version") if isinstance(upstream, dict) else None
    if not isinstance(upstream, dict) or not parsed_url or parsed_url.scheme not in {"https", "http"} or not parsed_url.hostname or not isinstance(upstream_version, str) or not upstream_version.strip() or upstream_version.strip().lower() in {"todo", "tbd", "pending", "unknown", "n/a"} or upstream_version.strip().startswith("<"):
        raise GateError("upstream_release requires a real HTTP(S) url and non-empty version")
    migration = evidence.get("api_migration_note")
    if not isinstance(migration, str) or not migration.strip() or migration.strip().lower() in {"todo", "tbd", "pending", "unknown", "n/a"} or migration.strip().startswith("<"):
        raise GateError("api_migration_note must contain an explicit note (use 'No migration required' when applicable)")
    verification = evidence.get("verification")
    if not isinstance(verification, dict) or verification.get("strict_quality_gate") != "just release-verify":
        raise GateError("verification.strict_quality_gate must name 'just release-verify'")
    commands = verification.get("commands")
    if not isinstance(commands, list) or not commands or any(not isinstance(command, str) or not command.strip() or command.strip().lower() in {"todo", "tbd", "pending"} or command.strip().startswith("<") for command in commands):
        raise GateError("verification.commands must list explicit non-empty verification commands")

    scopes = evidence.get("scopes")
    if not isinstance(scopes, list):
        raise GateError("dependency evidence scopes must be a list")
    evidence_scopes: dict[str, dict[str, object]] = {}
    for entry in scopes:
        if not isinstance(entry, dict) or not isinstance(entry.get("manifest"), str):
            raise GateError("each dependency evidence scope requires a manifest path")
        manifest = entry["manifest"]
        if manifest in evidence_scopes:
            raise GateError(f"duplicate dependency evidence scope: {manifest}")
        evidence_scopes[manifest] = entry
    if set(evidence_scopes) != set(paths_to_scopes):
        raise GateError(f"dependency evidence scopes do not match changed Cargo scopes: expected {sorted(paths_to_scopes)}")

    for manifest, (expected_manifest, lockfile) in paths_to_scopes.items():
        entry = evidence_scopes[manifest]
        if entry.get("lockfile") != lockfile:
            raise GateError(f"dependency evidence maps {manifest} to the wrong lockfile")
        manifest_bytes = _show(repo, target, manifest)
        lock_bytes = _show(repo, target, lockfile)
        if entry.get("manifest_sha256") != hashlib.sha256(manifest_bytes).hexdigest():
            raise GateError(f"manifest_sha256 does not match pushed tree for {manifest}")
        if entry.get("lock_sha256") != hashlib.sha256(lock_bytes).hexdigest():
            raise GateError(f"lock_sha256 does not match pushed tree for {lockfile}")
        changed_scope_files = sorted(path for path in (expected_manifest, lockfile) if path in scope_paths)
        if entry.get("changed_files") != changed_scope_files:
            raise GateError(f"changed_files for {manifest} must equal {changed_scope_files}")

        old_manifest = _show_optional(repo, base, manifest)
        old_lock = _show_optional(repo, base, lockfile)
        direct_names = _direct_dependency_names(old_manifest, manifest) | _direct_dependency_names(manifest_bytes, manifest)
        direct, transitive = _package_changes(
            _cargo_packages(old_lock, lockfile),
            _cargo_packages(lock_bytes, lockfile),
            direct_names,
        )
        if not direct and not transitive and old_manifest == manifest_bytes:
            raise GateError(f"dependency scope {manifest} has neither a package change nor an actual manifest change to document")
        _validate_change_list(entry.get("direct_changes"), direct, "direct_changes")
        _validate_change_list(entry.get("transitive_changes"), transitive, "transitive_changes")


def _declared_evidence_scopes(repo: Path, target: str, evidence_path: str) -> set[tuple[str, str]]:
    evidence_bytes = _show(repo, target, evidence_path)
    try:
        evidence = json.loads(evidence_bytes)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise GateError(f"invalid dependency evidence JSON {evidence_path}: {error}") from error
    if not isinstance(evidence, dict) or not isinstance(evidence.get("scopes"), list):
        raise GateError(f"dependency evidence {evidence_path} must declare a scopes list")
    scopes: set[tuple[str, str]] = set()
    for entry in evidence["scopes"]:
        if not isinstance(entry, dict) or not isinstance(entry.get("manifest"), str) or not isinstance(entry.get("lockfile"), str):
            raise GateError(f"each dependency evidence scope in {evidence_path} requires manifest and lockfile paths")
        scopes.add((entry["manifest"], entry["lockfile"]))
    return scopes


def validate_dependency_updates(repo: Path, base: str, target: str, commits: list[str], messages: dict[str, str]) -> bool:
    changes = changed_paths(repo, commits)
    manifest_paths = {path for path in changes if _is_supported_manifest(path)}
    lock_paths = {path for path in changes if _is_supported_lock(path)}
    evidence_paths = {path for path in changes if path.startswith("issues/") and path.endswith("/dependency-update-evidence.json")}
    unsupported = {
        path for path in changes
        if Path(path).name in {"pyproject.toml", "package.json", "poetry.lock", "uv.lock", "pdm.lock", "Pipfile.lock", "package-lock.json", "pnpm-lock.yaml", "yarn.lock"}
    }
    if unsupported:
        raise GateError(f"unsupported dependency manifest/lock changes: {', '.join(sorted(unsupported))}; only Cargo scopes are currently validated")
    if not manifest_paths and not lock_paths:
        if evidence_paths:
            raise GateError("dependency evidence must change in the same range as its Cargo manifest or lockfile")
        return False

    groups: dict[int, set[str]] = {}
    evidence_scope_pairs = {
        evidence_path: _declared_evidence_scopes(repo, target, evidence_path)
        for evidence_path in sorted(evidence_paths)
    }
    for path in sorted(manifest_paths | lock_paths):
        path_issues: set[int] = set()
        for commit in changes[path]:
            path_issues.update(issue_numbers(messages.get(commit, "")))
        if not path_issues:
            raise GateError(f"dependency path {path} is not linked to any positive Issue number in the pushed range")
        expected_scope = _scope_for_path(path, target, repo)
        owners: set[int] = set()
        for evidence_path, declared_scopes in evidence_scope_pairs.items():
            match = re.fullmatch(r"issues/([1-9][0-9]*)/dependency-update-evidence\.json", evidence_path)
            if not match:
                continue
            issue_id = int(match.group(1))
            evidence_commits = changes[evidence_path]
            evidence_references_issue = any(issue_id in issue_numbers(messages.get(commit, "")) for commit in evidence_commits)
            if issue_id in path_issues and evidence_references_issue and expected_scope in declared_scopes:
                owners.add(issue_id)
        if not owners:
            raise GateError(f"dependency path {path} has no uniquely referenced dependency-update-evidence.json owner")
        if len(owners) > 1:
            raise GateError(f"ambiguous dependency evidence owner for {path}: {sorted(owners)}")
        groups.setdefault(next(iter(owners)), set()).add(path)

    expected_evidence_paths = {f"issues/{issue_id}/dependency-update-evidence.json" for issue_id in groups}
    extra_evidence = evidence_paths - expected_evidence_paths
    if extra_evidence:
        raise GateError(f"unpaired dependency evidence files changed: {', '.join(sorted(extra_evidence))}")
    for issue_id, scope_paths in sorted(groups.items()):
        evidence_path = f"issues/{issue_id}/dependency-update-evidence.json"
        evidence_commits = changes.get(evidence_path, set())
        if not evidence_commits or not any(issue_id in issue_numbers(messages.get(commit, "")) for commit in evidence_commits):
            raise GateError(f"dependency changes require {evidence_path} to change in the same pushed commit range")
        _validate_dependency_evidence(repo, base, target, issue_id, evidence_path, scope_paths, changes)
    return True


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    commit_parser = subparsers.add_parser("commit-msg", help="require an Issue reference on non-default branch commits")
    commit_parser.add_argument("message_file", type=Path)
    commit_parser.add_argument("--repo", type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    try:
        message = args.message_file.read_text(encoding="utf-8", errors="replace")
        check_commit_message(args.repo.resolve(), message)
    except (OSError, GateError) as error:
        print(f"issue gate: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
