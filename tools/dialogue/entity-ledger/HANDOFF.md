# REJECTED: learned SYSTEM entity ledger

One frozen experiment completed. **No candidate DEV inference, second grid,
production integration, accepted model, commit, agent or external model call.**
The new representation recovers many correct CAL slot-turns but every declared
setting makes new incorrect assertions and loses baseline-correct outcomes.
Do not deploy the retained model. Approved v2 remains the supplied
610/1119 joint and716/1218 frames; these DEV scores were not remeasured here.

Base: 2a35c179f34e275b581afc4ac45e50ab0a032940. Only this new evidence directory
survives cleanup. All original crates/dialogue files are byte-identical; the
new offline example/module are archived in native/ with their complete patch.
No README, shared manifest, main CLI, loop, scorer, prior report or other track
was edited. The coordinator's separate worktree was not inspected.

## Exact experiment

[EXPERIMENT.md](EXPERIMENT.md) froze the hypothesis, features, 6/8/8 epoch
schedule, limits, split and16-member confidence grid before fitting. Three
small averaged Rust perceptrons learn SYSTEM BIO boundaries, semantic source
roles (including output-only slots), then candidate-versus-NONE binding.
The learned role labels come only from FIT SYSTEM span annotations. The role
and binding features mask the proposed entity; binding also masks every ledger
value in USER text. No stored answer list, gold runtime frame, entity synthesis,
pretrained model, reused rejected weights or hardcoded service pairing exists.

The private ledger keeps at most128 exact UTF8 spans from the last32 SYSTEM
turns, including source utterance/turn/byte range, predicted role and confidence.
Inputs cap4096bytes/128tokens; spans cap16tokens. Schema roles and finite bounded
weights are validated. The runtime InputTurn type rejects annotation fields.
The native harness is offline only: gold supervision and actual frozen model
snapshots are separate fields, unavailable to extraction/ranking functions.

All3174 selected TRAIN conversations are accounted for, including unselected
service turns:2532FIT/642CAL by SHA256(dialogue_id) integer mod5==0;
34022USER/34022SYSTEM turns,24415USER turns with unselected service frames.
The frozen baseline was actually replayed on utterance-only inputs first.
There are5654FIT and1432CAL missing-slot opportunities, including1033/270 with
no candidate. Ranker traces retain51647 candidates and all negatives.
Boundary training used27192FIT SYSTEM sequences; role training27160 aligned
spans over128 FIT-selected labels (minimum20 occurrences, capped128). Ten
retained-role annotations could not align to the declared token/span bounds.
No CAL examples proposed labels or trained weights.

The older frozen models saw all TRAIN, and previous diagnostics inspected both
partitions. These are **empirical new-component validation** counts, not unseen
calibration, general SGD accuracy, or candidate DEV results.

## Gate and complete assertion audit

Full642CAL conversations retain6830USER turns and2505 selected-service gold
frames. No USER turn is dropped. CAL joint diagnostics project gold services
to the approved schema; unselected-only correct abstentions contribute, so
these joint totals must not be compared with the179-conversation DEV score.
The unchanged upstream metric computes strict selected-frame outcomes.

All16 settings fail:474–604 correct scored fills,41–71 newly wrong scored
assertions,150–191 held/unscored changes, and2–4 lost baseline-correct joint
turns/frames. Held changes receive zero correctness credit. Complete per-grid
counts/IDs are in [gate.json](reports/gate.json).

| Source margin / rank / distinct-value gap | Initial events | Correct scored fills | New wrong scored | Held | Lost joint / frames |
|---|---:|---:|---:|---:|---:|
|0 /2 /2|207|604|71|191|4 /4|
|2 /16 /4 (strictest, diagnostic only)|159|474|41|150|2 /2|

The gate uses a preregistered persistent overlay on frozen baseline snapshots:
existing mechanisms' values take precedence, new missing-slot fills persist,
and every changed slot-turn is counted. It does **not** feed candidate values
back into old references/offers. This was a conservative pre-integration
rejection screen; even a pass would have required actual integrated CAL replay
before DEV. It is not a claim about a fully integrated model's accuracy.
No optional MultiModel field or draft-bundle behavior was implemented.

Raw proposals/golds/features, all22912CAL decisions, initial events for each
grid, and every changed stored slot-turn remain under artifacts/. Independent
review reconstructs every decision, checks every source against the actual
indexed SYSTEM turn, all byte slices, recency, FIT label selection, offline
gold matching, and numeric rank scores (maximum f64-vs-f32 error0.000162).
The strictest grid's41 errors diagnose26 predicted-role errors,10 reference
selection errors and5 span-boundary errors. This post-rejection classification
is diagnostic only; it caused no fit, grid change or DEV inference.

