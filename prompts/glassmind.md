# Prompt: glassmind, a general semantic language model

This is the one working prompt for glassmind. The earlier prompts in
`prompts/` stay, unedited, as history:

- `symbolic-language-model.md`;
- `keep-improving.md`;
- `general-semantic-model.md`;
- `steer-general.md`;
- `steer-harness.md`.

Read them for context on why things are the way they are. Where they
disagree with this prompt, this one wins. Commit this prompt and the two
untracked steering files in your first commit.

## Where you start

glassmind lives in `~/code/glassmind` (GitHub `gold-silver-copper/glassmind`).
It answers English questions about Rust code with no neural network or LLM
inside, and has started to act as a coding agent. A loop of Opus agents grew
it over 29 rounds. Read these first:

- `~/code/notes/projects/glassmind/progress.md`, the progress log. It is the
  truth about current state; trust it over this section.
- `~/code/notes/projects/glassmind/transcript-analysis-2026-10-10.md`: an
  outside review of everything done so far, with the problems it found.
- `README.md`, `docs/`, and `git log` on the current branch.

As of the evening of 2026-10-10:

- **Merged:** PR #1, the foundation. The `harness` branch carries:
  - the harness skeleton;
  - `glassmind do` with the rustbench adapter;
  - harness tasks as training data;
  - the normalizer model;
  - round 29.
- **Numbers at round 28:** training 99.2% right, validation 98.1%,
  held-out 72.0%; 59,391 syntax tokens, 1,090 rules, 431M agent tokens.
- **rustbench dev, first run:** fix-errors 15%, talk 30%, rustc-error-index
  6.6%; edits, fix-tests and implement 0%.
- **Maybe in flight:** a previous session may have started fixing the review's
  problems (uncommitted changes in `crates/loop`, `checks.rs`,
  `harness_tasks.py`). Check `git status` and the loop service
  (`tools/loop-service.sh status`). Keep what's sound, finish or redo the
  rest, and commit.

## What glassmind is for

**A general semantic language model.** A system that can stand in for the LLM
inside a coding agent and hold a useful conversation, built from three kinds
of material:

1. **Strictly typed Rust code** at the core. Concepts are types, reasoning
   is functions, reading and speaking are grammar. This is the part people
   read.
2. **Many small ML models.** Each has one measured job, is trained from data,
   has a held-out accuracy and calibrated confidences, and can be inspected.
   **Models propose and rank; typed code decides.**
3. **Queryable data.** Facts that don't compress (names, docs, API
   signatures, dictionary entries) live in indexed stores the code looks up.

**No LLM runs inside it.** Opus builds it; glassmind runs deterministically,
with a trace for every answer (`--explain`).

**Its signature front end is a coding harness built on glassmind.** It is a
terminal coding agent of its own, not a plugin for an existing one:

- it reads, searches and edits files;
- it runs `cargo build`, `cargo test` and `clippy`, and reads compiler errors
  and test results;
- it uses `git`;
- it reports what it did and why, and can explain every step;
- it asks when a request is ambiguous and says when something is beyond it,
  rather than guessing.

Chat comes with it: the harness without tools.

## First: measurement you can trust

The review found that today's held-out number can't be trusted:

- agents could read copies of the held-out answers and its style prompt;
- held-out cost decided which changes were accepted;
- two training personas ("hurried", "slack") copy the held-out phrasing;
- agents edited code the scorer depends on.

Read 66–72% as an upper bound. Before more growth, rebuild the measurement:

- **A final test set nothing touches.** Build it so that neither you nor the
  loop ever reads its items:
  - rustbench's `heldout` split (written outside glassmind);
  - human-written requests about Rust code: mine questions from public
    sources such as GitHub issues and discussions, the Rust users forum and
    Stack Overflow, phrased by people, with answers checked by tooling where
    possible;
  - fresh repos glassmind has never seen.

  Store it outside every path agents and the scorer's sandbox can see. Score
  it only at milestones, and report the number, never the items.
