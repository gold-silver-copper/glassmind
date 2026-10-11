# Native disk-backed facts: compatibility passed

A finite JSON store can now be imported into a versioned SQLite file and queried
through `glassmind-facts index-disk`, `ask-disk` and `predict-disk`. Answers and
sets use indexed lookups; the graph stays on disk. This is a storage/latency
improvement on known data, not new factual coverage or QA accuracy.

The same Rust normalization function builds alias keys and normalizes questions.
The format includes its Unicode version and rejects mismatched versions. Original
alias text preserves acronym casing. Queries enumerate bounded whole-token spans
from longest to shortest, keep all equal-length ambiguity, and stop after two
distinct candidate entities. Graph keys remain literal strings; leading zeroes,
IDs beyond integer range and lexical ordering are preserved. Primary keys remove
edge duplicates. Single answers need at most two matching IDs; sets separately
count the full local result before returning the first ten sorted IDs.

All queries use bound parameters and read-only connections. One transaction holds
the same read snapshot for the entire connection lifetime, including metadata.
A test commits an external WAL writer's alias change and verifies that only a
new connection sees it. Input, SQLite row size, cache and instruction-progress
bounds fail with errors, never partial set counts. An exhausted-budget test
verifies error propagation and recovery for the next request. The importer
reserves a new output file without overwrite and commits its schema/data atomically.

## Frozen same-data gates

Source: unchanged TRAIN-selected 999-entity Wikidata graph, 2,876 retained aliases,
35,882 distinct edges. Its JSON SHA256 is
`142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791`.
The relation model remains frozen at
`db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
Hash-verified question-only files and retained predictions are identified in
`inputs.json`. Source questions are the approved SimpleQuestions-Wikidata
TRAIN/DEV data, revision `df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, CC BY 3.0;
claims are the prior Wikidata CC0 acquisition, not QA answer triples.

| Split | Questions | Answer parity | Set parity | Retained old predictions | CLI answer/set parity |
|---|---:|---:|---:|---:|---:|
| TRAIN | 19,481 | 19,481 | 19,481 | 19,481 | 19,481 / 19,481 |
| DEV | 2,821 | 2,821 | 2,821 | 2,821 | 2,821 / 2,821 |

The unchanged JSON implementation and native disk implementation ran side by side;
their complete serialized Answer and AnswerSet objects match. Ordinary answers
also match the preserved original predictor's outputs. The actual streaming CLI
was checked separately in both modes. No labels entered inference, no model was
retrained and no threshold was selected. The full coordinator completed in
20.378 seconds within its frozen ten-minute, 8 GiB/no-swap scope.
The resulting native index is 2,281,472 bytes with SHA256
`1c0450201e9992bf7ee33f77f5a38039d045634efc020e44ff8323b7ac3958cd`.

A fixed first-1,000-TRAIN fresh-process measurement produced identical answers:
JSON baseline 0.476311 seconds / 70,564 KiB peak RSS; native disk 0.109055 seconds /
74,092 KiB. This single local measurement includes model loading and is not a
large-graph memory or general latency benchmark. The disk path used slightly more
memory on this small graph. Earlier coordinator-child RSS numbers include a
large Python launcher's inherited memory floor; the isolated measurements use
fresh small launchers and record their much lower peaks explicitly.

## Review, limits and retained evidence

Focused native tests cover Unicode combining alphabetic marks, punctuation,
acronym support, equal-length aliases, duplicate edges, forward/inverse cardinality,
lexical order, literal long IDs, refusal policies, invalid relations, malformed
JSON/schema, missing/old indices, byte/token limits, read-only reopening, external
writer consistency and VM-budget recovery. CLI tests cover import, direct set
queries, CRLF/blank/unterminated input lines, oversized input and invalid UTF-8.
Existing facts tests and package clippy pass. Workspace and main integration
results are recorded separately after central integration.

Review corrected a pre-integration lifetime issue: the initial implementation
opened one transaction per query. Its already passing parity results, binaries,
source and database are preserved in `first-parity-originals/` outside git. The
final lifetime snapshot version repeated all frozen parity gates successfully.
Initial compilation needed an explicit checked SQLite integer-to-usize conversion;
a clippy nesting correction did not change semantics. The first evaluation
helper stopped before import because GNU time was absent; its original helper
and log are retained. The working helper obtains per-child usage with `wait4`.

The native importer is bounded to 32 MiB JSON. It does not load the independently
acquired 9.65 GiB Python pilot, whose normalized keys are incompatible. A separate
next experiment must stream public source names through Rust normalization and
retain rank/qualifier/source metadata. Conversation and typed temporal/verifying
bundle paths retain their existing implementations. No existing production
model, facts score or default bundle is replaced by this change.

Artifacts: `/home/gsc/.cache/glassmind/continuous-20261010/coordinator/disk-facts`.
`PLAN.md` was frozen before implementation. All Cargo ran with 8 GiB memory,
zero swap, persistent disk TMPDIR, process-group timeouts and one build job;
tests used at most four threads. No model worker, runtime network, external
teacher, API key or held-out data was used.

## Central integration

Source worktree commit `70a446a` integrated as `3de724e`. All 19 facts source,
test, example and manifest/lock identities match between worktrees. Central
release binaries repeat exact streaming Answer/AnswerSet parity for all 22,302
questions. Existing conversation fixtures pass 28/28 in each of plain and
intent-model modes, including persisted memory/story state, default routing,
provenance, invalid bundles and display-only labels. Five existing main facts
unit tests pass. Workspace fmt/clippy/release and the strict four-turn smoke pass.
Hashes and results are in `integration/`; raw predictions remain in the owned
run directory, pinned by `artifacts.json`. The compatibility experiment is
complete. Next work is native ingestion of the independently acquired public
pilot, with its own resource/provenance plan and no new downloads yet.
