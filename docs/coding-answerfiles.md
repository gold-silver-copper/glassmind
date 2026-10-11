# Typed metadata answer files

Worker submission based on `712d3a5f1ecf1bae20eca26cfd1211ff6184a72a`.
Its exact source and evidence remain archived in
`/home/gsc/.cache/glassmind/continuous-20261010/answerfiles-run/submitted/`.
Coordinator integration below supersedes the dispatch and trace limitations.
Source hashes, byte counts, binary identities, individual outcomes and retained
result paths are in [validation.json](coding-answerfiles-evidence/validation.json).
No change to evaluation definitions, shared manifests, historical prompts,
quantity/fact/story logic, world indexing, or the ordinary action executor.
No commit, download, agent, extra LLM call, service, credential read or API use.
All owned subprocesses have finished.

## Preregistered hypothesis and baseline

Before editing, the audit found genuine typed Count/Extent answers but no exact
field-type spelling in the world representation. Hypothesis: fully consume a
metadata question and output directive, reuse those typed answers, and create a
plain answer file without executing a repair or rename. Acceptance: both terse
count requests produce independently derived correct values; independent count
and unique-name examples work with unrelated names/paths; unknown/ambiguous
references, incompatible formats, extra actions and unsafe/occupied destinations
refuse without source mutations or answer files. Unsupported chatty/type requests
may continue to refuse. The transcript flag retains its separate purpose.

Frozen baseline supplied by the coordinator: coding engine `2ce9230`, binary
SHA256 `cbe42583c56482cec7f8dca21a368f6984e5ecb491757714318af692f9fcfad8`.
This worker replayed all seven permitted original requests: zero answer files.
The isolated caller question already returns 3; enum “have” was interpreted as
HasField and refused, while “variants of” returns 6.

Independent source audit: three actual caller functions, not comment/string
mentions or number of call sites; six enum variants, despite a stale doc comment
saying five and a commented-out variant. Exact field spelling is `Vec<LineItem>`;
that is validation evidence only, not a runtime answer or added capability.

## Exact changes

- `English::exact_questions` exposes complete grammar questions without spelling
  repair, normalizer/model fallback or arbitrary free-text remarks.
- `english-rust::metadata_questions` disambiguates typed Variant possessive
  HasField relations as Member, only in this explicit metadata interface.
  Ordinary dialogue grammar and readings stay unchanged.
- `cli/src/answer_file.rs` parses a question plus the entire output directive,
  requires uniquely resolved references and one compatible answer value, and
  serializes Count as decimal digits plus newline or a singleton Extent as its
  item name plus newline. Type/prose/JSON output is unsupported. It calls only
  lookup/answer, never propose, remedy, check or apply.
- The `do`/`act` path recognizes write/put/save instructions before ordinary
  action handling, including malformed compound requests. Successful exports
  report `Wrote VALUE to PATH.` to stdout and `--reply-file`; refusals report the
  reason there. The answer file contains only the requested plain value.
- `tools/coding-answerfiles-dev.py` replays the three supplied workspaces/seven
  prompts and independently authored safety fixtures, compares exact content,
  hashes every original file, and records all additions. Its expected values
  belong only to this dev tool; the runtime contains no task/answer lookup.

Example (with a matching production declaration):

```sh
glassmind do /path/to/repo --reply-file /path/to/transcript.txt \
  'How many variants of `Signal`? Write just the number to reports/count.txt.'
```

The question must precede `?`. Supported directives use write/put/save,
optional please/just/only/the, number or name, to/into/in, optional a/file/named/
called, a filename, and optional “in/at [the] crate root”. Every remaining word
must be consumed. Filenames can be arbitrary single whitespace-delimited tokens,
including Unicode, extensions and nested paths, optionally quoted with matching
backticks/single/double quotes. Spaces in filenames are not supported.

## Path and overwrite contract

Answer destinations are relative to the supplied repository. Absolute paths,
empty/dot/parent components, missing parent directories, symlink directories,
symlink destinations and **every existing destination** are refused. Existing
source, answer and unrelated files are never overwritten. No directories are
created. Publication on Linux pins directories with O_DIRECTORY/O_NOFOLLOW,
fully writes and syncs a private staging file, then uses an atomic no-replace
hard link. Concurrent creation of the destination fails without replacing it;
readers cannot observe partially written answer content. Staging cleanup is
attempted on every normal return. Exceptional cleanup errors or process/power
loss can leave a private staging file; crash durability of the directory entry
is not guaranteed. Other platforms refuse publication.

The repository and its directory hierarchy must remain under the caller's
control during execution; this is not a sandbox against hostile concurrent
renames of entire directories. No concurrent rename/failure-injection testing
was performed. Static symlink and traversal escapes are explicitly tested.

`--reply-file` keeps its existing explicit CLI transcript-write contract and is
not the answer destination. It can therefore be written on a refusal. A resolved
collision between a parsed answer destination and the transcript is rejected
before either is written. Existing transcript overwrite behavior is unchanged.

## Results and rejected experiments

| Check | Baseline | Final |
|---|---:|---:|
| Fixed three terse dev prompts: usable/refused/wrong | 0/3/0 | 2/1/0 |
| All seven supplied dev prompts | 0/7/0 | 2/5/0 |
| Independent sample | — | 3/3 passed |
| Independent full | — | 27/27 passed: 6 exports, 21 refusals |
| Existing English/Rust unit tests | — | 35/35 passed |

