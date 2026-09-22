#!/usr/bin/python3
"""Offline finite HTTP-client stand-in for the T08 contract battery. Never opens a network connection.

scenario.json (cwd) carries model, prompt, version, tags, ps, optional post_ps, generated and an
optional `fault` object applied to the generate operation only:
  {"kind":"http_error","stdout":..,"stderr":..,"exit":N}  curl --fail-with-body shape: body on
                                                          stdout, one diagnostic line on stderr, exit N
  {"kind":"pause","seconds":N}                            sleep before answering
  {"kind":"signal","number":N}                            kill self with that signal (transport loss)
  {"kind":"exit","code":N} {"kind":"stderr"} {"kind":"large"} {"kind":"raw","text":..}
Every operation appends its name to calls.log so a test can prove which operations ran.
"""
import json
import os
import sys
import time
from pathlib import Path

root = Path.cwd()
s = json.loads((root / 'scenario.json').read_text())
operation = sys.argv[-1].rsplit('/', 1)[-1]
assert sys.argv[1] == '-q'
assert sys.argv[-1].startswith('http://127.0.0.1:11434/api/')
assert '--noproxy' in sys.argv and '--max-redirs' in sys.argv
assert sys.argv[sys.argv.index('--max-redirs') + 1] == '0'
assert '--fail-with-body' in sys.argv and '--proto' in sys.argv
assert sys.argv[sys.argv.index('--proto') + 1] == '=http'
assert '--retry' not in sys.argv and '--location' not in sys.argv
with (root / 'calls.log').open('a') as log:
    log.write(operation + '\n')
if operation == 'generate':
    raw = sys.stdin.buffer.read(300000)
    (root / 'captured-request.json').write_bytes(raw)
    request = json.loads(raw)
    assert request == {'model': s['model'], 'prompt': s['prompt'], 'stream': False, 'raw': True,
                       'truncate': False, 'shift': False, 'keep_alive': 60,
                       'options': {'num_ctx': 512, 'num_predict': 64}}
    (root / 'generation-started').write_text('fake client reached generation')
    fault = s.get('fault') or {}
    kind = fault.get('kind')
    if kind == 'pause':
        time.sleep(fault['seconds'])
    elif kind == 'signal':
        os.kill(os.getpid(), fault['number'])
        time.sleep(30)
    elif kind == 'http_error':
        sys.stdout.buffer.write(fault['stdout'].encode())
        sys.stdout.flush()
        sys.stderr.buffer.write(fault['stderr'].encode())
        sys.stderr.flush()
        sys.exit(fault['exit'])
    elif kind == 'exit':
        sys.exit(fault['code'])
    elif kind == 'stderr':
        sys.stderr.write('fixture diagnostic')
    elif kind == 'large':
        sys.stdout.write('x' * 70000)
        sys.exit(0)
    elif kind == 'raw':
        sys.stdout.write(fault['text'])
        sys.exit(0)
    value = s['generated']
else:
    value = s[operation]
    if operation == 'ps' and (root / 'generation-started').exists() and 'post_ps' in s:
        value = s['post_ps']
print(json.dumps(value, separators=(',', ':')))
