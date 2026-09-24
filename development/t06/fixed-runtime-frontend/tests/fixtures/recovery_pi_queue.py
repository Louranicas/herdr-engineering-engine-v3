"""Finite local Pi peer + actual Rust parent loss; never invokes a provider."""
import hashlib,importlib.util,json,os,select,sqlite3,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[5]
GEN='07000000-0000-4000-8000-000000000001'
EPOCH='07000000-0000-4000-8000-000000000002'
TASK='07000000-0000-4000-8000-000000000003'
MODES={'steering','follow-up','clear-before','clear-after','benign','wrong-session','wrong-id','late-generation','eof'}

def retain(path,value):
    with path.open('x') as f:
        json.dump(value,f,sort_keys=True);f.flush();os.fsync(f.fileno())
    fd=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY)
    try:os.fsync(fd)
    finally:os.close(fd)

def pin(path):
    b=path.read_bytes();return {'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}

def identity(pid):
    value=Path(f'/proc/{pid}/stat').read_text();tail=value[value.rfind(')')+2:].split()
    return {'pid':pid,'start_ticks':int(tail[19]),'state':tail[0],'namespace':os.readlink(f'/proc/{pid}/ns/pid')}

def send(value):
    raw=json.dumps(value,separators=(',',':')).encode()+b'\n'
    assert len(raw)<=4096
    os.write(1,raw)

def state(session='owned-pi-fixture'):
    return {'thinkingLevel':'off','isStreaming':False,'isCompacting':False,'steeringMode':'all','followUpMode':'one-at-a-time','sessionId':session,'autoCompactionEnabled':False,'messageCount':0,'pendingMessageCount':0}

def peer(path,mode):
    deadline=time.monotonic()+10;buf=b'';n=0;readable=True;steering=[];follow=[]
    retain(path/'peer-identity.json',identity(os.getpid()))
    while time.monotonic()<deadline:
        if (path/'peer-release').exists():return 0
        ready,_,_=select.select([0] if readable else [],[],[],.01)
        if not ready:continue
        chunk=os.read(0,4096)
        if not chunk:
            readable=False;retain(path/'peer-eof.json',{'commands':n,'steering':steering,'followUp':follow});continue
        buf+=chunk
        if len(buf)>4096:raise ValueError('frame bound')
        while b'\n' in buf:
            raw,buf=buf.split(b'\n',1);cmd=json.loads(raw);n+=1
            if n>16 or set(cmd)!={'id','type'}:raise ValueError('closed peer request')
            if cmd['type'] not in ('get_state','clear_queue','abort'):raise ValueError('unexpected peer action')
            retain(path/f'peer-command-{n}.json',{'raw':list(raw+b'\n'),'request':cmd})
            response={'type':'response','id':cmd['id'],'command':cmd['type'],'success':True}
            if cmd['type']=='get_state':
                response['data']=state('foreign-session' if mode=='wrong-session' and n>1 else 'owned-pi-fixture');send(response)
                if n==1:
                    steering=[] if mode=='follow-up' else ['old steering'];follow=[] if mode=='steering' else ['old follow-up']
                    retain(path/'queued.json',{'steering':steering,'followUp':follow})
                    if mode in ('steering','follow-up'):
                        send({'type':'queue_update','steering':steering if mode=='steering' else [],'followUp':follow if mode=='follow-up' else []})
            elif cmd['type']=='clear_queue':
                removed={'steering':steering,'followUp':follow}
                if mode!='clear-before':steering=[];follow=[]
                retain(path/'clear-observed.json',{'command':cmd,'effect':mode!='clear-before','removed':removed,'steering':steering,'followUp':follow})
                if mode in ('clear-before','clear-after'):continue
                if mode=='eof':os.close(1);continue
                response['data']=removed
                if mode=='wrong-id':response['id']='foreign-request'
                if mode=='late-generation':response['id']=cmd['id'].replace(':2:',':1:')
                send(response)
            else:send(response)
    raise TimeoutError('finite peer lifespan')

def snapshot(path):
    db=sqlite3.connect(f'file:{path}/store/generations/{GEN}/ledger.sqlite3?mode=ro',uri=True)
    try:
        db.execute('PRAGMA query_only=ON')
        names=[r[0] for r in db.execute("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")]
        rows={}
        for name in names:
            values=list(db.execute('SELECT * FROM "'+name.replace('"','""')+'"'))
            def cell(v):return {'bytes':v.hex()} if isinstance(v,bytes) else v
            rows[name]=sorted([[cell(v) for v in row] for row in values],key=repr)
        task=db.execute('SELECT state,generation,spent_ms,reserved_work_ms,reserved_verify_ms FROM tasks WHERE id=?',(TASK,)).fetchone()
        assert task==('running','2',0,900000,300000),task
        assert len(rows['tasks'])==len(rows['attempts'])==1
        for name in ('acceptances','verifications','task_stops'):assert rows[name]==[],name
        assert db.execute('SELECT count(*) FROM outbox JOIN events ON events.id=outbox.event_id WHERE events.task_id=?',(TASK,)).fetchone()[0]==0
        return rows
    finally:db.close()

def validate(report,mode):
    if report['mode']!=mode or report['driver_exit']!=(-9 if mode in ('steering','follow-up','clear-before','clear-after') else 0):raise ValueError('actual parent outcome')
    if report['peer_before']!=report['peer_after'] or report['peer_state_after'] in ('Z','X'):raise ValueError('peer liveness')
    if report['commands_before']!=report['commands_after']:raise ValueError('stale pipe command')
    if report['snapshot_before']!=report['snapshot_after']:raise ValueError('Store changed')
    if report['peer_exit']!=0 or report['diagnostics']!=[]:raise ValueError('peer cleanup')
    for r in report['inspections']:
        if r['execution_resumed'] is not False or r['recovery_complete'] is not False or r['physical_process_custody']!='unreconciled' or r['pi_queue_custody']!='unreconciled':raise ValueError('custody promotion')

def supervise(binary,inspector,mode,path,evidence):
    assert mode in MODES
    peer_process=driver=None
    with (evidence/'peer-stderr').open('wb') as peer_error,(evidence/'driver-stderr').open('wb') as driver_error:
        try:
            peer_process=subprocess.Popen([sys.executable,'-B',__file__,'--peer',str(path),mode],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=peer_error)
            driver=subprocess.Popen([str(binary),str(path),mode],stdin=peer_process.stdout,stdout=peer_process.stdin,stderr=driver_error)
            peer_process.stdout.close();peer_process.stdin.close()
            code=driver.wait(timeout=8)
            deadline=time.monotonic()+1
            while not (path/'peer-eof.json').exists():
                if time.monotonic()>deadline:raise TimeoutError('peer EOF not observed')
                time.sleep(.001)
            before=snapshot(path);commands={p.name:json.loads(p.read_text()) for p in sorted(path.glob('peer-command-*.json'))}
            peer_before=identity(peer_process.pid);peer_before.pop('state')
            if peer_process.poll() is not None:raise ValueError('peer not alive before inspector')
            inspections=[]
            for name in ('inspect-first','inspect-second'):
                argv=[str(inspector),'inspect-recovery',str(path/'store'),GEN,EPOCH,str(path/name)]
                r=subprocess.run(argv,capture_output=True,timeout=3,check=False)
                retain(evidence/(name+'-command.json'),{'argv':argv,'returncode':r.returncode,'stdout':list(r.stdout),'stderr':list(r.stderr)})
                assert r.returncode==0 and not r.stdout and not r.stderr,(r.returncode,r.stderr)
                report=json.loads((path/name/'report.json').read_text());inspections.append(report)
                assert snapshot(path)==before
            peer_after=identity(peer_process.pid);peer_state=peer_after.pop('state')
            if peer_process.poll() is not None or peer_state in ('Z','X'):raise ValueError('peer not alive after inspector')
            after=snapshot(path);after_commands={p.name:json.loads(p.read_text()) for p in sorted(path.glob('peer-command-*.json'))}
            (path/'peer-release').write_bytes(b'release\n');peer_exit=peer_process.wait(timeout=1)
            report={'mode':mode,'driver_exit':code,'peer_before':peer_before,'peer_after':peer_after,'peer_state_after':peer_state,'commands_before':commands,'commands_after':after_commands,'snapshot_before':before,'snapshot_after':after,'inspections':inspections,'peer_exit':peer_exit,'diagnostics':[p.name for p in (evidence/'peer-stderr',evidence/'driver-stderr') if p.stat().st_size]}
            # Queue effects are independent literal expectations, not inferred from Session status.
            queued=json.loads((path/'queued.json').read_text());assert queued=={'steering':[] if mode=='follow-up' else ['old steering'],'followUp':[] if mode=='steering' else ['old follow-up']}
            if mode.startswith('clear-'):
                cut=json.loads((path/'clear-observed.json').read_text());assert cut['effect']==(mode=='clear-after')
                assert bool(cut['steering'])==bool(cut['followUp'])==(mode=='clear-before')
                assert json.loads((path/'cut.json').read_text())['pending']==cut['command']['id']
            if mode=='benign':
                assert json.loads((path/'driver-result.json').read_text())['pi_idle'] is True
                assert [v['request']['type'] for v in commands.values()]==['get_state','clear_queue','abort','get_state']
            elif mode in ('steering','follow-up','wrong-session','wrong-id','late-generation','eof'):
                result=json.loads((path/'driver-result.json').read_text())
                expected={'steering':'Ordering','follow-up':'Ordering','wrong-session':'Session','wrong-id':'Correlation','late-generation':'Correlation','eof':'Poisoned'}[mode]
                assert result['refusal']==expected and result['poisoned'] is True
                assert result['generation']=='2' and result['session']=='owned-pi-fixture'
                if mode not in ('steering','follow-up'):assert result['pending']==list(commands.values())[-1]['request']['id']
            if mode in ('steering','follow-up','clear-before','clear-after'):
                marker=json.loads((path/'cut.json').read_text());assert marker['phase']==mode and marker['generation']=='2' and marker['task']==TASK
            assert not any(v['request']['type']=='prompt' for v in commands.values())
            # All peer command IDs must be bound to the actual Store task/attempt/generation.
            for n,value in enumerate(commands.values(),1):assert value['request']['id']==f'{TASK}:07000000-0000-4000-8000-000000000006:2:{n}'
            validate(report,mode);retain(evidence/'observation.json',report)
            for p in sorted(path.glob('*.json')):retain(evidence/p.name,json.loads(p.read_text()))
            # Guard sensitivity on actual evidence: none of these replays is another engine case.
            altered=dict(report,commands_after={});
            try:validate(altered,mode)
            except ValueError as e:assert str(e)=='stale pipe command'
            else:raise AssertionError('stale-command detector vacuous')
            return 0
        finally:
            for process in (driver,peer_process):
                if process is not None and process.poll() is None:
                    process.kill();process.wait(timeout=2)

def main():
    if sys.argv[1]=='--peer':return peer(Path(sys.argv[2]),sys.argv[3])
    if sys.argv[1]=='--supervise':return supervise(Path(sys.argv[2]),Path(sys.argv[3]),sys.argv[4],Path(sys.argv[5]),Path(sys.argv[6]))
    binary,inspector,mode,path,evidence=map(str,sys.argv[1:]);path=Path(path);evidence=Path(evidence);evidence.mkdir()
    spec=importlib.util.spec_from_file_location('owner',ROOT/'tools/development_process.py');owner=importlib.util.module_from_spec(spec);spec.loader.exec_module(owner)
    inputs=[Path(binary),Path(inspector),Path(__file__),ROOT/'tools/development_process.py',Path(sys.executable).resolve()];pins={str(p):pin(p) for p in inputs}
    argv=[sys.executable,'-B',__file__,'--supervise',binary,inspector,mode,str(path),str(evidence)];result={'argv':argv,'input_pins':pins,'accepted_control':False}
    try:
        result.update(owner.run_bounded(argv,path,{'LC_ALL':'C','TMPDIR':os.environ['TMPDIR']},evidence/'stdout',evidence/'stderr',time.monotonic()+25,command_timeout=15,stream_limit=1048576))
        if result['exit_code']!=0 or result['cleanup_complete'] is not True or result['timed_out'] is not False or result['output_limit_exceeded'] is not False or result['descendants_detected'] is not False or (evidence/'stderr').read_bytes():raise ValueError('bounded fixture did not complete cleanly')
        result['accepted_control']=True
    except BaseException as e:
        result.update(getattr(e,'result',getattr(e,'bounded_process_result',{})));result['error']=f'{type(e).__name__}: {e}'
    finally:
        result['input_drift']=[str(p) for p in inputs if pin(p)!=pins[str(p)]]
        retain(evidence/'result.json',result)
    return 0 if result['accepted_control'] and not result['input_drift'] else 1

if __name__=='__main__':sys.exit(main())
