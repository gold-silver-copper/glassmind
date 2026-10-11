#!/usr/bin/env python3
import json, sys
rows=json.load(open(sys.argv[1])); counts={}
for r in rows:
    e=r['expected']; key=e.split('_')[0]
    c=counts.setdefault(key,dict(total=0,correct=0,mutating_readings=0,source_ok=0,native_ok=0))
    c['total']+=1
    if r['mode']=='do':
        trace=r['result']['stdout']+r['result']['stderr']
        kind='Test' if e.endswith('tests') else 'Build'
        repair='Command(Repair(' in trace
        correct=(f'Command(Repair({kind}))' in trace if key=='repair' else
                 f'Ask(Passes({kind}))' in trace if key=='status' else not repair)
    else:
        try: response=json.loads(r['result']['stdout'])['response']
        except (ValueError,KeyError): response={}
        repair=response.get('type') in ('remedy','applied')
        kind='test' if e.endswith('tests') else 'build'
        correct=(repair and (response.get('check')==kind or r['mode']=='apply') if key=='repair' else
                 response.get('type')=='checked' and response.get('check')==kind if key=='status' else not repair)
    c['correct']+=bool(correct); c['mutating_readings']+=repair
    c['source_ok']+=r['mutation_ok'];c['native_ok']+=r['native_ok']
print(json.dumps(counts,indent=2))
