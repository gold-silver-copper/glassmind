# Explicit source-reference annotation pilot

Frozen before selecting the batch or requesting annotations, 2026-10-11.
This is a TRAIN-label quality experiment, not a solver or runtime promotion.
Root owns WORK/span-references on branch continuous/span-references. Large
artifacts stay in WORK/coordinator/span-references. No production edits.

The rejected GSM8K converter recovered 354/7473 uniquely value-grounded programs,
below its unchanged 1000-program gate. There were 2183 rows whose only exclusion
was ambiguous leaf sources. An equal value does not identify a question occurrence,
prior calculation or justified constant. Previous marginal operand-ranker gains
also failed whole-program checks. Do not repeat numeric-source priority or a
surface-context ranker configuration.

Hypothesis: explicit semantic source references, supported by exact text evidence,
can resolve useful training examples that numerical agreement alone cannot.
Use the existing official GSM8K TRAIN source/revision/license and frozen converter
records. Verify original artifact hashes, source memberships and native bindings.
Select exactly the first 32 TRAIN-FIT rows sorted by (group_sha256,row_id) whose
sole recorded reason is ambiguous_leaf. Keep at most one row per normalized group.
Do not inspect or select TRAIN-CAL, DEV, official test or held-out material for
this pilot. Do not replace difficult selected rows. Select before annotation.

The teacher receives only these selected questions, original TRAIN rationales,
question-only numeric spans, parsed calculator trees and complete source choices.
It must keep every original operation and explicit subtree edge; no simplified
programs, changed answers, rationale-only inputs or new constants. Every leaf
assignment needs an explicit input occurrence, prior-step reference or justified
native constant, with exact quoted text and UTF-8 evidence spans. Explain the
semantic role connecting that source to this operation. Numerical agreement,
source order and a generic preference for inputs are not evidence of identity.
Question evidence governs validity; a rationale that contradicts or adds missing
premises must be flagged instead of treated as authority.

Output status is resolved, equivalent, ambiguous, unsupported or label_error.
Resolved has one fully justified graph. Equivalent may retain up to four complete
correlated programs differing only by direct operand swaps at add/multiply nodes;
never emit independent leaf choices whose cross product admits wrong references.
Other genuine reference ambiguity must remain ambiguous, even when all choices
produce the annotated number. Subtraction/division operand order remains fixed.
Explicitly report missing textual justification for constants or source identity.

Before teacher execution, freeze a strict output schema, mechanical validator,
and authored validator contracts (including bad spans, future refs, constants,
correlated alternatives, and contradictory/missing premises). Also freeze parent
reference/counterfactual expectations for the first eight selected FIT questions
in the same selection order, without showing these expectations to the teacher.
These are permitted TRAIN reviews, not a held-out test. Counterfactual changes to
relevant versus irrelevant numeric occurrences must distinguish semantic source
identity from an equal-valued constant or distractor. If a selected question lacks
a unique defensible interpretation, record ambiguity instead of forcing one.

One bounded teacher session: Codex GPT-6 Astra fast via the existing ChatGPT login,
no API keys or provider/model fallback. Verify supported CLI/login status without
reading credential contents. Use the established filesystem isolation pattern;
only the selected batch and teacher output directory are visible, with original
TRAIN/CAL data, parent expectations, project/benchmark trees and private artifacts
masked. No nested workers, downloads, fitting, services or unsolicited messages.
Cap the owned session at 1200 seconds and 8GiB/no swap, with on-disk temporary
storage and output at most 2MiB. Preserve failed or interrupted outputs and usage.
No second annotation attempt selected in response to the first result.

Pilot acceptance: all mechanical validator contracts pass; every supplied row
remains accounted for; at least 24/32 receive parent-confirmed resolved/equivalent
labels; zero accepted assignments contradict text or the frozen parent checks.
For every accepted graph, original native execution and exact annotated arithmetic
must match, and the parent must review all source assignments and their evidence.
All eight independently frozen parent cases must agree, including any required
ambiguity/refusal. Report each exclusion, all correlations, exact agreement and
actual model usage. Numeric execution alone is never semantic proof.

If the gate fails, preserve it as a rejected annotation policy. If it passes,
freeze a separate expansion/training experiment before using more data or CAL.
The original converter's failed 1000-program gate stays failed; 32 annotated
examples do not satisfy it or establish inference accuracy. Existing quantity
runtime and published scores stay unchanged throughout this pilot.
