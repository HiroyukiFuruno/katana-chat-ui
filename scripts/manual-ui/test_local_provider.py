#!/usr/bin/env python3
"""設定の回帰でGUIやproviderを誤起動しないよう、子processへの引渡しを隔離して検査する。"""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LAUNCHER = REPO_ROOT / "scripts/manual-ui/local-provider.sh"


class LocalProviderLauncherTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory(prefix="kcu-local-provider-")
        self.root = Path(self.temp_dir.name)
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()
        self.config_dir = self.root / "claude-config"
        self.config_dir.mkdir()
        (self.config_dir / "settings.json").write_text(
            json.dumps({"permissions": {"defaultMode": "default"}}), encoding="utf-8"
        )
        self.env_file = self.root / "local-provider.env"
        self.env_file.write_text(
            "KCU_CLAUDE_OLLAMA_MODEL=qwen3:4b\n"
            "KCU_OLLAMA_ENDPOINT=http://127.0.0.1:11434\n"
            f"CLAUDE_CONFIG_DIR={self.config_dir}\n",
            encoding="utf-8",
        )
        self.received = self.root / "child.json"
        self._write_child_printer("just")
        self._write_child_printer("cargo")

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def _write_child_printer(self, name: str) -> None:
        executable = self.bin_dir / name
        executable.write_text(
            "#!/usr/bin/env python3\n"
            "import json, os, sys\n"
            f"with open({str(self.received)!r}, 'w', encoding='utf-8') as output:\n"
            "    json.dump({'args': sys.argv[1:], 'env': {key: os.environ.get(key) for key in ("
            "'KCU_CLAUDE_OLLAMA_MODEL', 'KCU_OLLAMA_ENDPOINT', 'CLAUDE_CONFIG_DIR')}}, output)\n",
            encoding="utf-8",
        )
        executable.chmod(0o755)

    def _run(self, *args: str, config_file: Path | None = None) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["PATH"] = f"{self.bin_dir}:{env['PATH']}"
        env["KCU_LOCAL_PROVIDER_ENV"] = str(config_file or self.env_file)
        for name in ("KCU_CLAUDE_OLLAMA_MODEL", "KCU_OLLAMA_ENDPOINT", "CLAUDE_CONFIG_DIR"):
            env.pop(name, None)
        return subprocess.run(
            ["bash", str(LAUNCHER), *args],
            cwd=REPO_ROOT,
            env=env,
            check=False,
            capture_output=True,
            text=True,
        )

    def _child_record(self) -> dict[str, object]:
        return json.loads(self.received.read_text(encoding="utf-8"))

    def test_each_manual_host_receives_local_settings_without_permission_override(self) -> None:
        expected_env = {
            "KCU_CLAUDE_OLLAMA_MODEL": "qwen3:4b",
            "KCU_OLLAMA_ENDPOINT": "http://127.0.0.1:11434",
            "CLAUDE_CONFIG_DIR": str(self.config_dir),
        }
        for host in ("egui", "floem", "gpui"):
            with self.subTest(host=host):
                self.received.unlink(missing_ok=True)
                result = self._run("manual-ui", host)

                self.assertEqual(result.returncode, 0, result.stderr)
                record = self._child_record()
                self.assertEqual(record["args"], [f"manual-ui-{host}"])
                self.assertEqual(record["env"], expected_env)

    def test_probe_uses_fixed_claude_provider_command(self) -> None:
        result = self._run("probe")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            self._child_record()["args"],
            [
                "run",
                "--locked",
                "--manifest-path",
                str(REPO_ROOT / "tools/host-support/Cargo.toml"),
                "--bin",
                "acp_provider_probe",
                "--",
                "claude-code",
            ],
        )

    def test_unknown_host_is_rejected_before_launch(self) -> None:
        result = self._run("manual-ui", "unknown")

        self.assertEqual(result.returncode, 2)
        self.assertIn("expected egui, floem, or gpui", result.stderr)
        self.assertFalse(self.received.exists())

    def test_missing_config_is_reported(self) -> None:
        missing = self.root / "missing.env"
        result = self._run("probe", config_file=missing)

        self.assertEqual(result.returncode, 1)
        self.assertIn("config is missing or unreadable", result.stderr)
        self.assertFalse(self.received.exists())

    def test_incomplete_config_is_reported(self) -> None:
        incomplete = self.root / "incomplete.env"
        incomplete.write_text("KCU_CLAUDE_OLLAMA_MODEL=qwen3:4b\n", encoding="utf-8")
        result = self._run("probe", config_file=incomplete)

        self.assertEqual(result.returncode, 1)
        self.assertIn("KCU_OLLAMA_ENDPOINT must be set", result.stderr)
        self.assertFalse(self.received.exists())

    def test_nonlocal_endpoint_is_rejected(self) -> None:
        external = self.root / "external.env"
        external.write_text(
            "KCU_CLAUDE_OLLAMA_MODEL=qwen3:4b\n"
            "KCU_OLLAMA_ENDPOINT=https://example.invalid\n"
            f"CLAUDE_CONFIG_DIR={self.config_dir}\n",
            encoding="utf-8",
        )
        result = self._run("probe", config_file=external)

        self.assertEqual(result.returncode, 1)
        self.assertIn("only permits the local Ollama endpoint", result.stderr)
        self.assertFalse(self.received.exists())

    def test_nondefault_permission_mode_is_rejected(self) -> None:
        (self.config_dir / "settings.json").write_text(
            json.dumps({"permissions": {"defaultMode": "acceptEdits"}}), encoding="utf-8"
        )
        result = self._run("probe")

        self.assertEqual(result.returncode, 1)
        self.assertIn("permissions.defaultMode=default", result.stderr)
        self.assertFalse(self.received.exists())


if __name__ == "__main__":
    unittest.main()
