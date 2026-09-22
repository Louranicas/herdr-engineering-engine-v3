"""Actual same-namespace PID reuse held across two real inspector calls, with the engine's physical custody arm asserted per inspection (pid_reused / live_same_identity / absent); refuses before any counter access outside a private namespace."""
import hashlib,importlib.util,json,os,select,signal,sqlite3,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
PI=Path(__file__).resolve().with_name('recovery_pi_queue.py')
GEN='07000000-0000-4000-8000-000000000001'
EPOCH='07000000-0000-4000-8000-000000000002'
TASK='07000000-0000-4000-8000-000000000003'
ATTEMPT='07000000-0000-4000-8000-000000000006'
MODES={'actual','guard','mismatch','alive','absent'}
VERDICTS={'actual':'actual-reuse','alive':'live-same-identity','absent':'absent-after-reap'}
CUSTODY={'actual':'pid_reused','alive':'live_same_identity','absent':'absent'}
# Exact pinned argv from ../pid-reuse-scout (actual-001/command.json); the fixture never alters it.
UNSHARE=['/usr/bin/unshare','--user','--map-root-user','--mount','--pid','--fork','--kill-child=KILL','--mount-proc','--propagation','private']
COUNTER=Path('/proc/sys/kernel/ns_last_pid')
GUARD='private user/pid/mount namespace guard'
PROC_GUARD='proc does not describe private PID namespace'

class Refused(Exception):
    """A fixture refusal with its own diagnostic; never an engine verdict."""

def retain(path,value):
    with path.open('x') as f:
        json.dump(value,f,sort_keys=True);f.flush();os.fsync(f.fileno())
    fd=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY)
    try:os.fsync(fd)
    finally:os.close(fd)

