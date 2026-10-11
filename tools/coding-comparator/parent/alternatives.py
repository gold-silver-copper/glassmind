"""Establish actual passing alternatives and distinguish their observed outputs."""
from pathlib import Path
import hashlib,json,os,subprocess
HERE=Path(__file__).resolve().parent;out=HERE/'alternatives';out.mkdir(exist_ok=False)
cases=json.loads((HERE/'cases.json').read_text());rows=[]
for c in cases:
    if 'alternatives' not in c:continue
    outputs=[]
    for i,files in enumerate(c['alternatives']):
        root=out/(c['name']+'-'+str(i));root.mkdir()
        for name,text in files.items():p=root/name;p.parent.mkdir(exist_ok=True);p.write_text(text)
        (root/'Cargo.toml').write_text('[package]\nname="probe"\nversion="0.1.0"\nedition="2024"\n[workspace]\n')
        env=dict(os.environ,TMPDIR=str(root/'tmp'),CARGO_TARGET_DIR=str(root/'target'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='4');(root/'tmp').mkdir()
        wrapper=['systemd-run','--user','--scope','--quiet','--collect','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','120','cargo']
        def run(args,label):
            r=subprocess.run(wrapper+args,cwd=root,env=env,capture_output=True,text=True,timeout=150)
            (root/(label+'.stdout')).write_text(r.stdout);(root/(label+'.stderr')).write_text(r.stderr)
            assert r.returncode==0,(c['name'],i,r.stderr,r.stdout);return r
        run(['test','--offline','-j1','--','--test-threads','4'],'native-test')
        if 'output_main' in c:
            p=root/'src/main.rs';p.write_text(p.read_text().replace('fn main() {}\n',c['output_main']))
            result=run(['run','--offline','--quiet','-j1','--bin','probe'],'observe')
        else:
            (root/'examples').mkdir();(root/'examples/observe.rs').write_text('use probe::*;\n'+c['output_example'])
            result=run(['run','--offline','--quiet','-j1','--example','observe'],'observe')
        outputs.append(result.stdout)
        assert result.stdout==c['outputs'][i],(c['name'],i,result.stdout,c['outputs'][i])
        print(c['name'],i,'passed',result.stdout.strip(),flush=True)
    assert len(set(outputs))==len(outputs)
    rows.append(dict(name=c['name'],alternatives=len(outputs),all_native_passed=True,outputs=outputs))
(out/'report.json').write_text(json.dumps(dict(cases_sha256=hashlib.sha256((HERE/'cases.json').read_bytes()).hexdigest(),cases=rows),indent=2)+'\n')
