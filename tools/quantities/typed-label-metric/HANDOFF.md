# Exact-label comparison v3 — bounded offline batch complete

Decision: authored evaluator checks PASS; submit for independent parent review,
NOT production approval, rebaseline or candidate acceptance. Frozen v2 and
historical ratewords remain REJECTED. No official MathQA metric claim. New v3
numbers are a comparability break and cannot retroactively rescue a candidate.

Source checkout: `80b9f636050accfbe6ca0fbdb0ecc2bc6ca531d9`.
Ownership: only NEW `tools/quantities/typed-label-metric/` files written.
Preexisting untracked `tools/quantities/metric-v2/` preserved unchanged. No commits,
Rust/runtime/legacy scorer/shared manifest/README/CLI/loop/chart changes, inference,
model calls, downloads, training, agents, services or surviving background jobs.

## Contract and chronology

Hypothesis/acceptance were stated before implementation. Inspected only authorized
TRAIN columns (Problem/options/correct); label examples/provenance are recorded in
DEFINITION.md. No question text entered normalization. Read rejected v2
DEFINITION/HANDOFF/parent-review as diagnostic evidence; did not tune v2.
Prior baseline 22 correct/4450 refused/3 wrong and rejected candidate23/4444/8
were recorded before editing. No candidate score criterion was used.

Implemented exact integer/decimal/rational/proper mixed literals, proper grouping,
full consumption, bounded five-string list decoding and unique ordered a-e binding.
Uses ast parsing/structural checks only, never evaluation; rejects concatenated
string literals, nonliteral elements and embedded labels. Explicit dictionary
units have exact scales/dimensions. No request parser, expressions, approximation,
implicit units, arbitrary digit extraction, suffix guessing or answer proximity.

Case 1, bare scalar: actual coefficient comparison only. Each result explicitly
flags that prediction dimension and question/requested units are not evaluated.
Case 2, explicit unit: compare explicit supported prediction unit's exact canonical
value AND dimension. Missing/unknown prediction units unscorable, incompatible
known dimensions wrong. Explicit percent is percentage_points, scale1; dimensionless
predictions cannot satisfy explicit-percent gold. Bare labels deliberately retain
legacy coefficient-only limits, even for a prediction with percent/another unit.

15 authored unittest groups passed before freeze, including deterministic property
loops across signed rationals, mixed forms, grouping, literal bounds, all aliases,
reciprocal conversions, round trips and dimension mismatches across time/length/
mass/speed/percent. Independent conversion constants derive inch=254/10000m,
foot=12in, mile=5280ft, km/h=1000/3600m/s, mph=5280*12*254/(10000*3600)m/s.
Option/list rejection, unknown text, missing predicted units and question
independence are explicit positive/negative tests. Numeric coefficient parsing is
also required for answers: malformed coefficients are unscorable.

FREEZE.json timestamp `2026-10-11T06:21:37.240556+00:00` precedes DEV options and
frozen answer-content access. Definition, evaluator, tests, replay, reproduction
script and input hash pins are frozen. First100 replay preceded full4475.
All15 tests passed again postfreeze; no failures or frozen-code edits occurred.
Postfreeze independent_review.py is a separate arithmetic evidence audit, not
imported by the evaluator and does not modify its decisions. It imports neither
metric.py nor any solver. It uses integer pairs, Decimal digit decomposition and
cross multiplication to independently verify3631 accepted gold normalizations,
all56 frozen answered records, all11 answered normalization changes and all5
grade changes. This is independent arithmetic in the same worker; separate parent
review is still outstanding.

Rejected approaches: global question patterns (v2's failed background-unit and
punctuated-alternative probes), inferring predicted units from gold, universal
percent/100 conversion, arbitrary suffix stripping, symbolic expression solving,
closest-answer selection, tolerance and score-based evaluator acceptance.
No unsuccessful v3 test run was concealed; there were no failed v3 experiments.

## Fixed-denominator results

| Series | Correct | Refused | Wrong | Unscorable answered | Total |
|---|---:|---:|---:|---:|---:|
| Original legacy baseline | 22 | 4450 | 3 | absent | 4475 |
| Original legacy rejected candidate | 23 | 4444 | 8 | absent | 4475 |
| New v3 frozen baseline | 22 | 4450 | 3 | 0 | 4475 |
| New v3 frozen rejected candidate | 28 | 4444 | 3 | 0 | 4475 |

