#!/usr/bin/env python3
"""Independent raw-row sample audit of the pilot SQLite import."""
import hashlib
import json
from pathlib import Path
import re
import sqlite3
import sys
import time

import pyarrow.parquet as pq


def main():
    root = Path(sys.argv[1]).resolve()
    report = json.loads((root / 'import-report.json').read_text())
    db = sqlite3.connect(f'file:{root / "pilot.sqlite"}?mode=ro', uri=True)
    started = time.monotonic()
    checks = []
    for spec in report['files']:
        path = root / 'raw' / spec['table'] / spec['path']
        with path.open('rb') as source:
            assert hashlib.file_digest(source, 'sha256').hexdigest() == spec['sha256']
        parquet = pq.ParquetFile(path)
        assert parquet.metadata.num_rows == spec['source_rows'] == sum(spec['counts'].values())
        wanted = {s['source_ordinal']: s for s in spec['samples']}
        assert set(wanted) == {0, spec['source_rows'] - 1} | set(range(0, spec['source_rows'], 100003))
        offset = 0
        checked = 0
        is_claim = spec['table'] == 'claims'
        # Read full qualifier structures here, independently of the importer's
        # projected key column, and inspect a deterministic sample of raw rows.
        columns = ['id', 'property', 'datavalue.id', 'rank', 'snaktype',
                   'datatype', 'statement_id', 'qualifiers'] if is_claim else None
        for group in range(parquet.num_row_groups):
            count = parquet.metadata.row_group(group).num_rows
            selected = sorted(i for i in wanted if offset <= i < offset + count)
            if selected:
                table = parquet.read_row_group(group, columns=columns, use_threads=False)
                for ordinal in selected:
                    raw = table.slice(ordinal - offset, 1).to_pylist()[0]
                    sample = wanted[ordinal]
                    row_id = spec['row_id_offset'] + ordinal + 1
                    assert sample['row_id'] == row_id
                    subject_item = re.fullmatch('Q[0-9]+', raw['id']) is not None
                    if is_claim:
                        expected_kept = subject_item and raw['rank'] in ['normal', 'preferred'] and raw['snaktype'] == 'value' and raw['datatype'] == 'wikibase-item'
                        found = db.execute('SELECT * FROM edges WHERE row_id=?', (row_id,)).fetchone()
                        if expected_kept:
                            expected = (row_id, int(raw['id'][1:]), int(raw['property'][1:]),
                                        int(raw['datavalue']['id'][1:]),
                                        int(raw['rank'] == 'preferred'), int(bool(raw['qualifiers'])),
                                        raw['statement_id'])
                            assert found == expected
                            forward = db.execute('SELECT 1 FROM edges WHERE subject=? AND property=? AND object=? AND row_id=?', (found[1], found[2], found[3], row_id)).fetchone()
                            inverse = db.execute('SELECT 1 FROM edges WHERE property=? AND object=? AND subject=? AND row_id=?', (found[2], found[3], found[1], row_id)).fetchone()
                            assert forward == inverse == (1,)
                    else:
                        expected_kept = subject_item and raw['value'] is not None
                        found = db.execute('SELECT * FROM names WHERE row_id=?', (row_id,)).fetchone()
                        if expected_kept:
                            normalized = ' '.join(''.join(c if c.isalnum() else ' ' for c in raw['value'].lower()).split())
                            expected = (row_id, int(raw['id'][1:]), raw['value'], normalized,
                                        int(raw['language'] == 'mul'), int(spec['table'] == 'aliases'))
                            assert found == expected
                            assert db.execute('SELECT 1 FROM names WHERE normalized=? AND entity=? AND row_id=?', (normalized, found[1], row_id)).fetchone() == (1,)
                    assert sample['kept'] == expected_kept == (found is not None)
                    checked += 1
            offset += count
        assert checked == len(wanted)
        checks.append({'table': spec['table'], 'path': spec['path'], 'raw_samples': checked})
        print(json.dumps(checks[-1]), flush=True)
    expected_edges = sum(f['counts'].get('kept', 0) for f in report['files'] if f['table'] == 'claims')
    expected_names = sum(f['counts'].get('kept', 0) for f in report['files'] if f['table'] != 'claims')
    assert db.execute('SELECT count(*) FROM edges').fetchone()[0] == expected_edges
    assert db.execute('SELECT count(*) FROM names').fetchone()[0] == expected_names
    plans = {}
    for name, query, params, index in [
        ('forward', 'SELECT object FROM edges WHERE subject=? AND property=?', (100, 31), 'edges_forward'),
        ('inverse', 'SELECT subject FROM edges WHERE property=? AND object=?', (31, 5), 'edges_inverse'),
        ('alias', 'SELECT entity FROM names WHERE normalized=?', ('london',), 'names_lookup'),
    ]:
        plan = db.execute('EXPLAIN QUERY PLAN ' + query, params).fetchall()
        assert index in str(plan)
        plans[name] = plan
    result = {'source_sample_checks': checks, 'total_raw_samples': sum(c['raw_samples'] for c in checks),
              'edges': expected_edges, 'names': expected_names, 'query_plans': plans,
              'seconds': round(time.monotonic() - started, 3)}
    (root / 'audit-report.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result), flush=True)


if __name__ == '__main__':
    main()
