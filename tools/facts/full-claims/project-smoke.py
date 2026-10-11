"""Compare new raw selection with retained, independently audited pilot rows."""
import importlib.util
import json
from pathlib import Path
import sqlite3
import sys
import time
import pyarrow.parquet as pq

here=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('project',here/'project.py')
project=importlib.util.module_from_spec(spec);spec.loader.exec_module(project)
source=Path(sys.argv[1]).resolve()
started=time.monotonic()
db=sqlite3.connect(f'{(source/"pilot.sqlite").as_uri()}?mode=ro',uri=True)
expected={r[0]:r for r in db.execute('SELECT * FROM edges WHERE row_id<=500000')}
raw=pq.ParquetFile(source/'raw/claims/all/part-00-of-86.parquet')
ordinal=kept=0
for batch in raw.iter_batches(batch_size=16384,columns=project.COLUMNS,use_threads=False):
    for row in batch.to_pylist():
        ordinal+=1
        got,_=project.select(row,ordinal)
        assert got==expected.get(ordinal),(ordinal,got,expected.get(ordinal))
        kept+=got is not None
        if ordinal==500000:break
    if ordinal==500000:break
assert kept > 0
# Retained identifiers cannot collapse leading zeros or overflow signed storage.
for value in ['Q01','Q0','Q9223372036854775808']:
    try:project.canonical(value,'Q')
    except ValueError:pass
    else:raise AssertionError(value)
print(json.dumps(dict(raw_rows=ordinal,kept=kept,seconds=time.monotonic()-started,result='exact pilot parity; canonical boundary checks pass')))
