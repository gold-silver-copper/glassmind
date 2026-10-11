# Data sources

Everything glassmind is trained or measured on, where it came from, and what
it became. The data itself lives outside git, and everything held out
(data, sources, generator prompts, the dev set, rustbench) lives where loop
agents cannot read it; the loop's config lists those places.

## Code to talk about (the Rust world)

| source | version | size | license | use |
|---|---|---|---|---|
| top_down_shooter_game (tds), ck, ghtui, memochat | the user's repos at the commits in `~/.cache/glassmind/sources/SOURCES` | 3.5–476 MB | the user's | QA training |
| bevy_noodle (noodle), fux | the user's repos at fixed commits (recorded with the held-out data) | 9–45 MB | the user's | QA held-out |

## Crates for coding tasks (the harness)

From crates.io at fixed versions, chosen for having no dependencies or
dev-dependencies, so they build and test in seconds offline. Tasks are
broken copies of them (`tools/datagen/harness_tasks.py`); labels come from
`cargo`.

| crate | version | size | license | use |
|---|---|---|---|---|
| dunce | 1.0.5 | 52K | CC0-1.0 OR MIT-0 OR Apache-2.0 | training |
| heck | 0.5.0 | 80K | MIT OR Apache-2.0 | training |
| json | 0.12.4 | 324K | MIT/Apache-2.0 | training |
| percent-encoding | 2.3.2 | 56K | MIT OR Apache-2.0 | training |
| quick-error | 2.0.1 | 100K | MIT/Apache-2.0 | training |
| scopeguard | 1.2.0 | 68K | MIT OR Apache-2.0 | training |
| siphasher | 1.0.4 | 116K | MIT OR Apache-2.0 | training |
| strsim | 0.11.1 | 96K | MIT | training |
| unicase | 2.10.0 | 148K | MIT OR Apache-2.0 | training |
| md5 | 0.8.1 | 52K | Apache-2.0 OR MIT | held-out |
| numtoa | 1.0.1 | 88K | MIT OR Apache-2.0 | held-out |
| roman | 0.2.1 | 36K | MIT | held-out |
| shlex | 2.0.1 | 92K | MIT OR Apache-2.0 | held-out |
| urlencoding | 2.1.3 | 52K | MIT | held-out |
| version-compare | 0.2.1 | 100K | MIT | held-out |

## rustbench (an outside benchmark)

