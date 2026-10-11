# HANDOFF — bounded repair ambiguity guard

**Locally accepted; coordinator full765 gate pending. Uncommitted.**
Source HEAD `eb22519f20325e72549a103cf99180ea0dbc2ba3` plus the three runtime
files recorded in `results.json`. Only corrected `repairambiguity2-run` was used.
No agents, LLM calls, services, credentials, hidden assets, expected patches,
old private logs, score changes, or commits. All owned subprocesses completed.

## Exact change

- `crates/world-rust/src/ambiguity.rs`: checked scalar AST interpreter and
  in-memory exact token replacement. One immutable bool or fixed-width integer
  argument (8/16/32/64 bits), bool or matching integer return, one tail expression.
  Supports scalar literals, argument references, parentheses, boolean operations,
  comparisons, checked arithmetic, and signed negation. Both branches are fully
  validated before evaluation; execution short-circuits `&&`/`||`. Every numeric
  operation respects the argument's Rust width. Division by zero, overflow,
  signed MIN / -1 and MIN % -1 are excluded. Numeric comparisons must have an
  argument anchor so unrelated literal expressions cannot silently default to
  the wrong Rust type. Calls, casts, floats, locals, macros, modules/imports/type
  declarations, non-doc attributes, and unsupported expressions give no evidence.
- `slips.rs`: registers the private helper. Proposal order and all existing
  source/test exclusion rules are untouched.
- `checks.rs`: retains the original first passing mutation. Examines at most
  64 subsequent proposals, same file only, at most 8 additional cargo trials,
  and only trials for which a concrete defined disagreement was already found.
  Both candidates must pass the original tests before withdrawal. The extra
  search has a 10-second compiler/test deadline (with bounded process cleanup),
  and no trial starts after 80 seconds of the existing 120-second search.
  Existing compiler-error repair and ordinary cargo behavior keep deadline=None.
  Follow-up processes retain outer timeout600/8GiB/zero-swap/setsid and add a
  shorter inner timeout plus scope lifetime. Independent timeout fixture passes.
- On refusal, saves `witness.json` under `GLASSMIND_SCRATCH/ambiguity-*` with
  original and both exact candidate sources, patches, typed input, both outputs,
  and source-tree fingerprint. Emits the record and saved path on stderr.
  CLI response schemas stay unchanged; the ordinary unresolved-test refusal is
  accompanied by the detailed audit in stderr/artifacts.
- `tools/coding-ambiguity/replay_witness.py`: independently checks exact patch
  application, optional unchanged original public-test trees for both candidates,
  and native Rust outputs against the retained AST witness. No expected answer
  is selected. `PLAN.md` and `results.json` retain preregistration and measurements.

Final conservative bounds also include 64KiB per source and AST depth64.
Input set: valid members of -128..255, type endpoints and immediate neighbors,
source literal neighbors; sorted, at most512. Bool has both inputs. No function
names, docs, task IDs, English proposals, or domain rules affect this selection.

## Before / after

| Original TRAIN-derived dev | Baseline verified / refused / failed verification | Candidate |
|---|---:|---:|
| Frozen sample4 | 2 / 2 / 0 | 2 / 2 / 0 |
| All27 unique sources | 6 / 21 / 0 | 5 / 22 / 0 |

Exactly one transition: `rb-fix-tests-leap-year` verified -> refused. The other
26 complete outcomes are identical, including exact patches for all five
remaining accepted repairs. No new claims. Visible verification is not correctness.

The previous first patch is line5 column52 `==` -> `!=`; the alternate patch is
line5 column18 `&&` -> `||`. On `is_leap(0u32)`, the former returns false and the
latter true. Both exact sources independently pass the unchanged original public
integration test. A separate native Rust test confirms both actual outputs and
their inequality. Zero is a valid u32 input; no domain-specific precondition or
expected calendar answer is inferred. This proves disagreement, not which
candidate is correct. The three-style full765 effect is **not measured here**.

Evidence root:
`/home/gsc/.cache/glassmind/continuous-20261010/repairambiguity2-run`.
Key artifacts: `baseline-{sample,full}`, `candidate-{sample,full}`,
`candidate-full/scratch/ambiguity-rb-fix-tests-leap-year-2966996-0/witness.json`,
`native-train-witness/{first,second,native}.log`,
`native-train-witness/report.json`, `native-independent-witness/report.json`,
`unit-tests.log`, `clippy-final.log`, `build-final.log`.

## Independent probes and checks

24 world-rust tests pass. New checks include:
- Exhaustive256 u8 short-circuit/arithmetic results and256 i8 checked arithmetic
  results against native Rust operations, plus zero division, u8/u64 overflow,
  i64 negation overflow and i8 MIN division/remainder overflow.
- Concrete threshold disagreement, identical-source non-evidence, unsupported
  calls even on dead branches, type suffix mismatch, unrelated default-integer
  expressions, floats, type shadowing, and test/cfg expression exclusion.
