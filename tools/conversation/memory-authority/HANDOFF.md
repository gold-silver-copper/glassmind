# HANDOFF — memory authority, useful candidate left uncommitted

Base: `9b875b854908cbbd5f5a6c8fce73ddcdbbb8ec8f` (production with explicit factsets).
No commits, new agents, LLM calls, services, downloads, training or credential
reads. Only permitted extracted CLINC TRAIN texts were inspected; TRAIN/VAL
files were hashed. Outer scope verified MemoryMax=8589934592, MemorySwapMax=0.
All cargo invocations used the supplied disk TMPDIR, existing CARGO_TARGET_DIR,
systemd scope 8GiB/0swap, setsid --wait, timeout 600/kill-after10s, -j1 except fmt,
and test threads4. No owned jobs remain running.

## Changes

- `crates/everyday/src/memory.rs`: typed-authority API retaining the original
  `respond` API/JSON, clear operand guidance for incomplete statements and
  accepted `change_user_name` proposals without typed names; invalidate stale
  correction focus after fallback/social/last-utterance/missing-value turns,
  including old JSON with a focus pointing to an absent value. Explicit writes,
  values, source turns and correction provenance remain intact. Four unit tests.
- `crates/cli/src/everyday.rs`: honor typed refusals before other domains;
  retain existing calculation continuation; allow learned personal reads with
  no story context only after checking no supported typed story operation.
  Reuse the evaluated fallback rather than predicting twice. No story solver,
  facts solver, quantity solver, classifier, thresholds, main CLI or scorer edits.
- New `docs/memory-authority.md` and this owned adapter/report directory.
  Original smoke definitions and explicit factsets route are unchanged.

## Frozen hypothesis and outcomes

PLAN.md, contracts.json, freeze.py and evaluate.py were hashed BEFORE runtime
edits. Fixed sample = first four authored sessions; full = all16 sessions,
93 reply/state checks, followed by unchanged original40 reply checks plus
one persistence/provenance check. Expectations never enter runtime input.

| Check | Baseline without model | Candidate without model | Baseline actual model | Candidate actual model |
|---|---:|---:|---:|---:|
| Fixed sample | 13/23 | 23/23 | 11/23 | 23/23 |
| New full contracts | 68/93 | 93/93 | 64/93 | 93/93 |
| Original memory41 | 41/41 | 41/41 | 40/41 | 41/41 |

The preserved original failure is `Call me tomorrow`: accepted
`change_user_name` became generic intent despite no typed name. The candidate
returns refusal and preserves the proposal without claiming the name changed.
Baseline artifacts and every failed baseline assertion are retained.

Actual-model calc27 =27/27 and story47 =47/47 for both binaries; reports compare
identically after excluding only story-report binary hash provenance. All
actual response objects remain identical. Archived factsets36 +session16 =52/52.
Four additional streams (model on/off × bundle on/off),19 replies each, have
byte-identical baseline/candidate output AND saved session files. Explicit
factsets route source prefix is byte-identical too.

Four everyday unit tests and two existing CLI everyday tests pass. Release build
for both binaries, affected all-target clippy with -D warnings, workspace fmt
--check and git diff --check pass. Commands/logs are in reports; the consolidated
reproduce.sh was assembled from executed individual commands, not rerun as a unit.

## Supplemental review limitation (not a preregistered win)

After implementation, learned_social.py was separately frozen and executed;
no runtime edits followed. Main `talk` still sends bare `goodbye` into the story
parser before learned social fallback, then refuses the following correction.
Baseline AND candidate: plain5/6, model4/7; stored name/source remain unchanged.
The same fixed check through the standalone everyday chat passes plain6/6 and
actual-model7/7. This demonstrates retained learned social handling in the
owned memory layer but leaves an inherited main talk routing issue unresolved.
These failures are retained and NOT counted as passing original/new93 checks.
No claim of complete general social/story routing acceptance is made.

The unchanged typed `Call me` capitalization heuristic remains narrow and can
be lexically ambiguous. No phrase blacklist was added. Calculation focus is an
explicit compatibility exception required by the existing calc27 contract.
No independent benchmark or unrestricted language understanding is claimed.

