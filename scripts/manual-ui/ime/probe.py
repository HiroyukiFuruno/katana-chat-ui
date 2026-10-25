"""Exercise a real Mozc/XIM session only inside a private Xvfb container."""

import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess
import time

OUT = Path('/artifacts')
WINDOWS = {'egui': '^katana-chat-ui harness egui$',
           'floem': '^katana-chat-ui harness floem$',
           'gpui': '^katana-chat-ui harness gpui$'}
STATE_SOURCES = {
    'egui': ('egui-kuc-runtime-frame',),
    'floem': ('floem-kuc-runtime-frame',),
    'gpui': ('gpui-kuc-runtime-frame', 'gpui-native-marked-range'),
}


def run(*args):
    return subprocess.check_output(args, text=True).strip()


def wait_for(query, label, process=None):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if process is not None and process.poll() is not None:
            raise RuntimeError('app exited while waiting for ' + label)
        result = query()
        if result:
            return result
        time.sleep(0.1)
    raise RuntimeError('timed out waiting for ' + label)


def complete_json_lines(text):
    # writerが未flushの末尾だけを保留し、完結した壊れたrecordはエラーにする。
    return [json.loads(line) for line in text.splitlines(keepends=True)
            if line.endswith('\n') and line.strip()]


def trace_events(path):
    if not path.exists():
        return []
    return complete_json_lines(path.read_text())


def complete_state_lines(text):
    records = []
    last_sequence = {}
    for line in text.splitlines(keepends=True):
        if not line.endswith('\n') or not line.strip():
            continue
        record = line.rstrip('\r\n')
        if not record.startswith('KCU_IME_STATE '):
            if record.startswith('KCU_IME_STATE'):
                raise ValueError('invalid KCU_IME_STATE diagnostic line')
            continue
        payload = json.loads(record[len('KCU_IME_STATE '):])
        if not isinstance(payload, dict) or payload.get('kind') != 'ime-state':
            raise ValueError('invalid KCU_IME_STATE record')
        if not isinstance(payload.get('source'), str) or not payload['source']:
            raise ValueError('invalid KCU_IME_STATE source')
        if type(payload.get('seq')) is not int or payload['seq'] < 0:
            raise ValueError('invalid KCU_IME_STATE sequence')
        if (type(payload.get('observed_at')) not in (int, float)
                or isinstance(payload['observed_at'], bool)
                or not math.isfinite(payload['observed_at'])
                or payload['observed_at'] < 0):
            raise ValueError('invalid KCU_IME_STATE timestamp')
        if type(payload.get('marked')) is not bool:
            raise ValueError('invalid KCU_IME_STATE marked state')
        previous = last_sequence.get(payload['source'])
        if previous is not None and payload['seq'] <= previous:
            raise ValueError('non-monotonic KCU_IME_STATE sequence')
        last_sequence[payload['source']] = payload['seq']
        records.append(payload)
    return records


def state_events(path):
    if not path.exists():
        return []
    return complete_state_lines(path.read_text())


def ibus_signal_blocks(text):
    return [block for block in re.split(r'(?=^signal time=)', text, flags=re.MULTILINE)
            if block.startswith('signal time=')]


def ibus_signal_time(block):
    match = re.search(r'^signal time=([0-9]+(?:\.[0-9]+)?)', block, flags=re.MULTILINE)
    return float(match.group(1)) if match else None


def ibus_context_path(block):
    match = re.search(r'\bpath=([^;\s]+)', block)
    return match.group(1) if match else None


def latest_input_context(blocks):
    for block in reversed(blocks):
        if 'interface=org.freedesktop.IBus.InputContext;' in block:
            context = ibus_context_path(block)
            if context is not None:
                return context
    return None


def cancel_ibus_state(text, baseline_signal_count, minimum_signal_time, context_path):
    if context_path is None:
        return {'hide_preedit': 0, 'empty_preedit': 0, 'cleared': False}
    fresh = ibus_signal_blocks(text)[baseline_signal_count:]
    input_context = [block for block in fresh
                     if 'interface=org.freedesktop.IBus.InputContext;' in block
                     and ibus_signal_time(block) is not None
                     and ibus_signal_time(block) >= minimum_signal_time
                     and ibus_context_path(block) == context_path]
    preedit_members = ('HidePreeditText', 'ShowPreeditText', 'UpdatePreeditText')
    for block in input_context:
        member = re.search(r'member=([^\s]+)', block)
        if member and 'Preedit' in member.group(1) and member.group(1) not in preedit_members:
            raise ValueError('unknown IBus preedit state member')
    preedit = [block for block in input_context
               if any(f'member={member}' in block for member in preedit_members)]
    hide = [block for block in preedit if 'member=HidePreeditText' in block]
    empty = [block for block in preedit
             if 'member=UpdatePreeditText' in block
             and 'string ""' in block
             and re.search(r'\n\s*boolean false\s*$', block)]
    latest = preedit[-1] if preedit else ''
    latest_is_empty = bool('member=UpdatePreeditText' in latest
                           and 'string ""' in latest
                           and re.search(r'\n\s*boolean false\s*$', latest))
    return {'hide_preedit': len(hide), 'empty_preedit': len(empty),
            'cleared': bool(hide) and ('member=HidePreeditText' in latest or latest_is_empty)}


