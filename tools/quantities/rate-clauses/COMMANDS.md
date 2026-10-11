# Commands and reproduction

All commands ran from
`/home/gsc/.cache/glassmind/continuous-20261010/rateclauses` unless stated.
The inherited CARGO_TARGET_DIR was
`/home/gsc/.cache/glassmind/continuous-20261010/rateclauses-run/target`.
Every cargo command explicitly used this wrapper (not a shell alias):

```sh
TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/rateclauses-run/tmp \
 systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
 setsid --wait timeout --kill-after=10s 600 cargo ARGS
```

Actual16 cargo invocations, in order, with the ARGS and output file:

|#|ARGS|Output|
|---|---|---|
|1|test -p glassmind-quantities -j1 --test rate_clauses -- --test-threads 4|tool output:1group passes40cases|
|2|test -p glassmind-quantities -j1 --test rate_clauses -- --test-threads 4|authored-tests.log, failed|
|3|test -p glassmind-quantities -j1 --test rate_clauses -- --test-threads 4|authored-tests-2.log|
|4|test -p glassmind-quantities -j1 -- --test-threads 4|quantity-tests.log, failed|
|5|test -p glassmind-quantities -j1 -- --test-threads 4|quantity-tests-2.log|
|6|test -p glassmind-quantities -j1 --test rate_clauses -- --test-threads 4|authored-tests-3.log, failed|
|7|test -p glassmind-quantities -j1 --test rate_clauses -- --test-threads 4|authored-tests-4.log|
|8|fmt -p glassmind-quantities|fmt.log|
|9|clippy --offline -p glassmind-quantities -j1 --all-targets -- -D warnings|clippy.log, failed|
|10|fmt -p glassmind-quantities|fmt-2.log|
|11|clippy --offline -p glassmind-quantities -j1 --all-targets -- -D warnings|clippy-2.log|
|12|build --offline -p glassmind-quantities -j1|build.log|
|13|fmt -p glassmind-quantities|fmt-final.log|
|14|clippy --offline -p glassmind-quantities -j1 --all-targets -- -D warnings|clippy-final.log|
|15|test --offline -p glassmind-quantities -j1 -- --test-threads 4|tests-final.log|
|16|build --offline -p glassmind-quantities -j1|build-final.log|

All redirected logs live in tools/quantities/rate-clauses. The candidate binary
was copied from target/debug/glassmind-quantities, then source/binary/adapter/probes
were hashed into FREEZE.json before running these actual checks/inference:

```sh
RUN=/home/gsc/.cache/glassmind/continuous-20261010/rateclauses-run
REPORT=tools/quantities/rate-clauses
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
export PYTHONDONTWRITEBYTECODE=1
cp "$RUN/baseline/quantities" "$TMPDIR/glassmind-quantities-baseline"
python "$REPORT/regressions.py" --binary "$RUN/candidate/quantities" \
 --baseline "$RUN/baseline/quantities" --output "$REPORT/regressions.json"
CANDIDATE_BINARY="$RUN/candidate/quantities" python tools/quantities/linearwords/check.py \
 > "$REPORT/linear-check.log"
cp "$TMPDIR/checks.json" "$REPORT/linear-check.json"
python tools/quantities/linearwords/absolute.py --binary "$RUN/candidate/quantities" \
 --output "$REPORT/absolute.json"
python "$REPORT/evaluate.py" --binary "$RUN/candidate/quantities" \
 --baseline "$RUN/baseline" \
 --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa \
 --output "$REPORT/reports/sample" --sample > "$REPORT/sample.log" 2>&1
python "$REPORT/evaluate.py" --binary "$RUN/candidate/quantities" \
 --baseline "$RUN/baseline" \
 --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa \
 --output "$REPORT/reports/full" > "$REPORT/full.log" 2>&1
```

After the failed full gate, candidate.patch/source/binary/reports were copied and
SHA256-verified, then tracked lib/parser bytes restored using git show HEAD:PATH;
new owned module/test removed. Post-DEV `python tools/quantities/rate-clauses/audit.py`
verified3 independent calculations; `git apply --check .../rejected/candidate.patch`
passed on the restored baseline. No source retuning or repeated DEV inference.

## Reproduce the rejected candidate in a disposable checkout

These reproduction setup commands are provided, not claimed executed. Do NOT
apply the rejected patch to a production checkout. Preserve the retained binary
and source hashes. Existing reports intentionally cannot be overwritten.

```sh
TASK=/home/gsc/.cache/glassmind/continuous-20261010/rateclauses
RUN=/home/gsc/.cache/glassmind/continuous-20261010/rateclauses-run
REPLAY="$RUN/replay-source"
git worktree add --detach "$REPLAY" bb89964dd887b2b68ae01fb8f0a9a92a4901f490
cp -a "$TASK/tools/quantities/rate-clauses" "$REPLAY/tools/quantities/"
cd "$REPLAY"
git apply tools/quantities/rate-clauses/rejected/candidate.patch
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
export PYTHONDONTWRITEBYTECODE=1
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
TMPDIR="$RUN/tmp" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
 setsid --wait timeout --kill-after=10s 600 \
 cargo test --offline -p glassmind-quantities -j1 -- --test-threads 4
# Replay original frozen binary; a rebuilt debug binary can differ by source path.
python tools/quantities/rate-clauses/evaluate.py --binary "$RUN/rejected/quantities" \
 --baseline "$RUN/baseline" \
 --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa \
 --output "$RUN/replay-results/sample" --sample
python tools/quantities/rate-clauses/evaluate.py --binary "$RUN/rejected/quantities" \
 --baseline "$RUN/baseline" \
 --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa \
 --output "$RUN/replay-results/full"
```

This adapter verifies the frozen candidate source too, so running it directly from
the rolled-back source tree correctly fails hash validation. Use the disposable
candidate checkout above. No runtime data is needed beyond JSON-string Problems;
MathQA labels enter only the external offline scorer after predictions are saved.
The retained binary is usable independently without any dataset or Python.
