# Parent review

The parent verified 51 artifact hashes and independently recomputed all TRAIN
and validation transitions for both index scopes. See [parent-review.json](parent-review.json).
The original TRAIN preservation and full-dev gates fail; runtime remains restored.
The subsequent [joint audit](../JOINT-AUDIT.md) explains coincidental answer matches
without changing the decision. Original submission:15 files,72,850 bytes,
archived and hash-verified before worktree removal.

# Link quality: one rejected boundary experiment

**REJECTED; runtime fully restored.** Source base
`176dc89ee9ce0ad2302474f03683334e37514b76`. Only new files under
`tools/facts/linkquality/` remain uncommitted. No CLI, scorer, model, index,
manifest, README, historical prompt, or other-track changes. No commit.

## Hypothesis, decision and exact changes

`PLAN.md` was written before implementation and before reading dev failures.
TRAIN showed false links where aliases occupied a question's requested-type
slot. The single nonlearned experiment classified every longest alias occurrence
immediately after `what`/`which` as an interrogative determiner boundary. It
withdrew an answer only if every longest occurrence had that role. This was
not an alias blacklist, fit model, or ranker. No entity names/IDs in features.

The candidate added `crates/facts/src/quality.rs`, a versioned closed policy
with unknown-policy deserialization rejection; an optional
`Store::answer_with_quality` API in `lib.rs`; and a question-only Rust
`linkquality_predictions` example emitting original/candidate Answer pairs.
The API applies the policy only AFTER unchanged linking, ambiguity, casing,
relation and claim checks. Retained entity/relation/answers/refusal are exact;
withdrawals clear answers and add a refusal while preserving entity/relation.
Absent policy calls the original answer path unchanged. No runtime Python.

TRAIN preregistration additionally required no correct entity-link loss.
The rule failed this gate: three correct TRAIN links were withdrawn, including
a relative-clause use of `which`, and two inverse questions whose generic type
is itself the benchmark subject. This is evidence against the semantic rule.
We did not change it. Subsequent dev evaluation was diagnostic only, to finish
the requested bounded counterfactual audit; it could not rescue acceptance.
The full dev preservation gate also fails by a wide margin.

`rejected.patch` contains all three candidate runtime files/diffs. It is
**unapplied** and passes `git apply --check`. `candidate-source.json` hashes the
exact tested sources. `audit.py` independently joins predictions to allowed
annotations with the unchanged scorer's exact status order; `overlap.py`
reports coverage/repetition; `reproduce.sh` replays the archived experiment.
Raw prediction, every-row audit, questions and binary artifacts remain outside
Git under `/home/gsc/.cache/glassmind/continuous-20261010/linkquality-run/artifacts`.
No quality model is shipped. Do not integrate this policy into main/talk.

## Measurements (right / refused / wrong)

| Scope and split | Frozen baseline | Rejected candidate |
|---|---:|---:|
| old500 fixed first200 dev | 12 /188 /0 | 12 /188 /0 |
| old500 full2821 dev | 256 /2545 /20 | 256 /2545 /20 |
| new999 fixed first200 dev | 3 /191 /6 | 1 /197 /2 |
| new999 full2821 dev | 47 /2712 /62 | 8 /2792 /21 |
| old500 full19481 TRAIN | 12 /19405 /64 | 12 /19405 /64 |
| new999 full19481 TRAIN | 806 /18025 /650 | 538 /18653 /290 |

The frozen main facts score binary reproduced the exact supplied full2821
baselines, including **62**, not63, new999 wrong answers. The offline baseline
status totals agree with it. All sample rows remain in200 and all full rows
remain in2821; unsupported rows are never dropped.

Old500 has zero changes. New999 dev has80 withdrawals: **39 scored-right to
refused**,41 wrong to refused,8 right kept,21 wrong kept,2712 old refusals kept.
Thus only8/47 baseline scored-right answers survive, far below required45.
Wrong21 passes the <=52 gate, but this is not an accepted improvement.
TRAIN has628 withdrawals:268 scored-right,360 wrong;625 wrong entity links
and3 correct entity links are withdrawn. One of those3 was scored right.

Every TRAIN/dev row has an external exact baseline/candidate claim audit.
There are **zero changed retained claims, zero revived refusals and zero new
wrong answers** in either scope on either split. Rust decisions match the
pre-implementation TRAIN diagnostic's exact withdrawal rows. All full TRAIN
and dev candidate baseline pairs equal outputs from the frozen prediction
binary. The optional-absent policy additionally matches baseline on both
fixed first200 samples. Complete lost/removed row lists are in the summary
JSONs; per-row paths/hashes are embedded in those summaries and `artifacts.json`.

A critical metric limitation: **45 of47 new999 baseline scored-right dev
answers link the wrong entity**. All39 scored-right losses are among those
45; the remaining8 scored-right include6 wrong-entity links. Answer equality
can therefore be coincidental. We still enforce the specified frozen
answer-only preservation criterion and reject. One entity-correct but
scored-wrong dev link (row1143) is also withdrawn. Old500's four existing
scored-right/wrong-entity cases remain unchanged. No scorer was redefined.

## TRAIN calibration and coverage selection

