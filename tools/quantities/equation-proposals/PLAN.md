# Frozen experiment: equation-proposals v1

Hypothesis: a TRAIN-supervised linear TF-IDF class centroid model can improve
English-to-arithmetic proposal coverage beyond lexical nearest-neighbor retrieval.
This is finite program classification, not unseen-program synthesis or entailment.
Base c9d10dc1a1348b757be3ea96db3ee600674e155d. One fit, no hyperparameter search.

Split ALL 29837 TRAIN by SHA256 of lowercase whitespace-normalized question with
ASCII digit runs replaced by '#'; integer digest modulo 5 == 0 is calibration.
All other groups FIT. No annotation filtering before splitting. Calibration keeps
ALL rows. No DEV annotations except Problem until saved predictions are hashed;
separate scoring then reads options/correct only. No DEV formulas read.

Numbers: exact ASCII integer/decimal lexemes, optional unary sign and conventional
comma grouping, left-to-right slots; fractions retain numerator/denominator slots.
Literal UTF-8 byte spans retained. Reject uncovered digits, Unicode numerals,
number words zero..nineteen/tens/hundred/thousand/million/billion/dozen/half/quarter,
malformed punctuation, exponents and digit-bearing words. Maximum 4096 input bytes,
16 slots. No silent numeric token skipping. Conservative unsupported refusal is
allowed; this representation is frozen even if coverage is poor.
Programs: 1..6 binary add/subtract/multiply/divide steps. Typed input (0..15), prior
step, or constant references; final step is output. Checked repository Rational.
Constants exactly [0,1,2,3,4,5,6,10,12,26,52,60,100,180,360,1000,3600,0.25,3.6,1.6].
No pi or approximate conversion constants. Reject unknown/malformed/forward refs,
overflow, division by zero. Training annotations use nN/#N/const_X conventions.

FIT cleaning: retain all bounded executable annotations whose dimensionless exact
numeric result agrees with the unchanged exact-label-comparison-v3 metric. Count
all exclusions. This supervision filter is NOT an accuracy result. Unrestricted
lexical bank includes every cleaned FIT example. Candidate classes are programs
with >=3 cleaned FIT rows, capped at 512 by descending count, canonical JSON tie.
All cleaned FIT rows in those classes contribute; no easy subset sampling.

Candidate features: lowercase ASCII word runs, word unigrams and adjacent bigrams,
plus numeric-slot count; binary presence, FNV1a64 hashing modulo 8192. FIT document
frequency gives idf=ln((1+N)/(1+df))+1. L2 normalize documents, average per class,
L2 normalize centroid. Sparse linear dot products; top-score tie by class order.
Class vocabulary, IDF and centroid weights use FIT only. No raw questions/IDs in
model. Model <=32MiB, <=512 classes, 8192 features. No runtime Python/LLM/retrieval.

Lexical baseline: lowercase ASCII word unigrams (no bigrams), binary TF times
idf=ln((1+N)/(1+df))+1, cosine nearest cleaned FIT example, earliest FIT row tie.
One common FIT vocabulary/IDF for unrestricted and candidate-restricted banks.
Returns nearest program even if execution then refuses; no fallback. Candidate
also executes only its top class. Selection score for both is top cosine score.
Use stronger baseline independently for raw correct and selected correct counts.

Thresholds chosen independently on ALL calibration rows: maximize selected count
subject to >=90% exact-label precision and >=30 selected, group tied scores, count
execution refusals/unscorable as wrong if selected (only executable outputs can
be selected). No qualifying threshold means disabled and fails gate. Scores are
benchmark selection scores, never calibrated confidence or semantic verification.

DEV first200 ordered by SHA256(question), row-index tie, then all4475 exactly once.
Freeze model, source, executable and prediction hashes BEFORE scoring each stage.
Gate1 raw >=250/4475 and >=25 above stronger lexical baseline. Gate2 candidate
selected >=50 correct at >=80% precision, >=10 extra correct over strongest
selected baseline, precision >= baseline precision-.02; calibration must qualify.
Gate3 byte-identical default solve JSON for first200/full4475 and all3116 public
synthetic rows versus supplied SHA5a94dba0b893fafed21906c2150e51de7fab94c67cadefba10008b2a187e4b9f;
old full score29 correct4443 refused3 wrong. Gate4 parent independent bounds,
semantic contrasts and integration review. Any failure means rejected, not deployed.

No default solve/parser edits, main shared CLI, scorers, old fixtures, shared
manifests, downloads, agents, credentials, provider calls or commits. Retain rejected
model/source/predictions in external frozen-candidate, with manifests. Scoped jobs
8GiB/0swap, persistent TMPDIR, setsid --wait timeout --kill-after=10s 600; cargo -j1
(except fmt), tests --test-threads 4. Roughly20 minute batch; report incompleteness.
