# Persistent coding tasks

`glassmind do --session FILE` keeps a typed coding task and conversation between
invocations. It currently supports the existing rename and build/test/lint repair
actions, test-verified implementation of supported function stubs, public field
addition with explicit constructor defaults, and ordinary
questions through the existing language frontend.

```sh
# Propose without applying; keep state outside the source repository.
glassmind do ./my-library --session ./coding-task.json --steps 1 \
  'rename struct Parcel to Package'

# A separate invocation revalidates the exact proposal, then applies and checks it.
glassmind do ./my-library --session ./coding-task.json 'continue'

# Context refers to the renamed item after the restart.
glassmind do ./my-library --session ./coding-task.json 'rename it to Delivery'
glassmind do ./my-library --session ./coding-task.json 'undo that change'
glassmind do ./my-library --session ./coding-task.json 'show task'
```

The default four-stage budget proposes and applies through the existing Rust
verifier. `--steps` accepts 1 through 8 semantic stages per invocation. `--steps 1` stops after initial inspection; `continue`, `apply it` or `yes` advances
the saved task. It does not repeatedly retry a blocked or completed task. A saved
repair with no remaining problems is already complete. `fix the tests` uses native
test verification; some renames use a build check, which does not prove behavior
on untested inputs. The recorded result identifies which check ran.

Existing paraphrases and target corrections use the same meaning layer. For
example, after proposing `rename struct Parcel to Package`, `sorry, I meant
Bundle` replaces the target while preserving `Package` as the requested name.
Answered read-only questions keep a pending task. An unresolved request cancels
pending execution and reports that a fresh action is needed. A replacement action, cancellation or
unsupported new instruction cannot silently authorize an older proposal.

Source fingerprints bind the session to one canonical repository. Any external
file change clears contextual references and invalidates pending execution and
undo. State a fresh explicit request afterward. Undo restores the bytes changed
by the last successfully verified action only when the source still matches its
recorded revision. It does not promise the restored code passes tests: undoing a
repair can deliberately restore the original failure.

The state includes the resolved action, status, typed proposal/application evidence
and limited undo data. It uses atomic replacement and an OS session lock. An
interrupted application is never automatically replayed. Saved proposals are
recomputed through the world before execution; cached diagnostic text containing
process IDs is not execution authority.

This first version bounds snapshots to 4,096 files, 4,096 directories and 32 MiB,
excluding `.git` and `target` directories, and refuses source symlinks. Session
and optional reply files must be outside the repository. Use one active coding
session per repository; the lock serializes one session file, not all programs
that might edit the repository. Session state uses a separate format from
`ask`/`chat` conversation files.

## Ordered plans

Separate up to four supported operations with `then` or semicolons:

```sh
glassmind do ./my-library --session ./coding-task.json --steps 8 \
  'fix the tests; then rename function combine to sum; then check the tests'
```

Every clause must pass complete-request parsing before the first operation runs.
The plan stores unresolved meanings, then resolves each operation against current
code and conversation when it reaches that step. A rename followed by `rename it`
therefore refers to the renamed item. Quoted delimiters remain part of the text.
Unsupported or conditional clauses refuse the whole new plan before any edits.

A proposal and its application each consume a stage; a read-only question uses
one. `continue` resumes the recorded next stage after a restart. Each edit uses
the existing native verifier. A failed check or blocked action stops the tail;
completed earlier edits remain in place. A repair can use the bounded revision process described below. `show task`
reports completed operations and the next clause. Undo restores only the last
verified edit and cancels the remaining plan. Answered read-only questions can
interrupt a pending plan without discarding it. A new action replaces the plan.

## Choosing among targets

When a rename has multiple supported targets, the session lists up to 32 stable,
numbered alternatives with qualified names and source locations. Select one in
a later invocation with `choose 2`, `use the second one` or
`choose Second::value`. The destination name and requested kind stay unchanged.
With `--steps 1`, selection only prepares a fresh proposal; `continue` applies it.
After previewing one of exactly two alternatives, `use the other one` switches
the proposal. With more alternatives, name or number the target explicitly.

