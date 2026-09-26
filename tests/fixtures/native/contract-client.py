#!/usr/bin/python3
"""Offline finite HTTP-client stand-in for the T08 contract battery. Never opens a network connection.

scenario.json (cwd) carries model, prompt, expect (the request's `raw` and `options`, written by the
test as literals — never read from the adapter's table), version, tags, ps, optional post_ps,
generated and an optional `fault` object applied to the generate operation only:
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
    expect = s['expect']
    # A null scenario prompt admits any prompt (the t28 runtime proofs drive several attempts, whose
    # prompts differ by their history, through one scenario); the t08 battery pins its prompt exactly.
    prompt = request['prompt'] if s['prompt'] is None else s['prompt']
    assert request == {'model': s['model'], 'prompt': prompt, 'stream': False, 'raw': expect['raw'],
                       'truncate': False, 'shift': False, 'keep_alive': 60,
                       'options': expect['options']}
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
    # A LIST of generate answers is consumed in order across calls (the t28 runtime proofs drive
    # several attempts through one scenario); a single object answers every call.
    if isinstance(value, list):
        counter = root / 'generate-count'
        served = int(counter.read_text()) if counter.exists() else 0
        counter.write_text(str(served + 1))
        if served >= len(value):
            sys.stderr.write(f'fixture: asked for generate answer {served + 1} of {len(value)} scripted\n')
            sys.exit(3)
        value = value[served]
else:
    value = s[operation]
    if operation == 'ps' and (root / 'generation-started').exists() and 'post_ps' in s:
        value = s['post_ps']
print(json.dumps(value, separators=(',', ':')))
