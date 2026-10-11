#!/usr/bin/env python3
"""One frozen candidate, fixed first100 then full2001; unchanged independent scorer."""
import collections, hashlib, importlib.util, json, os, pathlib, shutil, subprocess, sys
P=pathlib.Path
ROOT=P(__file__).resolve().parents[3]
RUN=ROOT.parent/'graphsyntax-run'
SOURCE=ROOT.parent/'udsyntax-source'
OWN=ROOT/'tools/reading/syntax-graph'
BASE=RUN/'baseline'
OUT=RUN/'candidate'
OUT.mkdir(exist_ok=True)
def write(p,v):p.write_text(json.dumps(v,indent=2)+'\n')
def identity(p):return {'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def call(args, name):
    with (OUT/(name+'.log')).open('w') as f:
        result=subprocess.run([sys.executable,str(ROOT/'tools/reading/syntax/measure.py'),str(OUT/(name+'-resources.json')),'setsid','--wait','timeout','--kill-after=10s','600',*map(str,args)],stdout=f,stderr=subprocess.STDOUT)
    if result.returncode:raise RuntimeError((name,result.returncode))
def predict(binary,model,inputs,outputs,name):call([binary,'predict','--model',model,'--input',inputs,'--output',outputs],name)
def read(p):return [json.loads(x) for x in p.read_text().splitlines()]
spec=importlib.util.spec_from_file_location('audit',ROOT/'tools/reading/syntax/parent/audit.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
for name,ident in json.loads((BASE/'manifest.json').read_text()).items():assert identity(BASE/name)==ident
if sys.argv[1]=='prepare':
    train=audit.corpus(SOURCE/'en_ewt-ud-train.conllu')
    ledger=[];nonproj=0;eligible=0;words=0
    for i,s in enumerate(train):
        heads=[x[2] for x in s];labs=[x[3] for x in s]
        reason=None
        if len(s)>128:reason='overlength'
        elif not audit.is_tree(heads,labs):reason='invalid_tree'
        if reason:ledger.append(dict(sentence=i,words=len(s),reason=reason));continue
        eligible+=1;words+=len(s)
        projective=True
        for d,h in enumerate(heads,1):
            for k in range(min(d,h)+1,max(d,h)):
                a=k
                for _ in s:
                    if a in (0,h):break
                    a=heads[a-1]
                if a!=h:projective=False
        nonproj+=not projective
    write(OUT/'eligibility.json',dict(total=len(train),eligible=eligible,words=words,nonprojective_included=nonproj,skips=ledger))
    write(OUT/'provenance.json',dict(version='EWT r2.18',revision='b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c',license='CC-BY-SA-4.0',downloads=0,files={str(p):identity(p) for p in [SOURCE/'en_ewt-ud-train.conllu',SOURCE/'en_ewt-ud-dev.conllu',SOURCE/'LICENSE.txt']}))
    sys.exit()
if sys.argv[1]=='evaluate':
    assert not (RUN/'FREEZE.json').exists()
    shutil.copy2(P(os.environ['CARGO_TARGET_DIR'])/'release/reading-syntax',OUT/'reading-syntax')
    for p in [ROOT/'crates/reading/src/syntax.rs',ROOT/'crates/reading/src/syntax_graph.rs',ROOT/'crates/reading/src/bin/reading-syntax.rs']:shutil.copy2(p,OUT/p.name)
    (OUT/'first100.jsonl').write_bytes(b''.join((BASE/'forms.jsonl').read_bytes().splitlines(keepends=True)[:100]))
    files=[*OWN.iterdir(),OUT/'model.bin',OUT/'reading-syntax',OUT/'syntax.rs',OUT/'syntax_graph.rs',OUT/'reading-syntax.rs',ROOT/'tools/reading/syntax/parent/audit.py',ROOT/'tools/reading/syntax/measure.py',*BASE.iterdir(),OUT/'first100.jsonl',OUT/'provenance.json',OUT/'eligibility.json']
    write(RUN/'FREEZE.json',{str(p):identity(p) for p in files if p.is_file()})
    binary=OUT/'reading-syntax';model=OUT/'model.bin'
    predict(binary,model,OUT/'first100.jsonl',OUT/'first100-predictions.jsonl','sample')
    dev=audit.corpus(SOURCE/'en_ewt-ud-dev.conllu')
    def scores(golds,preds):
        uas=las=words=0
        for g,p in zip(golds,preds):
            assert audit.is_tree(p['heads'],p['deprels'])
            for j,x in enumerate(g):
                words+=1;u=p['heads'][j]==x[2];uas+=u;las+=u and p['deprels'][j]==x[3]
        return dict(words=words,uas=uas,las=las)
    write(OUT/'sample-score.json',scores(dev[:100],read(OUT/'first100-predictions.jsonl')))
    predict(binary,model,BASE/'forms.jsonl',OUT/'full-predictions.jsonl','full')
    call([sys.executable,ROOT/'tools/reading/syntax/parent/audit.py','--source',SOURCE,'--predictions',OUT/'full-predictions.jsonl','--out',OUT/'audit'],'audit')
    old=read(BASE/'forms-predictions.jsonl');new=read(OUT/'full-predictions.jsonl')
    train=audit.corpus(SOURCE/'en_ewt-ud-train.conllu');seen={tuple(x[0] for x in s) for s in train}
    counters=collections.defaultdict(collections.Counter)
    with (OUT/'edge-transitions.jsonl').open('w') as ef,(OUT/'sentence-transitions.jsonl').open('w') as sf:
        for i,(g,a,b) in enumerate(zip(dev,old,new)):
            dup=tuple(x[0] for x in g) in seen
            ex={m:[True,True] for m in ['uas','las']}
            for j,x in enumerate(g):
                row=dict(sentence=i,token=j+1,duplicate=dup,old=[a['heads'][j],a['deprels'][j]],new=[b['heads'][j],b['deprels'][j]],gold=[x[2],x[3]])
                for m in ex:
                    bits=[int(p['heads'][j]==x[2] and (m=='uas' or p['deprels'][j]==x[3])) for p in [a,b]]
                    row[m]=bits
                    ex[m]=[v and bool(z) for v,z in zip(ex[m],bits)]
                    for group in ['all','duplicate' if dup else 'nonduplicate']:counters[group][m+'_'+''.join(map(str,bits))]+=1
                ef.write(json.dumps(row)+'\n')
            sf.write(json.dumps(dict(sentence=i,duplicate=dup,exact=ex))+'\n')
            for m,bits in ex.items():
                for group in ['all','duplicate' if dup else 'nonduplicate']:counters[group]['exact_'+m+'_'+''.join(str(int(x)) for x in bits)]+=1
    write(OUT/'transitions.json',dict(counters))
    for mode,inp,expected in [('forms','forms.jsonl','forms-predictions.jsonl'),('raw','raw-inputs.jsonl','raw-predictions.jsonl')]:
        output=OUT/('legacy-'+mode+'.jsonl');predict(binary,BASE/'model.bin',BASE/inp,output,'legacy-'+mode)
        assert output.read_bytes()==(BASE/expected).read_bytes()
    predict(binary,model,BASE/'raw-inputs.jsonl',OUT/'raw-predictions.jsonl','raw')
    for inp,p in zip(read(BASE/'raw-inputs.jsonl'),read(OUT/'raw-predictions.jsonl')):
        assert p['status']=='predicted' and audit.is_tree(p['heads'],p['deprels'])
        text=inp['text'].encode();end=0
        for span in p['spans']:
            assert text[span['start']:span['end']].decode()==span['form']
            assert not text[end:span['start']].decode().strip();end=span['end']
        assert not text[end:].decode().strip()
    probes=read(OWN/'authored.jsonl');authored={}
    for name,b,m in [('old',BASE/'reading-syntax',BASE/'model.bin'),('new',binary,model)]:
        predict(b,m,OWN/'authored-inputs.jsonl',OUT/(name+'-authored.jsonl'),name+'-authored')
        authored[name]=read(OUT/(name+'-authored.jsonl'))
    counts=collections.Counter()
    for g,a,b in zip(probes,authored['old'],authored['new']):
        for j,(h,l) in enumerate(zip(g['heads'],g['labels'])):
            bits=[int(p['heads'][j]==h and p['deprels'][j]==l) for p in [a,b]]
            counts[''.join(map(str,bits))]+=1
    write(OUT/'authored-score.json',dict(counts))
    write(OUT/'compatibility.json',dict(legacy_forms_byteidentical=2001,legacy_raw_byteidentical=2001,graph_raw_valid_spans=2001))
    print(json.dumps(json.loads((OUT/'audit/score.json').read_text()),indent=2))
    # Runtime row contracts are distinct from corpus accuracy.
    contracts=[{}, {'forms':[]}, {'forms':['x']*129}, {'text':'x'*16385}, {'forms':['a b']}, {'forms':['a'], 'heads':[0]}, {'forms':['a'],'text':'a'}, {'text':'a\u0000b'}, {'forms':['x']*128}, {'text':'Élodie can’t leave.'}]
    (OUT/'contracts.jsonl').write_text(''.join(json.dumps(x)+'\n' for x in contracts))
    for name,m in [('v1',BASE/'model.bin'),('v2',model)]:
        predict(binary,m,OUT/'contracts.jsonl',OUT/(name+'-contracts.jsonl'),name+'-contracts')
        preds=read(OUT/(name+'-contracts.jsonl'))
        assert all(p['status']=='refused' for p in preds[:8])
        assert all(p['status']=='predicted' for p in preds[8:])
    (OUT/'oversized-row.jsonl').write_text(json.dumps({'text':'x'*131073})+'\n')
    result=subprocess.run(['setsid','--wait','timeout','--kill-after=10s','600',str(binary),'predict','--model',str(model),'--input',str(OUT/'oversized-row.jsonl'),'--output',str(OUT/'oversized-output.jsonl')],capture_output=True)
    (OUT/'oversized-stderr.txt').write_bytes(result.stderr)
    assert result.returncode and b'JSONL row exceeds128KiB' in result.stderr
