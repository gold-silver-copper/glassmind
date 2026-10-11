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
