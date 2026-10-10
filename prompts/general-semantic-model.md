# Prompt: grow glassmind into a general semantic language model

## Where you start

glassmind exists and works. Read these first:

- `README.md` and `docs/` in `~/code/glassmind`;
- `prompts/symbolic-language-model.md`, the brief it was built from;
- `~/code/notes/projects/glassmind/progress.md`, the progress log.

Today glassmind answers English questions about Rust code with no neural
network or LLM inside. A loop of Opus agents grows its grammar, words and
concepts. It reached 98.6% on training conversations and 71% on held-out
ones.

It may still be running its loop as a user service. Check
`tools/loop-service.sh status`. Let the current round finish, stop the
service at a round boundary, and take over from its state. Keep everything
that works. This prompt changes where glassmind is going, not what it is
made of.

## Where it is going

**A general semantic language model:** a system that can stand in for the LLM
inside a coding agent and hold a useful conversation, built from three kinds
of material:

1. **Strictly typed Rust code.** The concepts it can think about are types.
   How it reasons is functions. How it reads and speaks is grammar. This
   stays the core and the part people read.
2. **A collection of small ML models.** Each has one measured job: inflecting
   words, ranking readings, correcting typos, guessing which module a request
   needs, predicting a likely next step. Each is small, trained from data, has
   a held-out accuracy and a calibrated confidence, and can be inspected. The
   user's `~/code/flexion` is the model for this: a Wiktionary dump goes in,
   a small Rust crate with a confidence for every answer comes out.
3. **Queryable data.** Facts that don't compress (names, documentation, crate
   APIs, dictionary entries, encyclopedic facts) live in indexed stores the
   code looks up, not in models and not in code.

**Word forms come from flexion.** Use `~/code/flexion` (GitHub
`gold-silver-copper/flexion`) for inflection. When glassmind needs a language
flexion doesn't have yet, add that language to flexion. Don't use the old
hand-written English crates (`english`, `english-core`, `english-phrase`,
`english-ccg`, anything in `~/code/english/` or `~/code/linguistics/`). When
flexion is wrong or missing something glassmind needs, fix it upstream in
flexion (a PR there) instead of working around it in glassmind.

**No LLM runs inside it.** Opus builds it; glassmind runs without one,
deterministically, with a trace for every answer (`--explain`). That is the
one thing that makes this project what it is.

## The signature front end: a coding harness built on glassmind

Build a coding agent whose "brain" is glassmind. It is a custom harness, not
a plugin for an existing one:

- a terminal session over a repository: the user asks questions, asks for
  changes, and points at failures;
- glassmind reads the request, plans, and acts through the harness's tools:
  read and search files, run `cargo build`, `cargo test` and `clippy`, read
  compiler errors, propose and apply edits, use `git`;
- it reports what it did and why, in English, and can explain every step;
- it never edits on a guess: when a request is ambiguous it asks;
- it says when something is beyond what it knows rather than inventing an
  answer.

The harness is where "general" gets tested, because coding work mixes
everything: understanding a request, knowledge of the language and its
libraries, reading errors, planning several steps, and holding a
conversation. A plain chat mode comes along with it (the harness without
tools), and so does the current `ask`/`chat` CLI.

