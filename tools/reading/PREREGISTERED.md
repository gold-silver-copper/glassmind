# Fixed experiment, 2026-10-10

Written before implementing or evaluating the model. Base source:
ed81553d1d2be8b1dbe2df6228bd5f560d510170.

Hypothesis: full BabyLM2025 strict-small training text supports a classical
byte 6-gram scorer that prefers more grammatical dev sentences and has lower
dev bits per byte than its add-half unigram baseline.

Fixed settings: all six train_10M archive members; no subset; byte alphabet
0..255 plus EOS=256; BOS=257 context only; five BOS context symbols at each
document start. Each LF-delimited training line is a document; remove LF
and a preceding CR only. Preserve other bytes, case, spaces and empty lines.
No vocabulary fitting, pruning or dev training. Order 6, recursive
Witten–Bell interpolation, unigram pseudocount 0.5 per predicted symbol.
Unseen contexts back off exactly. Score EOS once; BOS is not scored.

Baseline and candidate use the identical counts/artifact, with orders 1 and
6 respectively. Pair preference uses total negative log2 probability,
absolute difference <=1e-9 is a tie. Reading metric sums negative log2
probability including EOS and divides by total UTF-8 text bytes excluding
EOS; empty documents still contribute their EOS cost. No length dropping.

Pipeline check: first 20 grammar rows and first 12 reading rows in the
explicitly authorized frozen dev files. Then evaluate all 1340 pairs and
all 120 documents. Acceptance: strictly more good preferences AND lower
byte-weighted bits per byte than unigram on full dev, with full denominators
and all regressions reported. Do not retune after seeing sample/full results.

This is a separate probability metric, not the historical parse probe
(1 good / 3 bad / 1336 ties), not semantic understanding, and not held-out
performance. Keep corpus and model in the worker run cache. No new LLM calls,
generation, official test access, agents, shared scorer edits or commits.