No learned model was fitted. Before choosing the typed rule we checked the
specified zero-based every-tenth TRAIN calibration partition (1949 rows),
never using it for fitting. New999 currently answers86 CAL rows, of which
only14 have the correct entity ID; its other72 answered links are incorrect.
A keep/refuse selector cannot reach >=50 accepts at >=95% entity precision:
that would need at least48 correct links when only14 exist. No threshold
search, target relaxation, reject-all deployment, or second experiment occurred.
The non-calibration17532 rows have1370 answers,701 entity-correct/669 incorrect.
These are labels joined offline only, never runtime inputs.

The old500 snapshot is validation-subject-seeded. The supplied new999 snapshot
selected1000 TRAIN subjects by frequency/SHA256 ties from first90% TRAIN,
excluding every-tenth CAL rows; one redirect was excluded. That selection pool
has15778 rows/14109 unique subjects. The frozen actual999 index covers2667
selection rows,180/1949 CAL rows and232/2821 dev rows by gold subject ID.
It produces only11 entity-correct answered dev claims among109 answers.
Neither scope is open world or entity-disjoint evaluation.

Fit/CAL share189 distinct subject IDs and31 normalized complete questions.
Selection/CAL share181 subjects/26 questions; selection/dev share216 subjects/
51 questions. New999 selected-link alias phrases share145 between selection
and CAL and180 between selection and dev (longest normalized aliases for all
linked baseline predictions, including later refusals). Full counts and
old500 results are in `overlap.json`. These overlaps and coverage selection
make TRAIN results optimistic; no independent generalization claim is made.

## Validation

- Ten focused facts tests passed (five existing acronym/tie tests; five new
  boundary/monotonicity, one-word/punctuation, partial-name/title/literal-limit,
  preserved ambiguity/refusal and invalid-policy tests).
- Facts all-target clippy with `-D warnings`, package fmt check, release example
  build, `git diff --check`, and restored-state `git apply --check` passed.
- Partial-name/title/literal tests deliberately expose the narrow rule's limits:
  unrelated incomplete mentions still pass through unchanged. They do not
  certify mention completeness or semantic correctness.
- Frozen inference is question-only; annotation joins happen in separate
  offline audit processes. No training labels reach Rust inference.
- All cargo calls set on-disk TMPDIR, used existing CARGO_TARGET_DIR, and ran
  under `systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait
  timeout --kill-after=10s 600`, with `-j1` except fmt and four test threads.
  Outer scope verified MemoryMax8589934592 /MemorySwapMax0. No giant suite.
- A cross-filesystem rename while moving a diagnostic report failed; it was
  corrected with a copy/remove move. No inference or evaluation was affected.

## Source, license, bytes and hashes

New external data: **0 bytes**, no download or crawl. Existing immutable data:

| Artifact | Bytes | SHA256 |
|---|---:|---|
| TRAIN19481 | 1193868 | `25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b` |
| valid2821 | 173187 | `1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48` |
| old500 index | 287236 | `923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185` |
| frozen new999 index | 650045 | `142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791` |
| frozen relation model | 35773124 | `db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6` |
| frozen score binary | 12803712 | `076f5f89d620d4e6c4521d45799f85c585a9c6fb54004e74540c5c5ad37b4a3d` |
| frozen prediction binary | 7584984 | `1e5a4af21b05024c85c6e3173f782cd6d62c3a5ed27259fe642aeb30c56eb433` |
| rejected candidate prediction binary | 7747400 | `4eb512cc293e2738861337adecda7d33bddb8a3c441cb9bc9858f1a622997566` |

SimpleQuestions-Wikidata source `askplatypus/wikidata-simplequestions` revision
`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, CC BY3.0 per existing provenance.
Wikidata claims/aliases CC0, frozen supplied snapshots; no additional crawl
revision claimed. Relation model unchanged, existing TRAIN provenance.
No new learned parameters or licensed derived model to distribute. New code
follows repository MIT OR Apache-2.0. All source/binary/report/raw hashes and
byte counts are in `artifacts.json`; candidate source hashes are separate.

## Reproduction and limitations

In a disposable checkout of the source base, retain the run directory and
mounted TRAIN/valid/model, then run from repository root:

```sh
git apply tools/facts/linkquality/rejected.patch
RUN=/home/gsc/.cache/glassmind/continuous-20261010/linkquality-run \
OUT=/home/gsc/.cache/glassmind/continuous-20261010/linkquality-run/replay \
CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/linkquality-run/target \
bash tools/facts/linkquality/reproduce.sh
```

The script uses the same frozen score/prediction binaries, all required cargo
caps, fixed sample before full dev, and full TRAIN. Its final overlap command
uses the retained original raw artifact directory. The consolidated script
was syntax-checked; its constituent build/test/inference/audit commands were
run individually in this batch, not redundantly replayed as one script.
Generated binary byte hashes may depend on compiler/check-out paths; compare
prediction bytes/metrics and source hashes for semantic replay.

The failure is grammatical-role ambiguity, not an implementation tie-break
bug. `which` may introduce a relative clause; generic type mentions can be
subjects for inverse relations. A local token adjacency rule does not parse
those structures. Partial full names/titles/literals remain unverified;
missing question-type compatibility and incomplete inverse-index uniqueness
are separate unresolved limitations. No accepted mechanism is left disconnected:
there is no accepted mechanism. Runtime/index/model hashes remain unchanged.

No agents, LLM calls, credentials/API keys, services, held-out items, official
test contents, hidden-test code/generators, or old private logs were used.
All owned command sessions completed; no job is left running. This one bounded
experiment is complete as rejected evidence, not a measured product gain.
