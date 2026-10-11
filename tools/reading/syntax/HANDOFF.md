# Syntax batch: ACCEPTED against frozen experimental gates

One candidate, one actual TRAIN run, no retuning. Uncommitted changes ready for
coordinator review; no main-command integration or QA/action semantics changed.
Base source `a34306ab8d314551d54748189a34ab1f0d82d298`. Runtime and evaluator frozen
2026-10-11T08:53:25.246555Z before DEV; all frozen hashes reverified after scoring.

## Exact change and API

- New `crates/reading/src/syntax.rs`: deterministic averaged-perceptron UPOS tagger
  and labeled arc-standard parser, buffered validated model loading, sparse binary
  saving, `Model::infer(&[String])`, `Model::predict(Input)`, `tokenize(&str)` with
  UTF8 byte spans, and independent `valid_tree`. Four meaningful Rust tests.
- Minimal `pub mod syntax` export in `crates/reading/src/lib.rs`.
- New `crates/reading/src/bin/reading-syntax.rs`: separate `train` / `predict`
  adapter. Strict runtime JSONL has exactly forms OR text; denies annotation and
  ID fields. Only training adapter reads gold CoNLL-U. No runtime lookup/LLM.
- New `tools/reading/syntax`: preregistration, authored contrasts, standalone
  adapters/contract checks, reproduction, reports, provenance and artifact index.

Model schema `ewt-arc-standard-v1` / binary magic GMSYN001. Sorted TRAIN-derived
17-UPOS/51-dependency inventories, feature-hashed dense native tables and sorted
sparse binary weights. Tag hash65536, parser hash131072; up to19 tag and68 parser
features per decision. Lazy averaged weights, deterministic class tie order,
static projective oracle. Two tagger epochs plus three parser epochs; all ordered,
no random seed/shuffle. Parser uses the frozen tagger's PREDICTED tags during
training and inference, so no gold/predicted-tag mismatch. In-sample TRAIN tags
and oracle histories can nevertheless be easier than DEV inference histories.
No CAL: fixed uncalibrated predictor, no acceptance/confidence threshold.

Runtime limits:1..128 forms and16KiB aggregate form bytes; raw text16KiB; input
JSONL128KiB; exactly2*n transitions; model128MiB with bounded headers/dimensions/
inventories/sparse entries, sorted unique keys, finite bounded weights and exact
EOF. Invalid input/model explicitly refuses/errors. Single root, root-label
consistency, no cycles/selfheads checked independently on every output. Each
successful output explicitly says status=predicted and model_version. None of
these structural checks establish linguistic correctness or semantic truth.

## Frozen gates and measured evidence

Primary experiment uses **gold UD integer-word tokenization**, including every
word and punctuation. No DEV filtering or silently removed duplicates.

| Split | Words | LAS correct | LAS | UAS correct | UAS | baseline LAS | baseline UAS |
|---|---:|---:|---:|---:|---:|---:|---:|
| First100 DEV sentences | 2319 | 1738 | 74.9461% | 1834 | 79.0858% | 6 (0.2587%) | 167 (7.2014%) |
| ALL2001 DEV sentences | 25148 | 19423 | 77.2348% | 20540 | 81.6765% | 481 (1.9127%) | 2528 (10.0525%) |

Gate LAS>=45%, UAS>=55%, both strictly above the fixed previous-word chain: PASS.
Baseline attaches first word to root/root and every later word to previous/dep.
It is a transparent weak comparator, not a competitive same-dataset parser or
SOTA comparison. These scores are NOT comparable with hand-grammar/BliMP/SQuAD.

All2001 predictions emitted, all2001 valid single-root trees:100% sentence and
word coverage, zero unsupported/refused. All33 nonprojective DEV trees/977 words
remain scored:685LAS/734UAS (70.1126%/75.1279%). The model can only emit projective
trees. No DEV sentence exceeds128; max75 tokens. Overlength refusal independently
checked at129. Punctuation-excluded diagnostic only:17020LAS/18132UAS of22073
words,77.1078%/82.1456%; primary25148-word denominator remains authoritative.

