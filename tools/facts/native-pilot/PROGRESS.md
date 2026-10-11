# Native public pilot ingestion passed; central integration next

Worktree implementation and reports are complete. No model worker ran.
The full import, source audit, positive query probes and raw-view parity passed.
Production source is b44df61 plus literal-path correction bba402d; subsequent
commits add validation helpers and reports. See PARENT.md for exact evidence.

WORK=/home/gsc/.cache/glassmind/continuous-20261010.
Run: WORK/coordinator/wikidata-native. Immutable native.sqlite is12,306,595,840bytes,
SHA256 f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b.
59,853,388entities,74,983,525aliases,5,659,191edges,5,677,515statementwitnesses.
816.399seconds,277,036KiB processRSS,20,852,143,958sampled peakrunbytes: allfrozen
gates pass. Source audit69.283seconds;954rawrows, allindependentcounts match.
110positivequeryprobes pass (37ordinary answers,59sets,13truncated,largest1756).
853raw-view questions matchboth JSON/nativeordinaryandset outputs.

All jobs COMPLETE: import87682, audit47772, accountingbuild33676, witnessbuild28974,
witness/raw-view directcalls. Scopesinactive; no owned modelworkers/authmounts.
Retained candidate/import source b44df61; final-candidate/reader bba402d. Source
pilot remainsread-only; originalrawfiles andallfailedhelper/testlogs preserved.

Next integrate the full branch since91d702b (including58a172c and08fb892 plans/audit,
whichare notyetinroot), then runcentralworkspace checks and compare source/binary
provenance. Worktreecompatibility alreadyprovedall22302knownquestions bothmodes
andmain28/28eachplain/intent. Do notrerunlargeimport orauditwithoutnewdata/code
reason. Publishonly reviewedhead withgreenCI. Keep scores unchanged: theseare
storage/query checks, notQAaccuracy. Then freeze a separate strictjoint QA
coverage experiment onapprovedCAL/DEV using the fixed publicgraph andmodel.
Do notfetchtheother85claimshards withouta separateboundedplan.
