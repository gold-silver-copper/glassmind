# REJECTED CAL — evidence only, runtime restored

One bounded experiment at source `80b9f636050accfbe6ca0fbdb0ecc2bc6ca531d9`.
No commits, agents, additional LLM calls, services, credential access, API keys,
downloads, held-out/test data, scorer changes or shared-file edits. All owned
commands finished. Only NEW `tools/reading/unanswerable/` remains untracked.
Do not deploy the preserved patch or failed weights.

## Hypothesis, exact change, and decision

[PREREGISTER.md](PREREGISTER.md) records the hypothesis, exact features, single
training schedule, both gates and prior rejection lessons before runtime edits,
fitting or CAL. Genuine SQuAD2 unanswerables and wrong frozen-v1 spans provide
negative supervision for an optional answer-versus-null sparse linear selector.
The only rejected runtime edits were `qa.rs` exporting `qa/null.rs`, that new
module, and optional `reading-qa predict --null-model` / `--null-features` flags.
No main CLI changes. The span ranker and its model were unchanged. Candidate
outputs would always be unverified proposals; selecting a quote is not proof.

8192 hashed sparse features: scalar score/margin, lengths, answer shape, anchor
coverage/position/order, missing capital/numeric anchors, coarse negation,
repetition, first-question-word interactions and lexical anchor-side/boundary
features. See preregistration and the original [unapplied patch](rejected.patch)
for exact formulas. Zero initialization, 12 SHA-ID-ordered logistic AdaGrad
passes, rate .08, accumulator1, active-coordinate .0001 weight penalty; no
resampling, class balancing, alternative schedule or hyperparameter search.
First weights saved BEFORE CAL, never retrained. The saved threshold0 is an
unselected serialization placeholder, NOT a calibrated operating threshold.

CAL fails the frozen >=60% substantive precision AND >=100 substantive-correct
AND strict-correct>always-refuse gate. Independent saved-output audit rebuilt
all score tie groups and matched the complete frontier exactly.

| CAL, all2000 | Frozen v1 | Diagnostic prefix with best precision at >=100 correct |
|---|---:|---:|
| Substantive correct |116|100|
| Wrong |225|120|
| Correct expected refusals |518|569|
| Unsupported answerable |1141|1211|
| Substantive precision |34.0176%|45.4545%|
| Strict correct |634|669|

The second column of results is an unselected CAL frontier diagnostic, not an
accepted candidate. At >=60% precision, the maximum correct count is only48.
Always-refuse CAL correct606. No threshold relaxation, CAL rescue, second fit,
**candidate DEV sample or candidate full DEV**. Therefore final DEV acceptance
was not reached and no DEV improvement is claimed. This complies with the
explicit stop-on-CAL-failure rule, which supersedes routine sample/full evaluation.

Official metrics remain separate from strict diagnostics: frozen v1 CAL
EM31.7% / F132.8387406%; full SQuAD2 DEV baseline EM46.946854207% /
F147.435937977%, 433 substantive correct, 5140 correct expected refusals,
5064 unsupported,1236 wrong. Its strict5573 differs from official5574 because
one substantive `A` normalizes empty; it remains wrong in strict diagnostics.
Full DEV always-refuse5945/11873 (50.071591%). No candidate official DEV score.

## Selection, leakage, and provenance

Mapped the original80 v1 FIT/CAL article hashes to titles using ONLY old v1 TRAIN
and its selection report, then excluded all80 titles from SQuAD2 TRAIN. Remaining
articles sorted SHA256(title): first96 FIT, next32 CAL; questions SHA256(id),
first8000 / first2000. Natural distribution retained: FIT2072 unanswerables,
CAL606. FIT labels:1235 officially exact nonempty raw proposals,2072 genuine
unanswerables,4693 wrong-span negatives. No synthetic mismatches.

Independent audit verifies title/context/ID disjointness across ALL selected
articles (before question caps) and DEV, exact selection and input field sets.
Zero shared contexts, zero TRAIN rows removed. IDs, titles and annotations stay
in offline labels/reports. Inference inputs contain only context/question;
unknown fields rejected. No runtime answer tables or neural/LLM models.

