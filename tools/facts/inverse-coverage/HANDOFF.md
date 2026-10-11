# Incoming Wikidata coverage: rejected partial candidate

One bounded prospective batch, source `b569ac0eb0bab5a8b9d39cda904c816e77c08e63`.
**No measured correctness improvement; do not deploy this candidate or replace any
default.** The candidate fails the preregistered >=7 strict-joint-correct gate:
full DEV remains2. Acquisition is **incomplete**:18 successful truncated queries,
1 timeout,81 unqueried frozen pairs. This does not exhaust or disprove the broader
incoming-edge hypothesis. No score is claimed for complete top100 coverage.

Only new `tools/facts/inverse-coverage/` files are changed, uncommitted. No Rust,
CLI, scorer, existing model/index/script, shared manifest, README or historical
prompt edits. No agents, LLM calls, API keys, credentials, held-out data, hidden
materials, old private logs, services or persistent jobs. The existing session's
model/login was not changed or independently inspected; no additional model or
authentication operation was needed. All owned fetch/build/evaluation processes
finished. Outer scope verified8GiB/0swap; one compliant capped cargo build passed.

## What changed

- `PLAN.md` records hypothesis, baseline, immutable selection, acquisition limits
  and gates before implementation/network/DEV.
- `selection.json` freezes top100 TRAIN subject/inverse-property pairs. Selection
  reads only the first two TSV fields, never decodes object/question fields, and
  never reads DEV labels. First floor(.9*N), exclude zero-based i%10==0, require
  subject in frozen999; descending pair count then SHA256(subject TAB relation).
- `experiment.py` independently verifies reused provenance/selection, queries only
  primary WDQS with normal/preferred statement paths, retains raw evidence and
  explicit missing/truncated statuses, and deterministically merges witnessed
  edges using byte insertions. New source nodes have empty aliases.
- `test_experiment.py` contains7 public authored synthetic tests: frozen selection
  exclusions and answer/question independence; empty/truncated/missing responses;
  normal/preferred versus deprecated or mismatched statements; original-byte and
  alias preservation; deterministic merge/rebuild; missing-query handling; raw
  hash tampering rejection. Synthetic data never enters the real candidate.
- `evaluate.py` freezes candidate/model/scripts/binaries before DEV, invokes the
  unchanged official scorer and existing `tools/facts/joint-audit.py` audit
  implementation, separates question-only inference from annotations, and records
  complete transitions, hazards, ID equality and relocated bundle replay.
- `summarize.py`, `RESULTS.json`, `POLICY.md`, `COMMANDS.md` and this handoff retain
  review evidence. No runtime network, runtime LLM or answer lookup table.

## Frozen selection and graph

TRAIN19481rows; prefix17532; eligible15778rows;1934 inverse rows with base-index
subjects;464 distinct eligible pairs. Top100 account for1251 eligible TRAIN rows.
Selection SHA256:
`e83ebf84d3574b2fcea42254bfb65fc4077a5d6250e9317b65f35c8933af0a45`.
71 reused artifacts were verified against earlier provenance, including frozen
selection, base graph, raw sources, bundle, labels and relation model. Selection
was independently recomputed before fetch and again before merge. No gold answers
became graph claims, and no fetched labels broadened entity search.

18 independently returned sets each contain101 distinct source subjects: all are
TRUNCATED, none is a complete incoming set.1818 witnessed edges include one already
present edge;1817 net new claims increase35882→37699.1735 new aliasless source nodes
increase999→2734 entities. **All999 original entity records are exactly unchanged**
(no existing subject needed an extra edge in this batch); their original serialized
content is a byte-identical prefix before appended entities. Original aliases,
claims and display label mappings remain unchanged. Candidate index760174bytes.
Two offline rebuilds were byte-identical. Evidence: `base-preservation.json`,
`determinism.json`, `merge.json` in the run directory.

## DEV results and acceptance

Candidate, scripts, model and binaries frozen before first DEV access. Fixed
first200 ran once, then full2821 once, with no subsequent tuning. Baseline is the
rejected TRAIN999 snapshot, never the historical validation-seeded500 series.

| Scope / metric | Baseline right/refused/wrong | Candidate right/refused/wrong |
|---|---|---|
| First200 answer-only | 3 / 191 / 6 | 3 / 191 / 6 |
| First200 strict joint | 0 / 191 / 9 | 0 / 191 / 9 |
| Full2821 answer-only | 47 / 2712 / 62 | 47 / 2712 / 62 |
| Full2821 strict joint | 2 / 2712 / 107 | 2 / 2712 / 107 |

Full answer-only transitions: right→right47, refused→refused2712, wrong→wrong62;
every off-diagonal cell0. Full strict-joint transitions: right→right2,
refused→refused2712, wrong→wrong107; every off-diagonal cell0.
Sample transitions are likewise diagonal: answer3/191/6, joint0/191/9.

**Every standalone prediction is byte-identical to baseline**, including linked
entity and relation. Both old joint-correct rows retained; no new joint-wrong or
answer-only-wrong rows; no correct losses; no disagreement or changed prediction
to list. All109 accepted predictions remain: answer precision47/109=43.1193%,
joint precision2/109=1.8349%, unchanged. All alias links unchanged. Standalone/talk
answer IDs agree for both scopes on every sample/full row. Relocated opt-in
candidate bundle gives byte-identical talk output for both complete streams.
Only the >=7-joint-correct gate fails. Empty `hazards.json`, `disagreements.json`,
`alias-link-changes.json` and complete transitions/predictions are retained.
The raw candidate bundle is evidence only; no deployed/default bundle was written.

