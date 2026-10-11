#!/usr/bin/env python3
"""Source-derived API probes; complete independent numeric scan, no QA."""
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time


def connect(path):
    db=sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True)
    db.executescript('PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;')
    return db


def main():
    root=Path(sys.argv[1]).resolve();started=time.monotonic()
    numeric=connect(root/'claims.sqlite');native=connect(root/'native.sqlite')
    names=connect(root.parent/'wikidata-snapshot/pilot.sqlite')
    witnesses=json.loads((root/'raw-witnesses.json').read_text())
    cases=[];missing=over_limit=ambiguous=0
    for w in witnesses:
        for entity,relation in [(w['subject'],w['property']),(w['object'],'R'+w['property'][1:])]:
            raw=names.execute('SELECT name FROM names WHERE entity=? ORDER BY kind,language,row_id LIMIT 1',(int(entity[1:]),)).fetchone()
            if raw is None:missing+=1;continue
            question=raw[0]
            key=native.execute('SELECT normalized FROM aliases WHERE entity=? AND original=?',(entity,question)).fetchone()
            if len(question.encode())>4096 or (key is not None and len(key[0].split())>128):over_limit+=1;continue
            matches=[] if key is None else native.execute('SELECT DISTINCT entity FROM aliases WHERE normalized=? LIMIT 2',(key[0],)).fetchall()
            resolved=entity if matches==[(entity,)] else None
            ambiguous+=resolved is None
            cases.append(dict(question=question,entity=resolved,relation=relation,answers=[],total=0))
    # Projection has no secondary indexes. Scan it twice, with tiny indexed
    # desired tables on the inside of CROSS JOIN, rather than once per question.
    numeric.executescript('''CREATE TEMP TABLE desired_forward(anchor INTEGER,property INTEGER,case_id INTEGER,PRIMARY KEY(anchor,property,case_id)) WITHOUT ROWID;
        CREATE TEMP TABLE desired_inverse(anchor INTEGER,property INTEGER,case_id INTEGER,PRIMARY KEY(anchor,property,case_id)) WITHOUT ROWID;
        CREATE TEMP TABLE found(case_id INTEGER,answer TEXT,PRIMARY KEY(case_id,answer)) WITHOUT ROWID;''')
    groups={};case_groups={}
    for i,c in enumerate(cases):
        if c['entity'] is None:continue
        table='desired_forward' if c['relation'].startswith('P') else 'desired_inverse'
        key=(table,int(c['entity'][1:]),int(c['relation'][1:]))
        if key not in groups:
            groups[key]=len(groups)
            numeric.execute(f'INSERT INTO {table} VALUES (?,?,?)',(*key[1:],groups[key]))
        case_groups[i]=groups[key]
    numeric.executescript("""INSERT OR IGNORE INTO found SELECT d.case_id,'Q'||e.object FROM edges e CROSS JOIN desired_forward d WHERE e.subject=d.anchor AND e.property=d.property;
        INSERT OR IGNORE INTO found SELECT d.case_id,'Q'||e.subject FROM edges e CROSS JOIN desired_inverse d WHERE e.object=d.anchor AND e.property=d.property;""")
    results={}
    for group in groups.values():
        total=numeric.execute('SELECT count(*) FROM found WHERE case_id=?',(group,)).fetchone()[0]
        answers=[r[0] for r in numeric.execute('SELECT answer FROM found WHERE case_id=? ORDER BY answer LIMIT 10',(group,))]
        results[group]=(total,answers)
    for i,c in enumerate(cases):
        if i in case_groups:c['total'],c['answers']=results[case_groups[i]]
        assert c['entity'] is None or c['total']>0
    case_path=root/'witness-cases.json'
    case_path.write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
    result=subprocess.run([str(root/'disk-witness-probe'),str(root/'native.sqlite'),str(case_path),'--report-set-limits'],check=True,capture_output=True,timeout=300)
    report=json.loads(result.stdout)
    report.update(witnesses=len(witnesses),distinct_lookup_groups=len(groups),missing_names=missing,over_query_limit=over_limit,ambiguous=ambiguous,
                  independent_largest_cardinality=max(c['total'] for c in cases),seconds=time.monotonic()-started,
                  cases_sha256=hashlib.sha256(case_path.read_bytes()).hexdigest(),
                  binary_sha256=hashlib.sha256((root/'disk-witness-probe').read_bytes()).hexdigest())
    (root/'witness-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)


if __name__=='__main__':main()
