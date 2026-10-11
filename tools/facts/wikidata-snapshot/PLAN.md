# Independent Wikidata snapshot pilot

Frozen before acquisition or evaluation, 2026-10-11. This replaces the rejected
TRAIN-selected forward-anchor expansion; it does not relax that experiment's gate.

Hypothesis: an independently published, substantially broader Wikidata snapshot
can supply factual coverage without selecting subjects from QA labels. First
test acquisition and import feasibility, not answer accuracy. No TRAIN, CAL,
DEV or held-out questions, answers or entity IDs enter this pilot.

Source: `permutans/wikidata-{claims,labels,aliases}` on Hugging Face, derived by
`lmmx/wikidata-pq` from the official Wikidata 20260928 JSON dump, CC0-1.0.
This release excludes scholarly works (the publisher distributes those separately).
The README's older 17.7 GB claims estimate is superseded by the pinned current
file tree: 86 files, 44,633,986,614 bytes and a stated 789,435,443 rows.
Claims preserve statement IDs, ranks, qualifiers, references and unknown/no-value
snaks; do not silently equate all statements with current unqualified truth.

Pin the API-reported repository revisions and LFS SHA256 for every acquired file:

- claims `39e6837bef8b6f982fbf23acd19134e882400db7`;
- labels `51005b1a1495beae6371838584ad73eaf170ebc9`;
- aliases `6ac11164399601dc6cc24c39a65d03b90c4dccc7`.

Acquire only the lexically first claims shard (`all/part-00-of-86.parquet`),
all English labels and aliases, and the `mul` default-language names. Selection
uses file paths and language only. Expected compressed size: 1,475,606,067 bytes.
Keep raw files outside git. Bound total received data to 3 GiB, each request to
60 seconds without progress, at most two attempts/file, total acquisition to
1800 seconds. Hash-verified completed files can be reused; preserve failed partials.
Do not download remaining claims until this pilot has passed and a separate plan
has fixed the larger acquisition and evaluation gates.

Use a local pinned PyArrow 25.0.1 environment and bounded batches. Import all
well-formed Q-to-Q, P-labelled, normal/preferred, value-bearing claims from the
one fixed shard into a disk-backed SQLite pilot. Keep ranks and a flag for
qualifiers rather than declaring qualified claims timeless. Import all English
and default-language names, including ambiguity, with normalized whole-token
keys. No QA-derived property or entity selection. This is an acquisition/storage
experiment, not a replacement runtime bundle and not a complete graph.

Pilot acceptance: all downloaded bytes match publisher hashes; exact source row
accounting; deterministic schema with no silently discarded malformed entity
claims; SQLite integrity and indexed forward/inverse/name lookups pass; native
resource cap 8 GiB RAM, no swap; final pilot database below 12 GiB and import
under 30 minutes. Measure actual cost before extrapolating to the full graph.
Verify representative importer rows directly against raw Parquet and report
subject/edge/name counts, rank and qualifier exclusions, first/last subjects,
timing and peak memory. There is no QA score or shipped capability claim yet.

Sources:
- https://huggingface.co/datasets/permutans/wikidata-claims
- https://huggingface.co/datasets/permutans/wikidata-labels
- https://huggingface.co/datasets/permutans/wikidata-aliases
- https://github.com/lmmx/wikidata-pq
- https://arrow.apache.org/docs/python/parquet.html
