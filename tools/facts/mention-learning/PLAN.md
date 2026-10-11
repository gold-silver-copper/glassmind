# One learned mention-context selector: FIT-internal experiment

Freeze before features, split labels or fitting. Builds on candidate preparation
f1df139/720faff/da03980: 17,465 FIT rows after every-tenth CAL and all normalized
question overlaps were excluded. Source and candidate hashes stay unchanged.
There are 9,085 unique groups with a correct eligible span, including 1,400 missed
by the current longest-alias linker. These are oracle upper bounds, not gains.

Hypothesis: a small linear model can distinguish an intended subject mention from
request-language aliases using position, span shape and grammatical context,
without learning entity IDs or names. This differs from the rejected what/which
adjacency veto: weights are learned from FIT, several contextual positions are
available, and confident alternatives can recover an existing missed subject.

Prototype only; no production edits, CAL inference, DEV access, default promotion
or model workers in this stage. Reuse the exact native extracted candidate lists.
Only spans having one distinct, case-supported entity are eligible. Ambiguous
spans stay ineligible; never use gold IDs, claims or relation labels to disambiguate
one. Multiple eligible occurrences of the same entity combine by maximum score.

Features are generated in Rust. Exclude span text, entity IDs, answer IDs and
relation labels. Use three tokens on each side plus the first three question
tokens; retain only the fixed function-word inventory in the implementation, and
map every other token to OTHER (digits to NUMBER). This inventory comprises
interrogatives, auxiliaries, articles, prepositions, conjunctions, pronouns and
possessives, not entity/domain terms. Include word-count and byte-length buckets,
start/end flags, relative position quarter, digit presence, whether the span is a
longest matched alias, and eligible entity-count bucket. No token-frequency search,
feature search, particular question patterns or benchmark identities.

Use the existing native IntentModel with labels mention/other, 20epochs,
char_weight0, min_df2. Every eligible span in training gets the exact annotated
entity equality label; rows without a gold candidate remain negative evidence.
Partition by SHA256(normalized question): modulo5==0 is internal validation,
others train. All duplicates stay together. Keep conflicting-label groups and
report them; no outcome-dependent row removal. No validation features/labels enter
vocabulary or weight fitting. All original FIT rows remain in validation reports,
including rows without eligible spans.

A span's score is the signed prediction margin: positive for mention, negative for
other. Rank entity maxima. A tied top score never overrides the baseline. Override
only when the top margin >=m and its gap above the runner-up (or zero if absent)
is strictly >g. Otherwise preserve the exact native baseline entity, including
None. This is not a refusal-maximizing veto. Search only this fixed internal grid:
m=[0,0.1,0.25,0.5,0.75,1,1.5,2,3,4,8], g=[0,0.25,0.5,1,2]. Choose maximum net correct
entity gain subject to zero baseline-correct/unlinked cases becoming wrong and
zero lost correct links; ties prefer higher m then higher g. Require at least50
net correct internal-validation gains. Freeze selected threshold or rejection.
No model/feature/epoch/grid revision after that report, and no tuned retry.

Require native feature contracts (entity/name invariance, position distinctions,
case eligibility, repeated-entity aggregation and ties), finite scores, complete
row/group accounting, and a byte-identical native baseline reconstructed from
pinned candidates. These internal labels measure linking only, not source truth
or full QA. If this gate passes, freeze a separate CAL/full-QA validation plan
before any new CAL predictions or integration. A pass cannot lift the previously
rejected fullgraph promotion gate or alter the default facts bundle by itself.

Bound complete feature/train/score/report execution at1200seconds,8GiB/no swap,
256MiB added-run, model8MiB maximum, on-disk TMPDIR. Compilation separately600s
with<=4cargo jobs,8GiB/no swap. Pin all scripts, source and executable hashes before
execution. Preserve the one fitted model, original scores and failures. No new
sources, LLM calls, credentials or official held-out access.
