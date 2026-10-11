Run from the worker worktree; no full DEV. Public TRAIN adapter unchanged.

```bash
RUN=/home/gsc/.cache/glassmind/continuous-20261010/comparator-run
TRAIN=/home/gsc/.cache/glassmind/continuous-20261010/extremum-source
F=$RUN/frozen-candidate
export TMPDIR=$RUN/tmp CARGO_TARGET_DIR=$RUN/target
export GLASSMIND_CARGO_TARGET=$CARGO_TARGET_DIR/native
export GLASSMIND_SCRATCH=$F/native-scratch GLASSMIND_CHECK_CACHE=$F/native-checks
export COMPARATOR_EVIDENCE=$F/native-evidence
export DEDUP_ORDER_EVIDENCE=$F/dedup-evidence EXTREMUM_EVIDENCE=$F/extremum-evidence
scope() { TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"; }
scope cargo fmt --all -- --check
scope cargo test -p glassmind-world-rust -j1 -- --test-threads 4
scope cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
scope cargo build --release -p glassmind -j1
# For each binary (baseline/glassmind then frozen-candidate/glassmind), sample first:
scope python tools/coding-repair-dev.py --binary "$RUN/baseline/glassmind" --train "$TRAIN/train" --sources "$TRAIN/sources" --out "$F/baseline-sample" --sample
scope python tools/coding-repair-dev.py --binary "$RUN/baseline/glassmind" --train "$TRAIN/train" --sources "$TRAIN/sources" --out "$F/baseline-full" --check-cache "$F/baseline-sample/checks"
scope python tools/coding-repair-dev.py --binary "$F/glassmind" --train "$TRAIN/train" --sources "$TRAIN/sources" --out "$F/candidate-sample" --sample --check-cache "$F/baseline-first-full/checks"
scope python tools/coding-repair-dev.py --binary "$F/glassmind" --train "$TRAIN/train" --sources "$TRAIN/sources" --out "$F/candidate-full" --check-cache "$F/baseline-first-full/checks"
python tools/coding-comparator/compare.py --run "$F"
```
Actual initial baseline ordering mistake and duplicate are documented in ATTEMPTS.
Native children inherit supplied persistent TMPDIR and target subdirectories;
checks.rs runs every cargo under its unchanged resource/deadline wrapper.

Archive and audit after all runs finish:
```bash
python tools/coding-comparator/audit.py --run "$F"
python tools/coding-comparator/freeze.py --run "$RUN"
```
Final checks were repeated after adding the comparator-only older compiler-error
observation and the function type-path guard. Logs: test1.log, test2.log,
test-final.log; clippy1.log and clippy-final.log; release1.log and
release-final.log; fmt-final.log (failed wrapping check), fmt4.log and fmt-pass.log.
The unevaluated intermediate release is retained as initial-glassmind.
