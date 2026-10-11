# Preregistered single normalized search candidate

Hypothesis: local perceptron scores rank actions within states, but their arbitrary
state offsets make raw cumulative scores unsuitable for comparing prefixes.
Cumulative local log-softmax over LEGAL transitions removes those offsets and
improves prefix ranking. This is a deterministic objective, NOT calibrated probability.

Exactly one candidate: width4, temperature1.0, cumulative f64
sum_t [(score(a_t)-max_legal) - ln(sum_legal exp(score(a)-max_legal))].
Convert existing f32 scores to f64 before subtraction. Check every input score
(including illegal actions), local normalized score, and cumulative total finite.
Retain parent order then ascending action IDs; strict greater comparison retains
stable ties. No state merging. Unchanged tags/features/legal states/model weights.
Exactly2*n synchronous steps, n<=128; maximum64 labels/129 actions. At most4
retained parents,4 descriptors,4 temporary successors and one action-score vector;
<=132096 scored actions and1024 chosen clones per sentence. Reject widths except
1/4; width1 delegates original greedy solely for compatibility. Same16KiB input,
128KiB JSONL row,128MiB model limits. No default path/tokenizer/model changes.
New explicit --normalized-beam mode, optional API and omitted default search_mode.

Freeze these24 authored cases before code edits. Roles, word order, questions,
negation and punctuation have explicit core-edge expectations. Preserve all
baseline-correct core edges; these are development probes, not independent evidence.
Parent supplies its independent probes; do not read its contrast files.

Baseline full2001: LAS19423,UAS20540/25148 including punctuation,zero refusals.
Previous unnormalized width4 candidate is REJECTED:17879LAS/19051UAS; not rerun.
Acceptance fullLAS>=19673,UAS>=20540,all2001 valid trees,no new refusals;
authored and independent baseline-correct core edges retained. Report68TRAIN
form duplicates/137edges separately, never exclude. Default forms059e5ca... and
raw4c7b1b65... byte-identical; model47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6 fixed.

Freeze source/binary/model/probes/adapter/unchanged independent scorer BEFORE
first100 then full2001 ONCE. No DEV tuning, further candidate or postDEV search/
training edits. Record all25148 edge transitions and2001 sentence transitions.
Reuse supplied baseline predictions; candidate default forms/raw once each.
Run reading tests, raw/contracts/model serialization/invalid input checks,
fmt/clippy/release in required capped cargo scope. Record time/RSS and hashes.
On failure freeze exact patch/source/binary/results, verify archive, then restore
runtime files. Failure rejects this second search objective: future work requires
training or feature change, not a third score tweak. One bounded20minute batch,
no agents/LLM/downloads/retraining/commits. Parent integration only if accepted.
