# Verify a recorded relationship on a date

Glassmind can check whether one loaded statement supports an employer or citizenship relationship on an exact Gregorian calendar day:

```text
Check fact on 2011-01-01: Did Iris Vale work for Acorn Lab?
```

Use `glassmind talk --facts-dir BUNDLE`. This example uses the authored development graph; it is not a real-world assertion. The command accepts explicit names or Wikidata IDs and does not bind pronouns from conversation history. It preserves personal memory and story state.

The standalone equivalent is:

```sh
glassmind-facts verify-at --index BUNDLE/index.json \
  --temporal BUNDLE/temporal.json --date 2011-01-01 \
  'Did Iris Vale work for Acorn Lab?'
```

The optional `temporal_sha256` field in the existing bundle manifest declares the exact SHA-256 of `temporal.json`. Undeclared files are ignored. A declared missing, changed, incompatible, or unsupported asset prevents the bundle from loading. Without declared temporal data, the dated command refuses explicitly. Existing undated commands retain their behavior.

Only a strict interior is supported: the latest possible start day must precede the requested day, which must precede the earliest possible end day. Year and month precision describe whole ranges of possible days even when a timestamp contains filled lower fields. Boundary days, missing bounds, unsupported qualifiers, and absent records yield uncertainty, not a negative factual assertion. Both bounds must belong to the same original statement.

Supported source statements have one P580 start and one P582 end, no other qualifiers, non-deprecated rank, Gregorian dates in years 0001–9999, year/month/day precision, and zero timezone and uncertainty fields. Unsupported claims remain preserved in the metadata and carry explicit skip reasons when considered. The answer includes the exact directed edge, claim ID, raw bound values, derived day ranges, source location, raw-source hash, and bound index hash.

This establishes evidence in one recorded interval from a finite snapshot. It does not establish current truth, complete historical coverage, or a UTC instant.

`tools/facts/temporal/extract.py` builds the metadata from hash-verified raw Wikidata responses and the matching static index. See its `--help` and [reproduction script](../tools/facts/temporal/reproduce.sh). Metadata is capped at 16 MiB and 100,000 records; the bound static index is capped at 32 MiB. Production bundles should retain source manifests and original response bytes outside git.

Validation and limitations are recorded in the [parent review](../tools/facts/temporal/PARENT.md). The cached real-data slice inherits the existing 500 validation-subject-seeded index; it is not an independent open-world retrieval benchmark.

## Find subjects matching dated conditions

Use two or three positive employer or citizenship conditions on the same day:

```text
Match facts on 2012-01-01: Who worked for Amber Lab and was a citizen of North Republic?
```

These names belong to the authored development graph. The standalone command is
`glassmind-facts match-at --index BUNDLE/index.json --temporal BUNDLE/temporal.json --date 2012-01-01 'Who worked for Amber Lab and was a citizen of North Republic?'`.

Every condition needs its own eligible original claim with a strict interval
containing that day. Distinct employers or citizenships are supported. Duplicate
conditions are idempotent. Birthplace, place of death, negation, and ambiguous or
missing names cannot establish a dated match. Graph membership and dates never
choose between ambiguous names.

Glassmind checks the complete intersection before counting and displaying up to
ten subjects, sorted by ID. JSON uses `kind: fact_matches_at` and reports
`total_recorded_interval_matches`, `returned`, `truncated`, the query date and all
displayed claim witnesses. Zero matches means the loaded records do not provide
complete interval evidence for all conditions; it does not establish absence in
the world. Personal memory and story state are preserved. See the
[dated matching review](../tools/facts/timed-matching/PARENT.md) for evidence and limits.