## Network evidence, corrections and limitations

Official usage/policy/RDF docs were inspected before dataset requests; links and
interpretations are in `POLICY.md`. Primary endpoint only:
https://query.wikidata.org/sparql . Identifying project User-Agent, gzip acceptance,
45-second socket timeout, sequential requests, redirects refused. Query has
DISTINCT incoming item subjects and fixed ORDER BY subject LIMIT101, using
p:/ps: and explicit normal/preferred rank membership, never truthy-only claims.
One eligible statement/rank per returned subject is sampled as a witness.

Acquisition window2026-10-11T06:45:11.725342Z–06:50:02.879551Z.19 request attempts;
minimum start spacing5.001444495s;291.149693s total network-phase wall time including
local parser repair pauses;1,117,488 new decompressed response bytes. All remain
under110requests/600seconds/20MiB. Last attempted pair Q6581097/R21 timed out during
read; no HTTP status or response body was returned. It stays missing, not empty.
No retry after that timeout, no rate-policy workaround or endpoint switch.

Two **pre-DEV parser defects**, not graph/scoring experiments, were corrected:

1. Request2 returned legitimate lowercase `q…` in statement GUIDs. The initial
   validator incorrectly required uppercase; validate GUID subject prefix ignoring
   case, preserving original URI in raw data.
2. Request12 returned distinct IRIs in endpoint ORDER BY order, differing from
   Python lexical ordering (Q30/Q76 before Q100…). Preserve source ordering and
   check distinctness; do not change the SPARQL query or impose Python ordering.

Original failed ledgers are preserved in `fetch-before-local-parser-fix*.json`.
Both corrections replayed stored HTTP200 replies, then continued from the next
frozen pair within the **original**600-second wall deadline, request count and byte
budget. No response was refetched and no scope, rule, threshold or model was tuned
on DEV. The subsequent socket timeout ended network work conservatively at the
first transport failure, although the maximum budget had not been consumed.

The primary endpoint covers the main graph, not the scholarly graph. All18 returned
sets are truncated: no global uniqueness or complete negative knowledge is implied.
Endpoint data may lag edits or be cached. Temporal qualifiers, question constraints
and statement references are not modeled. **Numeric revisions are unavailable**:
OPTIONAL EntityData schema:version returned no bindings for all1818 witnesses.
Revision fields remain null; exact raw response/query hashes, HTTP headers and
request times provide snapshot identity, not an invented Wikidata revision.
Independent edges do not prove the annotated answer or broad semantic correctness.
The partial graph gives no prediction improvement; it does not justify a new
threshold, larger default index or retroactive acceptance of the old999 expansion.

## Source versions, licenses and bytes

New data: primary WDQS returned structured Wikidata data, **CC0-1.0**,18 retained
HTTP200 JSON responses totaling **1,117,488 decompressed bytes**. Source version is
the above request window plus each exact raw SHA256/query in `fetch.json`; no
numeric entity revision was supplied. The candidate760174bytes is derived graph
data under the same Wikidata license. No dataset labels or extra aliases fetched.

Reused SimpleQuestions source: askplatypus/wikidata-simplequestions revision
`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, **CC BY3.0**. TRAIN1193868bytes,
SHA256`25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b`;
DEV173187bytes, SHA256`1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48`.
No download or access to official held-out split. Existing Wikidata base650045bytes
and relation model35773124bytes reused unchanged; no new model or dependencies.

## Evidence and reproduction

Run root (raw graphs/bundles/predictions are outside git):
`/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/tmp/inverse-coverage`.

Exact commands, compliant build invocation and replay boundaries are in
[COMMANDS.md](COMMANDS.md). Minimal offline reproduction from this checkout:

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/target
export PYTHONDONTWRITEBYTECODE=1
python3 tools/facts/inverse-coverage/experiment.py verify
python3 -m unittest discover -s tools/facts/inverse-coverage -p 'test_*.py' -v
python3 tools/facts/inverse-coverage/experiment.py rebuild
```

The retained candidate freeze prevents silent script/model/index changes. Existing
sample/full directories prevent accidental repeated DEV overwrites. Full raw
prediction/model/binary/query/response hashes are in `candidate-freeze.json` and
`artifact-inventory.json`; baseline source ledger is `base-verification.json`.
No full workspace checks were needed because Rust was unchanged.

```text
base index 142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791
candidate  95c3bcae0790a8d4d2c8c39835573a7ab544584b4018fc08ceae644ab9ffb04c
model      db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6
fetch      35d8d220e45d9500dc8a30fe37ed6f1ea14cd2c2ca833c1196be03796141a986
freeze     19ae9507d3e55e18a9f3501d1a1d65b4da2b08979ee2c54f26f55aa695a5255d
```

Coordinator owns integration. Keep this as a rejected, incompletely acquired
prospective experiment; no additional fetching, tuning, commits or deployment
were performed after DEV.
