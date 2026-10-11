# QA syntax v3: one frozen candidate

Base 078578b0ac350a1b3baa5af20b1a37d21d576d23. Written before implementation,
fitting, calibration or DEV. All11 baseline manifest hashes verified. Original
4000 FIT /64 articles and1000 CAL /next16 articles independently reconstructed;
four files and selection.json byte-identical. Inspected only first16 FIT rows.

Hypothesis: predicted dependency paths and candidate attachment carry information
beyond symmetric lexical overlap and relative positions. Keep all v1 features,
top3 evidence sentences, contiguous max10 QA tokens, input limits,65536 weights,
five sequential averaged passive-aggressive passes, rate cap0.1. No search.
Parse question and each distinct evidence sentence once with frozen raw_unicode_v1
and approved syntax47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6.
Align exact byte span boundaries; candidate head is first token whose parent is
outside candidate. Missing question/sentence parse (>128 tokens included) or
unaligned boundaries have explicit missing features. Features: candidate head
UPOS/deprel, question wh deprel/UPOS, candidate head parent POS and matched-question
status; path up to3 ancestors between head and up to8 exact/stem-matched content
anchors, including direction, labels, shared parent and question anchor relation;
head/governor predicate matches; question and evidence negation, auxiliary,
subject/object child-label signals, all crossed with wh or role. Stem only fixed
suffix stripping ing/ed/es/s for matching, no vocabulary/entity/predicate lists.
Unknown words remain parser predictions; no dictionary lookup or truth rules.
Cache parse and anchor relations outside candidate loop, bounded cache16384 texts.
Model v3 wraps v1 rank weights, paired to exact SHA256 validated syntax content.
Explicit companion mandatory. Existing v1 API/serialization unchanged.

CAL unchanged: maximum coverage >=50% normalized exact and>=30 accepts, tied
margins grouped. No qualifying threshold: reject before DEV. No whole-TRAIN refit.
Freeze source/model/evaluator/probes before first200 SHA256(ID), then full10570
once. Official functions imported unchanged from qa/run.py; denominator all IDs.
Accept only raw exact>=2744, rawF1>=36.5184250480, selected right>=815 and wrong<=791;
raw >=6/8 first8 positive role probes and no newly selected clear predicate/negation
mismatch relative to v1. Report raw separately from selections; all-refusal fails.
Audit every transition, excluding normalized-empty answers from substantive right.
All predictions unverified. Any failed gate archives patch/model/reports and
restores runtime. No retuning. Previous support/roles experiments rejected for
CAL failure or coverage loss and semantic failures; their gates are not relaxed.
