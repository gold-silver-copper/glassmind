# AddField bounded development batch

## Coordinator verification

Integrated source: `cfa343e9948a61699061230f720d15f77b81ca52`.
The independently built release binary is SHA256
`5f76302ccde394ebf74e2b7d7bc0030ab12f0f34a2915b1f1618d68ba17cd11a`.
Workspace formatting, all-target clippy, release build and all four required
smoke turns pass. All 66 affected unit tests pass. Independent coordinator
replays reproduce 2/2 sample edits, 2 edits and 3 refusals across the five
supplied requests, all 15 independent probes, 35 declared-type probes and
8 ordinary query/export checks. Three additional scalar probes cover signed
integers with CRLF and preceding Unicode, booleans and Unicode characters.
Each preserves every original source byte outside the requested insertions.

[Coordinator artifacts](coding-addfields-evidence/integrated/validation.json)
record commands, logs, source and binary identities. The original ten worker
files (224,621 bytes) were archived and hash-verified before integration and
again before removing the completed worktree. Raw fixtures and logs remain
under `/home/gsc/.cache/glassmind/continuous-20261010/addfields-run`.
Full 765-request development evaluation is complete: **172 passed, 590 refused,
3 failed**. The two terse field-addition cases improve; all other 763 outcomes
are unchanged, preserving all previous passes.
[Complete comparison](results/coding-addfields.json).

The worker's original handoff follows; its "uncommitted" and "parent must run"
statements describe the submission, not the later integration state.

Base: 2a968eccb073e55aa86561e92cb10796cd338ebc. Frozen baseline SHA256:
8659839583857656c1fdf2d0f22e4fd57de245db84778f9ab5b4d754de62af41.

Preregistered hypothesis: reusable typed AddField with explicit scalar type/default,
closed package binding resolution, source-span insertions, scratch trials and
transactional application can improve the two terse visible development requests
without changing unrelated source or existing observable behavior.
Acceptance: sample two then all five original requests; structural postconditions,
public tests and runtime comparison; independent names/multifile/alias/Self and
negative ambiguity/macro/cfg/default/trailing-action/rollback probes. Existing
65 relevant tests and prior query/export probes must remain correct (old27's
obsolete unsupported_type expectation retained as raw evidence).

Bounds: refuse unsupported language in full, unknown macros, type aliases,
target Default derive, unsafe/default expressions, target struct updates and
ambiguous bindings. No broad benchmark claim; parent owns full765 evaluation.
No downloaded data. Supplied DEV provenance: revision
f79e71a2a26fdaf15f9bc55225a038b61e66fafb; license unknown, raw files remain
outside git. Public inline tests preserved. No hidden sources accessed.

## Final implementation and outcome

Uncommitted implementation, no integration or publication performed:

- `crates/world-rust/src/add_fields.rs`: reusable `AddField` and `FieldAddition`
  types, `RustWorld::propose_add_field` / `apply_add_field`. Resolves a unique
  named struct, module/import aliases and `Self`; inserts one public declaration
  and literal defaults at all resolved constructor sites. Uses existing typed
  `RustChange`, `checks::copy_tree`, `checks::apply`, and `checks::run_in`.
- Package/module resolver has a separate AddField mode supporting `cfg(test)`
  modules and conservative `super::*` expansion. Existing rename mode unchanged.
  Standard macro expression lists are parsed recursively; unknown macros refuse.
- Scratch build/test and structural declaration/type/constructor-count/value
  postconditions precede application. A fresh original-root test and structural
  check follow it. Original touched files and Cargo.lock existence/bytes are
  restored on failure. Cached scratch results are never used for this action.
- `crates/english-rust/src/add_fields.rs`: fully consumed bounded English parser.
  Minimal `do` hook calls the reusable world interface. No read-command changes.
  Generic example: `Add pub weight: u16 to Parcel. Use 7 for all existing constructors.`
- `tools/coding-addfields-dev.py`: visible development replay, independent
  fixtures, source-byte preservation, runtime-output and added-field assertions.
  It adds independent contract tests after checking preservation; original public
  tests, assertions and expected values are never changed except field insertions.

Final exact-source results:

| Check | Result |
|---|---|
| Frozen baseline sample / all supplied requests | 0 edits, 2 refusals / 0 edits, 5 refusals |
| Candidate fixed terse sample | 2 verified edits / 2 |
| Candidate all original requests | 2 verified edits, 3 unchanged refusals / 5 |
| Independent probes | 15/15; 2 positive, 13 refusal/rollback |
| Relevant Rust tests | 66/66: prior 65 plus one parser test |
| Prior declared-type probes | 35/35 |
| Prior ordinary-name/count/name/export probes | 8/8 |
| Old export suite | 26/27; known obsolete unsupported_type expectation |
| Affected all-target clippy, fmt, debug build, diff whitespace | pass |

Full DEV adapter deliberately exits 1 because three requests remain unsupported;
that is missing coverage, not a wrong edit. Chatty and hurried language forms
remain refused; no arbitrary preamble removal or partial action execution.
The old27 raw failure remains unchanged: `field-type` alone is added containing
`Vec<u16>\n`; every original file hash is preserved. No scorer was changed.

