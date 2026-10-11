"""Additional review contract, frozen before its first execution (after candidate1)."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('binary');p.add_argument('output',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
lines=['Elin went to the kitchen.','how are you Elin teleported to the attic','Where is Elin?']
cmd=[a.binary,'talk','--json','--model','/home/gsc/.cache/glassmind/models/everyday/intent.json','--session',str(a.output/'session.json')]
(a.output/'session.json').unlink(missing_ok=True)
r=subprocess.run(cmd,input='\n'.join(lines)+'\n',text=True,capture_output=True,check=True,timeout=60);(a.output/'replies.jsonl').write_text(r.stdout)
replies=[json.loads(l) for l in r.stdout.splitlines()];report=dict(checks=[replies[1]['kind']=='refusal',replies[2].get('answer') is None],inputs=lines)
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
