# Preregistered experiment, 2026-10-11, before implementation or DEV

Base 69ba975a9f44bd8f3aaf45be355fffe82cbc6577. Hypothesis: an averaged
structured BIO perceptron with learned transitions and reusable utterance/context
features can propose complete missing/correction spans that independent classifiers
miss, without losing any baseline correct joint turn or introducing wrong changed
slot assertions. Exactly one representation and schedule; no DEV retuning.

TRAIN-only inspection: 3174 conversations, 34022 USER turns, max265 characters;
selected active frames Travel6491/Ride3455/Weather2816. Span annotations:
location970, destination281, city678, date840. Use only noncategorical input slots
from full TRAIN schema. FIT2532/CAL642 via integer SHA256(id) mod5==0 for CAL.
Old frozen models used all TRAIN; CAL is held out only from NEW weights.

Representation: per-service linear-chain BIO, O plus B/I per input slot;
legal BIO transitions, learned emission and transition weights; token windows
+-2 lowercase words and uppercase/numeric/word shape; current prefixes/suffixes
length1..3; first/last; unique previous-system token features (max32). Feature
vocabulary FIT only, document frequency>=2, capped50000/service. Max256 tokens,
4096 UTF8 bytes/input, max32 labels. No embeddings, answer tables, new lexicon,
pretrained models or runtime annotation input. Sparse exact feature dictionaries.

Schedule: 8 epochs, deterministic original FIT order, online structured perceptron,
unit update, average after every sequence; no shuffling/refit/early stopping.
Confidence: minimum max-marginal gap over proposed BIO span and its adjacent
boundary tokens. Select once on CAL the lowest of [0,2,4,8,16,32] with >=30 raw
proposals and >=99% exact annotated span precision, pooled across services.
If none passes, reject calibration (no fallback threshold). One longest qualifying
span per slot overwrites/adds current predicted slot after unchanged service
advance and before unchanged references/offers. No deletion or transfer suppression.
Threshold is empirical precision, not a calibrated probability.

Freeze model and source SHA256 before DEV. Evaluate fixed first10 then all179
through unchanged evaluate.py/compare.py. Baseline610right459wrong50refused,
716/1218exact frames,1119USER turns. Accept only >=610joint and>=716frames,
all610correct retained, zero newly wrong changed assertions, and strict gain in
joint/frames OR repair of inherited incorrect assertions. Enumerate every change,
all7 inherited errors, raw proposals versus downstream claims and held-unscored
states, no free credit. Failure restores runtime and preserves rejected patch.

Validate bounded model/input handling, novel names/multiword/punctuation/correction/
Unicode/negatives, isolation/reverse reset; optional-absent v2 full bytes and legacy
45/202-of257/89.17509727626459%fuzzy unchanged. No second hypothesis after DEV.
All cargo/training/inference runs scoped8GiB0swap/setsid--wait/timeout600; cargo-j1,
tests--test-threads4, persistent TMPDIR and existing CARGO_TARGET_DIR. No new
external data, services, workers, LLM calls, credentials, keys, commits or testsplits.
