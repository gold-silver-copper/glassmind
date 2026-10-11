# Prospective forward-anchor enrichment feasibility gate

The verified FIT diagnostic finds 505 grounded forward answers versus one
inverse answer. Inverse-only anchors consume 411 of the 1000 selected slots;
most wrong forward links (588/590) substitute another entity when the annotated
anchor is absent. Test a different selection, before downloading or inference.

Freeze exactly1000 additional, previously unselected forward (P-relation) TRAIN
anchor IDs, using the same FIT prefix90% / i%10!=0 rule. Rank by forward FIT
frequency descending, then SHA256(QID) ascending; preserve every original1000
selected ID, including its missing redirect. No answer IDs or questions select
entities; no new claims/aliases are constructed from annotations.

Before any acquisition, count membership in the fixed every-tenth TRAIN CAL.
A subsequent candidate must gain at least10 joint-correct CAL answers with zero
new wrong claims and retain all baseline-joint-correct answers. If fewer than10
CAL rows have one of the new forward anchors, reject this forward-coverage path
as unable to supply its intended mechanism's minimum; do not fetch or lower the
gate. This is a mechanism-specific necessary bound, not a bound on incidental
inverse changes from fetched outgoing claims. Those would be a different
hypothesis and cannot rescue this one. Do not inspect DEV for this decision.

If feasible, retain the frozen selection for one bounded acquisition experiment;
CAL membership alone does not establish aliases, correct claims or model accuracy.
These TRAIN diagnostics are optimistic because the old model saw TRAIN. No
unseen-calibration or independent test claim. No runtime or score-series change.
