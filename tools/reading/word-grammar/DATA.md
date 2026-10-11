# Reused data and limits

No new data downloads or dependencies: 0 external bytes. TRAIN is the mounted
BabyLM 2025 strict-small10M archive, not the later challenge data. No upstream
commit ID is supplied; version is pinned by SHA-256
`6d9d2cb3b3feccf0669e74173edb55de6c7f12412ee02935d16b1b6b44160b39`.
Archive: 18,139,949 bytes. Mounted README: 3,589 bytes,
`20d4993f0f36a965add35cbf0554cbb42b131f60f29bd80db16ab56a3f40c9c0`.
All six extracted TRAIN members: 54,351,399 raw bytes, reused without extraction.

| Member (training order) | Raw bytes |
| --- | ---: |
| bnc_spoken.train | 4,884,146 |
| childes.train | 15,485,295 |
| gutenberg.train | 14,001,510 |
| open_subtitles.train | 10,828,244 |
| simple_wiki.train | 8,432,882 |
| switchboard.train | 719,322 |

Individual hashes and actual verified absolute paths are in verified-inputs.json.
The dataset's license is mixed. Reuse follows the previous corpus documentation
[../LICENSES.md](../LICENSES.md), whose recorded primary-source review this batch
has not repeated: BNC User Licence/fair dealing; CHILDES TalkBank terms with
CC BY-NC-SA 3.0 except otherwise indicated; Project Gutenberg conditions;
SimpleWiki Wikimedia terms with per-snapshot/per-document qualifications;
OpenSubtitles and Switchboard precise blanket redistribution grants unresolved.
The repository MIT/Apache license does not relicense TRAIN or derived count models.
Weights/corpus remain in the run cache, not in this source submission.

Authorized grammar dev: 210,468 bytes, 1340 rows, SHA-256
`37ad0a1eba3a9cd9c5eae850973c376b5e32a7968ecb95833eb2d32115f6304b`.
The previous provenance records original BLiMP master archive SHA-256
`8c043d41701fff01cabf22fb9a2fd8e15cecac5240b777ff41680f4eb7fffc17`.
This batch uses only the supplied original public dev JSONL; it does not access
that archive, new official BLiMP items, held-out data or hidden evaluation code.
No new grant or source revision beyond these recorded identities is asserted.

Reading dev was hash-verified only: 14,854 bytes, 120 rows, no candidate scoring.
Historical byte BPB 2.1892595993 and known 33/120 exact TRAIN-line overlaps stay
unchanged. No new grammar-overlap study is claimed.
