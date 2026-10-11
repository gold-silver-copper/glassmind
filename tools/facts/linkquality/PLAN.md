# Preregistered single experiment — 2026-10-11

Source 176dc89ee9ce0ad2302474f03683334e37514b76. Design frozen before edits
and before reading dev failures. Baselines reproduced with frozen score binary:
old500 first200 12/188/0; full2821 256/2545/20. New999 first200 3/191/6;
full2821 47/2712/62 (right/refused/wrong).

TRAIN-only evidence: existing answered new999 predictions have 701 correct /
669 incorrect entity links on non-calibration rows; every-tenth calibration
has 14 correct /72 incorrect. Thus learned selection's preregistered 95%
precision and >=50 accepts is mathematically impossible (at most14 correct).
No learned model fitted, threshold chosen, or target relaxed. Choose the
permitted nonlearned typed-constraint experiment instead.

Hypothesis: an alias immediately following interrogative determiner WHAT or
WHICH occupies a requested-type slot rather than a referential subject slot.
The representation is a token-span boundary role, not entity identity or an
alias blacklist. The two tokens are grammatical determiners, not learned
entity vocabulary. This is deliberately narrow; complete noun-phrase parsing,
partial full names/titles elsewhere, literal boundaries, question-type semantic
compatibility and inverse-index completeness remain unsolved.

Frozen semantic rule: compute occurrences of all normalized aliases of the
already-selected entity having the longest matching normalized byte length.
If EVERY such occurrence is immediately preceded by token `what` or `which`,
withdraw an existing non-refused answer. Otherwise preserve it exactly. Apply
only AFTER original linking, ties, acronym casing, relation and claim checks.
Never re-link, rank, fall back, resolve ambiguity, or change a retained claim.
Absent optional constraint preserves default behavior. No fitted parameters.

Before dev: audit all19481 TRAIN outcomes and entity labels offline. Reject if
any new/changed claim or any correct entity link is removed. Then evaluate
fixed first200 followed by full2821 for BOTH indices. Full acceptance: no
changed/revived claims; >=244/256 old500 and >=45/47 new999 baseline scored
correct retained; old wrong<=20 and new wrong<=52. Report every row and exact
counterfactual entity/relation/answers; explicitly report scored-right losses.
No design changes after dev; failing experiment is archived and runtime restored.

Source scope: old500 is validation-subject-seeded; new999 selected by first90%
TRAIN minus every-tenth calibration, actual999/1000 after redirect exclusion.
Neither is open world. Report repeated TRAIN/cal/dev subjects and phrases.
No downloads, held-out access, runtime Python, answer tables, LLM calls,
services, background jobs left alive, commits, or changes outside ownership.
