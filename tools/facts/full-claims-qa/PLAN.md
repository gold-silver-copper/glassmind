# Full independent claims snapshot: prospective QA calibration gate

Frozen before any QA question is inferred against the complete native graph.
Storage/source gates are complete; this experiment tests answering coverage with
unchanged language/linking behavior. The previous single-shard CAL result remains
rejected:2right/1940refused/7wrong versus TRAIN999baseline2/1863/84, seven newly
wrong and two lost correct. This experiment changes only independently acquired
claim coverage, not the acceptance criterion or existing reference annotations.

Inputs, immutable and rehashed before inference:
- Full graph:62772387840bytes, SHA256
  fc04ca4ac6f30468f619395a310181e23ebb45a44f776b81bb74178b357f0172.
  All86permutans/wikidata-claims files at39e6837bef8b6f982fbf23acd19134e882400db7;
  CC0-1.0 Wikidata20260928, scholarly works excluded. All en/mul names inherited
  from the audited native pilot. No benchmark-selected entity acquisition.
- Existing relation model SHA256
  db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6.
- Integrated CLI source7eb3de8, binary SHA256
  71ae5f437aed6f11db336662a51cee8c663770633ec23553194a53fe937cdfba.
  Copy/pin before inference. No model fit, thresholds, aliases or query-rule edits.
- Approved SimpleQuestions TRAIN19481rows SHA256
  25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b,
  askplatypus/wikidata-simplequestions revdf69267f1db4f85a71325e7e8c06e192bcaf0b0e,
  CC BY3.0. Existing normalized TRAIN questions SHA256
  d3cd1c39d916195757a304ce254a19e5f3b6b6b7fa19f6356cddbf9fff675eaf.
- Existing TRAIN999baseline fullTRAIN predictions SHA256
  eb09e5fc44e699e2ab08b94a44409ab8ad5a623224bd7340372c4326bcb0aee8.
  Original graph JSON SHA256
  142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791.

Use exactly the existing1949-row every-tenth TRAIN CAL (indices0,10,20,...), after
verifying full TRAIN question order. Runtime input is question text plus graph/model
only. Freeze full predictions and hash before offline reference joins. Use unchanged
joint-audit.audit exact annotated entity,directedrelation,soleanswer metric. Report
answer-only metric, correct links, allnine pairedstatus transitions, every newly
wrong/lostright and refusals. Keep all1949rows and exact IDs, with no post-result
redirect rewriting or annotation reinterpretation. CAL is reused development data;
no held-out access or independent model generalization claim.

Promotion prerequisite remains at least10net strictjoint correct CAL gains over
TRAIN999baseline, zero oldright/refused->wrong. Record allcorrect losses. A failure
leaves this an explicit storage artifact; no default promotion, DEV or tuned retry.
Only ifCALpasses run all2821approvedDEV unchanged against TRAIN999DEVbaseline,
requiring>=10netstrictjointgains and zero newlywrong fromright/refused there too.
Any eventual default integration is a separate reviewed task, not this gate itself.

Regardless ofCALoutcome, offline annotations may diagnose anchorpresence, link
correctness and annotated-relation knowledge (missing, uniquegold, nonunique with
gold, goldabsent). These remain oracle diagnostics; they cannot supply runtime
features, claims, further source selection or corrections. On failure choose a
new representation or language/data hypothesis using these scoped observations.

Finite600second complete run,120second inference,8GiB/no swap, private on-disk
TMPDIR andSQLITE_TMPDIR,256MiB addedrun excluding existing input assets. One run,
no downloads or model calls. Preserve any failure and partialoutputs. Parent pins
script and inputhashes before launch. Use external process supervision for the
whole time/storage cap. No fullsource rebuild or duplicate storage checks.
