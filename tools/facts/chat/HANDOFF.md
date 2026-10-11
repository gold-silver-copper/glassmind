# Local facts bundles in `glassmind talk`

Worker completed on base `05e80930ef2b06e8e054ece3ab4cc69666d19b98`.
Coordinator changes below generalize bundle loading and presentation; independent
integration checks are recorded under reports/integrated when complete.
Preregistration: [PLAN.md](PLAN.md). The exact original submission is archived
at `/home/gsc/.cache/glassmind/continuous-20261010/factchat-run/submitted/`.
No agents, additional LLM calls, services, API keys, credentials, held-out data,
hidden tests, generators, or old private logs were used. No runtime network.
All owned subprocesses have completed or been terminated; no downloader remains.

## Exact change and scope

- `crates/cli/src/main.rs`: add `talk --facts-dir PATH` and helper module.
- `crates/cli/src/everyday.rs`: load facts once before the conversation; default
  mode never opens fact assets. Preserve typed memory/calculation priority and
  established story priority. Fresh questions can reach the frozen facts engine
  before an empty story world masks them. Fact answers **and refusals** clear
  `last_slot`; that clearing persists through the existing session schema.
  `/reset` clears conversation/story state while keeping the immutable store.
- New `crates/cli/src/talk_facts.rs`: immutable adapter around existing
  `Store::answer`, singleton IDs, optional display-only labels, explicit missing
  labels, source attribution and uncertainty. JSON preserves `entity`,
  `relation`, `answers`, `refusal`; adds display entities, Wikidata links,
  forward/inverse relation direction and snapshot/model provenance.
- CLI Cargo.toml/Cargo.lock add the existing `glassmind-facts` path dependency
  and SHA256 hashing via sha2; dependency provenance is recorded separately.
- New `tools/facts/chat/`: bounded labels fetcher, development-only adapters,
  reproduction script and this evidence. No fact linking, answering, index,
  weights, scorer, story/quantity algorithms, shared manifests or README edits.

`--facts-dir` requires `manifest.json`, `index.json` and `relation.json`;
`labels.json` is optional. The version1 manifest declares `index_sha256`,
`relation_model_sha256`, `source`, `license` and `scope`. Actual SHA-256 digest
verification and typed validation reject swapped/corrupt assets. The binary
contains no dataset fingerprints or entity counts; another compatible bundle
can be loaded without recompilation. The manifest declares the pair, but does
not prove the publisher trustworthy or the classifier accurate. Model labels
must be forward P-number or inverse R-number Wikidata properties.

An optional labels map must identify the actual index SHA-256, declare its
source/license, and contain only existing subject/claim-object IDs. It never
supplies linker aliases, claims or answer candidates. Human replies are concise,
mark the result as a snapshot suggestion and link its source; JSON retains the
learned relation, asset identities and coverage scope. Missing labels explicitly
fall back to QIDs. Two independent-bundle unit checks verify replacement bundles,
swapped assets and display-only label isolation. The original worker's pinned
FNV loader and verbose presentation were replaced before publication.

Package relocatable copies with `tools/facts/chat/package.py --index INDEX
--model MODEL --labels LABELS --source SOURCE --license LICENSE --scope SCOPE
--output NEW_DIRECTORY`; omit `--labels` if unavailable. Existing different
assets refuse overwriting. See `reproduce.sh` for the frozen demonstration.
SHA256 dependencies and licenses are in [dependencies](reports/dependencies.json).

## Coordinator verification of the integrated source

Source `d1076c4c97638f7750d8d7745ce1ef5a8db16827`, frozen binary SHA256
`3a2431e7e0a2b686c831b94da32180f3dcbbb5da7686b1e5e5ceaa96836b01a0`.
Independent sample200/full2821 replay preserves every accepted answer set and
exact totals above. All four default/facts and model/no-model legacy runs retain
41 memory,27 calculation and47 story checks. Both final mixed/config/display
runs pass28/28. All8 CLI tests pass after the final boxing change, and workspace
fmt/all-target clippy/release/strict4/4 fixture smoke pass. Initial clippy flagged
large enum size; boxing the fact reply fixed it before the final checks.
The unchanged standalone facts predictor/scorer and index were compared against
the worker base; no algorithm, model or score definition changed.

