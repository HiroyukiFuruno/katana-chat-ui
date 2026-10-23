import copy
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import matrix


class BaselineContract(unittest.TestCase):
    def setUp(self):
        self.baseline = matrix.load_json(Path(__file__).parent / "baselines/aarch64.json")

    def test_missing_scenario_cannot_reduce_the_gate(self):
        names = matrix.SCENARIOS - {"narrow-chat"}
        self.baseline["scenarios"].pop("narrow-chat")
        with self.assertRaises(ValueError):
            matrix.validate_baseline(self.baseline, names)

    def test_missing_host_and_malformed_hash_are_rejected(self):
        for hashes in [{"egui": "0" * 64}, dict.fromkeys(matrix.HOSTS, "invalid")]:
            baseline = copy.deepcopy(self.baseline)
            baseline["scenarios"]["empty-chat"]["sha256"] = hashes
            with self.assertRaises(ValueError):
                matrix.validate_baseline(baseline, matrix.SCENARIOS)

    def test_viewport_dimensions_are_bounded_integers(self):
        for width, height in [(True, 900), (320, 239), (4097, 900), (320.5, 900)]:
            self.assertFalse(matrix.valid_viewport({"width": width, "height": height}))
        self.assertTrue(matrix.valid_viewport({"width": 320, "height": 240}))

    def test_reviewed_baseline_contains_every_required_case(self):
        matrix.validate_baseline(self.baseline, matrix.SCENARIOS)

    def write_requests(self, root):
        requests = root / "requests"
        requests.mkdir()
        for name, scenario in self.baseline["scenarios"].items():
            (requests / f"{name}.json").write_text(
                json.dumps({"viewport": scenario["viewport"]}), encoding="utf-8"
            )
        return requests

    def run_case_then_fail(self, root):
        calls = 0

        def run_case(*_args):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise RuntimeError("capture failed: empty-chat/floem")
            return root / "native-egui.png", "0" * 64

        return run_case

    def test_failed_matrix_records_diagnosis_and_completed_images(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "output"
            requests = self.write_requests(root)
            args = SimpleNamespace(
                baseline=Path(__file__).parent / "baselines/aarch64.json",
                requests=requests,
                output=output,
            )

            with mock.patch.object(matrix, "arguments", return_value=args), mock.patch.object(
                matrix.platform, "system", return_value="Linux"
            ), mock.patch.object(matrix.platform, "machine", return_value="aarch64"), mock.patch.object(
                matrix, "run_case", side_effect=self.run_case_then_fail(root)
            ), mock.patch.object(
                matrix, "verify_png", return_value={"path": "native-egui.png"}
            ):
                with self.assertRaisesRegex(RuntimeError, "capture failed"):
                    matrix.main()

            summary = json.loads((output / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["status"], "failed")
            self.assertEqual(summary["diagnosis"], {
                "type": "RuntimeError", "message": "capture failed: empty-chat/floem"
            })
            self.assertEqual(set(summary["images"]), {"empty-chat/egui"})


if __name__ == "__main__":
    unittest.main()
