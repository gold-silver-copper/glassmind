# Contextual source-role / NONE experiment — frozen before implementation

Status: planned; one representation and one fitting schedule. Start 2026-10-11.
Source checkout d84ba870465562456fb389805d3703ee12d41f23. No runtime integration
is planned unless the frozen CAL gate passes. Existing published DEV baseline:
610/1119 joint USER turns, 716/1218 strict frames, 179 dialogues. These are supplied
baseline results, not freshly measured candidate results. Prior entity-ledger
role/rank experiment rejected: all 16 members added 41–71 wrong assertions and
lost 2–4 correct states. No old role/rank weights are reused.

## Hypothesis and acceptance
Preceding actual USER text plus masked SYSTEM context and predicted state,
trained on actual frozen-boundary spans with explicit NONE, reduces erroneous
source-role assignments and supports safe missing-slot references without
filtering source owners to the three routed services. Only FIT may fit weights
or choose vocabulary. CAL is empirical new-component validation: old baseline
weights saw all TRAIN and prior diagnostics inspected FIT and CAL.

Exactly 2532 FIT / 642 CAL of 3174 selected TRAIN conversations, assigned by
SHA256(dialogue_id) integer modulo 5 == 0 for CAL. All USER and SYSTEM turns,
including unselected-service turns, retained. Supervision: all actual predicted
SYSTEM spans get a target. Deduplicate identical exact span/role annotations;
one unique exact role is positive if in FIT label vocabulary; no exact annotation,
multiple distinct exact roles, or out-of-label role => NONE. Missing annotation
is incomplete supervision, NOT proof of semantic absence. Character offsets are
converted to UTF-8 byte offsets offline; no approximate/gold boundary substitution.
Roles are schema service/slot strings from all TRAIN schema, never pair rules.

## Frozen representation / fit
Frozen BIO decoder and weights ONLY from supplied rejected-model.json. No fitting
of boundary or reuse of its old role/rank weights. Source role labels: NONE index
0 plus roles with >=20 unique exact predicted-span matches in FIT, ordered by
frequency descending then lexical, capped at 127 roles. Every predicted source
span is a training example, including all negatives. Eight deterministic
averaged-perceptron role epochs, then eight deterministic averaged-perceptron
reference rank epochs; zero initialization, original sorted conversation/turn/
span order, no shuffle, balancing, subsampling, second schedule or tuning.

Role features: bias, capped span token length, three adjacent tokens each side,
SYSTEM noncandidate bag-of-words, preceding actual USER unigrams/bigrams, and
baseline predicted active service names, intents and filled slot names. Mask
all current predicted spans, bounded previous raw span values and baseline
predicted state values in SYSTEM/USER lexical features. No gold values, gold
state or gold activity enters features. A role-independent raw span history
(128 spans, 32 SYSTEM turns) supplies masking only; NONE never enters the role
entity ledger (also 128 spans/32 SYSTEM turns). Input limits 4096 UTF-8 bytes/
128 tokens, span limit 16 tokens. Over-limit USER texts have no rank candidates;
over-limit SYSTEM texts have no extracted spans. All skipped turns still counted.

Source confidence is winning role score minus highest score of ANY other class,
including NONE, independent of boundary margin. A NONE winner never enters the
entity ledger. All per-class scores, decisions, raw spans, exact labels and hashed
features retained for independent reconstruction. No source-router owner filter.

Rank features reuse value-free role/target pair, source context, recency buckets
[0,1,2,4,8,16,32], latest-role flag, ledger count capped8, predicted intent/activity/
filled-slot names, masked current USER unigrams/bigrams. All known raw/accepted
entity and predicted state strings masked. No entity identity/equality feature.
Rank fitting uses all missing noncategorical input slots of predicted-active
services. Truth is highest-scoring candidate whose raw value exactly case-fold
matches offline annotated target values, deterministic first on ties; otherwise
NONE (fixed score0). All negatives and empty opportunities remain. Training uses
unchanged baseline state, not gold state or gold previous activity. Positive vs
negative updates use ordinary averaged structured perceptron, 8 epochs. Hashes:
FNV-1a, 262144 shared role bins and 262144 rank bins, 196608 frozen boundary weights.
Serialized total model <=16MiB, finite validated weights and role/schema checks.

## Frozen selection and audit
Exactly source role margin [0,2] x rank-vs-NONE [2,4,8,16] x gap [2,4]. Every
candidate with a different case-folded value competes regardless of source margin;
NONE score0 also competes. First candidate wins equal scores (gap will refuse
conflicting equal values). No further lexical vetoes, thresholds or retuning.
Full persistent overlay: apply only missing noncategorical input slots to baseline
predicted-active services after existing models, never overwrite baseline values,
never change routing/intent; retain overlay across later turns and clear the
corresponding held entry when baseline supplies that slot. All changed stored
slot-turns audited, including propagation. Held/inactive changes recorded separately
and receive no correctness credit. Gold scored input slots follow unchanged
upstream metric schema definitions; fixed denominators across all members.

Gate BEFORE candidate DEV: >=30 newly correct previously missing scored slot-turn
assertions, ZERO newly incorrect scored assertions, no lost baseline-correct CAL
joint turns or strict frames. Choose passing member by most new correct fills,
then joint gain, then lexical grid order. If none passes, archive/reject: no candidate
DEV, no alternate representation/grid. If passed, freeze sources/models/binary,
then unchanged evaluate.py first10 and full179. DEV acceptance >=620 joint,
>=726 frames, all610/716 retained, zero new wrong scored assertions, unchanged
routing/intent/denominator, reverse replay equal, legacy45 byte-identical.

## Sources, limits and reproduction
Both manifests independently hash/size verified before edits. contextledger-source:
3 payloads /20,369,681 bytes; rejected-model SHA256
661c80b75a808994427a53591c651cb3d1ddc2fc1e0624503d6b70e4ca09b0e0.
entityledger-source: 11 payloads /120,818,396 bytes; approved model SHA256
fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88.
SGD revision e852981ae34990f4358979625854259302feaa78, CC-BY-SA-4.0;
all derived annotations retain that license. Code MIT OR Apache-2.0.
No new downloads. Supplied input arrays independently equal every sorted TRAIN
speaker/utterance turn; all prediction turn indices align with all USER turns.
No held-out/private logs/parent folders read. No credentials/API/teacher/runtime
LLM. Outer scope verified MemoryMax8589934592/MemorySwapMax0. No agents or services.

Commands (all exact executed commands additionally retained in commands.log):
python3 tools/dialogue/context-ledger/prepare.py
TMPDIR=$TMPDIR systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt --all -- --check
TMPDIR=$TMPDIR systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind-dialogue --example context-ledger -j1 -- --test-threads 4
TMPDIR=$TMPDIR systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy -p glassmind-dialogue --all-targets -j1 -- -D warnings
TMPDIR=$TMPDIR systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build -p glassmind-dialogue --release --example context-ledger -j1
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$CARGO_TARGET_DIR/release/examples/context-ledger" tools/dialogue/context-ledger/artifacts/dataset.json tools/dialogue/context-ledger/artifacts /home/gsc/.cache/glassmind/continuous-20261010/contextledger-source/rejected-model.json
/home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/metrics-venv/bin/python tools/dialogue/context-ledger/audit.py
