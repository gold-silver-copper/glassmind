# Frozen experiment: optional width-4 syntax beam

Hypothesis: retaining cumulative-score alternatives recovers dependencies missed
by greedy inference with the exact approved model. Baseline LAS19423/UAS20540 of
25148 integer words including punctuation, 2001 DEV sentences, zero refusals.

One candidate only: width4, existing finite f32 transition scores summed without
normalization in f64. Tags, features, legal transitions, root/cycle validation,
tokenization, model version/serialization/weights unchanged. Exactly2*n synchronous
steps, n<=128. Enumerate parents in retained order then action IDs ascending.
Strictly greater cumulative score ranks first; exact ties retain enumeration order.
Keep at most4 transition descriptors then clone only selected successors; at most4
retained states and4 successors transiently, no unbounded search. Scores for every
action must be finite, including illegal actions (same refusal policy as greedy);
nonfinite cumulative totals refuse. Model label cap128 means <=257 actions,
<=4*257*256=263168 examined scores, <=1024 selected state clones per sentence.
No state merging, normalization, temperature, feature changes or DEV tuning.
Width1 is compatibility-only (delegates unchanged greedy); public API accepts1 or4
only. Standalone --beam selects4. Optional search_mode omitted on default output;
beam successes and row refusals carry explicit mode. Runtime accepts forms/text only.

Freeze 24 authored role/order/question/punctuation contrasts now, before edits.
Each has3 authored UD core edges; all default-correct edges must survive beam.
They are authored development tests, not independent parent review. Parent must
supply its independent contrasts before acceptance. No probe-specific behavior.

Freeze source, binaries, adapter, probes and model hashes before candidate DEV.
Evaluate first100 then all2001 exactly once. Reuse unchanged independent audit.py
scoring; report every edge correctness transition, sentence and duplicate groups.
Acceptance LAS>=19673, UAS>=20540,2001 valid trees,no new refusals, role preservation,
byte-identical default full forms and raw predictions,16 existing contracts and40
raw contrasts, scoped tests/clippy/fmt/release. Report time/RSS/slowdown.
Failure: archive source/patch/binary/reports before rolling back runtime files.
No downloads, training, runtime LLM, agents, commits, shared file edits or test access.
