"""Run the real-IME driver without exposing the host display or network."""

import datetime
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
output = Path(sys.argv[1]).resolve()
output.relative_to(root / 'tmp')
output.mkdir(exist_ok=False)
# container内の専用uidが、この新規artifact directoryにだけ書き込めるようにする。
output.chmod(0o777)
command = [
    'docker', 'run', '--rm', '--name', 'kcu-real-ime-check', '--cpus', '1',
    '--network', 'none', '--platform', 'linux/arm64',
    '--mount', f'type=bind,src={root},dst=/workspace,readonly',
    '--mount', f'type=bind,src={output},dst=/artifacts',
    '--mount', 'type=volume,src=kcu-v010-linux-target,dst=/target,readonly',
    '--workdir', '/artifacts', 'kcu-ime-linux-arm64:local', 'nice', '-n', '10',
    'bash', '/workspace/scripts/manual-ui/ime/run-display.sh',
]
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
with (output / 'outer.log').open('x') as log:
    result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
proof = {'argv': command, 'started_at': started,
         'finished_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
         'actual_exit': result.returncode}
(output / 'command-result.json').write_text(json.dumps(proof, indent=2) + '\n')
print('real IME actual exit:', result.returncode)
raise SystemExit(result.returncode)