Existing CAL scored assertions are untouched by construction; the strict raw
value audit found zero existing nonmatching asserted CAL values (missing slots
still make many baseline frames wrong). This does not erase inherited DEV
errors: the seven prior wrong Travel.location claims at17_00022 turns6/8/10
and17_00038 turns6/8/10/12 remain unresolved by unchanged source/model identity.
No claim of their repair, or accuracy credit for held states, is made.

## Validation, failure and preserved evidence

Four new native tests pass: Unicode exact byte provenance, deterministic
capacity/32-turn expiry/reset, NONE/ambiguity with exhaustive81 two-candidate
score pairs, annotation rejection/finite weights, and value-masked binding.
All14 existing library tests pass. Crate fmt check, affected all-target clippy
with -D warnings, and release example/package builds pass. Rebuilt approved
v2 reproduces all3174 TRAIN prediction bytes from the supplied frozen binary.
Draft and legacy source are unchanged. Legacy45, reverse DEV, and first10/full
candidate DEV were **not run**: the failed CAL gate forbids candidate DEV;
no new legacy score is claimed.

One audit infrastructure failure occurred: upstream NumPy integer counters
were not JSON serializable. Converting report counts to Python int fixed only
serialization; no scoring definition or fitted artifact changed. Original
script/failure log and final audit remain. No native build/test failure occurred.
The representation was fitted once; no second hypothesis, epoch schedule,
source model, threshold grid, or rejected spansequence weights were tried.

Before removing the two temporary crate files, original and final source,
complete new-source patch, model, native binaries, inputs/predictions/raw traces,
commands, failures and reports were archived and every archive member hash
verified. See reports/archive-verification.json and reports/restored-source.json.
Large local evidence is ignored by Git in artifacts/; preserve it for independent
review before any coordinator cleanup. native/ and rejected-source.patch are
compact, reviewable reproduction sources, not a production change.

## Reproduce

From this checkout root (source and metrics interpreter already mounted):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/entityledger-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/entityledger-run/target
bash tools/dialogue/entity-ledger/reproduce.sh
```

The script installs the archived offline source temporarily, rebuilds/tests,
replays the frozen TRAIN model, prepares supervision, fits once and audits all
CAL settings; it expects REJECTED_CAL and never runs candidate DEV. It removes
only its two temporary source files on exit and retains evidence in persistent
TMPDIR. Constituent commands ran in this batch; the consolidated script was
syntax-checked, not rerun end-to-end. reports/commands.log records executed
bounded Cargo/native/audit commands; reports/COMMANDS.md records preparation,
archive and limitations. All jobs finished; no owned background process remains.

## Data, license, identities and limitations

New external/downloaded data: **0bytes**. Source package11files/120818396bytes
was hash-verified against its supplied manifest. SGD revision
**e852981ae34990f4358979625854259302feaa78**, **CC-BY-SA-4.0**:
TRAIN45291003bytes, full TRAIN schema75764, selected schema3707,
LICENSE20139. Derived supervision28805932bytes; derived weights and data-bearing
reports remain CC-BY-SA-4.0. New code MIT OR Apache-2.0. Upstream metrics remain
unmodified. Exact package identities: reports/source.json.

| Artifact | Bytes | SHA256 |
|---|---:|---|
|New rejected weights|3572204|661c80b75a808994427a53591c651cb3d1ddc2fc1e0624503d6b70e4ca09b0e0|
|Evaluated native harness|11403288|d6ea632e16d531fe1ec71e7c611590bb4218a580e799beb1507ad529247a1c8f|
|Complete proposals|62308927|3f942ea8d47e7784ef36959b538e5c206cfd6b13223eabdae982097ecf261980|
|All CAL decisions|2349545|cb59fe5d0a86ad0823527df3fb53758a9c15dc35e3061f4652383d55bd2a01a5|
|All changed CAL stored claims|11005121|9402aaa6e0dee5c0ddf5bd6a9c08d72f774680990d1d0b11160fb19f08f9c2b0|

New persisted weights are below16MiB. Hashed lexical source features can overfit
and collide; semantic role confusion and referent selection remain substantial.
Provenance is not proof of intent, and learned negatives did not provide safe
correction/negation handling. Source confidence is a score margin, not a
calibrated probability. Native loading hardening for a deployable optional bundle
was not attempted after rejection. No generalization or production improvement
is claimed. This negative result supports a different source representation's
candidate coverage, but decisively fails the stated acceptance criterion.

Every Cargo invocation had persistent TMPDIR, the existing target directory,
8GiB/zero-swap systemd scope, setsid --wait, timeout --kill-after=10s600,
-j1 (except fmt), and test threads4. Outer scope8589934592bytes/0swap verified.
No agents, model calls, API keys/credential reads, new downloads, services,
held-out/officialTEST/hidden assets, private logs or commits. The already-running
host model/login mode was not independently attestable; no alternate model or
authentication was invoked.
