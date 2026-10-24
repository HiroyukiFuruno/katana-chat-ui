#!/usr/bin/env python3
"""Verify release-source fixture setup is isolated from inherited Git routing."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
VERIFICATION_DIR = ROOT / "tmp" / "verification"
TEST_SCRIPT = ROOT / "scripts" / "release" / "test-verify-release-source.sh"
GUARD_SCRIPT = ROOT / "scripts" / "release" / "verify-release-source.sh"
VERSION_SCRIPT = ROOT / "scripts" / "release" / "verify-version.sh"


def clean_git_env() -> dict[str, str]:
    return {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}


def run_git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(repo), "--no-optional-locks", *args],
        check=True,
        env=clean_git_env(),
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_fixture_scripts(release_dir: Path) -> None:
    release_dir.mkdir(parents=True)
    shutil.copy2(TEST_SCRIPT, release_dir / TEST_SCRIPT.name)
    shutil.copy2(GUARD_SCRIPT, release_dir / GUARD_SCRIPT.name)
    shutil.copy2(VERSION_SCRIPT, release_dir / VERSION_SCRIPT.name)


def create_foreign_repo(sandbox: Path) -> Path:
    foreign = sandbox / "foreign-git"
    foreign.mkdir(parents=True)
    setup_env = clean_git_env()
    subprocess.run(["git", "-C", str(foreign), "init", "-q"], check=True, env=setup_env)
    foreign_root = Path(run_git(foreign, "rev-parse", "--show-toplevel")).resolve()
    if foreign_root != foreign.resolve():
        raise AssertionError(f"foreign fixture root mismatch: {foreign_root}")
    run_git(foreign, "config", "user.email", "foreign@example.invalid")
    run_git(foreign, "config", "user.name", "foreign-fixture")
    (foreign / "Cargo.toml").write_text(
        '[package]\nname = "foreign-fixture"\nversion = "0.1.0"\nedition = "2021"\n',
        encoding="utf-8",
    )
    run_git(foreign, "add", "Cargo.toml")
    run_git(foreign, "commit", "-q", "-m", "foreign-initial")
    run_git(foreign, "tag", "-a", "v0.1.0", "-m", "release")
    return foreign


def foreign_snapshot(foreign: Path) -> dict[str, str]:
    # statusを先に確定し、indexのoptional refreshがsnapshotを変えないようにする。
    status = run_git(foreign, "status", "--porcelain")
    return {
        "head": run_git(foreign, "rev-parse", "HEAD"),
        "index": digest(foreign / ".git" / "index"),
        "config": digest(foreign / ".git" / "config"),
        "status": status,
    }


def routed_env(sandbox: Path, foreign: Path) -> dict[str, str]:
    env = clean_git_env()
    env.update(
        {
            "GIT_DIR": str(foreign / ".git"),
            "GIT_WORK_TREE": str(foreign),
            "GIT_INDEX_FILE": str(foreign / ".git" / "index"),
            "GIT_CONFIG_GLOBAL": str(sandbox / "empty-global"),
            "GIT_CONFIG_SYSTEM": str(sandbox / "empty-system"),
            "GIT_CONFIG_COUNT": "1",
            "GIT_CONFIG_KEY_0": "core.bare",
            "GIT_CONFIG_VALUE_0": "true",
        }
    )
    return env


def main() -> None:
    VERIFICATION_DIR.mkdir(parents=True, exist_ok=True)
    sandbox = Path(tempfile.mkdtemp(prefix="release-source-fixture-routing-", dir=VERIFICATION_DIR))
    foreign = create_foreign_repo(sandbox)
    release_dir = sandbox / "scripts" / "release"
    write_fixture_scripts(release_dir)
    before = foreign_snapshot(foreign)
    result = subprocess.run(
        ["bash", str(release_dir / TEST_SCRIPT.name)],
        cwd=sandbox,
        env=routed_env(sandbox, foreign),
        capture_output=True,
        text=True,
    )
    after = foreign_snapshot(foreign)
    evidence = {
        "exit": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "before": before,
        "after": after,
        "foreign_repo": str(foreign),
        "sandbox": str(sandbox),
    }
    evidence_path = sandbox / "routing-evidence.json"
    evidence_path.write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if result.returncode != 0:
        raise AssertionError(f"fixture regression failed; evidence={evidence_path}")
    if before != after:
        raise AssertionError(f"sanitized fixture changed foreign Git state; evidence={evidence_path}")
    print(f"release source fixture routing regression passed: {sandbox}")


if __name__ == "__main__":
    main()
