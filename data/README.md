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
