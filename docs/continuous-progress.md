# Continuous development checkpoint

Goal active: follow `prompts/grow-continuously.md` until the user stops it.
Previous goal turns made concrete progress; no blocking condition exists.

## Current source and publishing

Repository `/home/gsc/code/glassmind`, branch `harness`.
Merged PR #4 (conversation calculations), main merge `f83aefc`;
merged PR #5 (exact percentages/MathQA), main merge `d607271`.
Merged PR #6 (syntax repairs and acronym guard), merge `138676e`,
evaluated PR head `ed81553`. New verified local commits awaiting publication:
`ea52c58` (type renaming), `797ba62` (story conversation), `ebd9817`
(fixed byte probability scorer), `422d754` (story focus correction).
Their workers completed. A new dialogue worker is active below.
Do not push worker candidates until their affected full dev evidence passes.

The latest complete benchmark records are in `docs/results/tracks.jsonl`;
`docs/results/tracks.html` is regenerated at each batch. Historical reports
retain their original scope and source revisions.

## Completed and verified this continuous run

- Main `talk` calculates exact quantities while preserving memory and session
  behavior. Mixed synthetic conversation 18/27→27/27 with/without intent
  model; memory41/41; full3116 synthetic quantities through talk retain
  3056 correct,60 unsupported,0 wrong, including762 expected refusals.
  Source `b4b1b3f`, report `docs/talk-quantities.md`.
- Percentages: source `e3eb10f`; MathQA official dev4475 under the custom
  exact-numeric/no-choice metric improves5/4469/1→14/4458/3 right/refused/wrong.
  The two extra scored errors compare exact arithmetic with rounded choices;
  counts remain unchanged by adjudication. A separate300-case Fraction audit
  passes. All3116 synthetic status/value/unit outputs unchanged. Gold-option
  normalization cannot handle1229 rows; they stay in the denominator. No test
  downloaded. Details `tools/quantities/HANDOFF.md`.
- Coding: source `75e3269`; full765 rustbench dev at unchanged `f79e71a`
  improves160/602/3→163/599/3. Exactly three refusal→pass transitions for
  `fix-tests/steps-moving-average`, one per style; all other762 outcomes
  unchanged. `docs/results/coding-syntax-repairs.json` records binary/result
  hashes. Coordinator extended the worker AST guards to protect test-only
  locals, expressions, arms, fields and enums. Seven focused tests pass.
  Full evaluation session52107 is COMPLETE; do not restart or keep polling it.
- Facts: source `c9ed6ed`; unchanged500-entity index/frozen model. Full2821
  dev256/2526/39→256/2545/20, all correct answers retained, no new claims.
  Sample12/186/2→12/188/0. Train diagnostics remove154 wrong claims without
  losing a right answer or adding a claim. Rejecting acronym casing keeps
  the original ambiguity and never exposes another candidate. First filter
  design created new train errors and was rejected. Five focused tests pass;
  coordinator independently reproduced final full dev. Reports in
  `tools/facts/continuous/`.

Workspace fmt, all-target clippy, release build and strict4/4 fixture smoke
pass after integrating coding and facts. No held-out evaluation has been run
in this continuous batch; none of its items was inspected.

## Verified current batch and next experiment

WORK = `/home/gsc/.cache/glassmind/continuous-20261010`.

- **Type renaming**: `ea52c58`, conservative explicit module/import resolution
  for struct/enum names; ambiguous bindings, unexpanded macros and unknown
  source files refuse. Coordinator reproduced all9 local cases, including
  exact intended source, same runtime output for3 positives, and6 refusal/
  rollback cases. World-rust tests10/10. Integrated evidence is
  `docs/results/coding-edits-integrated.json`.
- **Story conversation**: `797ba62`, bounded512-assertion journal with legacy
  session compatibility and explicit prediction labels. Coordinator found
  bare ambiguous corrections could preserve stale story state; fixed routing
  only when story context exists. The first coordinator fix changed a
  memory-only clarification; rejected and narrowed. Six CLI tests pass (five original plus the new focus regression).
  Final source `422d754` also clears the previous personal-memory focus on
  story turns: a bare correction cannot replace an earlier name with a story
  location. An explicit memory question establishes its focus again.
  Final integrated47/47 mixed checks with/without intent model,41/41 memory,
  27/27 calculations with/without model, fullbAbI3966/18/3 of3987 and quantities
  3056/60/0 of3116 (including762 expected refusals). Final binarySHA
  `94c89f0184b1b8af183f727bb177c5c115d4b20e59d182d80a0496c2f030a0ef`.
  Reports in `docs/talk-stories-evidence/integrated/`; all those checks complete.
- **Reading**: `ebd9817`, fixed non-generating byte6-gram trained only on
  BabyLM2025 train10M. Coordinator full1340/120 evaluations exactly reproduce
  worker reports; two probability/persistence tests pass. Grammar unigram
  578good579bad183ties ->707good633bad0ties. **185 good-to-bad transitions,
  23/67 categories regress.** Reading byte-weighted BPB4.8212138605 ->
  2.1892595993; **33/120 excerpts exactly overlap training lines**, disclosed.
  Separate probability command; no semantic/independent-test claim. All
  settings/provenance/licenses/reproduction in `tools/reading/HANDOFF.md`.
  Model retained `WORK/reading-run/probability/byte6.bin` (68,861,296bytes).
