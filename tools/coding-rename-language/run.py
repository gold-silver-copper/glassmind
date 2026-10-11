#!/usr/bin/env python3
"""Authored development replay; reads only own cases and supplied public requests."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess
p = argparse.ArgumentParser()
p.add_argument('--binary', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--supplied', type=pathlib.Path, required=True)
p.add_argument('--sample', action='store_true')
p.add_argument('--both', action='store_true')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
assert os.environ['TMPDIR'].startswith('/home/')
assert os.environ['CARGO_TARGET_DIR']
root = a.output / 'workspace'
shutil.copytree(a.supplied / 'workspace', root, dirs_exist_ok=True)
rows = [dict(expected='rename', value='parse_line:parse_entry', text=r['text']) for r in json.loads((a.supplied/'requests.json').read_text())]
if not a.sample:
    for line in pathlib.Path(__file__).with_name('cases.tsv').read_text().splitlines():
        expected, value, text = line.split('\t', 2)
        rows.append(dict(expected=expected, value=value, text=text))
results = []
for i, row in enumerate(rows):
    cmd = ['systemd-run', '--user', '--scope', '-pMemoryMax=8G', '-pMemorySwapMax=0', 'setsid', '--wait', 'timeout', '--kill-after=10s', '600', str(a.binary.resolve()), 'ask', '--source', str(root.resolve()), '--explain', row['text']]
    done = subprocess.run(cmd, text=True, capture_output=True, timeout=620)
    (a.output/f'{i:03}.txt').write_text(done.stdout + '\nSTDERR:\n' + done.stderr)
    meanings = [line.strip().removeprefix('meaning:').strip() for line in done.stdout.splitlines() if line.strip().startswith('meaning:')]
    row.update(meanings=meanings, exit=done.returncode)
    if a.both:
        before = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*.rs')}
        do_cmd = cmd[:cmd.index('ask')] + ['do', str(root.resolve()), '--explain', row['text']]
        done_do = subprocess.run(do_cmd, text=True, capture_output=True, timeout=620)
        (a.output/f'{i:03}-do.txt').write_text(done_do.stdout + '\nSTDERR:\n' + done_do.stderr)
        do_meanings = [line.strip().removeprefix('meaning:').strip() for line in done_do.stdout.splitlines() if line.strip().startswith('meaning:')]
        after = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob('*.rs')}
        row.update(do_meanings=do_meanings, same_meanings=meanings==do_meanings, source_unchanged=before==after)
    results.append(row)
(a.output/'results.json').write_text(json.dumps(results, indent=2, ensure_ascii=False)+'\n')
print(json.dumps(dict(total=len(results), rename=sum(any('Command(Rename' in m for m in r['meanings']) for r in results), repair=sum(any('Command(Repair' in m for m in r['meanings']) for r in results))))
