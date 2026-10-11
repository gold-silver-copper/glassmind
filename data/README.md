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
