# Rejected work-rates batch; runtime restored

One bounded experiment at source `65e7240b074473ef761e6a81a64bbf64c2fb03f6`.
**Do not integrate the candidate patch:** full DEV added four correct answers;
the frozen acceptance criterion required five. No post-DEV rescue was attempted.
Only this new report/adapter directory remains uncommitted. No commits, downloads,
workers, model calls, services, main routing changes, scorer changes or shared
manifest changes. All owned execution scopes finished; outer worker scope remains
under coordinator control. No model weights were introduced.

## Exact experimental changes

Archived `candidate.patch` adds `crates/quantities/src/work_rates.rs`, one module
registration, a three-line parser dispatch, and one Rust integration test covering
33 frozen authored cases from `cases.json`. `Time`, signed `Rate`, `Work`, bound
agent constraints and one explicit `Stage` compose checked Rational operations.
Completion rate is one whole job / completion time, net rate is an exact sum,
stage work is elapsed time * net rate, and remaining time is remaining work /
continuing rate. Constant independent rates are explicitly stated in the method.

The parser fully consumes clauses for 2..3 arbitrary ASCII agent names, separate
or coordinated completion times, a combined time plus enough individual times to
infer exactly one unknown positive rate, simultaneous completion, and one explicit
named join/leave stage. Names and continuing agents are checked. Hours, minutes,
seconds and days convert through exact seconds; explicit query units take priority,
otherwise evidence supplies units. Partial initial capacity, varying rates,
redundant/contradictory declarations, unknown names, nonpositive individual times,
zero/nonpositive completion direction and multiple questions refuse. If a planned
stage lasts beyond completion, total completion stops at the exact earlier time
and remaining duration is zero.

This is a bounded grammar, not general work-question understanding. Stage language
is restricted to explicit `works alone for ... and then ... joins` or `they work
together for ... and then ... leaves` forms; no implicit schedules, alternating
work, partial initial volume, wage shares, efficiency relations, capacity amounts,
symbolic durations or multiple unknown rates. Mixed-unit separate declarations
convert; mixed-unit coordinated lists may refuse. Drain completion is supported
from a full container; staged drain schedules refuse. Combined drain inference,
leak narratives without explicit per-agent completion declarations and many natural
question variants remain unsupported. There was no observed staged DEV improvement;
all four new DEV answers were forward simultaneous three-agent problems. The
compositional family passed authored tests, but broader language coverage was
insufficient for the gate. Main talk routing was not tested by this worker.

## Frozen measurement and independent audit

The adapter changes only the obsolete 25-answer baseline selection to the supplied
current full4475 predictions and sets the preregistered >=34 threshold. It verifies
all question order/hashes, the supplied prediction hash, all32 old answer payloads,
unchanged exact-label-comparison-v3 FREEZE identities and legacy scorer hash.
Inference opens only Problem and saves every prediction before loading options and
correct for offline grading. First100 then full4475 were run once each after
freezing source, binary, probes and adapter in FREEZE.json. No metric, denominator,
rounding behavior or answer-selection policy changed.

| Run / metric | Baseline correct / refused / wrong | Candidate correct / refused / wrong |
|---|---|---|
| First100 v3 | 1 / 99 / 0 | 1 / 99 / 0 |
| Full4475 v3 | 29 / 4443 / 3 | 33 / 4439 / 3 |
| Full4475 legacy | 29 / 4443 / 3 | 32 / 4439 / 4 |

All32 original answer payloads are exactly unchanged. Zero new wrong/unscorable
v3 answers; zero old correct losses. All four new answers were audited independently
from Problem text (`semantic-audit.json`):

- 390: `1/(1/3+1/9+1/18) = 2 hours`.
- 706: `1/(1/25+1/5+1/10) = 50/17 days`.
- 1082: `1/(1/24+1/8+1/12) = 4 days`.
- 1828: `1/(1/3+1/9+1/15) = 45/23 days`.

