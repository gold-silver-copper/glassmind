# Continuous development checkpoint

Goal active: follow `prompts/grow-continuously.md` until the user stops it.

## Current batch

Starting source: `f998e27` on `harness`; merged upstream main `54fc197`.
Startup audit: no open PRs, loop inactive, no existing glassmind workers,
ChatGPT login confirmed. Resource audit: 109 GiB available, 1.1 TiB disk free.
No held-out evaluation is scheduled in this first batch; development only.

Experiments selected before implementation:

1. Coding: inspect training/dev-visible repair failures and implement a
   reusable bounded, verified repair improvement. Baseline rustbench full
   dev: 160 right, 602 refused, 3 wrong / 765. Accept only measured gains
   without new wrong answers; never inspect hidden test contents.
2. Quantities: establish an independently sourced train/dev word-problem
   benchmark, then improve general parsing if evidence supports it.
   Existing synthetic baseline: 3056 right (including 762 expected
   refusals), 60 unsupported, 0 wrong / 3116. Preserve that baseline and
   report external data as its own series.
3. Conversation integration: expose the existing verified quantities engine
   through `glassmind talk`, retaining name/preference priority, persistence
   and last-message recall. Baseline talk refuses supported arithmetic,
   conversions and dates without an intent model. Accept correct typed
   results and unchanged memory behavior on a fixed mixed conversation.

## Work ownership and resources

Coordinator owns CLI conversation integration, shared manifests, reports
and merges. Coding and quantities workers use isolated worktrees and are
limited to one cargo compilation job each. Workers leave changes uncommitted.
Worker root: `/home/gsc/.cache/glassmind/continuous-20261010`. Coding launcher
PID 1493627, scope `glassmind-continuous-coding.scope`; quantities launcher
PID 1493666, scope `glassmind-continuous-quantities.scope`. Each `*-run/`
contains process metadata, JSONL events, stderr, final summary and disposable
auth symlink. Bubblewrap masks other code, private data and account history;
workers receive GPT-6 Astra fast through ChatGPT login with API-key env removed.
Both scopes verified active after launch. Recheck before acting on PIDs.
The old loop remains stopped; its checkout and state must be preserved.

## Queue after this batch

Conversation integration completed and locally verified: mixed conversation
18/27 → 27/27 correct, zero wrong, both with/without frozen intent model;
41/41 memory checks; full 3116 synthetic quantity questions through talk retain
3056 correct, 60 refused, zero wrong (762 correct expected refusals).
Workspace fmt/clippy/build and strict 4/4 fixture smoke passed. Source commit
is recorded in the chat score journal; see `docs/talk-quantities.md`.
Coding and quantities workers remain in progress; poll their scopes/events.

- Generalize verified code editing beyond compiler suggestions.
- Broaden dialogue service coverage and facts retrieval without gold IDs.
- Improve reading on real English, measured separately from parser coverage.

Known models/data remain under `~/.cache/glassmind/{models,data}`. Historical
results and provenance: `README.md`, `data/README.md`, track reports and
`docs/results/tracks.jsonl`. Do not re-run or tune on old held-out evaluations.
