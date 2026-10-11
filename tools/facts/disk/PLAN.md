# Native disk-backed facts compatibility experiment

Frozen 2026-10-11, before implementation. Base160872a; isolated worktree
/home/gsc/.cache/glassmind/continuous-20261010/disk-facts, branchcontinuous/disk-facts.
Root owns this work; no model process or authentication mount is needed.

Hypothesis: an indexed disk-backed store can reproduce the existing finite JSON
store's entity linking and graph answers while allowing a much larger independently
sourced graph to remain outside process memory. The first experiment measures
semantic compatibility, resource behavior and usable native commands, not QA gain.

Use the existing Rust normalization at both ingestion and query time. The storage
pilot's Python keys are explicitly incompatible and must never be loaded as native
keys. Preserve original alias text for acronym casing, longest whole-token alias
selection and equal-length ambiguity. Preserve literal entity/property IDs, forward
and inverse deduplication, lexical ID order, full set cardinality, ten-item display
truncation and explicit refusals. Do not resolve alias ambiguity from answer labels
or change relation weights or acceptance thresholds.

First create/import a versioned SQLite representation from an existing JSON Store
and expose read-only inference through glassmind-facts commands. Retain the JSON
path. Share reasoning code where practical, keep query parameters bound, reject
incompatible schema, and bound input/work/memory with explicit errors/refusals.
No runtime LLM/network, benchmark-specific identities or embedded answer maps.
No existing talk capability may silently disappear; talk integration requires
separate routing/provenance review once the native API is correct.

Data allowed: existing source code/tests, independently authored fictional stores,
retained TRAIN and DEV questions for parity, frozen relation models and JSON graphs.
No held-out items, labels, predictions, hidden tests or generators. Do not read any
unrelated parent fixture directories. No additional public graph download in this
phase. Wider data and strict joint QA acceptance need a later frozen plan.

Gates, fixed before coding:
1. Native tests compare both stores on Unicode normalization (including combining
   alphabetic marks), punctuation/case, equal-length ties, acronym support that
   must not break an existing ambiguity or expose a shorter alias, no match,
   absent claims, duplicate edges, inverse direction, lexical order and set limits.
   Include malformed/schema/over-limit input behavior and read-only reopen.
2. Exact Answer/AnswerSet parity on the same existing 999-entity TRAIN-selected
   graph and frozen relation model, for all19,481TRAIN and2,821DEV questions.
   Question-only native inference; compare retained baseline answer predictions
   when verified. Do not retrain or use labels to select code. This is regression
   equivalence on known data, not an unseen accuracy result.
3. Show lookup queries use indexes and no full graph is loaded for a request.
   A fixed complete parity run must finish within10minutes and8GiB/no-swap;
   compare startup and process RSS against the existing JSON path. Do not enlarge
   limits after seeing failures. All Cargo scoped8GiB/no-swap, disk TMPDIR,
   setsid --wait timeouts, -j1, test threads at most4.
4. Existing facts tests plus relevant main regressions and required workspace
   fmt/clippy/release/strict smoke pass before integration. Independently review
   query completeness, alias and case equivalence, limits and schema handling.

After parity passes, fix a separate plan to ingest the public pilot's raw names
with native normalization, preserve its rank/qualifier/source metadata, and compare
resource behavior at real scale. The existing 9.65GiB pilot is read-only evidence;
do not mutate it. Broad graph acquisition, QA scoring and default promotion are
not implied by storage compatibility. Keep production scores unchanged unless a
new scored experiment independently supports a change.
