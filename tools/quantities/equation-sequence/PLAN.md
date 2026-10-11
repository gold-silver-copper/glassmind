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
