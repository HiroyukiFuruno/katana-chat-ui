import json
import unittest

from probe import complete_json_lines


class TraceRecords(unittest.TestCase):
    def test_inflight_suffix_does_not_hide_previous_complete_record(self):
        self.assertEqual(
            complete_json_lines('{"kind":"command","seq":1}\n{"kind":"up'),
            [{'kind': 'command', 'seq': 1}],
        )

    def test_final_newline_admits_the_record(self):
        self.assertEqual(complete_json_lines('{"draft":"日本語"}\n'), [{'draft': '日本語'}])

    def test_complete_malformed_record_is_not_silently_ignored(self):
        with self.assertRaises(json.JSONDecodeError):
            complete_json_lines('{"draft" "日本語"}\n')


if __name__ == '__main__':
    unittest.main()
