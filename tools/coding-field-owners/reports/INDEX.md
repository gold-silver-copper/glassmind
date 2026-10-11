Final acceptance evidence is `accepted-*`, `world-accepted.log`,
`clippy-final.log`, `fmt-final.log`, `build-accepted.log`,
`regressions-accepted/`, `summary.json` and `accepted.patch`.
Baseline evidence is `baseline-*`, `world-baseline.log`, `audit-baseline/`,
and `regressions-baseline/`. See HANDOFF.md for failed/stale iteration labels.

`artifact-hashes.json` hashes explicitly owned fixture directories, all retained
candidate CLI binaries, intermediate source snapshots, build/test logs and final
runtime/generator source files. Relative paths are rooted at the run directory
specified in HANDOFF.md; absolute source paths refer to this checkout. It excludes
the build target cache and login/session areas. No credential files were read.
Raw files not copied here remain in that run directory. Native stdout, source
snapshots and exact expected patch checks establish fixture behavior; retained
CLI hashes identify candidate executables, not every transient Cargo test binary.