[rustbench](https://github.com/gold-silver-copper/rustbench) at commit
`f79e71a`, MIT OR Apache-2.0: small std-only crates with one thing to fix
or change, a request in three styles (terse, chatty, hurried), and hidden
tests that judge the result.

- **dev split**: the fix-error, fix-test, edit and implement tasks without
  an output check (219 tasks, 657 turns) are training data
  (`tools/datagen/rustbench_tasks.py`). The scorer copies each task's
  hidden tests over a remedy and runs `cargo test`, as rustbench does.
- **held-out split**: measurement only, with rustbench's own runner and
  `tools/rustbench-agent.sh`. Never trained on, never turned into data,
  never shown to loop agents (it lives with the held-out data).

## Word knowledge

| source | version | size | license | became |
|---|---|---|---|---|
| Wikidata: 18 programming-concept items (subroutine, member variable, function call, record, class, test case, ...), English labels and aliases | fetched 2026-10-10 | 2 JSON files, 30 KB | CC0 | the tagger's word bridge (`crates/english/models/synonyms.tsv`, 144 words): a word it never saw read as the glassmind word for the same concept ("invoked" as "called", "member" as "field") |
| Open English WordNet | 2025 edition | 11 MB (XML) | CC BY 4.0 | computing senses for the same bridge; its commonest general senses were tried and left out (selection set 111 -> 106 right) |

Built by `tools/datagen/wordnet_bridge.py`. On the selection set the
bridge took the tagger from 108 to 111 right, none wrong; dev unchanged.

## Requests (generated)

All requests were written by Claude Opus 5.5 (`tools/datagen/phrase.py`),
working backwards from tasks with answers computed by tooling. No request
was written by a person.

| set | turns | generators | about |
|---|---|---|---|
| training | 7,557 (6,722 QA + 178 coding + 657 rustbench dev) | terse, chatty, newcomer, hurried personas; rustbench's own styles | training repos and crates, rustbench dev |
| validation | 267 | the training personas | held-out repos; guards acceptance and decides when the loop stops |
| dev | 497 | a hidden "texting" prompt | training repos; calibrates the small models' thresholds |
| held-out v2 | 711 (625 QA + 86 coding) | a style written by a separate agent; its prompt and requests were never read on the training side | held-out repos and crates; decides nothing |

Held-out v1 (a "query log" style, 725 turns) was retired on 2026-10-10: the
operator had read its prompt, and a training persona ("slack", since
deleted with its 1,107 turns) copied its examples with names swapped. Its
numbers up to loop round 29 are an upper bound. The v1 coding tasks also
reused the training breakages; v2's are broken by other means
(`tools/datagen/harness_tasks.py --breakers`, from a file outside the
repository).

## Toolchain

`rustc`/`cargo` from the stable toolchain run the coding tasks' checks; their
messages are the problems' details.

## Reading and grammar probes (2026-10-10)

Reused the existing downloads in `~/code/sememe/data/`; no sememe code or
crate is a dependency. `tools/reading-data.py` samples by a fixed SHA-256
rank of source member and row number. It prints counts only.

| source | version | size | license | use |
|---|---|---|---|---|
| [BLiMP](https://github.com/alexwarstadt/blimp) | local `blimp-master.zip`, SHA-256 `8c043d41701fff01cabf22fb9a2fd8e15cecac5240b777ff41680f4eb7fffc17` | 35,670,094 archive bytes; 1,340 dev pairs (20 per paradigm) | CC BY 4.0, per the archive's README | grammatical preference probe |
| [BabyLM](https://babylm.github.io/) | 2025 development archive, SHA-256 `b5af75aeb1ee5a36f65ee98f316fdae73b16a0ba3767cca3f3dfa5eaac2dc621` | 19,264,893 archive bytes; 120 dev documents (20 per domain) | mixed source terms: BNC, CHILDES/TalkBank, Gutenberg, OpenSubtitles, Wikipedia and Switchboard; see the distributed dataset README | grammar-to-meaning coverage probe |

`glassmind-reading` disables the normalizer and request-template tagger.
BLiMP prefers a sentence only if it has an unedited, complete meaning and
the other sentence does not; ties are reported explicitly, not credited as
right. This is a parse-only baseline, **not a probability-model BLiMP score**.
BabyLM counts documents that produce such a meaning, not whether that meaning
is correct. Documents over 48 tokens remain in the denominator as uncovered;
the report counts them separately. No bits-per-byte claim is made. BabyLM's
test archive is not used.

## Passage question answering (development experiment)

[SQuAD 1.1](https://rajpurkar.github.io/SQuAD-explorer/), repository revision
`eee5fdbf62f8613a7812b03419e6b29617b74fd1`, CC BY-SA 4.0: official TRAIN
87,599 questions from 442 articles (30,288,272 bytes) and public DEV 10,570
questions from 48 articles (4,854,279 bytes). No test data or model predictions
were fetched. An exact audit found no shared article titles, context strings or
question IDs across these two splits. All dev questions have annotated answers;
this dataset alone cannot establish refusal quality on unanswerable questions.

The experiment is separate from text probability and grammar coverage. Data,
hashes and overlap audit are retained under
`~/.cache/glassmind/continuous-20261010/readingqa-source/`. The unchanged official
`evaluate-v2.0.py` from the same revision supplies normalized EM and token F1 on
the all-answerable v1.1 data; no no-answer probabilities are supplied. Prediction
IDs must cover the full selected split before scoring, since the upstream script
otherwise omits missing predictions. The repository/scorer license is MIT.
No passage-QA capability or measured gain is claimed until independently verified.

## Everyday intents and conversation memory

[CLINC150 / oos-eval](https://github.com/clinc/oos-eval), revision
`828f8093932c8fe6ca7936c3d2e52903b1c523de`, CC BY 3.0
(Larson et al., 2019). The upstream JSON is 2,495,390 bytes. The training
fetcher saves 15,000 in-scope training rows (1,165,484 bytes) and 3,100
validation rows including 100 OOS requests (239,360 bytes), stopping before
any test items. The later OOS training section is intentionally unused.
Three identical utterances overlap official train and validation; the
reported official totals preserve them.

The word/character-feature margin model is trained on training only; its
refusal thresholds are selected on validation. A correct OOS refusal is a
right answer. `crates/everyday/EXPERIMENT.md` records full dev results and
reproduction. The frozen model has SHA-256
`a84fd59c3e49db08300f61ee33f6d73e64a8ea18109102b01685333b95936408`.
Models live under `~/.cache/glassmind/models/everyday/`; training/validation
JSONL and provenance live under `~/.cache/glassmind/data/clinc150/`.

The coordinator's `tools/heldout-tracks.py` scores the official 5,500-item
test split (including 1,000 OOS), outside worker sandboxes. Only totals are
reported; private test JSONL stays under `~/.cache/glassmind/heldout/`.
This test result does not choose the model or refusal thresholds.

Basic conversation memory is checked by 41 synthetic smoke assertions,
including corrections, Unicode, persistence, reset and previous-message
recall. These checks are not an independent dialogue benchmark. There is
no claim of SGD/MultiWOZ performance or external-service execution yet.

## Story reasoning

[bAbI tasks 1–20](https://huggingface.co/datasets/facebook/babi_qa), v1.2,
English 1k, CC BY 3.0 (Facebook, 2015): 20 training files, 2,947,923 bytes.
The 11,745,123-byte archive has SHA-256
`84f5296ab9a1ad0dc9464e08c491d65cd08830fca3acae9ab86f75e0fb81573c`.
The official download was unavailable; a fresh public S3 mirror download
matched the existing archive byte-for-byte by SHA-256.

`tools/stories/babi.py` opens only training members, partitions whole
stories by a fixed hash (16,013 train and 3,987 dev questions), and strips
answer keys, task IDs and supporting-fact IDs before invoking Rust.
The tiny motivation model learns state/action associations from training
assertions, not answer labels. Its provenance and exact reproduction are
in `tools/stories/`; the trained model is checked in.

Official test scoring uses the coordinator's totals-only runner, with the
archive stored under `~/.cache/glassmind/heldout/babi/`. Workers never read
its test members or predictions. The frozen model scored 19,916 right,
64 refusals and 20 wrong across all 20,000 official test questions.
No held-out failures were inspected or used to modify the reasoner.

### SimpleQuestions / Wikidata facts (2026-10-10)

- [SimpleQuestions-Wikidata](https://github.com/askplatypus/wikidata-simplequestions),
  commit `df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, CC BY 3.0:
  answerable train 19,481 rows / 1,193,868 bytes; validation 2,821 rows /
  173,187 bytes, including inverse relations. No test downloaded. Every tenth
  training row calibrates refusal; validation never fits weights.
- [Wikidata structured data](https://www.wikidata.org/wiki/Wikidata:Licensing),
  CC0, API snapshot from 2026-10-10: 20 cached responses / 27,696,032 bytes;
  500 indexed entities / 287,236 bytes. Validation subject IDs seed this
  benchmark-scoped candidate store. Validation answers and relations never
  populate the store; claims come independently from Wikidata. This is not
  an open-world entity-retrieval result. Rate limits stopped the crawl;
  missing coverage stays in the full validation denominator.
- Initial validation: 256 right, 2,526 refused, 39 wrong / 2,821.
  Multiple-answer claims are refused. Live facts may differ from the historical
  benchmark. Index, hashes, calibration totals and reproduction commands:
  [facts notes](../tools/facts/README.md). Relation weights remain in
  `~/.cache/glassmind/data/simplequestions/relation.json`.
- The later acronym-casing guard preserves all 256 correct answers and
  withdraws 19 false entity links: 256 right, 2,545 refused, 20 wrong.
  Index and model are unchanged; no new data was downloaded. It does not
  resolve old ambiguity or make new claims. Exact train/dev transitions and
  artifact hashes are in `tools/facts/continuous/`.

### Schema-Guided Dialogue (2026-10-10)

- [Official SGD](https://github.com/google-research-datasets/dstc8-schema-guided-dialogue/tree/e852981ae34990f4358979625854259302feaa78),
  version `e852981ae34990f4358979625854259302feaa78`, CC BY-SA 4.0.
  Fetcher scanned 504,306,737 bytes from explicit train/dev URLs only.
  Training retained 688 conversations containing `RideSharing_1`, 7,267,443
  bytes; 3,455 target-service frames supply labels. Full selected dev slice:
  all 45 single-service `RideSharing_1` conversations, 257 frames, 220,834 bytes.
- [Official metric implementation](https://github.com/google-research/google-research/blob/5b09c22d73a9d35eb6c5d2a99b95677a45053466/schema_guided_dst/metrics.py),
  version `5b09c22d73a9d35eb6c5d2a99b95677a45053466`, Apache-2.0, 11,209 bytes.
  NumPy 2.3.3 and fuzzywuzzy 0.18.0 with the pure-Python backend are pinned.
- Full slice: 202/257 exact states; official fuzzy joint goal 89.18%;
  256/257 correct active intents. The centroid baseline gets 52/257 exact
  states and 41.47% fuzzy joint goal. These are single-service slice results,
  not whole-SGD performance. No test split accessed. All 257 predictions
  reproduce from annotation-free runtime inputs.
- [Experiment and hashes](../crates/dialogue/EXPERIMENT.md). Retained source
  selection lives at `~/.cache/glassmind/data/sgd`; models at
  `~/.cache/glassmind/models/dialogue/model-{0,12}.json`.

### Numbers, units and dates (2026-10-10)

- Synthetic development suites `quantities-synthetic-dev-v1` (seed 20261010,
  2,080 rows / 314,258 bytes) and `quantities-extended-dev-v1` (seed 20261011,
  1,036 rows / 174,437 bytes), workspace MIT OR Apache-2.0 license.
  Python's `Fraction`, `Decimal`, `datetime` and `calendar` compute expected
  answers independently of the Rust implementation. These are generated
  development questions, not SVAMP/MAWPS or held-out benchmark results.
- Fixed samples precede full evaluation. Combined full dev: 3,056 right,
  60 refused, 0 wrong / 3,116. Right includes 762 expected refusals for invalid
  or ambiguous inputs; 2,294 substantive answers are correct. All 60 misses
  are unsupported English paraphrases. No external training data or test
  split was used. Sources, hashes, full reports and reproduction commands:
  [quantities handoff](../crates/quantities/HANDOFF.md).

### Conversation integration development checks

`tools/talk-dev.py` supplies a fixed 27-check synthetic conversation covering
calculation results, memory priority, correction focus, cross-process session
persistence, reset and invalid-input refusals. Arithmetic expectations use
Python Fraction and date expectations use datetime. This is product integration
coverage, not independent language data. The main `talk` command improves from
18 correct / 9 refused / 0 wrong to 27 / 0 / 0, both with and without the frozen
CLINC intent model. Existing 41 memory checks still pass. Routing all 3,116
existing synthetic quantity questions through `talk` preserves the standalone
solver's 3,056 / 60 / 0 result, including 762 expected refusals.

### External word problems: MathQA

[AllenAI MathQA](https://huggingface.co/datasets/allenai/math_qa), parquet
revision `fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`; Apache-2.0 per its
pinned dataset card. Only official train (29,837 rows, 9,013,733 bytes) and
validation (4,475 rows, 1,350,141 bytes) were downloaded. Official test was
not downloaded. Exact hashes and bounded fetch URLs are in
`tools/quantities/reports/source.json`.

Runtime receives question text only, without choices, labels or formulas.
The development adapter compares exact rational values against numeric gold
choices, with no tolerance or rounding. This is a custom numeric metric,
not official multiple-choice accuracy. All 4,475 rows count, including
1,229 choices outside the adapter's limited numeric normalization.
Baseline 5 right / 4,469 refused / 1 wrong becomes 14 / 4,458 / 3. The two
new scored errors are exact mathematical results versus rounded choices;
they remain counted wrong. Existing synthetic results are unchanged.
See [full provenance and limitations](../tools/quantities/HANDOFF.md).


## Fixed byte probability experiment (2026-10-10)

The separate `reading-probability` command trains a fixed byte six-gram on
all six BabyLM2025 strict-small10M training members: 54,351,399 raw bytes,
archive SHA-256 `6d9d2cb3b3feccf0669e74173edb55de6c7f12412ee02935d16b1b6b44160b39`.
Training produces 4,303,829 count entries in a 68,861,296-byte artifact kept
outside git. This is mixed-license research text; member attribution and
unresolved redistribution terms are recorded in
[the license report](../tools/reading/LICENSES.md). No corpus or weights
are distributed in this repository.

The same frozen dev probes are scored without truncation: all 1,340 grammar
pairs and 120 reading excerpts (10,399 text bytes). Bits per byte includes
one scored EOS per document in the numerator, no EOS in the byte denominator.
A post-evaluation audit found 33 reading excerpts equal full training lines;
none was removed. Grammar has 185 good-to-bad transitions and 23 regressing
categories despite its net gain. The old parse-only metrics remain distinct;
no semantic understanding or independent held-out claim is made.
[Settings, hashes, results and reproduction](../tools/reading/HANDOFF.md).


## Multi-service SGD and grounded slot transfer (2026-10-11 UTC)

Pinned SGD revision `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0.
Only explicit official train/dev shards and schemas were downloaded: 150 files,
504,306,737 bytes. No official test data was downloaded. Selection is by the
fixed schema service set RideSharing_1, Travel_1 and Weather_1, preserving
complete conversations: train 3,174 conversations/34,022 user turns/12,762 target
frames (45,291,003 selected bytes); dev 179 conversations, including 54 multi-service,
1,119 user turns and 1,218 gold frames. Runtime receives only speaker/utterance
arrays and the global schema, with no gold service lists, states or annotations.
Prediction isolation was checked by reversing conversation order and rejecting
annotation fields. All three services are predicted before gold is consulted.

The fixed joint12 router plus trained service trackers yields 549 exact joint
turns; learned, value-free slot relations raise this to 595/1,119, with 474 wrong
and 50 refused. All baseline-correct turns are retained. Exact service frames
rise 598→689/1,218; multi-service RideSharing remains 0/265. Four incorrect copy
events persist for 11 scored slot-turns. The earlier single-service slice remains
202/257 exact, 89.1751% fuzzy, and is a distinct metric/denominator. Rejected
router experiments are retained separately; no official held-out claim is made.

The 455-byte relation model stores only schema names and train evidence counts.
The candidate's frozen classifiers plus relations occupy 32,169,632 bytes,
SHA256 `9991b341b4da7e775e0fa7e3ca55a2a00f4935a0a7b94db16f02b2b6596be83c`;
large weights/data remain outside git. Source and selected-file hashes, all
new-error audits, license, cache locations and exact reproduction commands are
in [the reference-transfer handoff](../tools/dialogue/references/HANDOFF.md)
and [source manifest](../tools/dialogue/reports/data-manifest.json).

Enum-variant editing uses 14 independently authored local fixtures, 1,902 source
bytes, under the workspace MIT OR Apache-2.0 license, with no downloaded training
data. Four successful edits, nine expected refusals and rollback pass; all 765
RustBench dev outcomes remain unchanged. [Evidence](../docs/coding-variants.md).


A follow-up learned span selector uses the same train/dev files, with zero new
external bytes. SHA256(dialogue_id) splits the 3,174 train conversations into
2,532 fit and 642 calibration conversations; thresholds are fixed before dev.
Its 26 context-boundary rules store surrounding words/counts, never entity values.
The 8,132-byte rules and data-bearing reports retain CC-BY-SA-4.0 attribution.
The 32,174,437-byte combined model is outside git, SHA256
`deddfaac2f596c5090aeb65bf77d24470d1d44f61d2aad69469792cd26232781`.

On the same complete dev selection, exact joint turns rise 595→609/1,119,
exact frames 689→712/1,218, multi-service RideSharing 0→23/265. All 595 baseline
correct turns remain correct; 460 wrong and 50 refused remain. Eleven selections
add 32 correct scored slot-turn claims with no new wrong/unscored claims; the
inherited four copy errors across 11 slot-turns remain. All gains use later
system confirmations; initial indirect destination requests generally remain
unresolved. Independent training/full replay/legacy/isolation checks reproduce
worker results. [Span-selection provenance and limits](../tools/dialogue/offers/HANDOFF.md).


## Ratio-total MathQA follow-up (2026-10-11 UTC)

The same pinned MathQA train/dev files and unchanged exact-numeric evaluator are
used; no new downloads or learned model. Training inspection reads only Problem
text. Three fully consumed clause productions bind two-to-four positive rational
weights, their total, and an ordinal/min/max selector; checked arithmetic computes
the requested value. No choices, formulas, IDs or labels enter runtime.

Full dev improves14→16/4,475 correct,4,458→4,456 refused, with3 wrong unchanged.
All17 old answered records are preserved. The two new answers are400 and60 for
ratio/sum questions; both share training templates, so this is narrow template
coverage. First100 stays1/99/0, all3,116 synthetic outputs match the baseline,
and independent main-talk checks preserve memory/recall. No held-out data read.
[Full audit, source hashes and reproduction](../tools/quantities/wordproblems/HANDOFF.md).


## Facts in conversation and display labels (2026-10-11 UTC)

The existing frozen 500-entity Wikidata index and SimpleQuestions relation model
are now optional conversation assets. A portable manifest records their SHA256
identities and declared source/license/scope; alternate compatible bundles do
not require recompilation. The source index remains validation-subject-seeded,
so this integration makes no claim of open-world retrieval or new fact coverage.

A separate display-only map reuses original cached labels and 70 new official
Wikidata English-label responses (363,928 bytes, CC0-1.0). Only existing subject
and claim-object IDs were requested. The bounded fetch stopped after HTTP429;
initial error counts were not logged, and the report discloses that limitation.
Coverage is 3,806/10,677 eligible IDs and3,361/10,215 claim targets. Of276 accepted
dev replies,112 have answer labels (previously19); missing labels use QIDs/links.
Labels never enter entity matching or prediction. Model/index weights are unchanged.

The coordinator's fresh-context full2,821 dev replay matches the standalone
engine exactly:256 right,2,545 refused,20 wrong, all276 accepted ID sets preserved.
The SHA256 implementation adds8 locked dependencies totaling219,434 registry
archive bytes, already cached locally. Source revisions, per-response hashes,
licenses, remaining limitations and reproduction are recorded in the
[facts conversation handoff](../tools/facts/chat/HANDOFF.md).


## Compositional scalar constraints (2026-10-11 UTC)

Reuse the same MathQA train Problem-only text and fixed dev evaluator; no new
external bytes, models or dependencies. The parent-corrected candidate raises
16→22 right of4475, preserves all19 old answer records and keeps3 wrong.
Two worker-scored-correct difference questions now refuse because the sign of
the unknown is not determined. All3116 synthetic triples remain unchanged.
Local evidence includes50 permuted/contradictory affine cases and55 independently
enumerated absolute-difference cases, workspace MIT OR Apache-2.0. These are
development checks, not independent held-out benchmarks; two-variable coverage
has local evidence only. [Original training provenance](../tools/quantities/linearwords/HANDOFF.md)
and [corrected results and source hashes](../tools/quantities/linearwords/PARENT.md).


## Passage QA and rejected follow-ups (2026-10-11 UTC)

SQuAD1.1 TRAIN/DEV revision `eee5fdbf62f8613a7812b03419e6b29617b74fd1`
contains35,142,551 bytes (TRAIN30,288,272; DEV4,854,279), CC-BY-SA-4.0.
The pinned official scorer and source repository are MIT. Original fitting uses
4000 questions from64 TRAIN articles and1000 calibration questions from16 other
articles; all10570 DEV questions remain in the denominator. Inputs are context
and question only. [Original provenance and parent replay](../tools/reading/qa/PARENT.md).

The support selector and role-aware ranker experiments are rejected; their
patches stay unapplied and add no deployed models. The latter reuses exactly the
original fit/cal data with no downloads. Parent verified82 provenance hashes
and independently audited frozen predictions; [evidence and failures](../tools/reading/roles/HANDOFF.md).

## Rejected TRAIN-selected facts and joint audit (2026-10-11 UTC)

The new facts scope selects subjects from15778 TRAIN rows, excluding the old
every-tenth calibration rows and final10% prefix boundary. Its bounded Wikidata
fetch made39 requests and retained67,094,727 new raw bytes, CC0;28 subject
responses were reused. One redirect leaves999 of1000 chosen subjects. Raw
response hashes and timestamps exist, but revision IDs were omitted by the
original fetch parameters. The100-to999 expansion failed its precision gate;
neither index replaces the historical500-entity index.
[Selection, licenses and exact fetch evidence](../tools/facts/trainindex/HANDOFF.md).

A subsequent entity filter also failed its original gates and is not deployed.
The separate [joint annotation audit](../tools/facts/JOINT-AUDIT.md) reuses frozen
question-only predictions and public validation annotations, downloads no data,
and changes no original scorer or acceptance criterion. It reveals wrong-subject
answer coincidences:500-scope256 answer matches versus252 joint matches;
rejected999-scope47 versus2. These are distinct metrics and index scopes.


## Native repair witnesses and sequence proposals (2026-10-11 UTC)

The repair guard reuses27 original TRAIN requests and108 copied Cargo/source/public
test/lock files (29,232bytes). Every original file hash was rechecked. Typed scalar
witnesses are confirmed by native Rust in the original package before withdrawing
a repair. No expected solution, hidden test or generator content was inspected.
The parent added binding-context and executed-probe checks; all174 full-dev passes
remain and three failed requests become refusals. [Parent evidence](../tools/coding-ambiguity/PARENT.md).

The dialogue sequence model is rejected at TRAIN calibration:best458/470 spans
(97.45%) fails the frozen99% gate.2532FIT/642CAL conversations are split by hash;
new weights exclude CAL, though old baseline models used all TRAIN. The same
schedule was rerun to preserve failed weights; no first-run model hash exists.
No candidate DEV threshold or inference was used. All old v2/legacy predictions
remain unchanged. Zero new downloads; SGD source/license remain as documented
above. [Full rejection and parent audit](../tools/dialogue/spansequence/HANDOFF.md).

An offline coverage audit of the rejected999-entity facts snapshot finds only
three unique matching validation claims even with annotated entity/relation IDs.
This is a knowledge-coverage diagnostic, not a model score. It downloads no data
and does not alter the index, scorer or historical measurements.
[Definition and report links](../tools/facts/JOINT-AUDIT.md).


## SQuAD2 reading baseline (2026-10-11 UTC)

Downloaded only pinned public TRAIN/DEV from SQuAD-explorer revision
`eee5fdbf62f8613a7812b03419e6b29617b74fd1`:46,494,161bytes,CC-BY-SA-4.0.
TRAIN130319questions/43498unanswerable; DEV11873/5945unanswerable. Official
evaluator/MIT license reused from the existing source. No test or external
predictions fetched. Files stay outside git in WORK/readingqa2-source.

Frozen v1 fullDEV baseline:433 substantivecorrect,5140correctexpectedrefusals,
5064unsupported,1236wrong. Official EM46.94685% includes one normalization
coincidence:predictionA on an unanswerable item. Official metric is preserved,
with a separate5573-correct diagnostic excluding that substantive error. No new
model trained. [Hashes, definitions and reproduction](../tools/reading/qa2/README.md).

### TRAIN-selected inverse Wikidata acquisition (rejected partial experiment)

On 2026-10-11, a frozen TRAIN-only selection of 100 entity/inverse-property pairs
was queried through the primary Wikidata Query Service. Nineteen sequential
requests yielded 18 truncated 101-subject sets and one timeout; 81 pairs remain
unqueried. New decompressed CC0-1.0 data: **1,117,488 bytes**, retained outside Git
under `/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/tmp/inverse-coverage/`.
Raw response/query hashes and acquisition times pin the snapshot; per-entity
revision IDs were unavailable. All 999 original entity records and aliases were
preserved; 1,817 new claims and 1,735 aliasless nodes produced no development
prediction change. No candidate index is deployed. See
[provenance and limitations](../tools/facts/inverse-coverage/HANDOFF.md) and
[independent review](../tools/facts/inverse-coverage/PARENT.md).


## Bounded field ownership (2026-10-11 UTC)

New visible development fixtures are locally authored under MIT OR Apache-2.0;
no external downloads or held-out material. The implementation worker froze16
contracts containing6,651 input-source bytes and6,666 expected bytes. A separate
reviewer authored21 additional cases with8,654 input bytes and8,698 expected
bytes. Both owners' observable behavior is tested before and after each request;
exact source comparisons preserve unrelated fields, comments and strings. These
are scoped development contracts, not independent benchmark accuracy.
[Worker provenance](../tools/coding-field-owners/HANDOFF.md),
[independent review](../tools/coding-field-owners-review/HANDOFF.md), and
[parent validation](../tools/coding-field-owners/PARENT.md) preserve frozen hashes,
failed iterations, conservative refusals and the completed full-dev gate.


## Explicit percentage changes (2026-10-11 UTC)

The same pinned MathQA TRAIN/DEV files and unchanged exact-label-v3/legacy scorers
are reused. Design reads only Problem text from the first4000 TRAIN rows;160
matching texts total39,443 bytes under the source's Apache-2.0 terms. New authored
38-probe inputs total5,295 bytes, MIT OR Apache-2.0. No downloaded bytes, new model
or dependency. The parent adds78 independent rational/domain/reference checks.
All4475 DEV predictions take only Problem text; options/answers stay evaluator-side.
[Provenance and frozen design](../tools/quantities/percent-change/HANDOFF.md) and
[independent integrated validation](../tools/quantities/percent-change/PARENT.md)
record seven new justified answers, retained failures and compounding assumptions.

## UD English EWT syntax experiment

Pinned r2.18 revision `b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c`; CC-BY-SA-4.0.
Downloaded only official TRAIN (15,034,493 bytes), DEV (1,898,574), README (24,522)
and license (20,014), total 16,977,603 bytes. Official test was not downloaded.
TRAIN has 12,544 sentences/204,578 integer words; DEV 2,001/25,148. Multiword range
and decimal rows are counted separately; the model predicts basic dependencies.
All 68 DEV sentences with TRAIN-identical forms remain scored and are separately
reported. Primary scores use gold word segmentation, not raw-text tokenization.
The parser uses only forms at inference and predicted POS tags internally; gold
annotations stay in training/scoring. One fixed two-epoch tagger and three-epoch
parser fit; no DEV tuning or probability calibration. Raw models/data stay under
`~/.cache/glassmind/continuous-20261010/udsyntax-{source,run}`.
[Provenance and license notice](../tools/reading/syntax/NOTICE.md),
[independent audit](../tools/reading/syntax/PARENT.md).
