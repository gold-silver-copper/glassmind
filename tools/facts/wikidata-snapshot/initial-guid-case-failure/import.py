#!/usr/bin/env python3
"""Bounded storage pilot; consumes only the fixed public Parquet manifest."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import resource
import sqlite3
import sys
import time

import pyarrow as pa
import pyarrow.parquet as pq

ID = re.compile(r'Q[0-9]+\Z')
PROPERTY = re.compile(r'P[0-9]+\Z')
SEPARATORS = re.compile(r'[\W_]+')
CLAIM_COLUMNS = ['id', 'property', 'datavalue.id', 'datatype', 'rank',
                 'snaktype', 'statement_id', 'qualifiers.list.element.key']


def normalized(text):
    # Pilot only: Python Unicode word semantics must be checked against Rust
    # before this becomes a runtime index (not claimed by this storage experiment).
    return SEPARATORS.sub(' ', text.lower()).strip()


def qid(value):
    return isinstance(value, str) and ID.fullmatch(value) is not None


def claim(row, ordinal):
    if not qid(row['id']):
        return None, 'non_item_subject'
    if row['rank'] not in ('normal', 'preferred', 'deprecated'):
        raise ValueError('unknown claim rank')
    if row['rank'] == 'deprecated':
        return None, 'deprecated'
    if row['snaktype'] not in ('value', 'somevalue', 'novalue'):
        raise ValueError('unknown snaktype')
    if row['snaktype'] != 'value':
        return None, row['snaktype']
    if row['datatype'] != 'wikibase-item':
        return None, 'non_item_value'
    obj = (row['datavalue'] or {}).get('id')
    if not qid(obj) or not PROPERTY.fullmatch(row['property'] or ''):
        raise ValueError('malformed item claim')
    if not (row['statement_id'] or '').startswith(row['id'] + '$'):
        raise ValueError('malformed statement identity')
    return (ordinal + 1, int(row['id'][1:]), int(row['property'][1:]), int(obj[1:]),
            int(row['rank'] == 'preferred'), int(bool(row['qualifiers'])),
            row['statement_id']), 'kept'


def main():
    root = Path(sys.argv[1]).resolve()
    manifest = json.loads((root / 'pilot-manifest.json').read_text())
    dbpath = root / 'pilot.sqlite'
    if dbpath.exists():
        raise RuntimeError('preserve the existing pilot database; choose a new run')
    started = time.monotonic()
    pa.set_cpu_count(2)
    pa.set_io_thread_count(2)
    db = sqlite3.connect(dbpath)
    db.executescript('''
      PRAGMA journal_mode=DELETE;
      PRAGMA synchronous=NORMAL;
      PRAGMA temp_store=FILE;
      PRAGMA cache_size=-131072;
      PRAGMA max_page_count=3145728;
      CREATE TABLE edges (row_id INTEGER PRIMARY KEY, subject INTEGER NOT NULL,
        property INTEGER NOT NULL, object INTEGER NOT NULL, preferred INTEGER NOT NULL,
        qualified INTEGER NOT NULL, statement TEXT NOT NULL);
      CREATE TABLE names (row_id INTEGER PRIMARY KEY, entity INTEGER NOT NULL,
        name TEXT NOT NULL, normalized TEXT NOT NULL, language INTEGER NOT NULL,
        kind INTEGER NOT NULL);
    ''')
    report = {'schema': 1, 'scope': 'one fixed claims shard; all en/mul names',
              'python': sys.version, 'pyarrow': pa.__version__,
              'sqlite': sqlite3.sqlite_version, 'files': []}
    name_offset = 0
    for spec in manifest['files']:
        path = root / 'raw' / spec['table'] / spec['path']
        assert path.stat().st_size == spec['bytes']
        with path.open('rb') as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest() == spec['sha256']
        parquet = pq.ParquetFile(path)
        counts = Counter()
        ordinal = 0
        samples = []
        # Fixed raw row positions; independent of values, QA and acceptance.
        sample_positions = {0, parquet.metadata.num_rows - 1}
        sample_positions.update(range(0, parquet.metadata.num_rows, 100003))
        is_claim = spec['table'] == 'claims'
        lang = spec['path'].split('/')[0]
        columns = CLAIM_COLUMNS if is_claim else ['id', 'language', 'value']
        for batch in parquet.iter_batches(batch_size=16384, columns=columns, use_threads=False):
            records = []
            for row in batch.to_pylist():
                if is_claim:
                    record, reason = claim(row, ordinal)
                elif not qid(row['id']):
                    record, reason = None, 'non_item_subject'
                elif row['value'] is None:
                    record, reason = None, 'null_name'
                else:
                    if row['language'] != lang or lang not in ('en', 'mul'):
                        raise ValueError('name language mismatch')
                    record = (name_offset + ordinal + 1, int(row['id'][1:]),
                              row['value'], normalized(row['value']),
                              int(lang == 'mul'), int(spec['table'] == 'aliases'))
                    reason = 'kept'
                counts[reason] += 1
                if record is not None:
                    records.append(record)
                if ordinal in sample_positions:
                    samples.append({'source_ordinal': ordinal,
                                    'row_id': ordinal + 1 if is_claim else name_offset + ordinal + 1,
                                    'kept': record is not None})
                ordinal += 1
            sql = 'INSERT INTO edges VALUES (?,?,?,?,?,?,?)' if is_claim else 'INSERT INTO names VALUES (?,?,?,?,?,?)'
            db.executemany(sql, records)
            db.commit()
            if time.monotonic() - started > 1800:
                raise RuntimeError('import time budget exceeded')
        assert ordinal == parquet.metadata.num_rows == sum(counts.values())
        entry = {**spec, 'source_rows': ordinal, 'counts': dict(counts),
                 'samples': samples, 'row_id_offset': 0 if is_claim else name_offset}
        report['files'].append(entry)
        if not is_claim:
            name_offset += ordinal
        (root / 'import-progress.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps({'table': spec['table'], 'path': spec['path'],
                          'counts': dict(counts), 'seconds': round(time.monotonic() - started, 2)}), flush=True)
    db.executescript('''
      CREATE INDEX edges_forward ON edges(subject,property,object);
      CREATE INDEX edges_inverse ON edges(property,object,subject);
      CREATE INDEX names_lookup ON names(normalized,entity);
      CREATE INDEX names_entity ON names(entity,kind,language);
      ANALYZE;
    ''')
    report['integrity'] = db.execute('PRAGMA integrity_check').fetchall()
    assert report['integrity'] == [('ok',)]
    report['edges'] = db.execute('SELECT count(*) FROM edges').fetchone()[0]
    report['names'] = db.execute('SELECT count(*) FROM names').fetchone()[0]
    report['edge_subjects'] = db.execute('SELECT count(DISTINCT subject) FROM edges').fetchone()[0]
    report['ranks_qualifiers'] = db.execute('SELECT preferred,qualified,count(*) FROM edges GROUP BY preferred,qualified').fetchall()
    db.close()
    report['seconds'] = round(time.monotonic() - started, 3)
    report['peak_rss_kib'] = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    report['database_bytes'] = dbpath.stat().st_size
    assert report['seconds'] < 1800 and report['database_bytes'] < 12 * 1024**3
    (root / 'import-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'files'}), flush=True)


if __name__ == '__main__':
    main()
