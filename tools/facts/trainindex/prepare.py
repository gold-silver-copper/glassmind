#!/usr/bin/env python3
"""Freeze TRAIN-only scope, bounded public fetch, deterministic offline rebuild.

No validation input, question/answer table, runtime network, or model changes.
Run select, fetch (once per run), then rebuild. Raw artifacts belong outside git.
"""
import argparse
from collections import Counter
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
import gzip
import hashlib
import json
from pathlib import Path
import re
import time
import urllib.error
import urllib.parse
import urllib.request

REV = 'df69267f1db4f85a71325e7e8c06e192bcaf0b0e'
UA = 'GlassmindResearch/0.1 (https://github.com/gold-silver-copper/glassmind; TRAIN-scoped facts research)'
MAX_BYTES = 80 * 1024 * 1024

def sha(data):
    return hashlib.sha256(data).hexdigest()

def write(path, obj):
    path.write_text(json.dumps(obj, sort_keys=True, indent=2) + '\n')

def utc():
    return datetime.now(timezone.utc).isoformat()

def identity(path):
    data = path.read_bytes()
    return dict(path=str(path.resolve()), bytes=len(data), sha256=sha(data))

def select(train, out):
    assert train.name == 'train.tsv', 'TRAIN only'
    rows = train.read_text().splitlines()
    end = len(rows) * 9 // 10
    fit = [i for i in range(end) if i % 10 != 0]
    counts = Counter(rows[i].split('\t', 1)[0] for i in fit)
    assert all(re.fullmatch(r'Q[0-9]+', q) for q in counts)
    ordered = sorted(counts, key=lambda q: (-counts[q], sha(q.encode())))[:1000]
    assert len(ordered) == 1000
    doc = dict(version=1, source_revision=REV, train=identity(train),
               total_train_rows=len(rows), prefix_end_exclusive=end,
               fit_rows=len(fit), excluded_calibration_in_prefix=end-len(fit),
               excluded_tail=len(rows)-end, old_model_calibration_rows=sum(i%10==0 for i in range(len(rows))),
               fit_row_indices_sha256=sha(('\n'.join(map(str, fit))+'\n').encode()),
               rule='i < floor(0.9*N) and i%10 != 0; frequency desc, SHA256(QID) asc',
               entities=[dict(qid=q, count=counts[q], tie_sha256=sha(q.encode())) for q in ordered])
    dest = out/'selection.json'
    if dest.exists():
        assert json.loads(dest.read_bytes()) == doc, 'selection is immutable'
    else:
        write(dest, doc)
    print(json.dumps({k:v for k,v in doc.items() if k!='entities'}), flush=True)

def selection(out):
    return [r['qid'] for r in json.loads((out/'selection.json').read_bytes())['entities']]

def reply_entities(path):
    # A byte-budget cutoff can retain an incomplete response for audit, but it
    # must never create claims. API error replies likewise supply no entities.
    try:
        return json.loads(path.read_bytes()).get('entities', {})
    except (ValueError, AttributeError):
        return {}

def cached(paths, ids):
    entities, sources = {}, []
    for path in paths:
        used = []
        for q, e in reply_entities(path).items():
            if q in ids and q not in entities and 'missing' not in e and e.get('id') == q:
                entities[q] = e
                used.append(q)
        if used:
            sources.append(dict(**identity(path), selected_ids=used,
                                timestamp_note='original fetch timestamp unavailable; filesystem mtime only',
                                file_mtime_utc=datetime.fromtimestamp(path.stat().st_mtime, timezone.utc).isoformat()))
    return entities, sources

def retry_delay(headers):
    value = headers.get('Retry-After', '2')
    try:
        return max(2, float(value))
    except ValueError:
        return max(2, parsedate_to_datetime(value).timestamp()-time.time())

