#!/usr/bin/env python3
"""Pin completed projection and the reviewed native binary before import."""
import hashlib
import json
from pathlib import Path
import shutil
import sys


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f,'sha256').hexdigest()


def main():
    root,binary=map(lambda p:Path(p).resolve(),sys.argv[1:])
    assert digest(binary)=='b0a86c031e6c83d81af3a5672391f6385b9ec7eb1205743d4d79a7c5677cded7'
    # This is the build that passed the frozen format1/2 compatibility checks.
    projection=json.loads((root/'projection-report.json').read_text())
    resources=json.loads((root/'projection-resources.json').read_text())
    assert resources['passed'] and resources['seconds']<14400
    assert projection['source_rows']==789435443
    assert projection['canonical_retained_ids']==3*projection['statements']
    acquired=json.loads((root/'acquisition-report.json').read_text())
    assert acquired['all_publisher_hashes_match'] and acquired['source_bytes']==44633986614
    assert acquired['acquisition_sha256']==digest(root/'acquisition.json')
    acquisition=json.loads((root/'acquisition.json').read_text())
    assert projection['acquisition_sha256']==digest(root/'acquisition.json')
    assert len(acquisition['files'])==len(projection['files'])==86
    for f,record in zip(acquisition['files'],projection['files'],strict=True):
        assert all(record['source'][k]==v for k,v in f.items() if k!='url')
        assert sum(record['counts'].values())==record['source']['source_rows']
        assert sum(record['ranks'].values())==record['source']['kept_rows']
    path=root/'claims.sqlite'
    assert path.stat().st_size==projection['projection_bytes']<96*1024**3
    assert digest(path)==projection['projection_sha256']
    names=root.parent/'wikidata-native/native.sqlite'
    assert names.stat().st_size==12306595840
    names_hash=digest(names)
    assert names_hash=='f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b'
    base=json.loads((root.parent/'wikidata-native/source-manifest.json').read_text())
    manifest=dict(schema=1,scope='All86 public claims shards plus unchanged en/mul native names; publisher excludes scholarly works',
                  names_sha256=names_hash,names_bytes=names.stat().st_size,names_source=base,
                  projection_sha256=projection['projection_sha256'],projection_bytes=projection['projection_bytes'],
                  canonical_audit_sha256=digest(root/'projection-report.json'),
                  statements=projection['statements'],qualified_statements=projection['qualified_statements'],
                  files=[record['source'] for record in projection['files']])
    with (root/'source-manifest.json').open('x') as f:f.write(json.dumps(manifest,indent=2)+'\n')
    # Reserve destination so no previous evaluated binary can be overwritten.
    with (root/'candidate').open('xb') as out, binary.open('rb') as src:shutil.copyfileobj(src,out)
    (root/'candidate').chmod(0o555)
    command=[str(root/'candidate'),'import-wikidata-claims','--names',str(names),'--claims',str(path),
             '--manifest',str(root/'source-manifest.json'),'--database',str(root/'native.sqlite')]
    with (root/'native-invocation.json').open('x') as f:
        f.write(json.dumps(dict(command=command,binary_sha256=digest(root/'candidate'),
                  manifest_sha256=digest(root/'source-manifest.json'),seconds_cap=21600,
                  output_bytes_cap=192*1024**3,added_run_bytes_cap=384*1024**3),indent=2)+'\n')
    print(json.dumps(dict(command=command,statements=projection['statements'],qualified=projection['qualified_statements'])),flush=True)


if __name__=='__main__':main()
