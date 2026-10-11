# Percent change: worker gate passed, uncommitted candidate

One bounded experiment on base commit `5af9c4067d12d1f7a0ee05af14b84af4b321f68c`.
No commits, agents, model calls, services, downloads, credentials, hidden data,
scorer changes, or edits outside the assigned ownership. All owned jobs completed.
Parent integrated release/regressions and review remain pending.

## Exact change

Adds `crates/quantities/src/percent_change.rs`, registers it in `lib.rs`, and
calls it from the existing parser after scalar linear relations and before
percentage-of relations. Adds `tests/percent_change.rs` (three test groups).
The new module uses typed Direction/Change/Stages and existing checked Rational.
Each directed relation is after=before*(1+signed_percent/100); explicit new-price
references compose factors and inverse queries divide by the complete factor.
No runtime data, Python, labels, lookup table, model or new dependency.

Supported productions are fully anchored: one numeric forward/inverse change;
two explicit successive changes to a number/price; original-price/new-price
composition; annual population forward/inverse with town/village/city binding.
Arbitrary single-word item/place names and independently varied values are
supported. Exactly two distinct sequential changes are implemented (within the
plan's upper bound16); annual repetition is capped100. No generic multi-entity
algebra, unit-bearing change quantities, unknown rate products, percentage
points, or unspecified-direction net-change questions. Negative quantities and
magnitudes refuse; declines above100% refuse. Zero absolute bases work, inverse
zero factors refuse. Population outputs must be integral. Existing affine
machinery remains unchanged because its private solver is limited to two
variables; the existing exact Rational machinery is reused for directed stages.

Assumptions: ordinary annual population rate compounds on each year's population
without rounding; percentages of an unspecified original price require a positive
base. Unsupported/contradictory clauses and conflicting references refuse.
The grammar is narrow; these results do not establish broad word-problem ability.

## Fixed gates and independent audit

| Evaluation | Baseline correct/refused/wrong/unscorable | Candidate |
| --- | --- | --- |
| Fixed first100 DEV | 1 /99 /0 /0 | 1 /99 /0 /0 |
| Full4475 DEV exact-label-v3 | 22 /4450 /3 /0 | 29 /4443 /3 /0 |
| Full4475 unchanged legacy metric | 22 /4450 /3 /0 | 29 /4443 /3 /0 |

Transitions:22 correct->correct,3 wrong->wrong,4443 refused->refused,
7 refused->correct. All25 original answer payloads (including method/value/unit)
are identical; all22 prior correct retained. Sparse baseline production.json has
only25 answers; missing indices imply refusal, not a recoverable refusal reason.
Its full question-order hash and supplied identity were verified.

New answers independently derived from English in `semantic-audit.json`:
265:8000*(.7)^2=3920;1482:3744/(1.2)^2=2600;1751:2000*1.2=2400;
2854:70*1.5=105;3805:15000*(1.1)^2=18150;
4013:(1.2*1.2-1)*100=44%;4071:(1.25*1.3-1)*100=62.5%.
All seven mathematically justified under the stated annual-base assumptions;
no semantic/label discrepancies found. Two are explicit new-price composition,
four annual population, one scalar increase. Existing three wrong answers remain.

All3116 synthetic status/value/unit triples unchanged (canonical SHA256
`93b2bd1c9f8822af95a68d02663a90d122d24ca57d730c61c670e2b11079bbe9`):
3056 right,60 refused,0 wrong, including762 expected refusals.
Existing authored linear50 and absolute55 pass unchanged. New authored38 probes
improve16->38 passing; independent reversals, equal gains/losses, zero/negative
and invalid domains, reference conflicts, extra clauses, percentage points,
unknown products/units, and existing percentage-of behavior are covered.

Final scoped fmt/check, clippy all-targets, quantities tests (12 total), quantities
build and main CLI debug build passed in `validation-final2.log`. Every cargo
invocation explicitly used disk TMPDIR, supplied CARGO_TARGET_DIR, scratch under
ownrun, systemd8GiB/0swap scope, setsid --wait, timeout600/kill-after10s, -j1
(except fmt), test threads4. Outer scope also verified8GiB/0swap.
No workspace-wide suite or integrated release validation claimed.

## Freeze, iterations and retained evidence

PLAN.md and PLAN.sha256 preceded runtime edits and DEV inference. Design inspected
only Problem text from first4000 TRAIN rows;160 matching texts retained.
FREEZE.json pins source/binaries/probes/adapter before first100 then full4475.
Complete predictions were written before evaluator-side label loading/scoring.
The immutable typed-label-metric FREEZE and legacy scorer hash were verified.
There were no rejected capability candidates and no post-full runtime edits or
retuning. Pre-freeze clippy rejected explicit panic in the new test helper; this
was replaced with assert and all checks rerun. The initial check ran before the
new tests were added; final validation includes them. PROBES.sha256 records the
initial test-file hash; PROBES-final.sha256 and FREEZE.json record the final
pre-DEV test source after rustfmt/lint correction.

Baseline quantities SHA256:
`06a651b1817f022677f1c898ba8f71bb53eb2d6ed7ad5c42c707b356a869768a`.
Candidate patch:12732bytes, SHA256
`53578ccc5629ea02bfd366d973413420640fc6d19564abf966e029e42b0df7bd`.
Every individual runtime source hash is in FREEZE.json.
Retained binaries under `../percentchange-run/candidate/`:
- glassmind-quantities:25275728bytes, SHA256 `e94f0ba1b05ce12aee1dfd015815af0f6825bf0f3049c0501fb2a5eb2cfacf15`.
- glassmind:126845800bytes, SHA256 `6e1de5323493f759231525cd5347eb0324a851f469a300d3c3de89970af6f9a1`.
Full4475 predictions:780849bytes, SHA256
`98447f585880c5600b76b6bd949c4d7bfbb1f0754825ce2ee3404ef745813ebc`.

## Data provenance and reproduction

No new downloaded bytes. Reused AllenAI MathQA revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0 per supplied source card:
TRAIN9013733bytes, SHA256`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`;
DEV1350141bytes, SHA256`488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
Derived160 TRAIN texts39443bytes, same license; authored38 probe input rows5295bytes,
version percent-change-public-development-v1, MIT OR Apache-2.0.
Full hashes/source URLs/card identity in data-provenance.json and source.json.
Only Problem strings entered Rust; DEV options/correct stayed evaluator-side.
No test split was read. The metric is exact-label-v3, not official multiple-choice
accuracy; its bare scalar labels ignore prediction dimension as documented by
its unchanged definition. No label normalization or evaluation definition changed.

From this workspace, reproduce all worker checks and fresh sample/full inference:

```bash
bash tools/quantities/percent-change/reproduce.sh /home/gsc/.cache/glassmind/continuous-20261010/percentchange-run/reproduction
```

Use a fresh output directory (completed predictions cannot be overwritten).
This consolidated wrapper was written after executing its component commands;
the wrapper itself was not rerun. To rescore retained predictions without any
new inference:

```bash
PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps python3 tools/quantities/percent-change/evaluate.py --binary /home/gsc/.cache/glassmind/continuous-20261010/percentchange-run/candidate/glassmind-quantities --data /home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/mathqa --output tools/quantities/percent-change/full --score-only
python3 tools/quantities/percent-change/audit.py
```

The candidate remains applied for coordinator review/integration; no commit made.
