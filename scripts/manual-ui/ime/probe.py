"""Exercise a real Mozc/XIM session only inside a private Xvfb container."""

import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time

OUT = Path('/artifacts')
WINDOWS = {'egui': '^katana-chat-ui harness egui$',
           'floem': '^katana-chat-ui harness floem$',
           'gpui': '^katana-chat-ui harness gpui$'}


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
    env = dict(os.environ, KCU_TRACE_PATH=str(trace))
    images = []
    result = {'vendor': vendor, 'status': 'running', 'images': images}
    with (OUT / (vendor + '-app.log')).open('x') as log, bus_log.open('x') as bus_output:
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
            subprocess.run(['xdotool', 'type', '--clearmodifiers', '--delay', '60', 'tesuto'], check=True)
            images.append(capture(window, vendor + '-cancel-preedit', process))
            key('Escape')
            images.append(capture(window, vendor + '-cancelled', process))
            final = commands(trace)[len(before):]
            assert final == committed, final
            result.update(status='passed', committed_text='日本語', exact_commit_count=1,
                          submit_count=0, cancel_preserves_committed_draft=True,
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
        blocks = re.split(r'(?=^signal time=)', bus_log.read_text(), flags=re.MULTILINE)
        native = [block for block in blocks if 'interface=org.freedesktop.IBus.InputContext;' in block]
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
