# Offline MathQA metric audit — bounded batch complete; v2 NOT approved

Source checkout: `18b334ed81c59954b200a8de6138d8aa5ccad78e`.
Only NEW `tools/quantities/metric-v2/` files were written. No commit, Rust change,
solver import, inference, runtime deployment, candidate acceptance, existing scorer
edit, dataset edit, historical score edit, download, new dependency, agent, LLM
call, service or background job. Historical ratewords rejection remains REJECT.

**Decision:** useful label-format findings, but do NOT promote this frozen v2 as
an evaluator fix. Initial six authored test groups pass; two additional post-freeze
context probes fail. Rules and scores have not been tuned after DEV inspection.
The complete exploratory series below is a comparability break, not a runtime gain.
An independently reviewed request-clause parser and new prospective preregistration
are required before a production metric/rebaseline. This batch does not do that.

## Definition and evidence chronology

Read the requested prompt/checkpoint and current legacy adapter. Read TRAIN parquet
with only `Problem`, `options`, `correct` selected. TRAIN examples showed standard
unit suffixes and stringified option lists; hand calculations used TRAIN3/50/220/324.
`DEFINITION.md` states hypothesis, acceptance criterion, grammar, scope and rejected
approaches before DEV. `metric.py`, that definition and `test_metric.py` were hashed
in `FREEZE.json` at 2026-10-11 06:13:46 UTC before DEV options/frozen answers were
read. Initial tests passed. Then first100 replay preceded full4475 replay. No rule
changes followed. `review.py` adds diagnostics, not parsing rules or new scores.

Exact signed integer/decimal/rational/mixed notation uses Fraction, proper comma
groups and full string consumption. Negative mixed fractions mean -(whole+fraction).
Option binding requires exactly ordered a–e entries and the declared correct key;
stringified five-string lists are decoded with bounded `ast.literal_eval`, never
executed. Scalars <=64 chars; selected option <=256; question/options <=8192.
No formulas, arbitrary digit extraction, answer matching, word-number execution,
rounding or choice input into a solver.

Bare numbers retain the declared legacy requested-unit contract; this does not
prove their dimensions. Standard suffixes need an explicit `how many UNIT` or
`in UNIT` request; exact conversion uses gold scale / requested scale. Unsupported
units/contexts reject. Percent values stay percentage points under the explicit
percent-request rule; never divide by 100. The latter rule and punctuated ambiguity
handling are too weak, as the post-freeze failures below demonstrate.

Legacy unit exclusion is legitimate exact-numeric-subset scope. Treating `30 sec`
as unsupported was consistent with that frozen definition. In contrast, failing
to decode list serialization and concatenating mixed-number digits are genuine
parser deficiencies. Those findings still require versioning and rebaselining.

## Full fixed-denominator results (diagnostic only)

| Series | Correct | Refused | Wrong | Unscorable answered | Total |
|---|---:|---:|---:|---:|---:|
| Historical baseline | 22 | 4450 | 3 | legacy bucket absent | 4475 |
| Historical rejected candidate | 23 | 4444 | 8 | legacy bucket absent | 4475 |
| Frozen v2 baseline | 22 | 4450 | 2 | 1 | 4475 |
| Frozen v2 candidate | 28 | 4444 | 2 | 1 | 4475 |

Unsupported gold + refusal stays refused; unsupported gold + answer is explicitly
unscorable, never correct. No rows leave the denominator. If collapsing unscorable
back into wrong, v2 is 22/4450/3 and 28/4444/3; these remain NEW metric results.
First100 sample: both 1 correct,99 refused,0 wrong,0 unscorable; 34 golds unsupported.

Full gold parse transitions: 3009 parseable→parseable; 1090 unsupported→unsupported;
139 unsupported→parseable; 237 parseable→unsupported. V2 has 3148 parseable and
1327 unsupported golds. Historical 1229 remains unchanged. This is not optimization
of the unsupported count. Eight retained parseable mixed numbers change value;
none is answered by either frozen report. New rejections: 118 missing percent
requests,65 malformed option bindings,51 malformed numbers,3 malformed percents.
Example rejected bindings repeat labels (`a ) a ) ...`); no repair is guessed.