- **COMPLETE: full765 RustBench dev**, exec66778 terminal; do not poll or
  restart it. All outcomes unchanged:163pass599refused3fail. The nine local
  rename cases establish a bounded new capability, not a benchmark gain;
  all81 benchmark edit requests still refuse. Full report with hashes is
  `docs/results/coding-type-renames.json`. Frozen result/binary retained in
  `WORK/coordinator/edits-full/`. No held-out evaluation.

**Active worker: dialogue**, launcherPID1756969, scope
`glassmind-continuous-dialogue.scope`, worktree `WORK/dialogue`, base `eb15517`.
Owns crates/dialogue and new tools/dialogue only; no shared CLI/scorer changes.
GPT-6 Astra fast via ChatGPT, capped8GiB/zero swap. Goal: actual learned
service routing and separate state from utterance-only inputs. Initially
selected RideSharing_1+Travel_1 using training co-occurrences; no multi-service
dev conversations, so rejected before scoring. Added Weather_1 from training
availability. Fixed complete selection now179 dev conversations (54multi),
1119user turns,1218frames; model training/evaluation in progress. Frozen
RideSharing regression reproduced202/257 exact,89.1751%fuzzy. Inspect final
handoff; do not assume candidate accepted.

Worker downloaded permitted pinned train/dev into its namespace's temporary
cache `/home/gsc/.cache/glassmind/data/sgd-full`. Coordinator preserved all150
source files with manifest hash verification, plus selected files, at
**`WORK/dialogue-run/data-sgd-full`**. Do not lose this data or redownload.
The worker's internal path disappears when its namespace closes; fix/parameterize
reproduction paths during integration. Current namespace ownerPID1756977
(`/proc/1756977/root/...`) if further permitted artifact recovery is needed.
Do not read unrelated paths or credentials. Worker model/reports under
`WORK/dialogue-run/tmp/multi` persist normally.

Workspace fmt/all-target clippy/release build/strict4/4 smoke pass after
all integrations. Scalar bits-per-byte report support added to
`tools/track-report.py`, including separate plot scale, finite-value checks,
explicit tie counts and model metadata. Rate/scalar/invalid-input checks
pass; render is current. No cargo job should be started uncapped.

All storychat/edits/reading workers used GPT-6 Astra fast through ChatGPT;
scopes inactive and temporary auth symlinks removed. Their changed files were archived with SHA256 manifests to
`WORK/NAME-run/submitted`, then worktrees removed. Original story submission
differs from coordinator corrections, and summarized baseline reports link
the retained full originals in that archive. Do not restart workers or duplicate their output.
Their `NAME-run/{events.jsonl,summary.txt}` retain completed evidence.

## Completed worker cleanup and retained evidence

Coding, quantities and facts scopes ended; their temporary auth symlinks
were removed and worktrees removed. All quantity/facts files were verified
against integrated files before removal. Original coding submission is
archived in `WORK/coding-run/submitted` because coordinator guards differ.
Their logs, targets, development outputs and data remain in `*-run/`.
Known completed worker tokens:29,566,822 including cached input; parent and
unfinished worker usage excluded. Update `docs/results/worker-usage.json`
once per newly completed worker, without double-counting.

Frozen coding binary and full result: `WORK/coordinator/coding-full/`.
Before-results: `~/.cache/glassmind/grow-fast-20261010/keep-going-candidate/rustbench-dev/`.
Facts raw before/after train/dev predictions: `WORK/facts-run/tmp/`.
MathQA and PyArrow: `WORK/quantities-run/tmp/{mathqa,quantities-deps}`.
Do not delete retained data/models under `~/.cache/glassmind/{data,models}`.
The old improvement loop remains STOPPED; preserve its dirty checkout/state.

## Immediate next actions

1. Publish this verified batch as nextPR (#7), inspect CI and merge with actual
   headSHA. Full coding/session/reading evidence is complete; no benchmark
   process remains. PR6 already merged.
2. Review active dialogue worker when done, reproduce full declared selection
   and annotation isolation, preserve frozen single-service regression. Fix
   namespace-only data paths using the preserved cache above. Do not claim
   multi-service routing from oracle service names or score only easy frames.
3. Coding has plateaued on all81 edit tasks despite the valid local rename
   subset. Next coding hypothesis should target realistic source layouts and
   reusable requested transformations, with actual before/after behavior and
   rollback. Do not inspect hidden tests/generators or expected patches.
4. Facts expansion should use independently selected/training-derived entities;
   existing index is validation-seeded. Reading follow-up needs a declared
   independent dev design with overlap diagnostics. Keep measured scopes honest.

Read the active prompt for standing authorization and stop/cleanup rules.
This checkpoint is not a request for user approval or a completed goal.
