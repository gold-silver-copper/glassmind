Change of direction for glassmind.

Read `~/code/glassmind/prompts/general-semantic-model.md` now. From here on it is your working direction, in place of `prompts/symbolic-language-model.md`. Commit it to the repo with this message as the reason.

In short:

- **The goal grows.** glassmind becomes a general semantic language model: strictly typed Rust at the core, plus many small ML models, plus queryable data stores, with no LLM inside at run time.
- **Its signature front end is a coding harness built purely on glassmind.** It is a custom terminal coding agent that answers, edits, builds, tests and fixes errors in a repo, and explains every step. It can stand in for the LLM in a coding agent. Chat comes with it.
- **Outside data is now allowed, and finding it is part of the job.** For each source, decide by measuring whether it becomes:
  - types (measurement units, dates, Rust's item grammar);
  - a small model;
  - an indexed store.

  Build a module factory so adding a source is mostly configuration.
- **Never depend on flexion** (`~/code/flexion`, under active development): no dependency, no copy, none of its tooling, no edits or PRs. Read its source for ideas and build glassmind's own word-form models with glassmind's own model trainer. Don't use the old hand-written English crates.
- **The models span the whole stack, not just inflection:**
  - spelling;
  - category tagging;
  - parse ranking;
  - word senses and roles;
  - coreference and entity linking;
  - intent;
  - next action;
  - error → fix;
  - code;
  - realization.

  Models propose and rank; typed code decides. Train them on outside data and on everything glassmind gets right.
- **Very little is fixed.** You own the scorer and this prompt, and may change both when evidence says so, with the reason in the commit. Keep the old score alongside a new one so history stays comparable. The only fixed points:
  - no LLM at run time;
  - readable typed code at the core;
  - honest measurement, with a held-out set you never train on (hold out phrasing styles, not just items).

First steps:

1. Let the current loop round finish, then stop the loop service at a round boundary.
2. Record the current numbers in `~/code/notes/projects/glassmind/progress.md`.
3. Merge or open a PR for the work so far.
4. Then start on the harness's first real coding task and the module factory, and extend the loop to grow both.

Keep going for as long as it keeps improving, and don't ask me anything.
