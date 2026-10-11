#!/usr/bin/env python3
"""Synchronous child resource measurement; invoke inside the capped scope."""
import argparse
import json
import resource
import subprocess
import time
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--output', type=Path, required=True)
p.add_argument('command', nargs=argparse.REMAINDER)
a = p.parse_args()
command = a.command[1:] if a.command[:1] == ['--'] else a.command
start = time.monotonic()
result = subprocess.run(command, check=False)
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
a.output.write_text(json.dumps({'command': command, 'exit_code': result.returncode,
    'elapsed_seconds': time.monotonic() - start, 'peak_rss_kib': usage.ru_maxrss,
    'user_seconds': usage.ru_utime, 'system_seconds': usage.ru_stime}, indent=2) + '\n')
raise SystemExit(result.returncode)
