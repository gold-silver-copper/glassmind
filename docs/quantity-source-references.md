# Source references for quantity training labels

Explicit semantic source references resolved useful GSM8K training labels, but a
larger replication missed its frozen coverage gate. None of these experiments changes
quantity inference, production behavior, or published answer scores.

| Experiment | Selected TRAIN-FIT examples | Parent-confirmed labels | Frozen minimum | Decision |
| --- | ---: | ---: | ---: | --- |
| Initial pilot | 32 | 29 | 24 | Pass for replication only |
| Separate replication | 128 | 95 | 96 | Reject |
| Complete graph pilot | 32 | 22 | 24 | Reject |

The original value-only converter remains rejected at 354/7473 against its unchanged
1,000-program minimum. These new experiments do not retroactively repair that result.

The source is the existing official GSM8K TRAIN data, 7,473 rows, MIT license, revision
`3101c7d5072418e28b9008a6636bde82a006892c`. The deterministic normalized-question
split assigns groups with hash modulo 5 equal to 0 to TRAIN-CAL; only other groups
were available to these teachers. Selection used unique groups sorted by
`(group_sha256,row_id)` whose sole prior rejection was `ambiguous_leaf`: first 32
for the pilot, then the next 128 for replication. No replacement examples, fitting,
CAL or DEV evaluation were used. Source manifests, sizes and hashes are retained
in the [preparation record](../tools/quantities/span-expansion/prepare-report.json).

An annotation identifies each original arithmetic leaf as an exact question-number
occurrence, a prior computation, or a justified native constant. It includes exact
UTF-8 evidence and a semantic role. Every original calculator node is retained;
missing calculations cannot be invented or hidden in equal-valued constants.
Alternative programs remain correlated complete graphs and may differ only by
direct add/multiply operand swaps. Mechanical checks validate this representation
and exact arithmetic; they cannot prove the source has the required meaning.

Each batch received one isolated GPT-6 Astra fast/high teacher session through the
existing ChatGPT login. Teachers saw only their selected batch and protocol files.
Parent expectations, full data and project/evaluation trees were masked. Each run
had 1,200 seconds, 8 GiB/no swap and a 2 MiB annotation cap; replication also capped its
writable directory at 256 MiB. All five runs completed within limits, with no retries
or API-key authentication. Original outputs were archived and hash-verified before
review. Owned login symlinks were then removed; credential contents were excluded.

Root reviewed every accepted assignment. The pilot checked 202 assignments across
32 native program variants. Replication checked 643 assignments across 105 variants.
Every accepted original native execution matched, with zero accepted semantic
contradictions found. The 34 authored protocol contracts passed. Eight prospective
parent cases per experiment agreed, including required refusals; post-teacher
native original/counterfactual checks numbered 25 for the pilot and 27 for replication.
These are training-label audits, not independent test results or learned prediction.

Replication rejected 33 examples: 12 missing calculations or unrepresentable sources,
6 exact-occurrence or broader-permutation ambiguities,10 missing premises or unclear
target interpretations, and 5 rationale/target contradictions. Some refusals are
conservative readings of ordinary word-problem conventions. They are coverage
losses under this protocol, not proof that every rejected reference answer is false.
For example, repeated mentions can denote the same budget while the format insists
on one occurrence; a valid intermediate calculation can appear in prose but be
absent from calculator annotations. Lowering the minimum would not fix either gap.

A separately frozen complete-graph pilot selected the next 32 unique FIT groups,
allowing complete question-grounded calculations and justified coreferential quantity
references. Native limits stayed at 16 numeric occurrences, six binary operations
and the existing constants. It confirmed 22/32 graphs, below the unchanged minimum
of 24, and is also rejected. There were eight ambiguous and two unsupported rows.
Many exclusions concern conservative readings of ellipsis, frequency or cost scope;
they do not establish ten wrong reference answers.

All 51 new protocol contracts passed. Root reviewed 84 quantity groups, 84 operations
and 168 operand roles, finding zero contradictions in accepted graphs. Every one of
84 native intermediate calculations matched. All eight independently frozen parent
cases and 17 original/counterfactual native checks agreed. One accepted group joined
two mentions of the same twelve-month savings period. Descriptive units and matching
arithmetic remain review evidence, not automatic semantic proof.

The single additional Astra fast/ChatGPT teacher completed in 352.916 seconds within
all caps. Its 14 original files were archived and verified, and the owned login link
removed. It added 287,489 counted tokens (272,802 input including 211,328 cached,
plus 14,687 output), bringing the known completed-worker total to 203,481,375.
[Complete-graph evidence](../tools/quantities/complete-graphs/PROGRESS.md) preserves
all decisions and hashes. No expansion or fitting follows this rejected policy;
future quantity work needs a materially different supervision or learning hypothesis.

The [pilot report](../tools/quantities/span-references/PROGRESS.md),
[replication report](../tools/quantities/span-expansion/PROGRESS.md),
[final decision](../tools/quantities/span-expansion/final-report.json), and per-row
reviews retain the exact gates, provenance and limitations. Known completed-worker
tokens after all five sessions are 203,193,886, including cached input once; root and
interrupted usage remain excluded. The five sessions add 1,267,587 counted tokens.
