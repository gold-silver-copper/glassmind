# Frozen before production edits

Base cddb06558a3c580392a7b678187cbb35cb2b6709. Scope: crates/quantities and this directory only. No scalar grammar, main conversation CLI, models, fixtures, scorers, external data or session state changes.

Hypothesis: exact Unit::convert plus dimension-checked Rational ordering and fully consumed literal grammar adds comparison coverage with no authored wrong answers and zero legacy byte changes. Existing scalar solver refuses the three motivating questions. Preserve baseline responses and original source before implementation.

Grammar (case insensitive, whitespace runs normalize to one space):
`Is Q (greater than|less than|equal to|at least|at most) Q [?]`
`Which is (longer|shorter|heavier|lighter|faster|slower|hotter|colder), Q or Q [?]`
Q: a signed integer, decimal (including .5), or fraction with signed integer numerator/denominator; whitespace then an existing Unit name. No exponent, number words, compound arithmetic, implicit unit carry or pronoun binding. Optional single terminal question mark, leading/trailing whitespace. All tokens consumed. Unit::parse is the sole unit authority; normalize whitespace in multiword names. Raw input <=4096 UTF-8 bytes, reject NUL. Number token <=100 bytes. Bounded linear scan; no regex backtracking.

API: private-field validated ComparisonQuantity exact Rational + Unit + normalized name; public ComparisonQuery {left,right,operation} with typed Predicate/Selection enums. parse_comparison -> Result<ComparisonQuery, ComparisonError>; query.evaluate -> Result<ComparisonResult, ComparisonError>; compare_question composes these. Evaluation validates dimensions and adjective even for directly built queries. Result exposes both operands, dimension, exact Ordering enum, Predicate+bool or Selection+Left/Right/Equal, and witness {unit,left,right} exact rationals. Use left unit as witness, converting BOTH through existing Unit::convert. Reusable Rational::checked_cmp; checked arithmetic may refuse overflow, never f64. Rational JSON strings preserve exactness. Errors distinguish invalid input, grammar, number/unit, mismatched dimensions/adjective, overflow. Invalid queries cannot masquerade as false/equal. Scalar Answer remains untouched.

Native --compare-jsonl: JSON strings in; status comparison with typed result, or status refusal with typed error. Separate from unchanged --jsonl branch. Bound buffered transport line to 6*4096+2 bytes (+CR allowed), drain oversized lines, resume next line. Malformed JSON refuses. No state mutation. Direct native example accepts JSON quantities/operation and calls typed API for independent review.

Frozen authored.json contains independent expected positive/equality/false/all dimensions/all adjectives/affine/negative/fraction/case-space and negative-control cases. Unit witness magnitudes authored before implementation. Public authored scope only, no MathQA accuracy claim. Additional native algebraic/inverse/symmetric/overflow and boundary tests will verify invariants.

Acceptance: every frozen authored expectation correct, no authored wrong results; direct/parsed results identical; all relevant crate tests, package fmt/all-target clippy/release pass; fixed first200 ordinary solver output byte-identical BEFORE full4475 and authoredsynthetic3116 byte parity. Preserve all failures and rejected experiments. New coverage is typed literal reasoning only. Freeze final source/diff/binaries/reports/hashes in RUN/frozen-candidate; no commit.
