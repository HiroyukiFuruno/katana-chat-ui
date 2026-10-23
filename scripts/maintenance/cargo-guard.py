#!/usr/bin/env python3
"""Serialize KCU Cargo work and fail closed during build-cache maintenance."""

from __future__ import annotations

import argparse
import errno
import json
import os
import stat
import subprocess
import sys
from pathlib import Path

try:
    import fcntl
except ImportError:
    fcntl = None
    import msvcrt


MANIFESTS = (
    "Cargo.toml",
    "tools/host-support/Cargo.toml",
    "tools/e2e-host-app/Cargo.toml",
    "tools/manual-host-egui/Cargo.toml",
    "tools/manual-host-floem/Cargo.toml",
    "tools/manual-host-gpui/Cargo.toml",
    "scripts/screenshot/Cargo.toml",
)
CACHEDIR_TAG = "Signature: 8a477f597d28d172789f06886806bc55"
MAINTENANCE_SUBCOMMANDS = {"clean", "sweep"}


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def lock_path(root: Path) -> Path:
    return root / "tmp" / "maintenance" / "build-cache.lock"


def _path_has_symlink(path: Path, root: Path) -> bool:
    current = root
    try:
        parts = path.relative_to(root).parts
    except ValueError:
        return False
    for part in parts:
        current /= part
        if current.is_symlink():
            return True
    return False


def _git_environment() -> dict[str, str]:
    environment = os.environ.copy()
    for key in tuple(environment):
        if key.startswith("GIT_"):
            environment.pop(key)
    environment["GIT_TERMINAL_PROMPT"] = "0"
    return environment


def _git_output(root: Path, arguments: list[str], runner=subprocess.run) -> str:
    try:
        result = runner(
            ["git", "--literal-pathspecs", "-C", str(root), *arguments],
            cwd=root,
            env=_git_environment(),
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        raise ValueError(f"git inspection unavailable: {error}") from error
    if result.returncode != 0:
        detail = (result.stderr or "").strip()
        suffix = f": {detail}" if detail else ""
        raise ValueError(f"git inspection failed{suffix}")
    return result.stdout


def _validate_git_checkout(root: Path, runner=subprocess.run) -> None:
    output = _git_output(root, ["rev-parse", "--show-toplevel"], runner=runner).strip()
    if not output:
        raise ValueError("git checkout root is empty")
    reported = Path(output).resolve()
    if reported != root:
        raise ValueError(f"git checkout root mismatch: {reported}")


def _validate_git_target_contents(root: Path, target: Path, runner=subprocess.run) -> None:
    relative = target.relative_to(root).as_posix()
    contents = _git_output(
        root,
        ["ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", f"{relative}/"],
        runner=runner,
    )
    if any(contents.split("\0")):
        raise ValueError(f"target contains repository files: {target}")


def validate_targets(
    root: Path, candidates: list[Path], runner=subprocess.run
) -> list[Path]:
    root = root.resolve()
    _validate_git_checkout(root, runner=runner)
    targets: list[Path] = []
    for candidate in candidates:
        candidate = candidate.absolute()
        if _path_has_symlink(candidate, root):
            raise ValueError(f"target path contains symlink: {candidate}")
        path = candidate.resolve()
        try:
            relative = path.relative_to(root)
        except ValueError as error:
            raise ValueError(f"target is outside repository scope: {candidate}") from error
        if not relative.parts or ".git" in relative.parts:
            raise ValueError(f"target overlaps repository metadata: {path}")
        if "src" in relative.parts:
            raise ValueError(f"target overlaps source: {path}")
        if path.exists() and not path.is_dir():
            raise ValueError(f"target is not a directory: {path}")
        if path.exists() and not owned_target(path):
            raise ValueError(f"target ownership tag missing or invalid: {path}")
        if path.exists():
            _validate_git_target_contents(root, path, runner=runner)
        if path not in targets:
            targets.append(path)
    return targets


def target_paths(root: Path, cwd: Path | None = None) -> list[Path]:
    base = (cwd or Path.cwd()).resolve()
    candidates = [root / Path(manifest).parent / "target" for manifest in MANIFESTS]
    explicit = os.environ.get("CARGO_TARGET_DIR")
    if explicit:
        candidates.append(base / explicit)
    return validate_targets(root, candidates)


def owned_target(path: Path) -> bool:
    tag = path / "CACHEDIR.TAG"
    descriptor: int | None = None
    try:
        initial = tag.lstat()
        if not stat.S_ISREG(initial.st_mode):
            return False
        if os.name != "nt":
            nofollow = getattr(os, "O_NOFOLLOW", 0)
            if not nofollow:
                return False
            descriptor = os.open(tag, os.O_RDONLY | os.O_NONBLOCK | nofollow)
        else:
            descriptor = os.open(tag, os.O_RDONLY)
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode):
            return False
        expected = CACHEDIR_TAG.encode("ascii")
        actual = os.read(descriptor, len(expected))
        return actual == expected
    except (OSError, UnicodeError):
        return False
    finally:
        if descriptor is not None:
            os.close(descriptor)


def _is_cargo_executable(value: str) -> bool:
    name = Path(value).name
    return name == "cargo" or name == "cargo.exe" or name.startswith("cargo-")


