Run from the worktree. No evaluator/heldout assets are used.

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/extremum-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/extremum-run/target
export GLASSMIND_CARGO_TARGET="$CARGO_TARGET_DIR/native"
export GLASSMIND_SCRATCH=/home/gsc/.cache/glassmind/continuous-20261010/extremum-run/repro-scratch
RUN=/home/gsc/.cache/glassmind/continuous-20261010/extremum-run
SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/extremum-source
scope() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
TMPDIR="$TMPDIR" scope cargo fmt --all -- --check
TMPDIR="$TMPDIR" scope cargo test -p glassmind-world-rust -j1 -- --test-threads 4
TMPDIR="$TMPDIR" scope cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
TMPDIR="$TMPDIR" scope cargo build --release -p glassmind -j1
scope python tools/coding-repair-dev.py --binary "$RUN/baseline/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/repro-baseline"
scope python tools/coding-repair-dev.py --binary "$RUN/frozen-candidate/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/repro-sample" --sample
scope python tools/coding-repair-dev.py --binary "$RUN/frozen-candidate/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/repro-full"
scope python tools/coding-extremum/replay-legacy.py --binary "$RUN/frozen-candidate/glassmind" --out "$RUN/repro-legacy"
```

Authored contracts: original PLAN/contracts.json freeze retained before production
edits; contracts-native.json records the documented fixture/path corrections and
stronger supplementary two-passing case. Native contracts run inside world-rust
unit tests, retain sources and separate alternative outputs in EXTREMUM_EVIDENCE
when set. Main probe remains tools/coding-repair-dev.py, unchanged.
