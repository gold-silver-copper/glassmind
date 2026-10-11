# Frozen equivalent-reference supervision experiment

2026-10-11. TRAIN only; no production/deployment/selection, DEV, heldout, downloads,
LLMs, agents or threshold search. One matched six-epoch continuation only.
Hypothesis: strict expression-tree-equivalent operand references improve grounding
without any change to representation, legal actions or greedy decoder.

Source is the coordinator's readonly groundingaudit-source. Verify every supplied
hash. Reconstruct all 29837 TRAIN memberships from the original normalized-question
hash split, preserve original order, and require exact original 4590 FIT inputs and
1182 clean CAL. Freeze full membership/exclusions before fitting. Cleaning uses only
the original parser, retained original native gold execution and unchanged typed
numeric comparator. First 200 original CAL outputs must reproduce exactly through
both original binary and isolated native helper before fitting.

Fixed original clean CAL baseline: free numeric143/1182, exactAST123, refused46,
wrong993; gold full-plan+length oracle numeric221, exactAST202, refused74, wrong887;
teacher original-exact left2197/4138, right2499/4138. Original FIT free numeric1863,
exactAST1775, refused93; oracle numeric2334, exactAST2258, refused93; operands
12886/15753 and12857/15753. New equivalence metrics never replace these metrics.

Formal policy: each input index is a distinct symbolic atom regardless of numeric
value. Constants normalize with the original exact Rational. Prior references expand
recursively to their trees (original <=6 steps). Only Add/Multiply sort their two
children. Subtract/Divide are ordered. No flattening, cancellation, evaluation,
distributivity, number-position swaps or answer/English equivalence. Enumerate all
original legal operand pairs at each original gold op+prefix, keep exactly those
whose canonical tree equals gold. Always include original pair. This is deliberately
incomplete algebraically; no new actions, masks, constants or programs.

Arm A: singleton original operands. Arm B: partial-label left loss -log sum_{a in V}p(a),
right loss mean_{l in V}[-log sum_{r in V(l)}p(r|l)]. Each valid-left context has
weight 1/|V|, so each step retains one total right-head training weight. SGD ascent
on log target mass has score derivative 1[a in V]*p(a)/sum_V p - p(a).
Weight update is rate * context_weight / sqrt(feature_count) * derivative; all
right-context gradients for a step are computed at the same pre-right-update weights
and summed before application. Singleton exactly reduces to original SGD.
Stable logsumexp only for target mass; original scoring untouched in both arms.
Test gradient by independent central finite differences before fitting.

Both arms start from exact original model weights. OP f32 bytes never change.
Continue original epoch indices12..17, rate0.08/(1+0.2*epoch), original deterministic
row order rotation (k+epoch*997)%4590, without shuffling/seed. Six final epochs only;
no checkpoint selection, early stop, tuning or partial-artifact fallback. This
explicitly continues, rather than restarts, the original 12-epoch rate/order schedule.
Same original 8192 feature/hash, legal masks, f32 weights/f64 sums, greedy first-legal
strict-> ties, operations, six-step horizon, Rational and numeric comparator.
Each arm <=600s/8GiB/0swap; model <=16MiB; outer experiment <=1800s.

Before training: authored commutation/ordered-op/distinct-equal-input/duplicate-ref/
constant-normalization/no-association/no-cancellation/forward+missing-ref contracts;
independent exact-rational execution of allowed transformations over multiple
positive/negative/zero assignments and division failures. Numerical coincidence
never defines targets.

Evaluate original/A/B on every4590 FIT and1182 CAL, save full native records.
Free exactAST/numeric/refused/wrong; teacher exact left/right plus equivalent left
membership in V, and equivalent right membership in V(gold_left), denominators15753
FIT/4138 CAL. Whole-row exact/equivalent operands require all gold-context decisions
pass; also record predicted-left paired equivalent grounding separately. Oracle
supplies gold complete operation plan AND length, predicts args/prefix, forces stop
or horizon; report exactAST/numeric/refused/wrong and termination counts.

ALL continuation gates: B CAL free >=168 and oracle >=251, each strictly greater
than A; equivalent teacher left AND right rates >=3 percentage points above both
original AND A (integer minimum gain125/4138 for each comparison). Failure of any
gate rejects hypothesis. Record every original-correct loss and refusal->wrong
transition for both arms/splits/modes. No selection exists; zero confidently selected
answers. Numeric equality does not establish English semantics. No DEV even if all
pass; independent parent review required. Freeze sources, patch, commands, original
and final weights, full records, checks, failures and per-file hashes externally.
