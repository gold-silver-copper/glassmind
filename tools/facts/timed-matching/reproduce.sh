#!/bin/bash
# Run from repository root; uses the read-only existing sources and RUN baseline.
set -euo pipefail
S=tools/facts/timed-matching/scoped.sh
$S cargo fmt -p glassmind-facts -- --check
$S cargo test -p glassmind-facts -j1 --lib --test matching --test temporal --test timed_matching -- --test-threads 4
$S cargo clippy -p glassmind-facts -j1 --all-targets -- -D warnings
$S cargo build -p glassmind-facts --release --bins --examples -j1
$S cargo build -p glassmind --release -j1
$S python tools/facts/timed-matching/real.py
$S python tools/facts/timed-matching/legacy.py current
$S python tools/facts/timed-matching/joint.py
$S python tools/facts/timed-matching/main_replay.py
$S python tools/facts/timed-matching/cli_replay.py
$S python tools/facts/timed-matching/summarize.py
$S python tools/facts/timed-matching/raw_checks.py