Duplicates, retained in primary counts and per-sentence ledger:
-68 DEV sentences/137 words have exact TRAIN-identical native forms,137/137 both
metrics. Other1933sentences/25011words:19286LAS (77.1101%),20403UAS (81.5761%).
-65DEV sentences/129words have identical complete integer rows (all10columns,
comments excluded),129/129 both. Other1936/25019:19294LAS,20411UAS. Full annotation
identity is stricter than identical dependency targets and includes unused fields.

First100 predictions equal the exact full-output prefix. Ledger sums independently
recomputed and match totals. Source/model/evaluator freeze identities unchanged.

## Authored raw-text behavior and actual errors

Sixteen sentences authored before any model evaluation. Six active reversal
sentences (nurse/teacher, dog/cat, manager/driver) all recover both nsubj and obj:
6/6 sentences,12/12 edges, passing the prewritten>=4/6 gate. All33 specified core
edges across15 sentences are correct, including passive/by-agent, not/did-not,
question/statement, accented names and coordination. This is a small authored
behavior check, not broad raw-text accuracy. The remaining attachment ambiguity
has no gold choice: binoculars attaches to teacher/nmod; the instrument reading
would attach to saw/obl. Actual predictions/spans are retained, including the
Unicode example's dash attached to politely, which was not a preregistered edge.

Raw tokenizer joins Unicode alphanumeric scalars and marks in explicitly coded
combining ranges; each other nonwhitespace scalar is a separate token. It neither
segments sentences nor performs clitic/MWT expansion. Apostrophes/hyphens split;
UTF8 source spans are exact. UD's native forms may already expand contractions
and preserve forms differently, so gold-tokenized DEV accuracy cannot be claimed
for raw text. Tokenization tests include Élodie, em dash, apostrophe and combining
accent. The later API-contract input `Élodie can't—go! é` becomes
`Élodie can ' t — go ! é`; its tree wrongly makes t the root. This is retained as
an explicit raw-text limitation, not called a linguistic success.

The post-freeze contract sentence `Birds fly .` tags fly=NOUN and labels Birds
compound rather than nsubj. DEV has5725 wrong labeled edges,4608 wrong heads,
including307 wrongly chosen roots (structural validity is100%, root accuracy is
1694/2001). The first DEV error labels story as obj where gold is nsubj; the second
sentence makes Bush the root instead of nominated. Full per-sentence ledgers
retain all errors, and verification.json retains first12 and counts by gold label.

These predictions do not prove predicate identity, negation scope, argument
meaning, entailment, question-answer correctness or verified facts. Negative or
unknown syntax must never authorize QA claims, repairs or code actions. There is
no confidence calibration and no semantic acceptance threshold in this model.
Coordinator owns any optional main-command presentation of predicted syntax.

## Data, TRAIN skips and licensing

Read-only independently supplied UD English EWT r2.18 revision
`b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c`, CC-BY-SA-4.0. Source URL, exact SHA256s
and bytes in source-provenance.json; identities reverified. Source README and
license retained with attribution under external data-notice/. Model/derived
corpus artifacts distributed CC-BY-SA-4.0; code independently authored MIT.
See NOTICE.md for authorship, underlying-text caveat, license and no warranties.

| Source file | Exact bytes |
|---|---:|
| README.md | 24522 |
| LICENSE.txt | 20014 |
| en_ewt-ud-train.conllu | 15034493 |
| en_ewt-ud-dev.conllu | 1898574 |

Total16977603source bytes plus coordinator provenance1444bytes. No downloads,
pretrained models, official test, held-out/hidden material or old private logs.
Only first THREE TRAIN sentences inspected before plan, plus aggregate TRAIN
statistics. TRAIN12544sentences/204578integer words; skip2614range and37decimal
rows explicitly. DEV2001/25148; skip359range and4decimal rows explicitly. These
are basic dependencies, not enhanced decimal-node/enhanced-edge predictions.

Tagger learns12542sentences/204284words; two sentences294words exceed128 and skip.
Parser learns12256sentences/195962words. All287nonprojective TRAIN trees8457words
are excluded, including one already excluded as long: mutually exclusive runtime
skip counts2input +286nonprojective +0invalid-tree. Every skip is external
train-skips.jsonl with ID/word count/reasons. Tags are trained on eligible
nonprojective sentences too. No malformed inputs or byte-limit skips in this data.

