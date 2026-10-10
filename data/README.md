# Data sources

Everything glassmind is trained or measured on, where it came from, and what
it became. The data itself lives outside git (`~/.cache/glassmind/`), and
held-out data is kept where loop agents cannot read it.

## Code to talk about (the Rust world)

| source | version | size | license | use |
|---|---|---|---|---|
| top_down_shooter_game (tds), ck, ghtui, memochat | the user's repos at the commits in `~/.cache/glassmind/sources/SOURCES` | 3.5–476 MB | the user's | QA training |
| bevy_noodle (noodle), fux | the user's repos at the commits in `heldout/sources/SOURCES` | 9–45 MB | the user's | QA held-out |

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

## Requests (generated)

All requests were written by Claude Opus 5.5 (`tools/datagen/phrase.py`),
working backwards from tasks with answers computed by tooling. No request
was written by a person.

| set | turns | generators | about |
|---|---|---|---|
| training | 6,926 (6,696 QA + 230 coding) | terse, chatty, newcomer, hurried personas | training repos and crates |
| validation | 267 | the training personas | held-out repos (reported, never used to accept) |
| held-out | 725 (624 QA + 101 coding) | a separate "query log" generator prompt | held-out repos and crates |

## Toolchain

`rustc`/`cargo` from the stable toolchain run the coding tasks' checks; their
messages are the problems' details.
