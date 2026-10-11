# Full claims independent source validation

Frozen before full projection/native completion or query probes. No QA input.
This specifies implementation details of PLAN.md's raw-source checks.

Audit within3600seconds,8GiB/no swap, existing384GiB added-run cap. Independently
read positions0,last,and each100003rd row in every Parquet file using full raw
qualifiers; do not call project.py's selector. Compare eligibility and the entire
record against BOTH the numeric projection and final native witness. Eligible
triples must exist in native edges; excluded positions must have no witness.
Check all file hashes and row offsets again, exact rank/qualifier counts from
projection reports against native aggregates, and native metadata totals. Compare
native alias/occurrence totals and source-derived alias samples to the pinned
base. Obtain independent distinct triple count from the projection, not metadata.
Record indexed plans, full output hash, process RSS and wall time.

Query probes: use the first eligible deterministic raw sample from EACH shard;
probe both forward and inverse directions when that anchor has a source name.
Choose the first name by original pilot(kind,language,row_id), independent of QA.
Record missing names, native alias ambiguities and query-size exclusions. Derive
answer cardinality/first10 values independently from the complete numeric claims
projection in bounded disk-backed scans. A synthetic per-case model prescribes
the property as in the prior native pilot; this tests query APIs, not English QA.

Preserve existing query VM budgets. If a large set exhausts that budget, record
that bounded API error separately with the independent expected cardinality;
do not count it as a correct set response or increase query limits for this run.
Any successful ordinary/set response must match its independent reference.
Require positive ordinary and set responses in addition to index plans; report
all cases, including large-cardinality limitations. Do not skip inconvenient
cases by looking at the output. Keep startup/latency observations scoped to the
actual local batch, not a broad cold-cache performance claim.

Replay unchanged format1 graph on19481TRAIN+2821DEV ordinary/set predictions,
and format2 alias/query source fixtures, using the new reader. These are old
artifact regressions, not a QA evaluation of the full new graph. Preserve main
mixed28cases in both modes and required workspace checks. A later QA promotion
experiment requires a separate frozen gate; none has been run for full claims.
