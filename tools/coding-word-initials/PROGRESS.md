# Word-initial compositions: full DEV running

Root-owned WORK/word-initials, branchcontinuous/word-initials.
WORK=/home/gsc/.cache/glassmind/continuous-20261010.
RUN=WORK/coordinator/word-initials. No workers/models/teacher/APIcalls.
Knownworker usage unchanged201926299tokens; parent/interruptedexcluded.

Plan53ce67b frozenbeforeedits; runtime23e7525 adds18lines ofsignature-only
&str->String candidates.12firstcharacter/filter/case/ordercompositions plus6old.
32candidate90secondbudgets,uniqueacceptance,targetexecution/livecheck/rollback
unchanged. No grammar,test,dependency or runtimeEnglish/doc selection.
CandidateSHAefb48471993e9fb63181a7f171ffef428ef8938e24f29af20f5192a59d5b101b.
Rootcentral target/release/glassmind currently has thiscandidate; root source
hasnotintegrated it yet. Priorb04binary preserved atRUN/baseline andword-join/candidate.

AllpreDEVgates passed; pre-dev.json pinsreports andchecks source/binary identity.
23authorednativechecks:12newimplementations,5oldimplementations,6refusals.
Weakcasehas4distinct passingcandidates andunchangedsource. Oldwordjoin10 and
oldnative20 sourceoutcomes byteidentical. Independent18cases andsame18withunset
target allpass; extra10grammar/declarationcases pass. Existing76checks includes
those18repeatedenvironmentcases.59Rustworldtests pass. Workspacefmt/clippyall
targets/release/strict4smoke pass. All17authoredacceptedoutputs independently
recompiled; actual failingbaseline,targetexecution,uniquesurvivor/livechecks verified.

TRAINsample9:6accepted. Full60:6accepted vs5, all5prior successful sourceoutputs
byteidentical. Onegaininitials/terse fromsamepublicTRAIN; no unseen-taskclaim.
All6outputs independently recompiled andcompleteverifierevidence verified.
43retainedTRAINsourcefiles hashverified beforeuse; no newdata or hiddencontents.
RustBenchf79e71a2a26fdaf15f9bc55225a038b61e66fafb, clean.

Sessions COMPLETE:baseline54315,build2100,native/TRAIN52385,independent47453,
workspace10967,nativeverification91407,TRAINverification27475.
Full765DEV73728 RUNNING, unchangedtools/rustbench-dev.sh8GiB/no swap,3600s,
jobs1,testthreads4,ownfull765/native-target+check-cache.
NEXT: await terminalfullDEV; compareusingtools/coding-word-initials/compare-full.py
--before WORK/coordinator/word-join/full765/results.jsonl
--after RUN/full765/results.jsonl --binary RUN/candidate --out RUN/full765/comparison.json
Gate:retainall220passes,>=221passes,0fail. No runtimeedits whileevaluationruns.
Ifpass:review/recordexactresults, integrate sources centrally withhashidentity,
update README/docs/results, requiredpublishedchecks, PRandmerge exactreviewedhead.
No fullDEV result or shippingclaimyet. Preservefailedgates unchanged.
