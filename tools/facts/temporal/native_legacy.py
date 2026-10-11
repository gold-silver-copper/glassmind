"""Direct native legacy commands; additional byte checks, not a changed scorer."""
import json,subprocess,sys
from pathlib import Path
run=Path(sys.argv[1]);data=Path(sys.argv[2]);here=Path(__file__).resolve().parent
old=run/'baseline/glassmind-facts';new=run/'target/debug/glassmind-facts';checks=[]
def compare(args):
 a=subprocess.run([str(old),*args],capture_output=True,check=True,timeout=60).stdout
 b=subprocess.run([str(new),*args],capture_output=True,check=True,timeout=60).stdout
 assert a==b,(args,a,b)
 checks.append(dict(args=args,output=a.decode(),byte_identical=True))
for q in sorted(set(c['question'] for c in json.loads((here/'contracts.json').read_text()))):compare(['verify','--index',str(here/'index.json'),q])
for q in ['Who worked for Q2 and was a citizen of Q3?','Who was a citizen of Q3 and worked for Company?','Who was born in Q2 and worked for Q3?','Who worked for Shared and was a citizen of Q3?','invalid','x'*4097]:compare(['match','--index',str(here/'index.json'),q])
# Direct old/new Ask validates the pre-edit diagnostic predictor's actual binary semantics.
rows=(data/'valid.tsv').read_text().splitlines()[:8]
predictions=(run/'baseline/predictions.jsonl').read_bytes().splitlines(keepends=True)
for row,pred in zip(rows,predictions):
 q=row.split('\t',3)[3];compare(['ask','--model',str(data/'relation.json'),'--index',str(data/'index.json'),q]);assert checks[-1]['output'].encode()==pred
(run/'reports/native-legacy.json').write_text(json.dumps(checks,indent=2)+'\n');print(len(checks),'native legacy byte checks passed')
