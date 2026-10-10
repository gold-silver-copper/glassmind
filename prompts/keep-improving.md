# Prompt: keep improving glassmind, with no method rules

## Where things stand

glassmind is a system you talk to in English about Rust code, with no neural
network inside: its knowledge is Rust source (meaning types, an English
grammar, a Rust world), grown by an improvement loop of Opus agents. The
original brief is `prompts/symbolic-language-model.md`; read it for the
architecture and the deliverables, but **its method bans no longer apply**
(see "What you may do").

State at the start of this work (2026-10-10):

- **Repo**: `~/code/glassmind`, GitHub `gold-silver-copper/glassmind`. Draft
  PR #1 (branch `foundation`) holds the skeleton, data pipeline, oracle,
  scorer, seed and loop. CI is green on it.
- **The loop's own clone**: `~/.cache/glassmind/loop/repo`, branch `loop`.
  It has 26 rounds of accepted agent changes on top of `foundation` (last
  accepted commit `11d5556`). These are **not yet in PR #1**.
- **Loop state**: `~/.cache/glassmind/loop/state` (`config.json`,
  `baseline.json`, `history.jsonl`, `logs/loop.log`, a `STOP` file). The
  installed loop binary is `~/.cache/glassmind/loop/bin/glassmind-loop`.
  Resume with `tools/loop-service.sh start STATE BIN`; it continues at round 27.
- **Data**: training `~/.cache/glassmind/data/train` (6,696 turns: three
  training personas plus a "hurried" persona added at round 11), sources in
  `~/.cache/glassmind/sources/{tds,ck,ghtui,memochat}`. Held-out (624 turns,
  its own generator prompt `heldout/generator-logs.md`, repos noodle and fux)
  and validation (267 turns, training personas on held-out repos) live in
  `~/.cache/glassmind/heldout/`.
- **Results at round 26** (399M agent tokens): training 98.6% right,
  validation 97.0%, held-out 71.2%; code 19,094 → 58,319 syntax tokens,
  about 1,000 rules. Held-out was still rising when the loop was stopped.
  The gap between validation and held-out is writing style, not repos.
- **Progress log**: `~/code/notes/projects/glassmind/progress.md`. Read it
  first; keep it updated after every milestone.
- **A demo** of the round-26 build: `~/.cache/glassmind/demo.sh`.

## Problems found by using it

- **Replies that lost their words.** Some refusals print "(I have no words
  for this reply: …)". Compression rounds deleted the English sentences for
  them, because the scorer checks only the JSON answer, never the text.
- **The ▮ mark lands at the end** of the input instead of where
  understanding stopped ("which structs have the bouncy thing▮").
- **Phrasing holes**: "show context" works, "show me the context" doesn't.
- **Edit proposals don't say where or how many places change**, and the CLI
  has no `--apply` (the loop grew proposals, not the code that writes them).
- **Typos are handled by listing misspellings** one by one as word forms,
  which only helps where the same typos recur.
