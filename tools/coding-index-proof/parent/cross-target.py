from pathlib import Path
import argparse, json, os, subprocess

p=argparse.ArgumentParser()
p.add_argument('--binary',type=Path,required=True)
p.add_argument('--out',type=Path,required=True)
a=p.parse_args();a.out.mkdir(parents=True,exist_ok=False)
env=dict(os.environ);env.pop('GLASSMIND_CHECK_CACHE',None)
env['CARGO_BUILD_TARGET']='wasm32-unknown-unknown'
rows=[]
for ty, expected in [('i16',True),('i32',True),('i64',False),('i128',False)]:
    root=a.out/ty;(root/'src').mkdir(parents=True)
    (root/'Cargo.toml').write_text('[package]\nname="cross_width"\nversion="0.1.0"\nedition="2024"\n')
    text='pub fn lookup(xs: &[u8], rank: TYPE) -> Option<u8> { if rank < 1 || rank as usize > xs.len() { return None; } let index = rank - 1; Some(xs[index]) }\n'.replace('TYPE',ty)
    (root/'src/lib.rs').write_text(text)
    def run(args):
        q=subprocess.run(args,cwd=root,text=True,capture_output=True,env=env,timeout=90)
        return dict(exit=q.returncode,stdout=q.stdout,stderr=q.stderr)
    before=run(['cargo','check','-j1','--message-format=json'])
    assert before['exit'] and 'E0277' in before['stdout']
    result=run([str(a.binary.resolve()),'do',str(root.resolve()),'fix the build','--explain'])
    after=(root/'src/lib.rs').read_text();changed=after!=text
    assert changed==expected,(ty,result)
    assert after==(text.replace('xs[index]','xs[(index) as usize]') if expected else text)
    post=run(['cargo','check','-j1','--message-format=json'])
    assert (post['exit']==0)==expected
    rows.append(dict(type=ty,expected_edit=expected,changed=changed,before=before,result=result,post=post))
    print(ty,'edited' if changed else 'refused',flush=True)
(a.out/'report.json').write_text(json.dumps(dict(target=env['CARGO_BUILD_TARGET'],native_execution=False,scope='Actual compiler check for installed 32-bit wasm target; this does not execute wasm behavior.',cases=rows),indent=2)+'\n')
