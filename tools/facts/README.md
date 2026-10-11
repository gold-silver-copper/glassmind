# Facts from a finite Wikidata snapshot

`glassmind-facts` links the longest unambiguous English entity alias, proposes a
relation using the existing sparse margin classifier, and follows an indexed
Wikidata claim. It refuses missing or multiple answers. Reverse relations use
only edges present in the snapshot; this is not a complete inverse Wikidata index.
There is no generative model or network lookup at runtime.

SimpleQuestions-Wikidata revision `df69267f1db4f85a71325e7e8c06e192bcaf0b0e`,
CC BY 3.0, provides 19,481 training and 2,821 validation questions in the
`*_answerable` files (including reverse relations). Training uses 17,532 rows;
every tenth row, 1,949, calibrates refusal. Validation never fits the model.
No test data was downloaded. Dataset sizes and hashes are in `source.json`.

The **500-entity index is benchmark-scoped**, seeded by validation subject IDs,
not an open-world entity-retrieval benchmark. Validation relations and answers
are not inserted into the index: labels, aliases and claims were independently
retrieved from Wikidata on 2026-10-10 (CC0). The API rate-limited the crawl, so the
cached partial snapshot is deliberately retained and versioned in `index.json`.
Missing entities remain in the full validation denominator. Current Wikidata
claims may differ from the historical benchmark. Provenance hashes cover all
20 raw responses (27,696,032 bytes), retained in the local data cache.

The first 200 validation rows form the fixed sample: initially 12 right,
183 refused, 5 wrong, then 12/186/2 after refusing multi-answer claims.
The latest acronym-casing guard gives 12/188/0 on that sample and full
validation **256 right, 2,545 refused, 20 wrong / 2,821**, preserving all
previously correct answers. It withdraws false lowercase acronym matches
without exposing a different candidate. [Exact transitions and limitations](continuous/HANDOFF.md).
Entity linking is correct on 504/2,821, and uncalibrated relation top-1 on
2,664/2,821. These diagnostics are not answer accuracy. The initial full dev run
had 169 wrong; 130 were multi-answer outputs, now explicitly refused. No official
held-out score is claimed.

Reproduce using the fixed checked-in index (model weights remain in the cache):

```sh
DATA="$HOME/.cache/glassmind/data/simplequestions"
python3 tools/facts/prepare.py "$DATA" --dataset-only
export TMPDIR="$HOME/.cache/glassmind/facts-tmp"
mkdir -p "$TMPDIR"
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  cargo build --release -p glassmind-facts -j1
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  target/release/glassmind-facts train --train "$DATA/train.tsv" --model "$DATA/relation.json"
target/release/glassmind-facts score --input "$DATA/valid.tsv" \
  --model "$DATA/relation.json" --index tools/facts/index.json
target/release/glassmind-facts ask --model "$DATA/relation.json" \
  --index tools/facts/index.json "where was Douglas Adams born?"
```

The example may refuse because the snapshot is finite. To expand the index,
run `prepare.py` without flags; it uses one request at a time and backs off on
429 responses. A changed live snapshot needs a new measurement and hash.
