# Word sequence joining: candidate validation running

Root-owned WORK/word-join, continuous/word-join at7282d2e. No modelworker/auth/API.
WORK=/home/gsc/.cache/glassmind/continuous-20261010; RUN=WORK/coordinator/word-join.
Root runtime remains PR49; only this worktree changes implementation.rs (5lines).
Separate full-claims acquisition/projection jobs remain active; do not touch them.

Frozen plan55a04d9; nativecases/runner743530a before runtime edit; source7282d2e.
Baseline rootmain7473e886... copied intoRUN/baseline before building. All existing
approved20source/60TRAIN files hashverified against originalmanifest. No hidden
content or expected patches. Runtime sees signature only, retains old candidates
and adds word order/or reversed order with single-space joining. No grammarchange.
Existing32candidate/90s budgets, exactly1survivor/nativeverification unchanged.

Baseline10authoredcases passed expected behavior,6.766s: newnormalize/reverse
bothrefused; oldchars/compact/compact-reverse accepted; newweakambiguity accepted
byoldfamily butcandidate mustrefuse; remaining4expectedrefusals unchanged.
Baseline31583 COMPLETE. Outputs RUN/baseline-native, baseline-native.log.

Build/test60530 RUNNING, scopeglassmind-word-join-build, timeout900s/8GiB/no swap;
outer769910, cargo769931, tests770827 lastlive. CARGO_TARGET_DIR=/home/gsc/code/
glassmind/target; TMPDIR=RUN/tmp. Runs all world-rust release tests then builds main.
Several inherited exhaustive native contracts take over60s; do not treat that
notice as timeout/failure. Existing run is supervised. WatchRUN/build.log.

NEXT after build passes: copy root target/release/glassmind toRUN/candidate, hash
and freeze source. Runtools/coding-word-join/run.py candidate, old-native, then
train-sample (6requests) andtrain-full (60) into separate newRUN directories.
Gates: new2behaviors correct, ambiguity/unsupported refuse; alloriginalaccepted
nativecases preserved; TRAIN>=5accepted with3oldsourceoutcomes byteidentical.
Runner verifies source/test preservation; independently run nativechecks on all
accepted finalTRAIN results using unchanged existing verify.py adapted paths.
Then tools/check.sh withcargojobs1/samecache. Only afterthese pass, full765DEVvia
unchangedtools/rustbench-dev.sh; pinbinary/runner and usefull-gate.json baseline
f76e6f43...: >=220passes/all218oldpasses/0failed, no held-out. Benchmarktaskoverlap
withTRAIN must be explicit. Do not expandparserorchangegates afterseeingresults.
