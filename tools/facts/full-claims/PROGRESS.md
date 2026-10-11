# Full independent claims release: native complete, independent audit running

Current19:38UTC: acquisition/projection/rawreference/native import COMPLETE.
Native89022 exit0,3052.182seconds,371540KiBpeakprocessRSS,
142749644072sampledpeakaddedrunbytes. Original resource gates passed.
Native database62772387840bytes, now0444;74445176entities,74983525aliases,
362302267distinctedges,364044919statements,46448050qualified. Integrity and
foreign references checked3050.936s; committed3051.003s. No restart needed.

Independent audit77590 RUNNING,scopeglassmind-full-claims-audit,
supervisor1230237,child1230238.8GiB/no swap,3396seconds remaining from original
3600second source-audit budget after203.173srawreference; outertimeout3406s.
TMPDIR/SQLITE_TMPDIR=RUN/tmp,supervise.py384GiBaddedrun cap. All86file sample
comparisons finished by2.426s; aggregate counts,distinct/union checks andhashremain.
NEXT: after audit-resources.json passes, runwitnesses.py via supervisor with
3600seconds,8GiB/no swap andon-disktmp. Do not claim newgraphQA or promote default
assets before allsource/storage/probe gates and a separate QA experiment.

Projection29161 COMPLETE exit0:789435443rawrows,364044919statements,46448050
qualified,1092134757canonical retainedIDs. 25821511680bytes SHA
7fd86d572ba1f0b91de7cb4165543162195247b164d69363571bbf67c7128622.
2711.674s supervised,556268KiBpeakRSS,70529619240sampledpeakaddedbytes.
All resourcegates passed; claims.sqlite0444. Raw8009samples allagree withfinal
projectionreport, including3647eligible. Database audit remainsrequired.

Preparation18606 COMPLETE exit0. Pinned native binaryb0a86c03..., fullsource
manifestSHA782d12c471f9af31dfc86878ee3cddeab9609332702c33f0f431dc7a1251dba2
and originalrawreference before native start. See native-invocation.json.
Native89022 COMPLETE; former supervisor1022832,child1022833 terminal.
Original caps21600s/192GiBoutput/384GiBaddedrun,8GiBno swap,ownedprocessgroup,
TMPDIR/SQLITE_TMPDIR=RUN/tmp. Input projection andnative names are immutable.
NEXT: monitor existing job; onlyafter native-resources passed makeoutputreadonly.
Run independentaudit with3396sremaining budget via supervise.py thenwitnesses
under3600s. Preservefailures andgates; no overwrite/restart/re-fetch.

## Prior execution record (completed statuses above supersede old observations)

Root-owned worktree WORK/full-claims, branchcontinuous/full-claims. No model
workers, new authentication or model calls. Knownworker tokens unchanged201926299.
WORK=/home/gsc/.cache/glassmind/continuous-20261010.
RUN=WORK/coordinator/full-claims. Fixed plan84803eb before acquisition. Source
importer/projection bcc796c; validation plan59950f7; source audit/probes following.

RUN/fetch.py session64342, scopeglassmind-full-claims-fetch: COMPLETE exit0,7200s/96GiB
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
After native succeeds, makeoutputreadonly, runaudit.py under3396s/8GiB/supervisor,
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

Before prepare-native: after fetch session64342 exits successfully, run
RUN/audit-acquisition.py RUN under180s/8GiB. It pins the manifest, rehashes all86
rawfiles, audits85new-download ledger entries and makes only newrawfilesreadonly.
prepare-native.py now requires its acquisition-report.json in addition to the
completed projection resource report. Full raw acquisition gate is unchanged.

ACQUISITION COMPLETE:64342 exited0; all85new shards44104948009bytes downloaded
without retries in1495.359s (through final verified file), below96GiB/7200s caps.
Including reusedfirst,86files44633986614bytes. Independent all-file rehash21.354s
passed; audit78497 COMPLETE, allpublisherhashes match, newrawfiles nowreadonly.
Reports acquisition-report.json/fetch.jsonl; source inventorySHAe29894e9... .
Projection29161 stillRUNNING, last44files at1321s. No fullnative output yet.
prepare-native additionally pins the tested CLI SHA b0a86c03... before copying;
other track builds must not silently replace the previously verified importer.

Independent raw reference COMPLETE: session2492 exit0,203.173s,695308KiB peakRSS.
86files/789435443rawrows;8009fixed rawsamples,3647eligible. RawreferenceSHA
3fcd1c82d6fc1563121dc33e0f81b81481ffd773aa89f3e1003e4c56c429e15c.
reference-report.json retained. Samples reconstructed from full rawqualifiers
without project.py/database input. Rawreference/report hashes pinned beforeimport
by prepare-native.py. Final audit compares bothDBs to this reference; original
3600s combined audit gate leaves3396s. No sample/gate changes or QA inputs.
Intermediate independent comparison: all7817samples in first84projectedfiles
agree; final86/fullDB comparisons remain required. Projectionstillrunning.
