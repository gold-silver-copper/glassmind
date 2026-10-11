# Equation sequence: completed, rejected, runtime rolled back

One bounded experiment at base `c9d10dc1a1348b757be3ea96db3ee600674e155d`.
No commits, agents, external model calls, services, downloads, API keys or credential
reads. Only `tools/quantities/equation-sequence/` remains uncommitted. Production
crates and Cargo.lock are restored byte-for-byte to HEAD. No integration claimed.
Original production binary SHA256:
`5a94dba0b893fafed21906c2150e51de7fab94c67cadefba10008b2a187e4b9f`.

## Implemented experiment

The archived patch implements a real compositional model, not a program bank:
three linear softmax policies (operation/STOP, left operand, right operand),
5/42/42 action vocabularies and legal backwards-reference masks. Numeric-slot
local context, slot rank/sign/integer shape, word/step conjunctions, previous
operation and complete generated graph edge/use features accompany question
unigrams/bigrams. Right prediction conditions on the chosen left argument.
8192 hashed sparse input features per policy, dense f32 coefficients; 12 fixed
SGD epochs; greedy beam1; six binary-step maximum. All 4590 cleaned FIT rows and
1565 structures contribute, with no minimum-frequency restriction. No examples,
training IDs, program classes, pretrained assets or answer tables in the model.

Reused the exact retained rational executor and UTF8 binding (not centroid model).
Standalone Rust examples `equation_sequence`, `equation_fit`, `equation_executor`;
main CLI and default solving were never edited. Every emitted candidate has
unverified semantics, typed graph, exact value/spans, actual model SHA256 and
provenance. File loader cap corrected to 32,000,000 decimal bytes. Native CLI
rejects oversized rows and annotation objects. Existing locked sha2 0.10.9 was
linked for model hashing; no new package download or lock version introduced.
Patch, all modified source and tests retained locally under `rejected-source/`.

## Results (unchanged proposal-numeric-label-v1 / exact-label-comparison-v3)

| Set/model | Total | Correct | Refused | Wrong | Unscorable | Selected |
|---|---:|---:|---:|---:|---:|---:|
| TRAIN calibration candidate | 5925 | 175 | 1655 | 2563 | 1532 | 0 |
| Fixed sample candidate | 200 | 12 | 56 | 77 | 55 | 0 |
| Fixed sample stronger lexical | 200 | 32 | 76 | 44 | 48 | 0 |
| Full DEV candidate | 4475 | 263 | 1314 | 1722 | 1176 | 0 |
| Full DEV stronger lexical | 4475 | 663 | 1747 | 1132 | 933 | 0 |
| Full DEV restricted512 lexical | 4475 | 518 | 1850 | 1183 | 924 | 0 |

Unscorable conservatively counts wrong: candidate full2898, calibration4095,
sample132. Full candidate loses400 correct versus stronger lexical. Raw minimum
250 passed but binding >=688 gate failed. No threshold achieves TRAIN90%/30:
best precision among >=30 selections is99/196=50.5102%, score
-0.45969149342281107. Selection disabled, **experiment rejected**. All selected
correct/wrong counts zero and precision undefined, including both baselines;
no fictitious100% baseline and no selected-gate claim. DEV80%/50 and +10 gates
also fail. No threshold weakening, second configuration or DEV-driven refit.

Full DEV:3161 executable proposals,1357 distinct generated structures;
2174 proposals /1146 distinct structures absent from FIT. Twelve novel-structure
proposals match numeric labels. Sample:144 proposed,125 structures,101 novel
rows /96 novel structures, zero novel correct. Max generated steps6, beam1.
Novelty is versus FIT annotations only; no DEV formula was read. Numeric match
does not prove semantic correctness. Six unchanged public authored contrasts
score0/6, demonstrating poor semantics despite compositional expressivity.

## Validation and freezing

- All29837 retained TRAIN executor records and all29837 normalized question
  group hashes independently match before fit. Split fit23912/cal5925;
  exact cleaned4590/1565 confirmed. Baseline hashes and identical source revision
  verified; lexical inference not repeated. Same fixed200 question-hash sample.
- Fit target counts: operation/STOP19877,left15753,right15753. Mean action NLL
  by epoch:1.678676,1.308444,1.165843,1.074393,1.008563,0.957926,0.917491,
  0.884196,0.856108,0.831987,0.811042,0.792593. No early stopping or tuning.
