# Commands and replay boundary

Working directory: /home/gsc/.cache/glassmind/continuous-20261010/inversefacts
Source: b569ac0eb0bab5a8b9d39cda904c816e77c08e63. No Rust edits.
Set these before replay (neither path uses /tmp):

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/target
export PYTHONDONTWRITEBYTECODE=1
python3 tools/facts/inverse-coverage/experiment.py verify
python3 -m unittest discover -s tools/facts/inverse-coverage -p 'test_*.py' -v
python3 tools/facts/inverse-coverage/experiment.py rebuild
```

`verify` checks the immutable selection against only TRAIN subject/relation fields,
and verifies reused artifact bytes against prior provenance. `rebuild` only reads
retained independently fetched replies, verifies hashes/query binding, and emits
candidate-index.json and merge.json outside git. Safe offline replay does not
reset budgets or contact a service.

The one live network phase used:

```sh
python3 tools/facts/inverse-coverage/experiment.py fetch
python3 tools/facts/inverse-coverage/experiment.py resume-local-validation
# Same command once more for endpoint IRI ordering; original deadline preserved.
python3 tools/facts/inverse-coverage/experiment.py resume-local-validation
```

The second command only repaired an overly strict statement-GUID casing / endpoint IRI ordering checks
before DEV; it replayed stored replies and retained the original 600-second
wall deadline, byte count, request count and frozen pairs. The original failed
validation ledger is kept as fetch-before-local-parser-fix*.json. It is NOT a
rate-limit retry, endpoint switch or second fetch budget. Do not rerun either
command for reproduction; final ledger prevents another fetch or repair.

Existing unchanged binaries built once using the required command:

```sh
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 cargo build --release \
  -p glassmind -p glassmind-facts --bins --example continuous_predictions -j1
```

Prospective evaluation sequence, run only once after candidate finalization:

```sh
python3 tools/facts/inverse-coverage/evaluate.py freeze
python3 tools/facts/inverse-coverage/evaluate.py sample
python3 tools/facts/inverse-coverage/evaluate.py full
```

Existing directories deliberately prevent repeating DEV or overwriting the
freeze. For independent inference replay, preserve the run and use a separately
reviewed output path in an adapter copy; never retune the frozen scope or rules.
The unchanged package.py, official facts score and joint-audit.py implementation
are used directly. Full question-only standalone and talk streams, relocated
streams, exact transitions, hazards and disagreements stay in run/sample or
run/full. Synthetic fixtures belong only to tests and never to fetched data.
