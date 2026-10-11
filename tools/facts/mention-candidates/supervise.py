#!/usr/bin/env python3
"""Finite owned-process supervisor including unlinked SQLite temporary files."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def footprint(root, pid):
    paths = list(root.rglob('*'))
    try:
        paths += list(Path(f'/proc/{pid}/fd').iterdir())
    except FileNotFoundError:
        pass
    total, seen = 0, set()
    for p in paths:
        try:
            if str(p).startswith('/proc/') and not os.readlink(p).startswith(str(root) + '/'):
                continue
            stat = p.stat()
            key = stat.st_dev, stat.st_ino
            if key not in seen:
                seen.add(key)
                total += max(stat.st_size, 512*stat.st_blocks)
        except FileNotFoundError:
            pass
    return total


def main():
    root, name, seconds, *command = sys.argv[1:]
    root, seconds = Path(root).resolve(), int(seconds)
    limit = 2*1024**3
    def interrupted(signum, frame):
        raise RuntimeError(f"supervisor interrupted by signal {signum}")
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    started = time.monotonic()
    peak = 0
    failed_at = None
    with (root/f'{name}.log').open('xb') as log:
        p = subprocess.Popen(command, stdout=log, stderr=log, start_new_session=True)
        (root/f'{name}-process.json').write_text(json.dumps(dict(command=command,
             pid=p.pid, supervisor_pid=os.getpid(), seconds_cap=seconds, bytes_cap=limit))+'\n')
        try:
            while True:
                pid, status, usage = os.wait4(p.pid, os.WNOHANG)
                elapsed = time.monotonic()-started
                peak = max(peak, footprint(root, p.pid))
                if pid:
                    p.returncode = os.waitstatus_to_exitcode(status)
                    break
                if failed_at is None and (peak >= limit or elapsed >= seconds):
                    failed_at = elapsed
                    os.killpg(p.pid, signal.SIGTERM)
                elif failed_at is not None and elapsed >= failed_at+10:
                    os.killpg(p.pid, signal.SIGKILL)
                time.sleep(2)
        finally:
            if p.returncode is None:
                os.killpg(p.pid, signal.SIGKILL)
                p.wait()
    report = dict(returncode=p.returncode, seconds=time.monotonic()-started,
                  sampled_peak_bytes=peak, peak_process_rss_kib=usage.ru_maxrss,
                  passed=p.returncode==0 and failed_at is None and peak<limit
                         and time.monotonic()-started<seconds)
    (root/f'{name}-resources.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report),flush=True)
    if not report['passed']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
