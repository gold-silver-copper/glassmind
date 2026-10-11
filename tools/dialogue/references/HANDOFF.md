# Grounded predicted-slot transfer — bounded candidate

Base source: `cb6fb28119d42e4700960ed5732ddb960347a9e1`. Changes are uncommitted,
restricted to `crates/dialogue/` and new `tools/dialogue/references/`.

**Result:** one fixed variant improves actual end-to-end state tracking over
frozen joint12, meeting the declared sample/full/multi-state gates. Full-joint
turns improve **549→595/1119**, retaining all 549 baseline-right turns. This is
not a new successful router: routing weights and every routing prediction are
unchanged. There are **four incorrect copy events, persisting for 11 scored
slot-turns**, detailed below; do not describe this as error-free resolution.
Parent should independently rerun full dev before promotion. No commit or
publishing action was performed.

## Hypothesis and acceptance, fixed before editing

Train-only inspection found that missing target location slots have reliable
relations to another service's prior slot. Hypothesis: learned, value-free
slot-to-slot relations can fill otherwise missing input slots from grounded
prior **predicted** states without changing the router or service trackers.
The broader system-span idea was narrowed to this smaller representation
before implementation or dev measurement; no system-span extractor was fitted.

Acceptance: strictly improve joint12's first-10 sample **21/91** and full
**549/1119** joint turns; increase multi-service exact state coverage; retain
baseline-right cases where possible and report/investigate all new errors;
preserve legacy **202/257 exact, 89.17509727626459% fuzzy**. Same fixed complete
selection and unchanged `tools/dialogue/evaluate.py`, `compare.py`, and upstream
metrics. No changes to service selection or denominators. No dev calibration.

Only **one** variant was fitted/scored. Fixed training rule: at least 20
opportunities and zero mismatches. For each train USER frame, an absent prior
gold target input slot is an opportunity, including when the new gold target
slot is absent. Sources are other services' states from prior USER turns, never
same-turn annotations. Accepted relations: Travel.location→Weather.city 83/83,
Weather.city→Travel.location 82/82. Five other observed pairs had zero matches
and were rejected. These names are learned model data, not branches in code.
The 455-byte resolver contains only slot/service names and evidence counts;
no entities, answers, IDs, dialogue text, or value-generation labels.

Earlier joint-router and independent-router experiments remain rejected under
their original criteria. Their source and reports are unchanged. This batch
uses joint12 as a frozen ablation baseline, not evidence that its previous
routing hypothesis succeeded.

## Exact changes and runtime boundary

- `crates/dialogue/src/references.rs`: schema-validated relation model and
  `CopyProposal` with source provenance. Requires noncategorical input slots,
  empirical evidence, a Text-valued prior prediction, and whole-span support
  in prior utterance text. Copies at most 12 whitespace tokens verbatim from
  that prediction; no synthesis or normalization of output values. Conflicting
  supported values refuse (`None`); unsupported and categorical sources refuse.
- `multi.rs`: optional resolver (absent by default), per-session utterance
  history, and a snapshot of predictions **before** the current USER update.
  Only predicted-active services can receive fills; existing target values and
  inactive held states are preserved. Current-turn values cannot transfer by
  schema traversal order. Reset creates empty states and history.
- `lib.rs` exports the resolver; crate-local `dialogue-multi.rs` initializes
  newly trained multi models with no resolver. No shared/main CLI edits.
- New `train.py` fits relations; `assemble.py` attaches them while asserting
  frozen weights are structurally identical; `audit.py` reports all changed
  slots and their provenance; `train_diagnostic.py` diagnoses predicted-source
  errors using train only. `reproduce.sh` contains capped reproduction commands.

Runtime still deserializes only `speaker` and `utterance`, rejecting unknown
fields. It receives schema, model, and its own predicted history; never gold
frames/states, per-conversation service lists, spans, actions, API results,
IDs, or gold values. System utterance text is allowed. The post-inference audit
consults annotations only after predictions exist. No runtime LLM, answer
lookup table, or external entity store is introduced.

