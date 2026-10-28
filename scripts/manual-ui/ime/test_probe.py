import json
import unittest
from pathlib import Path
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).parent))
import probe

from probe import complete_json_lines


PREEDIT = '''signal time=100 sender=org.freedesktop.IBus path=/org/freedesktop/IBus/InputContext_1;
   interface=org.freedesktop.IBus.InputContext; member=UpdatePreeditText
   string "てすと"
   boolean true
'''
HIDE = '''signal time=101 sender=org.freedesktop.IBus path=/org/freedesktop/IBus/InputContext_1;
   interface=org.freedesktop.IBus.InputContext; member=HidePreeditText
'''
EMPTY = '''signal time=102 sender=org.freedesktop.IBus path=/org/freedesktop/IBus/InputContext_1;
   interface=org.freedesktop.IBus.InputContext; member=UpdatePreeditText
   string ""
   boolean false
'''
LOOKUP = '''signal time=103 sender=org.freedesktop.IBus path=/org/freedesktop/IBus/InputContext_1;
   interface=org.freedesktop.IBus.InputContext; member=UpdateLookupTable
'''
SHOW = '''signal time=103 sender=org.freedesktop.IBus path=/org/freedesktop/IBus/InputContext_1;
   interface=org.freedesktop.IBus.InputContext; member=ShowPreeditText
'''
VISIBLE = '''signal time=103 sender=org.freedesktop.IBus path=/org/freedesktop/IBus/InputContext_1;
   interface=org.freedesktop.IBus.InputContext; member=UpdatePreeditText
   string "てすと"
   boolean true
'''


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

    def test_state_protocol_ignores_only_incomplete_final_line(self):
        text = ('noise\n'
                'KCU_IME_STATE {"kind":"ime-state","source":"egui-kuc-runtime-frame",'
                '"seq":1,"observed_at":101.0,"marked":true}\n'
                'KCU_IME_STATE {"kind":"ime-state"')
        self.assertEqual(len(probe.complete_state_lines(text)), 1)

    def test_state_protocol_rejects_malformed_complete_or_error_lines(self):
        with self.assertRaises(json.JSONDecodeError):
            probe.complete_state_lines('KCU_IME_STATE {"kind"\n')
        with self.assertRaises(ValueError):
            probe.complete_state_lines('KCU_IME_STATE_ERROR source=egui\n')

    def test_state_path_can_be_read_while_writer_is_open_and_ignores_partial_tail(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'app.log'
            with path.open('w') as writer:
                writer.write(
                    'KCU_IME_STATE {"kind":"ime-state","source":"egui-kuc-runtime-frame",'
                    '"seq":0,"observed_at":101.0,"marked":true}\n'
                )
                writer.write('KCU_IME_STATE {"kind":"ime-state"')
                writer.flush()
                self.assertEqual(len(probe.state_events(path)), 1)

    def test_state_protocol_rejects_nonfinite_negative_and_nonmonotonic_values(self):
        base = ('KCU_IME_STATE {"kind":"ime-state",'
                '"source":"egui-kuc-runtime-frame",')
        for suffix in (
            '"seq":0,"observed_at":NaN,"marked":true}\n',
            '"seq":0,"observed_at":Infinity,"marked":true}\n',
            '"seq":0,"observed_at":-1.0,"marked":true}\n',
            '"seq":-1,"observed_at":101.0,"marked":true}\n',
            '"seq":0,"observed_at":101.0,"marked":true}\n'
            + base + '"seq":0,"observed_at":102.0,"marked":false}\n',
        ):
            with self.assertRaises(ValueError):
                probe.complete_state_lines(base + suffix)

    def test_ibus_without_active_context_fails_closed(self):
        self.assertFalse(probe.cancel_ibus_state(
            HIDE, 0, 100.0, None
        )['cleared'])


class CancellationValidatorTests(unittest.TestCase):
    def test_fresh_hide_preedit_clears_real_ibus_state(self):
        state = probe.cancel_ibus_state(PREEDIT + HIDE, 1, 100.5,
                                        '/org/freedesktop/IBus/InputContext_1')
        self.assertEqual(state, {'hide_preedit': 1, 'empty_preedit': 0, 'cleared': True})

    def test_old_hide_does_not_clear_after_escape(self):
        state = probe.cancel_ibus_state(HIDE + PREEDIT, 1, 100.5,
                                        '/org/freedesktop/IBus/InputContext_1')
        self.assertEqual(state, {'hide_preedit': 0, 'empty_preedit': 0, 'cleared': False})

    def test_empty_preedit_is_recorded_alongside_hide(self):
        state = probe.cancel_ibus_state(PREEDIT + HIDE + EMPTY, 1, 100.5,
                                        '/org/freedesktop/IBus/InputContext_1')
        self.assertEqual(state, {'hide_preedit': 1, 'empty_preedit': 1, 'cleared': True})

    def test_non_preedit_signal_does_not_overwrite_latest_preedit_state(self):
        state = probe.cancel_ibus_state(PREEDIT + HIDE + LOOKUP, 1, 100.5,
                                        '/org/freedesktop/IBus/InputContext_1')
        self.assertTrue(state['cleared'])

    def test_later_show_or_visible_preedit_fails_cleared_gate(self):
        for later in (SHOW, VISIBLE):
            state = probe.cancel_ibus_state(PREEDIT + HIDE + later, 1, 100.5,
                                            '/org/freedesktop/IBus/InputContext_1')
            self.assertFalse(state['cleared'])

    def test_unknown_preedit_member_fails_closed(self):
        unknown = HIDE.replace('HidePreeditText', 'UnknownPreeditText')
        with self.assertRaises(ValueError):
            probe.cancel_ibus_state(PREEDIT + unknown, 1, 100.5,
                                    '/org/freedesktop/IBus/InputContext_1')

    def test_old_or_different_context_hide_is_not_fresh(self):
        other = HIDE.replace('InputContext_1', 'InputContext_2')
        state = probe.cancel_ibus_state(PREEDIT + other, 1, 100.5,
                                        '/org/freedesktop/IBus/InputContext_1')
        self.assertEqual(state, {'hide_preedit': 0, 'empty_preedit': 0, 'cleared': False})

    def test_host_marked_false_must_be_fresh_strict_bool(self):
        events = [
            {'kind': 'ime-state', 'phase': 'cancelled', 'marked': True},
            {'kind': 'ime-state', 'phase': 'cancelled', 'marked': False},
        ]
        events[1].update(observed_at=101.0, seq=2)
        for event in events:
            event.update(source='egui-kuc-runtime-frame')
        self.assertTrue(probe.host_marked_state_cleared(
            events, 1, 100.5, {'egui-kuc-runtime-frame': 1},
            ('egui-kuc-runtime-frame',)
        ))
        self.assertFalse(probe.host_marked_state_cleared(
            events, 2, 100.5, {'egui-kuc-runtime-frame': 1},
            ('egui-kuc-runtime-frame',)
        ))
        self.assertFalse(probe.host_marked_state_cleared(
            [events[0], {'kind': 'ime-state', 'marked': 0,
                         'observed_at': 102.0, 'seq': 3,
                         'source': 'egui-kuc-runtime-frame'}], 1, 100.5,
            {'egui-kuc-runtime-frame': 1}, ('egui-kuc-runtime-frame',)
        ))

    def test_host_state_requires_fresh_timestamp_and_sequence(self):
        event = {'kind': 'ime-state', 'phase': 'cancelled', 'marked': False,
                 'observed_at': 100.0, 'seq': 2,
                 'source': 'egui-kuc-runtime-frame'}
        self.assertFalse(probe.host_marked_state_cleared(
            [event], 0, 100.5, {'egui-kuc-runtime-frame': 1},
            ('egui-kuc-runtime-frame',)
        ))
        event['observed_at'] = 101.0
        self.assertTrue(probe.host_marked_state_cleared(
            [event], 0, 100.5, {'egui-kuc-runtime-frame': 1},
            ('egui-kuc-runtime-frame',)
        ))

    def test_latest_host_state_wins_over_an_older_false_state(self):
        events = [
            {'kind': 'ime-state', 'source': 'egui-kuc-runtime-frame',
             'marked': True, 'observed_at': 100.0, 'seq': 1},
            {'kind': 'ime-state', 'source': 'egui-kuc-runtime-frame',
             'marked': False, 'observed_at': 101.0, 'seq': 2},
            {'kind': 'ime-state', 'source': 'egui-kuc-runtime-frame',
             'marked': True, 'observed_at': 102.0, 'seq': 3},
        ]
        self.assertFalse(probe.host_marked_state_cleared(
            events, 1, 100.5, {'egui-kuc-runtime-frame': 1},
            ('egui-kuc-runtime-frame',)
        ))

    def test_real_trace_without_host_state_is_not_a_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            bus = Path(directory) / 'ibus.log'
            trace = Path(directory) / 'host.jsonl'
            bus.write_text(PREEDIT + HIDE)
            trace.write_text('{"kind":"command","command":"Escape"}\n')
            self.assertIsNone(probe.cancellation_state(
                bus, trace, 1, 1, 100.5, '/org/freedesktop/IBus/InputContext_1',
                {'egui-kuc-runtime-frame': 0}, ('egui-kuc-runtime-frame',)
            ))

    def test_old_command_only_gate_accepts_missing_cancellation_state(self):
        committed = ['DraftChanged("日本語")']
        final = list(committed)
        old_gate_passed = final == committed and not any(
            'DraftChanged' in command for command in ['Escape'])
        self.assertTrue(old_gate_passed)
        self.assertFalse(probe.host_marked_state_cleared(
            [], 0, 100.5, {'egui-kuc-runtime-frame': 0},
            ('egui-kuc-runtime-frame',)
        ))

    def test_all_gpui_sources_must_be_latest_false(self):
        sources = ('gpui-kuc-runtime-frame', 'gpui-native-marked-range')
        events = [
            {'kind': 'ime-state', 'source': source, 'marked': False,
             'observed_at': 101.0, 'seq': 2}
            for source in sources
        ]
        self.assertTrue(probe.host_marked_state_cleared(
            events, 0, 100.5, dict.fromkeys(sources, 1), sources
        ))
        events.append({'kind': 'ime-state', 'source': sources[1], 'marked': True,
                       'observed_at': 102.0, 'seq': 3})
        self.assertFalse(probe.host_marked_state_cleared(
            events, 0, 100.5, dict.fromkeys(sources, 1), sources
        ))

    def test_unrelated_source_cannot_substitute_expected_source(self):
        event = {'kind': 'ime-state', 'source': 'unrelated',
                 'marked': False, 'observed_at': 101.0, 'seq': 2}
        self.assertFalse(probe.host_marked_state_cleared(
            [event], 0, 100.5, {'egui-kuc-runtime-frame': 1},
            ('egui-kuc-runtime-frame',)
        ))


if __name__ == '__main__':
    unittest.main()
