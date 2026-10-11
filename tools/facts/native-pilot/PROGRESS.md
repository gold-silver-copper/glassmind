# Native public pilot ingestion: full import running; compatibility passed

Format2 importer implemented in crates/facts/src/disk/pilot.rs with CLI command
import-wikidata-pilot. It pins the source database hash, attaches read-only, validates
numeric rows/source-file mappings, rebuilds Rust keys, retains statement witnesses
and alias provenance, embeds the portable manifest, and commits only after integrity
and foreign-key checks. Format1 JSON outputs retain the same serialized shape.

33 facts tests passed including3new pilot test groups. Workspacefmt/clippy/release
andstrict4/4smoke passed. Firstcompile rejected u64 SQLite conversions; corrected
with checked i64 conversions. Clippy flagged read-buffer slicing; corrected with
checked access. Logs focused.log/focused-fixed.log/workspace.log in externalrun.
No resource gate or selection changed. No model worker, new downloads or QA input.

Full import RUNNING using externalrun/run-import.py under8GiB/no-swap/3600seconds.
Session87682; scopeglassmind-wikidata-native-import.scope; supervisorPID749960,
importPID749961. Frozenimportsourceb44df61 andcandidateSHA
601e16d776c129a65998f6f650b5d5b5b189ece95af8743bd6fc075ebdfb7620.
Sourcehashverified4.995s, rowsvalidated20.683s, entitiesloaded45.365s.
Currentlyrebuildingnames; monitor existingjob/log, do not startanotherimport.
The runningbinaryusesabsolute outputpaths andis unaffectedbythe subsequentpathfix.
WORK=/home/gsc/.cache/glassmind/continuous-20261010; externalrun=WORK/coordinator/wikidata-native.
The runner reserves native.sqlite,tmp,andcandidate, copies the tested binary, and
tracks run growth including unlinked SQLite sort files through /proc/PID/fd.
Do not rerun in place: preserve anypartial. Fixedlimits remain24GiBoutput/32GiBaddedrun.

After import passes, run externalrun/audit-native.py with retainedPyArrowvenv,
8GiBscope,privateondiskTMPDIR/SQLITE_TMPDIRand600stimeout. It checks954fixedrawrow
positions, fullstatementwitnesses, aliasprovenance/nativekeys viaa smallversion1
rawsampleview, independententity/edgededup counts, rank/qualifiercounts, queryplans
andsource-namealiasresolution. It uses a freshsmall launcher forqueryRSS. NotQAaccuracy.
Then run boundedraw-view parity and existingformat1 parity; requiredintegration/main
regressions beforepublishing. Canonicalprerequisite already passed; do not rerun it.

OriginalPythonpilot10,367,000,576bytes SHAc3f1197705f131e74f8f7b794d3f08eaef7d3b48e8a312170185e6728bff1313
remainsimmutable. SixrawParquetfiles, manifestandprioraudit remain retained. Keep
fullqualifiers/references inrawsource; thisgraph includesqualified/historical claims.

## Subsequent output-path fix and compatibility

Review found that SQLite URI mode could reinterpret a relative filename beginning
file:. Canonicalize newlyreservedoutput paths andreadpaths forbothimportformats.
Firstregression caughtthatfixingonlythewriter stillleftthereaderpath affected;
output-path-tests.log preserves thatfailure. Finaltest coversbothformatCLIimports
andproves anexistingURItargetfile stays byte-identical. Final33factstests andclippy
pass (output-path-fixed.log). Allworkspacefmt/clippy/release and4/4smoke pass again
(workspace-final.log). Compatibilitymatchesall19481TRAIN+2821DEV ordinary/set
streams againstretainedv1outputs. Mainmixed28/28 eachplain/intent unchanged.
No additionalQA inference againstthenewpublicdatabase hasrun.

Current facts binary in coordinator/disk-facts/target SHA
8a284fd1f4c377f61d97bc78d30a80abcb116a3e15d794c93b6aa0a8e6af5492;
mainSHA9b80e692728b204ba2dd118a663fec3b0818a4fb4768a5c39a1406b1b1eb26c9.
Runningimportcandidate isseparatelycopiedandunchanged. The pathfix doesnotchange
transformation/querysemantics or require repeatingthefrozenlargeimport.
Sessions99690(initialcompilefailure),86376(testpass/clippyfailure),37363(initial
workspacepass),36372(pathregressionfailure),58672(finaltests/clippypass),16569(final
workspace+compatibilitypass) areCOMPLETE. Onlyimport87682 remainsactive.

Next: monitorimport87682. Ifpass, runprepared audit-native.py under600s8GiBscope,
withSQLITE_TMPDIR/TMPDIR=externalrun/tmp,usingwikidata-snapshot/venv/bin/python.
ThencomparethegeneratedrawsampleJSON/v1SQLiteview withdisk_parity exampleusing
source-name-queries.txt; rebuildtheexampleunderrequiredcargoscopeifneeded.
InspectactualpublicqueryprocessRSS fromquery-resource-report.json (freshlauncher).
Do notclaim broadQAaccuracy. Preservefailures anddonotrelax3600s/24GiB/32GiBgates.
Rootintegration/publishingawaitsthepublicingestionandauditresults.
