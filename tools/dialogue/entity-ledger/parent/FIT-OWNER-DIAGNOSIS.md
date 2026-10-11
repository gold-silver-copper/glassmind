# Prospective source-owner diagnostic after rejection

Use FIT only, without fitting, changing the confidence grid or running DEV.
Deduplicate extracted entities by conversation/source turn/UTF-8 span. Compare
predicted semantic role with exact original SYSTEM span annotations and with
the frozen service activity on the preceding USER turn. Report all source
entities, plus the subset whose annotated role owner was predicted active.

Hypothesis: the unconstrained 128-role classifier often assigns an entity to an
unrelated service even when predicted dialogue context supplies its source
owner. If this occurs frequently while correct roles remain compatible, it
supports a future role model constrained by predicted source context. Counts
are diagnostic upper bounds, not a tested correction or semantic-authority proof.
Do not select or modify this rejected model from these observations.
