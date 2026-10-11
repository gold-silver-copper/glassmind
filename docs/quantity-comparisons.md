# Exact quantity comparisons

Ask `glassmind talk` directly:

```text
Is 2 kg greater than 1500 g?
Which is longer, 3 feet or 1 metre?
Is 32 fahrenheit equal to 0 celsius?
```

The first answer is yes, with an exact witness of 2 and 3/2 kilograms. The second
selects the metre; the third reports equality. `Compare quantities: QUESTION`
explicitly selects the same operation. JSON replies have
`kind: "quantity_comparison"`, `status: "comparison"` or `"refusal"`, and a human
explanation. Successful results contain both operands, dimension, ordering,
predicate/truth or selection/selected operand, and an exact conversion witness.
False predicates and equal selections are successful mathematical results.

Supported predicates are greater than, less than, equal to, at least and at most.
Selections are longer/shorter, heavier/lighter, faster/slower and hotter/colder;
the adjective must match the units' dimension. Each operand needs its own
explicit known unit and an integer, decimal or fraction. Existing length, mass,
duration, volume, speed and temperature units are reused, including affine
temperature conversions. Values are literal mathematical magnitudes. There is
no extra physical-validity rule for negative mass or temperature.

Case and whitespace are normalized and one final question mark is optional.
Unknown or ambiguous units, incompatible dimensions, arithmetic expressions,
pronouns, appended assertions and malformed quantities refuse. Questions are
limited to 4096 UTF-8 bytes and cannot contain NUL. Checked integer overflow
refuses instead of rounding. This includes possible intermediate overflow even
when a mathematical final result would be representable.

Comparisons preserve personal memory, story state and provisional request drafts.
They advance the ordinary turn counter and last-message field and clear personal
correction focus. They do not infer operands from those other contexts. Explicit
comparison commands and recognized natural comparison stems reserve the route
even when malformed, so appended statements cannot modify another domain.

The separate quantities executable accepts JSON-string questions through
`glassmind-quantities --compare-jsonl`. Existing `--jsonl` and scalar `solve` are
unchanged. Rust callers can use `comparison::compare_question`, or construct a
`ComparisonQuery` and call `evaluate`; direct queries validate dimensions too.

The independent parent suite checks 24 exact comparisons and 17 refusals with a
separate Fraction/SI oracle, including all six dimensions, false and equal
results, negative magnitudes, fractions and affine conversions. All 24 direct
API results agree with parsed results. Fresh native scalar runs reproduce the
200-question sample, full 4475-question DEV and 3116 synthetic outputs byte for
byte. These are authored capability and compatibility checks, not a new MathQA
accuracy claim. See [worker evidence](../tools/quantities/unit-comparison/HANDOFF.md)
and the parent artifacts retained under
`~/.cache/glassmind/continuous-20261010/coordinator/unitcompare-independent`.
