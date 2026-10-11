# Exact target determination accepted

Source `3539a2d2b0682ac2ba9db02af6543af96a7aeaa6` fixes a semantic coverage
gap: equality elimination previously required every unknown to be uniquely
determined, even when the requested expression was fixed. The baseline refused
all twelve solvable examples among twenty new contracts. The candidate passes
all twenty: twelve substantive answers and eight expected refusals.

The language parser and existing bounds are unchanged. Row reduction now skips
free columns and detects contradictory constant rows. Full-rank execution is
preserved. For an incomplete system, the target and absolute-difference sign
constraints are reduced by the equality pivots. Exact Fourier-Motzkin elimination
projects the feasible set onto the requested value. A singleton interval answers;
an empty branch contributes no solution; a feasible non-singleton interval
refuses. Every feasible absolute-difference branch must agree. No arbitrary value,
positivity or integrality is assigned to a free variable.

There are at most four sign inequalities plus two halves of the target equality.
Two elimination rounds produce at most nine and twenty rows respectively; a
defensive 64-row cap and checked rational arithmetic bound the computation.
Overflow remains a refusal, including intermediate overflow for an otherwise
representable result. No runtime model, floating point or benchmark identity
enters the solver.

Validation:

- The baseline passes eight of the twenty behavior contracts (expected
  refusals); the candidate passes all twenty.
- A separate 63-case oracle derives exact affine ranges from rectangle corners.
  Absolute-value equalities express the rectangle bounds. All cases pass,
  including free operands, collapsed intervals and inconsistent bounds.
- All 50 prior composition and 55 absolute-root contracts pass.
- All 3,116 synthetic status/value/unit triples are unchanged: 3,056 right,
  including 762 expected refusals; 60 unsupported; zero wrong.
- Fixed MathQA first100 and full4,475 DEV are unchanged: full29 right,
  4,443 refused, three wrong; all32 old answer payloads retained, no new answers.
  The existing frozen exact-label-comparison-v3 metric is unchanged.
- The integrated `talk` command passes all20 contracts in both plain and
  intent-model modes. Each of the12 successes per mode preserves persisted
  memory, story and draft state, with recall checked after restart. Real facts
  and dialogue bundles are loaded.
- Package tests, three new target-determination test groups, all-target clippy,
  formatting, release build and integrated strict four-turn smoke pass.

No benchmark gain is claimed. The new journal series measures twelve substantive
answers out of twenty authored contracts, with eight expected refusals reported
separately. The preceding production MathQA wrong answers remain counted wrong.
All23 quantity source/test/manifests match the integrated checkout exactly.
Source and executable hashes, plans, reports and reproduction scripts accompany
this report; full raw inference stays in the external artifact directory.

Before any inference, two draft sum-of-absolute-values questions were clarified
with explicit `sum of ... and ...` grouping; the ambiguous draft and correction
record are retained. No labels changed after observing candidate results.
The first compatibility-helper invocation lacked `CARGO_TARGET_DIR` and failed
before inference; the corrected environment produced the passing result.

No new dataset or model call was used. Authored code/contracts are MIT OR
Apache-2.0. Existing MathQA TRAIN/DEV source revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6` is Apache-2.0; only fixed DEV questions
entered inference, with labels read afterward by the unchanged offline metric.
No held-out content was accessed. Raw artifacts:
`~/.cache/glassmind/continuous-20261010/coordinator/linear-targets`.

The copied adapters preserve their original artifact-relative layout. Reproduce
using the retained external directory (do not overwrite completed output files):

```sh
ART=/home/gsc/.cache/glassmind/continuous-20261010/coordinator/linear-targets
python3 "$ART/check.py" --binary "$ART/candidate" --out "$ART/recheck.json"
python3 tools/quantities/linearwords/absolute.py --binary "$ART/candidate" --output "$ART/recheck-absolute.json"
```
