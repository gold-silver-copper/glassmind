#!/usr/bin/env python3
"""Raw public-row audit and source-name query checks, without QA annotations."""
import hashlib
import json
from pathlib import Path
import re
import resource
import sqlite3
import subprocess
import time

import pyarrow.parquet as pq

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / 'wikidata-snapshot'
MODEL = ROOT.parent.parent / 'trainindex-run/tmp/trainindex/bundle1000/relation.json'


def connection(path):
    db = sqlite3.connect(path.resolve().as_uri() + '?mode=ro', uri=True)
    db.executescript('PRAGMA query_only=ON; PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;')
    return db


def main():
    started = time.monotonic()
    retained = json.loads((SOURCE / 'import-report.json').read_text())
    source = connection(SOURCE / 'pilot.sqlite')
    db = connection(ROOT / 'native.sqlite')
    sampled = []
    file_checks = []
    golden = {'entities': {}}
    statement_checks = 0
    for spec in retained['files']:
        path = SOURCE / 'raw' / spec['table'] / spec['path']
        with path.open('rb') as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest() == spec['sha256']
        parquet = pq.ParquetFile(path)
        positions = {0, spec['source_rows'] - 1} | set(range(0, spec['source_rows'], 100003))
        assert positions == {s['source_ordinal'] for s in spec['samples']}
        claims = spec['table'] == 'claims'
        columns = ['id', 'property', 'datavalue.id', 'rank', 'snaktype', 'datatype', 'statement_id', 'qualifiers'] if claims else None
        offset = 0
        checked = 0
        for group in range(parquet.num_row_groups):
            rows = parquet.metadata.row_group(group).num_rows
            wanted = sorted(p for p in positions if offset <= p < offset + rows)
            if wanted:
                table = parquet.read_row_group(group, columns=columns, use_threads=False)
                for ordinal in wanted:
                    raw = table.slice(ordinal - offset, 1).to_pylist()[0]
                    row_id = spec['row_id_offset'] + ordinal + 1
                    item = re.fullmatch(r'Q[0-9]+', raw['id']) is not None
                    if claims:
                        eligible = item and raw['rank'] in ('normal', 'preferred') and raw['snaktype'] == 'value' and raw['datatype'] == 'wikibase-item'
                        witness = db.execute('SELECT * FROM statements WHERE source_row=?', (row_id,)).fetchone()
                        if eligible:
                            expected = (row_id, raw['id'], raw['property'], raw['datavalue']['id'],
                                        int(raw['rank'] == 'preferred'), int(bool(raw['qualifiers'])), raw['statement_id'])
                            assert witness == expected, (witness, expected)
                            assert db.execute('SELECT 1 FROM edges WHERE subject=? AND property=? AND object=?', expected[1:4]).fetchone() == (1,)
                            statement_checks += 1
                        else:
                            assert witness is None
                    elif item and raw['value'] is not None:
                        golden['entities'].setdefault(raw['id'], {'aliases': [], 'claims': {}})['aliases'].append(raw['value'])
                        sampled.append((raw['id'], raw['value'], raw['language'], spec['table'], row_id))
                    checked += 1
            offset += rows
        assert checked == len(positions)
        file_checks.append({'table': spec['table'], 'path': spec['path'], 'raw_samples': checked})
    # Obtain native keys independently through the existing version-1 importer.
    # This tiny store contains only sampled raw names, never questions or labels.
    json_path = ROOT / 'raw-sample-view.json'
    json_path.write_text(json.dumps(golden, ensure_ascii=False) + '\n')
    subprocess.run([str(ROOT / 'final-candidate'), 'index-disk', '--index', str(json_path),
                    '--database', str(ROOT / 'raw-sample-view.sqlite')], check=True,
                   stdout=(ROOT / 'raw-sample-view-info.json').open('wb'))
    reference = connection(ROOT / 'raw-sample-view.sqlite')
    public_queries = []
    expected_entities = []
    excluded = 0
    query_skipped = 0
    for entity, alias, language, kind, row_id in sampled:
        key = reference.execute('SELECT normalized,uppercase FROM aliases WHERE entity=? AND original=?', (entity, alias)).fetchone()
        found = db.execute('SELECT normalized,uppercase,source_mask,occurrences,first_source_row FROM aliases WHERE entity=? AND original=?', (entity, alias)).fetchone()
        if key is None:
            assert found is None
            excluded += 1
            continue
        assert found[:2] == key
        provenance = source.execute('SELECT row_id,language,kind FROM names WHERE entity=? AND name=?', (int(entity[1:]), alias)).fetchall()
        mask = 0
        for _, lang, typ in provenance:
            mask |= 1 << (lang * 2 + typ)
        assert found[2:] == (mask, len(provenance), min(r[0] for r in provenance))
        assert (row_id, int(language == 'mul'), int(kind == 'aliases')) in provenance
        if '\n' in alias or '\r' in alias or len(alias.encode()) > 4096 or len(key[0].split()) > 128:
            query_skipped += 1
            continue
        # The full normalized source name is necessarily the longest span.
        # All entities with that exact key determine the link ambiguity.
        candidates = db.execute('SELECT DISTINCT entity FROM aliases WHERE normalized=? LIMIT 2', (key[0],)).fetchall()
        public_queries.append(alias)
        expected_entities.append(candidates[0][0] if len(candidates) == 1 else None)
    (ROOT / 'source-name-queries.txt').write_text(''.join(q + '\n' for q in public_queries))
    with MODEL.open('rb') as stream:
        assert hashlib.file_digest(stream, 'sha256').hexdigest() == 'db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6'
    query_start = time.monotonic()
    subprocess.run(['timeout', '--kill-after=10s', '120s', 'python3', str(ROOT / 'measure-query.py')], check=True)
    query_seconds = time.monotonic() - query_start
    outputs = [json.loads(line) for line in (ROOT / 'source-name-predictions.jsonl').read_text().splitlines()]
    assert len(outputs) == len(expected_entities)
    assert [r['entity'] for r in outputs] == expected_entities
    metadata = dict(db.execute('SELECT key,value FROM metadata'))
    with (ROOT / 'source-name-accounting.json').open('wb') as result:
        subprocess.run([str(ROOT / 'pilot-counts'), str(SOURCE / 'pilot.sqlite')], stdout=result, check=True)
    independent_names = json.loads((ROOT / 'source-name-accounting.json').read_text())
    assert independent_names['source_rows'] == retained['names']
    observed_aliases, occurrences = db.execute('SELECT count(*),sum(occurrences) FROM aliases').fetchone()
    assert observed_aliases == int(metadata['aliases'])
    assert occurrences == int(metadata['accepted_name_rows'])
    assert occurrences + int(metadata['excluded_short_name_rows']) == retained['names']
    assert observed_aliases == independent_names['eligible_aliases']
    assert occurrences == independent_names['eligible_name_rows']
    assert int(metadata['excluded_short_name_rows']) == independent_names['excluded_name_rows']
    expected_edges = source.execute('SELECT count(*) FROM (SELECT subject,property,object FROM edges GROUP BY subject,property,object)').fetchone()[0]
    assert expected_edges == db.execute('SELECT count(*) FROM edges').fetchone()[0] == int(metadata['edges'])
    expected_entities_count = source.execute('SELECT count(*) FROM (SELECT entity FROM names UNION SELECT subject FROM edges)').fetchone()[0]
    assert expected_entities_count == db.execute('SELECT count(*) FROM entities').fetchone()[0] == int(metadata['entities'])
    ranks = db.execute('SELECT preferred,qualified,count(*) FROM statements GROUP BY preferred,qualified').fetchall()
    assert [list(r) for r in ranks] == retained['ranks_qualifiers']
    plans = {}
    for name, query, params, index in [
        ('forward', 'SELECT object FROM edges WHERE subject=? AND property=?', ('Q100','P31'), 'PRIMARY KEY'),
        ('inverse', 'SELECT subject FROM edges WHERE property=? AND object=?', ('P31','Q5'), 'edges_inverse'),
        ('alias', 'SELECT entity FROM aliases WHERE normalized=?', ('london',), 'aliases_lookup')]:
        plan = db.execute('EXPLAIN QUERY PLAN ' + query, params).fetchall()
        assert index in str(plan) and 'SEARCH' in str(plan)
        plans[name] = plan
    with (ROOT / 'native.sqlite').open('rb') as stream:
        output_hash = hashlib.file_digest(stream, 'sha256').hexdigest()
    report = {'raw_file_checks': file_checks, 'raw_samples': sum(f['raw_samples'] for f in file_checks),
              'sampled_statement_witnesses': statement_checks, 'sampled_names': len(sampled),
              'sampled_ineligible_names': excluded, 'query_skipped_for_line_or_size_limits': query_skipped,
              'source_name_queries': len(public_queries), 'linked': sum(e is not None for e in expected_entities),
              'ambiguous': sum(e is None for e in expected_entities), 'query_seconds': query_seconds,
              'entities': expected_entities_count, 'aliases': observed_aliases, 'edges': expected_edges,
              'accepted_name_rows': occurrences, 'excluded_short_name_rows': int(metadata['excluded_short_name_rows']),
              'independent_name_accounting': independent_names,
              'ranks_qualifiers': ranks, 'query_plans': plans, 'database_sha256': output_hash,
              'seconds': time.monotonic() - started, 'audit_peak_rss_kib': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
              'query_resources': json.loads((ROOT / 'query-resource-report.json').read_text()),
              'limitation': 'Source-name consistency and runtime checks only; no QA accuracy claim.'}
    (ROOT / 'audit-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