## Fixed measurements

| Scope | Joint turns baseline→candidate | Exact frames baseline→candidate |
|---|---:|---:|
| First 10 complete conversations | 21→31 / 91 | 28→44 / 108 |
| Full 179 conversations | 549→595 / 1119 | 598→689 / 1218 |
| Multi-service 54 conversations | 92→138 / 492 | 130→221 / 591 |
| Single-service 125 conversations | 457→457 / 627 | 468→468 / 627 |

Full fuzzy joint sum: **664.1996→761.7596 / 1218**. All 3357 predicted frames
remain materialized. Active F1 stays 93.2005%; correct switches stay 42/184.
Multi-service Travel exact frames improve 0→91/158; Weather stays 130/168;
RideSharing stays **0/265**. All 44 actual copy events in this dev selection
are Weather.city→Travel.location. The reverse learned relation has train
support but no measured dev copy benefit in this batch.

Full joint transitions: **549 right→right, 46 wrong→right, 474 wrong→wrong,
50 refused→refused**; zero right→wrong/right→refused. Frame transitions:
**598 right→right, 91 wrong→right, 529 wrong→wrong**, zero right→wrong.
Sample transitions: 21 right retained, 10 wrong→right, 57 wrong retained,
3 refusals retained. Exact reports and every changed joint ID are in `reports/`.

## All newly incorrect additions, without metric relabeling

The copy audit records 304 changed stored slot-turns: 111 correct scored
additions, **11 incorrect scored additions**, and 182 held frames with no gold
frame on that turn (unscored, not claimed correct). Four incorrect initial
copy events account for all 11 incorrect additions:

| Dev conversation | USER turn indices | Copied value | Gold value | Failure |
|---|---|---|---|---|
| 17_00022 | 6, 8, 10 | San Rafael | Philly | Frozen source missed an earlier correction |
| 17_00025 | 6 | Washington D.C | Washington D.C. | Frozen span dropped final punctuation |
| 17_00034 | 6, 8, 10 | Washington D.C | Washington D.C. | Same boundary error |
| 17_00038 | 6, 8, 10, 12 | San | San Fran | Frozen span truncated the name |

All are Travel.location copies from prior predicted Weather.city, replacing an
absent target. Each value has prior utterance support, but that does not prove
it is the current correct referent. These frames were already incorrect under
the baseline; the new wrong slot claims are still errors, not harmless gains.
`reports/copy-audit.json` contains every changed slot-turn, all 44 initial copy
provenances and supporting prior utterance indices, and all 11 new errors.

After discovering the additions, a train-only frozen-prediction diagnostic
found 475 correct and 2 incorrect eligible transfer opportunities. Both train
errors exhibit the same final-punctuation truncation. This diagnoses the
train-gold versus runtime-predicted-source gap. No settings were changed after
dev; no entity-specific punctuation/city rule was added. Next representation
gap: source span boundaries, correction freshness, and calibrated source
reliability. System-offered attraction extraction remains unimplemented.

## Validation

- Four focused Rust tests pass, covering annotation rejection, empirical model
  evidence, schema checks, whole-span provenance, categorical-source refusal,
  ambiguity refusal, switching, existing-slot preservation, inactive held
  state isolation, fresh-session reset, and prior-versus-current-turn isolation.
- Final owned-crate fmt check, all-target clippy with `-D warnings`, release
  build, Python AST checks, Bash syntax, and `git diff --check` pass.
- Unchanged evaluator's full reverse-order replay is identical after reversing
  batches; annotated runtime input is rejected (179 conversations audited).
- New binary with resolver absent reproduces frozen full-dev predictions
  **byte-for-byte**. Legacy predictions also reproduce byte-for-byte: 45
  conversations, 257 frames, 202 exact, 256 correct intents, fuzzy sum229.18.
