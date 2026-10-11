#!/usr/bin/env bash
set -euo pipefail
: "${TMPDIR:?Persistent TMPDIR required}" "${CARGO_TARGET_DIR:?Use existing target directory}"
[[ "$TMPDIR" != /tmp* ]]
run=$(realpath "${1:?Owned run with baseline and dev-inputs}")
out=$(realpath -m "${2:?Fresh output directory}")
mkdir -p "$out"
export CARGO_NET_OFFLINE=true GLASSMIND_SCRATCH="$out/scratch"
scope=(systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600)
sha256sum -c tools/coding-function-refs/FROZEN.sha256
for split in sample full; do
  flags=(); [[ "$split" != sample ]] || flags+=(--sample)
  python tools/coding-function-refs/evaluate.py --binary "$run/baseline/glassmind" --output "$out/baseline-$split" "${flags[@]}" --supplied "$run/dev-inputs/workspace"
done
"${scope[@]}" cargo fmt -p glassmind-world-rust -- --check
"${scope[@]}" cargo test -p glassmind-world-rust -j1 -- --test-threads 4
"${scope[@]}" cargo clippy -p glassmind-world-rust --all-targets -j1 -- -D warnings
"${scope[@]}" cargo test -p glassmind -j1 edit::tests -- --test-threads 4
"${scope[@]}" cargo build -p glassmind -j1
cp "$CARGO_TARGET_DIR/debug/glassmind" "$out/glassmind"
for split in sample full; do
  flags=(); [[ "$split" != sample ]] || flags+=(--sample)
  python tools/coding-function-refs/evaluate.py --binary "$out/glassmind" --output "$out/candidate-$split" "${flags[@]}" --supplied "$run/dev-inputs/workspace"
done
python tools/coding-function-refs/contracts.py --binary "$out/glassmind" --inputs "$run/dev-inputs" --out "$out/contracts"
bash tools/coding-function-refs/regressions.sh "$out/glassmind" "$out/regressions"
# Inspect all result JSON/status files: inherited expectation failures are retained.
