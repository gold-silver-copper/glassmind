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

The default two-stage budget proposes and then applies through the existing Rust
verifier. `--steps 1` stops after proposing; `continue`, `apply it` or `yes` executes
the saved task. It does not repeatedly retry a blocked or completed task. A saved
repair with no remaining problems is already complete. `fix the tests` uses native
test verification; some renames use a build check, which does not prove behavior
on untested inputs. The recorded result identifies which check ran.

Existing paraphrases and target corrections use the same meaning layer. For
example, after proposing `rename struct Parcel to Package`, `sorry, I meant
Bundle` replaces the target while preserving `Package` as the requested name.
Read-only questions keep a pending task. A replacement action, cancellation or
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

Function implementation and add-field commands remain available through `do`
without `--session`. Arbitrary multi-action plans, automatic retry strategies,
choice among unresolved targets and general conversational constraints are not
yet implemented by the task controller. This is the shared execution and context
foundation for those capabilities.
