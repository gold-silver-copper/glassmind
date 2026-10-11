# Ratewords — one bounded batch, REJECTED

The frozen candidate fails the preregistered gate. **Do not integrate the runtime
patch.** All runtime changes were reversed; only this owned evidence directory
remains uncommitted. No commits, model calls, agents, services, dependencies or
data downloads. No processes remain owned after handoff. Main CLI source, shared
manifests, scorers, existing dev generators, historical reports and prompts were
not modified. No credentials, API keys, official test/held-out items, hidden code,
or old private logs were read.

## Hypothesis and exact candidate changes

[PLAN.md](PLAN.md) was written before implementation and any dev run, following
bounded inspection of **Problem only from first2000 TRAIN rows**. Inspection used
rate-word and short-speed filters, with selected evidence in
[train-evidence.json](train-evidence.json). No options, rationales or annotated
formulas entered design or runtime. The unchanged evaluator alone reads dev gold.

The sole hypothesis was dimension-checked affine traversal constraints:
known speed × unknown duration = moving length + stationary extent. Linked events
can determine an unknown moving length, then an unknown duration. Distinct typed
length/duration/speed expressions convert exact quantities to canonical SI before
using existing checked scalar elimination. Two unknowns and eight equalities;
no product of two unknowns, no floating point, no nonlinear solver.

The rejected patch contains precisely:

- `src/rate.rs`: internal reusable typed affine expressions, dimensional addition,
  known-speed/duration multiplication, equality constraints, exact solving and
  physical magnitude checks. Symmetric unknown-speed operation has a unit test.
- `src/rate_language.rs`: fully consumed bounded clauses for entity lengths,
  explicit speeds and crossings, named/reference binding, explicit output duration
  units and explicit same-speed linkage for multiple events. Contradictions,
  unbound/ambiguous referents, contextual clauses and incompatible dimensions refuse.
- `src/linear.rs`: crate-visible scalar operations/tokenizer and a typed-caller
  elimination entry point. Existing unordered absolute-difference behavior unchanged.
- `src/lib.rs`, `src/parser.rs`: internal module registration and library dispatch.
- `tests/rate_language.rs`: conversion/composition and refusal checks.

No problem IDs or answer tables occur in runtime. `audit.py` contains six
**post-freeze audit transcriptions**, solely for independent evidence, never used
by Rust/scoring/design. English assumes the ordinary complete-crossing event
(front enters through rear clears); both lengths and event speed are explicit.

## Results and every transition

| Fixed evaluation | Correct | Refused | Wrong | Total |
|---|---:|---:|---:|---:|
| Baseline first100 | 1 | 99 | 0 | 100 |
| Candidate first100 | 1 | 99 | 0 | 100 |
| Corrected baseline full | 22 | 4450 | 3 | 4475 |
| Frozen candidate full | 23 | 4444 | 8 | 4475 |

All **1229 unparseable-gold rows** remain in the full denominator. All **25 old
answer records are exactly identical**, including method/unit/status/value and
all three old scored errors. No old answer losses. Unchanged:22 correct,3 wrong,
4444 refused. Changes:1 refused→correct;5 refused→wrong.

Every newly answered problem was independently recomputed from its English with
Fraction, explicit unit conversion and full-crossing geometry, without using the
gold key as an oracle. Full questions and derivations are in [audit.json](audit.json).

| Dev row | English extents (m) | Explicit rate | Independent seconds | Scored transition |
|---|---|---|---|---|
|295|100 + 140|36 km/h = 10 m/s|24|refused→correct|
|563|150 + 150|36 km/h = 10 m/s|30|refused→wrong|
|1128|60 + 120|30 m/sec|6|refused→wrong|
|1986|100 + 170|36 km/h = 10 m/s|27|refused→wrong|
|2836|140 + 150|36 km/h = 10 m/s|29|refused→wrong|
|2873|120 + 150|36 km/h = 10 m/s|27|refused→wrong|

The five new scored errors have `expected: "None"` from the unchanged numeric gold
normalizer. They **still count wrong**. Options were not inspected to explain or
repair those normalizations. No metric changes, acceptance relaxation, full-dev
retuning or second hypothesis occurred. No new semantic error was detected in
these six audits, but the candidate fails both >=3 additional correct and zero
new scored-wrong criteria. This is rejected evidence, not a published gain.

Validation on the frozen candidate:

- 13 quantity tests pass; scoped all-target clippy, fmt and both quantity/main CLI
  builds pass. Initial clippy found two style issues; corrected before any dev.
- 43 independently authored Fraction probes pass:14 answer cases and29 refusals;
  names, coefficients, clause orders, miles/feet/yards, km/minutes, multi-event
  composition, contradictions, ambiguity, negative/zero/overflow/dimension limits.
