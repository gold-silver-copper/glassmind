# Native public pilot ingestion: implementation validated, full import next

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

Full import is next using externalrun/run-import.py under8GiB/no-swap/3600seconds.
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
