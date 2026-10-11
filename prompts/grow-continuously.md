# Goal: continuously build and improve glassmind until I stop you

Paste this into Codex to activate:

```text
/goal Follow /home/gsc/code/glassmind/prompts/grow-continuously.md continuously until I stop you. Select, implement, validate and ship improvements autonomously, then immediately start the next useful task. Do not ask for routine confirmation or mark the goal complete at a milestone.
```

Select GPT-6 Astra and fast mode in the parent Codex session before activating
the goal. Use the existing ChatGPT login. The instructions below also enforce
these preferences for any model processes the agent launches.

These instructions take effect when I explicitly activate or resume this
goal. Reading, reviewing or editing this file does not activate it. Old
checkpoint text saying "goal active" is historical evidence, not permission
to override my latest stop, pause or request to work on something else.

When activated through `/goal`, use that existing goal; do not create a
second goal or launch a separate unattended supervisor to keep yourself
alive. Let the goal runner manage continuations. On each continuation,
recover current state and carry out the next useful work.

Work in `/home/gsc/code/glassmind`, GitHub `gold-silver-copper/glassmind`.
Continuously make glassmind a more capable, reliable general semantic
language system and coding agent. Improve coding, conversation, reasoning,
facts, quantities and understanding real English. Ship measured improvements
and immediately choose the next useful experiment. Work autonomously until
I explicitly stop or pause you, subject to the actual platform limits.

This is an ongoing development goal, not a request for a plan or a single
batch. Completing a feature, merging a PR, reaching a score, exhausting the
current backlog or writing a report does not complete this goal. Each is a
checkpoint. Keep the goal active across ordinary turn boundaries and context
compaction; do not mark it complete after a milestone. Do not invent a token
budget or time limit. Respect any limit imposed by me or the platform.

## Authority and autonomy

When I activate this prompt, it supersedes conflicting project directions
in older prompts, including `prompts/grow-fast.md`. Preserve those files as
history. Higher-priority system and tool requirements still apply.

You have standing authorization to make ordinary project decisions, edit
code, train allowed models, obtain public datasets and dependencies, run
bounded experiments, delegate to workers, create isolated worktrees, commit,
push project branches, open PRs and merge them with `gh pr merge --merge`
when the required checks pass. Resolve ordinary conflicts and broken CI
yourself. No further confirmation is needed for these actions.

Do not ask me to choose the next task, approve a routine plan, authorize
another batch, review ordinary tradeoffs or tell you to continue. Make a
reasonable decision, record consequential assumptions, and execute. If one
track is blocked, work on another. Never stop with an offer to do work that
is already authorized.

Do not make my response part of the ordinary development cycle. When a
preference is unspecified, choose a reversible default consistent with the
project and record it. When an action requires authority you do not have,
finish the independent work, record that action as unavailable, and choose
another useful task. A blocked experiment is not a blocked development goal
while other meaningful authorized work remains.

This authorization does not extend to purchases, new paid infrastructure,
external messages to people, changing account permissions, force-pushing
shared history, deleting unrelated work or modifying other projects.
Choose an in-scope alternative when one of those would otherwise be needed.

## Model and authentication: fixed requirements

- Use **Codex GPT-6 Astra, fast mode**, through my existing logged-in
  **ChatGPT account**, for workers, reviewers and teacher generation.
- Never use my OpenAI API key, another provider's key, Claude or Opus.
  Remove API-key environment variables from child processes. Force ChatGPT
  login and do not fall back to API authentication or a different model.
- Verify login through supported status commands. Do not read, print, copy
  into logs or commit credential contents. Use the established login and
  sandbox integration; credentials must not enter project artifacts.
- The previous working CLI configuration was `codex --no-daemon exec
  --ignore-user-config --ephemeral --model gpt-6-astra
  -c 'service_tier="fast"' -c 'forced_login_method="chatgpt"'
  -c 'model_reasoning_effort="high"'`. Verify supported flags locally before
  launching; retain the model, fast tier and authentication requirements.
- A prompt cannot switch the model of an already-running parent session.
  Never claim it did. Apply these settings wherever you control model
  selection. If the required configuration is unavailable, continue useful
  work that does not need new model calls and record the limitation.

## Establish current state, then start work

Read `docs/continuous-progress.md` first if it exists, then `README.md`,
`data/README.md`, `docs/results/tracks.jsonl`, the latest
entries in `/home/gsc/code/notes/projects/glassmind/progress.md`, applicable
`AGENTS.md` files and the relevant track reports. Use `prompts/grow-fast.md`
and `prompts/glassmind.md` for architectural context, not stale instructions
to run old models, reactivate old test sets or repeat completed work.

Inspect git status, recent commits, remote branches, open PRs, current
workers, loop state and available resources. Preserve existing changes.
Prefer an isolated worktree over disturbing an occupied or dirty checkout.
Verify present facts rather than trusting a historical commit or scoreboard.

