"""Public CLI contracts: expectations authored before implementation, graph checked independently."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
root=Path(__file__).parent; cases=json.loads((root/'cases.json').read_text()); graph=json.loads((root/'graph.json').read_text())['entities']
a.output.mkdir(parents=True,exist_ok=True)
for count in (12,len(cases)):
    results=[]
    for i,c in enumerate(cases[:count]):
        # NUL cannot be represented in argv; it is exercised by native library tests.
        if '\x00' in c['question']: continue
        r=subprocess.run([a.binary,'verify','--index',str(root/'graph.json'),c['question']],capture_output=True,text=True,timeout=10)
        assert r.returncode==0,(i,r.stderr)
        got=json.loads(r.stdout);assert got['status']==c['status'],(i,c,got)
        if c['status']=='supported':
            s,p,o=c['edge'];assert o in graph[s]['claims'][p]
            w=got['witness'];assert [w['subject'],w['property'],w['object']]==c['edge'],(i,c,got)
            assert w['source_url']==f'https://www.wikidata.org/wiki/{s}#{p}'
            assert got['scope']=='local_snapshot'
        results.append({'case':i,'result':got})
    (a.output/f'contracts-{count}.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n')
    print(f'{len(results)}/{len(results)} CLI contracts passed (prefix {count})')
