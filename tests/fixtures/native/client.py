#!/usr/bin/python3
"""Offline finite HTTP-client stand-in. Never opens a network connection."""
import json
from pathlib import Path
import sys
import time
root=Path.cwd()
s=json.loads((root/'scenario.json').read_text())
operation=sys.argv[-1].rsplit('/',1)[-1]
assert sys.argv[1]=='-q'
assert sys.argv[-1].startswith('http://127.0.0.1:11434/api/')
assert '--noproxy' in sys.argv and '--max-redirs' in sys.argv
assert sys.argv[sys.argv.index('--max-redirs')+1]=='0'
assert '--retry' not in sys.argv and '--location' not in sys.argv
if operation=='generate':
    raw=sys.stdin.buffer.read(300000)
    (root/'captured-request.json').write_bytes(raw)
    request=json.loads(raw)
    assert request=={'model':s['model'],'prompt':'Return exactly seven.','stream':False,'raw':True,'truncate':False,'shift':False,'keep_alive':60,'options':{'num_ctx':512,'num_predict':64}}
    (root/'generation-started').write_text('actual fake process reached generation')
    if s.get('pause'):time.sleep(10)
    if s.get('exit'):sys.exit(s['exit'])
    if s.get('stderr'):sys.stderr.write('fixture diagnostic')
    if s.get('large'):sys.stdout.write('x'*70000);sys.exit(0)
    if 'raw' in s:sys.stdout.write(s['raw']);sys.exit(0)
    value=s['generated']
else:
    value=s[operation]
    if operation=='ps' and (root/'generation-started').exists() and 'post_ps' in s:value=s['post_ps']
print(json.dumps(value,separators=(',',':')))