- 50 existing linear composition and55 absolute-difference checks pass.
- All3116 synthetic status/value/unit triples are identical. 3056 correct,
  including762 expected refusals;60 unsupported;0 wrong. Canonical SHA256
  `93b2bd1c9f8822af95a68d02663a90d122d24ca57d730c61c670e2b11079bbe9`.
- Main CLI compiled. Parent-owned talk/memory41/calculation27/story47 replay was
  not performed; rejected candidate must not be published on these results.

[validation.json](validation.json), [regressions.json](regressions.json),
[probes.json](probes.json), [absolute.json](absolute.json) retain exact evidence.

## Exact provenance

Base source HEAD: `e03437a81f2d047babad5e122aec99a88b1c56af`.
Corrected baseline source: `687f2751ff256f2071330590345ef0652fb09baa`.
Baseline quantity binary SHA256:
`06a651b1817f022677f1c898ba8f71bb53eb2d6ed7ad5c42c707b356a869768a`.

Rejected quantity binary:25,534,224 bytes, SHA256
`a5d8aa25a10d11b0f90f36639ab36568d40934a75e63ed0475199f9b17d36f41`.
Rejected main CLI binary:126,816,496 bytes, SHA256
`9c7420e53e9dea2d8a68192473ee88f899de5902a3137c79e79eb3628b604915`.
Both retained under `/home/gsc/.cache/glassmind/continuous-20261010/ratewords-run/candidate/`.

[rejected.patch](rejected.patch):28,975 bytes, SHA256
`b7e820663422ab3f0d7aad97d271fef464cdcdcc86a4de13eb6b201f43bb2a78`.
[FROZEN.json](FROZEN.json) hashes all candidate quantity source/test files before
full dev; [source.json](source.json) lists changed-file bytes/hashes, binaries and
data provenance. Code and authored diagnostics:workspace **MIT OR Apache-2.0**.
`SHA256SUMS` covers every submitted evidence artifact except itself.

Existing AllenAI MathQA, Apache-2.0 per repository provenance, dataset revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`; card revision
`c4f1cc784c04c4957b50c97858f23893b633eea6`:

- TRAIN:9,013,733 bytes, SHA256
  `c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
- Validation:1,350,141 bytes, SHA256
  `488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.

Both hashes verified locally; exact source URLs are in source.json. **New external
data bytes:0.** Extracted TRAIN evidence and authored diagnostic JSON are new local
reports, not downloaded datasets; their exact bytes/hashes are in artifact manifest.

## Reproduction and limits

For retained binary/evaluation replay, from this checkout:

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/ratewords-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/ratewords-run/target
bash tools/quantities/ratewords/reproduce.sh
```

That script replays first100 then full4475, six independent derivations,43 probes,
all3116 synthetic comparisons,50 linear and55 absolute checks. It writes to own
RUN/replay and never applies the rejected patch. The consolidated script was
syntax-checked; its component commands were executed individually as recorded.

To reproduce source builds, use a **separate disposable clean checkout** of the
base HEAD, copy this evidence directory there, then execute:

```sh
git apply --check tools/quantities/ratewords/rejected.patch
git apply tools/quantities/ratewords/rejected.patch
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/ratewords-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/ratewords-run/target
capped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
capped cargo fmt -p glassmind-quantities -- --check
capped cargo clippy -p glassmind-quantities --all-targets -j1 -- -D warnings
capped cargo test -p glassmind-quantities -j1 -- --test-threads 4
capped cargo build -p glassmind-quantities -p glassmind -j1
CANDIDATE_BINARY="$CARGO_TARGET_DIR/debug/glassmind-quantities" bash tools/quantities/ratewords/reproduce.sh
```

Debug hashes may depend on source path/toolchain; retained binary hashes above
identify the actual evaluated executables. Every cargo invocation in this batch
used the mandated disk TMPDIR, existing target, systemd8GiB/zero swap,
setsid--wait,600-second timeout,-j1 (except fmt), and four test threads.

Representational limit: the English adapter only asks duration for one moving
extended object crossing a stationary extent, optionally linked to prior crossings
at explicitly the same speed. It does not solve general travel, moving/moving
relative motion, work/flow, acceleration, inferred speed constancy, output units
omitted by many TRAIN items, or ambiguous/nonlinear unknown products. The typed
unknown-rate operation is tested internally but has no English target production.
Observed benchmark coverage is one narrow surface family despite the compositional
core and broader local probes. No claim of broad quantity-language competence.

The requested measured improvement is incomplete because acceptance failed.
Runtime source restored byte-for-byte to base; rejected binaries are archived
separately and ordinary target binaries rebuilt from restored source. Parent owns
any further hypothesis, independent review and integration. No follow-on experiment
was started.
