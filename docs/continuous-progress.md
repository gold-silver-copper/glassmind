# Continuous development checkpoint

Goal active: follow `prompts/grow-continuously.md` until the user stops it.
Previous goal turns made concrete progress; no blocking condition exists.

## Current source and publishing

Repository `/home/gsc/code/glassmind`, branch `harness`.
Merged PR #4 (conversation calculations), main merge `f83aefc`;
merged PR #5 (exact percentages/MathQA), main merge `d607271`.
Merged PR #6 (syntax repairs and acronym guard), merge `138676e`,
evaluated PR head `ed81553`. Merged PR #7, head `0616ecc`, merge `16757c3`, CI38109674013 green.
Included verified source commits:
`ea52c58` (type renaming), `797ba62` (story conversation), `ebd9817`
(fixed byte probability scorer), `422d754` (story focus correction).
Their workers completed. Source commits `0bff6a1` (enum variants) and
`da96f49` (multi-service dialogue with grounded predicted-slot transfer) are
merged in PR #8, head `fcb81dbf17273dcca4d45f7a8df4d3428fb217a0`,
merge `cb150965a9d0e95ffd234a7f4bf38299e8370d86`; CI38110755616 passed1m15s. All affected dev
checks are complete. The offers follow-up is verified; package-binding and word-problem workers
are active below.

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

- **Enum variants**: source `0bff6a1704dcdd840058069c9e279a8fd61aa41e`.
  Unit/tuple/record constructors, patterns, explicit imports and inherent Self
  resolve conservatively. Main CLI kind bridge integrated; compiler-required
  oracle arms only label/skip the new kind, without evaluation changes.
  Sequential main-CLI sample3/3, full14/14 (four edits, nine refusals, rollback),
  previous type9/9. World14 + English35 tests pass. Full765 RustBench dev
  **COMPLETE**, session12870 terminal:163pass599refused3fail, every outcome
  unchanged including81 edit refusals. No benchmark gain. Source/binary/result
  hashes in `docs/results/coding-variant-renames.json`; local CLI evidence in
  `docs/coding-variants-evidence/integrated/`. Frozen binary/results remain
  `WORK/coordinator/variants-full/`. Do not rerun or poll completed evaluations.
- **Multi-service dialogue**: source `da96f494d4a6865924cb0aa5eea138b029158fc2`.
  Initial joint/independent-router hypotheses REJECTED under their original
  sample/full joint-state criteria. Reports `docs/dialogue-routing-experiment.md`
  and `docs/results/dialogue-routing-rejected.json`; no rejected chart gain.
  Their frozen joint12 is the ablation baseline for learned slot transfer.
  Fixed train-derived three-service selection: RideSharing_1, Travel_1, Weather_1;
  dev179 complete conversations(54multi),1119user turns,1218frames. Runtime sees
  speaker/utterance arrays only, never gold frame/service/ID metadata.
  Value-free455-byte relation model learns Travel.location↔Weather.city from
  train evidence83/83 and82/82. Copies only missing active-service input slots
  from prior predicted states supported in prior utterance text; no values
  stored in relation model, no overwrites or current-turn label transfer.
  Sample21→31/91 exactjoint, full549→595/1119, exactframes598→689/1218,
  multi92→138/492 turns and130→221/591 frames. All549 baseline-correct retained;
  remaining474wrong50refused. Routing unchanged(F1 93.20%,42/184 switches).
  MultiTravel0→91/158; **RideSharing0/265 still zero**.
  **Four incorrect copy events persist11 scored slot-turns**: stale city,
  missing final punctuation and truncated source; no dev-specific fix applied.
  New claims111correct/11incorrect scoredslotturns,182heldunscored. Preserve
  this error disclosure despite no correct→wrong whole-frame transitions.
  Coordinator independently refit/assembled and reran full with isolation,
  baseline replay and legacy regression: relation/full/legacy bytes identical.
  Old single-service202/257 exact89.1751%fuzzy unchanged. Four focusedtests pass.
  Full reports `tools/dialogue/references/HANDOFF.md` and `reports/integrated/`.
  Candidate model `WORK/references-run/tmp/references/model.json`,32169632bytes,
  SHA9991b341b4da7e775e0fa7e3ca55a2a00f4935a0a7b94db16f02b2b6596be83c.
  All coordinator evals complete(86827,56716); no repoll/restart.

**Offers worker COMPLETE**, scopeinactive/authremoved; submission archived in
`WORK/offers-run/submitted/` with sourcehashmanifest; worktree removed after archive hash verification.
Integrated source `ba98b5a`: optional learned bounded span selector after frozen
reference resolver, immediately previous SYSTEMsource only, exact UTF8provenance,
missing active input slots only, consumes source on nextUSER. H1rejectedtrainonly
zeroacceptedrules; H2propername shape yields26rules(25destination,1city).
Worker sample31→35/91, full595→609/1119, frames689→712/1218,
Ride0→23/265. All595correctretained,460wrong50refused;11events32correctnew
slotturns,zero newwrong/unscoredclaims. Inherited4wrongcopies/11slotturns unchanged.
Six tests and frozenbaseline/legacy replay pass inworker. Source/train/audit
reviewed. Candidate modelSHA deddfaac2f596c5090aeb65bf77d24470d1d44f61d2aad69469792cd26232781,
32174437bytes at `WORK/offers-run/tmp/offers/model.json`.
**Coordinator verification COMPLETE**: workspacecheck38227 andsix focusedtests80512
passed. Independenttrain/sample/full/isolation/baseline/legacy replay92470 completed;
model/full/legacy bytes identical. Do not repoll/rerun completedjobs.
Script `WORK/coordinator/offers-replay.sh` omitsduplicatecargo from reproducibility
script; setTMPDIR=`WORK/coordinator/offers-check`,CARGO_TARGET_DIR=root/target.
Candidate is verified and **PR9 MERGED**, head
`742cc2a9a0ac501f8de5d93d54f55c6476b168c5`, merge
`91b001505e88cbc65770f0f4b1ff52bb21034670`, CI38111006906 green1m8s.
README/data/scoreboard reflect609/1119,460wrong50refused. Evidence
`tools/dialogue/offers/reports/integrated/`. PR8 is already merged.