- **Select on something else.** Growth and compression rounds accept or
  reject on training plus a selection set (validation, the hidden dev set,
  rustbench `dev`), never on the final set. The old held-out set is now just
  another selection set; say so in reports.
- **Hide all of `~/.cache/glassmind/heldout/`** (and wherever the final set
  lives) from agents and from the sandbox that runs agent-written code. Only
  hiding `final-heldout` left copies readable. Don't name these paths in
  anything agents read.
- **Training data that doesn't imitate the test.** Rewrite or drop the
  "slack" persona. When you want noisier training phrasing, get it from real
  human text, never from the held-out generator's prompt or examples.
- **Protect the scorer.** The scorer, the remedy verifier (with its own
  cargo runner in `crates/loop/src/verify.rs`), the scorer's check results
  and the task generators are off limits to loop agents. They must not be
  able to write them, through code or through the cache. (`checks.rs` is
  glassmind's own harness code, which the loop grows; since 2026-10-10 the
  scorer does not use it.) You,
  the top-level session, may change the scorer deliberately. Each change
  goes in its own commit with the reason, re-scoring the last few accepted
  versions with both the old and the new scorer.
- **Re-baseline.** Report the final-set score of the current system next to
  the old held-out number. That gap is the first real result.

**Then decide whether the core is right.** If the clean score is low and
growth rounds barely move it, the bottleneck is the architecture, not
missing rules. In that case, change the core: for example, let the small
models do the reading and have the typed code only check and act, or switch
parser or formalism. You are the architect, not just the loop's operator.
Every few milestones, step back and ask whether the core is still the right
bet. A rewrite that scores better on the final set is always allowed.

## Fix the loop's mechanics

From the review, each with its measured cost:

- **Duplicate rule IDs** from parallel agents: 7 rejections, $19. Give each
  agent its own ID prefix, or rename on merge.
- **Gating on tiny score differences**: 6 rejections over 0.1–1.7. Use
  a margin or a significance test on the selection set, not a strict
  comparison.
- **The reviewer approves on an empty or unreadable reply.** Make that a
  rejection or a retry.
- **Agents patch single phrasings and exact typos.** Rules used by only one
  right answer grew from 35 to 211, and there are hand-made typo lists in
  `crates/english/src/spelling.rs`. Delete them as general mechanisms (the
  normalizer, the slip reader) cover them. Track "rules used once" and push
  it down.
