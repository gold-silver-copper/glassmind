# Full independent claims release: running, not yet validated

Root-owned worktree WORK/full-claims, branchcontinuous/full-claims. No model
workers, new authentication or model calls. Knownworker tokens unchanged201926299.
WORK=/home/gsc/.cache/glassmind/continuous-20261010.
RUN=WORK/coordinator/full-claims. Fixed plan84803eb before acquisition. Source
importer/projection bcc796c; validation plan59950f7; source audit/probes following.

RUN/fetch.py session64342, scopeglassmind-full-claims-fetch: RUNNING,7200s/96GiB
received cap. New85shards plus reused first from old snapshot. Sequential public
unauthenticated source requests; every completed shard passes publisher hash/size.
RUN/project.py session29161, scopeglassmind-full-claims-projection: RUNNING,
supervisor761927, child761928 (verify live).14400s/96GiBprojection/384GiBadded-run,
8GiB/no swap; raw data may arrive concurrently, waiting counts toward same deadline.
Both scripts pinned in initial-artifacts.json. Never restart into these outputs.
Check fetch.jsonl, projection-progress.json, projection.log and scope state.

Native importer creates format3 with original names provenance and every claims
file range; inputs both hash-pinned. Copies nativealiases, fullclaimsdedup,
allstatement witnesses and omitted-row gaps; source rank/qualification preserved.
Newoutput only, schema header after integrity/FKchecks,192GiB/21600s limit.
No native full output or full QA inference has run yet. No default promotion.

Validation completed:36facts tests pass, including3multi-shard groups for names,
forward/inverse answers, duplicate/GUID witnesses, malformedprojections, ranges,
hashes and literalpaths. Newprojection500000rawrows exactly matches retainedpilot
(86536kept); initial100000exclusion-only smoke preserved separately (0kept).
Workspacefmt/clippy/release/strict4smoke pass (session18599 COMPLETE); helper-specific
clippy passes after helperchange. Sourcepublicquery helper rebuilt (56281 COMPLETE).

Existing19481TRAIN+2821DEV ordinary/set outputs byte-identical onunchangedv1graph;
main28/28 eachplain/intent. All853oldformat2publicsource-name outputs byte-identical;
110positiveprobes still37ordinaryanswers/73refusals,59sets13truncatedmax1756.
Compatibility94911 COMPLETE. Independent newwitness-scan script reproduces all110
oldcases with identicalSHAcaeccf0249d6e16c5119385562bda55ee889a0eb6934c282899a8fca5b848100
in0.458s. These are regressions, not newgraph QA. No changed thresholds/model.
Binaryfacts b0a86c031e6c83d81af3a5672391f6385b9ec7eb1205743d4d79a7c5677cded7;
main e266763d169065af71ed0c8d6f883eb3c29857fbeb8dbef99f948c0cf8c26bb0;
probe84ac1fc499c8eec88aff28ab3d598109ecd426f30fbc7fd48d8784b9d2b79f47.

NEXT after successful projection-resources.json and complete fetch: run
RUN/prepare-native.py RUN WORK/coordinator/disk-facts/target/release/glassmind-facts
under a finite8GiBscope; it pins completedprojection and copiescandidate intoRUN.
Then run RUN/supervise.py RUN native 21600 [native-invocation.json command] under
8GiB/no swap, setsid--wait,timeout21610s and RUN/tmp bothTMPDIR/SQLITE_TMPDIR.
Do not start until existingprojection completed; verify all pinned manifests.
After native succeeds, makeoutputreadonly, runaudit.py under3600s/8GiB/supervisor,
thenwitnesses.py samecap. Frozenfirsteligible rawsamplepershard bothdirections;
large-setVMerrors are reported separately, nevercorrectness credit/limitincrease.
Do not inspectQA/DEV or claimcoverage gain before separateQAplan. Review full
source/data results before integration/publication. Existing rootPR49merged.

18:02:49 UTC resource observation: both scopes active; memory.max8589934592,
memory.swap.max0; both report oom0/oom_kill0. File-cache pressure reaches cap.
Fetch ownedPIDs760445/760446; projection group761884/761926/761927/761928.
Snapshot RUN/resource-observations.jsonl; do not confuse cgroupcache withprocessRSS.

Probe scan groups identical (direction,anchor,property) lookups before storing
independent answer sets, avoiding repeated materialization of common large inverse
sets. All original per-case checks remain. Second retained-pilot smoke reproduces
same110cases/hash/counts in0.451s;59distinct lookupgroups. Updated RUN/witnesses.py;
prior version preserved as witnesses-before-grouping.py. No runtime behavior change.
