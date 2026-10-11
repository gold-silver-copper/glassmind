# Prospective SYSTEM entity ledger experiment

Frozen before fitting, 2026-10-11. Base 2a35c179f34e275b581afc4ac45e50ab0a032940.
Baseline approved v2: 610/1119 joint, 716/1218 exact frames. Model SHA256
fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88;
full DEV prediction SHA256 8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d.
These are supplied baseline evidence, not a new measurement. Read offers,
references, boundaries, freshness, spansequence, token-endpoints and history
diagnosis reports. Previous freshness/USER-span hypotheses remain rejected.

Hypothesis: learned SYSTEM BIO boundaries followed by a learned semantic source
role classifier can expose output-only entities absent from the existing offer
extractor. A separate value-free ranker can select cross-turn references without
introducing wrong assertions. No prior rejected weights reused. Bounded inspection
was first three FIT conversations and schema names; only span annotations supply
supervision, never action values/API results. No entity list is constructed.

Source package entityledger-source manifest pins 11 files, 120818396 bytes.
SGD e852981ae34990f4358979625854259302feaa78, CC-BY-SA-4.0. TRAIN 45291003 bytes,
full TRAIN schema 75764, selected schema 3707, license 20139. Zero downloads.
Derived annotations, weights and data-bearing reports retain CC-BY-SA-4.0.
Code MIT OR Apache-2.0. Source manifests/hashes saved before fitting.

## ONE frozen representation and schedule

Split all3174 conversations: SHA256(id) integer mod5==0 CAL642, otherwise FIT2532.
All USER turns, including unselected services, retained. Old models saw all TRAIN
and earlier diagnostics inspected both groups: empirical new-component validation,
NOT unseen calibration. Only FIT chooses labels or fits weights.

Three averaged perceptrons, deterministic original dialogue/turn order, unit
updates, no shuffle/early-stop/refit. SYSTEM BIO boundary classifier: 6 epochs,
3 labels, legal I only after B/I, token features lowercase windows +/-2, token
shape, current prefix/suffix1..3, previous predicted BIO label. Semantic source
role classifier: 8 epochs, labels service/slot from FIT noncategorical SYSTEM
span annotations (minimum20 occurrences, max128 sorted frequency then name),
features masked source context, outside boundary words +/-3, length and predicted
prior-active services. Includes output-only roles. Boundary model trains every
SYSTEM token (other annotations/unaligned spans remain O); role classifier trains
all retained exact annotated spans. No value text in role features.

Ranker: 8 epochs, structured candidate-vs-NONE averaged perceptron. Candidate
features target service/slot, predicted intent/active services/filled slot names,
source role, masked source boundary context, current USER unigrams/bigrams,
recency buckets0/1/2/4/8/16/32, number of alternatives and latest-role indicator.
No candidate value text or gold previous state/activity in features. Mask ledger
values in USER lexical context. Train on actual frozen runtime missing-slot
opportunities, all candidate negatives; NONE wins if no correct candidate.
Gold matching offline only, case-insensitive exact (not fuzzy). No service-pair,
entity, task or lexical rejection rule. Reject ties/ambiguity by margin.

FNV hashed features; BIO 3x65536 floats, role shared262144 floats, rank262144
floats; averaging accumulator offline only. Total persisted new weights<=16MiB
(enforce serialized file bound too). No vocabulary/answer store. Input<=4096 UTF8
bytes and128 tokens; over-limit source/user abstains. Ledger newest128 spans,
last32 SYSTEM turns, source spans<=16 tokens; deterministic oldest eviction.
Preserve full source UTF8 text, byte endpoints, turn, predicted role, source
confidence and ranking evidence. Session reset clears ledger. Runtime types
reject annotations. Multiple roles/candidates retained, not latest-value priority.

Finite CAL grid: source minimum margin [0,2], rank-vs-NONE minimum [2,4,8,16],
rank gap against every different value [2,4] (16 combinations). Decode/train once;
no alternative representation/grid. Gate each member using full sequential CAL
state overlay: >=30 newly filled previously missing correct scored slot-turns,
ZERO newly incorrect scored assertions, zero baseline-correct joint/frame losses.
Held/inactive changes reported separately, no credit. Account for ALL changed
scored turns, including persistence. Select greatest correct fills then highest
confidence lexicographically. If none pass: archive/reject, NO candidate DEV.
Raw proposals, golds, baseline states, decisions/counts retained for review.

Optional MultiModel integration only after gate; baseline state trace shadow
overlay before that gate is conservative and provisional (does not establish
full integrated behavior). If gate passes, implement full integration and repeat
same CAL gate without changing weights/grid before DEV. Missing noncategorical
input slots, predicted-active only, after old mechanisms, never overwrite.

Before any candidate DEV freeze source/models/native binaries. Unchanged
 tools/dialogue/evaluate.py first10 then full179/1119 turns/1218 frames.
Accept >=620joint and>=726frames, retain all610/716, ZERO new wrong slot assertions,
no route/intent changes, exact denominators, audit all changes/inherited errors.
No DEV retune. Reverse-order identical, legacy45 byte-identical, absent old model
byte-identical. Draft bundle behavior unchanged. If no integration, report these
checks as unneeded/not run rather than claim them.

Focused native tests: exact Unicode byte provenance, eviction/reset, ambiguity,
NONE, finite/model limits, annotation rejection. Required fmt, affected all-target
clippy, release. Every Cargo invocation TMPDIR persistent, existing CARGO_TARGET_DIR,
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600, -j1 except fmt, tests --test-threads 4. Outer scope verified
8589934592/0. No agents/model calls/background survivors/credentials/commits.
