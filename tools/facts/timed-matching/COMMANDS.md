# Commands executed

Working directory: `/home/gsc/.cache/glassmind/continuous-20261010/timedmatch`.
`S=tools/facts/timed-matching/scoped.sh`; every command below invoked through `$S`.
The wrapper exports persistent TMPDIR and existing CARGO_TARGET_DIR and executes
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 "$@"`.
No cargo command was invoked without this wrapper. All test commands used4threads.
Filesystem reads/edits/copies were ordinary foreground operations.

```
cargo test -p glassmind-facts -j1 --lib --test matching --test temporal -- --test-threads 4
python tools/facts/timed-matching/freeze.py
cargo build -p glassmind-facts --release --examples -j1
python tools/facts/timed-matching/legacy.py baseline
cargo fmt -p glassmind-facts
cargo test -p glassmind-facts -j1 --test timed_matching fixed_sample -- --test-threads 4
cargo test -p glassmind-facts -j1 --lib --test matching --test temporal --test timed_matching -- --test-threads 4
cargo clippy -p glassmind-facts -j1 --all-targets -- -D warnings
cargo build -p glassmind-facts --release --bins --examples -j1
python tools/facts/timed-matching/real.py
python tools/facts/timed-matching/legacy.py current
```

After private-store/typed-count refinement, fmt, full affected tests, clippy and
facts release build above repeated; their `final-*` logs retain the results.
Original `real/` moved to `real-first/`, then the exact same real.py denominator
reran into `real/`. Supplemental test additions then ran:

```
cargo test -p glassmind-facts -j1 --test timed_matching -- --test-threads 4
cargo fmt -p glassmind-facts -- --check
cargo clippy -p glassmind-facts -j1 --all-targets -- -D warnings
cargo build -p glassmind --release -j1
python tools/facts/timed-matching/cli_replay.py
python tools/facts/timed-matching/joint.py
python tools/facts/timed-matching/main_replay.py
python tools/facts/timed-matching/summarize.py
python tools/facts/timed-matching/raw_checks.py
rustc --version
```

`legacy.py`, `cli_replay.py`, `main_replay.py` and `real.py` retain exact native
arguments in source and outputs in RUN/evidence. The native scorer and existing
joint-audit.py were not modified. `reproduce.sh` is the current-source rerun.
