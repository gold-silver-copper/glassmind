# Bounded local fact sets — completed, uncommitted

Base `35d3ea80d63791af1625d5019077d6ea2cfa5f26`; parent integrates the owned
uncommitted changes. No main.rs, shared manifests, models, indexes, scorers,
other tracks, historical prompts or README changes. No agents, additional LLM
calls, network, credentials, API keys, commits or persistent services. All owned
processes completed. Outer scope verified MemoryMax=8589934592 / MemorySwapMax=0.
This worker cannot change or independently attest its inherited serving model;
it made no model calls or authentication changes.

## Exact capability

- `crates/facts/src/lib.rs`: new `Store::answer_set`, `AnswerSet` and fixed
  `SET_DISPLAY_LIMIT=10`. The original `Store::answer` method is byte-identical.
  Same exact alias linker and same unchanged learned relation proposer consume
  only the question. Lexically sorted unique real edge IDs, accurate local
  total/returned/truncated counts; forward dedup borrows IDs and clones at most
  ten, inverse scans subjects once and retains at most ten. Refuses unknown or
  ambiguous aliases, uncertain/unsupported relation and missing claims.
- `crates/cli/src/talk_facts.rs`: deterministic prefix parser and bounded list
  presentation. `List answers to: <question>?` / `Show answers to: <question>?`,
  ASCII-case-insensitive prefixes. Frozen shape restrictions in PLAN.md apply.
  The remainder alone reaches inference. JSON kind `fact_set` retains existing
  entity/relation/answers/refusal/display/provenance and adds the three counts.
  Missing labels explicitly fall back to IDs; labels never link. Plain text
  includes each returned ID's source link and shared property/direction, local
  scope and learned-interpretation caveats. Bundle/hash/label guards unchanged.
- `crates/cli/src/everyday.rs`: 13-line explicit routing branch before story or
  personal-memory interpretation. Advances turn/last utterance and clears focus;
  preserves beliefs and story facts, including across restart and /reset. No
  intent fallback on malformed list commands; absent bundle explicitly refuses.
  Ordinary memory/calculation/story/default fact routing unchanged.
- New owned `tools/facts/sets` contracts, replay adapters and evidence, plus
  `docs/fact-sets.md`. No new source data or runtime answer mappings.

## Prospective gates and measurements

PLAN.md, contracts.json, contracts.py and regression.py were frozen and hashed
before runtime edits; `sha256sum -c tools/facts/sets/frozen.sha256` still passes.
Expected sets are independently derived from actual authored fixture edges,
never candidate outputs. Fixture proposer weights are a tiny synthetic test
input, not production weights or an accuracy/generalization claim. A separate
Rust unit test exercises a freshly trained tiny proposer. Production evaluation
uses the unchanged frozen SimpleQuestions relation model.

| Gate | Final result |
|---|---|
| Baseline on five authored positive contexts | 5/5 ordinary multiple-answer refusals; 5/5 list requests yield no set/IDs |
| Frozen authored product contracts | 36/36 checks pass |
| Additional state invariants, with/without synthetic intent fixture | 16/16 checks pass |
| First200, then all2821 standalone output bytes | identical baseline/candidate |
| First200, then all2821 ordinary facts-enabled talk JSON bytes | identical baseline/candidate |
| First200, then all2821 facts-disabled default talk JSON bytes | identical baseline/candidate |
| Unchanged full standalone scorer | 256 right /2545 refused /20 wrong of2821 |
| Existing memory/calculation/story suites, facts enabled and disabled | 41/41,27/27,47/47 in each mode |
| Targeted Rust tests | CLI11 + facts6 pass |
| Targeted all-target clippy `-D warnings`, fmt check, release build | pass |

Five positive contexts cover forward/inverse multi-answers, both forward/inverse
>10 truncation, duplicate unsorted edges, Unicode and missing labels. Negatives
cover ambiguity, unknown/display-only aliases, uncertain relation, empty claims,
malformed/empty/multiple-question/trailing-action requests and unsupported
follow-up. Sessions include name/preference/calculation/story interleaving,
persistence/reset, active story external requests, and exact state equality
apart from intended turn/last utterance/focus changes. Supplemental state checks
were written after implementation but follow the already frozen session gate;
the primary frozen fixture files were never changed.

