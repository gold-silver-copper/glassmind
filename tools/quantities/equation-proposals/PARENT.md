# Rejected equation-program classifier: independent review

Do not integrate this candidate. The fixed program-bank classifier loses to its
lexical comparator and fails its selection and independent semantic gates. The
existing quantities solver and main conversation behavior remain unchanged.

The terminal worker submission was archived before inspection:160 files,
70,053,395 bytes, independently SHA256-verified under
`WORK/equationproposals-run/terminal-originals` (WORK is
`/home/gsc/.cache/glassmind/continuous-20261010`). Its runtime patch is retained
as rejected evidence. No production source or dependency changed in this batch.

Parent independently verified58 original manifest paths, reconstructed the
23912FIT/5925calibration split from normalized TRAIN question hashes, confirmed
9521/2346 disjoint groups, and reconstructed the512 retained classes from the
4590 cleaned FIT rows/1565 generic programs. Source review confirms FIT-only IDF
and centroid construction and no DEV formulas. No raw question or example-ID
fields appear in the model schema. This audit is not a proof of every possible
leakage channel; source revision, licenses and exact paths are in HANDOFF.

Independent rescoring of all4475 saved predictions reproduces:

| Proposal method | Numeric matches | Refused | Wrong | Unscorable |
|---|---:|---:|---:|---:|
| Candidate | 499 | 1878 | 1179 | 919 |
| Unrestricted lexical | 663 | 1747 | 1132 | 933 |
| Restricted lexical | 518 | 1850 | 1183 | 924 |

Unscorable proposals count as wrong conservatively. This is the separately
named proposal-numeric-label metric, not verified English answers. All5925
calibration rows were retained. Independent tied-score threshold recomputation
confirms that none meets90% precision with>=30 selections. Candidate's best
eligible precision is111/211 (52.61%); it is descriptive, not a replacement
threshold. Selection remains disabled. Raw requires at least688 matches to
beat the stronger baseline by25; candidate has499. Both declared gates fail.
A native replay of the fixed200 sample is byte-identical to saved predictions.

Parent24 semantic questions were frozen before candidate inspection. Candidate
gets2correct,15refused,7wrong,0selected; ordinary solve gets2correct,22refused,
0wrong on the same questions. The two candidate successes are percent problems.
Its wrong proposals include13/8 for13 plus8 and13 for7 times6. The >=12/24 gate
fails. All proposed results explicitly say semantics unverified. Disabled
selection makes the selected-error check vacuous, not evidence of safe coverage.

Exact native execution passes8/8 binding/value contracts,7/7 malformed-program
contracts and9/9 additional numeric-spelling/input-bound contracts frozen before
inspection. Eight model shape/weight/reference/size checks reject gracefully.
A32,000,001-byte padded model loads: the implementation enforces32MiB rather than
a strict decimal32MB. The actual model is1,151,545bytes. This discrepancy remains
recorded; no exception or post-hoc acceptance is granted. The initial review
adapter skipped nested freeze entries; a separate explicit integrity check then
verified all originals unchanged, and the replay adapter was corrected. Original
adapter and results remain in parent/results.

Default-solve saved output pairs are byte-identical for200 sample,4475 DEV and
3116 synthetic rows. Full ordinary MathQA remains29right/4443refused/3wrong.
Parent did not rerun all native inference or claim workspace integration for
this rejected candidate. Worker16tests/fmt/clippy/release evidence is retained.

TRAIN-only representation follow-up: among1182 exact-label-agreeing calibration
annotations, only348 have a program in the512-class bank; another133 occur in
unrestricted FIT, and701 are unseen exact program structures. This is annotation
coverage, not an accuracy ceiling: algebraically equivalent programs can differ.
It motivates evaluating compositional operation/argument prediction rather than
another centroid threshold or class-count tweak. No DEV formula was inspected.

Completed worker usage:1,651,494 input tokens (including1,577,216 cached) plus
29,054 output =1,680,548; known cumulative completed worker tokens158,005,855.
Reasoning output is a subset of output, and cached input is not counted twice.
No new data, dependencies, API-key calls or model downloads. Runtime source and
model hashes, commands, immutable reports and all limitations remain in HANDOFF
and parent/results. The external frozen candidate remains available for replay.
