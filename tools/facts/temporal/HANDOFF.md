# Temporal recorded-interval verification — bounded worker handoff

Candidate complete, uncommitted. Source base
`07f3d0d00233e1c9f0847d410ebcdafc1e5549b9`. No parent acceptance or main
conversation integration is claimed. Changes are confined to `crates/facts` and
new `tools/facts/temporal`. No dependencies, downloads, model training, agents,
services, runtime LLM calls, answer tables, shared CLI changes or scorer changes.

## Supported semantics and exact changes

New public `glassmind_facts::temporal` module exposes `CalendarDate::parse`,
`TemporalIndex::from_bytes`, typed `verify_at(&CalendarDate, question)` and
`verify_at_str(date, question)`, with typed supported/unknown/refused outcomes.
New standalone `glassmind-facts verify-at --index INDEX --temporal TEMPORAL
--date YYYY-MM-DD QUESTION` needs no relation model.

A supported result means **one original recorded bounded P108 employer or P27
citizenship statement strictly contains the requested calendar day under the
conservative precision interpretation**. It does not establish current validity,
global/world truth, completeness, or that excluded dates are false. Policy:
`latest_start_day < requested_day < earliest_end_day`. Year/month/day precision
uses the entire Gregorian year/month/day as possible dates, including when lower
fields are filled. Bound days themselves are unknown. No UTC instant inference.

The separate immutable temporal object owns its exact SHA-bound static store.
The legacy exact positive question grammar and alias resolution run before
statement selection; ambiguity cannot be resolved by an edge or interval.
Explicit IDs and edge-only object IDs retain legacy handling. The only existing
verification source changes expose its parser and property internally; legacy
Verify/Match/Ask/AnswerSet/Score implementation behavior is preserved.

Loader validates version1, identifiers, ranks, claim ID/subject agreement,
mainsnak triple, static edge membership, provenance syntax, raw statement size,
and exact static-index hash. It rejects duplicate JSON object keys and conflicting
case-insensitive claim IDs; identical whole records collapse. Witness selection
sorts case-insensitive statement IDs. Historical lowercase-q statement IDs are
accepted and preserved; endpoint QIDs remain canonical uppercase.

The asset retains each whole original statement including **all qualifiers**.
Rust derives ranges from raw time values and trusts no precomputed bounds.
Missing/open/nonvalue/duplicate bounds, extra qualifiers, deprecated statements,
invalid fields, unsupported years/calendars/precisions, nonzero timezone or
uncertainty, reversed intervals and dates outside the strict interior return
explicit skip reasons and unknown unless another single statement supports.
Two open statements cannot combine. Structurally corrupt assets are loader errors.

Bounds: exact ASCII request YYYY-MM-DD, years0001..9999; raw times
+YYYY-MM-DDT00:00:00Z, precision9/10/11 and canonical Gregorian URI
http://www.wikidata.org/entity/Q1985727. Filled subordinate fields must themselves
be structurally valid; day with zero month is rejected. Static index cap32MiB,
metadata16MiB,100000 records,64KiB serialized statement; query4096bytes and
endpoint256bytes. Extractor cap128 files,8MiB/file,64MiB aggregate,1MiB manifest.

## Measurement and retained failures

`PLAN.md`, generator, authored fixture index and **56 contracts were frozen before
runtime editing**; their original hashes still match `frozen.sha256`. Missing old
verify-at is capability absence, not an accuracy baseline.

- All20 facts test functions pass, including the existing30 parent matching
  contracts and unchanged verification/matching/AnswerSet contracts. Four new
  temporal tests exercise56 frozen cases, actual byte/record limits, loader binding,
  schema, duplicates, deterministic selection, historical IDs and typed reasons.
- CLI/library full JSON equality:56 authored cases; all8 authored positives compare
  complete independently constructed witnesses. Invalid dates remain refusals;
  missing/ambiguous aliases remain distinct unknown outcomes.
- Fixed real sample:3 distinct subjects,15 queries; interior supported and both
  boundary/outside sides unknown. Complete witnesses independently reconstructed
  directly from hash-verified ORIGINAL bytes, not from extractor output.
- Full affected cached temporal dev:20 qualifying statements across7 subjects,
  100 queries;54 supported,46 unknown,100/100 match the independent datetime
  reference and CLI/library output. Overlapping statements may support a day that
  is another statement's boundary; one qualifying statement suffices by policy.
- All308 extracted records equal original raw statements exactly. Five extractor
  integrity contracts pass: deterministic bytes, preserved extra qualifier,
  corrupted raw hash rejected, index hash mismatch rejected, duplicate raw keys
  rejected. No eligibility filtering can silently erase adverse qualifiers.
- Unchanged `tools/facts/verification/legacy.py`: first200 then full2821 predictions
  AND actual native Score bytes equal baseline. First20012/188/0; full ordinary
  **256 right /2545 refused /20 wrong**, joint **252/2545/24** unchanged.
  Full predictions270305bytes SHA256
  `9d7fb56e29d8dd52275c93d8edd14c921c5748d8b4c3bc48c7fc7008574005b1`.
- Additional25 direct old/new native command comparisons pass:11 Verify,6 Match,
  8 Ask. Those8 Ask outputs also equal the pre-edit diagnostic predictor baseline.
