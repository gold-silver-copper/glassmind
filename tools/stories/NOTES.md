Standalone conversation/story track, 2026-10-10
================================================

`crates/stories` builds the Rust library and `glassmind-stories` binary. It has
no dependency on the main CLI, a language-model runtime, or flexion. The small
Python adapter downloads/extracts training members, partitions episodes, invokes
Rust, and scores responses. It contains no answer reasoning.

The world has typed entity, location, event, time, and action representations.
It tracks movement, inventory, transfers, location histories, negative and
disjunctive evidence, singular/plural discourse focus, and object `it` references.
Relations use inverse edges, transitive size reachability, spatial displacement
composition, and breadth-first path finding. Explicit timestamps are ordered
independently of assertion order. Kind rules support deduction; color induction
requires unanimous observed members of a kind and marks its answer defeasible.
Unrecognized assertions taint the story, forcing refusals until reset.

The optional 979-byte categorical action model is **learned in Rust from
assertions in training episodes**, with no questions, gold answers, task IDs,
or supporting-fact IDs. It counts `(prior state, action kind, target)` events
across 5,015 episodes and predicts only after at least three observations with
no conflicting outcome. Queries about motives additionally require a matching
observed action and the state preceding that action. The reply identifies this
as an inferred association, not logical proof. With no model loaded, these
questions are refused. The learned artifact is reproducible rather than a
handwritten mapping of benchmark answers.

Measurement
-----------

The fixed sample is 20 questions per task, selected by SHA-256 rank. Before this
batch the existing engine scored 254 correct / 127 refusal / 19 wrong on those
400 questions. Final: **397 / 3 / 0**. Removing the learned action model gives
**377 / 23 / 0**, on the identical sample. `sample-v2.json` records the intermediate
371 / 29 / 0 result before the spatial parser fix and motivation model.

Full development: **3,966 correct / 18 refusal / 3 wrong out of 3,987** (99.47%
correct). This is a development measurement, not a held-out result. No official
test examples were opened, parsed, extracted, or evaluated.

| Task | Correct | Refusal | Wrong |
|---|---:|---:|---:|
| 1 single supporting fact | 200 | 0 | 0 |
| 2 two supporting facts | 200 | 0 | 0 |
| 3 three supporting facts | 200 | 0 | 0 |
| 4 two argument relations | 200 | 0 | 0 |
| 5 three argument relations | 200 | 0 | 0 |
| 6 yes/no | 200 | 0 | 0 |
| 7 counting | 200 | 0 | 0 |
| 8 lists/sets | 200 | 0 | 0 |
| 9 negation | 200 | 0 | 0 |
| 10 indefinite knowledge | 197 | 0 | 3 |
| 11 basic coreference | 200 | 0 | 0 |
| 12 conjunction | 200 | 0 | 0 |
| 13 compound coreference | 200 | 0 | 0 |
| 14 time reasoning | 200 | 0 | 0 |
| 15 deduction | 200 | 0 | 0 |
| 16 induction | 182 | 18 | 0 |
| 17 position | 200 | 0 | 0 |
| 18 size | 198 | 0 | 0 |
| 19 paths | 200 | 0 | 0 |
| 20 motivations | 189 | 0 | 0 |

The 18 refusals are conflicting color examples. All three wrong answers are
duplicate disjunctions (`A or A`) in task 10, for which the labels say `maybe`
and set semantics gives `yes`. These remain counted wrong. No special rule was
added to imitate the labels. Full development failures were inspected only to
explain this discrepancy; the reasoner was not changed after that measurement.

Build and strict targeted clippy passed. Nine one-off interaction checks also
passed, including new entity names, `it` through transfers, temporal ordering,
group coreference, spatial composition, event-plus-question JSONL requests,
unseen-state refusal, and unsupported-assertion refusal. No unit tests added.

Provenance and split
--------------------

