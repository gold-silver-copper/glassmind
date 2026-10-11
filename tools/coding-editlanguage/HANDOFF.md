# HANDOFF — compositional AddField English

## Parent integration

Integrated source `eb22519f20325e72549a103cf99180ea0dbc2ba3`, binary SHA256
`bb5dae741d23b84d9ce07707ee536aeaf38089be34330724221ed50bb2f1bacb`.
Parent reproduced a trailing-conjunction bug (`. and` / `; and`), fixed it,
and independently verified 4/5 supplied edits, 13/13 behavioral probes,
35/35 declared-type checks, 8/8 queries, three incomplete-clause refusals,
38 English tests and workspace fmt/clippy/release/strict4/4 smoke.
Full765 dev: **172→174 passes,588 refused,3 failures**. Only two hurried
field-addition requests improve; all other763 outcomes remain unchanged.
[Full comparison](../../docs/results/coding-editlanguage.json) and
[parent validation](integrated/validation.json) contain hashes and evidence.
Original worker submission (5 files,50,089 bytes) was archived and hash-verified
before modification/removal. The following records the original worker result;
its parser and binary hashes differ from the parent-corrected integration.

## Original worker handoff

Base: `cfa343e9948a61699061230f720d15f77b81ca52`.
Acceptance met: supplied public dev **2/5 → 4/5 verified edits**, both baseline
successes retained, zero wrong edits. One unchanged refusal. No commit made.

## Exact changes

Only `crates/english-rust/src/add_fields.rs` changes production behavior.
The previous whole-sentence matching is replaced by source-span tokens, a checked
cursor, declaration/default/coverage/aside clause roles, and final unification.
Supported constituents include public field declarations in three forms, owner
prepositions, universal constructor scope or `new` plus other literals, explicit
literal defaults, polite modals and contractions, greetings/thanks and one narrow
epistemic aside. Clauses can precede/follow the declaration. Owner/field references
and repeated defaults must agree; one declaration and complete coverage are
required. Every clause and separator must be consumed. Dispatch recognizes a
complete declaration constituent (including a negated declaration for explicit
refusal), not keyword presence. Unrelated queries retain `None`.

Literal values come from their original byte spans; quotes, escapes, signs,
Unicode, numeric suffixes and backtick character values are preserved. Backticks
can delimit a constituent, including a char literal containing a backtick. Type
and literal compatibility still belongs to the existing world verifier. No world,
main CLI, grammar exports, shared manifests, scorer, historical prompt or README
changes. No new shared helper: there is currently only one consumer needing this
literal-preserving lexer. Existing English tokenization normalizes code calls and
quotes, so reusing it directly would change expressions.

New files: preregistration, this handoff, `evidence.json`, and `probes.py` in this
directory. The adapter imports existing cargo/run helpers without modifying the
original scorer. Its fixtures are diagnostics, never extra benchmark items.

## Measured outcomes

| Supplied original request | Frozen baseline | Candidate |
| --- | --- | --- |
| book pages, chatty | unchanged refusal | unchanged refusal |
| book pages, hurried | unchanged refusal | verified edit |
| book pages, terse | verified edit | verified edit |
| task priority, hurried | unchanged refusal | verified edit |
| task priority, terse | verified edit | verified edit |

The fixed terse sample was evaluated first: baseline 2/2, candidate 2/2. Then all
five were scored by unchanged `tools/coding-addfields-dev.py`, which exits 1 when
any request refuses: both full commands therefore exit 1, with 2 versus 4 passes.
Every edited dev fixture passes original public inline tests, added field/value
contracts, exact preservation of original source after removing only inserted
field text, and identical old program output. Each request starts with a fresh
copy of the original workspace. Each success updates five constructors.

Parsed-slot tests are distinct from these edits: 7 independently authored syntax
cases assert all four AddField slots. Negative unit cases check compounds,
conflicting defaults/owners, missing coverage, unknown prefixes/suffixes, calls,
negation, and unrelated fallback. End-to-end diagnostic adapter: **13/13**, six
verified edits using six struct names and i32/bool/char/u64/usize values, plus
seven unchanged refusals. Each positive preserves original assertions and old
output and passes a new typed field/default assertion. Frozen baseline on those
same diagnostics: zero edits, seven correct refusals, six unsupported positives
(adapter 7/13, exit 1). These are authored diagnostics, not independent held-out
performance. Baseline refusals are not counted as wrong edits.

Validation:
- `glassmind-english-rust`: 38/38 tests; final source, including declared-type and
  existing typed query/rename grammar regressions.
- Package all-target clippy with `-D warnings`: pass. Package fmt check: pass.
- Main CLI debug build: pass; actual full dev and independent replay use that CLI.
- Final binary declared-type/export adapter: 35/35, source-span and safety checks.
- Older answer-export adapter: 26/27 on BOTH binaries, all 27 outcome/answer/reply
  rows identical. Its `unsupported_type` expectation refuses an already-supported
  `Vec<u16>` declared-type answer. Kept the definition untouched; not reported as
  all green. Newer metatype adapter verifies that supported behavior.
