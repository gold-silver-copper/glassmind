# Multi-service dialogue experiment: rejected candidates

Two routing approaches were measured on a fixed three-service official dev
selection, then rejected against their declared end-to-end state criteria.
The runtime implementation remains in the local experimental commit
`cb6fb28`; it is not promoted to the main dialogue command.

Services were selected from schemas and training co-occurrences: RideSharing_1,
Travel_1, Weather_1. All 179 matching complete dev conversations across all20
shards remain in the denominator:1,119 user turns,1,218 gold frames, including
54 multi-service conversations. An initial two-service scope had no multi-service
dev conversations and was rejected before scoring. This is a scoped development
result, not whole-SGD or held-out performance.

| Router | Full active-service F1 | Full joint turns | Sample joint turns | Correct switches |
|---|---:|---:|---:|---:|
| Train centroid | 90.07% | 501/1119 | 21/91 | 38/184 |
| Joint trained | 93.20% | 549/1119 | 21/91 | 42/184 |
| Independent trained | 95.62% | 548/1119 | 21/91 | 116/184 |

Full joint success requires the exact predicted service set, correct intents,
and exact slots for every gold frame. All three service frames are predicted
before consulting labels; extra active services count against routing/joint
scores. Both candidates miss their sample state-improvement criterion. The
independent router also loses one full-joint success relative to the joint
router. Each transition adds two right-to-wrong and two right-to-refused turns;
none is hidden. Cross-service references and system-offered values remain
unresolved. All sample exact slot successes are Weather frames.

The frozen single-service command preserves202/257 exact and89.1751%fuzzy.
Reversed-batch replay matches every conversation, and annotation-bearing
runtime input is rejected. Targeted build/fmt/clippy and two focused tests pass.
These checks establish reproducibility and isolation, not a successful state
tracker. [Aggregate evidence](results/dialogue-routing-rejected.json).

Dataset revision `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0:
504,306,737 source bytes, explicit train/dev files only. All150 source files
were hash-verified and preserved at
`~/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full`.
Original code/reports are archived under the same run directory's `submitted/`;
models and predictions are in `tmp/multi/`. The unchanged pinned upstream
SGD metric code remains Apache-2.0. No held-out material was accessed.

A training-only diagnostic of changed noncategorical input values found239
ride-destination values in the previous system utterance, and77 travel-location
values in other selected service states. These counts motivate a new grounded
reference-resolution experiment; they are not dev results or fitted runtime
rules. Any candidate must improve actual joint state coverage on the unchanged
sample/full selection and report every new error.
