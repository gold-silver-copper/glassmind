# TRAIN-only dialogue diagnosis

Three span-proposal hypotheses have failed; do not change their thresholds or
fit another span extractor. Diagnose the approved model's remaining errors across
complete TRAIN conversations whose services are all in its three-service schema.
Use the existing selected TRAIN file and retain every turn of every qualifying
conversation. First run the fixed first-ten sample, then the complete qualifying
TRAIN subset. This is in-sample diagnosis, not development or held-out accuracy.

Freeze the native binary, approved bundle, source TRAIN and unchanged evaluator
before inference. Native input contains speaker and utterance only. Keep the
official scorer unchanged. Report routing, intent and slot error overlap, plus
missing, spurious and incorrect slot values by categorical/text role. For text
errors, distinguish absent current gold values, persisted incorrect predictions,
and new gold updates. Diagnostic annotations never enter runtime inputs.

No candidate, model training, threshold selection or runtime edits in this step.
Choose a subsequent hypothesis only after these diagnostics; any new model needs
its own prospective fit/calibration and development gates.
