# Facts batch: preserve acronym casing without creating fallback claims

2026-10-10. Base source: `1bab8eef98c78e2d24e3b91bf52fe91af8c35b17`.
Changes are uncommitted for coordinator integration. Only `crates/facts` and
new `tools/facts/continuous/` files changed. No agents, LLM calls, downloads,
services, credentials, API keys, held-out data, hidden tests, test generators,
or private logs were used. The existing main CLI, scorer, model, index,
denominator, shared reports and provenance files are unchanged.

## Hypothesis and acceptance, declared before the runtime edit

Lowercasing entity aliases can turn ordinary question words into false entity
mentions. Require original casing for a single all-uppercase alphabetic alias;
other aliases retain case-insensitive matching. Accept only fewer full-dev
wrong answers, every baseline right preserved, and no new wrong answers, with
the fixed first-200 sample evaluated before and after. Training diagnostics
must also show no new wrong answers. This is a generic lexical safeguard, with
no entity IDs, relation IDs, annotation-derived lists, or question exceptions
in the runtime change.

## Final measurements

| Evaluation | Before right/refused/wrong | After right/refused/wrong |
| --- | --- | --- |
| Fixed dev first 200 | 12 / 186 / 2 | 12 / 188 / 0 |
| Full dev 2,821 | 256 / 2,526 / 39 | 256 / 2,545 / 20 |
| Full train 19,481, diagnostic only | 12 / 19,251 / 218 | 12 / 19,405 / 64 |

The existing `glassmind-facts score` command produced both dev tables; raw
aggregate output is in `baseline-{sample,dev}.json` and `after-{sample,dev}.json`.
The separate question-only prediction example and offline comparison establish
per-item preservation: dev has 256 right→right, 2,526 refused→refused,
19 wrong→refused, and 20 wrong→wrong. Train has 12 right→right,
19,251 refused→refused, 154 wrong→refused, and 64 wrong→wrong. No new answers
or wrong claims occur. Train totals are diagnostics using the same status
conditions, not an independent generalization result.

All 19 removed dev claims linked ordinary “of” to the uppercase outfielder
abbreviation `OF`. Train removes 153 such errors and one `ET`/“et” collision.
These are actual entity-link mistakes, not just benchmark/snapshot differences.
The full-dev entity-linked-correct diagnostic stays 504 and relation top-1
stays 2,664. Correct answer coverage remains 256/2,821.

## Exact changes and verification

- `crates/facts/src/lib.rs`: share token normalization, preserve a cased copy
  of the question, and require casing support for uppercase alphabetic aliases
  among the original longest matching candidates. Rejecting casing cannot
  resolve a tie or expose a shorter match. Five focused tests cover ordinary
  words, explicit acronyms, case-insensitive descriptive aliases, explicit
  mixed-case alternatives, longest matches, ambiguity and fallback prevention.
- `crates/facts/examples/continuous_predictions.rs`: load the frozen model and
  store once, consume question-only lines from stdin, emit ordinary `Answer`
  JSON. No labels or scoring occur in this example.
- `tools/facts/continuous/compare.py`: offline train/valid-only annotation join
  for transitions, error symptoms, and file hashes. It does not feed runtime
  or replace the official score command. JSON reports preserve row-level
  transitions and hashes without copying questions into the repository.

Passed: five facts tests, facts release binary/example build, facts all-targets
clippy with `-D warnings`, facts fmt check, and `git diff --check`.
An initial format check and diagnostic-example indexing lint failed and were
fixed. No workspace-wide suite was run for this isolated facts change.
Every cargo invocation used the required 8 GiB / zero-swap systemd scope,
`setsid --wait`, 600-second timeout, disk TMPDIR, existing CARGO_TARGET_DIR,
`-j1` (except fmt), and four test threads. Outer scope caps were also verified.

## Rejected experiments and remaining failures

1. A broad short/common-word filter was rejected during the audit: aliases
   such as “Up!” have baseline scored-right results, so dropping them would
   violate preservation. No runtime version of this broader filter was kept.
2. The first acronym implementation removed casing failures before selecting
   the best candidate. Dev improved identically, but train gained three new
   wrong answers when old ties were broken. It also gained one coincidentally
   scored-right answer with an incorrect entity. This version was rejected;
   `rejected-filter-train-transitions.json` records it. The final additional
   acceptance gate preserves the original candidate set, making inference
   monotonic: an old answer is retained unchanged or becomes a refusal.

