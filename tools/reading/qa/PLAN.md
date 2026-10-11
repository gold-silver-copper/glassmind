# Frozen QA experiment (before edits / dev evaluation)

Hypothesis: a hashed linear structured span ranker trained on public TRAIN improves
raw normalized EM by >=2 percentage points and F1 by >=3 points over a fixed
lexical-overlap/extraction baseline. Acceptance additionally requires >=1% full-dev
accepted coverage and >=40% exact precision. Calibration selects maximal coverage
among score-margin thresholds with >=30 accepts and >=50% exact precision on TRAIN
calibration only. No qualifying threshold means all-refusal and candidate rejection.

Selection: SHA256 of concatenated article contexts (newline separator), first64
articles fit, next16 calibration; question SHA256(ID) ordering within selected
articles, caps4000/1000. Fit and calibration article-disjoint. Five deterministic
passes, averaged passive-aggressive pairwise rank updates, 65536 hashed features,
max10-token spans, top3 lexical-overlap sentences. Threshold uses margin between
best and next distinct answer string; a margin is not calibrated confidence.
Fixed sample200 dev by SHA256(questionID), then full10570. No dev tuning.
Baseline uses sentence overlap, nearby question-word overlap, span length and
capitalization/number shapes with fixed question-type priors. Same proposer.
Runtime input strictly context+question; no title/ID/annotations. Exact UTF8 byte
slice offsets, sentence evidence, predicted/refused status. Blank accepted output
for refusals; raw proposal reported separately. Source occurrence is not proof.
Existing byte and parsing entrypoints remain byte-for-byte unchanged.

Code MIT. Dataset-derived weights, selections and reports CC-BY-SA-4.0.
Dataset source pinned eee5fdbf62f8613a7812b03419e6b29617b74fd1,
local ../readingqa-source; no download. No held-out or private files.