Independent probes cover aliases/Self across lib/bin, unrelated fields,
colliding struct names, target Default derive, type aliases, unknown constructor
macros, struct updates, cfg, unsafe defaults/functions, trailing actions,
preexisting failed assertions, type overflow, local binding ambiguity and live
rollback. The live rollback test succeeds in scratch but fails in its original
workspace; original source bytes and absence of Cargo.lock are restored.
Public DEV tests exercise constructor literals in cfg(test)/super-glob modules,
vec!, format!, println!, constructor functions and ordinary closures. Added
contract tests assert primitive field types and requested values; old runtime
stdout and old fields' observable behavior are preserved.

## Artifacts and identities

[Machine-readable evidence](coding-addfields-evidence/validation.json) contains
per-file source hashes/bytes, supplied provenance and every raw report hash.
Source identity is base commit plus the eight-file source manifest (including the
adapter); SHA256 of compact sorted-key manifest JSON:
`93568548d0a5c072ef76e9371b513d8a7a9db4ac30e772b4914df7903dcb97e7`.

Frozen tested debug binary:
`/home/gsc/.cache/glassmind/continuous-20261010/addfields-run/tmp/glassmind-addfields`
SHA256 `2ad21e0584abdc07426c04217b8db633b7e17d88341ee7b024605aacd7ba23da`.

Raw evidence root is the same `tmp` directory. Final reports are
`addfields-accepted-{sample,full,independent}/results.json`,
`addfields-final-{metatypes,old27,query-regression}/results.json`,
`unit-tests-final.log`, `clippy-last.log`, `build-last.log`, `fmt-last.log`.
Baseline reports are `addfields-baseline-{sample,full}/results.json`.
Intermediate reports/logs remain there without being treated as final outcomes.

Supplied data: two DEV workspaces, twelve Cargo/source files, 5,025 bytes,
revision `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`; license unknown as provided.
Original request JSON: 972 bytes in two files; hashes in evidence. Original requests/provenance also remain outside git. No external data,
dependencies, model artifacts or downloads were added. Independent fixtures
were authored in this batch in the adapter (45 original fixture files, 4635 bytes), under MIT OR Apache-2.0, like the
repository. They are local development evidence, not benchmark items or scores.

## Reproduction

Run from the worktree, using the existing disk TMPDIR and target directory:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/addfields-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/addfields-run/target
cargo8() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
cargo8 fmt --all -- --check
cargo8 build -j1 -p glassmind
cargo8 test -j1 -p glassmind -p glassmind-world-rust -p glassmind-english-rust -- --test-threads 4
cargo8 clippy -j1 -p glassmind -p glassmind-world-rust -p glassmind-english-rust --all-targets -- -D warnings
binary="$TMPDIR/glassmind-addfields"
dev=/home/gsc/.cache/glassmind/continuous-20261010/addfields-run/dev-inputs
python3 tools/coding-addfields-dev.py --binary "$binary" --dev-inputs "$dev" --output "$TMPDIR/replay-addfield-sample" --sample
python3 tools/coding-addfields-dev.py --binary "$binary" --dev-inputs "$dev" --output "$TMPDIR/replay-addfield-full"
# Expected full exit 1: two verified, three refused; inspect preserved raw result.
python3 tools/coding-addfields-dev.py --binary "$binary" --output "$TMPDIR/replay-addfield-independent" --independent
python3 tools/coding-metatypes-dev.py --binary "$binary" --output "$TMPDIR/replay-addfield-metatypes"
python3 tools/coding-answerfiles-dev.py --binary "$binary" --output "$TMPDIR/replay-addfield-old27"
# Expected old27 exit 1: the unchanged obsolete unsupported_type expectation.
python3 tools/coding-answerfiles-regression.py --binary "$binary" --output "$TMPDIR/replay-addfield-queries"
```

Use fresh output paths. All fixture cargo subprocesses use the same required
scope/setsid/timeout/TMPDIR/-j1/test-thread caps. Outer worker scope verified
MemoryMax=8589934592 and MemorySwapMax=0. All owned jobs finished.

## Rejected attempts and limits

Initial build command used nonexistent package `glassmind-cli` (actual package
`glassmind`), then compilation exposed a missing Result type annotation. Both
were corrected. Clippy rejected three guarded indexing/slicing expressions and
manual parity arithmetic; these were replaced with fallible accessors and
`is_multiple_of`. Raw failed clippy logs are retained. The first prototype
reused environment-configured check caches through World::apply; this was
replaced before final acceptance with fresh checks and the existing lower-level
typed change machinery, because a scratch result does not verify the original
root. No semantic candidate was accepted on build-only evidence.

Supported types: primitive integers, bool, char; defaults must be explicit
literals (optional unary minus, compiler validated). Arbitrary/default function
expressions, strings/containers, generic bindings, type aliases, unsafe code,
unknown/item/qualified macros, non-expression-list macro syntax, target Default
derive, struct updates, custom/conditional attributes except cfg(test) modules,
unsupported package targets and ambiguous imports refuse conservatively.
A target's manual Default implementation can work when its literal is resolved;
derived Default is refused rather than silently synthesizing a different value.
Debug/Clone/equality derives are supported and existing assertions remain gates.
No external dependents are edited. No general session/meaning enum integration,
release build, full-workspace suite or full765 evaluation is claimed. Parent must
run full765 externally before publication. No agents, model calls, services,
credentials, hidden tests, hidden-test generators, held-out data or old private logs used.
