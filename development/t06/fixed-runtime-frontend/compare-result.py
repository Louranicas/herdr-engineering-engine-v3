#!/usr/bin/env python3
"""External development comparator. Never imported by the runtime or preparer."""
import argparse
import json
from pathlib import Path

def compare(root):
    failures=[]
    def load(name):
        try:
            value=json.loads((root/name).read_bytes())
            if not isinstance(value,dict):raise ValueError('object expected')
            return value
        except (OSError,ValueError,UnicodeError) as error:
            failures.append({'observation':name,'reason':'missing_or_invalid','detail':str(error)})
            return {}
    def fail(reason):failures.append({'reason':reason})
    frontend=load('frontend-result.json');driver=load('driver-result.json')
    runtime=load('runtime-observation.json');stored=load('store-readback.json');graphs=load('retained-graphs.json')
    if frontend.get('operation_returned_ok') is not True or frontend.get('returned_error','missing') is not None:fail('frontend returned error or incomplete observation')
    if driver.get('ok') is not True or driver.get('outcome')!='Accepted' or driver.get('error','missing') is not None:fail('driver did not accept')
    executions=runtime.get('executions')
    states=[]
    if not isinstance(executions,list):fail('execution inventory absent')
    else:
        for index,row in enumerate(executions):
            receipt=row.get('receipt') if isinstance(row,dict) else None
            if not isinstance(receipt,dict) or not isinstance(receipt.get('state'),str):
                failures.append({'reason':'receipt state absent','execution':index})
            else:states.append(receipt['state'])
    # These external expectations are never inputs to the engine.
    if states!=['FAIL','PASS_CANDIDATE']:fail('expected baseline FAIL then repaired PASS_CANDIDATE')
    head=stored.get('head');outbox=stored.get('outbox')
    if not isinstance(head,dict):fail('durable head absent');head={}
    if head.get('state')!='accepted' or not isinstance(head.get('accepted_event'),str) or head.get('cancellation') is not False:fail('durable acceptance absent')
    if head.get('reserved_work_ms')!=0 or head.get('reserved_verify_ms')!=0:fail('reserves not observed released after acceptance')
    if not isinstance(outbox,list) or len(outbox)!=1 or not isinstance(outbox[0],list) or len(outbox[0])!=3 or outbox[0][0]!=head.get('accepted_event'):fail('one matching acceptance outbox event missing')
    if stored.get('head_error','missing') is not None or stored.get('outbox_error','missing') is not None:fail('Store readback error or missing status')
    roots=graphs.get('roots')
    if not isinstance(roots,list) or len(roots)!=2 or any(not isinstance(r,dict) or 'error' in r or not isinstance(r.get('objects'),int) or r['objects']<=0 for r in roots):fail('exact receipt closure readback missing or failed')
    return {'scope':'external fixed development observation comparator; no module admission','expected_relation':'baseline FAIL -> repair PASS_CANDIDATE -> accepted once','observed_states':states,'failures':failures,'passed':not failures}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('run',type=Path);a=p.parse_args();result=compare(a.run)
    print(json.dumps(result,indent=2));raise SystemExit(0 if result['passed'] else 1)
