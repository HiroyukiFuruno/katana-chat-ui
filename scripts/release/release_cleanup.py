#!/usr/bin/env python3
"""公開済み release 後に明示された branch だけを安全に整理する。"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlparse


class CleanupError(RuntimeError):
    """cleanup 前提を満たさないか、対象 branch を保持した場合のエラー。"""


NO_UPSTREAM_REMOTE = "__kcu_cleanup_no_upstream__"


def run(args: list[str], repo: Path, *, capture: bool = True) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    for name in tuple(env):
        if name in {
            "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR",
            "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_CONFIG", "GIT_CONFIG_COUNT", "GIT_CONFIG_PARAMETERS", "GIT_CEILING_DIRECTORIES",
        } or name.startswith(("GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_")):
            env.pop(name, None)
    try:
        return subprocess.run(
            args,
            cwd=repo,
            check=False,
            stdout=subprocess.PIPE if capture else None,
            stderr=subprocess.PIPE if capture else None,
            text=True,
            env=env,
        )
    except OSError as error:
        raise CleanupError(f"cannot run {' '.join(args)}: {error}") from error


def git(repo: Path, *args: str) -> str:
    result = run(["git", *args], repo)
    if result.returncode != 0:
        detail = (result.stderr or "").strip()
        raise CleanupError(f"git {' '.join(args)} failed: {detail}")
    return result.stdout or ""


def origin_repo(repo: Path) -> str:
    url = git(repo, "config", "--get", "remote.origin.url").strip()
    if not url:
        raise CleanupError("remote.origin.url is not configured")
    if re.match(r"^[^/@:]+@[^/:]+:.+", url):
        host = url.split("@", 1)[1].split(":", 1)[0]
        path = url.split(":", 1)[1]
    else:
        parsed = urlparse(url)
        if parsed.scheme not in {"https", "http", "ssh", "git"} or not parsed.hostname:
            raise CleanupError(f"origin URL is not a GitHub URL: {url!r}")
        host = parsed.hostname
        path = parsed.path.lstrip("/")
    if host.lower() not in {"github.com", "ssh.github.com"}:
        raise CleanupError(f"origin host is not GitHub: {host}")
    path = path.removesuffix(".git").strip("/")
    parts = path.split("/")
    if len(parts) != 2 or not all(parts):
        raise CleanupError(f"origin URL must identify exactly OWNER/REPOSITORY: {url!r}")
    canonical_host = "github.com" if host.lower() == "ssh.github.com" else host.lower()
    return f"{canonical_host}/" + "/".join(parts)


def require_release_published(repo: Path, tag: str) -> None:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", tag):
        raise CleanupError("--release-tag must be a non-empty tag name without option or path syntax")
    repository = origin_repo(repo)
    result = run(["gh", "release", "view", tag, "--repo", repository, "--json", "isDraft", "--jq", ".isDraft"], repo)
    if result.returncode != 0:
        raise CleanupError(f"GitHub Release {tag} was not found on {repository}, or release lookup failed")
    if (result.stdout or "").strip().lower() != "false":
        raise CleanupError(f"GitHub Release {tag} on {repository} is still a draft or returned an invalid state")


def default_branch(repo: Path) -> str:
    symbolic = run(["git", "symbolic-ref", "--quiet", "--short", "refs/remotes/origin/HEAD"], repo)
    if symbolic.returncode == 0:
        value = (symbolic.stdout or "").strip()
        if value.startswith("origin/") and value.removeprefix("origin/"):
            branch = value.removeprefix("origin/")
            exists = run(["git", "show-ref", "--verify", "--quiet", f"refs/remotes/origin/{branch}"], repo)
            if exists.returncode == 0:
                return branch
            if exists.returncode != 1:
                raise CleanupError(f"cannot inspect origin/{branch}: {(exists.stderr or '').strip()}")
        raise CleanupError(f"origin/HEAD does not identify an existing origin branch: {value!r}")

    if symbolic.returncode != 1:
        raise CleanupError(f"cannot read origin/HEAD: {(symbolic.stderr or '').strip()}")
    candidates = []
    for branch in ("master", "main"):
        exists = run(["git", "show-ref", "--verify", "--quiet", f"refs/remotes/origin/{branch}"], repo)
        if exists.returncode == 0:
            candidates.append(branch)
        elif exists.returncode != 1:
            raise CleanupError(f"cannot inspect origin/{branch}: {(exists.stderr or '').strip()}")
    if len(candidates) == 1:
        return candidates[0]
    if not candidates:
        raise CleanupError("origin/HEAD is unset and neither origin/master nor origin/main exists")
    raise CleanupError("origin/HEAD is unset and both origin/master and origin/main exist; refusing ambiguous default")


def require_clean_worktree(repo: Path) -> None:
    status = git(repo, "status", "--porcelain=v2", "--untracked-files=all")
    if status:
        raise CleanupError("working tree has staged, modified, or untracked files; no branch switch or deletion was attempted")


def used_by_worktree(repo: Path, branch: str) -> bool:
    wanted = f"branch refs/heads/{branch}"
    return wanted in git(repo, "worktree", "list", "--porcelain").splitlines()


def validate_branch_name(repo: Path, branch: str) -> None:
    if not branch or branch.startswith("-"):
        raise CleanupError(f"invalid branch name: {branch!r}")
    result = run(["git", "check-ref-format", f"refs/heads/{branch}"], repo)
    if result.returncode == 1:
        raise CleanupError(f"invalid branch name: {branch!r}")
    if result.returncode != 0:
        raise CleanupError(f"cannot validate branch name {branch!r}: {(result.stderr or '').strip()}")


def local_branch_exists(repo: Path, branch: str) -> bool:
    result = run(["git", "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"], repo)
    if result.returncode not in {0, 1}:
        raise CleanupError(f"cannot inspect local branch {branch}: {(result.stderr or '').strip()}")
    return result.returncode == 0


def remote_tip(repo: Path, branch: str) -> str | None:
    ref = f"refs/heads/{branch}"
    result = run(["git", "ls-remote", "--heads", "origin", ref], repo)
    if result.returncode != 0:
        raise CleanupError(f"cannot read live origin tip for {branch}")
    rows = [row.split() for row in (result.stdout or "").splitlines() if row.strip()]
    matching = [row[0] for row in rows if len(row) == 2 and row[1] == ref]
    if not matching:
        return None
    if len(matching) != 1:
        raise CleanupError(f"origin returned an ambiguous live tip for {branch}")
    return matching[0]


def _is_ancestor(repo: Path, commit: str, base: str) -> bool:
    result = run(["git", "merge-base", "--is-ancestor", commit, base], repo)
    if result.returncode == 1:
        return False
    if result.returncode != 0:
        raise CleanupError(f"cannot check ancestry for {commit}: {(result.stderr or '').strip()}")
    return True


def delete_remote_at_tip(repo: Path, branch: str, expected_tip: str) -> None:
    validate_branch_name(repo, branch)
    if not re.fullmatch(r"[0-9a-fA-F]{40,64}", expected_tip):
        raise CleanupError(f"invalid expected remote tip for {branch}: {expected_tip!r}")
    ref = f"refs/heads/{branch}"
    result = run(["git", "push", f"--force-with-lease={ref}:{expected_tip}", "origin", f":{ref}"], repo)
    if result.returncode != 0:
        raise CleanupError(f"remote branch {branch} changed or could not be deleted with expected tip {expected_tip}: {(result.stderr or '').strip()}")


def delete_local_merged_to_default(repo: Path, default: str, branch: str) -> None:
    current = git(repo, "branch", "--show-current").strip()
    if current != default:
        raise CleanupError(f"default branch {default} must be checked out before deleting local branches")
    remotes = set(git(repo, "remote").splitlines())
    if NO_UPSTREAM_REMOTE in remotes:
        raise CleanupError(f"remote name {NO_UPSTREAM_REMOTE} is reserved for safe local branch deletion")
    local_tip = git(repo, "rev-parse", f"refs/heads/{branch}").strip()
    default_tip = git(repo, "rev-parse", "HEAD").strip()
    if not _is_ancestor(repo, local_tip, default_tip):
        raise CleanupError(f"local branch {branch} is not merged to checked out default {default}")
    result = run([
        "git", "-c", f"branch.{branch}.remote={NO_UPSTREAM_REMOTE}",
        "branch", "-d", "--", branch,
    ], repo)
    if result.returncode != 0:
        raise CleanupError(f"git branch -d retained {branch}: {(result.stderr or '').strip()}")


def cleanup_target(repo: Path, base: str, branch: str, delete_remote: bool) -> bool:
    validate_branch_name(repo, branch)
    current = git(repo, "branch", "--show-current").strip()
    if branch == base:
        print(f"retain default branch: {branch}", file=sys.stderr)
        return False
    if branch == current:
        print(f"retain current branch: {branch}", file=sys.stderr)
        return False
    if used_by_worktree(repo, branch):
        print(f"retain branch used by a worktree: {branch}", file=sys.stderr)
        return False

    local_exists = local_branch_exists(repo, branch)
    local_tip = None
    if local_exists:
        local_tip = git(repo, "rev-parse", f"refs/heads/{branch}").strip()
        base_tip = git(repo, "rev-parse", f"refs/remotes/origin/{base}").strip()
        if not _is_ancestor(repo, local_tip, base_tip):
            print(f"retain unmerged local branch: {branch}", file=sys.stderr)
            return False
    live = None
    if delete_remote:
        live = remote_tip(repo, branch)
        if live is None:
            print(f"remote branch already absent: {branch}")
        else:
            tracking_result = run(["git", "rev-parse", "--verify", f"refs/remotes/origin/{branch}"], repo)
            if tracking_result.returncode == 128:
                raise CleanupError(f"cannot read fetched tracking ref for {branch}: {(tracking_result.stderr or '').strip()}")
            if tracking_result.returncode != 0:
                print(f"retain remote branch without a fetched tracking ref: {branch}", file=sys.stderr)
                return False
            tracking = (tracking_result.stdout or "").strip()
            if tracking != live:
                print(f"retain remote branch whose live tip changed after fetch: {branch}", file=sys.stderr)
                return False
            base_tip = git(repo, "rev-parse", f"refs/remotes/origin/{base}").strip()
            if not _is_ancestor(repo, live, base_tip):
                print(f"retain unmerged remote branch: {branch}", file=sys.stderr)
                return False
            if remote_tip(repo, branch) != live:
                print(f"retain remote branch whose tip changed before deletion: {branch}", file=sys.stderr)
                return False
    elif not local_exists:
        print(f"no local branch to delete; remote deletion was not requested: {branch}")
        return True

    if delete_remote and live is not None:
        try:
            delete_remote_at_tip(repo, branch, live)
        except CleanupError as error:
            print(f"retain remote branch {branch}: {error}", file=sys.stderr)
            return False
        print(f"deleted remote branch: {branch}")

    if local_exists:
        try:
            delete_local_merged_to_default(repo, base, branch)
        except CleanupError as error:
            print(f"retain local branch {branch}: {error}", file=sys.stderr)
            return False
        print(f"deleted local branch: {branch}")
    return True


def cleanup_repository(repo: Path, branches: list[str], delete_remote: bool, *, sync_only: bool = False) -> int:
    if sync_only and (branches or delete_remote):
        raise CleanupError("sync-only cannot be combined with branch cleanup or remote deletion")
    if not sync_only and not branches:
        raise CleanupError("at least one branch or sync-only mode is required")
    require_clean_worktree(repo)
    fetched = run(["git", "fetch", "origin", "--prune"], repo)
    if fetched.returncode != 0:
        raise CleanupError(f"git fetch origin --prune failed: {(fetched.stderr or '').strip()}")
    base = default_branch(repo)
    switched = run(["git", "switch", base], repo)
    if switched.returncode != 0:
        raise CleanupError(f"cannot switch to default branch {base}: {(switched.stderr or '').strip()}")
    pulled = run(["git", "pull", "--ff-only", "origin", base], repo)
    if pulled.returncode != 0:
        raise CleanupError(f"git pull --ff-only origin {base} failed: {(pulled.stderr or '').strip()}")
    require_clean_worktree(repo)

    failures = []
    for branch in ([] if sync_only else branches):
        try:
            if not cleanup_target(repo, base, branch, delete_remote):
                failures.append(branch)
        except CleanupError as error:
            print(f"retain branch {branch}: {error}", file=sys.stderr)
            failures.append(branch)
    pruned = run(["git", "worktree", "prune"], repo)
    if pruned.returncode != 0:
        raise CleanupError(f"git worktree prune failed: {(pruned.stderr or '').strip()}")
    if failures:
        raise CleanupError(f"one or more explicit branches were retained: {', '.join(failures)}")
    if sync_only:
        print(f"release cleanup synchronized on {base}; no branches were deleted")
    else:
        print(f"release cleanup completed on {base}")
    return 0


def perform_cleanup(repo: Path, release_tag: str, branches: list[str], delete_remote: bool, *, sync_only: bool = False) -> int:
    require_release_published(repo, release_tag)
    return cleanup_repository(repo, branches, delete_remote, sync_only=sync_only)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release-tag", required=True)
    target_group = parser.add_mutually_exclusive_group(required=True)
    target_group.add_argument("--branch", action="append", help="explicit branch eligible for cleanup; may be repeated")
    target_group.add_argument("--sync-only", action="store_true", help="run release guard and repository sync without deleting branches")
    parser.add_argument("--delete-remote", action="store_true", help="also delete the matching origin branch after live-tip checks")
    args = parser.parse_args(argv)
    if args.sync_only and args.delete_remote:
        parser.error("--delete-remote cannot be used with --sync-only")
    try:
        return perform_cleanup(Path.cwd().resolve(), args.release_tag, args.branch or [], args.delete_remote, sync_only=args.sync_only)
    except CleanupError as error:
        print(f"release cleanup: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
