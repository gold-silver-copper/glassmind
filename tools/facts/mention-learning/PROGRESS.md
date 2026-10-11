# Learned mention-context selector: rejected on internal validation

Worktree WORK/mention-candidates, branch continuous/mention-candidates.
RUN=WORK/coordinator/mention-learning; WORK=/home/gsc/.cache/glassmind/continuous-20261010.
Frozen planf984146; implemented/pinned d9a2a2a. Fit14156 COMPLETE exit0,
2.001s supervised,244312KiBpeakprocessRSS,22179840sampledpeakbytes. All original
1200s/8GiB0swap/256MiB/model8MiB gates passed. No workers/auth/API use.

Native157357-byte IntentModel trained on19719spans:7596mention/12123other.
20epochs,char_weight0,min_df2. Complete17465FITrows separated bynormalizedquestion
SHAmod5; internalvalidation3598rows/3567groups. All candidate/native baseline links
reconstructed exactly. Features mask span text and entity IDs; fixedfunctionword
context andspan shape/position only. Two conflicting-label groups were retained.

No threshold selected:0/55fixedgrid settings meet>=50netcorrectlinkgains withzero
baselinecorrect/unlinked->wrong andno lostcorrect. Maximum gain219 comes with50
newwrong(44fromunlinked,6lostcorrect). Bestzero-new-error setting gains5, below50.
REJECTED. No newCALpredictions,DEV/fullQA,productionintegration or fittedmodelshipment.

At the maxgain diagnostic setting,41of44unlinked->wrong cases have no eligiblegold
span;3haveone. All6lostcorrect cases have a goldspan butanothercandidate outranksit.
Seven previouslywrong cases also change to a wrongnongoldspan. Partialnames/titles
and requestwords are accepted when the intendedentity is absent/ambiguous. This
indicates a need for wholemention boundaries/open-set evidence, not a lower margin.
No tunedretry or changedfeatures after this report. Originalmodel/scores/grid retained
andhashpinned; see report.json, failure-diagnosis.json andartifacts.json.

NEXT integrate research evidence, then pivot to coding. Futurefacts work needs a
materially different wholemention/absence representation and newlyfrozen design;
do not revive the failed adjacencyveto or relax prior fullgraphpromotion gates.
