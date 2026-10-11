#!/usr/bin/env python3
"""Simple independently authored native behavior and exact-source checks."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess
p = argparse.ArgumentParser()
p.add_argument('--binary', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args(); a.output.mkdir(parents=True, exist_ok=True)
assert os.environ['TMPDIR'].startswith('/home/')
assert os.environ['CARGO_TARGET_DIR']
prefix = ['systemd-run','--user','--scope','-pMemoryMax=8G','-pMemorySwapMax=0','setsid','--wait','timeout','--kill-after=10s','600']
def run(args, root):
    r = subprocess.run(prefix + args, cwd=root, capture_output=True, text=True, timeout=620)
    return dict(code=r.returncode, out=r.stdout, err=r.stderr)
source = '''// tally remains in this comment.
fn tally(n: u32) -> u32 { n * 3 + 1 }
fn unrelated() -> &'static str { "tally stays in this string" }
fn main() { let value = tally(7); println!("{} {}", value, unrelated()); }
#[cfg(test)] mod tests {
    #[test] fn arithmetic() { let value = super::tally(4); assert_eq!(value, 13); }
}
'''
expected = source.replace('fn tally(', 'fn count_items(').replace('= tally(', '= count_items(').replace('super::tally(', 'super::count_items(')
requests = [
    'Rename function tally to count_items everywhere (definition, callers, tests).',
    'rename tally -> count_items everywhere pls, all callers + tests too',
    "The function tally processes a value, so I'd like it called count_items instead. Could you rename it everywhere it's used? Behavior should stay exactly the same.",
]
rows = []
for i, text in enumerate(requests):
    for mode in ['ask', 'do']:
        root = a.output / f'{i}-{mode}'; (root/'src').mkdir(parents=True, exist_ok=True)
        (root/'Cargo.toml').write_text('[package]\nname="rename_language_native"\nversion="0.0.0"\nedition="2021"\n[workspace]\n')
        (root/'src/main.rs').write_text(source)
        before = run(['cargo', 'run', '--quiet', '-j1'], root)
        tests_before = run(['cargo','test','--quiet','-j1','--','--test-threads','4'],root)
        args = [str(a.binary.resolve()), 'ask', '--source', str(root.resolve()), '--apply', '--explain', text] if mode == 'ask' else [str(a.binary.resolve()), 'do', str(root.resolve()), '--explain', text]
        action = run(args, root)
        after = run(['cargo','run','--quiet','-j1'],root)
        tests_after = run(['cargo','test','--quiet','-j1','--','--test-threads','4'],root)
        actual = (root/'src/main.rs').read_text()
        row = dict(text=text,mode=mode,before=before,tests_before=tests_before,action=action,after=after,tests_after=tests_after,exact_source=actual==expected)
        row['pass'] = all(x['code']==0 for x in [before,tests_before,action,after,tests_after]) and before['out']==after['out'] and actual==expected
        rows.append(row)
(a.output/'results.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(dict(total=len(rows),passed=sum(r['pass'] for r in rows))))
raise SystemExit(0 if all(r['pass'] for r in rows) else 1)
