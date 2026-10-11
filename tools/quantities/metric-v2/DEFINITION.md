# Frozen metric v2 definition (before DEV options or frozen answers)

Hypothesis: a subset of legacy unparseable gold options has an unambiguous exact
scalar representation. A new offline metric can distinguish these from unsupported
expressions without consulting outputs. Acceptance: authored adversarial tests,
exact correct-key binding, complete 4475-row audit, both frozen reports validated,
and an independent explanation for every changed grade. No runtime gain or candidate
acceptance follows. Historical 1229 and ratewords rejection remain unchanged.

TRAIN-only observations: rows 3 (240 %), 50 (20 minutes), 63 (stringified option
list, 16), 75 (43 degrees symbol), 220 (18.75 days), 324 (3.2 hr) motivate rules.
Legacy explicitly defines an exact-numeric subset: exclusion of units is legitimate
scope, not proof of a scoring bug. List serialization mishandling and concatenating
whitespace in mixed fractions are parser deficiencies. Fixing either changes the
metric population and requires a separate version and rebaseline.

Rules frozen now:
* Options: exactly five unique a–e labels in order, either a comma-delimited string
  or a safely decoded list of five strings; bind only the declared correct key.
  Never select by proximity to a prediction. Reject malformed containers/labels.
* A scalar is a signed integer (optional proper thousands commas), exact decimal
  including leading-dot decimals, integer fraction with positive denominator, or
  signed mixed number W N/D with W>=0 and 0<=N<D. Negative mixed numbers mean
  -(W+N/D). Spaces around a slash are allowed; digits are never concatenated.
  No exponent, colon ratio, formula, arithmetic, word number, trailing period,
  multiple quantity, or none-of-these interpretation. Zero denominator rejects.
* Bound options to 8192 characters, selected text to 256, question to 8192,
  every numeric literal to at most 64 characters. No eval or formula execution.
* Bare scalars use the legacy contract: values are already in requested units.
  This is a declared assumption, not dimensional inference. Strict malformed
  grouping rejection and mixed-number interpretation intentionally differ.
* Recognized suffixes: seconds/minutes/hours/days, mm/cm/meters/km/inches/feet/miles,
  grams/kg, liters, degrees (including °), kmph/km/h/km/hr and mps/m/s, with explicit
  standard singular/plural and common abbreviations enumerated in code. Fixed
  exact conversion factors only; no currency, months, compound sums, generic
  'units', squared/cubed units or arbitrary noun suffixes.
* A suffixed gold requires explicit 'how many UNIT' or 'in UNIT' in Problem.
  All such recognized requests must agree on one unit. Unknown requested units,
  absent explicit requests, and dimension mismatch remain unscorable. No implicit
  choice of unit based on gold, candidate, or other quantities in the problem.
  Exact conversion: gold_value * gold_scale / requested_scale.
* A percent suffix requires an explicit percent request ('what percent', 'what
  percentage', 'percentage', 'gain percent', 'loss percent', 'profit percent',
  or 'percent increase/decrease'). Its value remains percentage points, never
  divided by 100. Bare percentage values retain the bare-scalar convention.
* Predictions are scalar Fractions under the frozen adapter's requested-unit
  contract. Any explicit prediction unit is validated against the requested unit.
  No rounding tolerance. Unsupported gold answered => unscorable; unsupported gold
  refused => refused. Report unscorable_gold separately. Four disjoint outcome
  buckets correct/refused/wrong/unscorable always sum to the full denominator.
* Audit categories describe ALL old unparseables (including all still unsupported)
  and ALL new rejections. Persist per-row reason, value, units, old/new parse status.
  Freeze first100 sample then full4475, with no rule changes after DEV inspection.

Rejected approaches before DEV: stripping arbitrary suffixes, inferring hours from
'how long', executing formulas, rounding to a candidate, accepting any substring
of digits, treating percent as a universal factor of 1/100. Unknown stays unknown.
