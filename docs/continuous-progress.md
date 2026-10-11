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
Facts worker subsequently launched from `1bab8ee`, PID 1537404, scope
`glassmind-continuous-facts.scope`, same worker root and model/auth settings.
Its worktree is `facts`, logs and metadata `facts-run`. Scope verified active.
It owns facts crate changes and a separate continuous report; frozen index
and relation model only. Hypothesis selection begins with auditing the 39
wrong dev answers; accept fewer actual wrong claims without losing baseline
right answers or introducing new wrong answers. Baseline 256/2526/39 of 2821.
The old loop remains stopped; its checkout and state must be preserved.

## Queue after this batch

Conversation integration completed and locally verified: mixed conversation
18/27 → 27/27 correct, zero wrong, both with/without frozen intent model;
41/41 memory checks; full 3116 synthetic quantity questions through talk retain
3056 correct, 60 refused, zero wrong (762 correct expected refusals).
Workspace fmt/clippy/build and strict 4/4 fixture smoke passed. Source commit
is recorded in the chat score journal; see `docs/talk-quantities.md`.
PR #4 merged after green CI at `f83aefc` on main. Coding worker remains in
progress; poll its scope/events. Quantities worker completed; scope inactive,
temporary auth symlink removed, worktree removed after all 16 changed files
were verified byte-identical to the committed root copies. Cache is retained.

Integrated quantities source `e3eb10f`: exact percentage grammar; MathQA
train/dev adapter. External full dev 5/4469/1 → 14/4458/3 right/refused/wrong
of 4475; sample 0/100/0 → 1/99/0. Two new scored errors are precision mismatch
with rounded choices; exact arithmetic independently checked, 300/300
coordinator Fraction audit correct. Decision: preserve mathematically exact
answers, report the two scored errors without changing the frozen metric.
All 3116 synthetic substantive outputs retain status/value/unit; integrated
talk remains 27/27 and full quantity routing 3056/60/0. Workspace checks and
strict 4/4 fixture smoke pass on the integrated source.

The worker read the locally authored synthetic regression generator despite
an overly broad brief restriction. No held-out access occurred; that suite
was already documented as same-author synthetic data. The disclosure is
retained in its handoff. No independent-language claim relies on that suite.
MathQA cache and PyArrow are under `quantities-run/tmp/{mathqa,quantities-deps}`;
keep those artifacts. Known completed-worker usage is 22,157,081 tokens,
including cached input, excluding parent and unfinished coding work.

PR #5 merged after green CI (47 seconds), main `d607271`.
Coding worker completed: 1→2 verified repairs /4 fixed sample, 5→6 /27
available training sources, no failed verification; old five edited outputs
identical. This is visible-test verification, not benchmark correctness.
Coordinator extended its AST guards to exclude test-only locals, expressions,
match arms, fields and enums. All seven focused tests pass. Workspace build
is underway, then full rustbench dev on the frozen integrated binary.
Coding scope stopped and auth symlink removed; its worktree is preserved for
comparison. Facts worker remains active. Known completed-worker tokens now
24,550,895 including cache; parent and unfinished facts work excluded.

Full coding dev is running under exec session **52107**, launched by
`GLASSMIND=.../coordinator/coding-full/glassmind tools/rustbench-dev.sh
.../coordinator/coding-full`. Root is the worker root above. Source `75e3269`;
frozen binary SHA-256
`f9018a5b737d3b548ea22f9d36245d304238493f07c6421d0c349ba547074d75`.
Workspace fmt/clippy/release build and strict 4/4 fixture smoke passed.
Poll the existing session; do not start a duplicate. Compare all dev verdicts
against `~/.cache/glassmind/grow-fast-20261010/keep-going-candidate/rustbench-dev/results.jsonl`.
No coding PR/push until full-dev evidence is reviewed. Current root has the
coding source committed; checkpoint/usage updates await their own commit.

Facts preliminary diagnosis: 19 of 39 wrong dev answers arise from the
uppercase alias `OF` matching ordinary “of” after normalization. The same
issue appears in training. Worker is testing a general case-preserving rule
for single-token all-uppercase aliases, with no changes to index/model/scorer.
Do not treat this preliminary diagnosis as a measured final improvement.

- Generalize verified code editing beyond compiler suggestions.
- Broaden dialogue service coverage and facts retrieval without gold IDs.
- Improve reading on real English, measured separately from parser coverage.

Known models/data remain under `~/.cache/glassmind/{models,data}`. Historical
results and provenance: `README.md`, `data/README.md`, track reports and
`docs/results/tracks.jsonl`. Do not re-run or tune on old held-out evaluations.