This is a product capability, **not a single-answer benchmark gain**. Historical
500 selection remains validation-subject-seeded; all inherited factual mistakes
remain. No gold/scorer/denominator changes. No held-out data, official test split,
hidden evaluation code/generators or old private logs were read. The rejected
999/inverse expansion was not adopted or fetched.

## Real local example and provenance

`List answers to: What language is spoken in Brotherhood of the Wolf?`
returns Q1321, Q150, Q652 (three matches), linked to Q1123891 / P364. The baseline
ordinary question refuses because of multiple local answers. These IDs were
independently verified against three published statement IDs in the existing
cached Wikidata response, not DEV annotations. See reports/real-example.json,
real-baseline.json and real-reply.{json,txt}. This is snapshot evidence, not a
claim about current global truth. No answer labels were supplied for this demo.

**New downloaded source bytes: 0.** Existing Wikidata snapshot retrieved
2026-10-10, CC0-1.0: index287236 bytes, SHA256
`923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`.
Frozen relation model35773124 bytes, SHA256
`db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
SimpleQuestions revision `df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, CC-BY-3.0;
DEV2821 rows /173187 bytes, SHA256
`1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48`.
The reused real-example raw response is1066603 bytes, with exact hash and claim
IDs in real-example.json. Authored fixture bundle byte counts, MIT OR Apache-2.0
license and per-file hashes are in reports/artifacts.json; no third-party source
bytes were acquired. Existing source licenses inherit unchanged.

Baseline binary74058600 bytes, SHA256
`0d5c9ea8be0e4d3c83b17508e5666ed814bb8b8f4e78364f478fd7d3e3a0ba21`.
Final binary74285432 bytes, SHA256
`fe04ebf44491eac39e019ef9021d6f70ca4cbe3300b6d3c3822dd233e5b06ead`.
Runtime patch13712 bytes, SHA256
`53a2a67d3b0073fde7795b7a085942ada2ba97199ab8583991a13888de72a39b`.
Source file, script, model, index, predictor and artifact hashes are recorded in
reports/artifacts.json. Full raw baseline/candidate streams and frozen binaries
remain at `/home/gsc/.cache/glassmind/continuous-20261010/factsets-run/tmp/sets`.
Legacy fixture reports hash executable wrappers; artifacts.json identifies the
actual final binary separately.

## Reproduction

From this checkout, using the retained frozen baseline (or rebuild this base in
an isolated checkout under the same cap and copy its two executables):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/factsets-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/factsets-run/target
export BASELINE_DIR="$TMPDIR/sets/baseline"
export RUN="$TMPDIR/sets-reproduction"
bash tools/facts/sets/reproduce.sh
```

The script packages existing assets offline, runs baseline contracts, sample200
then full2821 baseline, targeted cargo gates, candidate contracts/session checks,
sample200 then full2821 byte regression, both legacy conversation modes, and the
raw-claim-backed example. Every cargo/fixture command runs under
`TMPDIR=... systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`;
cargo uses `-j1` except fmt and tests use `--test-threads 4`.
Exact executed final sequence/log is retained in own run final.sh/final.log and
reports/validation.log. The consolidated reproduction script was syntax-checked;
its component commands were executed, not a redundant whole replay.

## Failures, rejected experiments and limits

The first candidate passed all behavioral gates but clippy rejected two guarded
byte indexes. Replaced them with checked access, removed parser indexing, and
changed the list's “Matched subject” wording to “Matched entity” for inverse
clarity. Final source was rebuilt and all affected gates rerun. First failure
log and initial candidate binary remain archived; no thresholds, fixtures or
gates were weakened. No semantic candidate or data experiment was rejected.

Grammar checks a bounded question shape, not semantic completeness. The inherited
relation classifier and exact alias linker can misinterpret a well-formed question;
no new relation-confidence or entity-disambiguation claim is made. Set counts
refer only to admitted edges in this finite snapshot, never a world-complete
answer list. The old model's inherited errors persist. Forward dedup requires
O(distinct matching targets) borrowed references; inverse traversal remains a
single full store scan, as in the existing method. No index redesign.

No giant workspace suites or other-track benchmarks were run. The production
conversation intent model was unavailable at the historical default path;
legacy41/27/47 ran without it in both bundle modes, and session bypass was tested
with/without the synthetic intent fixture. Parent should repeat integration
checks with its current production intent model and concurrent routing changes.
