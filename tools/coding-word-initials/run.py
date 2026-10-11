#!/usr/bin/env python3
import hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path
HERE=Path(__file__).resolve().parent
SOURCE=Path('/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-source')


def hashes(path):
    return {str(p.relative_to(path)):hashlib.sha256(p.read_bytes()).hexdigest() for p in path.rglob('*') if p.is_file()}


def main():
    mode,binary,destination=sys.argv[1:];binary=str(Path(binary).resolve());out=Path(destination).resolve();out.mkdir()
    if mode in ('baseline','candidate'):rows=json.loads((HERE/'native.json').read_text())
    elif mode=='old-native':rows=json.loads((HERE.parent/'coding-implementation-grammar/native.json').read_text())
    else:
        assert mode in ('train-sample','train-full')
        rows=json.loads((SOURCE/'requests.json').read_text())
        if mode=='train-sample':rows=[r for r in rows if r['source'] in ('rb-implement-sum-even','rb-implement-reverse-words','rb-implement-initials')]
    started=time.monotonic();results=[]
    for row in rows:
        root=out/row['id'].replace('/','_').replace('#','_');(root/'src').mkdir(parents=True)
        if 'source' in row:
            for rel in ['Cargo.toml','src/lib.rs']:shutil.copy2(SOURCE/'sources'/row['source']/rel,root/rel)
        else:
            (root/'Cargo.toml').write_text('[package]\nname="word_join_contract"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
            (root/'src/lib.rs').write_text(row['code'])
        before=hashes(root);original=(root/'src/lib.rs').read_text()
        env=dict(os.environ,GLASSMIND_SCRATCH=str(out/'scratch'),TMPDIR=str(out.parent/'tmp'),CARGO_TARGET_DIR=str(out.parent/'native-target'))
        result=subprocess.run([binary,'do',str(root),row['text']],cwd=root,env=env,capture_output=True,text=True,timeout=120)
        accepted='Applied test-verified proposal' in result.stdout;after=hashes(root)
        assert result.returncode==0,(row['id'],result.stdout,result.stderr)
        if mode in ('baseline','candidate'):assert accepted==row[mode],(row['id'],accepted,result.stdout)
        elif mode=='old-native':assert accepted==row['accept'],(row['id'],result.stdout)
        if mode=='candidate' and row['id']=='old-new-ambiguity':
            assert '2 passing candidates' in result.stdout,(row['id'],result.stdout)
        if not accepted:assert before==after,(row['id'],'unexpected modification')
        else:
            assert set(before)==set(after) and all(before[k]==after[k] for k in before if k!='src/lib.rs')
            marker='unimplemented!()' if 'unimplemented!()' in original else 'todo!()'
            left,right=original.split(marker,1);now=(root/'src/lib.rs').read_text();assert now.startswith(left) and now.endswith(right)
        entry=dict(id=row['id'],accepted=accepted,before=before,after=after,stdout=result.stdout,stderr=result.stderr)
        results.append(entry);(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
        print(json.dumps(dict(id=row['id'],accepted=accepted,seconds=time.monotonic()-started)),flush=True)
    (out/'report.json').write_text(json.dumps(dict(mode=mode,total=len(rows),accepted=sum(r['accepted'] for r in results),seconds=time.monotonic()-started,binary_sha256=hashlib.sha256(Path(binary).read_bytes()).hexdigest()),indent=2)+'\n')


if __name__=='__main__':main()
