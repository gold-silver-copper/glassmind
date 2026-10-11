# Glassmind: build the most promising capabilities continuously

Activate this direction explicitly with:

```text
/goal Follow /home/gsc/code/glassmind/prompts/grow-yolo.md continuously until I stop you. Prioritize shared language understanding, multi-step verified coding, and persistent task context. Implement substantial working features, validate and ship them, then immediately continue. Make routine decisions autonomously.
```

Reading, writing or reviewing this file does not activate or resume a goal.
My latest pause or stop overrides historical checkpoint text. When I explicitly
activate this direction, it supersedes the priorities and development cadence in
`grow-continuously.md`, `grow-fast.md` and older project prompts. Use the existing
goal runner; do not launch an independent supervisor to keep yourself running.

Work in `/home/gsc/code/glassmind`, repository `gold-silver-copper/glassmind`.
Build aggressively until I stop you, within the actual platform and account
limits. A feature, merged PR or benchmark gain is a milestone, not completion
of this continuing development goal. Do not invent a token or time budget.

## Direction

Build a system that understands useful language, maintains the user's task,
and performs actions whose results it can check. Concentrate effort on the
connection between learned language understanding and typed, verifiable actions.

Prioritize these three capabilities together:

1. **Shared language and task representation.** Represent intent, referenced
   entities, explicit constraints, alternatives, unresolved information and
   evidence. Resolve different phrasings into the same operation. Preserve
   negation, corrections and uncertainty. Reuse and extend existing meaning,
   grammar and mind components before introducing another representation.
2. **Multi-step verified coding.** Inspect a repository, locate relevant code,
   interpret diagnostics, choose a supported action, propose a patch, check
   behavior and revise from the result. Maintain explicit task and step state.
   Bound attempts, detect repeated failures, preserve user changes and roll
   back unsuccessful edits. Explain what was verified and what remains unknown.
3. **Conversation that carries work forward.** Persist the active task, target,
   constraints, choices and evidence across turns and session restarts. Handle
   follow-ups such as "use the other one," "keep that name," "undo that change"
   and "continue" when context supports a unique interpretation. Invalidate
   stale references when code changes. Refuse or request clarification when
   context is genuinely ambiguous; never silently invent a destructive intent.

Facts, quantities, reading and memory should supply capabilities to this shared
flow. Work on them when a concrete end-to-end task needs them. Preserve existing
behavior, but stop treating every track as a separate benchmark to optimize.
More isolated templates, annotation campaigns or graph acquisition are lower
priority unless they remove a demonstrated obstacle in the shared system.

## Implementation pace

Spend most of your effort writing, integrating and debugging useful production
code. Make larger coherent batches that deliver complete behavior through the
existing CLI. Prefer a working vertical slice over an elaborate framework.

Make consequential architecture decisions yourself. Choose reversible defaults,
refactor across crates when needed, and fix problems you discover. Do not ask me
to approve ordinary implementation details, select the next task or say continue.

Write as much useful code as the next capability requires. Do not optimize for
line count, produce speculative scaffolding, leave empty interfaces, or call
disconnected demos progress. A new abstraction should have a real caller in
the same batch. Keep working implementations reachable through normal commands.

Use a short plan with the user-visible behavior, affected components and checks.
Then implement. Do not spend a whole cycle preparing plans, provenance wrappers
or reports when implementation can proceed. Reuse existing runners and logs.
One concise checkpoint and links to retained evidence are normally sufficient;
create extra research paperwork only when the claim actually requires it.

An architectural feature may ship with flat legacy benchmark scores when it
demonstrates a new useful behavior and preserves existing correctness. Do not
require every feature to earn another point on an old task. Do not describe a
flat score as a gain. Keep the benchmark as a regression check, not the roadmap.

## Starting or resuming development

Briefly inspect current code, git state, applicable instructions, open work and
`docs/continuous-progress.md`. Check actual process and artifact state: old
"running" entries may already be terminal. Preserve unfinished changes and
verified results without letting the old queue dictate this new direction.
Do not restart completed jobs or re-run unchanged large experiments.

Determine which of the capabilities below already work. Do not rebuild a
completed task representation, plan runner or persistence layer because this
prompt describes them. Finish and validate promising unfinished integrations
first, then choose the most valuable remaining gap from the actual code.

If the shared task flow is missing, implement a shared task representation and
connect it to one complete coding flow in the existing CLI. Choose a real
supported action as the first consumer, then extend the flow to multiple steps.
Otherwise extend the existing flow with another useful verified action, stronger
constraint handling or better contextual language understanding. Reuse existing
parsers, source indexes, patch representations, checkers, rollback and session
storage. Let concrete integrations determine the abstraction; avoid a speculative
universal schema or a wholesale rewrite.

The integrated system should demonstrate:

- Several materially different phrasings reaching the same typed task while
  retaining explicit constraints; a correction changes only the intended field.
- An actual repository action that inspects code, proposes a change and records
  behavioral verification, with a failed attempt leaving the package intact.
- A follow-up referring to the prior target or choice, including after a session
  restart, without requiring the user to repeat the entire request.
- An ambiguous follow-up that does not edit the wrong target, and an external
  source change that invalidates stale task state before another edit.
- A bounded sequence of supported steps that uses the result of one step to
  choose the next, terminates on completion or lack of progress, and explains
  which parts of the requested task were actually accomplished.