def latest_host_marked_state(
    events, minimum_observed_at=0.0, minimum_sequence=-1, source=None
):
    states = [event for event in events
              if event.get('kind') == 'ime-state'
              and (source is None or event.get('source') == source)
              and type(event.get('marked')) is bool
              and type(event.get('observed_at')) in (int, float)
              and not isinstance(event.get('observed_at'), bool)
              and event['observed_at'] >= minimum_observed_at
              and type(event.get('seq')) is int
              and event['seq'] > minimum_sequence]
    return max(states, key=lambda event: (event['seq'], event['observed_at']), default=None)


def host_marked_state_cleared(
    events, baseline_event_count, minimum_observed_at, baseline_sequence, sources
):
    fresh = events[baseline_event_count:]
    return all(
        (state := latest_host_marked_state(
            fresh, minimum_observed_at, baseline_sequence.get(source, -1), source
        )) is not None and state['marked'] is False
        for source in sources
    )


def cancellation_state(
    bus_log, state_log, bus_baseline, state_baseline, minimum_observed_at, context_path,
    baseline_sequence, sources
):
    ibus = cancel_ibus_state(
        bus_log.read_text(), bus_baseline, minimum_observed_at, context_path
    )
    if not ibus['cleared']:
        return None
    states = state_events(state_log)
    if not host_marked_state_cleared(
        states, state_baseline, minimum_observed_at, baseline_sequence, sources
    ):
        return None
    host = {source: latest_host_marked_state(
        states[state_baseline:], minimum_observed_at,
        baseline_sequence.get(source, -1), source
    ) for source in sources}
    return {'ibus': ibus, 'host': host}


def commands(path):
    return [e['command'] for e in trace_events(path) if e['kind'] == 'command']


