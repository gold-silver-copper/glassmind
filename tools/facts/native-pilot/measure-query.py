#!/usr/bin/env python3
"""Measure the public query process from a fresh small Python launcher."""
import json
import os
from pathlib import Path
import resource
import subprocess
import time

root = Path(__file__).resolve().parent
model = root.parent.parent / 'trainindex-run/tmp/trainindex/bundle1000/relation.json'
command = [str(root / 'candidate'), 'predict-disk', '--model', str(model), '--database', str(root / 'native.sqlite')]
start = time.monotonic()
launcher_rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
with (root / 'source-name-queries.txt').open('rb') as inp, (root / 'source-name-predictions.jsonl').open('wb') as out:
    child = subprocess.Popen(command, stdin=inp, stdout=out)
    pid, status, usage = os.wait4(child.pid, 0)
    child.returncode = os.waitstatus_to_exitcode(status)
report = {'command': command, 'seconds': time.monotonic() - start, 'peak_rss_kib': usage.ru_maxrss,
          'launcher_peak_rss_kib': launcher_rss, 'returncode': child.returncode,
          'note': 'One local source-name batch including startup; no QA accuracy or general latency claim.'}
(root / 'query-resource-report.json').write_text(json.dumps(report, indent=2) + '\n')
assert child.returncode == 0
