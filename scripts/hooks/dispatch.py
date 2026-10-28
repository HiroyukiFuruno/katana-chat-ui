#!/usr/bin/env python3
"""release-verify を先行し、push 対象の新規 commit だけを検査する。"""

from __future__ import annotations

import hashlib
import os
import re
import stat
import subprocess
import sys
from pathlib import Path

from issue_gate import GateError, default_branch, git, git_environment, git_process, issue_numbers, validate_dependency_updates


def _is_zero_sha(value: str) -> bool:
    return bool(value) and set(value) == {"0"}


def _file_digest(path: Path) -> str:
    metadata = path.lstat()
    digest = hashlib.sha256(str(stat.S_IMODE(metadata.st_mode)).encode())
    if path.is_symlink():
        payload = b"symlink\0" + str(path.readlink()).encode()
    else:
        if not stat.S_ISREG(metadata.st_mode):
            raise GateError(f"cannot snapshot non-regular untracked path {path}")
        payload = path.read_bytes()
    digest.update(payload)
    return digest.hexdigest()


def source_snapshot(repo: Path) -> tuple[str, str, bool]:
    head = str(git(repo, "rev-parse", "HEAD")).strip()
    status = git(repo, "status", "--porcelain=v2", "--untracked-files=all", "-z", text=False)
    staged = git(repo, "diff", "--cached", "--binary", "HEAD", text=False)
    unstaged = git(repo, "diff", "--binary", text=False)
    untracked = git(repo, "ls-files", "--others", "--exclude-standard", "-z", text=False)
    assert isinstance(status, bytes) and isinstance(staged, bytes) and isinstance(unstaged, bytes) and isinstance(untracked, bytes)
    digest = hashlib.sha256()
    digest.update(status)
    digest.update(b"\0staged\0")
    digest.update(staged)
    digest.update(b"\0unstaged\0")
    digest.update(unstaged)
    for raw_path in sorted(item for item in untracked.split(b"\0") if item):
        relative = raw_path.decode("utf-8", errors="surrogateescape")
        digest.update(b"\0untracked\0")
        digest.update(raw_path)
        full_path = repo / relative
        try:
            digest.update(_file_digest(full_path).encode())
        except (OSError, IsADirectoryError) as error:
            raise GateError(f"cannot snapshot untracked path {relative}: {error}") from error
    return head, digest.hexdigest(), bool(status)


def release_gate_environment(repo: Path) -> tuple[dict[str, str], str | None]:
    env = git_environment()
    for name in (
        "JUSTFILE", "JUST_WORKING_DIRECTORY", "CARGO", "JOBS", "RUSTFLAGS", "RTK",
        "VERSION", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET",
        "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTUP_TOOLCHAIN",
    ):
        env.pop(name, None)
    try:
        justfile = (repo / "Justfile").read_text(encoding="utf-8")
    except OSError as error:
        env.pop("COVERAGE_MIN_LINES", None)
        return env, f"Justfile の COVERAGE_MIN_LINES 既定値を読めません: {error}"
    match = re.search(r'(?m)^\s*COVERAGE_MIN_LINES\s*:=\s*env_var_or_default\("COVERAGE_MIN_LINES",\s*"(\d+)"\)', justfile)
    if not match:
        env.pop("COVERAGE_MIN_LINES", None)
        return env, "Justfile の COVERAGE_MIN_LINES 既定値を特定できません"
    baseline = int(match.group(1))
    requested = os.environ.get("COVERAGE_MIN_LINES")
    if requested is None:
        env.pop("COVERAGE_MIN_LINES", None)
        return env, None
    if not requested.isdecimal():
        env["COVERAGE_MIN_LINES"] = str(baseline)
        return env, "COVERAGE_MIN_LINES は整数で指定してください"
    value = int(requested)
    if value < baseline:
        env["COVERAGE_MIN_LINES"] = str(baseline)
        return env, f"COVERAGE_MIN_LINES={value} は Justfile 既定値 {baseline} 未満です"
    env["COVERAGE_MIN_LINES"] = requested
    return env, None


