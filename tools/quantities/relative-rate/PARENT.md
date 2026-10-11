# Coordinator review: rejected relative crossing experiment

The candidate remains rejected: 29 correct explicit-label matches versus the
preregistered minimum of 31 on all 4,475 development rows. The frozen metric,
production solver and earlier rejected rate experiment remain unchanged.
No candidate runtime code was integrated.

The coordinator verified the original 30-file, 3,406,541-byte archive and
reconstructed all 24 frozen source/artifact identities in the worker checkout.
Saved first-100 and full predictions were rescored without repeating inference;
all ten report files matched byte for byte. All 25 production and 31 prior rate
answer payloads were preserved. The only added answer, row 2382, is independently
consistent with (1,150 + 600) metres / (150 km/h) = 42 seconds.
[Review record](parent-review.json) retains the exact checks.

Independent role probes found an additional semantic defect. At 36 and 72 km/h,
with lengths 100 and 200 metres, the candidate returns 30 seconds for both
“the faster train pass the slower train” and “the slower train pass the faster
train” in the same direction. The parser discards the speed-ranked passing
subject. The latter needs clarification or refusal under an overtaking reading;
the numerical calculation alone does not resolve that ambiguity.
[Exact probes and outputs](parent-role-probes.json) preserve the finding.
The opposite-direction development addition is symmetric and does not have this
particular problem.

The original `evaluate.py --score-only` does not enforce the saved inference
binary/prediction hashes itself. The coordinator checked those identities against
`inference.json` before replaying. Any future replay must retain that external
hash check; otherwise changed prediction bytes could silently be rescored.
The original script was preserved for reproducibility, not silently corrected.

Raw review artifacts are in
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/relativerate-review/`.
The original submission is in `relativerate-run/submitted/`, with its adjacent
`archive-manifest.json`. After reversing the temporary reconstruction, the
coordinator verified every archived file against the restored worker checkout
and repository copy before removing that owned checkout. No worker remains.
The original handoff retains its disclosure of reading a public development-test
generator despite the launcher's then-overbroad wording. Future launcher wording
now distinguishes permitted public tests from forbidden hidden evaluation assets.
