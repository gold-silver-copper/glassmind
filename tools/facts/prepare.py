#!/usr/bin/env python3
"""Pinned SimpleQuestions train/valid and a live, benchmark-scoped Wikidata index.
Never downloads test. Validation subject IDs seed the KB crawl, but validation
relations/objects never enter the index. Runtime must link the entity from text.
"""
import argparse, concurrent.futures, hashlib, json, time, urllib.request, urllib.error
from pathlib import Path
REV = 'df69267f1db4f85a71325e7e8c06e192bcaf0b0e'
BASE = f'https://raw.githubusercontent.com/askplatypus/wikidata-simplequestions/{REV}/'
UA = 'GlassmindResearch/0.1 (github.com/gold-silver-copper/glassmind)'
def download(url):
    for attempt in range(5):
        try:
            return urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent':UA}), timeout=90).read()
        except urllib.error.HTTPError as error:
            if attempt == 4: raise
            delay = max(60, int(error.headers.get("Retry-After", "60"))) if error.code == 429 else 2**attempt
            time.sleep(delay)
        except Exception:
            if attempt == 4: raise
            time.sleep(2**attempt)
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory',type=Path);p.add_argument('--dataset-only',action='store_true',help='fetch pinned train/valid only, without Wikidata');p.add_argument('--cached-only',action='store_true',help='build the finite snapshot from cached API responses without network access');a=p.parse_args()
    root=a.directory;root.mkdir(parents=True,exist_ok=True)
    for split in ('train','valid'):
        path=root/f'{split}.tsv'
        if a.cached_only and not path.exists():raise FileNotFoundError(path)
        if not path.exists():path.write_bytes(download(BASE+f'annotated_wd_data_{split}_answerable.txt'))
    if not (root/'LICENSE.txt').exists():(root/'LICENSE.txt').write_bytes(download(BASE+'LICENSE.txt'))
    if a.dataset_only:return
    rows=[line.split('\t') for line in (root/'valid.tsv').read_text().splitlines()]
    ids=sorted({r[0] for r in rows});cache=root/'wikidata';cache.mkdir(exist_ok=True)
    groups=[ids[i:i+25] for i in range(0,len(ids),25)]
    def fetch(group):
        name=hashlib.sha256('|'.join(group).encode()).hexdigest()[:16]+'.json'
        dest=cache/name
        if not dest.exists():
            if a.cached_only:return {}
            time.sleep(4)
            data=download('https://www.wikidata.org/w/api.php?action=wbgetentities&ids='+'%7C'.join(group)+'&props=labels%7Caliases%7Cclaims&languages=en&format=json')
            parsed=json.loads(data)
            if 'entities' not in parsed:raise ValueError('Wikidata response missing entities')
            dest.write_bytes(data)
        return json.loads(dest.read_bytes())['entities']
    entities={};files=[]
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
        for batch in pool.map(fetch,groups):
            for q,e in batch.items():
                labels=[]
                if 'en' in e.get('labels',{}):labels.append(e['labels']['en']['value'])
                labels.extend(x['value'] for x in e.get('aliases',{}).get('en',[]))
                claims={}
                for prop,statements in e.get('claims',{}).items():
                    answers=set()
                    for s in statements:
                        if s.get('rank')=='deprecated':continue
                        snak=s.get('mainsnak',{});v=snak.get('datavalue',{})
                        if v.get('type')=='wikibase-entityid':
                            target=v.get('value',{}).get('id')
                            if target and target.startswith('Q'):answers.add(target)
                    if answers:claims[prop]=sorted(answers)
                entities[q]={'aliases':sorted(set(labels)),'claims':claims}
            print(f'indexed {len(entities)}/{len(ids)} entities',flush=True)
    index=root/'index.json';index.write_text(json.dumps({'entities':entities},sort_keys=True))
    for path in sorted(cache.glob('*.json')):files.append({'file':path.name,'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    (root/'index-source.json').write_text(json.dumps({'simplequestions_revision':REV,'question_license':'CC BY 3.0','wikidata_license':'CC0','retrieved_unix':time.time(),'seed':'validation subject IDs only; cached-only mode may contain a partial subset; no validation relations or answers used to construct claims','cached_only':a.cached_only,'entities':len(entities),'index_bytes':index.stat().st_size,'index_sha256':hashlib.sha256(index.read_bytes()).hexdigest(),'wikidata_responses':files},indent=2)+'\n')
if __name__=='__main__':main()