def capture(window, name, process):
    path = OUT / (name + '.png')
    previous = None
    stable = 0

    def frame():
        nonlocal previous, stable
        subprocess.run(['import', '-window', window, str(path)], check=True)
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        stable = stable + 1 if digest == previous else 0
        previous = digest
        return stable >= 3 and int(run('identify', '-format', '%k', str(path))) >= 80

    wait_for(frame, name + ' stable image', process)
    data = path.read_bytes()
    geometry = dict(line.split('=', 1) for line in run('xdotool', 'getwindowgeometry', '--shell', window).splitlines())
    x, y = max(0, int(geometry['X']) - 100), max(0, int(geometry['Y']) - 100)
    width, height = int(geometry['WIDTH']) + 200, int(geometry['HEIGHT']) + 300
    # 外部候補windowも同じprivate displayから採取し、app単独captureの遮蔽と区別する。
    scene = OUT / (name + '-scene.png')
    subprocess.run(['import', '-window', 'root', '-crop', f'{width}x{height}+{x}+{y}', str(scene)], check=True)
    return {'path': str(path), 'sha256': hashlib.sha256(data).hexdigest(),
            'dimensions': [int.from_bytes(data[16:20], 'big'), int.from_bytes(data[20:24], 'big')],
            'captured_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'private_display_scene': str(scene)}


def find_window(vendor):
    result = subprocess.run(['xdotool', 'search', '--onlyvisible', '--name', WINDOWS[vendor]],
                            capture_output=True, text=True)
    return result.returncode == 0 and result.stdout.strip().splitlines()[-1]


def key(name):
    subprocess.run(['xdotool', 'key', '--clearmodifiers', name], check=True)


def probe(vendor):
    state = OUT / (vendor + '-state')
    state.mkdir()
    trace = OUT / (vendor + '-host.jsonl')
    bus_log = OUT / (vendor + '-ibus.log')
    app_log = OUT / (vendor + '-app.log')
    env = dict(os.environ, KCU_TRACE_PATH=str(trace), KCU_IME_STATE_TRACE='1')
    images = []
    result = {'vendor': vendor, 'status': 'running', 'images': images}
    with app_log.open('x') as log, bus_log.open('x') as bus_output:
        monitor = subprocess.Popen(['dbus-monitor', '--address', run('ibus', 'address'),
                                    "type='signal'"], stdout=bus_output, stderr=subprocess.STDOUT)
        process = subprocess.Popen(['/target/release/manual-host-' + vendor], cwd=state,
                                   env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            window = wait_for(lambda: find_window(vendor), 'native window', process)
            subprocess.run(['xdotool', 'windowactivate', '--sync', window], check=True)
            subprocess.run(['xdotool', 'mousemove', '--window', window, '640', '780', 'click', '1'], check=True)
            images.append(capture(window, vendor + '-before', process))
            before = commands(trace)
            # fresh HOMEのMozc設定がひらがなで起動する。状態は実preedit/commitで検査する。
            subprocess.run(['xdotool', 'type', '--clearmodifiers', '--delay', '60', 'nihongo'], check=True)
            images.append(capture(window, vendor + '-preedit', process))
            result['commands_after_preedit'] = commands(trace)[len(before):]
            key('space')
            images.append(capture(window, vendor + '-conversion', process))
            result['commands_before_commit'] = commands(trace)[len(before):]
            key('Return')
            wait_for(lambda: any('DraftChanged("日本語")' == c for c in commands(trace)),
                     'real Japanese commit', process)
            images.append(capture(window, vendor + '-committed', process))
            committed = commands(trace)[len(before):]
            assert [c for c in committed if 'DraftChanged' in c] == ['DraftChanged("日本語")'], committed
            assert not any('Action(Submit)' in c for c in committed), committed
            assert not any('DraftChanged' in c for c in result['commands_before_commit']), result
            preedit_started_at = time.time()
            subprocess.run(['xdotool', 'type', '--clearmodifiers', '--delay', '60', 'tesuto'], check=True)
            images.append(capture(window, vendor + '-cancel-preedit', process))
            state_before_cancel = state_events(app_log)
            pre_cancel_states = {
                source: latest_host_marked_state(
                    state_before_cancel, preedit_started_at, source=source
                )
                for source in STATE_SOURCES[vendor]
            }
            if any(state is None or state['marked'] is not True
                   for state in pre_cancel_states.values()):
                raise AssertionError('fresh host marked=true state is missing before Escape')
            cancel_bus_blocks = ibus_signal_blocks(bus_log.read_text())
            cancel_bus_baseline = len(cancel_bus_blocks)
            cancel_context = latest_input_context(cancel_bus_blocks)
            cancel_state_baseline = len(state_before_cancel)
            cancel_sequence = {
                source: max(
                    (event['seq'] for event in state_before_cancel
                     if event.get('source') == source and type(event.get('seq')) is int),
                    default=-1,
                )
                for source in STATE_SOURCES[vendor]
            }
            cancel_observed_at = time.time()
            key('Escape')
            cancellation = wait_for(
                lambda: cancellation_state(
                    bus_log, app_log, cancel_bus_baseline, cancel_state_baseline,
                    cancel_observed_at, cancel_context, cancel_sequence,
                    STATE_SOURCES[vendor],
                ),
                'fresh real IBus and host cancellation state', process)
            images.append(capture(window, vendor + '-cancelled', process))
            final_host = state_events(app_log)[cancel_state_baseline:]
            if not host_marked_state_cleared(
                final_host, 0, cancel_observed_at, cancel_sequence, STATE_SOURCES[vendor]
            ):
                raise AssertionError('host marked state was not false after cancelled capture')
            final = commands(trace)[len(before):]
            assert final == committed, final
            result.update(status='passed', committed_text='日本語', exact_commit_count=1,
                          submit_count=0, cancel_preserves_committed_draft=True,
                          cancel_ibus_signals=cancellation['ibus'],
                          cancel_host_marked=False,
                          cancel_host_state=cancellation['host'],
                          final_commands=final)
        except BaseException as error:
            result.update(status='failed', error=str(error), final_commands=commands(trace))
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            monitor.terminate()
            monitor.wait(timeout=10)
    result['ibus_log_sha256'] = hashlib.sha256(bus_log.read_bytes()).hexdigest()
    if result['status'] == 'passed':
        native = [block for block in ibus_signal_blocks(bus_log.read_text())
                  if 'interface=org.freedesktop.IBus.InputContext;' in block]
        preedit = [block for block in native if 'member=UpdatePreeditText' in block
                   and 'string "にほんご"' in block]
        conversion = [block for block in native if 'member=UpdatePreeditText' in block
                      and 'string "日本語"' in block]
        commit = [block for block in native if 'member=CommitText' in block
                  and 'string "日本語"' in block]
        cancel_preedit = [block for block in native if 'member=UpdatePreeditText' in block
                          and 'string "てすと"' in block]
        result['native_ime_signals'] = {'preedit': len(preedit), 'conversion': len(conversion),
                                        'commit': len(commit), 'cancel_preedit': len(cancel_preedit)}
        if not (preedit and conversion and len(commit) == 1 and cancel_preedit):
            result.update(status='failed', error='required real InputContext signals are missing')
    (OUT / (vendor + '-result.json')).write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    return result


if __name__ == '__main__':
    assert os.environ.get('KCU_X11_HEADLESS') == '1'
    assert os.environ.get('XMODIFIERS') == '@im=ibus'
    results = [probe(vendor) for vendor in WINDOWS]
    proof = {'platform': 'Linux/aarch64 private Xvfb + real IBus/Mozc',
             'macos_physical_ime': 'not tested by this Linux probe', 'results': results}
    (OUT / 'result.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n')
    raise SystemExit(0 if all(r['status'] == 'passed' for r in results) else 1)
