from pathlib import Path
import argparse, collections, json, re, string

p=argparse.ArgumentParser()
p.add_argument('--predictions',type=Path,required=True)
p.add_argument('--out',type=Path,required=True)
a=p.parse_args();root=Path(__file__).parent
cases=json.loads((root/'cases.json').read_text())
pred=[json.loads(x) for x in a.predictions.read_text().splitlines()]
assert len(cases)==len(pred)==20
def normalize(s):
    s=''.join(c for c in s.lower() if c not in string.punctuation)
    return ' '.join(re.sub(r'\b(a|an|the)\b',' ',s).split())
rows=[];counts=collections.Counter()
for c,prediction in zip(cases,pred):
    selected=prediction['status']=='predicted'
    span=prediction['proposal']
    raw=c['context'].encode()[span['start']:span['end']].decode() if span else ''
    assert not selected or prediction['answer']==raw
    assert selected or prediction['answer']==''
    supported=c['answer'] is not None
    raw_correct=supported and normalize(raw)==normalize(c['answer'])
    selected_correct=selected and raw_correct
    counts['positive_cases' if supported else 'negative_cases']+=1
    counts['raw_positive_correct']+=raw_correct
    counts['selected_positive_correct']+=selected_correct
    counts['wrong_selected']+=selected and not selected_correct
    counts['negative_selected']+=selected and not supported
    counts['refused']+=not selected
    rows.append(dict(name=c['name'],supported=supported,raw=raw,selected=selected,raw_correct=raw_correct,selected_correct=selected_correct,reason=prediction['reason']))
result=dict(counts=dict(counts),rows=rows,scope='Authored probes; unsupported questions require refusal. Raw guesses and selected wrong proposals are separate.')
a.out.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n')
print(json.dumps(result['counts'],indent=2))
