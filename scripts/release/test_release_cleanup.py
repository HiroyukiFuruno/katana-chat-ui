from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path
from unittest.mock import patch


sys.path.insert(0, str(Path(__file__).resolve().parent))
import release_cleanup as module


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def cleanup_step_script(workflow_text: str) -> str:
    lines = workflow_text.splitlines()
    step = next(
        index for index, line in enumerate(lines)
        if line.strip() == "- name: Verify publication and clean selected local release branch"
    )
    step_indent = len(lines[step]) - len(lines[step].lstrip())
    step_end = next(
        (
            index for index in range(step + 1, len(lines))
            if lines[index].lstrip().startswith("-")
            and len(lines[index]) - len(lines[index].lstrip()) <= step_indent
        ),
        len(lines),
    )
    run = next(index for index in range(step + 1, step_end) if lines[index].strip() == "run: |")
    run_indent = len(lines[run]) - len(lines[run].lstrip())
    body: list[str] = []
    for line in lines[run + 1:]:
        indentation = len(line) - len(line.lstrip())
        if line.strip() and indentation <= run_indent:
            break
        body.append(line)
    script = textwrap.dedent("\n".join(body)).strip("\n")
    invocation = 'python3 scripts/release/release_cleanup.py "${args[@]}"'
    if script.splitlines().count(invocation) != 1 or script.splitlines()[-1] != invocation:
        raise AssertionError("workflow cleanup invocation must occur once as the final run-body command")
    return script


