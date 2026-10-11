#!/usr/bin/env python3
"""Fetch display-only English labels for frozen index subject/object IDs.

Resumable, sequential official Wikidata API access. No question/gold input.
The persistent budget covers all resumed requests, not just one invocation.
"""
import argparse
import datetime
import email.utils
import hashlib
import json
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

API = 'https://www.wikidata.org/w/api.php'
MAX_BYTES = 10 * 1024 * 1024
MAX_SECONDS = 580  # Reserve margin within the requested ten-minute ceiling.


def digest(data):
    return hashlib.sha256(data).hexdigest()


def utc(timestamp):
    return datetime.datetime.fromtimestamp(timestamp, datetime.timezone.utc).isoformat()


def retry_delay(value, attempt):
    try:
        delay = float(value)
    except ValueError:
        delay = email.utils.parsedate_to_datetime(value).timestamp() - time.time()
    return max(5 * 2**attempt, delay)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--index', type=Path, required=True)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fetch', action='store_true')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    raw = args.output / 'raw'
    raw.mkdir(exist_ok=True)
    entities = json.loads(args.index.read_bytes())['entities']
    targets = {q for entity in entities.values() for values in entity['claims'].values() for q in values}
    ids = set(entities) | targets
    if len(ids) > 12000:
        raise ValueError('more than 12000 eligible IDs')
    labels, responses = {}, []

    def ingest(path, source):
        data = path.read_bytes()
        responses.append(dict(file=str(path.resolve()), source=source, bytes=len(data),
                              sha256=digest(data), date=utc(path.stat().st_mtime)))
        try:
            obj = json.loads(data)
        except ValueError:
            return
        for q, entity in obj.get('entities', {}).items():
            label = entity.get('labels', {}).get('en', {}).get('value')
            if q in ids and label:
                labels.setdefault(q, label)

    for path in sorted(args.cache.glob('*.json')):
        ingest(path, 'frozen cached Wikidata response')
    cached_labels, cached_targets = len(labels), len(targets & labels.keys())
    existing = sorted(raw.glob('*.json'))
    for path in existing:
        ingest(path, API)
    budget_path = args.output / 'download-budget.json'
    if budget_path.exists():
        budget = json.loads(budget_path.read_text())
    else:
        # Conservative migration for raw responses from the initial interrupted
        # run: count every retained byte and reserve 30s before its first response.
        budget = dict(started_unix=min((p.stat().st_mtime - 30 for p in existing), default=time.time()),
                      response_bytes=sum(p.stat().st_size for p in existing),
                      requests=0, rate_failures=0, consecutive_failures=0,
                      events=[], migrated_raw_responses=len(existing))
    stop = budget.get('stop', 'offline reuse')

    def save_budget():
        budget_path.write_text(json.dumps(budget, indent=2) + '\n')

    def remaining():
        return MAX_SECONDS - (time.time() - budget['started_unix'])

    if args.fetch:
        stop = 'complete'
        missing = sorted(ids - labels.keys())
        last_request = 0
        for offset in range(0, len(missing), 50):
            group = missing[offset:offset + 50]
            for attempt in range(3):
                if remaining() < 2 or budget['response_bytes'] >= MAX_BYTES:
                    stop = 'budget exhausted'
                    break
                if budget.get('halted') or budget['rate_failures'] >= 3 or budget['consecutive_failures'] >= 3:
                    stop = 'repeated failures; no further requests'
                    break
                time.sleep(max(0, 1 - (time.monotonic() - last_request)))
                url = API + '?' + urllib.parse.urlencode(dict(action='wbgetentities', ids='|'.join(group),
                    props='labels', languages='en', format='json', maxlag=5))
                last_request = time.monotonic()
                budget['requests'] += 1
                retry_after = '5'
                event = dict(date=utc(time.time()), ids=group, url=url)
                try:
                    request = urllib.request.Request(url, headers={
                        'User-Agent': 'GlassmindResearch/0.1 (github.com/gold-silver-copper/glassmind; display labels only)'})
                    with urllib.request.urlopen(request, timeout=max(1, min(25, remaining() - 1))) as response:
                        retry_after = response.headers.get('Retry-After', '5')
                        data = response.read(MAX_BYTES - budget['response_bytes'])
                    budget['response_bytes'] += len(data)
                    path = raw / (digest(data) + '.json')
                    path.write_bytes(data)
                    event.update(bytes=len(data), sha256=digest(data), retry_after=retry_after)
                    obj = json.loads(data)
                    if 'error' in obj or 'entities' not in obj:
                        if obj.get('error', {}).get('code') in {'maxlag', 'ratelimited'}:
                            budget['rate_failures'] += 1
                        raise ValueError(str(obj.get('error', 'missing entities')))
                    ingest(path, url)
                    budget['consecutive_failures'] = 0
                    event['result'] = 'success'
                    budget['events'].append(event)
                    save_budget()
                    break
                except (urllib.error.HTTPError, urllib.error.URLError, ValueError, TimeoutError, OSError) as error:
                    budget['consecutive_failures'] += 1
                    if isinstance(error, urllib.error.HTTPError):
                        retry_after = error.headers.get('Retry-After', '5')
                        if error.code in {429, 503}:
                            budget['rate_failures'] += 1
                    event.update(result='error', error=str(error), retry_after=retry_after)
                    budget['events'].append(event)
                    save_budget()
                    delay = retry_delay(retry_after, attempt)
                    if budget.get('halted') or budget['rate_failures'] >= 3 or budget['consecutive_failures'] >= 3:
                        stop = 'repeated failures; no further requests'
                        break
                    if delay + 2 >= remaining():
                        stop = 'retry delay exceeds remaining budget'
                        break
                    time.sleep(delay)
            if stop != 'complete':
                break
        budget['finished_unix'] = time.time()
        budget['stop'] = stop
        save_budget()
    report = dict(source='Wikidata English labels, display only', license='CC0-1.0',
        license_url='https://www.wikidata.org/wiki/Wikidata:Licensing',
        retrieved_utc=utc(time.time()), index_sha256=digest(args.index.read_bytes()),
        eligible_ids=len(ids), claim_targets=len(targets), cached_labels=cached_labels,
        cached_target_labels=cached_targets, labels=len(labels), target_labels=len(targets & labels.keys()),
        new_response_bytes=budget['response_bytes'], download_wall_seconds=budget.get('finished_unix', time.time())-budget['started_unix'],
        stop=stop, responses=responses, budget_file=str(budget_path.resolve()),
        raw_responses=[dict(file=str(p.resolve()), bytes=p.stat().st_size, sha256=digest(p.read_bytes()),
                            date=utc(p.stat().st_mtime)) for p in sorted(raw.glob('*.json'))])
    (args.output/'labels.json').write_text(json.dumps(dict(index_sha256=report['index_sha256'],
        source=report['source'], license=report['license'], labels=labels), sort_keys=True)+'\n')
    (args.output/'label-provenance.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in {'responses','raw_responses'}}))


if __name__ == '__main__':
    main()
