#!/usr/bin/env python3
"""Fit value-free punctuation boundaries on public TRAIN span endpoints only."""
import argparse, collections, hashlib, json, pathlib, re

def shape(value):
    word=value.split()[-1]
    out=[];i=0
    while i<len(word):
        c=word[i];cls='A' if c.isalpha() else 'D' if c.isnumeric() else None
        if cls:
            j=i+1
            while j<len(word) and (word[j].isalpha() if cls=='A' else word[j].isnumeric()):j+=1
            out.append(cls+str(min(3,j-i)));i=j
        else:out.append(c);i+=1
    return ''.join(out)

def main():
    p=argparse.ArgumentParser()
    for n in ('train','schema','baseline','output'):p.add_argument('--'+n,type=pathlib.Path,required=True)
    a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
    ds=json.loads(a.train.read_text());schemas=json.loads(a.schema.read_text())
    allowed={s['service_name']:{q['name'] for q in s['slots'] if not q['is_categorical']} & {n for i in s['intents'] for n in i['required_slots']+list(i['optional_slots'])} for s in schemas}
    rows=collections.defaultdict(list);split_counts=collections.Counter()
    for d in ds:
        split='calibration' if int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0 else 'fit'
        split_counts[split]+=1
        for ti,t in enumerate(d['turns']):
            text=t['utterance']
            for f in t['frames']:
                for s in f.get('slots',[]):
                    if s['slot'] not in allowed.get(f['service'],set()):continue
                    start,end=s['start'],s['exclusive_end'];value=text[start:end]
                    words=list(re.finditer(r"[^\W_]+|['-]+",value))
                    if not words:continue
                    stop=start+words[-1].end()
                    punct=re.match(r"[^\w\s'-]+",text[stop:])
                    if not punct or len(punct[0])>3:continue
                    key=(shape(text[start:stop]),punct[0])
                    if len(key[0])>64:continue
                    length=end-stop
                    rows[key].append((split,d['dialogue_id'],ti,length,value,t['speaker']))
    rules=[];evidence=[]
    for (sh,punct),rs in sorted(rows.items()):
        fit=[r for r in rs if r[0]=='fit'];cal=[r for r in rs if r[0]=='calibration']
        if not fit:continue
        lengths=collections.Counter(r[3] for r in fit)
        length=lengths.most_common(1)[0][0]
        fc=len({r[1] for r in fit});cc=len({r[1] for r in cal})
        bad=sum(r[3]!=length for r in rs)
        accepted=0<length<=len(punct) and fc>=5 and cc>=3 and bad==0
        record=dict(shape=sh,punctuation=punct,extension_chars=length,fit_conversations=fc,calibration_conversations=cc,mismatches=bad,fit_rows=len(fit),calibration_rows=len(cal),distinct_values=len({r[4] for r in rs}),speakers=dict(collections.Counter(r[5] for r in rs)),accepted=accepted)
        evidence.append(record)
        if accepted:rules.append({k:record[k] for k in ('shape','punctuation','extension_chars','fit_conversations','calibration_conversations','mismatches')})
    boundary=dict(version=1,rules=rules)
    model=json.loads(a.baseline.read_text());assert 'boundaries' not in model
    model['boundaries']=boundary
    model['version']=2
    (a.output/'model.json').write_text(json.dumps(model,separators=(',',':')))
    (a.output/'boundaries.json').write_text(json.dumps(boundary,indent=2)+'\n')
    report=dict(split_counts=split_counts,training_user_turns=sum(t['speaker']=='USER' for d in ds for t in d['turns']),rules=len(rules),evidence=evidence)
    (a.output/'training.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(dict(split_counts=split_counts,rules=rules),indent=2))
if __name__=='__main__':main()
