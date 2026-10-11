# Frozen prospective word trigram experiment

Written before source edits, training, or candidate dev evaluation. Base commit:
b569ac0eb0bab5a8b9d39cda904c816e77c08e63. One candidate only; no calibration,
no tuning after sample/full, no changes to existing scorers or shared tracks.

Hypothesis: local word trigrams trained on all BabyLM2025 strict-small TRAIN
capture ordering beyond frozen byte6 (707 good / 633 bad / 0 ties / 1340).
Acceptance: >=747 good on exactly all 1340 authorized frozen grammar-dev rows.
Ties receive no good credit. No promise to preserve every baseline preference.

## Fixed model

UTF-8 required. Tokenize original Unicode scalar values: maximal runs of
`char::is_alphanumeric()` form words, lowercased with Rust `char::to_lowercase`;
Unicode whitespace separates and is discarded; every other scalar is a separate
case-preserved punctuation token (including apostrophes, hyphens and combining
marks). No Unicode normalization, stemming, contraction or sentence splitting.
Case expansion happens inside each word after segmentation. No token length cap.
Vocabulary consists of tokens occurring >=2 times across TRAIN only, ordered
lexicographically by Rust string order. IDs: EOS=0, UNK=1, BOS=2, vocabulary>=3.
All discarded/unknown tokens map to UNK; literal special-token spellings receive
no special treatment. BOS context only, two BOS per document; one scored EOS.

Each LF-delimited TRAIN line is a document, removing LF and immediately preceding
CR only (same byte baseline); trailing LF creates no extra document. Include
empty lines. No cross-document histories. Two streaming TRAIN passes: vocabulary,
then all unigram/bigram/trigram counts. Input order exactly: bnc_spoken.train,
childes.train, gutenberg.train, open_subtitles.train, simple_wiki.train,
switchboard.train from existing reading-run/probability. Reuse extracted files;
verify against tools/reading/provenance.json before reading as TRAIN.

Predicted alphabet is vocabulary plus EOS/UNK, never BOS. Unigram prior fixed 0.5
per predicted type: (C(x)+0.5)/(N+0.5*(vocab_size+2)). For context lengths 1 then 2,
P(x|h)=(C(h,x)+T(h)*P(x|suffix(h)))/(N(h)+T(h)). Unseen context backs off exactly.
Score sum of negative log2 probabilities including EOS, no length normalization,
no byte mixture, no generation API. Smaller total cost wins; absolute difference
<=1e-9 ties. Runtime scoring takes text only; IDs/groups are adapter metadata.
Deterministic sorted binary persistence, maximum 150 MiB; 8 GiB/zero swap and
600 seconds per training/build/evaluation job. No settings selected from dev.

## Frozen evaluation and evidence

Verify original byte source hashes, byte6 artifact and retained report baseline.
Save source, binary, model and plan identities BEFORE candidate dev. First 20
original rows in original order, then exactly one full 1340-row candidate run.
The adapter calls the unchanged original byte scorer on the same strings.
Keep per-row baseline/candidate costs/choices, original row number/ID/group and
raw-row SHA256; record whole input hash. Summarize all 67 groups and all baseline
regressions, plus complete good/bad/tie transition matrix. Verify reproduced byte
aggregate and every group against retained report; preserve any failed report.
No post-full tuning. If rejected, archive source/patch/hashes before removing new
runtime files; keep tools evidence and immutable model/binary in own run cache.

Minimum tests: normalization across all predicted types for seen/unseen histories;
deterministic persistence/roundtrip and malformed input; empty/OOV/Unicode and
punctuation behavior. Resource-capped cargo fmt/clippy/build and affected tests,
-j1 except fmt, --test-threads 4 for tests, existing CARGO_TARGET_DIR, non-/tmp TMPDIR.
No full workspace suite. No downloads, model/API calls, agents, commits or services.

## Interpretation

Ranking accuracy only. Noninvertible lowercasing/whitespace/tokenization and UNK
collapse strings; token costs are NOT byte likelihood/BPB. OOV distinctions and
case are lost; local grammar preferences are not semantic truth. Preserve frozen
byte artifact/source/path and historical 120-document BPB 2.1892595993, with known
33/120 train overlap. No new reading evaluation. Only authorized public original
grammar-dev pairs; no official held-out or new BLiMP items. Dataset licenses remain
as documented in ../LICENSES.md; no blanket corpus/model redistribution license.
