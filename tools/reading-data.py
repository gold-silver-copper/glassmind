#!/usr/bin/env python3
"""Make deterministic dev samples without printing corpus sentences.

BLiMP comes from the local original archive; BabyLM uses dev.zip only.
The manifest records archive hashes, sample counts and the sampling policy.
"""
import argparse
import hashlib
import heapq
import json
import zipfile
from pathlib import Path


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--blimp',type=Path,required=True)
    p.add_argument('--babylm-dev',type=Path,required=True)
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--per-group',type=int,default=20)
    args=p.parse_args()
    if args.per_group<=0:p.error('per-group must be positive')
    args.out.mkdir(parents=True,exist_ok=True)
    manifest={'sampling':'lowest SHA256 of member name and line index, per group; seed glassmind-reading-v1','sources':{}}
    for track,archive in [('grammar',args.blimp),('reading',args.babylm_dev)]:
        counts={}
        with zipfile.ZipFile(archive) as z, (args.out/(track+'.jsonl')).open('w') as out:
            for name in sorted(z.namelist()):
                if track=='grammar' and not ('/data/' in name and name.endswith('.jsonl')):continue
                if track=='reading' and not name.endswith(('.txt','.dev')):continue
                heap=[]
                with z.open(name) as f:
                    for i,line in enumerate(f):
                        text=line.decode('utf-8').strip()
                        if not text:continue
                        digest=hashlib.sha256(f'glassmind-reading-v1:{name}:{i}'.encode()).digest()
                        key=int.from_bytes(digest,'big')
                        item=(-key,i,text)
                        if len(heap)<args.per_group:heapq.heappush(heap,item)
                        elif item>heap[0]:heapq.heapreplace(heap,item)
                group=Path(name).stem
                counts[group]=len(heap)
                for _,_,text in sorted(heap,reverse=True):
                    if track=='grammar':
                        row=json.loads(text)
                        data={'group':group,'good':row['sentence_good'],'bad':row['sentence_bad']}
                    else:data={'group':group,'text':text}
                    out.write(json.dumps(data)+'\n')
        manifest['sources'][track]={'archive':str(archive),'bytes':archive.stat().st_size,
            'sha256':hashlib.file_digest(archive.open('rb'),'sha256').hexdigest(),'counts':counts,'sample_size':sum(counts.values())}
    (args.out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({k:v['sample_size'] for k,v in manifest['sources'].items()}))

if __name__=='__main__':main()