Pinned source revision `eee5fdbf62f8613a7812b03419e6b29617b74fd1`:
[upstream SQuAD explorer](https://github.com/rajpurkar/SQuAD-explorer/tree/eee5fdbf62f8613a7812b03419e6b29617b74fd1).
Dataset CC-BY-SA-4.0; repository/official scorer MIT. Reused TRAIN42,123,633 bytes,
DEV4,370,528 bytes:46,494,161 dataset bytes, **zero new download bytes**. Original
v1 TRAIN30,288,272 bytes was used only to identify old article titles. Derived
FIT context/question JSONL6,707,920 bytes, CAL1,684,106; offline labels8,251,075
and2,060,032 bytes. Public-data-derived examples retain source license. Twelve
independent probes are newly authored diagnostic text, not independent benchmark
items. Exact source/scorer/data/result sizes and hashes: [provenance](reports/provenance.json).

Frozen v1 model873,881 bytes, SHA256
`b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5`.
Failed first weights205,742 bytes, SHA256
`0d9b397a11cd1f825320d6808d3dc70118875ef05507dadd99e13843a7cd7588`, retained
in [artifacts/first-weights.json](artifacts/first-weights.json) and raw run.
Original frozen source/patch/trainer identities: [rejected-freeze](reports/rejected-freeze.json).

## Validation and substantive limitations

Release build, crate fmt check and clippy (`--lib --bin reading-qa -D warnings`)
passed. Eight focused QA tests pass (six existing and two new): v1 serialization,
finite overflow refusal, metadata rejection, bounds, deterministic repeated
spans, UTF8 offsets, new selector serialization/weights/input bounds and provenance.
Full11873 default-path replay is byte-identical to the retained frozen baseline:
SHA256 `6dd02aa2757011bec9971e47aef04e08260426f089eea004396ac13f0f9666a4`.
This is a v1 regression replay, not candidate DEV evaluation. SQuAD1 results are
preserved by unchanged default proposal code and exact restoration; no old v1
DEV read or full10570 replay. No main CLI or broader cross-track check.

All12 role/predicate/negation/ambiguity/Unicode/no-support contrasts with positive
counterparts were authored before fitting. [Probe report](reports/probes.json)
retains every raw proposal, v1 behavior and new unthresholded score. No selected
behavior is asserted without an accepted CAL threshold. Notable failures:

- Role positive proposes `thanked Pavel` instead of `Pavel` (score -2.5928);
  unsupported reversed role proposes `Pavel` and scores higher (-.2094).
- Both painted/demolished contexts propose2009; unsupported painted score1.1297
  versus supported demolition1.5962. This distinction is insufficient evidence.
- Negated demolition score1.5853 is nearly identical to supported1.5962.
- Ambiguous two-person context still proposes one name (`Pavel`).
- Supported no-support counterpart has correct raw2014 but old v1 refuses it.

These expose frozen proposal errors and weak semantic discrimination, not
entailment. FIT article distribution has fewer unanswerables than DEV; sparse
hash collisions, crude suffix stemming and lexical shortcuts limit transfer.
No post-CAL features or training changes. Existing binary line/model readers
retain v1 behavior; optional model has a1MiB load cap and finite8192-weight
validation. This rejected patch is not a production integration candidate.

Runtime restored byte-for-byte AFTER preserving the original patch and weights.
`git diff --exit-code -- crates/reading` and `git apply --check .../rejected.patch`
pass. [validation.json](reports/validation.json) records independent audit results.
Raw run: `/home/gsc/.cache/glassmind/continuous-20261010/unanswerable-run/experiment`.

## Reproduction

Read-only audit of this retained run (does not train or infer):

```bash
python3 tools/reading/unanswerable/audit.py
```

Deliberate experimental replay ONLY in a disposable checkout at the base source
with these new evidence files copied in; do not overwrite original reports.
Existing raw run refuses overwriting weights/predictions. Use a fresh NULL_RUN.
The target directory must remain the provided existing cache, TMPDIR outside /tmp:

```bash
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/unanswerable-run/target
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/unanswerable-run/tmp
export NULL_RUN=/home/gsc/.cache/glassmind/continuous-20261010/unanswerable-run/reproduction
git apply tools/reading/unanswerable/rejected.patch
bash tools/reading/unanswerable/reproduce.sh
```

The script runs scoped fmt/build/8tests, prepare, one fit, CAL, authored probes,
and only if CAL passes fixed200/full11873. Actual operations were run individually,
not a second whole-script run. Every cargo invocation set TMPDIR and used
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, -j1 except fmt, --test-threads 4 when testing. Outer scope
verified8GiB/zero swap. No background jobs outlive this worker.
