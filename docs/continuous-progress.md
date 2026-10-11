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
checks are complete. PR10 (ratio totals) is merged, head `7a5e17627605f6628660c0529a5e96d2a87e96ce`,
merge `b8c28e5d1b4e8e1029b9d8f5935dc75645ac0226`, CI38111396832 green1m7s.
Package source05e8093 passed full765 regression with every outcome unchanged.
Repair source2ce9230 passed full765:166pass596refused3fail; onlythree
derive-debug styles improve, everyother762 outcome unchanged.
Facts source d1076c4 passed full facts and all conversation/config replay.
PR11 merged: head57eb656ded41d452bf5d8199cfcf182a73951162, merge
7f847d9ca372d64d9ab82c6815f1336d1594172c; CI38112513112 passed1m25s.
Linearwords source687f275 is integrated and fully verified, preparing publication.

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

**Packages worker COMPLETE**, scopeinactive/authremoved, archived submissions
`WORK/packages-run/submitted/`; worktree removedafterhashverification.
Integratedsource `05e80930ef2b06e8e054ece3ab4cc69666d19b98`: bounded Cargo
library/binarygraph, explicitimports/reexports, root-specificidentities, builtin
inertderives/literalprosedocs. Unknowncfg/macros/Debug/targetambiguity refuse.
Newtoml0.9.12+spec1.1.0 dependency;10newlockfilearchives782407bytes, licenses/hashes
in docs/coding-packages-evidence/provenance.json. No datasetdownload. Independent
fixtures3816bytes, MIT/Apache2. Source-only devsnapshots fromRustBenchf79e71a;
no upstreamLICENSEavailable, nohidden/solutioncontentread.
CoordinatorcheckedintegratedCLI: sample3/3, full15/15, type9/9,variant14/14.
Allsixsuppliedrealdevpromptsstill0edits andbefore/afterbuildpass. Fmt/alltarget
clippy/release/strict4/4smoke and16worldtests pass. Sessions22773,81603,90645
COMPLETE; no repoll/rerun. Reports docs/coding-packages-evidence/integrated/.
**Full765 RustBench dev COMPLETE**, execsession32029 terminal; do not repoll.
All outcomes unchanged: 163 pass, 599 refused, 3 fail. Comparison includes every
(task, style) and all765 cases. Frozen binary SHA
52a113aa63a18dca7449103d57142e24c17d4cdbe11f68b6a09bce92daf474f9,
source05e8093, outputs WORK/coordinator/packages-full. Report
`docs/results/coding-package-renames.json`; local package capability is not a
RustBench gain. Await repair full gate before publishing the combined batch.

**Repairs2 worker COMPLETE**, scope inactive, auth link removed, submission
archived under WORK/repairs2-run/submitted. Source integrated as
`2ce9230ce5f7ca940f1b822667036212daf2420d`. Compiler E0277 identifies a unique
struct/enum missing Debug; add standard fully-qualified derive with a whole
item anchor. No placeholder bodies or changes to fields/functions/tests.
Refuse direct conditional/custom attributes and ambiguous declarations;
conditional out-of-line parent module semantics are not fully resolved.
Parent workspace fmt/clippy/build/strict4/4 smoke and19world tests pass;
local sample1/3, full3/18 repaired; independent4repairs4refusals. Changed cases
build and pass functional probes; refusals retain original source and errors.
Sessions10901,39926,96015 COMPLETE. Integrated reports
`docs/coding-diagnostics-evidence/integrated/`.
**Full765 RustBench dev COMPLETE**, exec57671 terminal, frozen source2ce9230 binary SHA
dfd7453d8a9a3dd11174f40683bf4225972e4e1f7022f595b53ea4b37e6d4bdf
Exact digest also in
WORK/coordinator/repairs2-full/source.json. Separate cargo target/check cache/
scratch. Compared to completed packages-full163/599/3:166/596/3, exactly
three refusal→pass transitions for derive-debug, allother762unchanged. Report
docs/results/coding-standard-debug.json. No remaining coding gate or repoll.

**Factchat worker COMPLETE**, scope inactive, auth link removed; original
submission archived/hash-verified, worktree removed. Integrated source
`d1076c4c97638f7750d8d7745ce1ef5a8db16827` adds optional `talk --facts-dir`,
immutable index/model, display-only labels and persisted correction-focus clearing.
Parent replaced hardcoded asset hashes/counts/FNV with version1 manifest+SHA256,
portable packaging script and concise source-linked output. Independent bundle/
label-isolation unit tests pass. Facts algorithm/model/index/scorer unchanged.
Full parent facts replay COMPLETE: sample12/188/0, full256/2545/20 of2821;
all276 baseline accepted ID sets preserved. Existing20 wrong remain. Finite500
validation-subject-seeded index, no open-world/new coverage claim.
Workspace fmt/all-clippy/release/strict4/4smoke COMPLETE passed; exec78986 terminal.
Initial clippy large enum warning fixed by boxing FactReply; final check passed.
**Parent conversation/config replay COMPLETE**, exec86463 terminal, outputs
WORK/coordinator/factchat-check, script WORK/coordinator/factchat-replay.sh.
Full source/binary metadata in factchat-check/source.json, binary SHA
3a2431e7e0a2b686c831b94da32180f3dcbbb5da7686b1e5e5ceaa96836b01a0.
Final8 CLI tests passed after boxing; tests-final.log retained. Allfourlegacy
configurations retain memory41/41,calculations27/27,stories47/47. Both mixed
config/display runs28/28. Reports/integrated complete with hashes/log pointers.
Completed fact predictor build80376, first tests77389; no repoll.
Worker reports tools/facts/chat/HANDOFF.md and reports/; parent reports/integrated
complete. No runtime network/LLM. Optional English labels improve accepted-answer
presentation19→112/276;164 still need QID fallback. New70officialWikidata label
responses363928bytes CC0; final3806eligibleIDs3361claimtargetlabels. Bounded fetch
stopped on429, no further requests; initial HTTP error counts unknown/disclosed.
Manifest hashing adds8locked cached dependencies219434archive bytes; licenses,
checksums recorded in reports/dependencies.json. All raw data remains in worker
run; optional reproduction uses cached responses without downloading.