[Integrated results, binary identities and logs](reports/integrated/validation.json)
and [actual concise reply](reports/integrated/human-example.txt). Raw data, logs,
portable bundle and frozen binary remain in
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/factchat-check/`.

## Original worker measurements (before coordinator loader changes)

| Check | Result |
|---|---|
| Fixed first200, frozen standalone vs opt-in talk | both 12 right / 188 refused / 0 wrong |
| Full2821, fresh `/reset` before **every** question | both 256 right / 2545 refused / 20 wrong |
| All baseline accepted answer-ID sets | **276/276 preserved**, zero changes |
| Existing memory checks, default and facts | **41/41** each |
| Existing calculations, default/facts, with/without intent model | **27/27** each |
| Existing mixed stories, default/facts, with/without intent model | **47/47** each |
| New mixed/config/display checks with labels | **28/28** |
| New mixed/config/display checks with intent model and labels | **27/27**; same-size corruption check added afterward and run once without model |
| Existing CLI unit tests | **6 passed** |
| Final release build / all-target CLI clippy `-D warnings` / fmt | passed |
| Python compile / shell syntax / git diff whitespace | passed |

Memory fixture itself has no intent-model flag; its identical 41 checks were
repeated in each wrapper mode. The model-enabled calculation/story fixtures
really pass `--model`. Existing fixture definitions and scoring were unchanged.
Wrapper scripts inject only `--facts-dir` into the existing fixture subprocess
command. Their own reported `binary_sha256` describes the wrapper; the actual
binary hash is separately recorded below and in `reports/artifacts.json`.

New mixed cases cover persisted name/preference correction focus after facts,
unsupported facts, explicit memory overrides (including indexed entity names),
calculations, established stories about an indexed entity, facts coexisting
with stories, reset, restart, legacy memory JSON, missing-label presentation,
label/alias isolation, partial config, mismatched model/index/labels and
same-size index corruption. They verify the index hash remains unchanged.
The known fact utterance used in mixed checks comes from the fixed dev sample;
these are product integration checks, **not independent factual generalization**.

The adapter writes question-only streams, sends only utterance text plus reset
commands to `talk`, and separately compares returned IDs with the unchanged
standalone predictor/scorer. No gold subject, relation or answer IDs enter
runtime inputs. Full facts dev is separate from mixed conversation fixtures.
Without opt-in, the same 2821 fresh questions produce zero factual-ID responses
(2816 refusals, three story statements, two story answers); this describes
routing, not a new accuracy score. Reports: [sample](reports/sample.json),
[full](reports/full.json), [legacy](reports/legacy.json),
[mixed](reports/mixed.json), [mixed with model](reports/mixed-model.json).

## Labels: real display improvement, partial download

The original frozen cache supplies 483 eligible English labels, including only
**38/10215 claim targets**. The separate final map supplies **3806/10677 eligible
IDs**, including **3361/10215 claim targets**. Accepted dev responses with an
English answer label rise **19/276 → 112/276**; 164 accepted responses still
need explicit QID/link presentation. This changes display, not factual coverage.
The full sample/full dev replay with the final labels has identical answer IDs.

New data source: official `https://www.wikidata.org/w/api.php`, `wbgetentities`,
`props=labels`, `languages=en`, `maxlag=5`, only IDs already in index keys or
claim objects. No query/gold-derived fetch selection, descriptions, aliases,
claims, other datasets, keys or runtime access. License: **CC0-1.0**,
[Wikidata licensing](https://www.wikidata.org/wiki/Wikidata:Licensing).
Version is the dated response snapshots with individual SHA-256 hashes, rather
than a dump revision. All raw hashes/byte counts/UTC file dates and cache source
hashes are in [label provenance](reports/label-provenance.json).

Retained new responses: **70 files, 363928 bytes**. Final map: **126989 bytes**,
SHA-256 `21f0ecb65523c09562bfcf1efae44362d3222082ca71dcbce06ce7ba063ae9c1`.
Conservative download wall interval: **412.607 seconds**, within ten minutes.
Sequential requests contained at most50 IDs, with at least one second between
request starts; eligible universe10677 is below12000. No concurrency or bypass.
Stopped on an observed **HTTP429 / Retry-After38**; retained partial coverage
and made no further requests. The persistent budget is halted for this batch.

Fetcher limitation/rejected implementation: the initial process retained60
successful raw responses but did not record HTTP error events. Review found
that an HTTP200 maxlag response's Retry-After could be lost in its exception
path. No maxlag/error JSON was present in its retained raw responses. Stopped
that exact process, fixed handling and resumed with the original raw bytes and
a conservatively **shortened shared** wall budget. The resumed process retained
10 successes and logged the429, then was stopped. Initial HTTP-error counts
are unknown; do not represent the new budget log as a complete request log.
All retained response bodies are hashed. The corrected fetcher persists byte,
wall, error and retry budgets across resumes and stops on repeated failures.

All new label data remains at
`/home/gsc/.cache/glassmind/continuous-20261010/factchat-run/tmp/labels/`.
Raw data and the model are outside git; only reports and adapter source are new
repository files. Offline reproduction reuses these raw responses. Starting
from an empty run directory reuses only the frozen cached labels unless a
separate bounded fetch is explicitly chosen.

Original worker human output before coordinator presentation changes (also
[retained](reports/human-example.txt)):

> Local Wikidata snapshot: Zulu (Q10179; https://www.wikidata.org/wiki/Q10179). Matched Yesterday (Q1679545; https://www.wikidata.org/wiki/Q1679545); my learned interpretation of your question may be mistaken. This snapshot covers 500 selected entities, not all of Wikidata.

## Baseline, rejected assumptions and limitations

- This is the existing **500 validation-subject-seeded** index, not open-world
  retrieval. The inherited **20 wrong answers remain**. No new factual coverage
  or improved factual accuracy is claimed, and no thresholds/weights changed.
- Proposed fixture “Who wrote The Steel Mirror?” produced the frozen model's
  `uncertain relation` refusal. Kept that behavior; rejected the fixture's
  assumption of answerability and used an already accepted dev utterance for
  routing checks. No model/algorithm tuning followed this observation.
- The first cold build started before the dependency edit completed and failed
  with unresolved `glassmind_facts`; subsequent correctly configured builds,
  checks and tests passed. Its failure is retained in own run build-baseline.log.
- Initial/final scores were identical; there was no rejected routing variant.
- Bare corrections after facts require explicit personal-memory focus to edit a
  slot. They cannot revise the frozen store. Story routing remains conservative:
  an established story may refuse a supported story-shaped question before the
  fact route. The preservation claim is explicitly for fresh contexts.
- This batch did not run giant workspace suites, coding evals, full quantity or
  bAbI suites, or any held-out evaluation. The affected full facts dev and stated
  existing conversation gates were run. Coordinator checks final integration.

## Source identities and reproduction

Actual final CLI: **68046640 bytes**, SHA-256
`ebf91e3354b5809da19d68bc59273b41159c79fc63ac0271753cfc4053dc058b`.
Source hashes and unchanged facts/scorer hashes: [artifacts](reports/artifacts.json).
Frozen index:287236 bytes, SHA-256
`923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`,
Wikidata CC0, retrieved2026-10-10.
Frozen model:35773124 bytes, SHA-256
`db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
SimpleQuestions source `askplatypus/wikidata-simplequestions` revision
`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, CC BY3.0;
valid2821 rows/173187 bytes, SHA-256
`1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48`.
These assets were reused, not newly downloaded or trained.

From the integrated checkout, offline reproduction using a fresh owned directory:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/facts-chat-reproduction
export CARGO_TARGET_DIR=/home/gsc/code/glassmind/target
mkdir -p "$TMPDIR"
bash tools/facts/chat/reproduce.sh
```

The script runs every cargo command with TMPDIR and
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`,
`-j1` except fmt, and `--test-threads 4` for tests. It performs first200 before
full2821, then unchanged legacy fixtures and mixed/config checks.
No automatic downloading occurs in reproduction.

Run the integrated product after packaging a bundle:

```bash
target/release/glassmind talk --facts-dir "$TMPDIR/bundle" \
  --session "$TMPDIR/my-session.json"
```

Ask `What language is spoken in yesterday`. Add `--json` for IDs/provenance.
Omit `--facts-dir` for the default conversation. Packaging copies assets so the
bundle can be moved intact; no runtime network access is used. Offline fresh
reproduction uses the original cached labels. To retain the fuller partial map,
pass the archived `factchat-run/tmp/labels/labels.json` to `package.py`.
