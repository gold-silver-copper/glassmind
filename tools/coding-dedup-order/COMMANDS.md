Run from the worker worktree. All generated native cargo uses unchanged
checks::run_in_until isolation, fingerprint caches, caps and deadline controls.

```bash
RUN=/home/gsc/.cache/glassmind/continuous-20261010/deduporder-run
SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/extremum-source
export TMPDIR="$RUN/tmp"
export CARGO_TARGET_DIR="$RUN/target"
export GLASSMIND_CARGO_TARGET="$CARGO_TARGET_DIR/native"
export GLASSMIND_SCRATCH="$RUN/native-scratch"
export GLASSMIND_CHECK_CACHE="$RUN/native-checks"
export DEDUP_ORDER_EVIDENCE="$RUN/native-evidence"
export EXTREMUM_EVIDENCE="$RUN/extremum-evidence"
scope() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
scope cargo fmt --all -- --check
scope cargo test -p glassmind-world-rust -j1 -- --test-threads 4
scope cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
scope cargo build --release -p glassmind -j1
scope python tools/coding-repair-dev.py --binary "$RUN/baseline/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/baseline-full"
scope python tools/coding-repair-dev.py --binary "$RUN/baseline/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/baseline-sample" --sample --check-cache "$RUN/baseline-full/checks"
scope python tools/coding-repair-dev.py --binary "$RUN/frozen-candidate/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/candidate-sample" --sample --check-cache "$RUN/baseline-full/checks"
scope python tools/coding-repair-dev.py --binary "$RUN/frozen-candidate/glassmind" --train "$SOURCE/train" --sources "$SOURCE/sources" --out "$RUN/candidate-full" --check-cache "$RUN/baseline-full/checks"
python tools/coding-dedup-order/compare.py --run "$RUN"
```

Additional final evidence commands:
```bash
scope python tools/coding-dedup-order/replay.py --binary "$RUN/frozen-candidate/glassmind" --out "$RUN/ambiguity-replay"
python tools/coding-dedup-order/audit.py --run "$RUN"
python tools/coding-dedup-order/freeze.py --run "$RUN"
```
The supplied target is reused, with separate native subdirectories. The fixed
probe reuses only this worker's content-fingerprinted baseline cargo observations.
No original source is edited during diagnostics; offered repairs are applied
only to independent verification copies. `cases.py` recreates the original25;
`supplement.json` is separately frozen review coverage and is not overwritten.