**Linearwords worker COMPLETE**, scope inactive, auth link removed, submission
archived/hash-verified and worktree removed. Original worker24right/4448refused/
3wrong rejected: ordered interpretation of English difference returned-1 for
"difference between2and3". Parent correction enumerates both signs of at most4
absolute differences, checks their nonnegative conditions and all equalities,
and requires agreement among all feasible solutions. Explicit minus stays
ordered. Two worker-scored-correct cases now refuse because both signs of the
unknown are possible; no positivity assumption. Source
`687f2751ff256f2071330590345ef0652fb09baa` integrated locally, not yet published.

Parent full4475MathQA **COMPLETE**:16→22right,4456→4450refused,3wrong unchanged;
all19baseline answer records preserved; six new values independently recomputed.
Fixedfirst1001/99/0;1229unparseable gold rows retained. All3116synthetic triples
unchanged3056right(including762expectedrefusals),60unsupported,0wrong.
All50 independent composition +55 difference checks pass;9 quantity tests,
focusedclippy andworkspacefmt/allclippy/release/strict4/4smoke pass. Main talk
8/8newmixed checks in allfour facts/model configurations; old27/27 each with/
without model unchanged. Source/binary/log identities and original assumptions
in tools/quantities/linearwords/{PARENT.md,integrated/validation.json}.
Frozenquantitybinary SHA06a651b1817f022677f1c898ba8f71bb53eb2d6ed7ad5c42c707b356a869768a,
WORK/coordinator/linearwords-check. All jobs complete:19270focused,97485workspace,
27303main-talk. No repoll/restart. Original24 score is superseded by22.
All measured new benchmark gains areone-unknown; two-variablecomposition has
localfixture evidence only. No newdatasets/models/dependencies orheldoutaccess.
No active modelworker; launch settings remainGPT6AstrafastChatGPT/noAPIkeys.

**Wordproblems worker COMPLETE**, scopeinactive/authremoved; source/reports
archived `WORK/wordproblems-run/submitted/`; worktree removedafterarchiveverification.
Integratedsource `6ac211b7452383c52ec3791bee40f0b8f69b31ef`: fully consumed
ratio totals over2–4 positive rational weights; ordinal/rankqueries andadjacent
three-number chains. MathQAfull14→16right,4458→4456refused,3wrong unchanged;
all17oldansweredrecords preserved; fixedfirst1001/99/0 unchanged. Bothnewdev
answers sharetrainingtemplates, notbroadwordproblemunderstanding. All3116
syntheticstatus/value/unitoutputs unchanged. Coordinatorindependentfull/sample
replay identical toworker. Main talk4newindependentratioquestions0→4correct
with/withoutmodel;3memory/recall checks preserved; old27/27 each unchanged.
Fivequantitytests +workspacefmt/clippy/release/strict4/4smoke pass. Completed
sessions11570,48624,99520,37727; DO NOT repoll/restart. Integratedreports at
`tools/quantities/wordproblems/reports/integrated/`, frozenartifacts
`WORK/coordinator/ratio-check/`. Sourcehash/binaryhashes in validation.json.
Worker flaggedreadinglocalvisible syntheticdevadapter asprohibited; parent
clarifiedgoalpermits this repository-visible development tooling, withnohidden
orheldoutdata access. Archive retainsoriginalreport; frozenJSONLusedforregression.
MergedPR10 asrecordedabove; no additionalverification/publishing remains.

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
Known completed worker tokens:47,162,504 including cached input; parent and
unfinished worker usage excluded. Update `docs/results/worker-usage.json`
once per newly completed worker, without double-counting.

Frozen coding binary and full result: `WORK/coordinator/coding-full/`.
Before-results: `~/.cache/glassmind/grow-fast-20261010/keep-going-candidate/rustbench-dev/`.
Facts raw before/after train/dev predictions: `WORK/facts-run/tmp/`.
MathQA and PyArrow: `WORK/quantities-run/tmp/{mathqa,quantities-deps}`.
Do not delete retained data/models under `~/.cache/glassmind/{data,models}`.
The old improvement loop remains STOPPED; preserve its dirty checkout/state.

## Immediate next actions

1. PR11 merged after all coding/facts gates and green exact-head CI. No pending
   publishing or evaluation for that batch; do not re-run its full765 jobs.
2. Publish corrected linearwords source687f275 plus complete evidence; required
   CI and exact-head merge. All local gates are complete. Scoreboard/README22/4475.
3. Next coding investigation:42/42 RustBench dev explanation requests refuse;
   inspect the existing typed purpose/body evidence path and permitted dev
   request/source-only examples to choose a falsifiable explanation hypothesis.
   Do not inspect benchmark hidden replies/tests/solutions or change scorers.
4. Broader gaps remain MathQA natural language (22/4475), finite validation-seeded
   facts coverage, and genuine reading semantics beyond byte likelihood.
5. Goal stays active until user stops. Current owned modelworkers all stopped;
   old dirty improvement loop remains stopped. Preserve artifacts and exact refs.
