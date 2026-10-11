# Continuous development checkpoint

Goal active: follow `prompts/grow-continuously.md` until the user stops it.
Previous goal turns made concrete progress; no blocking condition exists.

## Current source and publishing

Repository `/home/gsc/code/glassmind`, branch `harness`.
Merged PR #4 (conversation calculations), main merge `f83aefc`;
merged PR #5 (exact percentages/MathQA), main merge `d607271`.
Local source commits awaiting the next PR: `75e3269` (reviewed syntax-based
repairs) and `c9ed6ed` (facts acronym guard), plus measured reports.
Do not push worker candidates until their affected full dev evidence passes.

The latest complete benchmark records are in `docs/results/tracks.jsonl`;
`docs/results/tracks.html` is regenerated at each batch. Historical reports
retain their original scope and source revisions.

## Completed and verified this continuous run

- Main `talk` calculates exact quantities while preserving memory and session
  behavior. Mixed synthetic conversation 18/27→27/27 with/without intent
  model; memory41/41; full3116 synthetic quantities through talk retain
  3056 correct,60 unsupported,0 wrong, including762 expected refusals.
  Source `b4b1b3f`, report `docs/talk-quantities.md`.
- Percentages: source `e3eb10f`; MathQA official dev4475 under the custom
  exact-numeric/no-choice metric improves5/4469/1→14/4458/3 right/refused/wrong.
  The two extra scored errors compare exact arithmetic with rounded choices;
  counts remain unchanged by adjudication. A separate300-case Fraction audit
  passes. All3116 synthetic status/value/unit outputs unchanged. Gold-option
  normalization cannot handle1229 rows; they stay in the denominator. No test
  downloaded. Details `tools/quantities/HANDOFF.md`.
- Coding: source `75e3269`; full765 rustbench dev at unchanged `f79e71a`
  improves160/602/3→163/599/3. Exactly three refusal→pass transitions for
  `fix-tests/steps-moving-average`, one per style; all other762 outcomes
  unchanged. `docs/results/coding-syntax-repairs.json` records binary/result
  hashes. Coordinator extended the worker AST guards to protect test-only
  locals, expressions, arms, fields and enums. Seven focused tests pass.
  Full evaluation session52107 is COMPLETE; do not restart or keep polling it.
- Facts: source `c9ed6ed`; unchanged500-entity index/frozen model. Full2821
  dev256/2526/39→256/2545/20, all correct answers retained, no new claims.
  Sample12/186/2→12/188/0. Train diagnostics remove154 wrong claims without
  losing a right answer or adding a claim. Rejecting acronym casing keeps
  the original ambiguity and never exposes another candidate. First filter
  design created new train errors and was rejected. Five focused tests pass;
  coordinator independently reproduced final full dev. Reports in
  `tools/facts/continuous/`.

Workspace fmt, all-target clippy, release build and strict4/4 fixture smoke
pass after integrating coding and facts. No held-out evaluation has been run
in this continuous batch; none of its items was inspected.

## Active workers: inspect live state before acting

All worker files are under
`/home/gsc/.cache/glassmind/continuous-20261010` (called WORK below).
Each `NAME-run/` has process.json, events.jsonl, stderr.log, summary.txt when
complete, and an isolated Codex home. Launcher is `WORK/launch.py`.

- **storychat**: launcher PID1566788, scope
  `glassmind-continuous-storychat.scope`, worktree `WORK/storychat`, base
  `e73f68c`. Owns CLI conversation integration, minimal stories persistence
  API and manifests. Hypothesis: existing story reasoner can answer across
  main talk turns/restarts without regressing memory or quantities. Reported
  preliminary47/47 mixed dialogue, original41/41 memory and27/27 quantity
  dialogue checks, full bAbI dev3966/18/3 of3987 through talk. Still auditing
  unsupported updates, persistence limits and conservative invalidation;
  verify final results rather than treating these as accepted.
- **edits**: launcher PID1589548, scope
  `glassmind-continuous-edits.scope`, worktree `WORK/edits`, base `e73f68c`.
  Owns world-rust and necessary typed language interfaces, not CLI. Hypothesis:
  conservative module-aware syn resolution can support struct/enum rename
  across references and imports while refusing ambiguous bindings. Installed
  rust-analyzer is only an unavailable-toolchain shim. Worker is implementing
  and measuring independent tiny-crate cases. Needs full rustbench dev before
  shipping; do not inspect hidden tests or expected patches.

Workers use GPT-6 Astra fast through ChatGPT login, with API-key environment
removed. Their bubblewrap sandboxes hide private data, other code and account
history. Each cargo command is capped8GiB/zero swap, own process group and
kill timeout, non-/tmp TMPDIR,1 compilation job. Resource ceilings include
all workers and benchmark jobs. Check before starting a large job.

## Completed worker cleanup and retained evidence

Coding, quantities and facts scopes ended; their temporary auth symlinks
were removed and worktrees removed. All quantity/facts files were verified
against integrated files before removal. Original coding submission is
archived in `WORK/coding-run/submitted` because coordinator guards differ.
Their logs, targets, development outputs and data remain in `*-run/`.
Known completed worker tokens:25,523,387 including cached input; parent and
unfinished worker usage excluded. Update `docs/results/worker-usage.json`
once per newly completed worker, without double-counting.

Frozen coding binary and full result: `WORK/coordinator/coding-full/`.
Before-results: `~/.cache/glassmind/grow-fast-20261010/keep-going-candidate/rustbench-dev/`.
Facts raw before/after train/dev predictions: `WORK/facts-run/tmp/`.
MathQA and PyArrow: `WORK/quantities-run/tmp/{mathqa,quantities-deps}`.
Do not delete retained data/models under `~/.cache/glassmind/{data,models}`.
The old improvement loop remains STOPPED; preserve its dirty checkout/state.

## Immediate next actions

1. Publish measured coding/facts batch; create PR, inspect CI and merge green.
2. Collect/review storychat and edits independently. Preserve legacy sessions,
   refuse unsupported story updates, protect source bindings and edit rollback.
3. Run fixed samples then full affected dev and required checks on integrated
   candidates. Publish source hashes, failures and limits; then choose next work.
4. Revisit broader reading, multi-service dialogue and facts coverage after
   these batches; avoid repeating saturated synthetic-only work.

Read the exact active prompt for standing authorization and stop/cleanup rules.
This checkpoint is not a request for user approval or a completed goal.