Dev error symptoms before→after: entity mismatch 27→8, relation mismatch 3→3,
inverse-answer mismatch 4→4, forward snapshot-answer mismatch 5→5. Train:
entity mismatch 193→39 and inverse-answer mismatch 25→25. These categories
describe annotation disagreement; they do not adjudicate all remaining claims.
Partial title/person aliases remain unsafe. Inverse uniqueness is only local
to the finite index. Some historical labels disagree with current claims or
with the wording of the question. No evaluator changes were made.

Four dev results scored right despite linking the wrong entity (rows 434,
470, 1705, 2423); these are retained and disclosed in `dev-transitions.json`.
The frozen answer-only metric must not be represented as entity correctness.

## Source and artifacts

No new external data: **0 bytes**. Existing sources were used unchanged:

- SimpleQuestions-Wikidata revision
  `df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, CC BY 3.0 according to existing
  provenance. Train 1,193,868 bytes / 19,481 rows; valid 173,187 bytes / 2,821
  rows. Source hashes are recorded in the transition JSON and existing
  `tools/facts/source.json`.
- Wikidata CC0 snapshot retrieved 2026-10-10: existing 500-entity index,
  287,236 bytes; SHA256
  `923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`.
  The index was historically seeded by validation subject IDs; this batch
  adds no seeds or claims. It remains a benchmark-scoped retrieval experiment.
- Frozen relation model: 35,773,124 bytes; SHA256
  `db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
  No weights or refusal thresholds changed.

`artifacts.json` fingerprints the exact implementation and diagnostic code.
Raw before/after prediction streams and the rejected train stream are retained
under `/home/gsc/.cache/glassmind/continuous-20261010/facts-run/tmp/`; their
bytes and hashes are in the comparison reports. They contain only train/dev
predictions. The coordinator should preserve this directory with the handoff.

## Reproduction

Run from this worktree. For the baseline, use the base commit above in a
separate checkout and the same score commands before applying this patch.
The diagnostic example can also be copied into that baseline checkout: it
does not modify inference. Existing target and TMPDIR paths are shown here.

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/facts-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/facts-run/target
FACTS_DATA=/home/gsc/.cache/glassmind/data/simplequestions
cap() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
cap cargo build --release -p glassmind-facts --bins --example continuous_predictions -j1
cap cargo test -p glassmind-facts -j1 -- --test-threads 4
cap cargo clippy -p glassmind-facts --all-targets -j1 -- -D warnings
cap cargo fmt -p glassmind-facts --check
"$CARGO_TARGET_DIR/release/glassmind-facts" score --input "$FACTS_DATA/valid.tsv" \
  --model "$FACTS_DATA/relation.json" --index tools/facts/index.json --limit 200
"$CARGO_TARGET_DIR/release/glassmind-facts" score --input "$FACTS_DATA/valid.tsv" \
  --model "$FACTS_DATA/relation.json" --index tools/facts/index.json
for split in valid train; do
  cut -f4 "$FACTS_DATA/$split.tsv" > "$TMPDIR/facts-$split-questions.txt"
  "$CARGO_TARGET_DIR/release/examples/continuous_predictions" \
    "$FACTS_DATA/relation.json" tools/facts/index.json \
    < "$TMPDIR/facts-$split-questions.txt" > "$TMPDIR/facts-reproduced-$split.jsonl"
done
python3 tools/facts/continuous/compare.py --data "$FACTS_DATA" --split valid \
  --before "$TMPDIR/facts-baseline-dev.jsonl" --after "$TMPDIR/facts-reproduced-valid.jsonl"
python3 tools/facts/continuous/compare.py --data "$FACTS_DATA" --split train \
  --before "$TMPDIR/facts-baseline-train.jsonl" --after "$TMPDIR/facts-reproduced-train.jsonl"
```

Limitation: genuine lowercase mentions of an uppercase-only alias can now
refuse; explicit mixed-case aliases and descriptive names remain supported.
The rule does not cover all ambiguity or expand coverage. This batch is
complete under the declared criterion; broader entity linking and inverse
completeness need separate experiments. No owned jobs remain running.