def fetch(cache, out):
    # Refuse a second network phase; cached rebuild is always safe and resumable.
    ledger_path = out/'fetch.json'
    if ledger_path.exists():
        raise SystemExit('fetch ledger already exists: no repeated network budget; use rebuild')
    ids = selection(out)
    cache_paths = sorted(cache.glob('*.json'))
    old, sources = cached(cache_paths, set(ids))
    write(out/'label-cache.json', [identity(p) for p in cache_paths])
    missing = [q for q in ids if q not in old]
    raw = out/'raw'
    raw.mkdir(exist_ok=True)
    ledger = dict(selection=identity(out/'selection.json'), started_utc=utc(), license='CC0-1.0', source='https://www.wikidata.org/w/api.php',
                  data_access='https://www.wikidata.org/wiki/Wikidata:Data_access',
                  licensing='https://www.wikidata.org/wiki/Wikidata:Licensing',
                  user_agent=UA, selected_cached=len(old), reused_sources=sources,
                  requests=[], new_decompressed_bytes=0, stop_reason=None)
    write(ledger_path, ledger)
    start = time.monotonic()
    last = start-2
    stop = False
    for offset in range(0, len(missing), 25):
        group = missing[offset:offset+25]
        url = ledger['source']+'?'+urllib.parse.urlencode(dict(action='wbgetentities', ids='|'.join(group), props='info|labels|aliases|claims', languages='en', format='json', maxlag=5))
        for attempt in range(2):
            delay = max(0, 2-(time.monotonic()-last))
            if time.monotonic()-start+delay >= 600 or len(ledger['requests']) >= 80 or ledger['new_decompressed_bytes'] >= MAX_BYTES:
                ledger['stop_reason'] = 'budget reached'
                stop = True
                break
            time.sleep(delay)
            last = time.monotonic()
            record = dict(ids=group, attempt=attempt+1, started_utc=utc(), url=url)
            ledger['requests'].append(record)
            write(ledger_path, ledger)
            headers = {}
            try:
                request = urllib.request.Request(url, headers={'User-Agent':UA, 'Accept-Encoding':'gzip'})
                with urllib.request.urlopen(request, timeout=min(45, 600-(time.monotonic()-start))) as reply:
                    headers = reply.headers
                    record['http_status'] = reply.status
                    stream = gzip.GzipFile(fileobj=reply) if headers.get('Content-Encoding') == 'gzip' else reply
                    # Read at most the remaining decompressed budget, no extra byte.
                    remaining = MAX_BYTES-ledger['new_decompressed_bytes']
                    data = stream.read(remaining)
                ledger['new_decompressed_bytes'] += len(data)
                path = raw/(sha(data)+'.json')
                path.write_bytes(data)
                record.update(identity(path))
                record['finished_utc'] = utc()
                obj = json.loads(data)
                if 'error' in obj:
                    raise ValueError(json.dumps(obj['error']))
                if 'entities' not in obj:
                    raise ValueError('missing entities map')
                record['success'] = True
                write(ledger_path, ledger)
                print(f"requests={len(ledger['requests'])} bytes={ledger['new_decompressed_bytes']}", flush=True)
                break
            except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError, ValueError, OSError) as error:
                if isinstance(error, urllib.error.HTTPError):
                    headers = error.headers
                    record['http_status'] = error.code
                record['error'] = str(error)
                record['finished_utc'] = utc()
                delay = retry_delay(headers)
                record['retry_delay_seconds'] = delay
                if attempt == 1 or time.monotonic()-start+delay >= 600:
                    ledger['stop_reason'] = 'repeated failure or retry exceeds budget'
                    stop = True
                write(ledger_path, ledger)
                if stop:
                    break
                time.sleep(delay)
        if stop:
            break
    ledger.update(finished_utc=utc(), elapsed_seconds=time.monotonic()-start)
    write(ledger_path, ledger)
    print(json.dumps(ledger), flush=True)

