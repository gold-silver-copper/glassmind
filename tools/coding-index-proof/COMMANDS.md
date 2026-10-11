# Executed commands and reproduction

Working directory: `/home/gsc/.cache/glassmind/continuous-20261010/indexproof`.
Final environment (inherited target/TMPDIR explicitly retained):

```sh
export OWNRUN=/home/gsc/.cache/glassmind/continuous-20261010/indexproof-run
export TMPDIR="$OWNRUN/tmp"
export CARGO_TARGET_DIR="$OWNRUN/target"
export GLASSMIND_SCRATCH="$OWNRUN/scratch"
export GLASSMIND_CARGO_TARGET="$CARGO_TARGET_DIR"
```

`reproduce.sh` contains the exact cap and core Cargo/replay commands. The actual
final commands used outputs `world-tests-final.log`, `fmt.log`, `clippy.log`,
`build-final.log`, `final-sample.log`, `final-full.log`. The accepted binary was
copied from `$CARGO_TARGET_DIR/debug/glassmind` to `$OWNRUN/accepted-glassmind`.
Initial build logs and candidate-sample/full iterations are retained separately.
The consolidated reproduction script itself was not rerun; its commands were run
individually as documented and logged.

Existing adapters were run unmodified:

```sh
python3 tools/coding-answerfiles-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/exports"
python3 tools/coding-metatypes-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/metatypes"
python3 tools/coding-answerfiles-regression.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/queries"
python3 tools/coding-diagnostics-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/diagnostics"
python3 tools/coding-addfields-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/addfields" --independent
python3 tools/coding-addfields-dev.py --binary "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/addfields14" --independent --cases alias-self unrelated default collision alias unknown-macro update cfg trailing unsafe-default unsafe-function test-fails type-mismatch local-shadow
python3 tools/coding-addfields-dev.py --binary "$OWNRUN/baseline/glassmind" --output "$TMPDIR/addfields-baseline-failure" --independent --cases live-rollback
python3 tools/coding-askverify/run.py --binary "$OWNRUN/accepted-glassmind" --out "$TMPDIR/askverify"
```

Export is intentionally not treated as all-pass. Both AddField full and baseline
live-rollback commands exit with the retained exception. No scorer or expectation
was changed. An initial askverify invocation incorrectly used `--output`, exited
before tests, and was corrected to `--out`; both logs remain. One exploratory
summary command used `cases` instead of the export report's `rows` key and failed;
this did not alter any inputs, predictions, or report.

Final proof sources and compiler outputs:
`reports/width-probes/width-{32,128}/src/lib.rs` and
`reports/width-probes/results.json`. Original sources and behavior tests for each
repair: `reports/final-full/CASE/{before,expected,after,behavior}.rs` with
`original-diagnostic.json`, `native.json`, and `result.json`.