def run_cleanup_dispatch(
    workflow_text: str,
    *,
    event_name: str,
    source_branch: str,
    version: str = "v0.2.2",
) -> tuple[subprocess.CompletedProcess[str], list[str] | None]:
    script = cleanup_step_script(workflow_text)
    prefix = script.rsplit('\npython3 scripts/release/release_cleanup.py "${args[@]}"', 1)[0]
    probe = prefix + '\nprintf "__CLEANUP_ARGS__\\n"\nprintf "<%s>\\n" "${args[@]}"\n'
    environment = dict(os.environ)
    environment.update({
        "RELEASE_EVENT_NAME": event_name,
        "SOURCE_BRANCH": source_branch,
        "RELEASE_VERSION": version,
    })
    result = subprocess.run(
        ["bash", "-eu", "-c", probe],
        cwd=REPOSITORY_ROOT,
        env=environment,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return result, None
    lines = result.stdout.splitlines()
    try:
        marker = lines.index("__CLEANUP_ARGS__")
    except ValueError as error:
        raise AssertionError("workflow run body did not reach the cleanup-argument probe") from error
    args = [line[1:-1] for line in lines[marker + 1:] if line.startswith("<") and line.endswith(">")]
    return result, args


def git(repo: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(["git", "-C", str(repo), *args], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if check and result.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {result.stderr}")
    return result


class ReleaseCleanupTests(unittest.TestCase):
    def test_release_workflow_cleanup_arguments_follow_event_and_source_branch(self) -> None:
        workflow = (REPOSITORY_ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        cases = (
            (
                "manual tag",
                "workflow_dispatch",
                "v0.2.2",
                0,
                ["--release-tag", "v0.2.2", "--sync-only"],
            ),
            (
                "release pull request",
                "pull_request",
                "release/v0.2.2",
                0,
                ["--release-tag", "v0.2.2", "--branch", "release/v0.2.2"],
            ),
            (
                "default branch",
                "push",
                "main",
                0,
                ["--release-tag", "v0.2.2", "--sync-only"],
            ),
        )
        for label, event_name, source_branch, exit_code, expected in cases:
            with self.subTest(label=label):
                result, args = run_cleanup_dispatch(
                    workflow,
                    event_name=event_name,
                    source_branch=source_branch,
                )
                self.assertEqual(result.returncode, exit_code, result.stderr)
                self.assertEqual(args, expected)

        for label, event_name, source_branch in (
            ("non-manual tag event", "push", "v0.2.2"),
            ("unknown branch", "pull_request", "feature/not-release"),
        ):
            with self.subTest(label=label):
                result, args = run_cleanup_dispatch(
                    workflow,
                    event_name=event_name,
                    source_branch=source_branch,
                )
                self.assertEqual(result.returncode, 1)
                self.assertIsNone(args)
                self.assertIn("Unrecognized release source branch", result.stderr)

    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.remote = self.root / "remote.git"
        git(self.root, "init", "--bare", "--initial-branch=main", str(self.remote))
        git(self.root, "init", "--initial-branch=main", str(self.repo))
        git(self.repo, "config", "user.name", "Cleanup test")
        git(self.repo, "config", "user.email", "cleanup-test@example.invalid")
        self.write("README.md", b"initial\n")
        self.commit("chore: initial fixture")
        git(self.repo, "remote", "add", "origin", "https://github.com/example/repo.git")
        git(self.repo, "config", "url." + self.remote.as_uri() + ".insteadOf", "https://github.com/example/repo.git")
        git(self.repo, "push", "-u", "origin", "main")
        git(self.repo, "fetch", "origin")
        git(self.repo, "remote", "set-head", "origin", "main")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write(self, relative: str, content: bytes) -> None:
        path = self.repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)

    def commit(self, message: str) -> str:
        git(self.repo, "add", "-A")
        git(self.repo, "commit", "-m", message)
        return git(self.repo, "rev-parse", "HEAD").stdout.strip()

    def make_branch(self, branch: str, *, merge: bool) -> str:
        git(self.repo, "switch", "-c", branch)
        self.write(f"{branch.replace('/', '-')}.txt", f"{branch}\n".encode())
        tip = self.commit(f"chore: create {branch}")
        git(self.repo, "push", "-u", "origin", branch)
        if merge:
            git(self.repo, "switch", "main")
            git(self.repo, "merge", "--no-ff", branch, "-m", f"merge {branch}")
            git(self.repo, "push", "origin", "main")
            git(self.repo, "fetch", "origin")
        else:
            git(self.repo, "switch", "main")
        return tip

    def run_cleanup(self, *args: str) -> subprocess.CompletedProcess[str]:
        branches = []
        delete_remote = False
        sync_only = "--sync-only" in args
        for index, arg in enumerate(args):
            if arg == "--branch" and index + 1 < len(args):
                branches.append(args[index + 1])
            if arg == "--delete-remote":
                delete_remote = True
        stdout = StringIO()
        stderr = StringIO()
        try:
            with redirect_stdout(stdout), redirect_stderr(stderr):
                result = module.cleanup_repository(self.repo, branches, delete_remote, sync_only=sync_only)
            return subprocess.CompletedProcess(args, result, stdout.getvalue(), stderr.getvalue())
        except module.CleanupError as error:
            return subprocess.CompletedProcess(args, 1, stdout.getvalue(), stderr.getvalue() + str(error))

    def assert_local_branch(self, branch: str, exists: bool) -> None:
        result = git(self.repo, "show-ref", "--verify", "--quiet", f"refs/heads/{branch}", check=False)
        self.assertEqual(result.returncode == 0, exists)

    def assert_remote_branch(self, branch: str, exists: bool) -> None:
        result = git(self.repo, "ls-remote", "--heads", "origin", f"refs/heads/{branch}")
        self.assertEqual(bool(result.stdout.strip()), exists)

    def test_release_guard_rejects_missing_draft_and_invalid_states(self) -> None:
        real_run = module.run
        states = iter([("false", 0), ("true", 0), ("", 1), ("invalid", 0)])

        def run(args, repo, *, capture=True):
            if args and args[0] == "gh":
                output, code = next(states)
                return subprocess.CompletedProcess(args, code, output, "")
            return real_run(args, repo, capture=capture)

        with patch.object(module, "run", side_effect=run):
            module.require_release_published(self.repo, "v1.2.3")
            with self.assertRaisesRegex(module.CleanupError, "draft"):
                module.require_release_published(self.repo, "v1.2.3")
            with self.assertRaisesRegex(module.CleanupError, "not found"):
                module.require_release_published(self.repo, "v1.2.3")
            with self.assertRaisesRegex(module.CleanupError, "invalid state"):
                module.require_release_published(self.repo, "v1.2.3")

    def test_release_guard_precedes_git_cleanup_engine(self) -> None:
        calls: list[str] = []

        def guard(repo: Path, tag: str) -> None:
            calls.append("guard")

        def engine(repo: Path, branches: list[str], delete_remote: bool, *, sync_only: bool = False) -> int:
            calls.append("engine")
            self.assertEqual(branches, [])
            self.assertFalse(delete_remote)
            self.assertTrue(sync_only)
            return 0

        with patch.object(module, "require_release_published", side_effect=guard), patch.object(
            module, "cleanup_repository", side_effect=engine
        ):
            self.assertEqual(module.perform_cleanup(self.repo, "v1.2.3", [], False, sync_only=True), 0)
        self.assertEqual(calls, ["guard", "engine"])

    def test_sync_only_requires_one_target_mode_and_rejects_conflicts(self) -> None:
        with patch.object(module.Path, "cwd", return_value=self.repo), patch.object(module, "perform_cleanup", return_value=0) as perform:
            self.assertEqual(module.main(["--release-tag", "v1.2.3", "--sync-only"]), 0)
        perform.assert_called_once_with(self.repo.resolve(), "v1.2.3", [], False, sync_only=True)

        for argv in (
            ["--release-tag", "v1.2.3"],
            ["--release-tag", "v1.2.3", "--sync-only", "--branch", "release/example"],
            ["--release-tag", "v1.2.3", "--sync-only", "--delete-remote"],
        ):
            with self.subTest(argv=argv), redirect_stderr(StringIO()), self.assertRaises(SystemExit):
                module.main(argv)

    def test_sync_only_syncs_default_and_preserves_every_branch(self) -> None:
        self.make_branch("release/merged", merge=True)
        self.make_branch("release/unmerged", merge=False)
        self.make_branch("release/current", merge=True)
        git(self.repo, "switch", "release/current")

        publisher = self.root / "publisher"
        git(self.root, "clone", str(self.remote), str(publisher))
        git(publisher, "config", "user.name", "Cleanup publisher")
        git(publisher, "config", "user.email", "cleanup-publisher@example.invalid")
        (publisher / "published.txt").write_text("advance default branch\n")
        git(publisher, "add", "published.txt")
        git(publisher, "commit", "-m", "chore: advance default")
        published_main = git(publisher, "rev-parse", "HEAD").stdout.strip()
        git(publisher, "push", "origin", "main")

        result = self.run_cleanup("--sync-only")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("no branches were deleted", result.stdout)
        self.assertEqual(git(self.repo, "branch", "--show-current").stdout.strip(), "main")
        self.assertEqual(git(self.repo, "rev-parse", "HEAD").stdout.strip(), published_main)
        for branch in ("main", "release/merged", "release/unmerged", "release/current"):
            self.assert_local_branch(branch, True)
        for branch in ("main", "release/merged", "release/unmerged", "release/current"):
            self.assert_remote_branch(branch, True)

    def test_origin_main_fallback_cleans_only_explicit_local_branch(self) -> None:
        self.make_branch("release/not-selected", merge=True)
        self.make_branch("release/merged", merge=True)
        git(self.remote, "symbolic-ref", "HEAD", "refs/heads/hidden")
        git(self.repo, "symbolic-ref", "--delete", "refs/remotes/origin/HEAD")
        result = self.run_cleanup("--branch", "release/merged")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("release cleanup completed on main", result.stdout)
        self.assert_local_branch("release/merged", False)
        self.assert_remote_branch("release/merged", True)
        self.assert_local_branch("release/not-selected", True)
        self.assert_remote_branch("release/not-selected", True)

    def test_dirty_or_untracked_worktree_is_rejected_before_cleanup(self) -> None:
        self.make_branch("release/merged", merge=True)
        self.write("untracked.txt", b"keep me\n")
        result = self.run_cleanup("--branch", "release/merged")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("untracked files", result.stderr)
        self.assert_local_branch("release/merged", True)
        self.assert_remote_branch("release/merged", True)
        self.assertTrue((self.repo / "untracked.txt").exists())

    def test_unmerged_branch_is_retained_and_reported(self) -> None:
        self.make_branch("release/unmerged", merge=False)
        result = self.run_cleanup("--branch", "release/unmerged", "--delete-remote")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unmerged local branch", result.stderr)
        self.assert_local_branch("release/unmerged", True)
        self.assert_remote_branch("release/unmerged", True)

    def test_used_by_worktree_reads_existing_checkout_ref(self) -> None:
        self.make_branch("release/used", merge=True)
        git(self.repo, "switch", "release/used")
        self.assertTrue(module.used_by_worktree(self.repo, "release/used"))
        self.assertFalse(module.used_by_worktree(self.repo, "release/not-selected"))

    def test_remote_deletion_requires_opt_in_and_preserves_default(self) -> None:
        self.make_branch("release/not-selected", merge=True)
        self.make_branch("release/remote", merge=True)
        without_opt_in = self.run_cleanup("--branch", "release/remote")
        self.assertEqual(without_opt_in.returncode, 0, without_opt_in.stderr)
        self.assert_local_branch("release/remote", False)
        self.assert_remote_branch("release/remote", True)

        with_opt_in = self.run_cleanup("--branch", "release/remote", "--delete-remote")
        self.assertEqual(with_opt_in.returncode, 0, with_opt_in.stderr)
        self.assert_remote_branch("release/remote", False)
        self.assert_local_branch("release/not-selected", True)
        self.assert_remote_branch("release/not-selected", True)

        default = self.run_cleanup("--branch", "main", "--delete-remote")
        self.assertNotEqual(default.returncode, 0)
        self.assertIn("default branch", default.stderr)
        self.assert_local_branch("main", True)
        self.assert_remote_branch("main", True)

    def test_remote_tip_change_after_fetch_is_retained(self) -> None:
        self.make_branch("release/live", merge=True)
        other = self.root / "other-clone"
        git(self.root, "clone", str(self.remote), str(other))
        git(other, "config", "user.name", "Cleanup test")
        git(other, "config", "user.email", "cleanup-test@example.invalid")
        git(other, "switch", "--track", "origin/release/live")
        path = other / "remote-advance.txt"
        path.write_text("advanced remote tip\n")
        git(other, "add", "remote-advance.txt")
        git(other, "commit", "-m", "chore: advance selected branch")
        git(other, "push", "origin", "release/live")

        retained = module.cleanup_target(self.repo, "main", "release/live", True)
        self.assertFalse(retained)
        self.assert_local_branch("release/live", True)
        self.assert_remote_branch("release/live", True)

    def test_compare_and_swap_refuses_remote_advance_after_tip_read(self) -> None:
        self.make_branch("release/cas", merge=True)
        expected = module.remote_tip(self.repo, "release/cas")
        self.assertIsNotNone(expected)
        other = self.root / "other-clone"
        git(self.root, "clone", str(self.remote), str(other))
        git(other, "config", "user.name", "Cleanup test")
        git(other, "config", "user.email", "cleanup-test@example.invalid")
        git(other, "switch", "--track", "origin/release/cas")
        (other / "advance.txt").write_text("retain this newer commit\n")
        git(other, "add", "advance.txt")
        git(other, "commit", "-m", "chore: advance remote tip")
        advanced = git(other, "rev-parse", "HEAD").stdout.strip()
        git(other, "push", "origin", "release/cas")

        with self.assertRaisesRegex(module.CleanupError, "expected tip"):
            module.delete_remote_at_tip(self.repo, "release/cas", str(expected))
        self.assertEqual(module.remote_tip(self.repo, "release/cas"), advanced)

    def test_local_delete_uses_default_branch_even_when_tracking_another_branch(self) -> None:
        self.make_branch("release/not-selected", merge=True)
        self.make_branch("release/merged", merge=True)
        git(self.repo, "branch", "--set-upstream-to=origin/release/not-selected", "release/merged")
        config_before = (self.repo / ".git/config").read_bytes()
        result = self.run_cleanup("--branch", "release/merged")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(module.local_branch_exists(self.repo, "release/merged"))
        config_after = (self.repo / ".git/config").read_bytes()
        self.assertNotIn(module.NO_UPSTREAM_REMOTE.encode(), config_after)
        self.assertNotEqual(config_before, config_after)

    def test_git_delete_refuses_tip_that_is_merged_only_to_its_own_upstream(self) -> None:
        self.make_branch("release/unmerged-tip", merge=False)
        config_before = (self.repo / ".git/config").read_bytes()
        result = git(
            self.repo,
            "-c", f"branch.release/unmerged-tip.remote={module.NO_UPSTREAM_REMOTE}",
            "branch", "-d", "--", "release/unmerged-tip", check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not fully merged", result.stderr)
        self.assertTrue(module.local_branch_exists(self.repo, "release/unmerged-tip"))
        self.assertEqual((self.repo / ".git/config").read_bytes(), config_before)

    def test_reserved_command_remote_name_collision_fails_closed(self) -> None:
        self.make_branch("release/merged", merge=True)
        git(self.repo, "remote", "add", module.NO_UPSTREAM_REMOTE, str(self.remote))
        with self.assertRaisesRegex(module.CleanupError, "reserved"):
            module.delete_local_merged_to_default(self.repo, "main", "release/merged")

    def test_ff_only_failure_aborts_before_target_cleanup(self) -> None:
        self.make_branch("release/merged", merge=True)
        self.write("local-only.txt", b"local divergence\n")
        self.commit("chore: local divergence")
        other = self.root / "other-clone"
        git(self.root, "clone", str(self.remote), str(other))
        git(other, "config", "user.name", "Cleanup test")
        git(other, "config", "user.email", "cleanup-test@example.invalid")
        git(other, "switch", "main")
        path = other / "remote-only.txt"
        path.write_text("remote divergence\n")
        git(other, "add", "remote-only.txt")
        git(other, "commit", "-m", "chore: remote divergence")
        git(other, "push", "origin", "main")

        result = self.run_cleanup("--branch", "release/merged")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("pull --ff-only", result.stderr)
        self.assert_local_branch("release/merged", True)
        self.assert_remote_branch("release/merged", True)

    def test_ambiguous_default_fallback_fails_closed(self) -> None:
        self.make_branch("release/merged", merge=True)
        git(self.repo, "symbolic-ref", "--delete", "refs/remotes/origin/HEAD")
        git(self.repo, "switch", "-c", "master")
        self.write("master.txt", b"master branch\n")
        self.commit("chore: add remote master")
        git(self.repo, "push", "-u", "origin", "master")
        git(self.repo, "switch", "main")
        git(self.repo, "fetch", "origin")
        git(self.remote, "symbolic-ref", "HEAD", "refs/heads/hidden")
        git(self.repo, "symbolic-ref", "--delete", "refs/remotes/origin/HEAD", check=False)
        result = self.run_cleanup("--branch", "release/merged")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("both origin/master and origin/main", result.stderr)
        self.assert_local_branch("release/merged", True)
        self.assert_remote_branch("release/merged", True)


if __name__ == "__main__":
    unittest.main()