All original files remain byte-identical in every replay and fixture. Every
noncollision transcript exactly equals stdout. Focused all-target clippy with
warnings denied, the CLI debug build, and workspace fmt check pass.
Final binary SHA256:
`6882660e3efcf47cd51d2fe5c12992682bbc3f943560c42ce831fbf8d9bcd32b`.

Rejected experiment 1: reject all grammar idle tokens. This also rejected “how
many”; fixed sample 0/3, independent sample 1/3. Replaced with no free-text capture
and no edits/guesses; fixed grammar words may still participate.
Rejected experiment 2: globally add have/has as membership vocabulary. Export
checks passed but existing grammar tests regressed to 33/35 due to ambiguous
field readings. Removed those rules and confined typed disambiguation to the
metadata interface. Final 35/35 and sample/full export replays pass.

At worker handoff this was a local-dev gain; the coordinator's subsequent full
765-case result is recorded below. The worker did not reevaluate the supplied
starting score of 166 pass / 596 refused / 3 fail. No held-out
items or tests, task manifests, solutions, generators or private historical logs
were read. The omitted two hurried original requests were not reconstructed.

## Provenance and reproduction

Production input: coordinator-supplied RustBench dev, revision
`f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, three production-only workspaces,
3,418 bytes including manifests/locks; seven original requests in 1,539 bytes
of JSON. All supplied production hashes independently verified. Upstream license
is unknown (no supplied LICENSE); raw inputs stay outside git. No new dataset or
dependency downloads. Independent fixture Rust/manifests: 6,286 aggregate bytes
across 27 cases (including repeated setup), authored here, MIT OR Apache-2.0.
A separate plain-query/transcript regression fixture adds 96 bytes under the same license and gives byte-identical baseline/candidate replies for a function named `save`. Per-file hashes and provenance are in validation.json.

From this worktree, preserving the provided target directory:

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/answerfiles-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/answerfiles-run/target
cargo_capped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 cargo "$@"
}
cargo_capped build -p glassmind -j1
cargo_capped test -p glassmind-english-rust --lib -j1 -- --test-threads 4
cargo_capped clippy -p glassmind -p glassmind-english -p glassmind-english-rust --all-targets -j1 -- -D warnings
cargo_capped fmt --all -- --check
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" \
  --dev-inputs "$TMPDIR/../dev-inputs" --sample --output "$TMPDIR/replay-answerfiles-sample"
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" \
  --dev-inputs "$TMPDIR/../dev-inputs" --output "$TMPDIR/replay-answerfiles-full"
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" \
  --sample --output "$TMPDIR/replay-answerfiles-independent-sample"
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" \
  --output "$TMPDIR/replay-answerfiles-independent-full"
```

Use fresh output directory names; the tool deliberately refuses to reuse them.
For baseline replay substitute `$TMPDIR/glassmind-baseline`. The dev tool invokes
no cargo and finishes each CLI process before proceeding. Resource-capped cargo
checks above were run with the existing target directory, one job, four test
threads and no storage under /tmp.

Limitations: five supplied prompts still refuse; exact source-type spelling and
function-purpose explanations remain unsupported. Metadata inherits the existing
syntactic world index's resolution/scope limitations, including macros, aliases
and conditional compilation; this batch does not establish compiler-complete
whole-program counts. The worker did not run a release build or the full benchmark; parent gates and
corrections follow.


## Coordinator integration and rejected dispatch behavior

The final worker still refused ordinary unquoted queries about functions named
write/put/save. Parent independent baseline/candidate probes reproduced all three
regressions. Dispatch now requires an instruction boundary and, outside a prior
question, a value-output noun. Quoted and unquoted function names no longer
trigger export; ordinary "write a function" remains with the existing action
path. Eight regression checks retain all six name-query replies/transcripts and
verify exports with and without --explain, including no whitespace after `?`.

The export path now prints the exact typed metadata question, resolved answer
and fired world rules under --explain. The exported file remains the plain
value, and the separate transcript remains the result message. This is an
explicit query/export trace, not a claim of compiler-complete source resolution.

Parent replay passes sample2/3, supplied full2/7, all27 original local fixtures
(6exports21refusals), and all8 ordinary-query/trace checks. Eight CLI tests and
35 English/Rust tests pass; workspace fmt/alltargetclippy/release/strict4/4smoke
pass. [Integrated outcomes](coding-answerfiles-evidence/integrated/) retain
source-file hashes, stdout/transcripts and rejected I/O cases. Full765 RustBench
dev is the remaining publication gate; no overall benchmark gain claimed yet.

## Coordinator full development gate

Source `82f2a2e917acbf97958122a0a7c5ef01fe4a7eee`, frozen release binary
`a4b625c0e9c06a65d5226236fe6154e1003a50c4a643a5b2e7acd9a5bf69a2b9`,
passes **168 / 765** RustBench dev requests, with **594 refused and 3 failed**.
The unchanged benchmark revision is `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`.
Compared with the preceding 166/596/3 run, exactly two terse metadata-output
requests change from refusal to pass. Every other task/style outcome is unchanged;
all 166 prior passes remain. No held-out evaluation was run.

[Full comparison and artifact hashes](results/coding-answerfiles.json) retain
exact identities, transitions and local gates. The full job completed successfully;
its output remains under `~/.cache/glassmind/continuous-20261010/coordinator/answerfiles-full/`.
This verifies count exports, not general function explanations or exact field types.
