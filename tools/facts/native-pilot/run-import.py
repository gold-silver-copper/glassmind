#!/usr/bin/env python3
"""Supervise the already-frozen native ingestion gate; no model or QA input."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import shutil
import time

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent
BINARY = ROOT / 'candidate'
PILOT = BASE / 'wikidata-snapshot/pilot.sqlite'
OUTPUT = ROOT / 'native.sqlite'
TMP = ROOT / 'tmp'
LIMIT = 32 * 1024**3


def footprint(pid):
    # Include unlinked SQLite sorting files, which directory walks miss.
    entries = list(ROOT.rglob('*'))
    try:
        entries += list(Path(f'/proc/{pid}/fd').iterdir())
    except FileNotFoundError:
        pass
    seen = set()
    total = 0
    for path in entries:
        try:
            if str(path).startswith('/proc/'):
                target = os.readlink(path)
                if not target.startswith(str(ROOT) + '/'):
                    continue
            stat = path.stat()
            key = (stat.st_dev, stat.st_ino)
            if key not in seen:
                seen.add(key)
                total += max(stat.st_size, stat.st_blocks * 512)
        except FileNotFoundError:
            pass
    return total


def main():
    if OUTPUT.exists() or TMP.exists() or BINARY.exists():
        raise RuntimeError('preserve existing run outputs; do not rerun in place')
    TMP.mkdir(mode=0o700)
    shutil.copy2(BASE / 'disk-facts/target/release/glassmind-facts', BINARY)
    env = os.environ.copy()
    env['TMPDIR'] = env['SQLITE_TMPDIR'] = str(TMP)
    command = [str(BINARY), 'import-wikidata-pilot', '--pilot', str(PILOT),
               '--manifest', str(ROOT / 'source-manifest.json'), '--database', str(OUTPUT)]
    manifest = {'command': command, 'binary_sha256': hashlib.sha256(BINARY.read_bytes()).hexdigest(),
                'max_seconds': 3600, 'max_output_bytes': 24 * 1024**3,
                'max_added_run_bytes': LIMIT, 'memory_max': '8G', 'memory_swap_max': 0}
    (ROOT / 'invocation.json').write_text(json.dumps(manifest, indent=2) + '\n')
    started = time.monotonic()
    peak = 0
    failure = None
    with (ROOT / 'import-info.json').open('wb') as out, (ROOT / 'import-stages.jsonl').open('wb') as err:
        process = subprocess.Popen(command, env=env, stdout=out, stderr=err)
        (ROOT / 'import-pid.json').write_text(json.dumps({'pid': process.pid, 'supervisor_pid': os.getpid()}) + '\n')
        while True:
            pid, status, usage = os.wait4(process.pid, os.WNOHANG)
            peak = max(peak, footprint(process.pid))
            elapsed = time.monotonic() - started
            if pid:
                process.returncode = os.waitstatus_to_exitcode(status)
                break
            if (peak >= LIMIT or elapsed >= 3600) and failure is None:
                failure = 'run growth cap exceeded' if peak >= LIMIT else 'time cap exceeded'
                process.terminate()
            if elapsed >= 3610:
                process.kill()
            time.sleep(2)
    report = {**manifest, 'returncode': process.returncode, 'failure': failure,
              'seconds': time.monotonic() - started, 'peak_process_rss_kib': usage.ru_maxrss,
              'peak_added_run_bytes_sampled': peak, 'output_bytes': OUTPUT.stat().st_size if OUTPUT.exists() else 0}
    report['passed_resource_gate'] = process.returncode == 0 and failure is None and report['seconds'] < 3600 \
        and report['output_bytes'] < 24 * 1024**3 and peak < LIMIT
    (ROOT / 'import-resource-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)
    if not report['passed_resource_gate']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
