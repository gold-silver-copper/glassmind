#!/usr/bin/env python3
"""Check every source identifier and name language, independently of importer."""
import json
from pathlib import Path
import sys
import time

import pyarrow.compute as pc
import pyarrow.parquet as pq


def main():
    root = Path(sys.argv[1]).resolve()
    manifest = json.loads((root / 'pilot-manifest.json').read_text())
    started = time.monotonic()
    results = []
    for spec in manifest['files']:
        parquet = pq.ParquetFile(root / 'raw' / spec['table'] / spec['path'])
        counts = {'rows': 0, 'invalid_ids': 0, 'invalid_properties': 0,
                  'invalid_language': 0, 'null_names': 0}
        claims = spec['table'] == 'claims'
        columns = ['id', 'property'] if claims else ['id', 'language', 'value']
        for batch in parquet.iter_batches(batch_size=65536, columns=columns, use_threads=False):
            ids = batch.column('id')
            valid_ids = pc.fill_null(pc.match_substring_regex(ids, '^[QP][0-9]+$'), False)
            counts['invalid_ids'] += batch.num_rows - pc.sum(valid_ids).as_py()
            if claims:
                valid_props = pc.fill_null(pc.match_substring_regex(batch.column('property'), '^P[0-9]+$'), False)
                counts['invalid_properties'] += batch.num_rows - pc.sum(valid_props).as_py()
            else:
                valid_lang = pc.fill_null(pc.equal(batch.column('language'), spec['path'].split('/')[0]), False)
                counts['invalid_language'] += batch.num_rows - pc.sum(valid_lang).as_py()
                counts['null_names'] += batch.column('value').null_count
            counts['rows'] += batch.num_rows
        assert counts['rows'] == parquet.metadata.num_rows
        assert counts['invalid_ids'] == counts['invalid_properties'] == counts['invalid_language'] == 0
        results.append({'table': spec['table'], 'path': spec['path'], **counts})
        print(json.dumps(results[-1]), flush=True)
    report = {'files': results, 'seconds': round(time.monotonic() - started, 3)}
    (root / 'source-audit.json').write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    main()