Resume verified unfinished work from the current checkpoint before opening
duplicate experiments. Confirm whether recorded jobs are still running or
have completed; an observation timeout is not a failed experiment. Reuse
retained models, datasets and results when their provenance matches. Treat
historical handoffs as leads, not as current state or a fixed checklist.
Start the first useful bounded experiment after this audit.

## The continuing work cycle

Repeat this cycle for as long as the goal is active:

1. Select a concrete capability gap using train/dev evidence and actual
   product behavior. State a falsifiable hypothesis, the baseline, a bounded
   experiment and the acceptance criterion before editing. Prefer useful
   generalization per unit of time and tokens over easy benchmark points.
2. Implement the smallest coherent improvement. Use parallel workers for
   independent tracks when resources permit; give each an isolated worktree,
   explicit ownership, permitted data and a measurable deliverable. Integrate
   centrally. Count all workers, reviewers and loop agents toward the cap.
3. Evaluate on the track's fixed development sample. Inspect train/dev
   failures, fix mistakes and compare against the same baseline. A failed
   hypothesis is useful evidence: record it and change the approach.
4. Before pushing a coherent batch, run the affected track's full dev split,
   relevant cross-track regression checks and the repository's required
   fmt/clippy/build/smoke checks. Review the diff for correctness, answer
   leakage, architecture and unintended changes. Fix failures before merging.
5. Commit and push verified changes, open or update the PR, and merge when
   required CI is green for the reviewed head. Use
   `gh pr merge NUMBER --merge --match-head-commit SHA` with that exact head;
   if it changes, review and validate the new changes before merging.
   Update the scoreboard and checkpoint with the exact
   evaluated commit. If remote publishing is unavailable, preserve verified
   local commits and continue independent work; do not claim they merged.
6. Choose and begin the next experiment without asking me. Keep a short
   rolling queue with reasons and evidence, replenished from observed gaps.

Choose the next action yourself even when the queue is empty: reproduce a
real train/dev failure, identify the missing general capability, and test a
bounded improvement. Keep each experiment finite so its results can inform
the next one; the development goal itself has no predetermined end. A
worker finishing or timing out returns control to this cycle, not to me.

After three experiments on the same hypothesis fail to produce a useful
gain, change the hypothesis, representation, data or track. Do not repeat
the same evaluation without a new reason. A plateau triggers investigation
and a new approach, not goal completion. Do not manufacture progress through
cosmetic edits, endless planning, documentation churn or additional wrappers.

Keep both coding and conversation moving. Maintain the existing story,
dialogue, facts, quantities, grammar, reading and memory tracks; prioritize
by dev evidence and periodically revisit neglected tracks. Near-perfect
synthetic scores should lead to independently sourced dev data and broader
capability checks. Integrate successful components into usable glassmind
commands instead of accumulating disconnected benchmark demos.

## What glassmind is allowed to become

Preserve the core design: typed Rust reasoning, small inspectable learned
models that propose or rank, and indexed factual stores. No runtime LLM
calls or general-purpose language-model fallback inside glassmind. A learned
code proposer must consume code rather than English, propose candidates,
and leave acceptance to types and checks. Small language classifiers and
rankers remain allowed. Development-time Astra teacher calls are allowed
through the required ChatGPT login.

Generalize from data and program structure. Do not hardcode benchmark
sentences, task identities or expected answers. Preserve deterministic
behavior, useful explanations and explicit uncertainty. For coding actions,
try bounded candidates, verify them and roll back failures. Passing a build
alone does not establish behavioral correctness. Never weaken tests, erase
assertions or redefine success to make a repair pass.

Keep reusable knowledge in the proper layers. No panics in non-test code.
Do not depend on `/home/gsc/code/flexion` or sememe's crates; reading ideas
and reusing appropriately licensed data is allowed. Compression rounds and
code-size penalties stay off; refactor when it enables measurable progress
or removes an actual maintenance obstacle.

## Measurement that remains honest over a long run

- Select changes and allocate effort using train/dev evidence. Never use
  official held-out results to select a patch, tune a threshold, choose the
  next experiment or decide when to stop.
- Neither you nor development workers may inspect held-out items, labels,
  per-item predictions, generating prompts or hidden-test contents. Keep
  them outside worker access. Use the isolated evaluator's aggregate totals
  only, on a predetermined reporting cadence, not after each candidate.
  Keep the retired human-final and contaminated legacy evaluations disabled.
- Never let candidate code, training workers or teacher generation gain
  access to evaluator secrets. Workers must not edit scorers to improve their
  own acceptance. Legitimate evaluator fixes require separate evidence and
  a re-baseline; label the resulting break in comparability.
- Preserve benchmark denominators, official split boundaries and metrics.
  Separate correct substantive answers, correct expected refusals,
  unsupported/refused answers and wrong answers. Keep baseline right/refused
  cases from becoming confidently wrong. Prefer genuine additional coverage
  over maximizing refusal. Never present synthetic, scoped or dev results as
  broad independent test performance.
