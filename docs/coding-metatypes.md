# Exact declared field types

Bounded worker batch on source `38ef238c560d11f0627adb14a946ceb935bc4288`
(includes safe exports `82f2a2e`). Changes are uncommitted. No agents, model
calls, credentials, API keys, services, datasets or dependencies were added.

## Preregistered hypothesis

Before editing, the audit found `Record.type_idents` and `TypeMentions`, which
retain identifiers but cannot express a complete type. The frozen binary SHA256
is `a4b625c0e9c06a65d5226236fe6154e1003a50c4a643a5b2e7acd9a5bf69a2b9`.
Replayed baseline: fixed first three terse requests 2 usable / 1 refused / 0
wrong; all seven 2 usable / 5 refused / 0 wrong, no source changes.

Hypothesis: retaining the exact syn type span with a unique owner and field,
then exposing a reusable typed query, will enable declared-type replies and
safe exports. Acceptance set before editing: sample 3/3, preserve both old
full-seven exports, exact independent type syntax and provenance, conservative
ambiguity/refusal behavior, and unchanged count/name/query/write protections.
No benchmark acceptance definitions were changed.

## Implementation and semantics

`world-rust::declared_types::{DeclaredTypeQuestion, DeclaredType}` and
`RustWorld::declared_type` are reusable without English or the CLI. The index
stores each named struct field's complete type slice, owner and field handles,
and half-open UTF-8 byte offsets. Character columns from syn are converted to
byte offsets; Unicode, multiline whitespace and internal comments survive.
This is the syntax between the first and last type tokens, not trivia outside
the type span. `TypeMentions` remains unchanged and is never used as a type
answer. Explicit aliases stay spelled as aliases; no compiler/type resolution
or inference is claimed.

Owner and field must each be unique. Duplicate struct owners are rejected even
if only one has the requested field. Same-named aliases/enums/unions/traits also
block an unqualified owner. Unknown fields, tuple indices, malformed indexed
files, custom attributes, cfg and macros refuse. Macro/cfg guards conservatively
cover the entire indexed source set, including unrelated code. Ordinary
index/count/name behavior is unchanged.

`english-rust::declared_types` supplies a complete, bounded parser and a spoken
reply. Forms include `type of Owner.field?`, `What is the type of Owner::field?`
and `What is the type of the fieldname field on Owner?`, with optional identifier
quotes and optional `the Owner struct`. Recognized compound or trailing text
refuses. No prose normalization, suffix deletion or runtime lookup table is used.

Both standalone `ask` and ordinary `do` use this typed operation, as does the
existing export helper. `ask --json` returns a typed `declared_type` object or
`refused` reason for this interface. This Rust-specific interface does not add
variants to the cross-world `Question`/`Reply` enums or teach the dialogue engine
new session behavior. `ask --session` explicitly refuses these questions;
interactive chat and conversational references are not implemented.

Exports add `type` format and fully consumed exact-source modifiers to the
existing write/put/save grammar. Publication uses the existing `publish` function
and its atomic hard-link/no-clobber/no-symlink protections. The output is exact
source spelling plus one newline. `--reply-file` remains the separate transcript;
answer/transcript collisions refuse. `--explain` prints typed question, answer
with offsets and the fired `rust.metadata.declared_type` rule. No new writer or
source mutation mechanism exists.

## Measurement and limitations

Final measurements and exact source/binary identities are in
[coding-metatypes-evidence/validation.json](coding-metatypes-evidence/validation.json).
The original seven-request script and its fixed value definitions were not edited.
The final candidate measured sample 3/3, full 4/7, 0 wrong and no source mutations.
Independent sample3/3 and full35/35 pass (13 exports,22 refusals; 5,916 fixture
source/manifest bytes). World21, English/Rust36 and CLI8 unit tests pass; scoped
all-target clippy with warnings denied, debug build and workspace fmt pass.
The three chatty preambles still refuse because their complete language is not
supported. This is a local-dev gain, not a new full765 score. The coordinator
owns the full765 evaluation; the supplied previous 166/596/3 is not remeasured.

Independent fixtures exercise generics, references/lifetimes, tuples, arrays,
qualified paths, associated types, aliases, function pointers, raw pointers,
trait objects, multiline comments, Unicode and nested source/output paths.
Negative probes cover owner/field duplicates, alias-owner collisions, absent
names, tuple indexing, malformed source, cfg, macros, custom attributes,
trailing actions/text, incompatible output and unsafe destinations. All source
bytes are compared before/after. Ordinary JSON replies additionally validate
that provenance offsets select exactly the returned spelling.

