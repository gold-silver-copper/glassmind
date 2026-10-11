Run from the repository root. The existing target is retained throughout:

```bash
RUN=/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
export GLASSMIND_SCRATCH="$RUN/test-scratch"
```

Every Cargo command below was wrapped with this exact prefix (fmt alone omits
-j1; tests use four threads). Native subprocesses use the same scope/group/600s
outer limit plus a 10s per-trial deadline and 75s enumeration/90s apply budget.

```bash
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt --all -- --check
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test --offline -p glassmind-english -p glassmind-english-rust -p glassmind-world-rust -p glassmind -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test --offline -p glassmind-english-rust -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test --offline -p glassmind coding:: -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy --offline -p glassmind-english -p glassmind-english-rust -p glassmind-world-rust -p glassmind --all-targets -j1 -- -D warnings
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build --offline -p glassmind -j1
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build --offline --release -p glassmind -j1
```

The baseline native command was `run.py baseline-native SOURCE/baseline-glassmind
baseline-native`, and boundary commands were `run.py boundaries BINARY TAG`.
The first 32-row baseline was followed by the complete34-row baseline before
runtime edits and freeze; no original row or expected result was dropped.
Candidate sequence: sample3 + boundaries34; full authored20; environment10;
independent accepted-output native verification. After the structural compound
revision: English42 + codingCLI9, boundaries34 + sample3 + authored20,
environment10, clippy/release/fmt, then pre-TRAIN archive. TRAIN sequence was
sample3 (baseline0, final3), independent verification, full60 exactly once,
independent verification of its three successes. Release binary also passed
sample3 and boundaries34. Raw commands' scope identities are in reports/*.log.

Replay uses the retained binary; it does not rebuild or silently replace it:

```bash
bash tools/coding-implementation-grammar/reproduce.sh review-unique-tag
```

This wrapper collects the individually executed steps; a second complete replay
was not run. The summary/report counts refer to retained actual invocations.
The supplemental `distinct.py` and `package_probe.rs` inspect only authored
fixture outputs/API behavior; their logs preserve both finite survivors and the
cross-package refusal. `parser_audit.rs` intentionally includes the archived first
revision for comparison; it requires the retained revision1 path.

Final post-archive audit: `audit_freeze.py` reproduced two accepted forms excluded
by the frozen grammar. Its zero status confirms reproduction of the defect; the
replay wrapper exits1 after that audit. The candidate is rejected. The original
complete TRAIN run was not repeated and no contract was revised to accept it.