- Structural comparison confirms every router and per-service weight unchanged.
  JSON reserialization changes model-file bytes; it does not retrain weights.

Every cargo command had TMPDIR, systemd scope MemoryMax8G/MemorySwapMax0,
`setsid --wait timeout --kill-after=10s 600`, `-j1` except fmt, and tests used
`--test-threads 4`. Training/inference were also capped. Outer scope verified
8GiB/zero swap. No agents, LLM calls, services, credentials, API keys, downloads,
held-out/test items, test generators, old private logs, or commits were used.
No new jobs remain running at handoff. Required host model/login configuration
cannot be independently attested by this already-running worker; no alternate
model/authentication was launched.

## Sources, licensing, bytes and persistent artifacts

No new external data was obtained (**0 download bytes**). Reused official
SGD train/dev source at Google `google-research-datasets/dstc8-schema-guided-dialogue`
revision **e852981ae34990f4358979625854259302feaa78**, **CC-BY-SA-4.0**;
existing 150 permitted source/license files total **504,306,737 bytes**.
Training selection: 3174 conversations; selected train JSON45,291,003 bytes.
Dev JSON1,207,178 bytes; schemas3707 bytes each. No selection/preparation rerun.
Derived relation weights and data-bearing reports retain the training/data
license. Existing upstream metric is Apache-2.0 at
`5b09c22d73a9d35eb6c5d2a99b95677a45053466`, 11,209 bytes, unchanged.

Mounted read-only data used here:
`/home/gsc/.cache/glassmind/data/sgd-full`.
Its actual persistent host location:
`/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full`.
Frozen models/predictions:
`/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/multi`.
All new large artifacts are persistent under
`/home/gsc/.cache/glassmind/continuous-20261010/references-run/tmp/references`;
binaries under `references-run/target/release`. No `/tmp` storage used.

| Artifact | Bytes | SHA256 |
|---|---:|---|
| Frozen joint12 model | 32200901 | `7ad9f82b9adfd5671511feeca787193ce82ea2bf5f781ea5c6353befdff3428f` |
| Candidate model.json | 32169632 | `9991b341b4da7e775e0fa7e3ca55a2a00f4935a0a7b94db16f02b2b6596be83c` |
| relations.json | 455 | `c90a7aa0bc0b075f677048618ae7862d435c0e293afa197c645463aba692df07` |
| Full candidate predictions | 402034 | `59ade4c58ed459f5e234c0a8cd0bde3d926454359b925acf2b55bd109d221058` |

`reports/artifacts.json` records exact paths, SHA256 and sizes for source files,
unchanged evaluators, selected data, binaries, baseline/candidate models and
predictions. Large classifier models and raw training runtime inputs remain outside git.
The tiny455-byte, value-free relation artifact is retained in reports for inspection.

## Reproduction

From this worktree, on the persistent host:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/references-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/references-run/target
export SGD_ROOT=/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full
bash tools/dialogue/references/reproduce.sh
```

Inside this worker's namespace set `SGD_ROOT=/home/gsc/.cache/glassmind/data/sgd-full`
instead. The script reuses the existing metrics Python at
`dialogue-run/tmp/metrics-venv/bin/python` and existing legacy cache/model paths
specified in it. It builds/checks, fits train-only relations, assembles frozen
weights, runs fixed sample then full dev and isolation audit, unchanged compare,
copy audit, legacy regression, train-source diagnostic, and frozen replay.
Individual commands were executed in this batch; no claim that the whole
assembled script was rerun a second time is made.

Limitations: this is the declared three-service dev scope, not full SGD or a
held-out result. The resolver fills missing slots only; it cannot correct a
wrong filled target or reliably detect stale/truncated source predictions.
Evidence counts are empirical train counts, not calibrated probabilities.
No broad reference resolution, service execution, date normalization, unseen
service generalization or improved ride-destination tracking is claimed.
