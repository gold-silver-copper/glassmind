# Positive local-edge verification — completed, uncommitted

Base `bb619fbaa7d9e0e58dfd76bc6fa9cb72fe9fbe4a`. Only `crates/facts` and new
`tools/facts/verification` changed. Coordinator integrates. No commits, downloads,
new model/LLM calls, agents, API keys, credential access, services, index expansion,
training, scorer changes, held-out items/code/generators, or old private logs.
Inherited serving model cannot be independently attested or switched by this worker.
Outer scope memory.max=8589934592 and memory.swap.max=0 verified. All owned commands
completed; no background jobs remain.

## Exact changes

- `crates/facts/src/verification.rs`: new read-only `Store::verify(&str) -> Verification`.
  Whole positive grammar for P19/P20/P27/P108; grammar/tense/politeness and exclusions
  frozen in PLAN before implementation. No model required. Unique full endpoint
  aliases preserve normalization, acronym casing veto and ambiguity-before-casing;
  validated explicit QIDs are optional. Full directed object membership, including
  objects beyond display10 and edge-only object IDs. No property-presence shortcut.
- `Supported` carries exact subject/property/object, each endpoint's resolution
  (`alias`/`explicit_id`), subject#property source URL and `scope: local_snapshot`.
  Typed `Unknown` covers missing/ambiguous endpoint or absent edge; typed `Refused`
  covers unsupported/bounded/ambiguous syntax. Neither absence nor another target
  becomes False/No. Store is not mutated, no conversation state is consulted.
- `crates/facts/src/lib.rs`: six lines declaring/re-exporting this private module's
  API. Everything from original `normalize` through EOF, including old link,
  answer, answer_set and tests, is byte-identical to base.
- `crates/facts/src/main.rs`: only new standalone Verify variant/arm, exposed as
  `glassmind-facts verify --index INDEX 'Was SUBJECT born in OBJECT?'`.
  Removing those additions yields byte-identical old source: Train/Score/Ask unchanged.
- `crates/facts/tests/verification.rs`: three tests exercising frozen authored
  fixtures, read-only serialization equality, resolution tags, >10 values and the
  supplemental delimiter contrasts. New tools contain fixture generator, contracts,
  replay/archive/demo scripts and small reports. Fixture graph IDs are test-only.

Question bound4096 bytes is checked before case conversion/parse allocation;
endpoint bound256 bytes before normalization; oversized aliases skipped before
normalization; no recursion. Store traversal bounded by loaded aliases/entities
and the requested edge vector. CLI argument and index loading use normal existing
Clap/serde allocation; these bounds govern question processing, not index file size.

## Frozen gates and results

PLAN, graph,67 cases, generator and CLI contracts were hashed before runtime edits;
`frozen.sha256` still verifies. Expected triples were independently authored and
asserted against fixture edges, never produced from candidate output.

| Gate | Result |
|---|---|
| Baseline verify capability | absent; published facts binary exits2 for new command |
| Primary native contracts | 67/67:16 Supported,20 Unknown,31 Refused |
| CLI fixed first12, then full | 12/12 and66/66; NUL case native-only because argv cannot contain NUL |
| Supplemental shared-space delimiter contrasts | 5/5:1 positive,4 refusals |
| Existing native facts/factset tests | 6/6; unchanged |
| New native tests | 3/3;72 total authored contrasts |
| Scoped fmt, all-target clippy -D warnings, release | all pass |
| First200 legacy predictions | byte-identical;12right188refused0wrong |
| Full2821 legacy predictions | byte-identical;256right2545refused20wrong |
| Full2821 joint entity/relation/answer | unchanged252right2545refused24wrong |
| First200/full native scorer output | byte-identical to provided baseline facts binary |

Retained full2821 baseline prediction stream reused (no baseline predictor replay).
Native baseline scorer alone rerun for first200/full because its exact raw output
was not in the supplied baseline bundle. Existing cache dev-final metrics were
stale and were not treated as the accepted baseline. Candidate predictor and
scorer consume the unchanged index/model. Main CLI/talk integration is outside
this batch; parent must run its independent contracts and talk integration gates.
This is explicit verification coverage, not a benchmark accuracy gain.