def pin(path):
    b=path.read_bytes();return {'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}

def identity(pid):
    """Kernel-described identity of one PID: /proc stat field 22 and the PID namespace link."""
    value=Path(f'/proc/{pid}/stat').read_text();tail=value[value.rfind(')')+2:].split()
    return {'pid':pid,'start_ticks':int(tail[19]),'namespace':os.readlink(f'/proc/{pid}/ns/pid')},tail[0]

def oracle(old,new):
    """The one reuse oracle: every dimension on which `new` is not the same-namespace reused PID with a later start."""
    mismatched=[]
    if new['pid']!=old['pid']:mismatched.append('pid')
    if new['namespace']!=old['namespace']:mismatched.append('namespace')
    if not new['start_ticks']>old['start_ticks']:mismatched.append('start')
    return mismatched

def require_reuse(old,new):
    mismatched=oracle(old,new)
    if mismatched:raise Refused('no actual reuse: '+','.join(mismatched))

def sensitivity(old):
    """Fixed literal alterations of the actually observed old identity, one dimension each; the same oracle must name exactly that dimension."""
    later=dict(old,start_ticks=old['start_ticks']+1)
    cases={'pid':dict(later,pid=old['pid']+1),'namespace':dict(later,namespace=old['namespace']+'/altered'),'start':dict(later,start_ticks=old['start_ticks'])}
    named={name:oracle(old,candidate) for name,candidate in cases.items()}
    for name,found in named.items():
        if found!=[name]:raise Refused('oracle insensitive: '+name)
    if oracle(old,later)!=[]:raise Refused('oracle refuses its literal control identity')
    return named

def read_line(fd,deadline,limit=4096):
    buf=b''
    while b'\n' not in buf:
        remaining=deadline-time.monotonic()
        if remaining<=0:raise Refused('identity read timeout')
        ready,_,_=select.select([fd],[],[],remaining)
        if not ready:raise Refused('identity read timeout')
        chunk=os.read(fd,limit+1-len(buf))
        if not chunk:raise Refused('identity EOF')
        buf+=chunk
        if len(buf)>limit:raise Refused('identity bound')
    line,rest=buf.split(b'\n',1)
    if rest:raise Refused('unexpected bytes after identity')
    return json.loads(line)

def spawn(binary,path,mode,evidence,record,children):
    """Start one directly owned child at whatever PID the private counter yields; read its self-reported identity within a bound."""
    with (evidence/f'{mode}-stderr').open('wb') as error:
        child=subprocess.Popen([str(binary),str(path),mode],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=error,close_fds=True)
    children[mode]=child;record['children_started']+=1
    reported=read_line(child.stdout.fileno(),time.monotonic()+5)
    observed,state=identity(child.pid)
    if reported!=observed or state in ('Z','X') or child.poll() is not None:raise Refused(f'{mode} identity: reported {reported} observed {observed} state {state}')
    writer=json.loads((path/f'{mode}-writer.json').read_text())
    if writer['identity']!=observed:raise Refused(f'{mode} workspace write')
    return child,observed

def release(child,name,record):
    """Release through the owned pipe and reap the direct child; no signal to any PID."""
    child.stdin.write(b'x');child.stdin.flush();child.stdin.close()
    code=child.wait(timeout=5);child.stdout.close()
    record[name+'_reaped']={'pid':child.pid,'exit':code}
    if code!=0:raise Refused(f'{name} exit {code}')

def alive(child,expected):
    current,state=identity(child.pid)
    if current!=expected or state in ('Z','X') or child.poll() is not None:raise Refused(f'held process not alive: {current} state {state}')
    return current

def gone(expected):
    """The reaped old PID must have no /proc entry at all (the private proc shows only this namespace)."""
    if Path(f'/proc/{expected["pid"]}').exists():raise Refused(f'old PID {expected["pid"]} still present')
    return {'pid':expected['pid'],'present':False}

def binding(path,old):
    """Independently re-parse the ledger: the old identity is bound in the roster observation of one unresolved attempt."""
    db=sqlite3.connect(f'file:{path}/store/generations/{GEN}/ledger.sqlite3?mode=ro',uri=True)
    try:
        db.execute('PRAGMA query_only=ON')
        observations=[json.loads(row[0]) for row in db.execute('SELECT body FROM roster_observations ORDER BY sequence')]
        if len(observations)!=1 or json.loads(observations[0]['input']['actual_identity'])!=old:raise Refused('old identity not bound in roster observation')
        attempts=list(db.execute('SELECT id,task_id,generation,state,effect,cleanup,used_ms FROM attempts'))
        if attempts!=[(ATTEMPT,TASK,'1','running','pending','pending',None)]:raise Refused(f'unresolved old attempt: {attempts}')
        pins=list(db.execute('SELECT attempt_id,record_id FROM roster_pins'))
        if pins!=[(ATTEMPT,observations[0]['input']['record_id'])]:raise Refused(f'roster pin: {pins}')
        return {'observation':observations[0],'attempts':attempts,'pins':pins}
    finally:db.close()

def pi_snapshot():
    spec=importlib.util.spec_from_file_location('recovery_pi_queue',PI);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module.snapshot

def inspect(inspector,path,name,evidence,record):
    argv=[str(inspector),'inspect-recovery',str(path/'store'),GEN,EPOCH,str(path/name)]
    child=subprocess.Popen(argv,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,close_fds=True)
    try:out,err=child.communicate(timeout=3)
    except subprocess.TimeoutExpired:
        record['unreaped'].append({'name':name,'pid':child.pid});raise Refused(f'{name} timeout')
    retain(evidence/(name+'-command.json'),{'argv':argv,'returncode':child.returncode,'stdout':list(out),'stderr':list(err)})
    if child.returncode!=0 or out or err:raise Refused(f'{name} rc={child.returncode} stderr={err!r}')
    return json.loads((path/name/'report.json').read_text())

def validate(record,custody,live):
    """Inspector and ledger oracle over the retained record; `custody` is the one arm the engine must report for the old attempt and `live` the identity /proc must have shown it (None for absent)."""
    if record['snapshot_before']!=record['snapshot_after']:raise Refused('Store changed')
    if len(record['inspections'])!=2:raise Refused('two inspections required')
    for r in record['inspections']:
        if r['execution_resumed'] is not False or r['recovery_complete'] is not False or r['replay_authorized'] is not False or r['pi_queue_custody']!='unreconciled':raise Refused('custody promotion')
        if r['physical_process_custody']!=custody:raise Refused(f'physical custody {r["physical_process_custody"]!r} is not {custody!r}')
        expected={'pid_reused':'process_identity_reused','live_same_identity':'pi_queue_unreconciled','absent':'acknowledgement_unrecorded'}[custody]
        decisions=r['reconciliation']['attempts']
        if r['policy_permits_execution'] is not False or len(decisions)!=1 or decisions[0]['attempt']!=ATTEMPT or decisions[0]['name']!='retain_unknown' or decisions[0]['decision']['reason']['reason']!=expected or decisions[0]['decision']['process']['custody']!=custody:raise Refused(f'policy decision for {custody!r} is not retain_unknown/{expected}: {decisions}')
        entries=r['physical_process_identities']
        if len(entries)!=1:raise Refused(f'one classified attempt expected: {entries}')
        e=entries[0]
        if e['attempt']!=ATTEMPT or e['record_id']!=record['binding']['observation']['input']['record_id'] or e['observed']!=record['old'] or e['custody']!=custody or e['error'] is not None:raise Refused(f'physical identity entry: {e}')
        if live is None:
            if e['live'] is not None or e['differs']!=[]:raise Refused(f'absent entry: {e}')
        else:
            seen=dict(e['live'] or {});state=seen.pop('state',None)
            if seen!=live or state in (None,'Z','X') or e['differs']!=(['start_ticks'] if custody=='pid_reused' else []):raise Refused(f'live identity entry: {e}')
        inventory=r['inventory']
        if inventory['generation']!=GEN or [t['state'] for t in inventory['tasks']]!=['running'] or [a['state'] for a in inventory['attempts']]!=['running']:raise Refused('attempt not left unresolved')
        if inventory['acceptances'] or inventory['verifications'] or inventory['stops'] or inventory['pending_delivery']:raise Refused('dispatch, settlement, acceptance or outbox observed')
    expected=live if live is not None else {'pid':record['old']['pid'],'present':False}
    if record['identity_before_inspection']!=expected or record['identity_after_inspection']!=expected:raise Refused('held identity drift')

def alarm(signum,frame):
    raise TimeoutError('inner fixture deadline')

def init(binary,inspector,mode,path,evidence,parent):
    """Namespace init (PID1 of the private namespace) or, outside one, the process the guard must refuse."""
    signal.signal(signal.SIGALRM,alarm);signal.alarm(13)
    me,_=identity(os.getpid())
    record={'kind':'hee3-development-pid-reuse-fixture/1','mode':mode,'self':me,'user_namespace':os.readlink('/proc/self/ns/user'),'mount_namespace':os.readlink('/proc/self/ns/mnt'),'parent':parent,'counter_opened':False,'counter_written':False,'counter_reset':False,'children_started':0,'inspections':[],'actual_reuse':False,'refusal':None,'verdict':None,'late_reaped':[],'unreaped':[]}
    children={}
    try:
        if os.getpid()!=1 or me['namespace']==parent['pid'] or record['user_namespace']==parent['user'] or record['mount_namespace']==parent['mnt']:raise Refused(GUARD)
        one,_=identity(1)
        if one['namespace']!=me['namespace']:raise Refused(PROC_GUARD)
        record['proc_mounts']=[line for line in Path('/proc/self/mountinfo').read_text().splitlines() if ' /proc ' in line]
        if not record['proc_mounts']:raise Refused(PROC_GUARD)
        record['counter_opened']=True;record['counter_before']=COUNTER.read_text().strip()
        COUNTER.write_text('1\n');record['counter_written']=True
        old,old_identity=spawn(binary,path,'old',evidence,record,children)
        if old_identity['pid']!=2:raise Refused(f'old PID {old_identity["pid"]} is not the private PID2')
        record['old']=old_identity
        if mode!='alive':
            release(old,'old',record);children.pop('old')
        record['binding']=binding(path,old_identity)
        time.sleep(.03)
        if mode=='actual':
            COUNTER.write_text(str(old_identity['pid']-1)+'\n');record['counter_reset']=True
        held,expected,held_name=None,None,None
        if mode in ('actual','mismatch'):
            new,new_identity=spawn(binary,path,'replacement',evidence,record,children)
            record['new']=new_identity;record['observed_mismatch']=oracle(old_identity,new_identity)
            if mode=='mismatch':
                record['sensitivity']=sensitivity(old_identity)
                release(new,'replacement',record);children.pop('replacement')
                require_reuse(old_identity,new_identity)
                raise Refused('mismatch mode observed actual reuse; a fake mismatch is not actual reuse')
            require_reuse(old_identity,new_identity)
            held,expected,held_name=new,new_identity,'replacement'
        elif mode=='alive':
            held,expected,held_name=old,old_identity,'old'
        record['expected_custody']=CUSTODY[mode]
        def observe():return alive(held,expected) if held is not None else gone(old_identity)
        snapshot=pi_snapshot();before=snapshot(path)
        record['identity_before_inspection']=observe()
        for name in ('inspect-first','inspect-second'):
            record['inspections'].append(inspect(inspector,path,name,evidence,record))
            if snapshot(path)!=before:raise Refused('Store changed')
            observe()
        record['identity_after_inspection']=observe()
        record['snapshot_before']=before;record['snapshot_after']=snapshot(path)
        validate(record,CUSTODY[mode],expected)
        if held is not None:
            release(held,held_name,record);children.pop(held_name)
            record[held_name+'_release']=json.loads((path/f'{held_name}-release.json').read_text())
            if record[held_name+'_release']['trailing_bytes']!=0 or record[held_name+'_release']['identity']!=expected:raise Refused(f'{held_name} release record')
        record['actual_reuse']=mode=='actual' and oracle(old_identity,expected)==[];record['verdict']=VERDICTS[mode]
    except Refused as error:
        record['refusal']=str(error);record['verdict']='refused'
    except BaseException as error:
        record['error']=f'{type(error).__name__}: {error}';record['verdict']='error'
    finally:
        for name,child in children.items():
            try:
                if child.stdin is not None and not child.stdin.closed:
                    try:child.stdin.write(b'x');child.stdin.flush()
                    except OSError:pass
                    child.stdin.close()
                record['late_reaped'].append({'name':name,'pid':child.pid,'exit':child.wait(timeout=2)})
            except subprocess.TimeoutExpired:
                record['unreaped'].append({'name':name,'pid':child.pid})
            finally:
                if child.stdout is not None and not child.stdout.closed:child.stdout.close()
        record['diagnostics']=sorted(p.name for p in evidence.glob('*-stderr') if p.stat().st_size)
        for p in sorted(path.glob('*.json')):retain(evidence/p.name,json.loads(p.read_text()))
        retain(evidence/'observation.json',record)
        print(json.dumps({'verdict':record['verdict'],'refusal':record['refusal'],'actual_reuse':record['actual_reuse']}),flush=True)
    return 0 if record['verdict']==VERDICTS.get(mode) else 1

def expect(mode,result,o,evidence):
    """Outer expectation per control; each refuses with its own diagnostic."""
    if result['cleanup_complete'] is not True or result['timed_out'] is not False or result['output_limit_exceeded'] is not False or result['descendants_detected'] is not False or (evidence/'stderr').read_bytes():raise ValueError('bounded fixture did not complete cleanly')
    if o['mode']!=mode or o['unreaped']!=[] or o['late_reaped']!=[] or o.get('error') is not None or o['diagnostics']!=[]:raise ValueError('fixture record not clean')
    if mode=='actual':
        if result['exit_code']!=0 or o['verdict']!='actual-reuse' or o['actual_reuse'] is not True or o['counter_reset'] is not True:raise ValueError('actual reuse not demonstrated')
        if oracle(o['old'],o['new'])!=[] or o['old']['pid']!=2 or o['old_reaped']['exit']!=0 or o['replacement_reaped']['exit']!=0 or len(o['inspections'])!=2:raise ValueError('actual reuse record')
        if json.loads(o['binding']['observation']['input']['actual_identity'])!=o['old']:raise ValueError('old identity binding')
        validate(o,'pid_reused',o['new'])
    elif mode=='alive':
        if result['exit_code']!=0 or o['verdict']!='live-same-identity' or o['actual_reuse'] is not False or o['counter_reset'] is not False or 'new' in o:raise ValueError('live original not demonstrated')
        if o['old']['pid']!=2 or o['old_reaped']['exit']!=0 or o['old_release']['identity']!=o['old'] or len(o['inspections'])!=2:raise ValueError('live original record')
        if json.loads(o['binding']['observation']['input']['actual_identity'])!=o['old']:raise ValueError('old identity binding')
        validate(o,'live_same_identity',o['old'])
    elif mode=='absent':
        if result['exit_code']!=0 or o['verdict']!='absent-after-reap' or o['actual_reuse'] is not False or o['counter_reset'] is not False or 'new' in o:raise ValueError('absent original not demonstrated')
        if o['old']['pid']!=2 or o['old_reaped']['exit']!=0 or len(o['inspections'])!=2:raise ValueError('absent original record')
        if json.loads(o['binding']['observation']['input']['actual_identity'])!=o['old']:raise ValueError('old identity binding')
        validate(o,'absent',None)
    elif mode=='guard':
        if result['exit_code']!=1 or o['verdict']!='refused' or o['refusal']!=GUARD:raise ValueError('namespace guard did not refuse')
        if o['counter_opened'] is not False or o['counter_written'] is not False or o['children_started']!=0 or 'old' in o or o['self']['pid']==1 or o['self']['namespace']!=o['parent']['pid']:raise ValueError('guard refused after counter access or child start')
    elif mode=='mismatch':
        if result['exit_code']!=1 or o['verdict']!='refused' or o['refusal']!='no actual reuse: pid' or o['actual_reuse'] is not False or o['counter_reset'] is not False:raise ValueError('differing replacement identity was not refused')
        if o['observed_mismatch']!=['pid'] or o['new']['pid']==o['old']['pid'] or o['new']['namespace']!=o['old']['namespace'] or o['inspections']!=[] or o['old_reaped']['exit']!=0 or o['replacement_reaped']['exit']!=0:raise ValueError('mismatch record')
        if o['sensitivity']!={'pid':['pid'],'namespace':['namespace'],'start':['start']}:raise ValueError('oracle sensitivity')
    else:raise ValueError('unknown mode')

def main():
    if sys.argv[1]=='--init':
        binary,inspector,mode,path,evidence=sys.argv[2:7];parent=dict(zip(('pid','user','mnt'),sys.argv[7:10]))
        return init(Path(binary),Path(inspector),mode,Path(path),Path(evidence),parent)
    binary,inspector,mode,path,evidence=map(str,sys.argv[1:]);path=Path(path);evidence=Path(evidence);evidence.mkdir()
    assert mode in MODES
    spec=importlib.util.spec_from_file_location('owner',ROOT/'tools/development_process.py');owner=importlib.util.module_from_spec(spec);spec.loader.exec_module(owner)
    inputs=[Path(binary),Path(inspector),Path(__file__),PI,ROOT/'tools/development_process.py',Path(sys.executable).resolve(),Path(UNSHARE[0])];pins={str(p):pin(p) for p in inputs}
    parent=[os.readlink('/proc/self/ns/'+x) for x in ('pid','user','mnt')]
    inner=[sys.executable,'-B',__file__,'--init',binary,inspector,mode,str(path),str(evidence),*parent]
    argv=(inner if mode=='guard' else UNSHARE+inner)
    env={'LC_ALL':'C','PATH':'/usr/bin:/bin','PYTHONWARNINGS':'error','PYTHONDONTWRITEBYTECODE':'1','TMPDIR':os.environ['TMPDIR']}
    result={'argv':argv,'unshare':mode!='guard','parent_namespaces':parent,'input_pins':pins,'accepted_control':False}
    try:
        result.update(owner.run_bounded(argv,path,env,evidence/'stdout',evidence/'stderr',time.monotonic()+25,command_timeout=15,stream_limit=1048576))
        expect(mode,result,json.loads((evidence/'observation.json').read_text()),evidence)
        result['accepted_control']=True
    except BaseException as e:
        result.update(getattr(e,'result',getattr(e,'bounded_process_result',{})));result['error']=f'{type(e).__name__}: {e}'
    finally:
        result['input_drift']=[str(p) for p in inputs if pin(p)!=pins[str(p)]]
        retain(evidence/'result.json',result)
    return 0 if result['accepted_control'] and not result['input_drift'] else 1

if __name__=='__main__':sys.exit(main())