A choice can unblock the current step of an ordered plan. It cannot bypass native
verification, revive a cancelled task or edit stale source. Selecting a field
retains its explicit owner and uses the same owner-aware position audit as an
explicit qualified rename. Unsupported targets can still remain blocked. After
a successful rename, `rename it` refers to the selected item's new name, even
after a build/test check or diagnostic question about the whole repository.

## Bounded repair revisions

A repair request that has no complete proposal can try up to four revisions in
an isolated copy. Each trial uses the existing compiler suggestions, supported
trait/index repairs or checked test-repair models. The resulting diagnostics
select the next supported candidate. This can fix a compiler error first and
then repair the test failure that compilation exposes.

The real repository stays untouched throughout the trial. Every candidate and
its before/after diagnostics are recorded in the task. One trial uses one stage
of `--steps`; `continue` reconstructs the saved trial and checks its candidates
again. The four-attempt limit persists across invocations. Repeated source or
diagnostics, an unsupported next repair, or exhausted attempts stop the task.
An interrupted or externally stale task is never automatically replayed.

Once the requested check passes in scratch, Glassmind composes one final
proposal. Applying it consumes another stage, independently replays the trial,
and uses the native verifier and rollback on the real source. A build check
still proves compilation only; a test check covers only those tests. Undo restores
all edits from the successful composed action together. A repair inside an
ordered plan must finish before later operations run. `show task` reports the
attempt count, diagnostic counts and stopping reason.

## Implementing a function

`implement gather_values` uses the existing finite, signature-directed native
proposer and keeps its verified patch in the task. `--steps 1` previews without
editing. A later `continue` reruns the native proof, compares the exact typed
patch and applies it through the implementation verifier. Saved evidence paths
and candidate counts explain the preview; they cannot authorize an edited patch.

Implementation supports the same top-level stubs in `src/lib.rs` as standalone
`do`. Existing failing tests must execute the exact target, and exactly one
candidate in the bounded family must pass. Passing tests do not prove prose
requirements, doc comments or behavior outside their coverage. Missing tests,
an unexecuted target, ambiguous passing candidates and unsupported code refuse.
Compiler warning source excerpts do not count as target execution evidence.

After locating a function, `implement it` or `fill it in` resolves the prior
target, including after restart. A correction changes the target and keeps the
implementation action. Quote identifiers that are also English grammar words,
for example ``where is `total`?``. Plans can combine implementation, rename and
verification; each action keeps its own proposal/application stages and undo.
Existing rename restrictions still apply, including collisions with identifiers
in generated code.

Conditional branches, arbitrary new repair families and general conversational constraints
are not yet implemented by the controller.

## Adding fields with retained specifications

A public field addition retains its owner, field name, primitive type and one
explicit literal default for every existing constructor. It uses the existing
closed-package constructor proof, then checks the scratch build and tests before
proposing. Application recomputes the patch, checks it again, and performs a fresh
native check in the repository with rollback on failure.

```sh
glassmind do ./my-library --session ./coding-task.json --steps 1 \
  'Add pub weight: u16 to Parcel; use 7 for all existing constructors'
glassmind do ./my-library --session ./coding-task.json 'continue'
glassmind do ./my-library --session ./coding-task.json 'undo'
```

After an unambiguous reference to a struct, `Add pub ready: bool to it. Use true
for every existing constructor.` uses that struct. A quoted owner such as `it`
remains a literal identifier. A target correction preserves the field name, type
and default. Plans can add a field, rename its owner, then check tests; semicolons
inside the field specification belong to the same operation. Plans accept at
most four operations and 32 separated clauses within the existing byte limit.
Missing or conflicting defaults, unsupported constraints and unresolved owners
block execution. A later type rename can still be blocked by its existing
restrictions on cfg modules and opaque macros; a successful earlier field
addition remains recorded and can be undone. The existing restrictions on generic structs, opaque macros,
side-effecting defaults and ambiguous constructor graphs still apply. Existing
tests establish only their covered behavior.