def extract(entity):
    aliases = [e['value'] for e in entity.get('aliases', {}).get('en', [])]
    if 'en' in entity.get('labels', {}):
        aliases.append(entity['labels']['en']['value'])
    claims = {}
    for prop, statements in entity.get('claims', {}).items():
        targets = set()
        for statement in statements:
            if statement.get('rank') == 'deprecated':
                continue
            value = statement.get('mainsnak', {}).get('datavalue', {})
            if value.get('type') == 'wikibase-entityid':
                qid = value.get('value', {}).get('id')
                if qid and qid.startswith('Q'):
                    targets.add(qid)
        if targets:
            claims[prop] = sorted(targets)
    return dict(aliases=sorted(set(aliases)), claims=claims)

def rebuild(cache, out):
    ids = selection(out)
    # Exact selected entities only, same original-cache precedence as fetch.
    ledger = json.loads((out/'fetch.json').read_bytes())
    if 'selection' in ledger:
        assert identity(out/'selection.json')['sha256'] == ledger['selection']['sha256'], 'selection changed since fetch'
    paths = []
    # Freeze response membership and content, not whatever happens to appear in
    # the cache on a later replay. Error/partial raw replies remain auditable.
    entries = [(cache/Path(s['path']).name, s) for s in ledger['reused_sources']]
    entries += [(out/'raw'/Path(s['path']).name, s) for s in ledger['requests'] if 'path' in s]
    for path, record in entries:
        actual = identity(path)
        assert actual['sha256'] == record['sha256'] and actual['bytes'] == record['bytes'], f'raw response changed: {path}'
        paths.append(path)
    raw, sources = cached(paths, set(ids))
    label_cache_path = out/'label-cache.json'
    if not label_cache_path.exists():
        write(label_cache_path, [identity(p) for p in sorted(cache.glob('*.json'))])
    label_paths = []
    for record in json.loads(label_cache_path.read_bytes()):
        path = cache/Path(record['path']).name
        actual = identity(path)
        assert actual['sha256'] == record['sha256'] and actual['bytes'] == record['bytes'], f'label response changed: {path}'
        label_paths.append(path)
    label_paths += sorted(p for p in paths if p.parent == out/'raw')
    report = dict(selection=identity(out/'selection.json'), sources=sources, raw_without_entities=[identity(p) for p in paths if not reply_entities(p)], scopes={})
    for size in (100, 1000):
        chosen = ids[:size]
        entities = {q:extract(raw[q]) for q in chosen if q in raw}
        dest = out/f'scope{size}'
        dest.mkdir(exist_ok=True)
        index = dest/'index.json'
        index.write_text(json.dumps(dict(entities=entities), sort_keys=True))
        eligible = set(entities)
        for e in entities.values():
            for values in e['claims'].values():
                eligible.update(values)
        labels = {}
        # Reuse labels from cached replies only for eligible display IDs; never
        # turn labels-only IDs into searchable entities or fetch extra labels.
        for path in label_paths:
            for q, e in reply_entities(path).items():
                label = e.get('labels', {}).get('en', {}).get('value')
                if q in eligible and label:
                    labels.setdefault(q, label)
        write(dest/'labels.json', dict(index_sha256=sha(index.read_bytes()), source='Wikidata English labels (cached replies)', license='CC0-1.0', labels=labels))
        report['scopes'][str(size)] = dict(requested=size, actual=len(entities), missing_ids=[q for q in chosen if q not in raw], index=identity(index), labels=identity(dest/'labels.json'), entity_revisions={q:raw[q].get('lastrevid') for q in entities}, claim_values=sum(len(v) for e in entities.values() for v in e['claims'].values()))
    write(out/'rebuild.json', report)
    print(json.dumps(report['scopes']), flush=True)

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('command', choices=['select','fetch','rebuild'])
    p.add_argument('--source', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    if a.command == 'select':
        select(a.source/'train.tsv', a.output)
    elif a.command == 'fetch':
        fetch(a.source/'wikidata', a.output)
    else:
        rebuild(a.source/'wikidata', a.output)

if __name__ == '__main__':
    main()