- **Coding remedies that copy the generator** (`undo_slip` is the inverse of
  `harness_tasks.py`'s `FLIPS`). Make training breakages varied and keep the
  ones used for selection and the final set different from training. A fix
  passes because it is the fix, not because it reverses a known mutation.
- **Duplicate training turn IDs** (16 IDs, 42 rows): deduplicate.

## Machine safety (two near-crashes tonight)

A broken generated test reached 82 GB of RAM plus 56 GB of swap. Then four
orphaned doc tests reached 11 GB each. Both were outside any memory cap, and
the user's terminal has been OOM-killed before.

- **Every cargo build or test on generated, broken or agent-written code**
  runs in a memory-capped scope in its own process group, and the whole group
  is killed on timeout. That covers `harness_tasks.py`, `checks.rs`, remedy
  verification, rustbench runs and the scorer. For example:

  ```
  systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 setsid ...
  ```

  Killing only `cargo` leaves the test binaries running.
- **The loop and any big job** run in `glassmind.slice` (`MemoryHigh 28G`,
  `MemoryMax 36G`, `MemorySwapMax 0`).
- **Limits:** at most 4 agents at once, cargo at `-j 4` per agent and `-j 8`
  overall, `--test-threads 4`. The user's other projects (sememe, rig) can use
  60 GB on their own, so check free memory before heavy work.
- **Never write to `/tmp`** (a RAM disk here), including doc-test temp files:
  set `TMPDIR` to `~/.cache/claude-tmp/` or a gitignored folder for every
  cargo run.
- **Never `pkill -f`** a pattern that matches your own shell. Use exact PIDs or
  `systemctl --user stop`.

## Credentials and models

- Use only Claude Opus 5.5 through the plain CLI on the user's login:
  `claude -p --model claude-opus-5-5 --no-session-persistence`, with
  `ANTHROPIC_API_KEY` and every other API-key variable unset.
- Never read or copy `~/.claude/.credentials.json` or any token, and never
  use the OpenAI, Gemini or other keys in the environment.
- Sandboxed agents must not be able to write `~/.claude` or `~/.claude.json`.

## Learning from data

Outside data is allowed, and finding it is part of the work:

- Wiktionary and Kaikki;
- Wikidata;
- Universal Dependencies;
- the Rust reference and std docs;
- rustc's error index;
- docs.rs and crates.io metadata;
- open-source repos;
- forum and Q&A dumps;
- unit and calendar standards;
- whatever a failure points to.

Record each in `data/README.md` with its version, size and license. For
each source, decide by measurement what it becomes:

- **Types:** closed, logic-heavy knowledge. Units and conversions, dates,
  Rust's grammar of items, error categories. A wrong unit conversion should
  be a compile error.
- **A small model:** large but regular knowledge. Keep a model only if it
  beats a plain table or rule of the same size on the selection set.
- **Queryable data:** independent facts. Index them; don't half-remember
  them in a model.

Build this as a **module factory**: a source plus a short description gives
a typed interface, a store or model, a held-out report and confidences.
Build it when the second or third source needs it, not before. Keep sources
to tens of gigabytes.

**Never depend on flexion** (`~/code/flexion`, under active development):

- no crate dependency;
- no vendored copy;
- none of its tools or training code;
- no edits;
- no PRs or issues.

Read its source for ideas. Don't use the old English crates (`~/code/english/`,
`~/code/linguistics/`) either.

## The models

**One shared trainer, glassmind's own.** It lives in glassmind's crates: data
plus a short description go in, and out comes a small Rust model with
calibrated confidences, a held-out report and an inspectable dump. Start with
cheap, inspectable families: decision lists or trees, linear and
maximum-entropy models, n-grams, averaged perceptrons. Use bigger ones only
when measurement says so. Today `crates/models` holds one hand-tuned edit
model; turn it into this trainer.

A starting list. Reorder it by measured gain on the final set and the
selection set:

