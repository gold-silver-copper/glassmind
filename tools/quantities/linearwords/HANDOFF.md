# Linear scalar English — bounded implementation handoff

Original worker candidate (superseded by coordinator semantic correction), based on source `298660591211d62da9ce8c02927c631828d49ace`.
Ownership: `crates/quantities` and new `tools/quantities/linearwords` only.
Coordinator found signed interpretation of English differences unsound; see
[PARENT.md](PARENT.md) for the counterexample and revised acceptance checks.
Original files are preserved in the worker submission archive.
No shared CLI, manifests, evaluator, README, historical report or prompt edits.

## Hypothesis, baseline and acceptance (stated before editing)

Problem-only training evidence showed fraction-of-number expressions, times,
sum/difference and comparative equalities. Selected examples and source metadata
are in `train-evidence.json` (zero-based official train indices). In particular,
402/6735 express a number minus its fraction; 4622/10856 express comparative
percentage amounts; 2272 shows multiple scalar constraints; 1519 motivates
compositional operations but its temporal wording remains unsupported.

Hypothesis: a fully consumed grammar for scalar expressions and equalities,
with explicit references and target, can compose these operations through one
exact constraint solver, including independently rearranged clauses.
Acceptance: at least three extra correct full-dev answers, no introduced semantic
errors or baseline correct losses, and every existing synthetic status/value/unit
unchanged. Fixed first100 dev for iteration, one candidate full4475 evaluation
after the implementation and independent checks were coherent. No full-dev
question or answer harvesting was used to choose or extend this grammar.

Baseline supplied by coordinator: full16 right/4456 refused/3 wrong of4475;
first100 1/99/0; synthetic3056/60/0 of3116, including762 expected refusals.
Supplied frozen reports had a different build hash from the supplied executable.
Replaying that actual executable independently confirmed the exact baseline;
repository-visible baseline reports here are those fresh replays.

## Changes

- `src/linear.rs`: dimensionless `ScalarExpr` (two rational coefficients plus
  constant), typed `Equality` constraints and an explicit target expression.
  Exact elimination solves one or two unknowns, requires full rank, and checks
  every original equality, including redundant/contradictory extra constraints.
- Recursive token grammar composes sums, ordered differences, plus/minus,
  scalar/fraction/percentage factors, twice/thrice and more/less-than amounts.
  Each clause is parsed independently; clause order does not encode an equation
  template. Single-number references and first/second references cannot mix.
  Repeated indefinite introductions, unbound singular references, unsupported
  tokens, units, nonlinear forms, extra prose and missing targets refuse.
  Bounds: 4096 input bytes inherited from parser, depth16, eight constraints,
  checked i128 rational intermediates. Overflow refuses.
- `src/lib.rs` registers the private module; `src/parser.rs` tries a successful
  complete linear parse before the existing percentage parser. Nonmatching
  inputs continue through existing parsers. No runtime data files, Python, model,
  network calls, answer tables or formula labels.
- `tests/linear_language.rs`: three tests covering four training-concept
  examples, eight novel cases and twenty refusal cases. Existing five tests pass.
- `check.py`: new local development adapter only; compares all3116 frozen
  synthetic outputs and independently constructs50 additional cases using
  Python Fraction. Includes six permutations of three clauses, negative and
  fractional values, affine query expressions, singular systems, and contradictory
  extra constraints. It does not read old generators or evaluator secrets.
- New reports, training evidence, arithmetic audit and `reproduce.sh` stay here.

## Original worker measurements, not final acceptance

| Unchanged evaluation | Baseline right/refused/wrong | Candidate right/refused/wrong |
| --- | --- | --- |
| Official MathQA validation first100 | 1/99/0 | 1/99/0 |
| Official MathQA validation full4475 | 16/4456/3 | **24/4448/3** |
| Existing synthetic3116 | 3056/60/0 | 3056/60/0 |

All19 previously answered dev rows retain their entire answer and outcome.
All3116 synthetic status/value/unit triples are identical, including762 correct
expected refusals (2294 substantive correct answers). Independent local checks:
50/50. No denominator rows were removed. Full external coverage remains tiny;
this is eight additional correct answers, not broad mathematical understanding.

The unchanged exact-numeric metric retains1229 nonnumeric gold options in the
full denominator. Any answer with unscorable gold counts wrong. The same three
baseline errors remain: row454 exact80/3 vs rounded26.67; row4045 exact536.0268
vs choice536; row3795 unit-bearing unscorable conversion option. No tolerance,
rounding, choice proximity or scorer change was applied. New gold mismatches:0.

## Independent semantic/arithmetic audit of every new answer

Rows are zero-based official validation indices. Arithmetic was translated
manually from each newly answered question and independently evaluated with
Python Fraction, then compared against both Rust and gold; all agree. No train
annotations or benchmark programs were accessed or executed.

