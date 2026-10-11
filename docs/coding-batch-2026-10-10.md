# Coding batch: verified requests and compiler repairs

Changes are uncommitted. No agents, LLM calls, credentials, held-out items,
hidden tests, generators, loop code, or flexion were used.

## Measured development results

The fixed sample is the first four single-turn training conversations in file
order for each of `build_status`, `fix_build`, `test_status`, and `fix_tests`.
This is a training-derived development sample, not a benchmark test score.

| Kind | Control right/refusal/wrong | Final right/refusal/wrong |
|---|---:|---:|
| build_status | 0 / 4 / 0 | 3 / 1 / 0 |
| fix_build | 2 / 2 / 0 | 4 / 0 / 0 |
| test_status | 4 / 0 / 0 | 4 / 0 / 0 |
| fix_tests | 4 / 0 / 0 | 4 / 0 / 0 |
| Total | 10 / 6 / 0 | 15 / 1 / 0 |

The control disables `coding.intent.fallback`, `mind.accept_verified`, and
`rust.remedy.iterate_suggestions` in the same binary. Runner safety and rollback
fixes are present in both. Each reported repair was separately applied to a
fresh source copy and checked with cargo; status replies were compared to the
training labels, including complete problem sets. No hidden tests were copied.

Sample input SHA-256:
`2f39dff03cfa6a7253609b760bcf0579214a8ba9e562a977e26afc78b5555cfc`.

## Implementation

- A Bernoulli naive Bayes coding-intent fallback reuses `IntentModel`. It
  proposes only typed build/test questions and repair requests, at a 0.98
  posterior threshold. The grammar keeps priority. An `other` class and an
  explicit negated-request guard preserve refusals outside its scope.
- The mind accepts a guessed check or complete repair only when the world
  verified the requested check and surviving replies agree. It states the
  assumed interpretation. Unchecked guesses and conflicting readings retain
  clarification.
- Compiler repairs can use three additional rounds of compiler suggestions.
  Each round runs in scratch, and edits are composed back onto the original
  coordinates only after the requested check passes.
- Applying an edit saves the affected files and restores them on a failed
  check or tool error. Multi-source remedies retain problems in untouched
  sources. The CLI writes final applied results to `--reply-file` and returns
  failure for rejected edits.
- Cargo requires an 8 GiB scope, zero swap, one compilation job, a 600-second
  timeout, and four test threads. `setsid --wait` keeps the scope alive when
  `setsid` must fork. Linter failures without readable diagnostics fail closed;
  check-cache format v4 invalidates the older behavior.

The model has 7,513 examples: 6,857 other, 56 build status, 476 build repair,
23 test status, and 101 test repair; 2,514 word features; 110,415 bytes.
All 44 turns from the sample's 16 source groups were excluded from training,
not just the 16 evaluated turns. Model SHA-256:
`a826ad8653d7902825681682025abc993d92d97e0f0fd5a59010b1b82788accb`.
An opening-sentence feature experiment caused one wrong intent and was reverted.

Targeted build, clippy with warnings denied, formatting, and five existing
world tests passed. Manual CLI smoke checks confirmed successful rename plus
reply-file output, byte-for-byte restoration of source after a rejected rename,
and refusal of a multi-source repair with an unresolved source.

Not implemented: repo-state interpretation of vague requests, new rename kinds,
new test-repair algorithms, or loop/CI changes. Test repair uses the existing
candidate search. The remaining dev refusal is an uncertain build-status intent.
No claims are made about unseen styles, external macro consumers, or hidden tests.

## Data provenance

No new downloads. `train-v3` contains 6,081 conversations / 7,557 turns:
1,346,051 input bytes and 3,271,881 label bytes (4,617,932 total). Generated
request/label data has no separately declared license in the available data
documentation. Code-source licenses below come from the source manifests.

| Source | Version | Sample copies | Nonhidden source bytes, excluding target | License |
|---|---|---:|---:|---|
| dunce | 1.0.5 | 2 | 43,756 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| heck | 0.5.0 | 5 | 216,894 | MIT OR Apache-2.0 |
| json | 0.12.4 | 2 | 513,617 | MIT/Apache-2.0 |
| percent-encoding | 2.3.2 | 3 | 106,238 | MIT OR Apache-2.0 |
| quick-error | 2.0.1 | 2 | 125,615 | MIT/Apache-2.0 |
| scopeguard | 1.2.0 | 1 | 34,382 | MIT OR Apache-2.0 |
| unicase | 2.10.0 | 1 | 110,783 | MIT OR Apache-2.0 |

Input SHA-256: `461fa4b0a7157185b231da0aece444342470f84cad8fb3e151845204cc46602f`.
Label SHA-256: `9dd38876af1d7b9adb38f9b13ada4896c691fe84842b20e714658d1d2fe6e845`.
The classifier uses the remaining train-v3 labels, including rustbench dev-derived
training turns (documented source revision `f79e71a`, MIT OR Apache-2.0), and QA
training turns. It reads no underlying rustbench test material or generators.

## Exact reproduction

Use the output directory outside the Cargo workspace; otherwise scratch crates
inherit the surrounding workspace. All commands below run sequentially.

```bash
cd ~/code/glassmind
set -e
export TMPDIR="$PWD/scratch/tmp"
export CARGO_TARGET_DIR="$HOME/.cache/glassmind/coding-target"
mkdir -p "$TMPDIR"
capped() {
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=5s 600 "$@"
}
capped cargo fmt -p glassmind -p glassmind-mind -p glassmind-world-rust --check
capped cargo clippy -p glassmind -p glassmind-mind -p glassmind-world-rust --all-targets -j 1 -- -D warnings
capped cargo build -p glassmind -j 1
capped cargo test -p glassmind-mind -p glassmind-world-rust -j 1 -- --test-threads 4
dev_args=(--binary "$CARGO_TARGET_DIR/debug/glassmind"
  --train /home/gsc/.cache/glassmind/data/train-v3
  --sources /home/gsc/.cache/glassmind/sources
  --out "$CARGO_TARGET_DIR/dev-baseline")
capped python3 tools/coding-dev.py "${dev_args[@]}" --baseline
capped python3 tools/coding-dev.py "${dev_args[@]}"
cat "$CARGO_TARGET_DIR/dev-baseline/baseline-report.json"
cat "$CARGO_TARGET_DIR/dev-baseline/verified-report.json"
```

Reproduce the included model after the sample inputs have been written:

```bash
python3 tools/train-coding-intent.py \
  --train /home/gsc/.cache/glassmind/data/train-v3 \
  --exclude-inputs "$CARGO_TARGET_DIR/dev-baseline/inputs.jsonl" \
  --out crates/cli/models/coding.tsv
```