| Layer | Model | What it predicts | Data |
|---|---|---|---|
| Spelling | Normalizer | "teh fn" → "the fn"; texting style; splitting identifiers | Wikipedia typo edits, synthetic noise, chat logs |
| Morphology | Inflection and analysis (glassmind's own) | Lemma + features → form; form → lemma + features | Wiktionary |
| Lexical | Category tagger (supertagger) | Each word's likely grammatical category in context | UD treebanks, its own accepted parses |
| Syntax | Parse ranker | Which reading people mean: attachment, scope of "and" | Treebanks, its own parses |
| Semantics | Sense chooser | Which concept a word means here ("crate": package or box) | Wiktionary senses, Wikidata, context |
| Semantics | Role labeler and plausibility | Who did what to whom; what can be renamed or compiled | PropBank/FrameNet-style data, its own worlds |
| Reference | Coreference and entity linking | What "it" means; which item a name refers to | Its own dialogues, Wikidata, the code index |
| Discourse | Intent classifier | Question, command, correction, complaint, chit-chat | Generated and mined conversations |
| Planning | Next action (harness) | Search, read, edit, build, test or ask next | Agent transcripts, git histories |
| Code | Error → fix | Given a rustc error, the likely kind of fix | rustc error index, build-fixing commits |
| Code | Syntax-level code model | Likely next item, API call or name over Rust syntax trees | Open-source Rust |
| Generation | Realization ranker | Which possible sentence sounds most natural | Corpora, its own outputs |

Likely first:

1. the category tagger and parse ranker, against the phrasing gap;
2. error → fix and next action, where the harness starts to stand in for
   an LLM.

**Self-training:** everything glassmind gets right (parses, resolutions,
fixes that passed the tests) becomes training data for these models.

## How it grows

The loop stays the engine:

- growth rounds fix failure groups in worktrees;
- compression rounds delete, merge and generalize;
- acceptance against the scorer.

Change the loop however it needs to change. It also:

- grows the harness from task failures;
- searches for data when a failure group points to missing knowledge;
- trains and retrains models;
- writes harder training tasks as easy ones are solved, with outcomes checked
  by tooling.

The training material is:

- generated conversations;
- harness tasks from real repos at fixed commits;
- rustbench's `dev` split.

**Never rustbench's `heldout` split**, which belongs to the final set.

Knowledge goes in the layer where it is reusable. Name every type, rule and
model after what it is, with a doc comment written from its uses; agents
read this code too.

## Scoring

- **Wrong is far worse than "I don't know."** A confident wrong answer or
  a broken edit costs much more than an honest refusal or a question back.
- Score end-to-end tasks with tooling-checked outcomes: answers, edits that
  compile and pass tests (including hidden ones), fixes, multi-turn
  conversations, requests that need several modules.
- Keep the QA score comparable across history while adding harness and model
  scores beside it.
- The headline chart shows, against Opus tokens spent:
  - final-set and selection-set scores, one line per task kind;
  - code size, model size and data size.

## You may change this prompt

It is your working direction, not a contract. Change any part, including the
scorer and the architecture, when evidence says so. Put the reason and the
evidence in the commit and the progress log. Rewrite sections cleanly rather
than appending notes; results go in the progress log.

Only these are fixed:

- no LLM inside at run time;
- readable typed code at the core;
- honest measurement: a final set nothing tunes on, and agents never reading
  held-out material;
- the machine-safety and credential rules above.

## How you work

- Run for as long as it keeps improving. Don't ask the user anything and
  don't ask about time or token cost.
- Keep the loop a resumable service with checkpoints. Store every expensive
  LLM output so a restart reuses it. Wait out usage limits instead of
  counting them as failures.
- While a loop round runs, do other useful work (the trainer, data sources,
  harness code) instead of sleeping on it.
- Non-test code must not panic (no `unwrap`, `expect`, indexing or `panic!`),
  enforced by the workspace clippy lints. Use few, meaningful tests and no
  snapshot or exact-output tests. Edit code with the Edit tool, not
  `str.replace` scripts that fail silently.
- Ship PRs to `gold-silver-copper/glassmind` at milestones. Merge each with
  `gh pr merge --merge` once CI is green, then check out the latest `main`.
  The next milestone, PR #2, has three parts:
  - the clean measurement;
  - the loop fixes;
  - the harness passing three kinds of coding task (fix a compiler error,
    rename across a crate, make a failing test pass) on the final set, with
    at least one model showing a measured gain there.
- Keep `~/code/notes/projects/glassmind/progress.md` current: what's done,
  current numbers, decisions and why, what's next. Write it so a fresh
  session can pick up from it alone.
- When the session ends, stop every process, service, worktree and
  background shell you started, after a final checkpoint.

## Each milestone report says

- The chart, and plainly whether the final-set score is still rising.
- The gap between selection-set and final-set scores, and what explains it.
- What the harness can do now, with real transcripts and one `--explain`
  trace.
- The modules and models: source, what each became and why, size, and
  held-out accuracy.
- How much of the system is code, models and data, and how that changed.
- What it still can't do, what limited growth, and whether the core is still
  the right bet.
- Copy-pasteable commands starting with `cd ~/code/glassmind` to start the
  harness on a repo, chat, explain an answer, run rustbench, and run one
  loop round.