The unchanged old27 suite reports 26/27: its frozen `unsupported_type` case
expects refusal, while the candidate correctly creates `Vec<u16>\n`. Its `safe`
flag includes the obsolete no-new-file expectation, so this row is labelled
UNSAFE by that script despite no source modification. Raw results are retained;
no metric was silently amended. The other26 cases and all8 quoted/unquoted
write/put/save and trace regressions pass.

Rejected approaches: assembling types from `TypeMentions` or pretty-printing
syn tokens would lose syntax and were rejected during design. Broad chatty-text
stripping was not attempted because it would silently discard instructions.
Initial clippy rejected an index operation as potentially panicking; replaced
with checked access. An independent baseline replay has a harness limitation:
ordinary refusal JSON uses the previous generic schema, while the new probe
expects the declared-type schema. Its raw `passed` total is not a comparable
semantic score; baseline created zero of the13 requested positive type exports.
The final candidate is evaluated against both exact content and typed provenance.

Other limitations: syntax indexing is not Cargo/compiler-complete. It recursively
indexes ordinary `.rs` files, may include inactive/orphan files, skips symlinks,
and does not resolve imports, package/module-qualified owners, aliases, cfg,
macro expansion or derive name shadowing. Unrelated macros/cfg may cause false
refusals. Safe builtin derive names are accepted syntactically. This is a
snapshot of source, not a guarantee that types compile or names resolve.
Directory race/crash durability limitations of the existing exporter remain.
No release build or broad benchmark run is claimed by this worker.

## Provenance and reproduction

Only the supplied three production-only dev workspaces and seven request strings
were read. Source: RustBench dev `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`,
3,418 production bytes including Cargo manifests/locks, 1,539 request JSON bytes.
Upstream license unknown: no LICENSE was supplied. Raw copies and original
provenance stay outside git in `../metatypes-run/dev-inputs`. No held-out items,
hidden tests/generators, task manifests, solutions or private historical logs
were read. Independent fixtures are authored here, MIT OR Apache-2.0; exact
aggregate bytes and hashes are retained in the validation evidence.

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/metatypes-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/metatypes-run/target
cargo_capped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
cargo_capped test -p glassmind-world-rust -p glassmind-english-rust -p glassmind --lib --bins -j1 -- --test-threads 4
cargo_capped build -p glassmind -j1
cargo_capped clippy -p glassmind-world-rust -p glassmind-english-rust -p glassmind --all-targets -j1 -- -D warnings
cargo_capped fmt --all -- --check
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs "$TMPDIR/../dev-inputs" --sample --output "$TMPDIR/replay-types-sample"
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs "$TMPDIR/../dev-inputs" --output "$TMPDIR/replay-types-full"
python3 tools/coding-metatypes-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --sample --output "$TMPDIR/replay-types-independent-sample"
python3 tools/coding-metatypes-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/replay-types-independent-full"
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/replay-types-old27"
python3 tools/coding-answerfiles-regression.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/replay-types-regression8"
```

Use fresh output names. Substitute `$TMPDIR/glassmind-baseline` for baseline.
Every cargo run used TMPDIR, the existing target, systemd8GiB/0swap, setsid
--wait, timeout600/kill-after10, -j1 and four test threads. The outer worker scope
was independently verified at8GiB/0swap. No storage under /tmp. All owned jobs
must be complete at handoff.

Final runtime source-manifest SHA256: `88f86cc6e33ee34a3637f87b9c4abd26a2f909afe2723e7c94d875a00a338e7f`.
Final evaluated debug binary SHA256: `a886bda67cc9a9a23764de52798dbe3fab961f6061110da4ff2dd5424adfb182` (121,662,912 bytes).
Frozen at `/home/gsc/.cache/glassmind/continuous-20261010/metatypes-run/tmp/glassmind-metatypes-verified`. All owned execution sessions completed.

## Coordinator integration

Original source/reports are archived with hashes at
`~/.cache/glassmind/continuous-20261010/metatypes-run/submitted/`.
The parent independently rebuilt and replayed sample3/3, full4/7, all35 type
fixtures and8 ordinary-name/trace checks. Two additional probes verify exact
UTF-8 offsets across CRLF/Unicode source and refusal of a trailing command.
All65 scoped tests and workspace fmt/all-target clippy/release/strict4/4smoke pass.
The old27 suite retains its original refusal expectation:26 pass; its one
`unsupported_type` result is verified as the correct `Vec<u16>` export, with
zero original-file changes and only the requested file added. No scorer was
changed. Raw results and parent evidence are in
[coding-metatypes-evidence/integrated/](coding-metatypes-evidence/integrated/).
Full765 against the latest168/594/3 baseline is required before publication.
