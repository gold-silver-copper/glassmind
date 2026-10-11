from pathlib import Path
import collections,hashlib,importlib.util,json,struct
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];RUN=ROOT/'graphsyntax-run';F=RUN/'frozen-candidate'
spec=importlib.util.spec_from_file_location('unchanged_audit',Path('/home/gsc/code/glassmind/tools/reading/syntax/parent/audit.py'));audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
train=audit.corpus(ROOT/'udsyntax-source/en_ewt-ud-train.conllu');dev=audit.corpus(ROOT/'udsyntax-source/en_ewt-ud-dev.conllu')
old=[json.loads(s) for s in (F/'baseline/forms-predictions.jsonl').read_text().splitlines()];new=[json.loads(s) for s in (F/'candidate/full-predictions.jsonl').read_text().splitlines()]
seen={tuple(w[0] for w in sent) for sent in train};groups=collections.defaultdict(collections.Counter)
for gold,b,c in zip(dev,old,new):
    duplicate=tuple(w[0] for w in gold) in seen;correct={k:[[],[]] for k in ['uas','las']}
    for i,w in enumerate(gold):
        for j,r in enumerate([b,c]):
            correct['uas'][j].append(r['heads'][i]==w[2]);correct['las'][j].append(r['heads'][i]==w[2] and r['deprels'][i]==w[3])
    for kind,(bc,cc) in correct.items():
        for group in ['all','duplicate' if duplicate else 'nonduplicate']:
            groups[group][f'exact_{kind}_{int(all(bc))}{int(all(cc))}']+=1
            for bv,cv in zip(bc,cc):groups[group][f'{kind}_{int(bv)}{int(cv)}']+=1
assert {k:dict(v) for k,v in groups.items()}==json.loads((F/'candidate/transitions.json').read_text())
def tagger(path):
    b=path.read_bytes();assert b[:8]==b'GMSYN001';length=struct.unpack_from('<I',b,8)[0];h=json.loads(b[12:12+length]);start=12+length
    buckets,classes,count=struct.unpack_from('<III',b,start);end=start+12+8*count
    assert classes==len(h['tags']);return h,b[start:end]
h1,t1=tagger(F/'baseline/model.bin');h2,t2=tagger(F/'candidate/model.bin');assert t1==t2 and h1['tags']==h2['tags'] and h1['labels']==h2['labels']
assert h1['version']=='ewt-arc-standard-v1' and h2['version']=='ewt-labeled-graph-v2'
# Legacy predictions are retained exact byte streams, not re-scored approximations.
for name in ['forms','raw']:
    assert (F/f'candidate/legacy-{name}.jsonl').read_bytes()==(F/f'baseline/{name}-predictions.jsonl').read_bytes()
rollback=json.loads((RUN/'terminal-originals/worktree/tools/reading/syntax-graph/rollback.json').read_text())
for name,e in rollback.items():
    for base in [RUN/'terminal-originals/worktree',Path('/home/gsc/code/glassmind')]:
        b=(base/name).read_bytes();assert len(b)==e['bytes'] and hashlib.sha256(b).hexdigest()==e['sha256']
assert not (RUN/'terminal-originals/worktree/crates/reading/src/syntax_graph.rs').exists()
freeze=json.loads((F/'FREEZE.json').read_text())
for name,e in freeze.items():
    original=Path(name)
    if str(original).startswith(str(ROOT/'graphsyntax/crates/reading/')):path=F/'candidate'/original.name
    elif original.is_relative_to(ROOT/'graphsyntax'):
        rel=original.relative_to(ROOT/'graphsyntax')
        path=Path('/home/gsc/code/glassmind')/rel if str(rel) in {'tools/reading/syntax/parent/audit.py','tools/reading/syntax/measure.py'} else RUN/'terminal-originals/worktree'/rel
    else:path=original
    assert path.is_relative_to(ROOT) or path in {Path('/home/gsc/code/glassmind/tools/reading/syntax/parent/audit.py'),Path('/home/gsc/code/glassmind/tools/reading/syntax/measure.py')},path
    b=path.read_bytes();assert len(b)==e['bytes'] and hashlib.sha256(b).hexdigest()==e['sha256'],path
report=dict(full_sentences=len(dev),full_words=sum(map(len,dev)),all_transition_counts_agree=True,tagger_bytes=len(t1),tagger_sha256=hashlib.sha256(t1).hexdigest(),legacy_forms_and_raw_bytes_identical=True,rollback_matches_root=True,frozen_identities_verified=len(freeze),trained_model_bytes=(F/'candidate/model.bin').stat().st_size,corpus_gate=False,parent_core_gate=False)
(HERE/'evidence-review.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