## Rejected attempts and scope

One measured runtime candidate; no measured candidate was rejected/restored.
Before editing, rejected global model disabling and blanket calculation-focus
clearing on contract grounds (not claimed as measured experiments). Baseline
revealed typed empty statements and learned reads being intercepted by stories;
the scoped authority-routing fix was included before candidate evaluation.
General social routing was left for separate review after the supplemental
failure, rather than altering those checks or extending this batch.

## Data and identities

No new third-party data: **0 downloaded bytes**. Local authored frozen contracts:
**12,617 bytes**,16 synthetic sessions, repository **MIT OR Apache-2.0** license.
Authored generator source is freeze.py; its hashes and all source/binary/data
hashes and sizes are recorded in identities.json and baseline-hashes.json.
CLINC source: Larson et al. (2019), clinc/oos-eval revision
`828f8093932c8fe6ca7936c3d2e52903b1c523de`, **CC-BY-3.0**. Existing extracted
TRAIN15,000 rows/1,165,484 bytes; VAL3,100 rows/239,360 bytes. TRAIN examples and
counts for change_user_name, user_name, repeat, change_ai_name, reminder_update,
weather and greeting are retained in train-evidence.json with attribution here.
No official test contents, hidden generators or private logs were accessed.

| Artifact | SHA256 |
|---|---|
| Baseline main binary | `623e8c0c9556af2cf6c0298cb00a436bcabb09590736955f04e0bf3b8a9677ac` |
| Candidate main binary | `62b812cad756d4044eb3272871055002cf488cf0179663f64df6967831c0d198` |
| Candidate everyday binary | `6d639341e598a7ce69ce05eb1d20d225e2d5118748e3fc1048a8d370f6ce20a0` |
| memory.rs | `291fa35b91a608af485c723925c9bea431e6006b060205993ba3ffa56e211c11` |
| CLI everyday.rs | `baa765a6c624fb8a721f5284c22f97d5f5dd607c4a2d164f8ae02ac7e747e55d` |
| Classifier source (unchanged) | `41ee730bb6351f29ff8d0d0ef9791062405f0b3f62fb3e35c115d43dec21e5a7` |
| Production intent model (25,524,518 bytes, unchanged) | `a84fd59c3e49db08300f61ee33f6d73e64a8ea18109102b01685333b95936408` |
| TRAIN | `f27778e9c16448d040d9c364a1b369493711865419613a277270a54781ea87c2` |
| VAL | `39afa094ba8495e9b8b30595394937bdd7b5b8e6dcdba71a9a380047759c88f9` |
| Frozen contracts | `4309d3c42eb7c5256cddbf7f9cc7474e4525356e52a6246d16a3e0af2d1e4db5` |
| Original memory smoke (unchanged) | `eee606b8dd2dc3497dac124625a260371ac087352f3c5a649950e90434a01729` |
| Runtime patch | `0c6394eb2b551460760b491dea5f5504a360aa80148e12496637dfca6e905d09` |

Raw full3100 intent evaluation was NOT rerun. Frozen classifier/adapter source,
model and dataset hash identities were verified against the baseline; unchanged
raw predictions are a structural expectation, not a claimed fresh evaluation.
No historical prediction logs were read or compared.

## Reproduction

From this worktree, preserving the supplied target directory:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/memoryauthority-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/memoryauthority-run/target
export BASELINE=/home/gsc/.cache/glassmind/continuous-20261010/memoryauthority-run/baseline/glassmind
export MODEL=/home/gsc/.cache/glassmind/models/everyday/intent.json
export OUT=tools/conversation/memory-authority/replay
bash tools/conversation/memory-authority/reproduce.sh
```

The script reproduces baseline failures, sample/full checks, original regressions,
facts byte comparison and supplemental social limitation; core checks assert
93/93+41/41 while supplemental failures remain explicitly reported. No eval
expectations are rewritten. Keep this candidate uncommitted for coordinator
independent review/full affected regression before publishing.
