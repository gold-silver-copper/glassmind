# HANDOFF — prospective social routing

Status: useful candidate; all specified local acceptance gates PASS.
Uncommitted, coordinator independently replays/integrates. No owned jobs remain.
Base1331ac715eadcf7b9169423e947cedf861eb42a3. Only runtime changes are
crates/cli/src/everyday.rs and crates/cli/src/talk_stories.rs. New owned adapter,
reports and docs/social-routing.md. No shared manifests, main CLI, memory engine,
classifiers, thresholds, models, solvers, original scorers/fixtures or prompts edited.

## Changes

- Main talk accepts a fully consumed bounded social form before unsupported
  story assertion invalidation, reusing the existing memory/model response.
  Learned intent alone never authorizes this bypass. Without a model, the
  nonliteral forms still return the original ordinary refusal, preserving story
  context and clearing personal correction focus.
- Explicit personal memory authority, calculations/facts and supported story
  operations retain priority. A social prefix plus an explicit appended clause
  cannot become a coordinated story actor. Unconsumed text after a complete
  social prefix also invalidates when no supported story operation consumes it.
  A leftover social proposal through the ordinary question/command fallback
  becomes an explicit refusal rather than a confident greeting.
- Story adapter exposes a non-mutating plan and a single apply transition.
  Its existing parsing, safety checks, question reasoning, limits, invalidation
  and normalized persisted assertions are unchanged. The old eager disposable
  story probe is removed. No story mutations occur during probing.
- One new routing unit test covers preservation plus two mixed-clause cases;
  one story unit test verifies repeated probes cannot mutate history/world,
  followed by exactly one accepted state transition.

## Frozen gate and results

PLAN.md, freeze.py, contracts.json and evaluate.py were hashed before runtime
edits. Fixed sample first6 sessions; full41 sessions/490 checks each mode.
Seven social forms include learned salutations, farewells, thanks, conversation
closings and wellbeing questions. Contexts none/active/invalid, repeated calls,
process restarts, exact last utterance, turns, names/preferences/source turns,
focus/corrections and reset are covered. Fifteen contrasts exercise movement,
unsupported movement, appended clauses/commands, negation, quoted speech,
questions versus assertions, and corrections; five typed-priority checks.

| Gate | Baseline plain | Final plain | Baseline model | Final model |
|---|---:|---:|---:|---:|
| Fixed6 sessions | 118/126 | 126/126 | 106/126 | 126/126 |
| Full41 sessions | 467/490 | 490/490 | 437/490 | 490/490 |
| Original supplemental | 5/6 | 6/6 | 4/7 | 7/7 |
| Memory93 + original41 | 134/134 | 134/134 | 134/134 | 134/134 |

Changed-event audit:50 events in each mode, all inputs/replies/state retained;
all84 saved social snapshots replay correctly through fresh processes. Actual
social labels/texts audited: greeting/Hello!, goodbye/Goodbye!, thank_you/
You're welcome.; personalized literal Hello retained. No failed final contracts.
Original calc27/story47 are identical and pass with the production model;
original facts36+16 pass for both baseline and final binaries. All11 scoped
Rust tests pass (3CLI everyday,4story adapter,4memory), release build, affected
all-target clippy -D warnings, fmt --check and git diff --check pass.

Full bAbI TRAIN-derived story dev with production intent model: sample400,
397correct/3refused; full3987 questions/47625 turns,3966correct/18refused/3wrong.
All response objects byte-identical baseline/candidate at every turn, zero new
wrong answers or lost typed assertion responses. All inherited failures retained.
Full ordinary facts2821 and native765 belong to parent and were NOT rerun here.
Raw intent3100 NOT rerun. No hidden/official test material or private logs read.

## Rejected attempt and deviations

Candidate1 fixed the social interception and passed full story byte equivalence,
but a separate additional frozen review probe exposed an inherited stale-answer
path: `how are you Elin teleported to the attic` followed by `Where is Elin?`.
Baseline returned an intent and stale location (0/2); candidate1 refused the first
turn but retained stale location (1/2). Candidate2 requires consuming the rest
of a known social prefix before preserving stories (2/2). This supplemental
probe is not represented as part of the original preregistered490 checks.
The candidate1 patch/source/binary hashes and partial checks are archived in
reports/candidate1 and ../socialrouting-run/candidate1; its unfinished session
and regression launches were stopped using their exact owned process trees.
No incomplete candidate1 run is counted as passing. Full final checks rerun.

The first baseline invocation omitted --sample and thus ran full41 first despite
its folder name baseline-sample. The explicit fixed sample followed. This is a
measurement-order deviation, not a fixture change; both original outputs remain.
Candidate2 sample precedes full. No thresholds, fixtures or expectations changed.
No global model disabling, goodbye blacklist or blanket invalidation suppression.

## Provenance and identities