V2 baseline→candidate transitions: 4444 refused→refused,22 correct→correct,
6 refused→correct,2 wrong→wrong,1 unscorable→unscorable. No correct losses or new
wrong answers in these frozen outputs. Legacy→v2 baseline: only index3795 moves
wrong→unscorable. Candidate additionally moves five wrong→correct (below).
All 23 legacy candidate corrects remain correct. All original report bytes survive.

## Every legacy-unparseable gold accounted for

The first table uses mutually exclusive parsing reasons (all1229); the second,
more descriptive classification is in `full/review.json`.

| v2 disposition/reason | Count |
|---|---:|
| Bare scalar now parseable | 59 |
| Explicit unit now parseable | 78 |
| Percentage points now parseable | 2 |
| Recognized suffix, missing explicit requested unit | 294 |
| Unsupported suffix/expression | 419 |
| Unsupported text/expression | 335 |
| Malformed numeric | 24 |
| Non-scalar choice text | 10 |
| Ambiguous requested unit | 2 |
| Malformed percent | 3 |
| Missing explicit percent request | 2 |
| Malformed option binding | 1 |

Descriptive breakdown: 139 now parseable;294 missing-unit-context;155 colon ratio/
time;29 symbolic/compound;288 currency prefixes;224 other suffix/expression;
23 other malformed numeric;27 powers/area/volume;32 other text;10 none/neither;
2 ambiguous contexts;3 malformed percent;2 missing percent context;1 binding.
Categories classify text, not mathematical solvability. No unsupported expression
is evaluated. `full/gold-audit.jsonl` has exactly4475 ordered records, each with
question hash, correct key, selected text (null on rejected container), legacy
value, v2 value, reason, units and both grades. `full/review.json` gives an exact
normalization explanation for ALL139 newly parseable rows and all8 changed values.
These machine explanations are normalization traces, not independent word-problem
proofs. The five grade improvements have the independent derivations below.

## Independent derivations for every changed answer grade

All row indices are zero-based. All five golds are the correctly bound numeric
value followed by `sec`; every question explicitly asks `how many seconds`.
One sec = one requested second. Prediction unit `seconds` also has scale1.
No answer tolerance or inferred unit is involved.

| Candidate index | Selected gold | Exact independent English calculation |
|---|---|---|
| 563 | 30 sec | (150+150)m / (36×1000/3600 m/s) =30s |
| 1128 | 6 sec | (60+120)m / (30 m/s) =6s |
| 1986 | 27 sec | (100+170)m / (36×1000/3600 m/s) =27s |
| 2836 | 29 sec | (140+150)m / (36×1000/3600 m/s) =29s |
| 2873 | 27 sec | (120+150)m / (36×1000/3600 m/s) =27s |

Full traversal requires train length + bridge length. These arithmetic derivations
agree with the provided English-only audit but do not use it as a gold lookup.
The sixth new answer, index295, was already correct under legacy: (100+140)/10=24s.

Index3795 changes wrong→unscorable in BOTH reports. Gold `480 km`; request says
`into km`, outside the frozen grammar (`in UNIT`). We did not add `into` after seeing
DEV. Independent exact conversion is 300×1.609344=482.8032 km =301752/625 km,
not480 km. Thus this is an unsupported bucket change, not a correct-answer gain;
a future parser accepting `into` would still grade this output wrong against gold.
The other two wrongs remain: index454 has gold26.67 versus exact80/3 percent;
index4045 has gold536 versus exact1340067/2500. No rounding rescue.

The eight mixed-number changes (legacy→v2):1235 231/13→300/13;
1911 31/3→10/3;2572 331/3→100/3;2719 41/5→21/5;
3164 311/13→50/13;3232 213/7→150/7;3610 31/5→16/5;
3839 331/3→100/3. These are representation corrections only; all stay refused.

## Validation and rejection evidence

Six unittest groups PASS: exact signed/zero/decimal/mixed values; malformed numeric
rejection; exact unit conversions; unknown/mismatched/ambiguous context rejection;
option splitting/key integrity including thousands commas and list serialization;
four hand-computed TRAIN cases. A pre-freeze follow-up added compound-unit rejection
(`meters per second` must not be accepted as meters); passed before hashing.

