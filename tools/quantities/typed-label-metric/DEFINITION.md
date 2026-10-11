# Exact-label comparison v3: frozen question-independent contract

Hypothesis (before DEV): selected-label syntax supports a defensible exact-label
comparison with explicit uncertainty and no request-language interpretation.
Acceptance is zero authored adversarial/property failures, independently checked
exact normalization derivations, intact source/report provenance and all 4475
rows accounted for. Neither a solver score increase nor a parseability increase
is an acceptance criterion. Parent independent review/rebaseline remains required.
If a postfreeze test fails or the contract is unsound, retain evidence and REJECT;
never patch this version after DEV. Ratewords and metric-v2 remain REJECTED.

## Scope and two cases

Gold normalization receives ONLY options and correct key, then ONLY the selected
text. Comparison receives that parsed label and a frozen structured answer record.
There is no question parameter, question lookup, runtime inference or choice of
gold using prediction value/proximity. Problem is used only for provenance/order.

1. **bare_scalar**: compare the actual numeric coefficient exactly, ignoring its
   unit field. This preserves legacy scalar-value comparison, subject to the new
   literal grammar. Set `question_requested_units_evaluated=false` and
   `prediction_dimension_evaluated=false`. Bare-label accuracy ignores prediction
   dimension, as legacy does. It is neither dimensional nor mathematical correctness.
2. **explicit_unit**: both gold and prediction must explicitly name supported
   units. Compare exact coefficient*scale AND dimension. Missing/unsupported
   prediction unit -> unscorable; supported incompatible dimension -> wrong,
   even at zero. Compatible unequal canonical values -> wrong. Never guess a
   prediction unit from gold. Physical-label equality does not verify dataset or
   question semantics. Question/requested units are never evaluated here either.

Explicit percent has dimension `percentage_points`, scale 1: 25% means 25
percentage points, never 1/4. Gold percent requires explicit percent prediction.
Bare gold 25 can equal coefficient 25 with ANY predicted unit, including percent,
solely under case 1. This asymmetric scalar contract is intentional and flagged;
it does not make percent and dimensionless quantities interchangeable in case 2.

Unsupported/malformed gold + answered record -> unscorable; any refusal -> refused.
Unsupported/malformed prediction coefficient -> unscorable. All rows remain;
report correct/refused/wrong/unscorable and conservative correct/refused/
(wrong+unscorable). Also report gold case/reason counts and case-specific outcomes.
No official MathQA metric claim: v3 is a new series and comparability break.

## Closed grammar and bounds

Trim only ASCII spaces/tabs/newlines/CR at the outer label/container boundary;
inside numeric literals use spaces/tabs only. ASCII digits only, no Unicode sign
or digits. Selected text <=256 characters; numeric literal <=64 characters;
options <=8192 characters. All numeric forms require full consumption:

- integer: optional single +/-, ungrouped digits OR first group 1-3 digits followed
  by one or more comma groups of exactly 3 digits. Leading zeros allowed.
- decimal: signed integer part (same grouping) dot one-or-more digits, or leading
  dot and digits. No trailing dot or exponent.
- rational: signed integer numerator / unsigned integer positive denominator;
  integer grouping permitted; spaces/tabs around slash permitted. Improper
  fractions allowed; zero/negative denominator rejected.
- proper mixed number: optional sign, unsigned integer whole, whitespace,
  unsigned integer numerator / unsigned positive denominator with 0<N<D.
  Whole may be zero; sign applies to entire sum. A zero fractional part must be
  written as an integer or rational, not mixed notation. Digits never concatenate.

A suffix may directly follow a scalar (30sec) or follow ASCII whitespace. It must
be an exact case-insensitive ASCII alias in metric.py UNITS, not a prefix or an
arbitrary noun. No punctuation stripping. Canonical dimensions/scales:
seconds 1, minutes 60, hours 3600, days 86400 (time);
mm 1/1000, cm 1/100, m 1, km 1000, inch 127/5000, foot 381/1250,
mile 201168/125 (length in meters);
gram 1/1000, kilogram 1 (mass in kilograms);
m/s 1, km/h 5/18, mph 1397/3125 (speed in meters/second);
percent/% 1 (percentage_points).
The complete fixed alias table is part of the frozen code; no inferred plurals,
products, powers, degrees, volume, currency, months, `units`, textual numbers,
colon ratios, formulas, arithmetic, symbolic guesses, tolerance or digit extraction.

Options are exactly five nonempty strings labeled a,b,c,d,e in that order,
with lowercase correct key exactly one of a-e. Raw comma-separated options split
only at comma + a-e label + `)`, retaining thousands commas. Alternatively decode
an <=8192-character Python/JSON-style stringified list with ast.parse, requiring
one List of exactly five string Constants, no other AST node kinds, each <=1024
characters. No evaluation/calls/nesting/tuples/dicts/concatenation repair. Actual
non-string containers reject. Each entry has one a-e label; embedded/repeated
label tokens reject conservatively. Exactly the selected option is normalized;
nonselected text is only container-validated, never used to infer gold.

Pseudocode:
  selected = bind_exactly_ordered_five(options, correct)
  gold = parse_full_scalar(selected) OR parse_full_scalar_and_exact_unit(selected)
  if refusal: refused
  else if invalid gold or invalid actual.value: unscorable
  else if gold bare: exact coefficient equality
  else if actual.unit missing/unknown: unscorable
  else if dimensions differ: wrong
  else: exact (gold.coefficient*gold.scale == actual.coefficient*actual.scale)

## TRAIN provenance and uncertainty before DEV

Only public TRAIN Problem/options/correct were loaded, with Problem unused.
29837 rows, 9013733 bytes, SHA256
c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99.
Selected labels: TRAIN3 `240 %`, 50 `20 minutes`, 63 list-bound `16`,
220 `18.75 days`, 324 `3.2 hr`, 10 `81 kg`, 37 `390 miles`, 175 `40 cm`,
44 `3.5 kmph`, 773 `4 4 / 5`. TRAIN15 `30 % .`, 75 `43 °`, 99 `20 π cm`,
202 `678.8 sq cm`, 268 `10 pi feet`, 319 `224 km .` remain unsupported by design.
Units/conversions also reflect standard exact definitions, not mined DEV values.
No source-card embedded examples, rationale/formula columns or held-out data read.

Baseline historical: 22 correct/4450 refused/3 wrong, 25 answers; rejected candidate
23/4444/8, 31 answers. Frozen v2 failed context probes; global `in UNIT` patterns,
background percentages and punctuated alternatives are rejected approaches.
Also rejected before implementation: question parsing, implicit unit inference,
closest-choice matching, suffix stripping, formula evaluation and tolerance.
Predeclared uncertainty: valid labels outside this closed grammar are unscorable;
ambiguous abbreviations have only the declared dictionary meaning; labels can be
noisy/rounded; matching them provides no proof of a solver's semantics. Sparse
reports have no per-refusal payload, only implied refusals/full-question hash.
