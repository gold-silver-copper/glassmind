# Token endpoint completion rejected before DEV

The frozen TRAIN gate rejects this representation: all 50 generated extensions
disagree with source annotations (46 FIT, four CAL). None of the 1,635 candidate
key/length rules qualifies. No runtime change or candidate DEV evaluation occurred.
The approved v2 model and its previously measured 610/1,119 correct joint turns
remain unchanged; that DEV score was not measured again for this rejection.

The coordinator verified the original 30-file, 1,677,366-byte archive, all 28
owned-file manifest entries, ten external artifacts, seven frozen identities,
four source-data identities and 29 restored dialogue files against the original
commit and current checkout. The archived runtime patch is empty. Original
repository copies were rechecked before removing the inactive worktree and
temporary login link. Raw evidence remains in
`/home/gsc/.cache/glassmind/continuous-20261010/tokenendpoints-run`.

An independent audit recomputed source locations, UTF-8 byte boundaries,
annotation endpoints, conversation partitions and every rule's acceptance
conditions from all 165 retained reference proposals. It reproduces the gate
exactly. Of the 164 copies without existing punctuation correction, 158 already
have the complete annotated source endpoint; six lack a matching annotation.
There are no annotated truncated source values in this actual proposal set.
The 50 extensions include two missing-annotation cases, counted as mismatches.
The source result is a post-rejection diagnostic, not a new selection criterion.

The diagnostic Rust source follows the production reference call site: snapshot
prior predictions, advance the service, consider only missing slots in active
services, apply the existing punctuation rule, then assert equality with actual
inference. These assertions passed in the retained native replay. Parent review
did not repeat that inference; it checked the code and replay identities and
performed an independent offline endpoint audit. The original classifiers saw
all TRAIN, including CAL, so this is empirical new-rule validation, not independent
probability calibration or proof of referent correctness.

Five authored offline contrasts pass again; shell syntax passes. Original eight
Rust tests and diagnostic build passed. No new dependencies or downloads. SGD
source revision `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0; retained
derived annotation data keeps that license. All seven inherited wrong slot-turns
remain unresolved. This is the third rejected span-proposal hypothesis; further
work must change the representation or track rather than adjust this gate.

Reproduce the independent audit from the repository root:

```sh
python3 -B tools/dialogue/token-endpoints/parent-audit.py \
  --train /home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/data-sgd-full/selected/train.json \
  --reports tools/dialogue/token-endpoints/reports \
  --output /home/gsc/.cache/glassmind/continuous-20261010/coordinator/tokenendpoints-review/independent.json
```

The original [handoff](HANDOFF.md), [frozen plan](PLAN.md),
[identity audit](parent/identities.json) and [independent counts](parent/independent.json)
retain the full distinction between observations, unchanged baselines and rejection.
