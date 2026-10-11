# Facts selection diagnosis and rejected expansion

Parent diagnostic, 2026-10-11; no runtime change and no new inference. Original
inputs, predictions, graph and selection were hash-verified. Artifacts are copied
unchanged from `WORK/coordinator/facts-selection-diagnosis`, where
`WORK=/home/gsc/.cache/glassmind/continuous-20261010`.

The fixed eligible FIT subset is the prefix 90% of 19,481 TRAIN rows excluding
every tenth row: 15,778 rows. These are in-sample diagnostics, not unseen results.

| Direction | Rows | Annotated anchor in index | Correct anchor linked | Answered | Answer match | Strict anchor/relation/answer match |
|---|---:|---:|---:|---:|---:|---:|
| Forward P | 12,066 | 733 | 679 | 1,109 | 752 | 505 |
| Inverse R | 3,712 | 1,934 | 1,669 | 167 | 1 | 1 |

Among wrong forward-anchor answers, 588 have an annotated anchor absent from the
index; only two have it present. The index's original 1,000 requested subjects
contain 559 forward-only anchors, 411 inverse-only anchors and 30 used both ways.
The FIT pool contains 11,892 forward-only, 2,187 inverse-only and 30 shared anchors.
Most forward proper-name subjects occur too sparsely for this small selection
to provide broad coverage. Another alias filter cannot create missing claims.

The separately frozen forward expansion selected 1,000 additional, previously
unselected forward FIT anchors by frequency and SHA256 tie-break, without using
answer IDs or question text. Its selection was hashed before joining CAL.
Only **three** of the fixed 1,949 CAL rows have a newly covered forward anchor,
below the prospective prerequisite of ten. **Rejected before acquisition**;
there is no expanded graph, candidate inference or DEV result. This is a bound
on the proposed forward-coverage mechanism, not on incidental inverse effects.
Do not fetch these entities or lower the gate after seeing the result.

`report.json` contains the full FIT counters and permitted TRAIN examples.
`forward-feasibility.json` records the fixed CAL prerequisite result.
`artifacts.json` pins every original script/plan/result. The first diagnostic
attempt incorrectly treated the plain-text question file as JSONL and stopped
before counting; its original script remains preserved. The corrected script
verified full question order against TRAIN before reporting.

Next approach: independently sourced graph coverage. The separate
`../wikidata-snapshot/PLAN.md` selects public snapshot files by path and language,
without evaluation entity IDs, and measures storage feasibility before QA gains.
