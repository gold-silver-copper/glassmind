#!/usr/bin/env python3
"""Freeze independent raw expected rows before the native importer runs."""
import hashlib
import json
from pathlib import Path
import re
import resource
import sys
import time

import pyarrow as pa
import pyarrow.parquet as pq


def digest(path):
    with path.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()


def main():
    root=Path(sys.argv[1]).resolve();started=time.monotonic()
    assert digest(root/'acquisition.json')=='e29894e98c4001db53ca5b93d9420e82e4736fd2985c28bd8251c899d2718b18'
    assert pa.__version__=='25.0.1'
    acquisition=json.loads((root/'acquisition.json').read_text())
    files=[];samples=[];offset=0
    for spec in acquisition['files']:
        path=root/'raw'/spec['table']/spec['path']
        assert path.stat().st_size==spec['bytes'] and digest(path)==spec['sha256']
        parquet=pq.ParquetFile(path);rows=parquet.metadata.num_rows
        positions={0,rows-1,*range(0,rows,100003)}
        within=checked=kept=0
        for group in range(parquet.num_row_groups):
            nrows=parquet.metadata.row_group(group).num_rows
            wanted=sorted(p for p in positions if within<=p<within+nrows)
            if wanted:
                table=parquet.read_row_group(group,columns=['id','property','datavalue.id','datatype','rank','snaktype','statement_id','qualifiers'],use_threads=False)
                for ordinal in wanted:
                    raw=table.slice(ordinal-within,1).to_pylist()[0]
                    row_id=offset+ordinal+1
                    eligible=bool(re.fullmatch(r'Q[0-9]+',raw['id'])) and raw['rank'] in ('normal','preferred') and raw['snaktype']=='value' and raw['datatype']=='wikibase-item'
                    expected=None
                    if eligible:
                        expected=[row_id,raw['id'],raw['property'],raw['datavalue']['id'],int(raw['rank']=='preferred'),int(bool(raw['qualifiers'])),raw['statement_id']]
                        kept+=1
                    samples.append(dict(file=spec['path'],ordinal=ordinal,row_id=row_id,expected=expected))
                    checked+=1
            within+=nrows
        assert checked==len(positions) and within==rows
        files.append(dict(path=spec['path'],sha256=spec['sha256'],bytes=spec['bytes'],source_rows=rows,row_id_offset=offset,samples=checked,eligible=kept))
        offset+=rows
        print(json.dumps(dict(path=spec['path'],samples=checked,eligible=kept,seconds=time.monotonic()-started)),flush=True)
        if time.monotonic()-started>=600:raise RuntimeError('raw reference time cap exceeded')
    assert len(files)==86 and offset==789435443
    reference=dict(schema=1,source_rows=offset,files=files,samples=samples)
    with (root/'raw-reference.json').open('x') as f:f.write(json.dumps(reference,indent=2)+'\n')
    report=dict(source_rows=offset,files=len(files),samples=len(samples),eligible=sum(f['eligible'] for f in files),
                seconds=time.monotonic()-started,peak_rss_kib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
                source_manifest_sha256=digest(root/'acquisition.json'),reference_sha256=digest(root/'raw-reference.json'),
                script_sha256=digest(Path(__file__)),pyarrow=pa.__version__,python=sys.version)
    assert report['seconds']<600
    with (root/'reference-report.json').open('x') as f:f.write(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report),flush=True)


if __name__=='__main__':main()
