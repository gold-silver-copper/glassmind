from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess

p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
source = 'pub fn function(xs: &[i64]) -> i64 { todo!() }\n#[cfg(test)] mod tests { use super::*; #[test] fn behavior() { assert_eq!(function(&[-7,2,4,9]),8); assert_eq!(function(&[-3,-2]),-5); } }\n'
refusals = [f'function {description}{action}' for description in ['needs a body', 'needs implementing']
            for action in ['', ', implement it', '. please implement function']]
refusals += ['implement function fold_values']
accepted = ['implement function', 'implement function in src/lib.rs', 'function is a stub, implement it']
cases = [dict(request=s, accept=False) for s in refusals] + [dict(request=s, accept=True) for s in accepted]
a.out.mkdir(parents=True, exist_ok=False)
(a.out/'cases.json').write_text(json.dumps(cases, indent=2)+'\n')
rows = []
for i, case in enumerate(cases):
    folder = a.out / str(i)
    root = folder/'project'
    (root/'src').mkdir(parents=True)
    (root/'Cargo.toml').write_text('[package]\nname="literal_function"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
    (root/'src/lib.rs').write_text(source)
    env = dict(os.environ)
    env.update(TMPDIR=str(folder/'tmp'), CARGO_TARGET_DIR=str(folder/'target'),
               GLASSMIND_SCRATCH=str(folder/'scratch'))
    Path(env['TMPDIR']).mkdir()
    def snapshot():
        return {str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest()
                for f in root.rglob('*') if f.is_file()}
    before = snapshot()
    result = subprocess.run([str(a.binary.resolve()), 'do', str(root), case['request']],
                            text=True, capture_output=True, env=env, timeout=180)
    (folder/'action.stdout').write_text(result.stdout)
    (folder/'action.stderr').write_text(result.stderr)
    after = snapshot()
    if case['accept']:
        assert 'Applied test-verified proposal' in result.stdout, result.stdout
        updated = (root/'src/lib.rs').read_text()
        prefix, suffix = source.split('todo!()')
        assert updated.startswith(prefix) and updated.endswith(suffix) and updated != source
        assert set(before) == set(after)
        assert all(before[k] == after[k] for k in before if k != 'src/lib.rs')
        checked = folder/'verify'
        shutil.copytree(root, checked)
        check = subprocess.run(['systemd-run','--user','--scope','--quiet','--collect',
            '-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s',
            '120','cargo','test','--offline','-j1','--','--test-threads','4'],
            cwd=checked, env=env, text=True, capture_output=True, timeout=150)
        (folder/'native.log').write_text(check.stdout+check.stderr)
        assert check.returncode == 0 and '1 passed' in check.stdout
    else:
        assert before == after
        assert 'unsupported or unconsumed implementation request' in result.stdout
    rows.append(dict(request=case['request'], accepted=case['accept'], exit=result.returncode,
                     before=before, after=after))
    (a.out/'report.json').write_text(json.dumps(rows, indent=2)+'\n')
print(json.dumps(dict(total=len(rows), implemented=sum(r['accepted'] for r in rows),
                      refused=len(refusals))))