- Four meaningful Rust contracts pass before fit and again on final source:
  exact UTF8 binding, unsupported spelling/bounds, arithmetic/reference refusal,
  malformed models plus state/operand-conditioned feature/mask checks.
- Authored CLI boundary, malformed model, decimal size cap, oversized row and
  annotation-input checks pass. Six semantic contrasts fail and remain retained.
- Cargo fmt/release build/clippy pass. Initial clippy rejected unchecked indexing
  in new bigram code; safe destructuring/index access fixed before fit, original
  failed log retained. No invalid fit or prediction attempt, one fit only.
- Source/binaries frozen before fit; model frozen before inference; calibration,
  sample and full outputs individually hashed before their scoring. DEV reads
  only Problem before predictions; options/correct after full predictions.
  No DEV formula, rationale, category, heldout or parent-owned tests/results read.
- Default solve sample200/full4475/synthetic3116 are byte-identical to retained
  supplied baseline, preserving ordinary29/4443/3 and synthetic3056/60/0.
  Full output SHA256 `5cf77234ace536a7f147c3a1bc4ac832a1052e2ac50c05ea232a1ce4a175eb98`;
  sample `dc6b9f8d10bcf64c23295e666e67edd63f10616ad9068b9b34458a87aa0a85bf`;
  synthetic `ab818fdee8b4b383c8e0439dae83d9136926c6a5b4558c66033dfff4dac895ab`.
- All frozen artifacts verified before rollback. No owned jobs remain running.
  Outer and all cargo/native/training/evaluation scopes use8GiB/0swap,
  setsid --wait, timeout600/kill-after10, persistent TMPDIR; cargo-j1 and tests4.

## Provenance and retained artifacts

Existing MathQA source revision `fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`,
Apache-2.0 per existing source card. No newly downloaded data.
TRAIN29837:9,013,733bytes SHA256
`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
DEV4475:1,350,141bytes SHA256
`488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
Source card7,438bytes SHA256
`8051c9a1e42245600985ff1b890dfe3ecce804c47555d83ca7a02a2e3f4b4cc2`.
Total existing source/card bytes10,371,312. Source mount:
`/home/gsc/.cache/glassmind/continuous-20261010/equation-source`.

Persistent complete archive (roughly95MiB, no credential data):
`/home/gsc/.cache/glassmind/continuous-20261010/equationsequence-run/tmp/equation-sequence`.
Contains exact source, four binaries, model, TRAIN input/executions/split,
question-only inference inputs, all predictions, metrics and hash manifests.
Small reports, logs, rejected patch/source copied into this owned tools directory.
Runtime model contains weights only, not retained offline training examples.

| Artifact | Bytes | SHA256 |
|---|---:|---|
| model.json | 8046176 | `2b56181582960a862b2da422b0988cf3e20671949c80253bb5b0eaa2d49478e3` |
| equation_sequence | 16658576 | `7f8f6f8658cedde3590fbf7dd2e462cb88f505e3f0dcc22e6249dd1f9079c0d3` |
| equation_fit | 17050144 | `5b2a02b6795e1d050b6cdb7723a3f167b6ae16278cce12e02d96a742d7bf84de` |
| full-candidate.jsonl | 2718909 | `f542de12b9e0f239795df1367161cc3f203736d3d969d70ef8a582b5a7cb06db` |
| sample-candidate.jsonl | 124426 | `df60bc8a87da14ee9075f6cf58d9c1e8c552669ba58cf94d375834e8c704df98` |
| calibration-candidate.jsonl | 3643109 | `5eb26fc5476845d27ea5c558b9627c30b1f0803c73a869abb730b647f99da354` |
| rejected-runtime.patch | 27141 | `ffcaf3c0f2a70cae218b1279ed116360a2746dd79cd8ada3b9737d83cf42922d` |
| prefit-manifest.json | 6078 | `876ec8eec8d2f0d520b3853f90914436e430d6dc3ecead9c8f1747fa36813701` |

## Reproduction