- Two actual cargo repair fixtures using `x < 3 && x < 9`. With only input3
  asserted, two passing mutations differ at input4 and the guard refuses. With
  input4 also asserted false, the alternate fails and the first repair is kept.
  Both fixtures verify original source/test bytes were unchanged by search.
- Native replay of the retained independent threshold witness confirms outputs.
- A deliberately slow authored test is terminated by a500ms follow-up deadline;
  the timeout produces no pass and completes within5s.

All original108 input source files rehashed against preparation provenance,
including every `tests/visible.rs`. Targeted clippy `--lib -D warnings`, fmt check,
CLI build, and `git diff --check` pass. Final rebuilt CLI has the same SHA256 as
the exact candidate used for sample/full27. No workspace-wide or hidden gate run.

## Source and binary identities

Baseline binary SHA256:
`bb5dae741d23b84d9ce07707ee536aeaf38089be34330724221ed50bb2f1bacb`.
Candidate binary SHA256:
`2f4476462328a59cc72ef03356275126eb41a5725c6da135024c1f02ceb27d30`.
Exact per-source, input and report SHA256 values are in `results.json`.
Frozen sample input SHA256:
`e7e40240d859bdc23a6c66f81d4b33d19a420a329743a7a48892b1e6bef5d61b`.
Full27 input SHA256:
`6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164`.

No new external data/downloads: **0 bytes**. Reused source revision
`f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, MIT OR Apache-2.0 per supplied prior
provenance;108 files/29,232 bytes. Original TRAIN requests9,016 bytes; metadata-only
labels2,669 bytes, no separately declared request-data license. New authored
fixtures and implementation use the repository's MIT OR Apache-2.0 license.
Raw copied source and compiled probes remain outside git.

## Reproduce

From this worktree with the existing target (do not use /tmp):

```bash
run=/home/gsc/.cache/glassmind/continuous-20261010/repairambiguity2-run
export TMPDIR="$run/tmp"
export CARGO_TARGET_DIR="$run/target"
capped() {
  TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
    setsid --wait timeout --kill-after=10s 600 "$@"
}
capped cargo fmt -p glassmind-world-rust --check
GLASSMIND_SCRATCH="$TMPDIR/probe-scratch" capped cargo test -p glassmind-world-rust -j1 -- --test-threads 4
capped cargo clippy -p glassmind-world-rust --lib -j1 -- -D warnings
capped cargo build -p glassmind -j1
```

For each binary, run sample before full with fresh output names. Actual recorded
runs use `$run/baseline-sample/checks` as the shared content-fingerprinted cargo
observation cache; final accepted edits are always independently tested fresh.

```bash
for version in baseline candidate; do
  capped python3 tools/coding-repair-dev.py \
    --binary "$run/$version/glassmind" \
    --train "$run/dev-inputs/train" --sources "$run/dev-inputs/sources" \
    --out "$run/replay-$version-sample" --sample \
    --check-cache "$run/baseline-sample/checks"
  capped python3 tools/coding-repair-dev.py \
    --binary "$run/$version/glassmind" \
    --train "$run/dev-inputs/train" --sources "$run/dev-inputs/sources" \
    --out "$run/replay-$version-full" \
    --check-cache "$run/baseline-sample/checks"
done
capped python3 tools/coding-ambiguity/replay_witness.py \
  --witness "$run/candidate-full/scratch/ambiguity-rb-fix-tests-leap-year-2966996-0/witness.json" \
  --root "$run/dev-inputs/sources/rb-fix-tests-leap-year" \
  --out "$run/replay-native-witness"
```

## Rejected approaches and limits

No score-tuned variant, task-specific exception, broader refusal rule, or
replacement correct formula was tried. During implementation, broad literal
inference was narrowed before dev evaluation to argument-anchored numeric
comparisons; otherwise Rust's default i32 could be misrepresented. Admission-only
follow-up timing was replaced before final evaluation by subprocess deadlines.
These are implementation corrections, not additional measured experiments.
No first-launch baseline/artifact was reused.

Agreement on sampled inputs is **not proof of equivalence**. Unsupported source,
insufficient time, exhausted proposal bounds, panics, and absent witnesses all
keep the old first-candidate policy. Inputs cover only the listed finite set;
functions with richer syntax and alternative patches beyond the bounded window
remain unchecked. The original first-candidate search still has its pre-existing
coarse120s check between cargo invocations; this batch bounds the added work,
not all existing repair latency or queue/copy latency. File-copy work itself
is not preempted by the subprocess deadline. Long-running baseline cargo tests
can still exceed the parent budget, as before.

Coordinator must run full765 and inspect exact transitions: retain all current
passes, reduce the three failures, introduce no failures. Local evidence supports
integration review but does not claim that pending gate passed.
