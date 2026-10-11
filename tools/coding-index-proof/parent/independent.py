from pathlib import Path
import argparse, hashlib, json, os, subprocess

p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
a = p.parse_args()
base = Path(__file__).parent
cases = json.loads((base/'frozen.json').read_text())
a.out.mkdir(parents=True, exist_ok=False)
rows = []
for c in cases:
    root = a.out/c['name']
    (root/'src').mkdir(parents=True)
    (root/'Cargo.toml').write_text('[package]\nname="indexproof_parent"\nversion="0.1.0"\nedition="2024"\n')
    before = c['source']
    (root/'src/lib.rs').write_text(before)
    if c['expected_supported']:
        (root/'tests').mkdir()
        zero = c['name'] == 'zero_based'
        tests = '''use indexproof_parent::lookup;
#[test] fn original_rank_behavior() {
  let values = [17, 31, 59];
  for rank in [i32::MIN, -100, -1, 0, 1, 2, 3, 4, 100, i32::MAX] {
    let expected = if LOWER && UPPER { Some(values[OFFSET as usize]) } else { None };
    assert_eq!(lookup(&values, rank), expected, "rank {rank}");
    assert_eq!(lookup(&[], rank), None, "empty rank {rank}");
  }
}
'''.replace('LOWER', 'rank >= 0' if zero else 'rank >= 1').replace('UPPER', 'rank < 3' if zero else 'rank <= 3').replace('OFFSET', 'rank' if zero else '(rank - 1)')
        (root/'tests/behavior.rs').write_text(tests)
    files = {str(f.relative_to(root)): f.read_bytes() for f in root.rglob('*') if f.is_file()}
    check = subprocess.run(['cargo','check','-j1','--message-format=json'], cwd=root, text=True, capture_output=True, timeout=90)
    (root/'before.jsonl').write_text(check.stdout)
    (root/'before.stderr').write_text(check.stderr)
    assert check.returncode != 0 and 'E0277' in check.stdout, c['name']
    result = subprocess.run([str(a.binary.resolve()), 'do', str(root.resolve()), c['request']], text=True, capture_output=True, timeout=180)
    (root/'solver.stdout').write_text(result.stdout)
    (root/'solver.stderr').write_text(result.stderr)
    after = (root/'src/lib.rs').read_text()
    changed = after != before
    for name, content in files.items():
        if name != 'src/lib.rs':
            assert (root/name).read_bytes() == content, (c['name'], name)
    test_exit = None
    minimal = not changed
    if changed:
        alternatives = ('index as usize','(index) as usize','(index as usize)','((index) as usize)')
        minimal = any(after == before.replace('xs[index]', 'xs['+x+']') for x in alternatives)
        assert c['expected_supported'] and minimal, (c['name'], after)
        native = subprocess.run(['cargo','test','-j1','--test','behavior','--','--test-threads','4'],cwd=root,text=True,capture_output=True,timeout=90)
        test_exit = native.returncode
        (root/'native.stdout').write_text(native.stdout)
        (root/'native.stderr').write_text(native.stderr)
        assert test_exit == 0, c['name']
    row = dict(case=c['name'],expected_supported=c['expected_supported'],changed=changed,minimal_edit=minimal,native_test_exit=test_exit,reply=result.stdout,exit=result.returncode,source_sha256=hashlib.sha256(before.encode()).hexdigest(),after_sha256=hashlib.sha256(after.encode()).hexdigest())
    rows.append(row)
    print(c['name'], 'edited' if changed else 'refused', 'native', test_exit, flush=True)
(a.out/'report.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(a.binary.read_bytes()).hexdigest(),cases=rows,edits=sum(r['changed'] for r in rows),unsafe_edits=0),indent=2)+'\n')
