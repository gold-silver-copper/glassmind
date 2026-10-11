# Two-sequence composition: native gate passed, regressions running

WORK=/home/gsc/.cache/glassmind/continuous-20261010; RUN=WORK/coordinator/sequence-composition.
Worktree WORK/sequence-composition, branch continuous/sequence-composition.
Frozen plan73fe0b0 and fixtures/source hashes0e07286; implementation c3790af.
Only production edit:75lines in world-rust implementation.rs, adding16signature-only
operations for exactly two integer slices/Vecs returning a matching Vec. Existing
single-argument generator and verifier source bytes exactly preserved.

Baseline22refusals; candidate16implementations/6expectedrefusals, all22pass64.884s.
Weak case16passing candidates correctly refuses unchanged. Full59Rustworldtests,
workspacefmt/alltargetclippy/release and strict4case smoke passed. Build92719 and
workspace15687 complete0. Candidate SHA256
dd6a823946637dd5daacba2f1033a1c3e1fc4421b147dabc8dacf8b16aef92bf.
All200Rust/manifest/lockinputhashes in RUN/candidate.json.

Local suite83876 RUNNING, scopeglassmind-sequence-local,2400s8GiB0swap,jobs1,
on-diskTMPDIR=RUN/tmp. Drivercheck-local.sh records separate logs. Existing89
native cases, TRAIN12sample/60full, independent16native + allacceptedTRAIN native
recompilation and evidence checks pending. No newmodelworkers orAPIkeys.

NEXT afterterminal local suite: python tools/coding-sequence-composition/pre-dev.py RUN.
FrozenfullDEV comparator requiresretain221passes +>=1gain,0failures onall765.
Launch unchangedtools/rustbench-dev.sh onlyafterpre-dev gate. No runtimechanges
whileevaluating. Do not claim fullDEV gain orshipmentbeforecomparison/integration.

All pre-DEV gates passed, report RUN/pre-dev.json. Native22(16accepted), existing89
source outcomes unchanged, full59Rusttests, workspace requiredchecks pass.
TRAINsample12accepted8; full60accepted8, all6previousacceptedsourceoutputs exactly
retained. Gains merge-sorted-unique hurried/terse, both suppliedTRAIN; notunseen.
Independent native16/TRAIN8 recompiled, targetexecution/uniquesurvivor/livecheck
and baselinefailure evidence allverified. Runtimehashes unchanged.

Initial local83876 stopped on missing historicalfreeze.py before independent cases.
Restoredoriginal exactmanifest hash in46cc2e8, didnotexecute generator orchange
cases/runner/scorer. First43oldchecks preserved; remaining suite14291 COMPLETE0.
Original errorlog retained. No runtimefix or test relaxation.

Full765DEV67238 RUNNING under unchanged tools/rustbench-dev.sh3600s8GiB0swap,
jobs1/threads4, private RUN/full765 native-target/check-cache/check-scratch/tmp.
NEXT await terminal job, then compare:
python tools/coding-sequence-composition/compare-full.py --before
WORK/coordinator/word-initials/full765/results.jsonl --after RUN/full765/results.jsonl
--binary RUN/candidate --out RUN/full765/comparison.json
Require221retained,>=222passes,0failures. If pass integrate reviewed source,
update README/scorejournal, centralrequiredchecks and exactgreen PRmerge.
No fullDEVgain or shippingclaim yet. Candidate and production unchanged.
