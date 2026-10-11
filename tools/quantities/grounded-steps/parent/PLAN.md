# Preregistered TRAIN-only source audit

Three learned MathQA configurations failed their continuation/acceptance gates.
Change data/supervision rather than tune the same ranker. Hypothesis: verified
calculator annotations in GSM8K official TRAIN provide more directly grounded
compositional arithmetic supervision. This first batch audits that proposition;
it does not train a model, run held-out evaluation, or change runtime behavior.

Fetch only official openai/grade-school-math TRAIN and LICENSE at pinned commit
3101c7d5072418e28b9008a6636bde82a006892c; cap each response12MiB. Never fetch,
list or inspect official test items. Official TRAIN remains TRAIN. A deterministic
normalized-question hash group partitions FIT/CAL for future TRAIN diagnostics,
not an official or independent development benchmark. No test-to-train relabeling.

Before corpus conversion, freeze exact supported numeric binding, rational
expression grammar, annotation/final-answer validation, constant inventory and
ambiguity policy. Strictly separate all arithmetic-valid annotations from the
subset uniquely grounded to original question occurrences and earlier steps.
Repeated equal values and prior-step/input collisions are explicit ambiguity;
never silently choose a positional occurrence, derive missing constants from
answers, or smuggle rationale text into runtime input. No eval/exec of input.
Report exclusion counts over the full denominator, arithmetic/final consistency,
coverage by step/operator/number count, and group membership with zero leakage.

Parent-authored adversarial contracts freeze independently before candidate
inspection. Every accepted converted program must execute in the original native
quantity executor to the annotation's final numeric value, with exact source and
native input identities retained. Question-only current solver baseline is useful
but not a model-selection gate. No learned training, tuning, DEV, teacher calls or
runtime replacement in this batch. Continuation needs at least1000 uniquely
grounded exact programs and no failures on the independent converter contracts;
this is a data-readiness gate, not solver accuracy or shipping acceptance.
