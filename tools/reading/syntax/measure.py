#!/usr/bin/env python3
"""Supervise one foreground native command; record child rusage, no background jobs."""
import json
import pathlib
import resource
import subprocess
import sys
import time

start = time.monotonic()
result = subprocess.run(sys.argv[2:], check=False)
r = resource.getrusage(resource.RUSAGE_CHILDREN)
pathlib.Path(sys.argv[1]).write_text(json.dumps({
    'argv': sys.argv[2:], 'exit_code': result.returncode,
    'wall_seconds': time.monotonic()-start,
    'user_seconds': r.ru_utime, 'system_seconds': r.ru_stime,
    'max_rss_KiB': r.ru_maxrss, 'major_page_faults': r.ru_majflt,
    'minor_page_faults': r.ru_minflt,
}, indent=2)+'\n')
sys.exit(result.returncode)