def _push_updates(lines: list[str]) -> list[tuple[str, str, str, str]]:
    updates = []
    for line in lines:
        fields = line.split()
        if not fields:
            continue
        if len(fields) != 4:
            raise GateError(f"malformed pre-push ref line: {line!r}")
        local_ref, local_sha, remote_ref, remote_sha = fields
        local_sha = local_sha.lower()
        remote_sha = remote_sha.lower()
        for value in (local_sha, remote_sha):
            if len(value) not in {40, 64} or any(character not in "0123456789abcdef" for character in value):
                raise GateError(f"malformed object id in pre-push ref line: {line!r}")
        updates.append((local_ref, local_sha, remote_ref, remote_sha))
    return updates


def _new_commits(repo: Path, old_sha: str, new_sha: str) -> tuple[str, list[str]]:
    if _is_zero_sha(new_sha):
        return old_sha, []
    if not _is_zero_sha(old_sha):
        ancestor = git_process(repo, "merge-base", "--is-ancestor", old_sha, new_sha)
        if ancestor.returncode == 1:
            raise GateError(f"non-fast-forward update refused: {old_sha} is not an ancestor of {new_sha}")
        if ancestor.returncode != 0:
            raise GateError(f"cannot check push ancestry: {ancestor.stderr.strip()}")
        base = old_sha
    else:
        branch = default_branch(repo)
        base = str(git(repo, "merge-base", f"refs/remotes/origin/{branch}", new_sha)).strip()
        if not base:
            raise GateError(f"cannot bound new branch commits against origin/{branch}")
    commits = str(git(repo, "rev-list", "--reverse", f"{base}..{new_sha}")).splitlines()
    return base, commits


def run_dispatch(repo: Path, push_text: str) -> int:
    repo = repo.resolve()
    try:
        before = source_snapshot(repo)
    except GateError as error:
        print(f"pre-push dispatcher: {error}", file=sys.stderr)
        return 1
    environment, coverage_error = release_gate_environment(repo)
    try:
        gate = subprocess.run(["just", "release-verify"], cwd=repo, env=environment, check=False)
    except OSError as error:
        print(f"pre-push dispatcher: cannot run just release-verify: {error}", file=sys.stderr)
        return 127
    if gate.returncode != 0:
        print(f"pre-push dispatcher: just release-verify failed with exit {gate.returncode}; later checks skipped", file=sys.stderr)
        return gate.returncode

    try:
        if coverage_error:
            raise GateError(coverage_error)
        after = source_snapshot(repo)
        if before[2] or after[2]:
            raise GateError("push is refused because the source worktree was dirty before or after release-verify")
        if before != after:
            raise GateError("HEAD or source state changed while just release-verify was running")
        head = after[0]
        updates = _push_updates(push_text.splitlines())
        if not updates:
            raise GateError("pre-push supplied no ref updates; refusing to skip push validation")
        default = default_branch(repo)
        for _local_ref, local_sha, remote_ref, remote_sha in updates:
            if not remote_ref.startswith("refs/heads/") or _is_zero_sha(local_sha):
                continue
            if local_sha != head:
                raise GateError(f"release verification ran for HEAD {head}, but pushed ref {remote_ref} points to {local_sha}")
            branch = remote_ref.removeprefix("refs/heads/")
            base, commits = _new_commits(repo, remote_sha, local_sha)
            if not commits:
                continue
            messages: dict[str, str] = {}
            for commit in commits:
                message = str(git(repo, "show", "-s", "--format=%B", commit))
                messages[commit] = message
                found = issue_numbers(message)
                if branch != default and not found:
                    raise GateError(f"new commit {commit} on non-default branch {branch!r} has no positive Issue reference")
            validate_dependency_updates(repo, base, local_sha, commits, messages)
        if source_snapshot(repo) != after:
            raise GateError("source changed during pre-push validation")
    except GateError as error:
        print(f"pre-push dispatcher: {error}", file=sys.stderr)
        return 1
    return 0


def main() -> int:
    try:
        return run_dispatch(Path.cwd(), sys.stdin.read())
    except GateError as error:
        print(f"pre-push dispatcher: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