Baseline main SHA256 `981f8b100cd82ec9edfa23f2c2d379f2dfb17976e6b32ebab2c68d335c6efdb7`.
Final facts binary13128752 bytes, SHA256
`e99c3eb220e0c61ec580e0b820a753f4015763a3e389e074b0abc75abb575a88`.
Full prediction270305 bytes, SHA256
`9d7fb56e29d8dd52275c93d8edd14c921c5748d8b4c3bc48c7fc7008574005b1`.
Detailed source/binary/asset hashes in reports/artifacts.json.

## Real evidence and data provenance

Deterministic lexical selection over existing index edges in the four properties,
without reading DEV labels for selection: `Was Q100303 born in Q2533654?` is Supported
with witness Q100303/P19/Q2533654. Q100303's existing aliases include Karl G. Maeser.
Independent cache inspection found published statement
`q100303$EDDE6E93-DC6C-4FC5-BB64-C5E312B85289`, normal rank, no qualifiers.
Raw response `faea9d8294db2ab9.json`,1880743 bytes, SHA256
`0ac074d1a50473a0e453a8b9cd4ba1a6a9fa5926ae8b683d169d603483d4f332`.
See reports/real-example.json and real-reply.json. No live claim about current truth.

New downloaded data: **0 bytes**. New authored graph/cases/supplemental:20138 bytes,
repository MIT OR Apache-2.0 license; generator and frozen hashes retained.
Existing Wikidata source retrieved2026-10-10, CC0-1.0, index287236 bytes,
SHA256 `923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`.
Scope remains500 validation-subject-seeded entities. Cached entity `lastrevid` is
unavailable; exact content hashes pin the snapshot, not an invented revision.
Existing SimpleQuestions revision`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`,
CC-BY-3.0; DEV2821 rows/173187 bytes, SHA256
`1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48`.
Frozen relation model35773124 bytes, SHA256
`db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
Model/index/source/provenance assets unchanged. New verification never loads the model.

## Reproduction and retained originals

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/factverify-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/factverify-run/target
export BASELINE_DIR=/home/gsc/.cache/glassmind/continuous-20261010/factverify-run/baseline
export RUN="$TMPDIR/verification-replay"
bash tools/facts/verification/reproduce.sh
```

Use a fresh RUN because archive creation intentionally refuses overwriting originals.
Every cargo command sets TMPDIR and runs under
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`,
with `-j1` except fmt, and `--test-threads 4` for tests. Existing CARGO_TARGET_DIR used.
Exact executed scripts/logs: `$TMPDIR/verification/checks.sh`,
`$TMPDIR/verification/checks-v2.log`, and `$TMPDIR/verification/final-v2.{sh,log}`. Consolidated reproduction was syntax-checked;
its component commands ran, not a redundant consolidated replay.

Raw outputs/binaries/patches are external in `$TMPDIR/verification`:
`frozen-candidate-v2/` is final source/binary/probes frozen BEFORE final native
contracts; its manifest was checked against the final live source and artifacts.
`legacy-v2/`, `contracts-v2/`, `real-example-v2/` preserve raw outputs. A final
`submitted/` archive includes the complete handoff/reports. No rollback occurred.

Two rejected implementations are preserved honestly:
1. Initial parser omitted a delimiter at the auxiliary boundary; original case
   “Was Born in Hope born in Port of Spain?” incorrectly supported. Original
   binary/failure log preserved in initial-rejected; its source was reconstructed
   exactly by removing the subsequently inserted guard. Corrected without changing
   original frozen cases or grammar.
2. First guarded candidate passed67 but review found that non-overlapping matching
   missed adjacent predicate phrases sharing a space. Five supplemental contrasts
   were frozen before correction; four incorrectly supported on that binary.
   `frozen-candidate/` preserves exact pre-correction patch,source,binaries,probes;
   overlap-before.json preserves all outputs. Final search detects overlapping
   delimiters too. No case/threshold/acceptance weakening.

Remaining limitations: small conservative grammar, no Unicode normalization beyond
existing lowercase/tokenization, some valid names refused by reserved words or
predicate-delimiter ambiguity, no date/qualifier/rank reasoning. Finite snapshot
support is not globally current/exhaustive truth. Index lacks statement UUIDs and
qualifiers: witness IDs allow a real source link, but caller must attach a verified
bundle manifest/provenance. Labels are display-only and never new aliases. Parent
owns facts-enabled talk routing, presentation/provenance and independent acceptance.
