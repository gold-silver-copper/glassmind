import hashlib,json,math,mmap,struct,subprocess
from collections import Counter,defaultdict
from pathlib import Path
W=Path('/home/gsc/.cache/glassmind/continuous-20261010')
R=W/'wordgrammar-run/experiment'; O=W/'coordinator/wordgrammar-review'
model=R/'word3.bin';binary=R/'reading-word-probability'
frozen=json.loads((R/'frozen.json').read_text())
for p in [model,binary]:
 b=p.read_bytes(); actual={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()};assert actual==frozen['artifacts'][str(p)],str(p)
data=Path('/home/gsc/.cache/glassmind/grow-fast-20261010/reading-data/grammar.jsonl')
assert hashlib.sha256(data.read_bytes()).hexdigest()=='37ad0a1eba3a9cd9c5eae850973c376b5e32a7968ecb95833eb2d32115f6304b'
rows=[json.loads(l) for l in data.read_text().splitlines()]
saved=[json.loads(l) for l in (R/'full-costs.jsonl').read_text().splitlines()]
assert len(rows)==len(saved)==1340
# Independent arithmetic on persisted counts. Only fixed sparse dev positions
# and four new strings, no candidate tuning or repeated full inference.
chosen=[0,19,233,500,999,1339]
texts=[rows[i][k] for i in chosen for k in ['good','bad']]+['','Élan can\'t jump!','\tA   cat\n','quuxxyzzy 🐙']
NONE=2**32-1;BOS=2;EOS=0

def tokenize(text):
 out=[];word=''
 for c in text:
  if c.isalnum():word+=c.lower()
  else:
   if word:out.append(word);word=''
   if not c.isspace():out.append(c)
 if word:out.append(word)
 return out
with model.open('rb') as f:
 m=mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ)
 assert m[:8]==b'GMWRD003';docs,tokens,vocabn,entries=struct.unpack_from('<4Q',m,8);at=40
 vocab=[]
 for _ in range(vocabn):
  n=struct.unpack_from('<I',m,at)[0];at+=4;vocab.append(m[at:at+n].decode());at+=n
 assert vocab==sorted(set(vocab)); ids={s:i+3 for i,s in enumerate(vocab)}
 sequences=[[ids.get(s,1) for s in tokenize(t)]+[EOS] for t in texts]
 contexts={(NONE,NONE)};keys=set()
 for sequence in sequences:
  a=b=BOS
  for c in sequence:
   contexts.update([(NONE,b),(a,b)]);keys.update([(NONE,NONE,c),(NONE,b,c),(a,b,c)]);a,b=b,c
 counts={};totals=defaultdict(lambda:[0,0]);orders=[0,0,0];last=None
 assert len(m)-at==entries*20
 for a,b,c,n in struct.iter_unpack('<IIIQ',memoryview(m)[at:]):
  key=(a,b,c);assert last is None or last<key;last=key
  assert n>0 and c!=BOS and c<vocabn+3
  if key in keys:counts[key]=n
  if (a,b) in contexts:totals[a,b][0]+=n;totals[a,b][1]+=1
  orders[0 if b==NONE else 1 if a==NONE else 2]+=n
 assert orders==[tokens+docs]*3
 results=[]
 for index,(text,sequence) in enumerate(zip(texts,sequences)):
  a=b=BOS;cost=0.0
  for c in sequence:
   p=(counts.get((NONE,NONE,c),0)+0.5)/(tokens+docs+0.5*(vocabn+2))
   for x,y in [(NONE,b),(a,b)]:
    n,t=totals[x,y]
    if n:p=(counts.get((x,y,c),0)+t*p)/(n+t)
   assert 0<p<=1;cost-=math.log2(p);a,b=b,c
  if index<12:
   row=chosen[index//2];side='good' if index%2==0 else 'bad';actual=saved[row]['candidate_'+side+'_token_bits'];source={'dev_row':row+1,'side':side}
  else:
   p=subprocess.run([str(binary),'score','--model',str(model),'--text',text],text=True,capture_output=True,check=True,timeout=60)
   actual=json.loads(p.stdout)['token_bits_including_eos'];source={'fresh_text':text}
  assert abs(cost-actual)<1e-10,(source,cost,actual)
  results.append(dict(source=source,independent_bits=cost,saved_or_rust_bits=actual))
transitions=Counter();groups=defaultdict(Counter)
for i,(row,s) in enumerate(zip(rows,saved),1):
 assert s['row']==i and s['id']==row.get('id') and s['group']==row['group']
 for prefix,g,b in [('baseline','baseline_good_bits','baseline_bad_bits'),('candidate','candidate_good_token_bits','candidate_bad_token_bits')]:
  assert math.isfinite(s[g]) and math.isfinite(s[b]);choice='tie' if abs(s[g]-s[b])<=1e-9 else 'good' if s[g]<s[b] else 'bad';assert choice==s[prefix+'_choice']
 transitions[s['baseline_choice']+' -> '+s['candidate_choice']]+=1;groups[row['group']][s['candidate_choice']]+=1
report={'checks':len(results),'count_orders':orders,'documents':docs,'tokens':tokens,'vocabulary':vocabn,'entries':entries,'model_sha256':hashlib.sha256(model.read_bytes()).hexdigest(),'sampled_cost_checks':results,'transitions':dict(transitions),'rows':len(rows),'groups':len(groups),'scope':'Independent persisted-count Witten-Bell arithmetic on 12 saved costs and 4 fresh strings; all saved decisions/ordering/finite values audited. No retraining or repeated full inference.'}
(O/'independent.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ['checks','rows','groups','transitions']}))
