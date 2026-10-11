# Persistent coding tasks

`glassmind do --session FILE` keeps a typed coding task and conversation between
invocations. It currently supports the existing rename and build/test/lint repair
actions, plus ordinary questions through the existing language frontend.

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
verifier. `--steps` accepts 1 through 8 semantic stages per invocation. `--steps 1` stops after proposing; `continue`, `apply it` or `yes` executes
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
completed earlier edits remain in place. There is no automatic retry. `show task`
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

Function implementation and add-field commands remain available through `do`
without `--session`. Conditional branches, automatic revision strategies and
general conversational constraints are not yet implemented by the controller.
