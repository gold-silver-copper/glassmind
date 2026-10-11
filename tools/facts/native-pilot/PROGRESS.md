# Native public pilot: validated and centrally integrated

The importer and public database passed all frozen gates. Root merge e5487ef
integrates worktree c873f4b. All 23 facts files and Cargo.lock match; central
workspace checks and strict 4/4 smoke pass. Central CLI reproduces all 853 public
source-name predictions. See PARENT.md and the central-* reports.

WORK=/home/gsc/.cache/glassmind/continuous-20261010.
Run: WORK/coordinator/wikidata-native. The native.sqlite artifact is read-only,
12,306,595,840 bytes, SHA256
f11f5f93363e037c94261187d06461f39e6f7bade9cec4674e1205073847131b.
It has 59,853,388 entities, 74,983,525 aliases, 5,659,191 distinct edges and all
5,677,515 statement witnesses. Import/index/integrity took 816.399 seconds,
277,036 KiB process RSS and 20,852,143,958 sampled peak run bytes. Limits unchanged.

Independent audit: 954 raw rows, exact independent counts, 853 public name
queries and 110 additional witness probes pass. Witness probes include 37
ordinary answers, 59 sets and 13 truncated sets; largest count 1,756. They use
synthetic prescribed relations, not a QA accuracy evaluation. Raw-view JSON/
native parity matches all 853 ordinary/set outputs. Existing 22,302 TRAIN/DEV
outputs and main 28/28 checks in both modes remain unchanged.

All experiment/build jobs complete. Import87682, audit47772, helper33676 and
28974, central build3005 are terminal; no model workers or auth mounts. Preserve
all inputs, original binaries, partial-helper logs and artifacts. Do not rerun
completed large import/audits without a new reason.

Next publish the reviewed root head after CI passes. Then freeze a separate
strict entity/relation/answer QA experiment on approved CAL/DEV, using the fixed
public graph and existing model. No QA questions have run against it yet. The
other 85 claim shards remain unfetched pending a separate bounded plan.