These are development acceptance examples, not sentences to hardcode. Use
varied wording, identifiers and independent fixtures. Deliver intermediate
working slices when helpful, then immediately continue toward the full flow.
Afterward choose the next gap from actual usage of this integrated system.
Rank candidates by useful tasks unlocked, reuse across capabilities, evidence
that the approach can work, and implementation cost. Reassess after every shipped
batch. Do not exhaust easy benchmark cases while larger product gaps remain.

## Development loop

Repeat while the goal is active:

1. Pick a capability that unlocks useful tasks across the system. State the
   expected behavior and a few meaningful acceptance checks before editing.
2. Implement it through the product, including failures and recovery. Use
   independent workers where they accelerate real work; give each clear
   ownership and integrate centrally. Avoid parallel edits to the same files.
3. Run focused checks while iterating. Fix defects without weakening the tests.
   Review the patch, then run the required workspace checks and affected full
   development evaluations once for the coherent batch. Repeat only when new
   changes, failures or unresolved concerns justify it.
4. Commit, push, open or update a PR and merge when required CI passes for the
   exact reviewed head. Use `gh pr merge --merge --match-head-commit SHA`.
   Record measured behavior and limitations, then start the next useful batch.

A failed idea is a reason to change the representation, implementation or data,
not to repeat the same experiment indefinitely. If one dependency is blocked,
make independent progress elsewhere. Never manufacture activity through cosmetic
changes or repeated evaluations. Do not stop at a plan or an offer to do work
that this prompt already authorizes.

## Architecture and evidence

Keep Glassmind's core: typed Rust reasoning, small inspectable learned models
that propose or rank, and indexed factual stores. No runtime LLM calls or
general-purpose language-model fallback inside Glassmind. A learned code
proposer consumes code; types and checks decide whether its proposal is accepted.
Language models that classify or rank interpretations remain allowed.

Preserve deterministic behavior where supported, explicit uncertainty, useful
traces and rollback. Passing a build is not behavioral verification; passing
tests establishes only the behavior they cover. Do not erase assertions, edit
scorers to make a candidate win, hide wrong answers among refusals, or claim
general capability from a narrow synthetic result. No panics in production code.
Do not introduce dependencies on flexion or sememe crates. Compression rounds
and code-size penalties remain off.

Use approved train/dev material to develop and evaluate. Keep official held-out
items, labels, predictions, hidden tests and generating prompts outside agent,
worker and teacher access. Never use held-out results to choose changes. Keep
retired or contaminated evaluations disabled. Preserve split boundaries and
denominators; label task overlap, source limitations and synthetic evidence.
Record enough provenance to reproduce a result without building a reporting
system around every patch. Keep failed experimental models out of production.

## Model, authentication and authority

Use **Codex GPT-6 Astra, fast mode, through my logged-in ChatGPT account** for
any development workers, reviewers or teacher calls. Never use my OpenAI API key,
another provider's key, Claude or Opus. Remove API-key environment variables
from child processes; verify supported login status without reading or exposing
credential contents. Do not fall back to API authentication or another model.
Verify current CLI options locally before starting a worker.

A prompt cannot switch an already-running parent session's model. Apply my
requested settings wherever you control model selection and never claim a
switch you did not perform. If the required worker configuration is unavailable,
continue useful local work and record the limitation. Use teacher generation
only when it serves a concrete learned component; avoid speculative large runs.

Upon activation, I authorize normal project edits, allowed training and public
data acquisition, isolated worktrees, parallel workers, commits, pushes, PRs
and merges after checks. No routine confirmation is needed. Preserve unrelated
work. Purchases, new paid infrastructure, external messages to people, account
permission changes, force-pushing shared history and unrelated deletion are
outside this authorization. This development style does not override platform
permissions or my later instructions.

## Resources and continuity

Keep every cargo execution, including generated-project checks, inside a
memory-capped systemd scope with `MemoryMax=8G`, `MemorySwapMax=0`, its own process
group, `setsid --wait`, a finite timeout and descendant cleanup. Reuse verified
wrappers. Use on-disk temporary storage outside `/tmp`; at most four cargo jobs
per run, eight across the project, and four test threads per run. At most four
model workers total, or the lower platform limit. Bound each experiment and
watch available memory and disk. Track owned jobs and preserve original worker
outputs before cleanup; never kill unrelated processes or delete unrelated caches.

Keep a concise current handoff in `docs/continuous-progress.md`: what works,
what changed, checks and exact results, remaining issues, branches/PRs, owned
running jobs, artifact paths and the next concrete action. Replace stale status
at the top instead of accumulating contradictory instructions. Update the
README and existing score journal/HTML when measured claims change. Record
known model usage honestly, including cached input once and accounting gaps.

Give brief updates about meaningful findings and decisions. Answer my questions
directly while preserving the active task unless I change or stop it. A turn
boundary, compaction, completed feature or merged PR does not complete this goal.
Do not require human intervention for ordinary continuation.

When I pause or stop, stop starting work, terminate owned workers and services,
preserve code and evidence, and leave a short handoff. Never restart yourself
after a pause, stop, exhausted limit or lost runtime. Resume only with explicit
authorization. No prompt guarantees execution beyond actual account or platform
availability. If no meaningful authorized work remains possible, preserve a
resumable checkpoint and follow the platform's blocked-goal rules.

Once explicitly activated: inspect enough to act, implement the next useful
integrated slice, validate it, ship it, and keep building the next capability.

---

Goal lifecycle reference: [OpenAI Docs — Using Goals in Codex](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex).
