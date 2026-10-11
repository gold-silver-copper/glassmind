# Frozen experiment (before implementation, training, or DEV)

Hypothesis: TRAIN-supervised local word/tag/stack/child features learn useful
basic dependency direction and labels beyond a fixed previous-word chain.
One candidate only; no dev retuning, calibration or threshold search.

Source: UD English EWT r2.18 b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c,
CC-BY-SA-4.0; source README and license read; only first THREE TRAIN sentences
inspected, plus aggregate stats in train-inspection.json. No test obtained/read.
Read-only source directory supplied by coordinator. TRAIN 15,034,493 bytes,
DEV 1,898,574, README 24,522, license 20,014. Exact hashes in source provenance.

Representation: greedy averaged-perceptron UPOS tagger, two ordered TRAIN epochs,
then labeled arc-standard parser, three ordered TRAIN epochs, static oracle.
Sorted label inventories derived from TRAIN only. Parser trains on frozen
predicted tags from the tagger, never gold tags; tagger learns gold TRAIN UPOS.
No gold lemmas/features/enhanced edges used. No random shuffle or seed.
Tag features: lowercase/current/context forms, prefix/suffix 1..4 Unicode
characters, case/digit/hyphen shape, preceding two predicted tags.
Parser features: top three stack / next three buffer words and predicted tags;
word/tag conjunctions, tag triples, distance, existing left/right child words,
tags and labels, valency. Deterministic FNV/mixing feature hashing; tag table
2^16 buckets and parser table 2^17 buckets; dense bounded native weights,
sparse sorted binary persistence. f32 weights, lazy f64 totals, integer clocks.
Action ties select first legal action: SHIFT, sorted labeled LEFT, labeled RIGHT.
Root attachment only final RIGHT(root), no other root labels. Exactly 2*n
transitions; no cycles/selfheads. TRAIN malformed/nonprojective/overlimit trees
skip parser training with counts; no DEV skipping from denominator.
Maximum 128 input forms, total 16KiB; no empty/whitespace forms. Model <=128MiB,
finite weights, exact dimensions/inventory/version and sorted sparse keys
validated before use, buffered bounded loading. Strict JSON input forms OR text,
no annotation/ID fields. Explicit refusal on unsupported input/invalid structure.

Raw tokenizer: Unicode alphanumeric plus combining marks in contiguous word
runs; each other nonwhitespace scalar separate (including apostrophe/hyphen).
UTF8 byte offsets retained. No linguistic clitic/multiword expansion; native
UD integer forms are a GOLD TOKENIZATION experiment, distinct from raw text.
CoNLL-U range/decimal rows skipped and counted explicitly.

Acceptance: ALL DEV integer words incl punctuation, unsupported counts as misses:
LAS >=45%, UAS >=55%, both strictly above fixed previous-word baseline
(first word root, others previous word with dep label). Every emitted prediction
must be structurally valid with exactly one root. Report coverage and separate
punctuation-excluded diagnostic (gold UPOS != PUNCT). Nonprojective and long DEV
remain in primary denominator. Audit exact TRAIN/DEV duplicates by both forms
and complete integer annotations, report duplicate/nonduplicate metrics without
removal. Fixed first100 in original order, then full DEV after binary/model/
source freeze. No repeated model selection. No scores comparable to old grammar,
BliMP, SQuAD, calibration, facts or QA. Baseline deliberately weak, not SOTA.
Authored six active sentences require both core labeled edges correct in >=4/6
(stricter explicit reading of core-edge gate; also report individual edges/12).
Other authored contrasts retain actual errors, no extra tuning. Failure archives
candidate sources/model/reports and restores runtime exports/files.

CPU/memory/time recorded. Training one native invocation <=600s, <=5 total TRAIN
epochs; no downloads/dependencies/model calls. Cargo scoped8GiB/0swap, setsid wait,
600s timeout, TMPDIR external, existing target, -j1/tests4. Minimum relevant Rust
checks plus unchanged QA/byte tests and fmt/clippy/build as feasible. No commits.
