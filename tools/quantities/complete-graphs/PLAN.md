# Complete question-grounded operation graphs: prospective pilot

Frozen before selecting or inspecting this batch, 2026-10-11. Worktree
WORK/quantity-graphs, branch continuous/quantity-graphs. External artifacts belong
in WORK/coordinator/quantity-graphs. No production changes or fitting in this pilot.

The exact-occurrence, fixed-calculator-graph replication was rejected at 95/128
against its unchanged minimum96. Root found no contradictions in its accepted
links, but 12 refusals involved missing/unrepresentable calculations and six involved
exact-occurrence or broader-permutation ambiguity. Other refusals concern premise
and target problems. Preserve the rejected result and its original outputs.

Hypothesis: a complete operation graph grounded in the question, with explicit
references joining coreferential numeric mentions, can provide reliable training
supervision without inheriting missing nodes or misleading intermediate descriptions
from the reference rationale. This is a representation change, not another attempt
at the rejected annotation policy or a relaxed minimum. It is not solver accuracy.

Selection: reuse the same pinned official GSM8K TRAIN source, frozen converter
archive and native executor. Reverify all source/archive hashes and native bindings.
Select the next32 distinct TRAIN-FIT groups after the already selected first160,
sorted by(group_sha256,row_id), with sole original reason ambiguous_leaf. No row
replacement, new source, CAL/DEV inspection or official held-out access. This is a
controlled pilot on the same eligible population, not a broader dataset claim.
Freeze the selected batch before teacher execution. Do not claim a paired gain over
the previous128, which contain different questions.

The teacher receives only the32 selected questions, TRAIN reference answers and
rationales, and exact question-only numeric spans. Original calculator nodes are
not constraints or teacher inputs. The question governs every accepted premise.
A valid new graph may explicitly supply calculations absent from the old annotations,
choose a coherent equivalent calculation, or disregard a misleading intermediate
rationale description. It must record such rationale issues. If the final reference
answer is wrong, or the question lacks a justified interpretation, refuse and explain;
do not silently repair the question or train against a corrected label in this pilot.

Representation: a row contains either one fully justified native-compatible graph
or a nonacceptance status. Its quantities name used question inputs, with meaning,
unit and exact UTF-8 question evidence. Each quantity has one or more question-number
indices. Multiple indices are allowed only when their mentions denote the same
quantity/entity/state/unit, not merely the same numerical value. Values must agree;
groups must be disjoint and every listed mention must be cited. Explain the semantic
coreference of a multi-mention group. The compiler chooses the lowest occurrence
only after a group is justified; it is not a numeric-source preference rule.

Each operation identifies two sources: a declared quantity, an earlier step, or
an existing native constant. Every operand has a role and evidence; constants need
question evidence licensing their lexical, structural or unit-conversion meaning.
Every step has an output meaning/unit and an exact rational value. Derived values
must be computed, not hidden in constants. Every step must contribute to the final
answer, and every declared quantity must be used. Descriptive unit strings are
review evidence; do not claim an automatic semantic type proof from their presence.

Native constraints remain unchanged: at most16 bound numeric occurrences, at most
six steps, add/subtract/multiply/divide, the existing20 constants, past-only step
references, exact arithmetic, and the last step as result. A quantity group compiles
to the existing Input argument; this annotation format is not a runtime resolver.
No added operators, number-word binder changes, larger search bounds or LLM calls
inside glassmind. No model fitting or learned accuracy claim in this pilot.

Before teacher execution, freeze the strict schema, compiler/mechanical validator
and authored contracts. Cover UTF-8 boundaries, missing/duplicate IDs, duplicated
or overlapping mention groups, unequal-value coreference, missing group evidence,
future refs, unjustified/new constants, altered final answers, dead steps/quantities,
valid missing-step reconstruction, valid repeated-mention coreference, and an
explicit mechanically passing equal-value distractor that requires semantic review.
Reuse prior failures only as declared development contract examples, not as unseen
validation data. Freeze the final contract count and hashes before the teacher.

Parent preparation: read the first eight selected questions without their answers
or rationales, and freeze semantic references and manually expected outputs first.
For each acceptable case, freeze at least two discriminating numeric mutations
(or one when only one meaningful input exists). If a quantity has coreferential
mentions, mutate them together; do not turn a consistent question into an accidental
contradiction by changing only one repeated statement. Also freeze required refusals
for defective or underdetermined questions. Only after this reference freeze compare
the parent outputs to source labels and record any label conflict. References and
all other source rows stay masked from the teacher. These are TRAIN reviews, not
held-out evaluation or proof of arbitrary counterfactual completeness.

Exactly one teacher session, GPT-6 Astra fast/high through the existing ChatGPT
login, API-key environment scrubbed, forced ChatGPT authentication, no fallback or
nested workers. Verify local CLI support and isolated login without reading
credentials. Only selected batch/protocol and its output directory are exposed;
full TRAIN/CAL, parent references and project/evaluator trees remain masked.
Cap at1200seconds,8GiB/no swap,2MiBannotations,256MiB writable teacher directory,
and use on-disk temporary storage. Preserve and hash originals before review or
cleanup; remove only owned auth links after terminal scope verification. A failed
session consumes the single attempt. No repair or retry selected after its output.

Acceptance: all rows accounted for; all frozen protocol contracts pass; accepted
native graphs and declared intermediate arithmetic are exact; at least24/32 are
parent-confirmed justified graphs; zero teacher-accepted semantic contradictions;
all eight parent references and their native counterfactuals agree. Structural
identity to one parent graph is not required when another fully reviewed graph is
semantically equivalent. Root reviews every quantity group, source role, constant,
operation, target and evidence citation. Record every exclusion, actual usage and
resource outcome. Mechanical or final-number agreement alone is never approval.

If accepted, freeze a separate replication and eventual training/data-size design
before selecting more data, fitting or inspecting CAL. If rejected, retain the
failure and change the hypothesis rather than adjusting this minimum. The earlier
29/32 pilot, rejected95/128 replication, and rejected354/7473 value-only converter
remain separate historical experiments with unchanged gates and no runtime gain.
