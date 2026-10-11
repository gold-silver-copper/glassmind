# Shared-subject fact matching — bounded batch handoff

Implemented, uncommitted; no main conversation integration is claimed.
Base source: `71a06184a29e2df4d381f7e993676864125fe8d3`.
Complete source-file hashes and bytes: `run/reports/artifacts.json`.
Source-tree digest (canonical JSON of that file's source_files map):
`bef8cacc9db5ff4bbd4a4f028ca5bc7fa7fc652911cde52a0aba70114c639c3f`.

## Changes

- New `crates/facts/src/matching.rs`: reusable `MatchPredicate`,
  `MatchConstraint`, `ResolvedMatchConstraint`, `MatchResult`, `SubjectMatch`,
  `MatchWitness`, typed refusal/unknown reasons, `parse_match_question`,
  `Store::match_question` and `Store::match_constraints`.
- Positive P19/P20/P27/P108 conjunctions, raw 2..3 constraints. All endpoint
  validation/resolution precedes claims; full aliases/collision/case behavior
  reuses verification. Canonical explicit QIDs may be edge-only objects.
  Full directed claim vectors intersect before display truncation. Canonical
  unique constraints, lexical subjects, full total, returned count, truncation,
  <=10 subjects, exact witnesses and Wikidata URLs for every condition.
- Standalone `glassmind-facts match --index INDEX QUESTION`, no model required;
  JSON includes scope, no-temporal-guarantee text and SHA256 of loaded index bytes.
  Added already-locked sha2 0.10.9 to the owned facts manifest; Cargo.lock only
  adds the dependency edge, no package version or download changes.
- `verification.rs` changes only four helper visibilities to pub(super).
  Existing verification/answer/answer_set bodies and Store.validate unchanged.
- New matching library tests and public authored development contracts/tools.
  No other crate, main CLI, scorer, shared manifest, old fixture, README, data,
  historical prompt or progress-document edits. No agents, model calls,
  credentials, API keys, downloads, commits, services or surviving jobs.

## Frozen grammar and scope

See original `PLAN.md` and `frozen.sha256`. `Who CLAUSE and CLAUSE[ and CLAUSE]?`.
Clauses: `was born in`, `died in`, `is a citizen of`, `was a citizen of`,
`works for`, `worked for`. ASCII case-insensitive keywords, exact internal spaces,
full endpoints, optional exterior ASCII spaces. <=4096 question bytes,
<=256 endpoint bytes, raw 2..3 conditions, exactly one possible accepted parse.
AND/OR/negation/conditional/temporal/comparison/instruction/pronoun reservations
are documented in PLAN. No open-ended English, context, current-validity claims,
negation, union, reversed edges, model fallback, answer table or mutation.

Predicate words inside aliases are retained in full. AND-containing aliases are
unsupported: a post-review guard checks at most three full normalized alias spans
and refuses any spanning a reserved separator, before claims. The grammar is
unchanged. The extra contract and rationale were frozen before corrective editing
in `SEPARATOR-AUDIT.md`, `separator-contract.json`, `separator-frozen.sha256`.
Library callers attach verified bundle provenance; CLI hash identifies the bytes,
not their upstream authenticity. Zero is no matches in this loaded Store only.

## Validation and measured outcome

- Original frozen contracts: 45/45 library; 44/44 CLI (NUL cannot travel in argv).
  Full exact subject sets/counts/constraints/witness URLs checked, not status alone.
- Additional native invariants: 5/5 — sole match beyond first ten candidates,
  reversed subjects and duplicated/reversed claims, empty store, reserved alias
  refusal with and without claims. Separator regression also passes in library.
- Facts Rust: 15 tests passed (6 existing unit, 4 existing verification, 5 new
  matching); no failed tests. Scoped offline release build, clippy all-targets
  with warnings denied, fmt check and git diff --check pass.
- Existing current public verification contracts pass. 152 native verification/
  first20 dev ask outputs byte-identical to baseline.
- Legacy answer_set first200: exact baseline/candidate bytes from the same probe
  compiled separately against saved original and final source; 15 nonempty,
  including 3 multivalued results. 30,710 bytes, SHA256
  `dfeed55c0b1dabddecbc24935c5a7e4275d9044e50c21321bfb92f7a9ff87356`.
- Unchanged legacy adapter `tools/facts/verification/legacy.py`: fixed first200,
  then full2821, final predictions AND native score bytes equal baseline.
  Sample: answer/joint 12 right /188 refused /0 wrong; 19,120 prediction bytes,
  SHA256 `8015991afccc77932038ca25ae8b9451ab22acd4aa4c53b5c59356dc1e3b34e9`.
  Full: answer **256/2545/20**, joint **252/2545/24** (right/refused/wrong);
  270,305 prediction bytes, SHA256
  `9d7fb56e29d8dd52275c93d8edd14c921c5748d8b4c3bc48c7fc7008574005b1`.
  Saved baseline/candidate sample/full predictions and scores under run/baseline
  and run/legacy-final. Existing wrong counts are retained, not reinterpreted.
- Real index query discovered by a generic pair search:
  `Who works for Q193727 and was born in Q18419?`
  P108/Q193727 subjects = {Q1242948,Q168041}; P19/Q18419 subjects =
  {Q1242948,Q1338869}; conjunction = {Q1242948}, strictly smaller than both.
  Exact witnesses `(Q1242948,P108,Q193727)` and `(Q1242948,P19,Q18419)`.
  Full result/provenance in `run/reports/real-example-final.json`. This is finite
  index capability evidence, not independent benchmark accuracy or global truth.

## Retained failures / rejected implementation

The initial separator implementation silently accepted a known alias spanning
AND; rejected that behavior and added the bounded alias-only refusal guard.
Original failing output remains `run/reports/separator-original-failure.json`.
Original 45 labels never changed; final and first contract outputs retained.
Initial clippy failure was an unwrap in a non-test-marked fixture helper; changed
helper to return Result, keeping test assertions. Original log retained.
Unchanged historical verification `contracts.py` stops at old case35. Baseline and
candidate both disagree with old cases35/36: current alias semantics support
"Born in Hope" and return unknown for the other complete alias. No scorer/fixture
was edited. Current cases-alias-v2 contracts pass. Exact comparisons and old
mismatches retained in run/legacy-queries. No other measured experiment rejected.
The missing baseline match command is a missing capability, not an accuracy score.

## Inputs, versions, licensing and identities

No new external data. Existing read-only source directory:
`/home/gsc/.cache/glassmind/continuous-20261010/factmatch-source`.
SimpleQuestions source askplatypus/wikidata-simplequestions revision
`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`; questions CC BY 3.0, Wikidata CC0.
Existing index is finite **500 validation-subject-seeded** entities, cached-only;
no index expansion or fresh raw-cache audit performed by worker.
- index.json: 287,236 bytes,
  `923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`.
- relation.json: 35,773,124 bytes,
  `db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
- valid.tsv: 173,187 bytes, 2821 rows,
  `1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48`.
- train.tsv: 1,193,868 bytes; not used for training in this batch; metadata/hash
  verified, not used as a new corpus. All source metadata/license hashes in artifacts.
New authored development data, version frozen 2026-10-11, MIT OR Apache-2.0:
`graph.json` 6,009 bytes; `contracts.json` 11,439 bytes; separator contrast 7,094
bytes. Exact SHA256 in frozen manifests and artifacts.json. These are synthetic
contracts, not heldout data. Generated graph variants are only development checks.
Rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0 (5f94df478 2026-08-27).
sha2 0.10.9 already locked; no dependency download.
Baseline native SHA256:
`a15a7b555724aad0d5a49076a1e443a4d56b1214220fb65d0029f164989710fc`.
Final native: 13,755,840 bytes, SHA256
`90861d7dfe781d45c7eccfe10253edefbe197e47fb4e2bbc57a14472205e7203`.

## Reproduction and frozen delivery

From worktree root (reuse these existing directories):

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/factmatch-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/factmatch-run/target
tools/facts/matching/reproduce.sh
```

Every build/test/native process is under `capped.sh`: systemd user scope,
MemoryMax8G/MemorySwapMax0, setsid --wait, timeout --kill-after=10s 600.
Cargo builds/tests/clippy use -j1; tests --test-threads 4; fmt no job flag.
The metadata-only scoped `cargo --version` has no job flag. Outer worker scope
MemoryMax8589934592/MemorySwapMax0 was verified. No /tmp storage.
For original baseline reproduction, saved source and binary/predictor are in
run/baseline; `capped.sh python tools/facts/matching/baseline.py` captures first200
then full2821 without changing the original scorer. `final_gates.sh` performs
final sample/full comparison; `legacy_sets.py` compiles the exact same development
probe against both source snapshots. All commands use only permitted source files.

Frozen candidate path:
`/home/gsc/.cache/glassmind/continuous-20261010/factmatch-run/frozen-candidate`.
Contains source, binaries, code patch, contracts/tools, reports, predictions,
baseline evidence and a per-file SHA256/bytes manifest verified after copying.
Owned tools run/frozen-candidate points there. No commit created.

Limits: narrow explicit grammar; <=3 conditions and <=10 displayed subjects;
no complete-world claims, temporal qualifiers, current employment/citizenship,
independent benchmark gain, official heldout evaluation, main Match facts route,
main conversation/memory/story integration or raw-cache witness confirmation.
Those integration checks belong to the coordinator; no approval or result claimed.