- `git diff --check`: pass. No full workspace suite or full765 run by this worker;
  coordinator owns the full765 gate.

## Evidence, provenance and limits

Machine-readable hashes, all per-item outcomes and exact raw-result paths are in
`evidence.json`. Raw fixtures/results remain outside git at
`/home/gsc/.cache/glassmind/continuous-20261010/editlanguage-run/`.

Final parser SHA256:
`c28265c739409bc3ede44af54e55a42098323b0502f3cf14bef31a5a9cec69a3`
(16,697 bytes).
Frozen binary SHA256:
`5f76302ccde394ebf74e2b7d7bc0030ab12f0f34a2915b1f1618d68ba17cd11a`.
Final candidate binary SHA256:
`2d08433640d312e7a102a3257c05656591f17ca797c3092cc2858ba3561ac93f`.

No downloads, training or new external data. Supplied visible dev source revision
`f79e71a2a26fdaf15f9bc55225a038b61e66fafb`: 5,025 Cargo/source bytes plus 972
prompt-file bytes (JSON formatting included), two workspaces/five original
requests. License unknown in supplied provenance; raw data is not added to git.
File hashes and byte counts, including supplied provenance, are in evidence.json.
New adapter/embedded diagnostic fixture specification: 5,280 bytes, original work
under project MIT OR Apache-2.0 license, SHA256
`d9667de6de5572d5a88ae65495f9e9d3058585a40a8b099b63bb51ed41e21505`.
Production code and reports use the same project license.

The chatty request needs a representation for motivation, constructor-location
explanation and plural/pronominal references with scoped defaults. This batch does
not provide that representation. Arbitrary motivational prose is not stripped.
Grammar remains bounded; it is not broad natural-language understanding. Plural
references currently use owner + `s`; irregular plurals and nested arbitrary Rust
syntax are unsupported. Strings, containers, float types, function defaults and
unsupported world graphs remain refused by the unchanged world verifier.

Rejected/failed attempts: initial baseline invocation used relative workspace
paths, yielding package-graph refusals (0/2); reran absolute paths and obtained the
correct 2/2 baseline. First clippy run found two potentially panicking token
indexes; changed them to checked access and reran clippy/tests/build. No alternate
parser was tuned and discarded. Preamble deletion / skipping unknown context was
rejected as an unsafe design, not presented as an evaluated improvement.

Outer scope verified MemoryMax=8GiB, MemorySwapMax=0. Every direct or adapter
cargo run used disk TMPDIR, existing CARGO_TARGET_DIR, memory scope, setsid --wait,
timeout 600, -j1 except fmt, and four test threads. Existing runtime cargo wrapper
was inspected and already enforces those constraints. No agents, new LLM calls,
services, keys/credentials, held-out inputs, task generators or private logs read.
No background work remains owned by this batch.

## Reproduction

Run from this checkout, using fresh output directories (adapters require them not
to exist). Full provided-dev commands are expected to exit 1 for their refusals.
The frozen baseline diagnostic command also exits 1 for unsupported positives.

```bash
TASK_RUN=/home/gsc/.cache/glassmind/continuous-20261010/editlanguage-run
export TMPDIR="$TASK_RUN/tmp"
export CARGO_TARGET_DIR="$TASK_RUN/target"
limited_cargo() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
limited_cargo fmt --package glassmind-english-rust -- --check
limited_cargo test -p glassmind-english-rust -j1 -- --test-threads 4
limited_cargo clippy -p glassmind-english-rust --all-targets -j1 -- -D warnings
limited_cargo build -p glassmind -j1
python tools/coding-addfields-dev.py --binary "$TASK_RUN/baseline/glassmind" --dev-inputs "$TASK_RUN/dev-inputs" --sample --output "$TASK_RUN/repro-baseline-sample"
python tools/coding-addfields-dev.py --binary "$TASK_RUN/baseline/glassmind" --dev-inputs "$TASK_RUN/dev-inputs" --output "$TASK_RUN/repro-baseline-full"
python tools/coding-addfields-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs "$TASK_RUN/dev-inputs" --sample --output "$TASK_RUN/repro-candidate-sample"
python tools/coding-addfields-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --dev-inputs "$TASK_RUN/dev-inputs" --output "$TASK_RUN/repro-candidate-full"
python tools/coding-editlanguage/probes.py --binary "$TASK_RUN/baseline/glassmind" --output "$TASK_RUN/repro-baseline-independent"
python tools/coding-editlanguage/probes.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TASK_RUN/repro-independent"
python tools/coding-metatypes-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TASK_RUN/repro-metatypes"
python tools/coding-answerfiles-dev.py --binary "$TASK_RUN/baseline/glassmind" --output "$TASK_RUN/repro-baseline-answerfiles"
python tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TASK_RUN/repro-answerfiles"
```