## Validation, resources, original errors, rejected alternatives

PASS: workspace fmt/check, workspace clippy/all-targets -Dwarnings, complete
workspace release build. Reading tests12/12:4new syntax,6unchanged QA,2unchanged
byte-model tests. No existing test or evaluation definition changed. Syntax tests
cover UTF8/input bounds, unknown annotations, oracle reconstruction and
nonprojectivity, cycles/multiple roots, deterministic model roundtrip,
truncation/trailing data/NaN/dimension corruption. Additional16CLI input/span
contracts passed; real model NaN corruption rejected. Real learned model
byte-exact load/save/reload and deterministic predictions checked at lengths
1/2/3/127/128;129refuses. No large unrelated test suites or main QA evaluation.

Training native process:3.374356wall seconds,3.316170user +0.047848system seconds,
291672KiB max RSS. Full native inference:0.143606wall,0.139065user +0.003965system,
60016KiB max RSS. These are single warm-environment observations, not robust
benchmarks; timing excludes separate Python corpus/scoring. Detailed per-command
resource records retained. Every Cargo invocation had TMPDIR, existing target,
8GiB/zero-swap scope, setsid--wait,600s timeout/10s kill delay,-j1 (except fmt),
testing threads4. Outer8GiB/zero-swap verified. No agents/services/background jobs
or additional model calls launched; no credentials/API keys read or used.

Original issues retained: initial fmt check reported only new-file formatting;
fixed before training/evaluation. First training LAUNCH exited127 because
`/usr/bin/time` does not exist, before binary ran. Replaced instrumentation with
foreground Python resource wrapper, then the only actual training run succeeded.
Initial TRAIN-audit helper double-counted nonprojective totals (574instead of287);
original JSON retained and helper fixed before freeze/DEV, without touching model
or scorer/gates. No rejected learned alternative or second candidate attempted.
No acceptance gates lowered. No runtime code changed after DEV freeze.

Workspace feature unification rebuilt a different binary; an additional full
forms-only replay verified byte-identical predictions, with both binaries kept.
This is a build-compatibility check, not another fitted model or selected run.

## Retained artifacts and reproduction

External root:
`/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/artifacts`.
`ARTIFACTS.json` indexes every raw retained artifact and read-only source file by
absolute path/bytes/SHA256. Small reports are copied into reports/; large model,
inputs/predictions/ledgers and binaries remain external. All measured commands
are recorded in COMMANDS.md; reproduce.sh consolidates the successful sequence
and is syntax-checked (not claimed as a second complete run).

Authoritative model `model.bin`,12554724bytes:
`47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6`.
Frozen evaluated binary `reading-syntax`:
`6aecfd2e1fccf38362f43a9f771b82b2e7cb3c762c4f2241ee6457b6080e3e15`.
Final workspace binary `reading-syntax-workspace`:
`9c8f994cc9180a2a68bb582aa279c71076f368c3545ae7607508f9614b15b0a7`.
Full predictions:
`059e5ca85b312f9ea692d01fcb30e67f6334fad002b704e686c59ecb7638710d`.

To reproduce from repository root, set inherited CARGO_TARGET_DIR and non-/tmp
TMPDIR, then `OUT=/absolute/new/output bash tools/reading/syntax/reproduce.sh`.
For inference only:

```bash
ART=/home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/artifacts
"$ART/reading-syntax" predict --model "$ART/model.bin" \
  --input forms-or-text.jsonl --output predictions.jsonl
```

Library usage: `Model::load(BufReader::new(File::open(path)?))?`, then
`model.infer(&forms)?` or `model.predict(Input { forms:None, text:Some(text) })?`.
Use `tokenize` independently when span inspection is useful. JSON input must not
include IDs or gold annotations; association with corpus IDs is offline only.

No commits made. No shared manifests, README/data README, existing QA/probability,
English/world, main CLI, loop, scorers or historical prompts modified. All owned
commands are terminal. Main command integration remains intentionally owned by
the coordinator; this handoff makes no automatic truth/repair integration claim.