- Record source, revision, license, size, split, preprocessing, model hash,
  evaluated commit, commands and limitations. Avoid entity selection from
  evaluation labels when claiming open-world retrieval. Audit runtime inputs
  for annotation leakage when expanding adapters.
- Update `docs/results/tracks.jsonl` and regenerate
  `docs/results/tracks.html` with the existing reporting tools. Keep separate
  series for changed datasets and metrics. Record known tokens, including
  cached tokens, and identify incomplete accounting honestly.
- Use small fixed samples while iterating, full affected dev before shipping,
  and a few meaningful tests. Do not repeatedly run the entire workspace
  suite or all benchmarks after trivial changes. Keep CI's nonempty,
  all-correct fixture smoke gate intact.

## Keep the machine usable

Every cargo execution, including generated code and benchmark subprocesses,
must run in a memory-capped systemd scope with its own process group, a
timeout and cleanup that terminates descendants. Use the verified wrappers
where available. Preserve `MemoryMax=8G`, `MemorySwapMax=0`, and
`setsid --wait`; plain `setsid` can return before the job finishes.

Use an on-disk `TMPDIR` outside `/tmp`. At most four concurrent model workers
in total, at most four cargo jobs per run and eight across the project, and
at most four test threads per run. These are ceilings, not targets. Check
free memory and disk before large jobs and reduce concurrency when other
projects need resources. Bound dataset downloads, model training and cache
growth as well as cargo. Never remove unrelated caches to create space.

Track exact owned PIDs, process groups, scopes, worktrees and temporary login
mounts. Never use `pkill -f` or kill another project's work. Start long jobs
as supervised background work, inspect results on completion, and do useful
independent work while they run. Avoid blocking waits longer than 60 seconds.

Before modifying or removing a worker's worktree, preserve its original
patch, report, commands and results in an owned artifact directory and
verify their hashes. Review the preserved submission before integration.
Record whether each experiment is running, completed, rejected or
interrupted; retain enough evidence to avoid repeating it after a restart.

The existing improvement loop is optional infrastructure. Before restarting
it, verify its current binary, config, baseline, authentication, sandbox,
worker budget and dataset paths. Preserve a dirty loop checkout. Restart
only when it serves a concrete current hypothesis; never run a second loop
against the same state or blindly reuse historical settings. Do not leave
workers running independently after the controlling goal stops.

## Continuity, reporting and stopping

Maintain `docs/continuous-progress.md` as a concise checkpoint: active
hypothesis, current branches/commits, baselines, exact results, completed and
rejected experiments, queued work, PR/check status, running jobs and resource
ownership, artifacts and reproduction commands. Update it at each batch and
before a handoff or compaction. Do not store credentials or held-out items.
Always leave one concrete next action and enough context to execute it:
the relevant paths, baseline, expected result and verification command.
Keep a few independent alternatives ready so one blocked dependency does
not turn the checkpoint into a request for human direction.
Keep the current handoff at the top and move completed batch details into
linked reports. Record a short decision for each rejected approach so future
continuations can avoid repeating it without rereading the entire history.
Append a five-line update to the existing external progress log every few
hours and at a major milestone. Keep README scores consistent at merges.

Give brief progress updates about findings and decisions while working.
Each active turn should advance implementation, an experiment, validation
or delivery whenever an authorized path is available. Reading checkpoints,
rewriting plans and reporting status alone are not a development cycle.
While a job runs, advance independent work; when only that job can unblock
progress, monitor its existing handle instead of starting duplicate jobs.
An ordinary turn-ending summary is a checkpoint, not a reason to stop:
preserve the active goal and resume from the recorded next action. Do not
restart completed investigations after compaction or claim a process is
still running without checking it.

Treat new messages as steering for the active goal unless I stop, pause or
replace it. Answer status questions briefly and continue the authorized
work. Keep durable checkpoints sufficient for a fresh session to recover
without requiring me to explain the project again. Never infer fresh
authorization from a summary when the actual conversation says otherwise.

For transient failures, honor retry guidance and use bounded backoff while
progressing elsewhere. Do not bypass rate limits, switch accounts or use an
API key. For a persistent blocker, try other independent tracks and local
work. Only when no meaningful authorized path remains should you report a
blocked goal, following the platform's blocked-state requirements. State
the exact blocker and preserve a resumable checkpoint; do not fabricate work
or call the goal complete.

When I stop or pause you, stop starting work, terminate owned workers and
services, preserve code and evidence, clean only disposable resources you
created, and leave a concise handoff. Never restart yourself after a stop,
pause, cancellation or platform limit. No prompt can guarantee execution
through an expired login, exhausted account limit, closed runtime or lost
machine; preserve progress so an explicitly resumed goal can continue.

Once I activate or resume this goal: audit current state, select the
highest-value measurable next experiment, execute it, and keep repeating
the cycle without waiting for me.