After full replay, three additional authored context probes were recorded:
1. Background hours plus requested meters correctly rejects (PASS).
2. `The percentage of waste is known. What fraction remains?`, gold `25 %`,
   incorrectly accepts25 instead of rejecting (FAIL).
3. `How many hours, or days, does it take?`, gold `3 hours`, incorrectly accepts3
   instead of rejecting ambiguous units (FAIL).

Thus the parser fails the intended unambiguous-context acceptance requirement.
These are retained counterexamples, not silently fixed post-DEV. No v2 deployment
is recommended. Existing unit-case findings remain useful narrow audit evidence.
Additional limitations: bare scalar units are assumed by contract, not resolved;
parser conservatively rejects many legitimate English/unit forms; parsing a label
never verifies dataset mathematical correctness; rounded/noisy labels remain exact;
no independent test or runtime generalization evidence was obtained.

Frozen report validation asserts exactly25/31 answered records and unique sorted
indices, matching per-answer Problem strings, all legacy expected values/outcomes,
4475 total and 4450/4444 implied refusals, and all25 baseline records preserved in
candidate. Both reports' question hashes match the full official question JSONL SHA256:
`bf0c0a3d095234fe1b00707e0d0b748477225acf4de1cb633ed6095b66e425f2`.
The sparse reports do not preserve individual refusal payloads; only their full
coverage/count plus question hash can be verified. No re-inference was performed.

## Source/provenance and access limitations

Existing AllenAI MathQA, Apache-2.0 per retained card; dataset revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, card revision
`c4f1cc784c04c4957b50c97858f23893b633eea6`. Upstream homepage
https://math-qa.github.io/math-QA/ ; source URLs already retained in existing
`tools/quantities/reports/source.json`. No external network/downloads, new source
data0bytes, no installed dependencies. Reused TRAIN29837 rows/9,013,733 bytes,
validation4475 rows/1,350,141 bytes (10,363,874 existing parquet bytes total).
SHA256 TRAIN `c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`;
validation `488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
Frozen report sizes/hashes, binary identities, legacy adapter hash and freeze hashes
are in `full/summary.json`; owned evidence hashes/bytes are in `ARTIFACTS.json`.

Only public TRAIN/DEV parquet columns Problem/options/correct were read, plus
source metadata, current code/docs and explicitly supplied frozen reports/audit.
No official test, hidden-test code/generators or old private logs were accessed.
**Access deviation:** reading the existing source card for provenance unexpectedly
printed its embedded public TRAIN example including a rationale and annotated
formula. It was not used to design the parser or derive scores. No such parquet
columns were loaded. This incidental exposure is disclosed rather than claiming
zero rationale text was ever visible.

No credentials/API keys were read or used. No new model calls were made; this
worker cannot change or independently attest its parent-selected model/tier.
Outer scope verified MemoryMax8589934592, MemorySwapMax0. Python only; no cargo,
no /tmp storage, no persistent child jobs. Initial pyarrow lookup used the wrong
sibling path and failed before any data load; corrected to quantities-run/tmp/
quantities-deps. No content outside ownership was written.

## Reproduction and exact new files

From checkout root:

```sh
bash tools/quantities/metric-v2/reproduce.sh tools/quantities/metric-v2/reproduced
```

This runs six unittest groups, fixed first100, full4475, then post-freeze diagnostic
review (two failures retained as JSON). Override MATHQA_DATA, MATHQA_FROZEN,
MATHQA_DEPS for relocated retained inputs. No binaries, network or credentials.
`audit.py` verifies the frozen parser/definition/test hashes on every invocation.

New code: `metric.py` isolated experimental parser, `audit.py` frozen-output replay,
`review.py` normalization/failure diagnostics, `test_metric.py`, `reproduce.sh`.
New evidence: `DEFINITION.md`, `FREEZE.json`, this handoff, `ARTIFACTS.json`, and
sample/full JSON summaries, complete row ledgers, changed-grade records; full
also has `review.json`. Rejected directions (before DEV): suffix stripping,
implicit unit inference, formulas, tolerance, closest-option scoring. Rejected
experiment (after DEV): promote this v2 grammar; failed context probes prohibit it.
Coordinator may review the evidence but must keep legacy history and candidate
rejection intact. Further implementation/rebaseline is intentionally incomplete.