Source: Facebook bAbI tasks 1–20, **v1.2, English 1k**, copyright 2015-present
Facebook, Inc.; **Creative Commons Attribution 3.0 Unported (CC-BY-3.0)**.
The [Facebook dataset card](https://huggingface.co/datasets/facebook/babi_qa)
identifies the license. The pre-existing local LICENSE.txt agrees. Attribution
also applies to the derived learned model; code uses the workspace license.

The original official URL
`http://www.thespermwhale.com/jaseweston/babi/tasks_1-20_v1-2.tar.gz` returned
HTTP 404. The alternate Meta host returned 403. The pre-existing archive was
verified byte-for-byte against a fresh download from
`https://s3.amazonaws.com/text-datasets/babi_tasks_1-20_v1-2.tar.gz`.
The adapter uses this mirror and pins the checksum before extraction:

* Archive: **11,745,123 bytes**;
  SHA-256 `84f5296ab9a1ad0dc9464e08c491d65cd08830fca3acae9ab86f75e0fb81573c`.
* Extracted: **20 English training members, 2,947,923 bytes, 20,000 questions**.
* Development: **1,252 episodes, 3,987 questions**.
* Training: **5,015 episodes, 16,013 questions**.

Extraction opens only regular members matching
`tasks_1-20_v1-2/en/qa[0-9]+_*_train.txt`. It skips all other members without
opening them and never calls `extractall`. No test extraction option exists.
`source.json` records all twenty training member sizes and hashes.

For each task, complete episodes (reset at line number 1) are ranked by
SHA-256 of `glassmind-stories-v1/{task}/{zero_based_episode_index}`. The first
`floor(episode_count/5)` episodes form dev; the rest form train. This is an
80/20 split by whole episode, with integer rounding. Variable episode lengths
explain the 16,013/3,987 question counts. Questions are expanded only after the
split, so contexts from the same episode cannot cross partitions. The dev
sample takes the first 20 questions per task ranked by SHA-256 of
`glassmind-stories-v1/sample/{task}/{episode}/{zero_based_turn}`. Turn indices
include question lines. Prior gold answers and supporting facts never enter
the model process. Set answers are compared order-insensitively; path answers
retain their order. All other answers use case-insensitive exact match.

Reproduce
---------

From this worktree:

```sh
bash tools/stories/reproduce.sh
```

The script sets TMPDIR inside the worktree, uses one cargo job, and runs build,
clippy, Rust event-only model training, the fixed sample, and full dev in named
8 GiB / zero-swap scopes with 600-second process-group timeouts. It waits for
each whole scope before proceeding and writes fresh artifacts under `scratch`.
No cargo tests run, so test-thread configuration is inapplicable. For just
evaluation with the existing build:

```sh
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 python3 tools/stories/babi.py eval --model tools/stories/action-model.json --sample 20 --report scratch/stories/sample.json
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=5s 600 python3 tools/stories/babi.py eval --model tools/stories/action-model.json --sample 0 --report scratch/stories/dev.json
```

Interactive use:

```sh
.cache/stories-target/debug/glassmind-stories --model tools/stories/action-model.json
```

Enter one assertion or question per line; `/reset` starts a new story. `--jsonl`
accepts `{"story":["Nora went to the atrium."],"question":"Where is Nora?"}`,
which resets before loading the story. Stateful requests use `{"event":"..."}`,
`{"question":"..."}`, or `{"reset":true}`. A request containing both `event`
and `question` applies the event before answering. Replies contain `answer`
(string or null) and `reason`. Model training is also available directly as
`glassmind-stories --train model.json` with assertion-only story JSONL on stdin.

Limits and unimplemented work
-----------------------------

This is a controlled-English story reasoner, not unrestricted conversation.
The parser is handwritten; it was not learned from broad English corpora.
Pronouns use recent discourse focus, spatial edges assume axis-aligned unit
displacements, sizes are abstract strict ordering rather than physical geometry,
and inventory starts empty in a complete story. Temporal support covers the
four relative day parts used here, not calendar arithmetic. Induction and
motivation are defeasible. Dialogue preferences, external factual knowledge,
learned parse ranking, main-CLI integration, persistence, and multilingual input
were not implemented. No held-out or general-English accuracy is claimed.

All changes are uncommitted; the coordinator can integrate the standalone crate,
adapter, provenance/model files, and reports. Git metadata was not accessible
inside this worker's filesystem view. No existing main CLI, loop, README,
historical prompt, or data/README was edited.