Conservative unscorable-as-wrong: v3 baseline22/4450/3, candidate28/4444/3.
First100: both1 correct/99 refused/0 wrong/0 unscorable; gold cases62 bare,
17 explicit-unit,21 unsupported. No score or rule was tuned from the sample.

| Gold case | Rows | Baseline correct/refused/wrong/unscorable | Candidate correct/refused/wrong/unscorable |
|---|---:|---|---|
| Bare scalar, coefficient only | 2828 | 21/2806/1/0 | 22/2805/1/0 |
| Explicit unit, typed label equality | 803 | 1/800/2/0 | 6/795/2/0 |
| Unsupported gold | 844 | 0/844/0/0 | 0/844/0/0 |

803 explicit labels:210 time,156 length,36 mass,39 speed,362 percentage_points.
Unsupported labels remain in the denominator; all844 happen to be refused in both
reports. Unsupported answered labels would be unscorable, covered by tests.

Gold parse transitions:2769 old scalars→bare,358 old scalars→explicit,
119 old scalars→unsupported;59 old unsupported→bare,445→explicit,725→unsupported.
All1229 historical unsupported golds accounted for:59 exact scalar,445 exact unit,
724 unsupported/malformed label,1 binding rejection. New119 rejections:65 binding,
54 unsupported/malformed. Total reasons:2828 scalar,803 explicit,66 binding,
778 unsupported/malformed. Rejection conservatism is deliberate, not semantic proof.

Complete4475 ledger records ordered index, question hash, correct key, selected
text, legacy value, typed normalization, reason, frozen answers or implied refusals,
old/new grades, flags and comparison reasons. A separate984-row changes ledger
includes every new parse/rejection, typed-unit change (even equal coefficients),
and changed bare coefficient. It does not exclude losses or unsupported rows.
Nine mixed coefficients differ from legacy, all remain refused:
479:54→50/3;1235:231/13→300/13;1911:31/3→10/3;2572:331/3→100/3;
2719:41/5→21/5;3164:311/13→50/13;3232:213/7→150/7;
3610:31/5→16/5;3839:331/3→100/3.

## Every grade change and remaining wrong label

Baseline legacy→v3: no grade changes. Candidate: exactly5 wrong→correct;
all23 old correct labels retained. Zero losses, zero new wrongs, zero unscorable
transitions. Baseline→candidate under v3:22 correct unchanged,3 wrong unchanged,
4444 refusals unchanged,6 refusal→correct. This is descriptive frozen-output
comparison, not prospective capability acceptance.

Every changed grade has a direct independent label derivation:

| DEV index | Selected gold | Frozen prediction | Independent exact normalization |
|---|---|---|---|
| 563 | 30 sec | 30 seconds | 30*1=30*1 seconds |
| 1128 | 6 sec | 6 seconds | 6*1=6*1 seconds |
| 1986 | 27 sec | 27 seconds | 27*1=27*1 seconds |
| 2836 | 29 sec | 29 seconds | 29*1=29*1 seconds |
| 2873 | 27 sec | 27 seconds | 27*1=27*1 seconds |

No question semantics or arithmetic word-problem solution is claimed. All are
explicit time labels, with exact scale1 on both sides. Original legacy adapter
left the suffixed gold unsupported. Index295's gold is bare24; its frozen24 seconds
passes only the coefficient contract, never dimensional correctness.

All other answered normalization changes also checked:454 gold2667/100 percentage
points differs from80/3;1587 gold18 percentage points equals18 percent;
3795 gold480km canonical480000m differs from301752/625km canonical2414016/5m.
The third persistent wrong is bare536 vs1340067/2500. No rounding rescue.
These derivations, the full56 answered checks and nine changed mixed coefficients
are in reports/independent-review.json. All3631 independently recomputed accepted
normalizations are in reports/independent-normalization-derivations.jsonl.

## Integrity and source provenance