Run only in an isolated checkout of the recorded base. Reapplying the rejected
patch is for reproduction, not production integration. Use a fresh run directory
(the trainer intentionally refuses to overwrite a fitted model):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/equationsequence-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/equationsequence-run/target
export SEQUENCE_RUN="$TMPDIR/equation-sequence-reproduction"
git apply tools/quantities/equation-sequence/rejected-runtime.patch
bash tools/quantities/equation-sequence/reproduce.sh
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 python tools/quantities/equation-sequence/contracts.py
```

`reproduce.sh` uses the existing PyArrow directory, cargo-j1/testthreads4 and
bounded scopes. Offline experiment imports unchanged split/parser/label metric
helpers from retained worker source; that frozen directory and source mount must
remain available. Every child native execution receives its own identical scope.
All exact commands/logs and frozen implementation are retained. No repeated
training run is claimed; reproduction determinism has not independently been run.

## Limitations and disposition

Factorization can compose unseen shapes, but this fit does not learn reliable
operation/operand semantics; teacher-forced loss does not imply free-decoding
correctness. Greedy decoding compounds local mistakes. The experiment does not
establish entailment, broad arithmetic coverage, reliable confidence or safe
answer routing. Unsupported number spellings and bounded annotation cleaning
exclude much TRAIN data; calibration/DEV denominators were never narrowed.
Parent semantic/executor checks were not run or waived. No production improvement
is claimed; archive evidence only. Future work needs better structural learning
and argument grounding, not a wider class bank or weaker threshold. Current
configuration is rejected and must not be integrated.

Frozen PLAN is reproduced verbatim below; original file/hash remains unchanged.
SHA256 `bb635ad26da1dbd595290cd6a16d6250e1127106f82624261a1baf606e12f1b4`.

---
# Frozen equation sequence experiment (2026-10-11)
Base c9d10dc1a1348b757be3ea96db3ee600674e155d. One fitting configuration.
Hypothesis: teacher-forced compositional linear operation/argument policies can
assemble unseen program structures, improving on whole-program lexical retrieval.
All 4590 cleaned FIT rows / 1565 structures; exact retained normalized question
SHA256 modulo 5 split; calibration all 5925, DEV all 4475. Retained executor and
TRAIN executions checked before fit. No DEV formulas/rationales/categories.

Representation: typed add/subtract/multiply/divide + stop policy (5 actions),
left and right operand policies (42 actions: 16 inputs, 20 declared constants,
6 prior steps). Legal masks disallow missing inputs and forward references.
8192 hashed binary features: word unigrams/bigrams, word-by-step conjunctions,
number-slot ordinal/local +/-3 word context, numeric rank/sign/integer shape,
step index, previous operations and operand identities/use, current operation,
and selected left operand for the right policy. No program classes or examples
in model. Dense f32 coefficients with sparse inputs, zero initialization.
Softmax SGD teacher forcing, 12 deterministic epochs, corpus order rotated by
(epoch*997)%4590; seed 0 (no RNG); learning rate 0.08/(1+0.2*epoch), feature
normalization 1/sqrt(active feature count). No class pruning or tuning.
Greedy beam=1, maximum 6 binary steps; STOP masked at step0; forced termination
after step6, no execution fallback. Argmax ties lowest action index. Reject
nonfinite scores, illegal model/graph, unsupported spelling, zero division and
overflow. Confidence selection score = mean chosen conditional log softmax
across generated actions including chosen STOP (forced step6 termination adds
no action). It is a ranking score, never a truth probability.
Model <=32,000,000 bytes; question <=4096 UTF8 bytes; <=16 numeric slots;
20 executor constants unchanged. Source spans and graph/value always explicit
unverified semantics, actual SHA256 model hash and source provenance.

Freeze source/binary/model and every prediction batch before label scoring.
Fixed first200 SHA256(question) DEV sample then full4475, no tuning from sample.
Raw acceptance >=250 and >=25 above stronger lexical663 => >=688/4475.
TRAIN cal: maximum coverage at >=90% numeric-label precision, >=30 selections,
score ties grouped. None => selection disabled AND reject. DEV selected >=50
correct, >=80% precision, >=10 additional correct vs selected lexical baseline;
precision loss <=.02 where baseline defined (both baselines disabled here).
Metric proposal-numeric-label-v1 using unchanged exact-label-comparison-v3;
unscorable conservatively wrong. Preserve all denominators/refusals.
Default solve sample200/full4475/synthetic3116 must byte-match retained baseline.
Meaningful executor/model/feature/graph contract tests before fit, full TRAIN
executor consistency. Report fit losses/action counts and generated structures
novel versus FIT (no DEV formula). Parent semantic checks remain separate.
If gates fail: preserve patch, source, binary, model, predictions and evidence;
rollback runtime; never weaken gates or refit. No agents/new model calls/downloads.
