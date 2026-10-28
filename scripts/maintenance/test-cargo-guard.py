#!/usr/bin/env python3
"""Regression tests for the KCU Cargo/cache serialization guard."""

from __future__ import annotations

import importlib.util
import os
import stat
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EXPECTED_CACHEDIR_TAG = "Signature: 8a477f597d28d172789f06886806bc55"
MODULE_SPEC = importlib.util.spec_from_file_location("kcu_cargo_guard", ROOT / "scripts/maintenance/cargo-guard.py")
assert MODULE_SPEC and MODULE_SPEC.loader
GUARD = importlib.util.module_from_spec(MODULE_SPEC)
MODULE_SPEC.loader.exec_module(GUARD)


class CargoGuardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.fixture_root = tempfile.TemporaryDirectory(prefix="kcu-build-cache-guard-")
        cls.addClassCleanup(cls.fixture_root.cleanup)
        cls.fixture = Path(cls.fixture_root.name).resolve()
        (cls.fixture / ".gitignore").write_text(
            "target/\n**/target/\nCACHEDIR.TAG\n", encoding="utf-8"
        )
        cls.git_env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        cls.git_env["GIT_TERMINAL_PROMPT"] = "0"
        subprocess.run(["git", "--literal-pathspecs", "init", "-q"], cwd=cls.fixture, env=cls.git_env, check=True)
        checkout = subprocess.run(
            ["git", "--literal-pathspecs", "-C", str(cls.fixture), "rev-parse", "--show-toplevel"],
            cwd=cls.fixture,
            env=cls.git_env,
            check=True,
            capture_output=True,
            text=True,
        )
        assert Path(checkout.stdout.strip()).resolve() == cls.fixture.resolve()
        for manifest in GUARD.MANIFESTS:
            target = cls.fixture / Path(manifest).parent / "target"
            target.mkdir(parents=True, exist_ok=True)
            (target / "CACHEDIR.TAG").write_text(EXPECTED_CACHEDIR_TAG + "\n", encoding="utf-8")
            (cls.fixture / manifest).write_text("[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n", encoding="utf-8")
        cls.bin = cls.fixture / "bin"
        cls.bin.mkdir(parents=True, exist_ok=True)
        cls.cargo = cls.bin / "cargo"
        cls.cargo.write_text(
            "#!/bin/sh\n"
            "case \"$1\" in metadata) printf '{\"target_directory\":\"%s\"}\\n' \"$FIXTURE_TARGET\";; "
            "hold) echo ready > \"$READY_FILE\"; while [ ! -f \"$RELEASE_FILE\" ]; do sleep 0.05; done;; "
            "parallel) echo $$ >> \"$PARALLEL_READY_LOG\"; while [ ! -f \"$PARALLEL_RELEASE_FILE\" ]; do sleep 0.05; done;; "
            "clean) exit 7;; *) exit 0;; esac\n",
            encoding="utf-8",
        )
        cls.cargo.chmod(cls.cargo.stat().st_mode | stat.S_IXUSR)
        cls.child_failure_cargo = cls.fixture / "child-failure-bin" / "my-cargo"
        cls.child_failure_cargo.parent.mkdir(parents=True, exist_ok=True)
        cls.child_failure_cargo.write_text(
            "#!/bin/sh\n"
            "case \"$1\" in metadata) printf '{\"target_directory\":\"%s\"}\\n' \"$FIXTURE_TARGET\";; "
            "clean) case \"$*\" in *host-support*) exit 9;; *) exit 0;; esac;; *) exit 0;; esac\n",
            encoding="utf-8",
        )
        cls.child_failure_cargo.chmod(cls.child_failure_cargo.stat().st_mode | stat.S_IXUSR)
        cls.proxy = cls.bin / "proxy"
        cls.proxy.write_text(
            "#!/bin/sh\n"
            "echo \"$*\" >> \"$PREFIX_LOG\"\n"
            "exec \"$@\"\n",
            encoding="utf-8",
        )
        cls.proxy.chmod(cls.proxy.stat().st_mode | stat.S_IXUSR)
        cls.env = os.environ.copy()
        cls.env["FIXTURE_TARGET"] = str(cls.fixture / "target")
        cls.parallel_log = cls.fixture / "parallel.log"
        cls.parallel_release = cls.fixture / "parallel-release"
        cls.env["PARALLEL_READY_LOG"] = str(cls.parallel_log)
        cls.env["PARALLEL_RELEASE_FILE"] = str(cls.parallel_release)
        cls.prefix_log = cls.fixture / "prefix.log"
        cls.env["PREFIX_LOG"] = str(cls.prefix_log)
    def setUp(self) -> None:
        self.env = self.__class__.env.copy()
        self.ready_file = self.fixture / f"ready-{self._testMethodName}"
        self.release_file = self.fixture / f"release-{self._testMethodName}"
        self.env["READY_FILE"] = str(self.ready_file)
        self.env["RELEASE_FILE"] = str(self.release_file)
        os.environ.update({key: self.env[key] for key in ("FIXTURE_TARGET", "PARALLEL_READY_LOG", "PARALLEL_RELEASE_FILE", "READY_FILE", "RELEASE_FILE", "PREFIX_LOG")})

    def test_run_serializes_with_cleanup_and_preserves_exit(self) -> None:
        command = [str(self.cargo), "hold"]
        child = (
            "import importlib.util, pathlib, sys; "
            "s=importlib.util.spec_from_file_location('guard', sys.argv[3]); "
            "m=importlib.util.module_from_spec(s); s.loader.exec_module(m); "
            "raise SystemExit(m.run([sys.argv[2], 'hold'], False, pathlib.Path(sys.argv[1])))"
        )
        process = subprocess.Popen(
            [sys.executable, "-c", child, str(self.fixture), str(self.cargo), str(ROOT / "scripts/maintenance/cargo-guard.py")],
            cwd=self.fixture,
            env=self.env,
        )
        deadline = time.time() + 2
        while time.time() < deadline and not self.ready_file.exists():
            time.sleep(0.02)
        self.assertTrue(self.ready_file.exists())
        refused = GUARD.run([str(self.cargo), "clean"], True, self.fixture)
        self.assertEqual(3, refused)
        self.release_file.touch()
        self.assertEqual(0, process.wait())
        self.assertEqual(7, GUARD.run([str(self.cargo), "clean"], True, self.fixture))

    def test_normal_runs_share_lock(self) -> None:
        child = (
            "import importlib.util, pathlib, sys; "
            "s=importlib.util.spec_from_file_location('guard', sys.argv[3]); "
            "m=importlib.util.module_from_spec(s); s.loader.exec_module(m); "
            "raise SystemExit(m.run([sys.argv[2], 'parallel'], False, pathlib.Path(sys.argv[1])))"
        )
        processes = [
            subprocess.Popen(
                [sys.executable, "-c", child, str(self.fixture), str(self.cargo), str(ROOT / "scripts/maintenance/cargo-guard.py")],
                cwd=self.fixture,
                env=self.env,
            )
            for _ in range(2)
        ]
        deadline = time.time() + 2
        while time.time() < deadline and (not self.parallel_log.exists() or len(self.parallel_log.read_text().splitlines()) < 2):
            time.sleep(0.05)
        self.assertEqual(2, len(self.parallel_log.read_text().splitlines()))
        self.assertTrue(all(process.poll() is None for process in processes))
        self.parallel_release.touch()
        self.assertEqual([0, 0], [process.wait() for process in processes])

    def test_cleanup_rejects_unguarded_open_target_file(self) -> None:
        target = self.fixture / "target"
        held = target / "held.bin"
        held.write_text("held", encoding="utf-8")
        holder = subprocess.Popen(
            [sys.executable, "-c", "import os, time; f=open('target/held.bin'); open(os.environ['READY_FILE'], 'w').close();\nwhile not os.path.exists(os.environ['RELEASE_FILE']): time.sleep(0.05)"],
            cwd=self.fixture,
            env=self.env,
        )
        deadline = time.time() + 2
        while time.time() < deadline and not self.ready_file.exists():
            time.sleep(0.02)
        self.assertTrue(self.ready_file.exists())
        try:
            self.assertEqual(3, GUARD.run([str(self.cargo), "clean"], True, self.fixture))
        finally:
            self.release_file.touch()
            holder.wait()

    def test_target_scope_rejects_outside_and_symlink(self) -> None:
        previous = os.environ.get("CARGO_TARGET_DIR")
        try:
            os.environ["CARGO_TARGET_DIR"] = str(self.fixture.parent / "outside-target")
            with self.assertRaises(ValueError):
                GUARD.target_paths(self.fixture)
        finally:
            if previous is None:
                os.environ.pop("CARGO_TARGET_DIR", None)
            else:
                os.environ["CARGO_TARGET_DIR"] = previous
        symlink = self.fixture / "symlink-target"
        symlink.symlink_to(self.fixture / "target")
        os.environ["CARGO_TARGET_DIR"] = str(symlink)
        with self.assertRaises(ValueError):
            GUARD.target_paths(self.fixture)
        os.environ.pop("CARGO_TARGET_DIR", None)

    def test_target_scope_rejects_source_and_missing_tag(self) -> None:
        self.assertEqual(EXPECTED_CACHEDIR_TAG, GUARD.CACHEDIR_TAG)
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [self.fixture / "crates" / "demo" / "src"])
        untagged = self.fixture / "untagged-target"
        untagged.mkdir(exist_ok=True)
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [untagged])
        incomplete = self.fixture / "incomplete-target"
        incomplete.mkdir(exist_ok=True)
        (incomplete / "CACHEDIR.TAG").write_bytes(EXPECTED_CACHEDIR_TAG[:-1].encode("ascii"))
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [incomplete])
        invalid = self.fixture / "invalid-tag-target"
        invalid.mkdir(exist_ok=True)
        (invalid / "CACHEDIR.TAG").write_bytes((EXPECTED_CACHEDIR_TAG[:-1] + "0").encode("ascii"))
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [invalid])

    def test_metadata_resolver_uses_all_manifests_and_fails_closed(self) -> None:
        class Result:
            def __init__(self, returncode: int, stdout: str = "") -> None:
                self.returncode = returncode
                self.stdout = stdout
                self.stderr = ""

        target = self.fixture / "target"
        calls = []

        def runner(command, **kwargs):
            calls.append(command)
            return Result(0, '{"target_directory":"%s"}' % target)

        resolved = GUARD.metadata_target_paths(self.fixture, ["rtk", str(self.cargo), "clean"], runner=runner)
        self.assertEqual([target], resolved)
        self.assertEqual(len(GUARD.MANIFESTS), len(calls))
        self.assertTrue(all(command[0:2] == ["rtk", str(self.cargo)] for command in calls))

        def failing_runner(command, **kwargs):
            return Result(9)

        with self.assertRaises(RuntimeError):
            GUARD.metadata_target_paths(self.fixture, [str(self.cargo), "clean"], runner=failing_runner)
        self.assertEqual(
            "clean",
            GUARD.cargo_subcommand(["rtk", str(self.cargo), "+nightly", "--config", "build.target-dir=x", "clean"]),
        )
        with self.assertRaises(ValueError):
            GUARD.metadata_target_paths(self.fixture, [str(self.cargo), "clean", "--manifest-path", "other.toml"], runner=runner)
        with self.assertRaises(ValueError):
            GUARD.metadata_target_paths(self.fixture, [str(self.cargo), "clean", "--target-dir", "other-target"], runner=runner)
        sweep_calls = []
        GUARD.metadata_target_paths(self.fixture, [str(self.cargo), "sweep", "--time", "7"], runner=lambda command, **kwargs: (sweep_calls.append(command) or Result(0, '{"target_directory":"%s"}' % target)))
        self.assertTrue(all("--time" not in command for command in sweep_calls))

    def test_open_file_inspection_fails_closed_when_tool_missing(self) -> None:
        def missing_tool(*args, **kwargs):
            raise FileNotFoundError("lsof")

        self.assertTrue(GUARD.active_open_files([self.fixture / "target"], runner=missing_tool))

    def test_cleanup_preserves_custom_command_prefix_and_arguments(self) -> None:
        result = GUARD.run([str(self.proxy), str(self.child_failure_cargo), "clean", "--color", "never"], True, self.fixture)
        self.assertEqual(9, result)
        lines = self.prefix_log.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(GUARD.MANIFESTS) + 2, len(lines))
        clean_lines = [line for line in lines if "clean --color never --manifest-path" in line]
        self.assertEqual(1, len(clean_lines))
        self.assertTrue(all(str(self.child_failure_cargo) in line for line in clean_lines))

    def test_cleanup_propagates_first_child_failure(self) -> None:
        self.assertEqual(9, GUARD.run([str(self.child_failure_cargo), "clean"], True, self.fixture))

    def _tagged_target(self, relative: str) -> Path:
        target = self.fixture / relative
        target.mkdir(parents=True, exist_ok=True)
        (target / "CACHEDIR.TAG").write_text(EXPECTED_CACHEDIR_TAG + "\n", encoding="utf-8")
        return target

    def test_rejects_tracked_file_inside_owned_target(self) -> None:
        target = self._tagged_target("tracked-target")
        tracked = target / "Cargo.toml"
        tracked.write_text("[package]\nname = \"tracked\"\n", encoding="utf-8")
        subprocess.run(["git", "--literal-pathspecs", "-C", str(self.fixture), "add", "-f", "--", str(tracked.relative_to(self.fixture))], cwd=self.fixture, env=self.git_env, check=True)
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [target])

    def test_rejects_nonignored_untracked_file_inside_owned_target(self) -> None:
        target = self._tagged_target("nonignored-target")
        (target / ".gitignore").write_text("CACHEDIR.TAG\n", encoding="utf-8")
        (target / "Cargo.toml").write_text("[package]\nname = \"untracked\"\n", encoding="utf-8")
        (target / "src").mkdir()
        (target / "src" / "lib.rs").write_text("pub fn fixture() {}\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [target])

    def test_accepts_ignored_default_and_nested_generated_packages(self) -> None:
        default = self._tagged_target("target")
        nested = self._tagged_target("tools/generated/target")
        for target in (default, nested):
            (target / "Cargo.toml").write_text("[package]\nname = \"generated\"\n", encoding="utf-8")
            (target / "src").mkdir(exist_ok=True)
            (target / "src" / "lib.rs").write_text("pub fn generated() {}\n", encoding="utf-8")
        self.assertEqual([default, nested], GUARD.validate_targets(self.fixture, [default, nested]))

    def test_rejects_source_file_under_tagged_crate_ancestor(self) -> None:
        crate = self._tagged_target("crates/demo")
        source = crate / "src" / "lib.rs"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text("pub fn source() {}\n", encoding="utf-8")
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [crate])

    def test_rejects_tracked_file_in_metacharacter_directory_literally(self) -> None:
        target = self._tagged_target("cache[1]")
        (target / ".gitignore").write_text("CACHEDIR.TAG\n", encoding="utf-8")
        tracked = target / "Cargo.toml"
        tracked.write_text("[package]\nname = \"literal\"\n", encoding="utf-8")
        subprocess.run(["git", "--literal-pathspecs", "-C", str(self.fixture), "add", "-f", "--", str(tracked.relative_to(self.fixture))], cwd=self.fixture, env=self.git_env, check=True)
        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [target])

    def test_git_missing_fails_closed(self) -> None:
        target = self._tagged_target("missing-git-target")

        def missing_git(*args, **kwargs):
            raise FileNotFoundError("git")

        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [target], runner=missing_git)

    def test_git_nonzero_fails_closed(self) -> None:
        target = self._tagged_target("failed-git-target")

        class Result:
            returncode = 1
            stdout = ""
            stderr = "git failed"

        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [target], runner=lambda *args, **kwargs: Result())

    def test_git_requires_own_checkout_and_sanitizes_routing_environment(self) -> None:
        target = self._tagged_target("routing-target")
        calls = []

        class Result:
            returncode = 0
            stdout = str(self.fixture) + "\n"
            stderr = ""

        def runner(command, **kwargs):
            calls.append((command, kwargs))
            if "rev-parse" in command:
                return Result()
            result = Result()
            result.stdout = ""
            return result

        old = {key: os.environ.get(key) for key in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE")}
        os.environ["GIT_DIR"] = str(self.fixture / "foreign.git")
        os.environ["GIT_WORK_TREE"] = str(self.fixture.parent)
        os.environ["GIT_INDEX_FILE"] = str(self.fixture / "foreign.index")
        try:
            GUARD.validate_targets(self.fixture, [target], runner=runner)
        finally:
            for key, value in old.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value
        self.assertTrue(calls)
        self.assertTrue(all(call[0][0:3] == ["git", "--literal-pathspecs", "-C"] for call in calls))
        self.assertTrue(all(call[1]["env"].get("GIT_DIR") is None for call in calls))
        self.assertTrue(all(call[1]["env"].get("GIT_WORK_TREE") is None for call in calls))
        self.assertTrue(all(call[1]["env"].get("GIT_INDEX_FILE") is None for call in calls))

    def test_git_unknown_root_fails_closed(self) -> None:
        target = self._tagged_target("unknown-root-target")

        class Result:
            returncode = 0
            stdout = str(self.fixture.parent) + "\n"
            stderr = ""

        with self.assertRaises(ValueError):
            GUARD.validate_targets(self.fixture, [target], runner=lambda *args, **kwargs: Result())


if __name__ == "__main__":
    unittest.main()
