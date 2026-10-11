# Parent decision: rejected as submitted

The candidate passes its worker-authored 490 checks in each model mode but loses
previously correct learned replies on the broader CLINC validation audit. Main
talk with a reset before each of 3,100 inputs yields 1,303 correct accepted learned
proposals on the baseline and 1,302 on the candidate: three losses and two gains.
The losses are ordinary wellbeing questions. All 23 changed rows were audited;
there are no newly wrong accepted learned proposals. Typed social/memory replies
are not graded by this diagnostic, so these counts are not classifier accuracy
or complete conversation correctness.

The parent audit binary is 4dac87aa5361a8d3360ba3025b28385b1c87d3c40f24159d1bcbfc816ba32d3b.
Final worker binary 900973a03613904ec656ccaa92187a14243bb1ac7e534570151fdfd38182a754
has identical bytes and metadata in all 30 allocated ELF sections except the
excluded GNU build-id note. Final source hashes match the observed source.
The parent independently checked that equivalence; whole executable hashes
are different. Worker HANDOFF explains the test-only rebuild. No integrated
runtime or full accepted-candidate verification is claimed.

The blanket refusal of social proposals outside the finite grammar explains the
lost replies. The next experiment separates explicit story-state authority from
ordinary learned fallback, retaining old safety gates and all 1,303 previously
correct proposals. It must not whitelist validation sentences.

The original submission is preserved at
`/home/gsc/.cache/glassmind/continuous-20261010/socialrouting-run/submitted`:
745 archive files and all 740 worker artifact entries verified. Large story
streams and session files stay there; this directory contains a byte-identical
selection of small original reports and adapters, plus this parent review.
Thus `artifacts.json` documents the external full archive, not only these copies.
The two original patch hashes differ solely in Git's i/ versus c/ old-path
prefixes; the hunks and resulting source are identical. Original HANDOFF's
local acceptance statement is historical, superseded by this parent rejection.

[Verification](parent/verification.json), [all changed rows](parent/changed-rows.json)
and the unchanged [audit adapter](parent/intent-routing.py) preserve the evidence.
Data: existing CLINC150 public VAL, 3,100 rows, CC-BY-3.0, pinned revision
828f8093932c8fe6ca7936c3d2e52903b1c523de. Runtime receives text only; labels are
used offline. Data/model/binary hashes are in the reports. No held-out material,
retraining, changed threshold or scorer was used.
