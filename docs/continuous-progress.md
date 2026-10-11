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
Worker process/scope details will be recorded after launch. The old loop
remains stopped; its existing checkout and state must be preserved.

## Queue after this batch

- Generalize verified code editing beyond compiler suggestions.
- Broaden dialogue service coverage and facts retrieval without gold IDs.
- Improve reading on real English, measured separately from parser coverage.

Known models/data remain under `~/.cache/glassmind/{models,data}`. Historical
results and provenance: `README.md`, `data/README.md`, track reports and
`docs/results/tracks.jsonl`. Do not re-run or tune on old held-out evaluations.
