# REJECTED — one bounded rate-clause batch, runtime rolled back

Do not integrate as an accepted runtime change. Full DEV failed both the >=34
correct gate and the zero-new-metric-wrong gate. No post-DEV tuning or second
candidate occurred. Parent unseen-wording gate was NOT run. No readiness claim.
Only new `tools/quantities/rate-clauses/` evidence remains uncommitted. The original
candidate source, patch, binary and results were archived and SHA256-verified
BEFORE rollback; `git diff -- crates/quantities` is empty. No commit was made.

Source: `bb89964dd887b2b68ae01fb8f0a9a92a4901f490`, quantities runtime baseline
unchanged from408d16b as supplied. Own run:
`/home/gsc/.cache/glassmind/continuous-20261010/rateclauses-run`.
Original candidate binary: `candidate/quantities`, archived again at
`rejected/quantities`, 27,501,480 bytes, SHA256
`ff0f94d8dc72da238131a41fb3a76c6c5d764e0e13d02802aed466f78f5840b9`.
It is an unoptimized standalone quantities binary; main CLI was not rebuilt.

## Hypothesis, freeze, and exact implementation

PLAN.md and 40 initial authored cases (24 positive,16 negative) were frozen before
runtime edits; PLAN-TESTS.sha256 pins them. Independently computed positive values
use reciprocal rates, with distinct names, reordered facts, two clause realizations,
three agents, inverse queries, tank direction and explicit time conversions.
Three subsequently frozen authored sets brought validation to65 cases:40positive,
25negative. Supplemental sets were authored before their corresponding grammar
changes. Seven draft state-proposal cases were rejected before implementation or
execution and are NOT counted; their assumptions were invalid. No expectation
was changed to turn a failed execution into a pass.

The archived candidate adds `crates/quantities/src/rate_clauses.rs` and
`crates/quantities/tests/rate_clauses.rs`, one lib module declaration and a seven-line
parser fallback before expression lexing. Existing accepted routes precede it.
No dependency or Cargo.lock change. No mainCLI/world/English/scorer/manifest edits.

Representation: typed `Agent`, source-anchored `WorkId`, job/tank kind, signed
completion-time `Fact`, duration `Query`, explicit simultaneous group and initial
state, normalized-source clause text/spans. All facts must bind the same introduced
workload; another indefinite workload refuses. Local it/the same job/tank/work
references resolve only after introduction. Agent lists preserve textual order
for paired times, then canonicalize identities for rate operations. Named groups
and explicit local they/both/all references compose with independent factual
clauses, conditional premises, verb/work ellipsis and query duration slots.

Execution uses checked Rational operations only: sum all individual rates for a
2–3-agent simultaneous query, or subtract known individual rates from one group
rate for exactly one missing individual duration. Complete consumption, bound
operands, deterministic conflict checks and positive directed completion are
required. Actual simultaneous tank queries require an explicit empty/full state.
The answer method states the constant independent simultaneous-rate assumption.
No staged operations, capacities, partial volumes, nonconstant rates, arbitrary
number collection, question/task identities, runtime Python/LLM or answer tables.
The post-DEV audit table is reporting-only and is not a runtime dependency.

FREEZE.json at `2026-10-11T10:01:00.220368+00:00` pins source/tests, authored probes,
grammar, adapter, unchanged metric files, baseline binaries/predictions and allowed
parquets. First100 Problem-only inference saved10:01:10.769650UTC; full4475 saved
10:01:19.538854UTC. Each inference completed and was saved before its labels were
loaded. Only options/correct were then decoded to score. No rationale, formulas,
categories, programs, official test or hidden data were accessed. Full inference
ran ONCE; sample inference ran ONCE. Source/binary/adapter remained frozen.

## Measured results

| Evaluation | Baseline correct/refused/wrong | Candidate correct/refused/wrong |
|---|---|---|
| v3 first100 |1/99/0|1/99/0|
| v3 full4475 |29/4443/3|31/4440/4|
| unchanged legacy full4475 |29/4443/3|30/4440/5|

Unscorable answered:0. All32 old answers, including the3 old wrong ones, are
preserved as identical objects AND byte-identical raw standalone JSON answer
lines, verified by replaying only those32 Problems with the supplied baseline
binary. Exactly3 new answers;2 match v3 labels,1 does not. Reports retain all4475
rows and all prediction bytes. No denominator, label, metric or gate was changed.

Full prediction SHA256:
`905aa4df540cf65c743415dae8a5e03428ad9e132e4d12fdda6f0d2878b87229`.
Sample prediction SHA256:
`76511eb5a59de3497ae56e6021f8d7b41b6fea68130c3028739141d7e6ff5796`.

Validation before DEV:
- 65/65 authored outcomes:40 substantive positives,25 expected refusals.
- 16 quantities Rust tests pass, including4 new integration test groups.
- Scoped cargo fmt, all-target quantities clippy with warnings denied, and build pass.
- Synthetic3116 identical outcomes:3056correct including762expected refusals,
 60unsupported refusals,0wrong; canonical digest unchanged
 `93b2bd1c9f8822af95a68d02663a90d122d24ca57d730c61c670e2b11079bbe9`.
- Linear50, absolute55, percentchange38, previously authored parent78 all pass.
 Percentchange/parent outcomes explicitly compared to the supplied baseline.
- Main integration, workspace checks, and NEW parent unseen wording were not run.
 The older parent78 is a regression set, not independent acceptance evidence.

