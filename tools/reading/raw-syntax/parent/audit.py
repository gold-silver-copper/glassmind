from pathlib import Path
import collections,hashlib,importlib.util,json
r=Path('/home/gsc/.cache/glassmind/continuous-20261010');source=r/'udsyntax-source';baseline=r/'coordinator/rawsyntax-baseline';candidate=r/'rawsyntax-run/candidate';root=Path('/home/gsc/code/glassmind')
spec=importlib.util.spec_from_file_location('tree_audit',root/'tools/reading/syntax/parent/audit.py');tree=importlib.util.module_from_spec(spec);spec.loader.exec_module(tree)
gold=json.loads((baseline/'gold.json').read_text());old=[json.loads(l) for l in (baseline/'predictions.jsonl').read_text().splitlines()];new=[json.loads(l) for l in (candidate/'full-predictions.jsonl').read_text().splitlines()];assert len(gold)==len(old)==len(new)==2001
train={tuple(t[0] for t in sent) for sent in tree.corpus(source/'en_ewt-ud-train.conllu')};trans=collections.Counter();groups=collections.defaultdict(collections.Counter);spans=0;unchanged=0;changed_still_wrong=0
for g,b,c in zip(gold,old,new):
 assert b['status']==c['status']=='predicted' and c['structurally_valid'] and tree.is_tree(c['heads'],c['deprels']);assert len(c['forms'])==len(c['spans'])==len(c['heads'])==len(c['deprels'])==len(c['upos'])<=128
 oldexact=b['forms']==g['forms'];newexact=c['forms']==g['forms'];trans[(oldexact,newexact)]+=1;k='train_duplicate_forms' if tuple(g['forms']) in train else 'nonduplicate_forms';groups[k]['sentences']+=1;groups[k]['baseline_exact']+=oldexact;groups[k]['candidate_exact']+=newexact
 raw=g['text'].encode();last=0
 for t in c['spans']:
  assert type(t['start']) is int and type(t['end']) is int and last<=t['start']<t['end']<=len(raw);assert raw[t['start']:t['end']].decode()==t['form'];last=t['end'];spans+=1
 assert [t['form'] for t in c['spans']]==c['forms'];assert ''.join(c['forms'])==''.join(ch for ch in g['text'] if not ch.isspace())
 if b['forms']==c['forms']:
  unchanged+=1;assert {k:v for k,v in b.items() if k!='tokenization'}=={k:v for k,v in c.items() if k!='tokenization'}
 elif not newexact:changed_still_wrong+=1
 assert c['tokenization']=='raw_unicode_apostrophe_v2'
assert trans[(True,False)]==0 and trans[(True,True)]==1370 and trans[(False,True)]==186 and trans[(False,False)]==445
assert (candidate/'forms-predictions.jsonl').read_bytes()==(r/'rawsyntax-run/baseline/forms-predictions.jsonl').read_bytes();assert (candidate/'model.bin').read_bytes()==(r/'rawsyntax-run/baseline/model.bin').read_bytes()
result=dict(source_revision='b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c',license='CC-BY-SA-4.0',metric='Exact integer-word FORM sequence per official DEV sentence; no normalization or exclusions; not dependency/semantic accuracy',total=2001,baseline_exact=1370,candidate_exact=1556,remaining_mismatch=445,transitions={str(k):v for k,v in trans.items()},candidate_spans_valid=spans,complete_source_coverage_sentences=2001,valid_predicted_trees=2001,new_refusals=0,unchanged_sequences_with_all_other_fields_identical=unchanged,changed_sequences_still_mismatching=changed_still_wrong,groups={k:dict(v) for k,v in groups.items()},full_prediction_sha256=hashlib.sha256((candidate/'full-predictions.jsonl').read_bytes()).hexdigest(),provided_form_byte_identical_sha256=hashlib.sha256((candidate/'forms-predictions.jsonl').read_bytes()).hexdigest(),model_sha256=hashlib.sha256((candidate/'model.bin').read_bytes()).hexdigest(),frozen_full_gate_pass=True,main_integration='pending')
(Path(__file__).parent/'full-audit.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