| Row | Semantic translation and target | Independent result |
| --- | --- | --- |
| 1094 | 11x=132; target x | 12 |
| 1252 | (15/100)(30/100)(50/100)x=108; target x | 4800 |
| 1410 | x−(2/5)x=510, hence x=850; target (12/100)x | 102 |
| 1702 | 14x=210; target x | 15 |
| 2410 | (50/100)x=(40/100)120+180=228; target x | 456 |
| 3079 | x−(3/4)x=100, hence x=400; target (25/100)x | 100 |
| 3339 | (35/100)x=(50/100)x−24; target x | 160 |
| 4461 | (20/100)x=(30/100)120+80=116; target x | 580 |

The fraction possessives refer to the single introduced number; comparative
amounts bind to the right-hand expression, and percentage targets are amounts,
not a requested percentage unit. Difference follows ordered subtraction, the
existing arithmetic convention. Absolute-value difference is not implemented.
All eight measured gains involve one unknown; the two-unknown composition
capability is supported by independent local evidence, not a measured dev gain.

## Rejected approaches and intermediate failures

No benchmark-scored candidate was rejected or tuned after the full-dev run.
Rejected numeric-fragment extraction, complete problem templates, arbitrary
one/other reference binding, temporal transformation prose, and nonlinear targets
as outside this bounded grammar. Initial routing after the percentage parser
failed focused tests because that parser eagerly refused abstract percentages;
moved the complete linear parse before it. Initial const initialization failed
because Rational::integer is not const; replaced with ordinary constructors.
Clippy rejected explicit panic! in a test helper; replaced with an assertion.
Repeated indefinite introductions were explicitly guarded before full dev.
No cosmetic fallback was substituted for the hypothesis.

## Provenance and new data

Existing mounted MathQA, AllenAI redistribution of Amini et al.; dataset revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0 (dataset card revision
`c4f1cc784c04c4957b50c97858f23893b633eea6`, as documented by the prior handoff).
Sources: https://huggingface.co/datasets/allenai/math_qa and
https://math-qa.github.io/math-QA/ . No downloads in this batch.

- Train:29837 rows, 9,013,733 bytes; SHA256
  `c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
- Official validation:4475 rows, 1,350,141 bytes; SHA256
  `488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
- Train content was read only via PyArrow23.0.1 `columns=['Problem']`.
  Existing PyArrow mount reused; no package download/install. Files hashed for
  provenance. No train rationale/options/formulas/programs/other annotations
  were inspected. The unchanged dev scorer accesses validation gold as before.
- New local data: `linearwords-local-development-v1`, MIT OR Apache-2.0,
  deterministic construction without random seed, 50 rows/8,380 JSONL bytes.
  SHA256 `4fd43ddbce1fc5f0eda31934363aee05454ec1c77c77cafc737fec70ce8db299`.
  Generated by the new `check.py`, retained at `$TMPDIR/novel-cases.jsonl`.
  These cases are local development checks, not held-out/independent benchmark data.
- `train-evidence.json` (1,826 bytes) is a small attributed extract of permitted training
  Problem text, not a new training corpus. File hashes are in `SHA256SUMS`.

## Reproduction, validation and resource ownership

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/linearwords-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/linearwords-run/target
bash tools/quantities/linearwords/reproduce.sh
```

Requires the frozen `$TMPDIR/glassmind-quantities-baseline` already supplied by
the coordinator. The script points to the existing quantities-run data/dependency
mounts and writes replay reports under owned TMPDIR. It invokes the unchanged
`tools/quantities/evaluate_mathqa.py`; its SHA256 remains
`226f416ddff557bb752a41ae1e799235b0bcab72d2d07287ece12630db48f69d`.

Passed: package fmt check; all-target package clippy with warnings denied;
eight package integration tests; package build; fixed dev sample; full dev;
frozen synthetic comparison;50 independent cases; git diff --check.
Every cargo invocation used explicit owned TMPDIR, the existing target directory,
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, -j1 except fmt, and --test-threads4 for tests.
Outer `glassmind-continuous-linearwords.scope` verified MemoryMax8589934592,
MemorySwapMax0. All started work completed; no continuing jobs/services.

Candidate binary SHA256:
`b21efdee12c35e964ece31115644de1e5707066f724d28368615c7117698ef30`.
Frozen baseline binary SHA256:
`5b11dd50ad89b8370f944396a2953402a242f6a057d67ac6ece32c9918fecfe8`.
Full question stream SHA256:
`bf0c0a3d095234fe1b00707e0d0b748477225acf4de1cb633ed6095b66e425f2`.
Sample question stream SHA256:
`1c42e4ea2b67b4e7783b5e10695bffd839c097739e32e6edf8cca6d3f25df269`.
Both canonical synthetic streams SHA256:
`93b2bd1c9f8822af95a68d02663a90d122d24ca57d730c61c670e2b11079bbe9`.

Main talk was not rebuilt; coordinator verifies automatic library integration.
No agents, LLM calls, credentials/API keys, test split, held-out/hidden code or
old private logs were used. No commit. Existing model/session configuration was
not changed, and no unverified claim of changing model/tier/authentication is made.
