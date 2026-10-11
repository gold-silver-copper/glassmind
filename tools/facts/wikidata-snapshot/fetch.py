#!/usr/bin/env python3
"""Download a fixed manifest, unauthenticated, with bounded bytes and attempts."""
import hashlib
import json
import math
from email.utils import parsedate_to_datetime
from pathlib import Path
import sys
import time
import urllib.error
import urllib.request


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def retry_delay(value):
    """None means stop rather than retry before an unknown/long server delay."""
    if value is None:
        return 10
    try:
        seconds = int(value) if value.isdigit() else math.ceil(
            parsedate_to_datetime(value).timestamp() - time.time())
    except (TypeError, ValueError, OverflowError):
        return None
    return max(0, seconds) if seconds <= 60 else None


def main():
    root = Path(sys.argv[1]).resolve()
    manifest = json.loads((root / 'pilot-manifest.json').read_text())
    started = time.monotonic()
    received = 0
    log = root / 'fetch.jsonl'
    for spec in manifest['files']:
        target = root / 'raw' / spec['table'] / spec['path']
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            assert target.stat().st_size == spec['bytes']
            assert digest(target) == spec['sha256']
            print('verified existing', target, flush=True)
            continue
        for attempt in range(2):
            part = target.with_suffix(f'.attempt{attempt}.partial')
            if part.exists():
                raise RuntimeError(f'preserved previous attempt exists: {part}')
            event = {'table': spec['table'], 'path': spec['path'], 'attempt': attempt}
            retry = False
            try:
                if time.monotonic() - started > 1800:
                    raise RuntimeError('acquisition time budget exceeded')
                request = urllib.request.Request(spec['url'], headers={
                    'User-Agent': 'glassmind-public-snapshot-pilot/1.0',
                    'Accept-Encoding': 'identity',
                })
                h = hashlib.sha256()
                written = 0
                with urllib.request.urlopen(request, timeout=60) as response, part.open('xb') as out:
                    while chunk := response.read(1024 * 1024):
                        received += len(chunk)
                        written += len(chunk)
                        if received > 3 * 1024**3 or written > spec['bytes']:
                            raise RuntimeError('download byte budget exceeded')
                        if time.monotonic() - started > 1800:
                            raise RuntimeError('acquisition time budget exceeded')
                        out.write(chunk)
                        h.update(chunk)
                if written != spec['bytes'] or h.hexdigest() != spec['sha256']:
                    raise RuntimeError('publisher size/hash mismatch')
                part.rename(target)
                event.update(status='verified', bytes=written, sha256=h.hexdigest())
            except (urllib.error.URLError, TimeoutError) as error:
                # Do not log redirect URLs, which can contain signed CDN parameters.
                event.update(status='network_error', error_type=type(error).__name__)
                if isinstance(error, urllib.error.HTTPError):
                    event['http_status'] = error.code
                    retry = error.code in (429, 500, 502, 503, 504)
                    delay = retry_delay(error.headers.get('Retry-After'))
                else:
                    retry, delay = True, 10
                if retry and attempt == 0 and delay is not None:
                    time.sleep(delay)
                else:
                    retry = False
            finally:
                event.update(elapsed_seconds=round(time.monotonic() - started, 3),
                             total_received_bytes=received)
                with log.open('a') as out:
                    out.write(json.dumps(event, sort_keys=True) + '\n')
                print(json.dumps(event, sort_keys=True), flush=True)
            if target.exists():
                break
            if not retry:
                raise RuntimeError('download stopped; see fetch ledger')
        else:
            raise RuntimeError('attempt budget exhausted')


if __name__ == '__main__':
    main()
