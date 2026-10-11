# Reproduction
Run from this worktree. All commands below use the supplied persistent locations:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/argumentswap-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/argumentswap-run/target
RUN=/home/gsc/.cache/glassmind/continuous-20261010/argumentswap-run
cap() { systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
cap cargo fmt --all -- --check
GLASSMIND_SCRATCH="$RUN/test-scratch" GLASSMIND_CARGO_TARGET="$CARGO_TARGET_DIR/test-native" cap cargo test -p glassmind-world-rust -j1 -- --test-threads 4
cap cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
cap cargo build -p glassmind --release -j1
```

For each binary (`$RUN/baseline/glassmind` then `$RUN/frozen-candidate/glassmind`), using a fresh output label (`baseline` or `guarded`, the final candidate):

```bash
cap python3 tools/coding-repair-dev.py --binary "$BINARY" --train /home/gsc/.cache/glassmind/data/train-v3 --sources /home/gsc/.cache/glassmind/sources --out "$RUN/$LABEL-sample" --sample
cap python3 tools/coding-repair-dev.py --binary "$BINARY" --train /home/gsc/.cache/glassmind/data/train-v3 --sources /home/gsc/.cache/glassmind/sources --out "$RUN/$LABEL-full" --check-cache "$RUN/$LABEL-sample/checks"
cap python3 tools/coding-argument-swap/native.py --binary "$BINARY" --out "$RUN/authored-$LABEL"
cap python3 tools/coding-argument-swap/native.py --binary "$BINARY" --out "$RUN/supplement-$LABEL" --contracts tools/coding-argument-swap/supplement.json
cap python3 tools/coding-argument-swap/replay_choice.py --run "$RUN" --label guarded
python3 tools/coding-argument-swap/summarize.py --run "$RUN" --label guarded
```

The unchanged TRAIN probe and authored runner wrap every generated cargo test in the same 8 GiB/zero-swap scope, `setsid --wait`, 600s timeout with 10s kill grace, `-j1` and four test threads. The Rust verifier also retains its existing per-child cleanup/deadline implementation. No new services, model calls, dependencies or downloads are needed. Original failures and intermediate build/clippy logs remain in RUN; use fresh output paths for additional replays to avoid overwriting evidence.
