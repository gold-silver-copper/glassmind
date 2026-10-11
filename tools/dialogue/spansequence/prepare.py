"""TRAIN-only offline split and supervision; never invoked by runtime."""
import argparse, hashlib, json, pathlib
p=argparse.ArgumentParser();p.add_argument('--root',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
ds=json.loads((a.root/'selected/train.json').read_text());schemas=json.loads((a.root/'train/schema.json').read_text())
selected={'RideSharing_1','Travel_1','Weather_1'}
slots={s['service_name']:sorted(x['name'] for x in s['slots'] if not x['is_categorical'] and x['name'] in {n for i in s['intents'] for n in i['required_slots']+list(i['optional_slots'])}) for s in schemas if s['service_name'] in selected}
rows=[];splits={'FIT':[],'CAL':[]}
for d in ds:
 cal=int(hashlib.sha256(d['dialogue_id'].encode()).hexdigest(),16)%5==0;splits['CAL' if cal else 'FIT'].append(d['dialogue_id']);system=''
 for t in d['turns']:
  if t['speaker']=='SYSTEM':system=t['utterance'];continue
  for f in t['frames']:
   if f['service'] not in slots or not f.get('state'):continue
   rows.append(dict(cal=cal,service=f['service'],system=system,utterance=t['utterance'],spans=[s for s in f.get('slots',[]) if s['slot'] in slots[f['service']]]))
a.output.mkdir(parents=True,exist_ok=True)
(a.output/'training.json').write_text(json.dumps(dict(slots=slots,rows=rows),ensure_ascii=False))
(a.output/'split.json').write_text(json.dumps(splits,indent=2)+'\n')
print({k:len(v) for k,v in splits.items()},slots,len(rows))