- Facts fmt, all-target clippy with warnings denied, debug binaries/examples build,
  diff whitespace checks pass. Every cargo invocation used supplied persistent
  TMPDIR/target, systemd8GiB/0swap scope, setsid --wait,600s timeout/10s kill grace,
  -j1 except fmt, test threads4. Outer8GiB/0swap cap independently checked.

Original failures are retained, not erased: real loading initially rejected
historical lowercase-q statement IDs; first clippy rejected guarded indexing;
checked-access rewrite initially caused two borrowed JSON/String comparison errors;
clippy then flagged test-helper unwrap/indexing; a strengthened record-cap test
initially used null records and hit schema rejection. `EXPERIMENTS.md` explains
fixes. No temporal-policy experiment was accepted/rejected to optimize scores.
Superseding successful logs: tests-complete.log,clippy-complete.log,
fmt-complete.log,build-final.log,proof-final.log,legacy-complete.log.

## Sources, versions, bytes and licenses

No newly downloaded data. Supplied historical20 Wikidata response files total
**27696032 bytes**, CC0. Their hash manifest3375bytes has SHA256
`78d05da449945acd9d098e8742fd29c8ebb741ca531077d51510d98f3f2524df`.
Responses omit lastrevid/modified, so exact raw-file hashes identify the source
version; no more precise revision/date is asserted. Cache is partial and seeded
with historical validation subjects (500 indexed entities), explicitly **not an
independent retrieval benchmark**. There are24 nondeprecated two-bound P108/P27
claims;20 have the supported qualifier/time shape. Metadata retains308 indexed
P108/P27 statements, including ineligible ones; one nonindexed source claim skipped.

Derived metadata `run/temporal.json`: **268153 bytes**, CC0, SHA256
`55097953db9b0f041ec4a7c2b5a7513586b1db9f572856cb84aa8610f484ee3d`.
Static index287236bytes SHA256
`923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`.
Relation model35773124bytes SHA256
`db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
SimpleQuestions source revision `df69267f1db4f85a71325e7e8c06e192bcaf0b0e`,
questions CC BY3, Wikidata CC0. Supplied train1193868bytes/19481rows and
valid173187bytes/2821rows; only valid used for legacy replay, no training.
Source/license/manifests are copied into the frozen provenance directory.
Authored fixtures are development contracts, not independent benchmark data.

Three raw witnessed statements (exact details and date vectors in real-proof.json):

| Subject | Statement UUID (subject prefix omitted) | Raw response SHA256 |
|---|---|---|
| Q1029283 | 0b0bde31-4b47-db91-8556-e2f83caa5f75 | 0ac074d1a50473a0e453a8b9cd4ba1a6a9fa5926ae8b683d169d603483d4f332 |
| Q1157672 | 47C9D3C9-F13F-4F3B-BC04-948839501583 | 80d3edde986ee530f3db13c9a03df07760a3c0a23f5b075ed7d73fb3e7f21df9 |
| Q123190 | 183BC9DB-725C-497E-AA0D-0773E3E5821C | 33a3aaa16f4ccb181ef9990468064328d49b58ab1c680aed8c238ceb97336188 |

Parent-supplied primary documentation citations are preserved in frozen PLAN.md.
Raw hashes authenticate bytes against the supplied manifest, not against a live
Wikidata service. Rust validates provenance syntax and index binding; it does not
fetch/re-hash original response files at runtime. Assets should be produced with
the supplied hash-verifying offline extractor. No authoritative personal claims.

## Reproduction and frozen candidate

From this checkout run `bash tools/facts/temporal/reproduce.sh`. It uses sibling
`temporal-run`, `temporal-source`, and `factmatch-source`; RUN can override the
run directory containing the retained baseline. All cargo commands are capped.
The baseline diagnostic predictor was compiled from untouched pre-edit facts
source, and native Score uses the supplied actual baseline standalone binary.
Baseline identity and hashes are in reports/baseline-identity.json. No main binary
was changed; parent's later conversation/memory/story integration gates remain.

Example (paths relative to checkout):

```sh
../temporal-run/frozen-candidate/binary/glassmind-facts verify-at \
  --index ../factmatch-source/index.json \
  --temporal ../temporal-run/frozen-candidate/temporal.json \
  --date 1980-06-01 'Did Q168041 work for Q49088?'
```

Freeze destination:
`/home/gsc/.cache/glassmind/continuous-20261010/temporal-run/frozen-candidate`.
It contains source, new tools/contracts, complete patch including new files,
binaries, extracted metadata, reports including original failures, baseline,
source/license manifests and a SHA256/byte manifest verified after copying.
Candidate debug standalone26880056bytes SHA256
`6ebd8026fe05398b31c2e680ff97f1a33928e2c1a0a46bd760fd59c1ecf46bec`.
Library proof executable14744104bytes SHA256
`0e5bbf2a57565658f2f557f2bec94a4644876869180af572a43a90f11b0a244c`.

No jobs are left running by the worker. No commit, main-bundle integration,
held-out evaluation or parent temporal acceptance was performed. Future explicit
conversation routing belongs to the parent after independent acceptance.