**Active worker: packages**, launcherPID1964290, scope
`glassmind-continuous-packages.scope`, worktree `WORK/packages`, base `fcb81db`.
Owns world-rust/necessary typedEnglish and newcodingpackage fixtures/reports.
Hypothesis: bounded explicit library/binary package graph and reexports support
actual type/variant renames. Preserve unknown attrs/macros/cfg refusals, exact
bindings, rollback, type9/9 andvariant14/14. Dev workspace-only sources/prompts
for two refused rename tasks copied into ownedrun/dev-inputs; NO hidden tests,
solutions or expected patches. Worker notes cfg(test),Debug, matches! still
blockthese realdevcases; avoidclaimingactualbenchmarkgainfromlocalfixtures.
Latest: firstsample0/3 exposed TOMLdocument parsingbug; worker corrected it
and is rerunning localchecks. All realdevcases maystillrefuse(cfg/Debug/matches).
A clippy collapsible-if failure is being fixed. Candidate NOT accepted.
No extra service/worker/model calls. GPT6Astra fastChatGPT login confirmed.
Parentmustfull765beforepublishingcandidate. Run artifacts ownTMPDIR/target.

**Active worker: wordproblems**, launcherPID1990887, scope
`glassmind-continuous-wordproblems.scope`, worktree `WORK/wordproblems`,
base `742cc2a`. Owns crates/quantities and newtools/quantities/wordproblems.
Selected family: two-to-four-number ratio totals, ordinal/largest/smallest
queries and three-number adjacent-ratio chains, fromTRAIN Problemtext. Baseline
first100/full independently reproduced; representation fixedbeforeediting.
Noformula/rationale
inspection, noansweroptions at runtime, checkedrational/units andfullconsumption.
BaselineMathQA14right4458refused3wrong/4475; first1001/99/0; synthetic3056right
(including762expectedrefusals)60refused0wrong/3116. Fixedsamplemustnotregress,
fullcorrectmustincrease, preserveoldcorrect andsyntheticoutputs; auditallnew
answers includingroundingmismatchesaswrong. No dev-specific answer patterns.
Existingraw train/devParquet+PyArrow read-only fromWORK/quantities-run/tmp;
zero new downloads. ModelGPT6Astrafast/ChatGPT/noAPIkeys,8GiB0swap,j1.
Report tools/quantities/wordproblems/HANDOFF.md; no manifest/CLI/scorerchanges.

Retained SGD train/dev150files504306737bytes, pinned
e852981ae34990f4358979625854259302feaa78,CC-BY-SA4, source+selection hashes:
`WORK/dialogue-run/data-sgd-full`. Rescued and hashverified before namespace exit;
original temporary namespace no longer exists. Reproduction scripts accept
SGD_ROOT and default to this persistent path. Do not redownload.
Initial dialogue prototype archived `WORK/dialogue-run/submitted/`;
references/variants submissions archived with hash manifests in their run dirs.
Their scopes ended/auth symlinks removed; archived worktrees removed after
verifying every changed-file hash against its archive.

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
Known completed worker tokens:36,747,202 including cached input; parent and
unfinished worker usage excluded. Update `docs/results/worker-usage.json`
once per newly completed worker, without double-counting.

Frozen coding binary and full result: `WORK/coordinator/coding-full/`.
Before-results: `~/.cache/glassmind/grow-fast-20261010/keep-going-candidate/rustbench-dev/`.
Facts raw before/after train/dev predictions: `WORK/facts-run/tmp/`.
MathQA and PyArrow: `WORK/quantities-run/tmp/{mathqa,quantities-deps}`.
Do not delete retained data/models under `~/.cache/glassmind/{data,models}`.
The old improvement loop remains STOPPED; preserve its dirty checkout/state.

## Immediate next actions

1. PR8 merged after green CI; no publishing or benchmark work remains for it.
   README/data/scoreboard updated; reports identify exact evaluated source.
2. PR9 merged and all independentreplay/checks complete; no pendingpublishing.
   Review packages/wordproblems when done, verifyacceptedchanges andcontinue.
3. Next coding experiment: dev rename requests parse but refuse. Permitted
   workspace sources expose library+binary roots, docs/derive attributes and
   reexports that the closed resolver rejects. Investigate a sound bounded
   extension using independent fixtures; do not read hidden tests or solutions.
4. Keep coding and conversation advancing. Other gaps: full MathQA mostly
   unsupported language; facts open-world retrieval; independent reading
   semantics (current byte likelihood is not comprehension).
5. Active goal continues until user stops. Stop owned jobs on pause/stop,
   preserve artifacts, never restart the old stopped loop blindly.
