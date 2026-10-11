#!/usr/bin/env python3
"""Fixed raw-witness probes; relation proposals are synthetic, not QA results."""
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import time

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / 'wikidata-snapshot'


def readonly(path):
    db = sqlite3.connect(path.resolve().as_uri() + '?mode=ro', uri=True)
    db.executescript('PRAGMA query_only=ON; PRAGMA cache_size=-65536; PRAGMA temp_store=FILE; PRAGMA mmap_size=0;')
    return db


def main():
    start = time.monotonic()
    source = readonly(SOURCE / 'pilot.sqlite')
    public = readonly(ROOT / 'native.sqlite')
    retained = json.loads((SOURCE / 'import-report.json').read_text())
    claims = next(f for f in retained['files'] if f['table'] == 'claims')
    witnesses = []
    for sample in claims['samples']:
        row = source.execute('SELECT subject,property,object FROM edges WHERE row_id=?', (sample['row_id'],)).fetchone()
        if row is not None:
            witnesses.append(row)
    assert len(witnesses) == 59
    raw_cases = []
    names = {'entities': {}}
    missing_names = 0
    for subject, prop, obj in witnesses:
        for anchor, relation in [(subject, 'P' + str(prop)), (obj, 'R' + str(prop))]:
            name = source.execute('SELECT name FROM names WHERE entity=? ORDER BY kind,language,row_id LIMIT 1', (anchor,)).fetchone()
            if name is None:
                missing_names += 1
                continue
            question = name[0]
            entity = 'Q' + str(anchor)
            names['entities'].setdefault(entity, {'aliases': [], 'claims': {}})['aliases'].append(question)
            raw_cases.append((anchor, entity, relation, question))
    (ROOT / 'witness-names.json').write_text(json.dumps(names, ensure_ascii=False) + '\n')
    with (ROOT / 'witness-names-info.json').open('wb') as info:
        subprocess.run([str(ROOT / 'final-candidate'), 'index-disk', '--index', str(ROOT / 'witness-names.json'),
                        '--database', str(ROOT / 'witness-names.sqlite')], stdout=info, check=True)
    reference = readonly(ROOT / 'witness-names.sqlite')
    cases = []
    over_limit = 0
    for anchor, entity, relation, question in raw_cases:
        key = reference.execute('SELECT normalized,uppercase FROM aliases WHERE entity=? AND original=?', (entity, question)).fetchone()
        if len(question.encode()) > 4096 or (key is not None and len(key[0].split()) > 128):
            over_limit += 1
            continue
        found = public.execute('SELECT normalized,uppercase FROM aliases WHERE entity=? AND original=?', (entity, question)).fetchone()
        assert found == key
        linked = [] if key is None else public.execute('SELECT DISTINCT entity FROM aliases WHERE normalized=? LIMIT 2', (key[0],)).fetchall()
        resolved = entity if linked == [(entity,)] else None
        if resolved is None:
            values, total = [], 0
        else:
            field, predicate = ('object', 'subject') if relation.startswith('P') else ('subject', 'object')
            arguments = (anchor, int(relation[1:]))
            values = [r[0] for r in source.execute(f"SELECT 'Q'||{field} FROM edges WHERE {predicate}=? AND property=? GROUP BY {field} ORDER BY 1 LIMIT 10", arguments)]
            total = source.execute(f'SELECT count(*) FROM (SELECT {field} FROM edges WHERE {predicate}=? AND property=? GROUP BY {field})', arguments).fetchone()[0]
            assert total > 0
        cases.append(dict(question=question, entity=resolved, relation=relation, answers=values, total=total))
    case_path = ROOT / 'witness-cases.json'
    case_path.write_text(json.dumps(cases, ensure_ascii=False, indent=2) + '\n')
    result = subprocess.run([str(ROOT / 'disk-witness-probe'), str(ROOT / 'native.sqlite'), str(case_path)],
                            check=True, capture_output=True)
    report = json.loads(result.stdout)
    report.update(fixed_raw_witnesses=len(witnesses), missing_names=missing_names, over_query_limit=over_limit,
                  seconds=time.monotonic() - start, case_sha256=hashlib.sha256(case_path.read_bytes()).hexdigest(),
                  binary_sha256=hashlib.sha256((ROOT / 'disk-witness-probe').read_bytes()).hexdigest())
    (ROOT / 'witness-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