No downloads, new model calls, agents, services, API keys or credential reads.
Outer scope MemoryMax8589934592/MemorySwapMax0 verified. Every cargo (including
fmt) used persistent supplied TMPDIR, existing CARGO_TARGET_DIR, systemd user
scope8GiB/0swap, setsid --wait, timeout600/kill-after10s; -j1 except fmt and test
threads4. Source and binary identities are in identities.json, original sources,
model, TRAIN/VAL and all20 bAbI TRAIN member hashes in baseline-hashes.json.
Original frozen artifacts verified unchanged with sha256sum -c.

Authored contracts version1: 4374 bytes,41 sessions, repository MIT OR Apache-2.0.
CLINC Larson et al.2019 / clinc/oos-eval revision
828f8093932c8fe6ca7936c3d2e52903b1c523de, CC-BY-3.0; existing extracted
TRAIN15000 rows/1165484bytes, VAL3100 rows/239360bytes. Only TRAIN forms inspected;
VAL hashed only. train-evidence.json retains first20 texts from each of six
relevant100-row intents (4289 derived bytes). No model/threshold selection/training.
bAbI Facebook tasks1-20 v1.2 English1k, CC-BY-3.0: existing20 TRAIN files,
2947923bytes; unchanged public seed/episode split and normalization from
tools/stories/babi.py. No official test members read. Downloads: **0 bytes**.
Production model unchanged25524518bytes, SHA256
`a84fd59c3e49db08300f61ee33f6d73e64a8ea18109102b01685333b95936408`.
Baseline binary `62b812cad756d4044eb3272871055002cf488cf0179663f64df6967831c0d198`:
frozen memory-authority binary, conversation source matches1331ac7; provisional
native isolation absent from that baseline binary, irrelevant to talk.
Evaluated full-session binary `4dac87aa5361a8d3360ba3025b28385b1c87d3c40f24159d1bcbfc816ba32d3b`.
Final source rebuild binary `900973a03613904ec656ccaa92187a14243bb1ac7e534570151fdfd38182a754`.
A test-only edit landed during the first release compilation, so the final
rebuild changed debug/build identity. Reconstructing the earlier test-only source
reproduced4dac exactly; restored final source rebuilt900973 exactly. All30
allocated ELF sections (including exact bytes, addresses, sizes, flags and
alignment) are identical except the deliberately excluded GNU build-id note;
nonallocated debug/symbol/section metadata differ. binary_equivalence.py and
reports/binary-equivalence.json retain this explicit equivalence audit. A naive
whole stripped-file comparison still differs in section-table metadata; no
whole-file equality is claimed. Both full executable artifacts are retained in
../socialrouting-run/provenance.

The final900973 binary directly reran sample/full story gates (same raw bytes
and3966/18/3), and plain sample126/full490. Model full490, memory/regression
and84-state audit ran on4dac, whose executable sections are identical; those
expensive model restart checks were not additionally rerun on900973. This exact
binary distinction is a reproducibility limitation, not silently relabeled
results. Parent independent final integrated replay remains required.
Final everyday.rs `dc39b5da752a9c311711b72fc6123d8c51c6fc65ce7856941fb5ba9c775f4908`.
Final talk_stories.rs `4f0dfc287c4f51fe90a8becb00041f9dd090bf51938dccc65551b27c3430482c`.

## Reproduce and limits

From this worktree (same existing target, persistent scratch):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/socialrouting-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/socialrouting-run/target
export BASELINE=/home/gsc/.cache/glassmind/continuous-20261010/socialrouting-run/baseline/glassmind
bash tools/conversation/social-routing/reproduce.sh
```

This script assembles executed individual commands; it was not additionally
rerun as a single unit. Original public scorers imported/called without edits.
Changed social replies and states are retained in changed-social*.json, complete
raw inputs/replies/state in each events.json; changes-summary.json lists every transition category; state-audit.json independently
restarts each social snapshot and asks for the known referent. Unit/build/clippy/
format logs remain in reports. Raw full story replies and inherited failures stay.

The social grammar is deliberately finite; this does not establish open-domain
social understanding. Unconsumed social proposals may refuse greetings outside
these full forms. Ambiguous social-prefix/entity names can be conservatively
refused; the complete affected story corpus has no changed assertions/answers.
Unknown language outside these bounded forms still relies on existing story
safety behavior, not a general clause parser. No external service execution.
Parent should independently replay the actual integrated binary and full
ordinary facts2821 before publishing. No commit or deployment performed.

Final runtime patch SHA256 `9aca1750252fc53743b2529483b6d7973a59a7d72450ff3c73ad639373075297`.
Full baseline and final story-output SHA256 both
`8df8189e8022cf436aa1aaf6e0aaeaabced18cd3b01b41faedb4f1d19010b480`.
All retained owned files are indexed with exact sizes/hashes in artifacts.json.