Grow it the way glassmind grew: start with the kinds of coding task that are
common and checkable, and widen. For example: answer questions about the code
(today's world), rename and move things, fix a compiler error, add a field and
update its uses, make a failing test pass, add a test, explain an error.

## Learning from data: find it, then decide what it becomes

The old brief allowed no outside data. That restriction is lifted. Part of
the work is **searching for data sources when glassmind needs them**:
Wiktionary and Kaikki dumps, Wikidata, the Rust reference and standard
library docs, docs.rs and crates.io metadata, rustc's error index, open
source repositories, Stack Overflow dumps, unit and calendar standards, and
whatever else a failure points to. Record each source, its version, size and
license in `data/README.md`.

For each source, decide what it becomes, and decide by measuring:

- **Types.** Closed, stable, logic-heavy knowledge belongs in the type system:
  measurement units and their conversions, dimensions, dates and calendars,
  numbers, Rust's own grammar of items and types, error categories. A wrong
  unit conversion should be a compile error, not a low-confidence guess.
- **An ML model.** Large but regular knowledge, where a small model predicts
  most cases from patterns: inflection, spelling and typo correction,
  which reading of an ambiguous sentence people mean, which crate a name
  probably comes from, which fix a common compiler error usually needs.
  Keep the model only if it beats a plain table or rule of the same size on
  held-out data.
- **Queryable data.** Knowledge that is mostly independent facts: names,
  definitions, API signatures, documentation, encyclopedic facts. Index it;
  don't compress it into a model that half-remembers it.

Mixed sources split. Wiktionary gives inflection (a model), definitions and
translations (data), and part-of-speech facts that feed the grammar (types or
data).

Build this as a reusable machine, not one-off code: a **module factory** that
turns a source plus a short description into a module with:

- a typed interface;
- its store and models;
- a held-out report;
- confidences.

Adding the hundredth source should mostly be configuration. Watch disk and
memory: there is room for tens of gigabytes of sources, not terabytes, and a
download or a training run must never take down the machine.

## The models it needs

Inflection is only the bottom layer. The pattern for every model is: **models
propose and rank, typed code decides.** The grammar and the types define
what is possible; a model says which possibility is most likely. Answers stay
checkable while data handles the messy parts.

A starting list, bottom to top. Reorder it by measured value and add what
the failures call for:

| Layer | Model | What it predicts | Data |
|---|---|---|---|
| Spelling | Normalizer | "teh fn" → "the fn"; texting style; splitting identifiers | Wikipedia typo edits, synthetic noise, chat logs |
| Morphology | Inflection (flexion) and its inverse, analysis | Lemma + features → form; form → lemma + features | Wiktionary |
| Lexical | Category tagger (a "supertagger") | Each word's likely grammatical category in context | Universal Dependencies treebanks, plus glassmind's own accepted parses |
| Syntax | Parse ranker | Which of the grammar's readings people mean: attachment ("rename the field in Player"), scope of "and" | Treebanks, plus its own parses |
| Semantics | Sense chooser | Which concept a word means here ("crate" as a Rust package vs. a box) | Wiktionary senses, Wikidata types, context |
| Semantics | Role labeler and plausibility model | Who did what to whom; what can be an agent of "compile" or a target of "rename" | PropBank/FrameNet-style data, its own worlds |
| Reference | Coreference and entity linking | Which earlier thing "it" means; which item a name refers to | Its own dialogues, Wikidata, the code index |
| Discourse | Intent / speech-act classifier | Question, command, correction, complaint, or chit-chat | Generated conversations |
| Planning | Next-action model (for the harness) | Given the request and state, the likely next step: search, read, edit, build, test, ask | Agent transcripts, git histories |
| Code | Error → fix model | Given a rustc error, the likely kind of fix | The rustc error index, commits that fix build failures in open-source repos |
| Code | Syntax-level code model | Likely next item, API call or name over Rust syntax trees | Open-source Rust (docs.rs, crates.io) |
| Generation | Realization ranker | Which of the grammar's possible sentences sounds most natural | Corpora, plus a fluency model over its own outputs |

The ones likely to matter most first:

1. **The category tagger.** It prunes the parser's search, making parsing
   both faster and more accurate. CCG parsers are only practical because of
   supertaggers.
2. **The parse ranker.** Understanding sloppy, varied phrasing is
   glassmind's biggest gap: held-out phrasing styles score far below
   training ones.
3. **Intent and entity linking.** They route requests and resolve names: the
   glue that makes many modules feel like one system.
4. **Next action and error → fix.** This is where the harness starts to
   stand in for an LLM.

**Self-training.** Everything glassmind gets right (parses, resolutions,
fixes that made the tests pass) becomes training data for these models. As
the typed system grows, the models improve, and they make the typed system
faster and less brittle.

## Measuring progress

You own the scorer, and you may change it. Measurement is what keeps this
honest, so treat it with care rather than with rules:

- Score what the system is for: end-to-end tasks with answers checked by
  tooling. These include:
  - questions answered exactly;
  - edits that compile and pass tests;
  - fixes that make a failing test pass;
  - conversations whose later turns depend on earlier ones;
  - requests that need several modules at once.
- Keep glassmind's principle: **wrong is far worse than "I don't know"**. A
  confident wrong answer or a broken edit costs much more than an honest
  refusal.
- Keep a **held-out set** the loop never trains on, written by a different
  generator and phrasing style, about different repositories and sources.
  The biggest gap glassmind found so far was phrasing style, not unfamiliar
  code: hold out styles, not just items. Agents shouldn't be able to read
  it.
- When you change the scorer, re-score the last few accepted versions with
  both the old and the new scorer and record both, so the history stays
  comparable. Change it because it measures the goal better, never to make a
  number look better.
- The headline is a chart over the whole run: held-out score on each kind of
  task against Opus tokens spent, with code size, model size and data size
  on the same timeline.

## How it grows

The existing loop is the starting point:

- growth rounds: agents in worktrees fix groups of failures;
- compression rounds: delete, merge and generalize;
- acceptance against the scorer.

Change it however it needs to change. It now also:

- **searches for data** when a failure group points to missing knowledge, and
  builds modules through the factory;
- **trains and retrains models** as data and code change, keeping a model
  only while it beats the alternatives;
- **grows the harness** from task failures, like the grammar;
- **writes its own training tasks**: harder conversations and coding tasks as
  the easy ones are solved, always with answers computed by tooling.

Knowledge goes where it belongs: language in the language crates, each kind
of data in its world or module, and only language-neutral meaning in the
meaning layer. Name every type, rule and model after what it is, with a doc
comment written from its uses; agents read this code too.

## You may change this prompt

This prompt lives at `prompts/general-semantic-model.md` and is your working
direction, not a contract. Very little here is fixed. Change any of it,
including the scorer and the architecture, when evidence says you should:

- **Say why.** Each edit goes in a commit with its reason and evidence (a
  measurement, a failed experiment). Record it in the progress log.
- **Keep it a prompt.** Rewrite sections cleanly rather than appending notes.
  Results go in the progress log and the reports.
- The only things not to change are the point of the project:
  - no LLM inside at run time;
  - readable, typed code at the core;
  - honest measurement.

## Practicalities learned so far

- Every Opus call goes through one wrapper:
  - a concurrency limit shared across processes;
  - waiting out usage limits instead of counting them as failures;
  - a ledger row per call;
  - stopping child calls when the parent stops.
- Store every expensive LLM output (generated data, agent patches) so a
  restart reuses it instead of paying again.
- Judge a change after its effects settle (retrain the models it touches,
  rerun what it affects), not from a first noisy measurement.
- Generated teaching data needs a consistency mechanism: show the generator
  the conventions already in use, or contradictions pile up.
- Run the loop as a resumable service with checkpoints, and keep the progress
  log current, so any session can pick the work up.
- Use only Claude Opus 5.5 (`claude -p --model claude-opus-5-5`) on the
  user's Claude Code login, never an API key from the environment.

## How you work

- Run for as long as it keeps improving. Don't ask about time or token cost,
  and don't ask the user for anything: the work needs no input from them.
- Protect the machine:
  - at most 4 agents at once;
  - cargo with `-j 8` and `--test-threads 8`;
  - never use all cores or a runaway amount of memory;
  - never write to `/tmp` (use `~/.cache/claude-tmp/` or a gitignored
    folder).
- Non-test code must not panic: no `unwrap`, `expect`, indexing or `panic!`,
  enforced by the workspace clippy lints.
- Few, meaningful tests; no snapshot or exact-output tests.
- Ship progress as PRs to `gold-silver-copper/glassmind` at milestones (the
  harness doing its first real task, the module factory, each new kind of
  task working). Merge each with `gh pr merge --merge` once CI is green, then
  check out the latest `main`.
- Keep `~/code/notes/projects/glassmind/progress.md` current: done, current
  numbers, decisions and why, next.
- When the session ends, stop every process, service, worktree and background
  shell you started, after a final checkpoint.

## What each milestone report says

- The chart, and plainly whether the held-out score is still rising.
- What glassmind can do now in the harness, with real transcripts and one
  `--explain` trace.
- The modules: each source, what it became (types, model or data), why, its
  size and held-out accuracy.
- How much of the system is code, models and data, and how that split
  changed.
- What it still can't do, what limited growth, and what comes next.
- Copy-pasteable commands starting with `cd ~/code/glassmind` to:
  - start the harness on a repo;
  - chat;
  - explain an answer;
  - run one loop round.
