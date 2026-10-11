#!/usr/bin/env python3
"""Independent raw/source/native accounting, with no project.py or QA inputs."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import resource
import sqlite3
import sys
import time

import pyarrow.parquet as pq


def digest(path):
    with path.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()


def readonly(path):
    db=sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True)
    db.executescript('PRAGMA query_only=ON; PRAGMA cache_size=-65536; PRAGMA temp_store=FILE; PRAGMA mmap_size=0;')
    return db


def main():
    root=Path(sys.argv[1]).resolve()
    started=time.monotonic()
    manifest=json.loads((root/'source-manifest.json').read_text())
    projection=json.loads((root/'projection-report.json').read_text())
    assert digest(root/'projection-report.json')==manifest['canonical_audit_sha256']
    numeric=readonly(root/'claims.sqlite')
    native=readonly(root/'native.sqlite')
    base=readonly(root.parent/'wikidata-native/native.sqlite')
    file_checks=[];witnesses=[];ranks=Counter();offset=0;statement_checks=0
    for spec,prior in zip(manifest['files'],projection['files'],strict=True):
        assert spec==prior['source'] and spec['row_id_offset']==offset
        ranks.update(prior['ranks'])
        path=root/'raw/claims'/spec['path']
        assert path.stat().st_size==spec['bytes'] and digest(path)==spec['sha256']
        raw=pq.ParquetFile(path)
        assert raw.metadata.num_rows==spec['source_rows']
        positions={0,spec['source_rows']-1,*range(0,spec['source_rows'],100003)}
        within=checked=kept=0;first=None
        for group in range(raw.num_row_groups):
            nrows=raw.metadata.row_group(group).num_rows
            wanted=sorted(p for p in positions if within<=p<within+nrows)
            if wanted:
                table=raw.read_row_group(group,columns=['id','property','datavalue.id','datatype','rank','snaktype','statement_id','qualifiers'],use_threads=False)
                for ordinal in wanted:
                    row=table.slice(ordinal-within,1).to_pylist()[0]
                    row_id=offset+ordinal+1
                    eligible=bool(re.fullmatch(r'Q[0-9]+',row['id'])) and row['rank'] in ('normal','preferred') and row['snaktype']=='value' and row['datatype']=='wikibase-item'
                    a=numeric.execute('SELECT * FROM edges WHERE row_id=?',(row_id,)).fetchone()
                    b=native.execute('SELECT * FROM statements WHERE source_row=?',(row_id,)).fetchone()
                    if eligible:
                        expected=(row_id,row['id'],row['property'],row['datavalue']['id'],int(row['rank']=='preferred'),int(bool(row['qualifiers'])),row['statement_id'])
                        assert b==expected,(spec['path'],ordinal,b,expected)
                        numbers=(row_id,*(int(x[1:]) for x in expected[1:4]),*expected[4:])
                        assert a==numbers,(spec['path'],ordinal,a,numbers)
                        assert native.execute('SELECT 1 FROM edges WHERE subject=? AND property=? AND object=?',expected[1:4]).fetchone()==(1,)
                        kept+=1;statement_checks+=1
                        if first is None:first=dict(file=spec['path'],ordinal=ordinal,row_id=row_id,subject=expected[1],property=expected[2],object=expected[3])
                    else:
                        assert a is None and b is None,(spec['path'],ordinal)
                    checked+=1
            within+=nrows
        assert checked==len(positions)
        offset+=spec['source_rows']
        file_checks.append(dict(path=spec['path'],samples=checked,eligible=kept))
        if first is not None:witnesses.append(first)
        print(json.dumps(dict(file=spec['path'],samples=checked,eligible=kept,seconds=time.monotonic()-started)),flush=True)
    assert offset==789435443
    observed_ranks={f'{a},{b}':n for a,b,n in native.execute('SELECT preferred,qualified,count(*) FROM statements GROUP BY preferred,qualified')}
    assert dict(ranks)==observed_ranks
    meta=dict(native.execute('SELECT key,value FROM metadata'));old_meta=dict(base.execute('SELECT key,value FROM metadata'))
    assert json.loads(meta['public_claims'])==manifest
    assert int(meta['statements'])==sum(ranks.values())==projection['statements']
    assert int(meta['qualified_statements'])==sum(n for k,n in ranks.items() if k.endswith(',1'))
    aliases,occurrences=native.execute('SELECT count(*),sum(occurrences) FROM aliases').fetchone()
    assert aliases==int(meta['aliases'])==int(old_meta['aliases'])==74983525
    assert occurrences==int(meta['accepted_name_rows'])==int(old_meta['accepted_name_rows'])==84955531
    assert meta['excluded_short_name_rows']==old_meta['excluded_short_name_rows']=='34481'
    # Retained raw-derived alias sample from the prior independently audited pilot.
    raw_view=json.loads((root.parent/'wikidata-native/raw-sample-view.json').read_text())
    checked_names=0
    for entity,value in raw_view['entities'].items():
        for alias in value['aliases']:
            sql='SELECT normalized,uppercase,source_mask,occurrences,first_source_row FROM aliases WHERE entity=? AND original=?'
            assert native.execute(sql,(entity,alias)).fetchone()==base.execute(sql,(entity,alias)).fetchone()
            checked_names+=1
    distinct=numeric.execute('SELECT count(*) FROM (SELECT subject,property,object FROM edges GROUP BY subject,property,object)').fetchone()[0]
    assert distinct==native.execute('SELECT count(*) FROM edges').fetchone()[0]==int(meta['edges'])
    numeric.execute('ATTACH DATABASE ? AS names',((root.parent/'wikidata-native/native.sqlite').resolve().as_uri()+'?mode=ro',))
    expected_entities=numeric.execute("SELECT count(*) FROM (SELECT id FROM names.entities UNION SELECT 'Q'||subject FROM edges)").fetchone()[0]
    assert expected_entities==native.execute('SELECT count(*) FROM entities').fetchone()[0]==int(meta['entities'])
    plans={}
    for name,sql,args,index in [
        ('forward','SELECT object FROM edges WHERE subject=? AND property=?',('Q100','P31'),'PRIMARY KEY'),
        ('inverse','SELECT subject FROM edges WHERE property=? AND object=?',('P31','Q5'),'edges_inverse'),
        ('alias','SELECT entity FROM aliases WHERE normalized=?',('london',),'aliases_lookup')]:
        plan=native.execute('EXPLAIN QUERY PLAN '+sql,args).fetchall()
        assert 'SEARCH' in str(plan) and index in str(plan)
        plans[name]=plan
    report=dict(files=file_checks,raw_samples=sum(f['samples'] for f in file_checks),statement_samples=statement_checks,
                alias_samples=checked_names,entities=expected_entities,aliases=aliases,edges=distinct,ranks=observed_ranks,
                query_plans=plans,database_bytes=(root/'native.sqlite').stat().st_size,database_sha256=digest(root/'native.sqlite'),
                seconds=time.monotonic()-started,peak_rss_kib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
                limitation='Independent source/storage checks, not QA accuracy')
    assert report['seconds']<3600 and report['database_bytes']<192*1024**3
    (root/'audit-report.json').write_text(json.dumps(report,indent=2)+'\n')
    (root/'raw-witnesses.json').write_text(json.dumps(witnesses,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='files'}),flush=True)


if __name__=='__main__':main()
