#!/usr/bin/env python3
"""Score frozen models on official test splits; print aggregate totals only.

Run as the coordinator, outside worker sandboxes. The output directory must be
outside the repository and included in every worker's private/hidden paths.
No test examples, predictions, task-level failures or labels are printed.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import signal
import subprocess
import tarfile
import urllib.request

CLINC_REV='828f8093932c8fe6ca7936c3d2e52903b1c523de'


def digest(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file,'sha256').hexdigest()


def run(command, payload=None):
    process=subprocess.Popen(command,stdin=subprocess.PIPE if payload is not None else subprocess.DEVNULL,
                             stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=True)
    try:
        stdout,stderr=process.communicate(payload,timeout=600)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid,signal.SIGKILL)
        process.communicate()
        raise RuntimeError('held-out evaluator timed out') from None
    if process.returncode:
        # A model's error output might contain examples: never surface it here.
        raise RuntimeError(f'held-out evaluator exited {process.returncode}')
    return stdout


def clinc(args):
    url=f'https://raw.githubusercontent.com/clinc/oos-eval/{CLINC_REV}/data/data_full.json'
    with urllib.request.urlopen(url,timeout=60) as stream:
        data=json.load(stream)
    rows=data['test']+data['oos_test']
    path=args.out/'clinc-official-test.jsonl'
    with path.open('w') as file:
        for text,label in rows:
            file.write(json.dumps({'text':text,'label':label})+'\n')
    try:
        totals=json.loads(run([str(args.everyday_binary.resolve()),'eval','--model',str(args.clinc_model.resolve()),
                               '--dev',str(path),'--per-label','0']))
    except (ValueError,KeyError):
        raise RuntimeError('invalid aggregate output from intent evaluator') from None
    return {'dataset':'CLINC150 '+CLINC_REV,'split':'heldout','totals':totals,
            'model_sha256':digest(args.clinc_model),'input_sha256':digest(path)}


def babi(args):
    items=[]
    with tarfile.open(args.babi_archive,'r:gz') as tar:
        for member in tar:
            path=PurePosixPath(member.name)
            if not (member.isfile() and len(path.parts)==3 and path.parts[0]=='tasks_1-20_v1-2'
                    and path.parts[1]=='en' and path.name.endswith('_test.txt')):
                continue
            stream=tar.extractfile(member)
            if stream is None:raise RuntimeError('unreadable test member')
            task=int(path.name.split('_')[0][2:])
            context=[]
            for raw in stream:
                number,text=raw.decode().strip().split(' ',1)
                if number=='1':context=[]
                if '\t' in text:
                    question,answer,*_=text.split('\t')
                    items.append((task,context.copy(),question,answer))
                else:context.append(text)
    if len(items)!=20000:raise RuntimeError('unexpected bAbI official test count')
    payload=''.join(json.dumps({'story':story,'question':question})+'\n' for _,story,question,_ in items)
    output=run([str(args.stories_binary.resolve()),'--jsonl','--model',str(args.stories_model.resolve())],payload)
    try:
        replies=[json.loads(line) for line in output.splitlines()]
    except ValueError:
        raise RuntimeError('invalid story evaluator output') from None
    if len(replies)!=len(items):raise RuntimeError('wrong number of story replies')
    counts=Counter()
    for (task,_,_,expected),reply in zip(items,replies,strict=True):
        got=reply.get('answer')
        normalize=lambda s: ','.join(sorted(s.strip().lower().split(','))) if task==8 else s.strip().lower()
        verdict='refusal' if got is None else 'right' if normalize(got)==normalize(expected) else 'wrong'
        counts[verdict]+=1
    return {'dataset':'bAbI 1.2 English 1k official test','split':'heldout',
            'totals':{'total':len(items),**{k:counts[k] for k in ('right','refusal','wrong')}},
            'model_sha256':digest(args.stories_model),'archive_sha256':digest(args.babi_archive)}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--track',choices=['intents','stories'],required=True)
    for name in ['everyday-binary','clinc-model','stories-binary','stories-model','babi-archive']:
        parser.add_argument('--'+name,type=Path)
    args=parser.parse_args()
    args.out=args.out.resolve()
    repo=Path(__file__).resolve().parents[1]
    if args.out.is_relative_to(repo):parser.error('held-out output must be outside repository')
    required=(args.everyday_binary,args.clinc_model) if args.track=='intents' else (args.stories_binary,args.stories_model,args.babi_archive)
    if any(p is None or not p.exists() for p in required):parser.error('supply existing binaries, model and archive for the chosen track')
    args.out.mkdir(parents=True,exist_ok=True)
    report=clinc(args) if args.track=='intents' else babi(args)
    (args.out/(args.track+'-totals.json')).write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
