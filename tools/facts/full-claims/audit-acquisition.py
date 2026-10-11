#!/usr/bin/env python3
"""Verify the complete frozen inventory and acquisition ledger before import."""
import hashlib
import json
from pathlib import Path
import sys
import time


def digest(path):
    with path.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()


def main():
    root=Path(sys.argv[1]).resolve();started=time.monotonic()
    manifest=json.loads((root/'acquisition.json').read_text())
    ledger=[json.loads(line) for line in (root/'fetch.jsonl').read_text().splitlines()]
    files=manifest['files'];assert len(files)==86
    verified=[r for r in ledger if r['status']=='verified']
    assert len(verified)==85
    by_path={r['path']:r for r in verified};assert len(by_path)==85
    assert set(by_path)=={f['path'] for f in files[1:]}
    attempts={}
    for row in ledger:
        attempts.setdefault(row['path'],[]).append(row['attempt'])
        assert row['elapsed_seconds']<7200 and row['total_received_bytes']<=96*1024**3
    assert all(v==[0] or v==[0,1] for v in attempts.values())
    checks=[]
    for i,f in enumerate(files):
        path=root/'raw'/f['table']/f['path']
        assert path.stat().st_size==f['bytes'] and digest(path)==f['sha256']
        if i:
            row=by_path[f['path']]
            assert row['sha256']==f['sha256'] and row['bytes']==f['bytes']
            assert not path.is_symlink()
            path.chmod(0o444)
        checks.append({k:f[k] for k in ('table','path','bytes','sha256')})
    assert sum(f['bytes'] for f in files)==44633986614
    assert sum(f['bytes'] for f in files[1:])==44104948009
    report=dict(files=checks,source_bytes=44633986614,new_source_bytes=44104948009,
                total_received_bytes=ledger[-1]['total_received_bytes'],
                acquisition_seconds_through_last_verified_file=ledger[-1]['elapsed_seconds'],
                audit_seconds=time.monotonic()-started,retries=sum(len(v)-1 for v in attempts.values()),
                acquisition_sha256=digest(root/'acquisition.json'),fetcher_sha256=digest(root/'fetch.py'),
                ledger_sha256=digest(root/'fetch.jsonl'),all_publisher_hashes_match=True)
    with (root/'acquisition-report.json').open('x') as out:out.write(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='files'}),flush=True)


if __name__=='__main__':main()