- **Rule bloat**: the code tripled, with many phrase-specific rules ("where
  does X live", "in the codebase"). The size price (0.01 per syntax token)
  is probably too low.
- **Nothing has been tested on a human-written request.** Every request,
  including the held-out set, was written by Opus.

## What you may do

There are no method rules. Improve glassmind however you judge best, and
judge every change by measured results. In particular, all of these are
allowed if they help:

- fuzzy or noisy-channel matching of words and names, a learned typo model,
  spelling correction;
- outside data: BabyLM (in `~/code/sememe/data/babylm/`), other corpora,
  word lists, anything you find useful;
- learned probabilities on rules (counted by EM or otherwise), a different
  grammar formalism (CCG, PMCFG, anything), a different parser;
- changing the meaning types, the layers, the loop, the scorer, the answer
  format, the data generators, the tests, the CLI.

Update the loop to match: its reviewer prompt (`crates/loop/src/rounds.rs`,
`REVIEW`) and `docs/growing.md` still forbid keyword spotting and fuzzy
matching. Replace those bans with what actually matters: no faked answers,
knowledge in the layer where it is reusable, readable named rules.

## What stays

- **Agents never see the held-out data**, and nothing tunes on it directly.
  Without that, the numbers mean nothing. If you add a human-written held-out
  set, protect it the same way.
- **Only Claude Opus 5.5, through the plain `claude` CLI on the user's
  login**: `claude -p --model claude-opus-5-5 --no-session-persistence`,
  with `ANTHROPIC_API_KEY` and the other API-key variables unset. No API
  keys, no other models, no separate auth or tokens.
- **Machine safety.** A memory spike has killed the user's terminal
  multiplexer before.
  - Run the loop and any big job in its own capped systemd slice, e.g.
    `systemd-run --user --slice=glassmind.slice -p MemoryHigh=24G -p MemoryMax=32G -p MemorySwapMax=0 -p OOMScoreAdjust=800`.
  - Cap cargo at `-j 8` and tests at `--test-threads 8`, and keep agents'
    builds within that (four agents each building `-j 8` was the heaviest
    load). At most 4 agents at once.
  - Never write to `/tmp`; use `~/.cache/claude-tmp/` or a gitignored
    folder in the repo.
  - Never `pkill -f` with a pattern that matches your own shell's command
    line; it killed this work's shell twice. Match exact PIDs or use
    `systemctl --user stop`.
  - The user's other project, sememe, can use 64 GB on its own. Check free
    memory before starting heavy work.
- **Non-test code must not panic** (no `unwrap`, `expect`, indexing,
  `panic!`), enforced by the workspace clippy lints.
- **Few, meaningful tests**, no snapshot or exact-output tests.
- **Determinism**: the same sources, conversation and input give the same
  output.
- **When finished**, stop every process, service, worktree and background
  shell you started.

## What to do

You own this fully, including the order. A reasonable path:

1. **Bring the loop's work into the PR.** Merge branch `loop` from
   `~/.cache/glassmind/loop/repo` into `foundation` and push, so PR #1
   carries the 26 rounds. Check CI.
2. **Measure what matters**, so the loop can't quietly break it:
   - every reply must have words, and what the system says should read back
     to the same meaning;
   - the ▮ mark must sit where understanding stopped (label it in the data);
   - edit proposals must state the changes; add `--apply` and test it on a
     fixture repo;
   - a human-written held-out set if one can be had without asking the
     user (for example requests mined from public GitHub issues and review
     comments about the source repos, labelled by the oracle). Report
     plainly whether any human-written requests were tested.
3. **Make understanding general rather than listed.** A learned typo model
   in place of the misspelling lists; rule probabilities for choosing between
   readings; BabyLM or other real text as an extra cost term on the English
   layer (bits to write it down), if it measurably helps held-out. Delete
   what each of these makes redundant.
4. **Fight bloat**: try a higher size price, more frequent compression,
   and briefs that ask agents to replace phrase-specific rules with general
   ones. Track rules, code size and readings per sentence against coverage.
5. **Run the loop to its plateau**: until the held-out score has not
   improved for many rounds. It waits out usage limits on its own (429s are
   retried every 10 minutes); if rounds still come back empty, remove them
   from `history.jsonl` before trusting the stopping rule.
6. **Optional growth tests**, if time allows: a second world (Markdown docs
   or a SQLite database) or a second human language, to show the meaning
   layer is neither code-shaped nor English-shaped.

## Deliverables

- PR #1 with all of the above, merged with `gh pr merge --merge` once CI is
  green, then the latest `main` checked out.
- The main result chart: training, validation and held-out scores against
  agent tokens spent, with code size on the same timeline, and a plain
  answer to whether the held-out score keeps rising or levels off. Mark the
  points where training data was added (rounds 10 and 11) and where the
  method rules were lifted.
- A readability check: number of rules, how often each is used, code size
  against coverage.
- A final report: repo and PR links; the chart and its answer; the gap
  between training-style and held-out scores; what the lifted rules made
  possible and what it cost; one real `--explain` trace; what limited growth
  and what should come next; how you checked it works, with copy-pasteable
  commands starting with `cd ~/code/glassmind` to ask a question, try Tab
  completion, continue a saved conversation, propose and apply an edit, and
  run one loop round.