No exact-versus-rounded gold mismatches occurred among new answers. Legacy rejects
390's explicit `2 hours` label; v3 correctly compares its physical duration. This
legacy discrepancy is reported separately and was not repaired in the scorer.

## Validation and rejected attempts

- Frozen authored33: baseline18/33 (all18 refusals), candidate33/33 (15 positive,
  18 adversarial). The positives cover inverse rate, stages, stage completion
  boundaries, arbitrary names, signed flows and mixed units.
- Synthetic3116: exactly identical status/value/unit outcomes,3056right including
  762expectedrefusals,60refused,0wrong. Canonical hash on both binaries:
  `93b2bd1c9f8822af95a68d02663a90d122d24ca57d730c61c670e2b11079bbe9`.
- Existing linear50, absolute55, percent-change38 and parent78: all pass.
- Scoped fmt/check, clippy all-targets with warnings denied, quantity build and
  all quantity tests pass (12 existing test functions plus new33-case test).
- Initial compile verification stopped on Clippy checked-indexing/style warnings;
  fixed before DEV. Initial probes32/33 exposed passive-query branch precedence;
  fixed before DEV. A local edit script failed to locate a formatted line and made
  no source change; the same intended pre-DEV fix was then applied successfully.
  Logs and initial failed probe evidence are retained. No failed expectations were
  deleted or weakened; PLAN/probes were frozen before runtime editing.
- Original source, patch, frozen candidate binary, reports and raw evidence were
  archived and hash-verified before rollback. `git diff` is empty for all tracked
  runtime files; owned new module/test removed only after preservation. The restored
  baseline source builds. `git apply --check candidate.patch` succeeds.

## Data and provenance

MathQA source version `fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0 per
supplied source card. Reused TRAIN parquet9,013,733bytes and DEV parquet1,350,141bytes;
no official test material accessed. New download bytes: **0**. Authored development
fixture version `work-rates-public-development-v1`, MIT OR Apache-2.0,33rows,7220bytes.
Exact hashes are in data-provenance.json. No annotations, formulas, programs,
operations, categories or options were used to choose implementation behavior.

Access deviation: a full source-card read accidentally displayed one unrelated
annotated combinatorics example. It was not used in this work. TRAIN parquet reads
projected Problem only, first4000 for design. DEV options/correct were read only
after frozen prediction persistence for scoring; no other DEV annotations read.
No credentials or private logs were accessed; no API keys or LLM calls used.

## Artifacts and reproduction

Raw evidence: `/home/gsc/.cache/glassmind/continuous-20261010/workrates-run/evidence`.
Original archived sources: sibling `workrates-run/submitted`, with MANIFEST.json
and RAW-MANIFEST.json verified before rollback. The archive contains the original
source files, adapter, plan, probes, patch and pre-rollback audit. Large binary,
full predictions, ledgers and logs stay outside the repository.

- Candidate binary SHA256:
  `19a3acfb072b32b7d79ff4d5894b946b083857d552ec05a0220664cdf3d30d55`.
- Candidate patch SHA256:
  `a307d2218bca1adb4f9b28d7bb9b7f23f6b1c22139e4917777c4318aa4a16d7b`.
- Full prediction SHA256:
  `a9e2d18fcb0b6166f3c21ef25627550145de57013b052dc26bf77451000215a1`.

Reproduction requires a disposable checkout at the source revision with this owned
directory present, and applying the **rejected experimental patch** before building:

```sh
git apply tools/quantities/work-rates/candidate.patch
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/workrates-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/workrates-run/target
bash tools/quantities/work-rates/reproduce.sh /home/gsc/.cache/glassmind/continuous-20261010/workrates-run/reproduction-new
```

The unchanged script uses the retained sibling baseline, readonly MathQA mount and
existing PyArrow dependency path. Every cargo/native inference invocation uses
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`; cargo uses `-j1` except fmt, tests `--test-threads 4`.
Do not run reproduce.sh against the restored runtime and call it candidate replay.
The existing archived candidate binary can be used for scoring-only replay without
rebuilding. No main build/routing or unrelated coding benchmark claim is made.