Both frozen reports are pinned by bytes/hash before content replay and preserved
verbatim in sample/full originals. Recomputed all legacy expected values and grades,
original22/4450/3 and23/4444/8,1229 old unparseables, exact25/31 answered lengths,
unique/sorted/in-range integer indices, actual answer status, question text equality,
full question hash and denominator, and identical25 baseline records in candidate.
Each ledger has the complete required index range in source order. No inferred
answer from a refusal or gold selection using actual value.

Full4475 question JSONL SHA256:
`bf0c0a3d095234fe1b00707e0d0b748477225acf4de1cb633ed6095b66e425f2`.
Original baseline7947 bytes:
`205da7cff1dafa306500ba47b50dbf4819d9889a3795f0911a6030f0f728af53`.
Original candidate10278 bytes:
`9264e0903e0f5c0cb41ad276d17c01254efd7bdb6d1e2e4c4d2d9dcfbf76826d`.
Unchanged legacy adapter3141 bytes:
`226f416ddff557bb752a41ae1e799235b0bcab72d2d07287ece12630db48f69d`.
Freeze hashes in FREEZE.json; exact evidence hashes/bytes in ARTIFACTS.json;
original binary identities in reports/full/summary.json. Binaries not executed.

New downloaded/source data:0 bytes; no new dependencies. Reused existing AllenAI
MathQA Apache-2.0 per retained source metadata, dataset revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, card revision
`c4f1cc784c04c4957b50c97858f23893b633eea6`, homepage
https://math-qa.github.io/math-QA/ . License metadata read only from existing
`tools/quantities/reports/source.json`; source-card contents never opened.
TRAIN29837 rows/9013733 bytes SHA256
`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`;
DEV4475 rows/1350141 bytes SHA256
`488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
Reused total10363874 parquet bytes. Only Problem/options/correct columns decoded;
file hashing reads raw file bytes for integrity without decoding other columns.
No official test/heldout, hidden-test code/generators, rationale/formula columns,
source-card examples, private historical logs, credentials or API keys read.
No new model calls; parent-selected model/tier cannot be independently attested here.
Outer scope verified MemoryMax8589934592/MemorySwapMax0. Python only, no cargo,
no /tmp storage, no persistent jobs. Readonly input mounts unchanged.

## Reproduction

From this checkout root (Python3.10+; retained pyarrow23.0.1):

```sh
PYTHONDONTWRITEBYTECODE=1 python tools/quantities/typed-label-metric/reproduce.py \
  --output tools/quantities/typed-label-metric/reproduced
PYTHONDONTWRITEBYTECODE=1 python tools/quantities/typed-label-metric/independent_review.py \
  --reports tools/quantities/typed-label-metric/reproduced/full \
  --output tools/quantities/typed-label-metric/reproduced/independent-review.json
```

Defaults use sibling quantities-run/tmp/mathqa and quantities-run/tmp/quantities-deps,
and typedmetric-run/baseline. Override --data, --deps, --frozen to relocate inputs.
The first command reruns15 tests then first100 then full4475; replay verifies frozen
hashes each time. Both commands were executed successfully against reports/.
No cargo needed; do not invoke unscoped cargo during review.

Exact added code: metric.py, test_metric.py, replay.py, reproduce.py,
independent_review.py. Definitions/pins: DEFINITION.md, INPUTS.json, FREEZE.json.
Evidence: pre-freeze-tests.txt, reports/sample and reports/full summaries, complete
ledgers, changes ledgers, changed-grade files and verbatim originals; independent
review/normalization records, this handoff and ARTIFACTS.json.

## Limits and remaining work

Bare-label accuracy ignores prediction dimension, as legacy does. Physical-label
equality does not verify dataset/question semantics. Abbreviations mean exactly
the dictionary entry (e.g. m=meters); no request disambiguation is attempted.
Closed grammar rejects many meaningful labels. Exact labels can be rounded/noisy.
Sparse reports preserve no individual refusal payloads; only full question hash,
answered indices and aggregate counts establish implied refusal coverage.
All validation is TRAIN/DEV, no independent heldout performance claim. Property
checks are finite; arithmetic review is not a separate reviewer. Parent must
independently review this evaluator, commit/rebaseline separately, and predeclare
future capability acceptance. No prospective runtime work or candidate acceptance
was attempted or is authorized by these results.
