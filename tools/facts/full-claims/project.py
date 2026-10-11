#!/usr/bin/env python3
"""Claims-only projection of the frozen full public release; no QA inputs."""
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

COLUMNS = ['id', 'property', 'datavalue.id', 'datatype', 'rank', 'snaktype',
           'statement_id', 'qualifiers.list.element.key']
ID = re.compile(r'Q[0-9]+\Z')
PROPERTY = re.compile(r'P[0-9]+\Z')
LIMIT = 96 * 1024**3
SECONDS = 14400


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def canonical(value, prefix):
    number = int(value[1:])
    if not 0 < number <= 2**63 - 1 or prefix + str(number) != value:
        raise ValueError('noncanonical or overflowing retained identifier')
    return number


def select(row, row_id):
    subject = row['id']
    if not isinstance(subject, str) or not ID.fullmatch(subject):
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
    prop = row['property']
    if not isinstance(obj, str) or not ID.fullmatch(obj) or not PROPERTY.fullmatch(prop or ''):
        raise ValueError('malformed eligible claim')
    guid = row['statement_id']
    if not isinstance(guid, str):
        raise ValueError('missing statement GUID')
    anchor, separator, suffix = guid.partition('$')
    if anchor.upper() != subject or not separator or not suffix:
        raise ValueError('malformed statement GUID')
    return (row_id, canonical(subject, 'Q'), canonical(prop, 'P'), canonical(obj, 'Q'),
            int(row['rank'] == 'preferred'), int(bool(row['qualifiers'])), guid), 'kept'


def main():
    root = Path(sys.argv[1]).resolve()
    acquisition = json.loads((root / 'acquisition.json').read_text())
    started = time.monotonic()
    pa.set_cpu_count(2)
    pa.set_io_thread_count(2)
    output = root / 'claims.sqlite'
    with output.open('xb'):
        pass
    db = sqlite3.connect(output)
    db.executescript('''PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
        PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;
        CREATE TABLE edges(row_id INTEGER PRIMARY KEY, subject INTEGER NOT NULL,
        property INTEGER NOT NULL, object INTEGER NOT NULL, preferred INTEGER NOT NULL,
        qualified INTEGER NOT NULL, statement TEXT NOT NULL);''')
    page = db.execute('PRAGMA page_size').fetchone()[0]
    db.execute(f'PRAGMA max_page_count={(LIMIT-1)//page}')
    report = dict(schema=1, acquisition_sha256=digest(root/'acquisition.json'),
                  pyarrow=pa.__version__, python=sys.version, files=[], samples=[],
                  canonical_retained_ids=0)
    assert pa.__version__ == '25.0.1'
    offset = total = qualified = 0
    for spec in acquisition['files']:
        path = root / 'raw' / spec['table'] / spec['path']
        assert path.stat().st_size == spec['bytes'] and digest(path) == spec['sha256']
        parquet = pq.ParquetFile(path)
        nrows = parquet.metadata.num_rows
        positions = {0, nrows-1, *range(0, nrows, 100003)}
        counts, ranks = Counter(), Counter()
        ordinal = 0
        for batch in parquet.iter_batches(batch_size=16384, columns=COLUMNS, use_threads=False):
            rows = []
            for row in batch.to_pylist():
                row_id = offset + ordinal + 1
                record, reason = select(row, row_id)
                counts[reason] += 1
                if record is not None:
                    rows.append(record)
                    ranks[f'{record[4]},{record[5]}'] += 1
                    qualified += record[5]
                if ordinal in positions:
                    report['samples'].append(dict(file=spec['path'], ordinal=ordinal,
                                                  row_id=row_id, reason=reason, record=record))
                ordinal += 1
            db.executemany('INSERT INTO edges VALUES (?,?,?,?,?,?,?)', rows)
            db.commit()
            if time.monotonic() - started >= SECONDS:
                raise RuntimeError('projection time gate exceeded')
        assert ordinal == nrows == sum(counts.values())
        total += counts['kept']
        report['canonical_retained_ids'] += 3*counts['kept']
        entry = {k:v for k,v in spec.items() if k != 'url'}
        entry.update(source_rows=nrows, kept_rows=counts['kept'], row_id_offset=offset)
        report['files'].append(dict(source=entry, counts=dict(counts), ranks=dict(ranks)))
        offset += nrows
        (root/'projection-progress.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(dict(path=spec['path'], rows=nrows, kept=counts['kept'],
                              seconds=time.monotonic()-started)), flush=True)
    assert offset == 789435443
    assert db.execute('SELECT count(*) FROM edges').fetchone()[0] == total
    assert db.execute('SELECT count(*) FROM edges WHERE qualified=1').fetchone()[0] == qualified
    assert db.execute('PRAGMA integrity_check').fetchall() == [('ok',)]
    db.close()
    output.chmod(0o444)
    report.update(source_rows=offset, statements=total, qualified_statements=qualified,
                  projection_bytes=output.stat().st_size, projection_sha256=digest(output),
                  seconds=time.monotonic()-started,
                  peak_rss_kib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    assert report['seconds'] < SECONDS and report['projection_bytes'] < LIMIT
    (root/'projection-report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('files','samples')}),flush=True)


if __name__ == '__main__':
    main()
