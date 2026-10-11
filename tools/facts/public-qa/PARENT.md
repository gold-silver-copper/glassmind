# Public first-shard QA comparison: promotion rejected

The frozen 1,949-row every-tenth TRAIN calibration experiment did not pass its
promotion prerequisite. No DEV inference followed, no threshold or alias rule
changed, and the public graph did not replace a default bundle. The native
importer's storage/query validation remains valid; it was never a QA accuracy claim.

| Fixed CAL scope | Strict entity/relation/answer right | Refused | Wrong |
|---|---:|---:|---:|
| Retained TRAIN999 reference | 2 | 1,863 | 84 |
| Independent public first claim shard | 2 | 1,940 | 7 |

The required net gain was ten strict correct answers with zero newly wrong
baseline-right/refused cases. Actual gain was zero and seven old refusals became
wrong under the exact annotation metric. Both prior strict correct answers became
safe refusals. The two new correct answers came from one prior refusal and one
prior wrong answer. All 1,949 rows remain accounted for.

Answer-only counts were 26/1,863/60 for the reference and 2/1,940/7 for the public
pilot. All 24 coincidental baseline answer matches have the wrong anchor or
relation; the unchanged strict audit does not credit them. Neither the historical
scorer nor this experiment's criteria were redefined.

## What the diagnostic establishes

The native index contains an eligible alias for 1,946 annotated anchors. Correct
links increased from 153/1,949 to 843/1,949; linked predictions total 594 and 1,013,
respectively. Correct anchor-and-relation proposals increased from 146 to 792.
Registry coverage, linking and successful answers are distinct measurements.

For the 1,946 present anchors, supplying the annotated relation only as an offline
diagnostic gives 1,642 missing local claim sets, 296 nonempty sets without the
annotated answer, six unique annotated answers and two nonunique sets including
it. This one lexical shard has very little of the required claim coverage despite
the broad name index. This oracle diagnostic is not model accuracy or evidence
of global inverse uniqueness.

All seven newly wrong predictions have the correct annotated anchor and inverse
relation. Independent database checks confirm that each returns its sole local
inverse match while the annotated answer is absent. The two lost correct forward
answers have no corresponding claim in this shard. These findings explain the
fixed metric failures; they do not establish that every alternative recorded
answer is globally false or justify changing the annotations.

## Reproduction and bounds

The [plan](PLAN.md) was frozen at366b058 before benchmark inference. Evaluator
source7645195 uses the existing `joint-audit.audit` unchanged. It checks full TRAIN
question order and input hashes, runs only extracted question text through a
pinned CLI/model/graph, freezes predictions and their hash, then joins annotations
offline. Full per-row evidence stays in
`WORK/coordinator/wikidata-qa/report.json`; `WORK` is
`/home/gsc/.cache/glassmind/continuous-20261010`.

The run completed in6.542 seconds under600 seconds/8GiB/no-swap, with a120-second
inference bound. Added run files were32,187,379 bytes, below256MiB. No external model
calls, workers or downloads occurred. [inputs.json](inputs.json),
[artifacts.json](artifacts.json), [summary.json](summary.json) and
[failure-diagnosis.json](failure-diagnosis.json) preserve provenance and checks.
This is reused development CAL, not held-out performance or a fresh test of the
pre-existing relation model. Dataset source is CC BY3.0; public Wikidata is CC0.

The next useful step is broader independently selected claim coverage with a
bounded ingestion design that reuses the already-correct native alias index.
Another alias-only filter cannot create the absent triples. Do not acquire
CAL-selected entities, lower this experiment's gate, or run DEV for this rejected
candidate. Freeze a new acquisition/resource plan before downloading more shards.