def _cargo_indexes(command: list[str]) -> tuple[int, int]:
    for index, value in enumerate(command):
        if _is_cargo_executable(value):
            skip_value = False
            for subcommand_index, argument in enumerate(command[index + 1 :], index + 1):
                if argument == "--":
                    break
                if skip_value:
                    skip_value = False
                    continue
                if argument in {"--config", "--color", "-Z"}:
                    skip_value = True
                    continue
                if argument.startswith("-"):
                    continue
                if argument.startswith("+"):
                    continue
                return index, subcommand_index
    raise ValueError("the command does not contain a Cargo executable and subcommand")


def cargo_subcommand(command: list[str]) -> str | None:
    try:
        return command[_cargo_indexes(command)[1]]
    except ValueError:
        return None


def _operation_index(command: list[str]) -> tuple[str, int]:
    for index in range(len(command) - 1, -1, -1):
        if command[index] in MAINTENANCE_SUBCOMMANDS:
            return command[index], index
    raise ValueError("cleanup command does not contain clean or sweep")


def acquire(lock, blocking: bool) -> bool:
    try:
        if fcntl is not None:
            flags = (fcntl.LOCK_EX if not blocking else fcntl.LOCK_SH)
            if not blocking:
                flags |= fcntl.LOCK_NB
            fcntl.flock(lock.fileno(), flags)
        else:
            lock.seek(0, os.SEEK_END)
            if lock.tell() == 0:
                lock.write("0")
                lock.flush()
            lock.seek(0)
            mode = msvcrt.LK_LOCK if blocking else msvcrt.LK_NBLCK
            msvcrt.locking(lock.fileno(), mode, 1)
        return True
    except OSError as error:
        if not blocking and error.errno in {errno.EACCES, errno.EAGAIN, errno.EDEADLK}:
            return False
        raise


def active_open_files(targets: list[Path], runner=subprocess.run) -> bool:
    if os.name == "nt":
        try:
            result = runner(
                ["tasklist", "/fo", "csv", "/nh"], capture_output=True, text=True, check=False
            )
        except OSError:
            return True
        return result.returncode != 0 or any(
            name in result.stdout.lower() for name in ('"cargo.exe"', '"rustc.exe"')
        )
    for target in targets:
        if not target.exists():
            continue
        try:
            result = runner(
                ["lsof", "+D", str(target)], capture_output=True, text=True, check=False
            )
        except OSError:
            return True
        if result.stdout.strip() or result.returncode not in {0, 1} or result.stderr.strip():
            return True
    return False


def metadata_target_paths(
    root: Path,
    command: list[str],
    runner=subprocess.run,
) -> list[Path]:
    _, subcommand_index = _operation_index(command)
    prefix = command[:subcommand_index]
    tail_args = command[subcommand_index + 1 :]
    if any(
        argument == "--manifest-path"
        or argument.startswith("--manifest-path=")
        or argument == "--target-dir"
        or argument.startswith("--target-dir=")
        for argument in tail_args
    ):
        raise ValueError("cleanup does not accept --manifest-path or --target-dir")
    metadata_tail = tail_args if command[subcommand_index] == "clean" else []
    targets: list[Path] = []
    for manifest in MANIFESTS:
        metadata = prefix + [
            "metadata",
            *metadata_tail,
            "--manifest-path",
            str(root / manifest),
            "--no-deps",
            "--offline",
            "--locked",
            "--format-version",
            "1",
        ]
        result = runner(metadata, capture_output=True, text=True, check=False, cwd=root)
        if result.returncode != 0:
            raise RuntimeError(f"cargo metadata failed for {manifest}: exit {result.returncode}")
        try:
            payload = json.loads(result.stdout)
            target = Path(payload["target_directory"])
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            raise RuntimeError(f"cargo metadata returned invalid target for {manifest}") from error
        targets.append(target if target.is_absolute() else (root / target))
    return validate_targets(root, targets)


def run(command: list[str], cleanup: bool, root: Path) -> int:
    path = lock_path(root)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+") as lock:
        if not acquire(lock, blocking=not cleanup):
            print("guarded cargo command is active; refusing cache maintenance", file=sys.stderr)
            return 3
        targets = metadata_target_paths(root, command) if cleanup else []
        if cleanup and active_open_files(targets):
            print("active cargo target files detected; refusing cache maintenance", file=sys.stderr)
            return 3
        result = subprocess.run(command, cwd=root if cleanup else None, check=False)
        if not cleanup or _operation_index(command)[0] != "clean":
            return result.returncode
        if result.returncode != 0:
            return result.returncode
        _, subcommand_index = _operation_index(command)
        prefix = command[:subcommand_index]
        tail_args = command[subcommand_index + 1 :]
        first_failure = result.returncode
        for manifest in MANIFESTS[1:]:
            child = subprocess.run(
                prefix + ["clean"] + tail_args + ["--manifest-path", str(root / manifest)],
                cwd=root,
                check=False,
            )
            if first_failure == 0 and child.returncode != 0:
                first_failure = child.returncode
            if first_failure != 0:
                break
        return first_failure


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("run", "cleanup"))
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("a Cargo command is required after --")
    if args.mode == "cleanup":
        try:
            _operation_index(command)
        except ValueError as error:
            parser.error(str(error))
    try:
        return run(command, args.mode == "cleanup", repo_root())
    except (OSError, RuntimeError, ValueError) as error:
        print(f"build cache guard failed safely: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