## Independent arithmetic and unit audit of every new answer

`audit.py` uses integer work counts over chosen common durations, without importing
runtime code or metric code. This is an independent derivation by the same worker,
not a separate reviewer. Exact input Problems and calculations are in audit.json.

| DEV index | Derivation | Candidate | v3 |
|---|---|---|---|
|2469|In36days joint9jobs minus A3 minus B2 leaves C4jobs;36/4|9days|correct|
|3869|In30days joint5jobs minus A3 leaves B2jobs;30/2|15days|correct|
|4126|In20days joint20jobs minus A1 leaves B19jobs;20/19|20/19days|wrong|

All three numerical durations and day units agree with the supplied Problem text
under the stated assumptions. Index4126 selects `0.95 days`, i.e.19/20days. That
number equals B's jobs/day rate, not the derived days/job duration. This is evidence
of a label discrepancy, NOT a reason to redefine the gate: the new metric-wrong
answer and insufficient coverage still reject this candidate. Legacy additionally
rejects index2469's suffixed9days label under its unchanged numeric parser.

## Rejected work and actual limitations

Prior supplied WORKRATES experiment was already rejected:33<34 correct and0/17
independent solvable positives; its disclosed17 probes were not read or copied.
This batch used a clause representation, not whole-question productions.

Prefreeze draft failures are retained, not hidden:
1. Conditional suffix extraction originally treated a middle conditional factual
   clause as a trailing condition. Two authored test groups failed. Exact source
   and log: attempts/draft-condition. Replaced reordering with separately bound
   trailing-condition handling; original source spans remain in order.
2. `are open` matched the prefix of `are opened`; one authored group failed.
   Source/log: attempts/draft-condition-prefix. Fixed production ordering.
3. A duration-slot production could span earlier clauses; one authored group
   failed. Source/log: attempts/draft-query-slot. Restricted its subject grammar.
4. First clippy run rejected unchecked vector indexing and two nested-if forms;
   log retained as clippy.log. Checked access replaced indexing; final clippy passes.
5. Solo `fill an empty tank` was considered as initial-state evidence and rejected
   before coding: it describes an individual benchmark, not the queried joint
   initial condition. Draft fixture and rationale retained unchanged under
   attempts/rejected-state-proposal; it is not an accepted test set.

No separate candidate inference/binary was produced for these prefreeze drafts;
compiled authored-test failures are not DEV experiments. Final frozen failed
candidate original source/binary/reports are retained in full. No second DEV run.

Coverage is still narrow: only3 additional DEV answers despite40 authored positive
cases. One-word ASCII names with limited role prefixes; singular local work only;
explicit reference must follow introduction; query generally last (a trailing
simultaneous condition is supported); no arbitrary narrative ordering, unnamed
pipe introductions, group-system elimination, partial work or stages. Normalized
source provenance is internal to the transient model, not exposed in Answer JSON.
Many ordinary language forms remain refusals. These limitations and the failed
coverage gate mean this is evidence for a future representation review, not a
shippable improvement. Parent unseen>=8positive/all-negative gate remains unknown.

## Sources, bytes, license, and access boundary

New downloads:0bytes. New model calls/agents/services:0. No runtime LLM/Python.
Reused AllenAI MathQA revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0 (user-supplied provenance).
Only mounted train.parquet and validation.parquet were opened; source-card.md
was not opened. Training inspection decoded Problem only. Raw parquet hashing
read bytes for integrity without decoding annotation columns.

TRAIN29837rows:9,013,733bytes, SHA256
`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
DEV4475rows:1,350,141bytes, SHA256
`488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
Total reused parquet10,363,874bytes. Retained Problem-only TRAIN language excerpts:
train-language.json11,717bytes, train-composition.json10,835bytes, Apache-2.0.
These overlap and are language inspection, not an independent score set. The first
35 TRAIN wording examples were also viewed before PLAN, without other columns.
New authored acceptance JSON65cases:12,631bytes (8182+1473+1364+1612), repository
MIT OR Apache-2.0; exact per-file hashes in FREEZE.json. Rejected state drafts are
separately marked and excluded. Existing public authored dev generators permitted.
No heldout/hidden material, private old logs, credentials or API keys read.
Evaluator documentation was consulted for its unchanged scoring contract, not for
runtime rules. Parent-selected model/tier/login cannot be attested by this worker;
no new model process or credential-status inspection was needed or launched.

Outer scope verified memory.max8589934592 and memory.swap.max0. All16 cargo
invocations set the supplied non-/tmp TMPDIR, use systemd-run8GiB/0swap,
setsid --wait, timeout600/kill-after10, existing CARGO_TARGET_DIR, -j1 except fmt,
and --test-threads4 for tests. No jobs survive handoff; no pkill or services.
Detailed reproduction and actual command forms: COMMANDS.md.

## Archive and rollback integrity

`rejected/candidate.patch` includes both new Rust files plus minimal registration;
`rejected/crates/quantities/` preserves exact source. It passes `git apply --check`
on the restored baseline. Full binary/report backup is outside Git at ownrun
`rejected/`. REJECTED-BEFORE-ROLLBACK.json records85files/32,080,445bytes and their
SHA256s before rollback, and was verified before removing/restoring runtime files.
The later audit/handoff/command reports are recorded by ARTIFACTS.json. Runtime
rollback restored the original tracked bytes with git-show and removed only the
new owned module/test. All failed candidate evidence remains available.
