#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
import hashlib, importlib.util, json, os, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[3]
RUN=pathlib.Path(os.environ.get('SUPPORT_RUN', ROOT.parent/'qasupport-run'/'support'))
BASE=RUN.parent/'baseline'
spec=importlib.util.spec_from_file_location('qa_run',ROOT/'tools/reading/qa/run.py'); qa=importlib.util.module_from_spec(spec); spec.loader.exec_module(qa)
RUN.mkdir(parents=True,exist_ok=True)
old=json.loads((BASE/'selection.json').read_text())
excluded=set(old['fit']['article_hashes']+old['cal']['article_hashes'])
articles=sorted((a for a in qa.load('train-v1.1.json')['data'] if qa.article_hash(a) not in excluded),key=qa.article_hash)
selection={}
for name,aa,cap in [('fit',articles[:96],8000),('cal',articles[96:128],2000)]:
 rs=sorted(qa.rows(aa),key=qa.order)[:cap]
 selection[name]={'article_hashes':[qa.article_hash(a) for a in aa],'ids':[r['id'] for r in rs],'rows':len(rs)}
 qa.lines(RUN/(name+'.input.jsonl'),[dict(context=r['context'],question=r['question']) for r in rs]); qa.write(RUN/(name+'.labels.json'),rs)
assert not set(selection['fit']['article_hashes'])&set(selection['cal']['article_hashes'])
qa.write(RUN/'selection.json',selection)
print({k:v['rows'] for k,v in selection.items()})
