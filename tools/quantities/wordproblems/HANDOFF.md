# Ratio-total experiment (preregistered before implementation)

Base source: 742cc2a9a0ac501f8de5d93d54f55c6476b168c5. Hypothesis: fully consumed ratio-and-sum clauses over abstract numbers add real coverage with exact checked arithmetic. One representation: RatioTotal { positive rational weights (2–4), positive rational total, selector (ordinal/min/max) }. Adjacent three-number ratios normalize to weights [a*c,b*c,b*d] for a:b and c:d. No entity guessing, units, rounding, choice inputs, labels, or runtime tables.

Coordinator integration source: `6ac211b7452383c52ec3791bee40f0b8f69b31ef`.
Independent release-binary sample/full replay reproduces every worker answer.
All 3,116 synthetic status/value/unit outputs match the frozen baseline; all
five quantity tests and workspace fmt/clippy/build/strict4/4 smoke pass.
Existing conversation checks remain27/27 with and without the intent model.
Four independent ratio questions through the main `talk` command improve0→4
correct in both configurations; memory and prior-calculation recall are preserved.
Integrated evidence and binary hashes are in `reports/integrated/`.

Grammar (case/whitespace insensitive, punctuation explicitly consumed):
- `the ratio of COUNT numbers is RATIOS and their sum is TOTAL . QUERY`
- `COUNT numbers are in the ratio RATIOS [.] [and] [their sum is | add up to give a sum of] TOTAL . QUERY`
- `the sum of [COUNT] numbers is TOTAL . [if] the ratio {between [the] first and [the] second be | of the first to the second is} a:b {and that between [the] second and [the] third be | and that of the second to the third is | and the ratio of the second to the third is} c:d , [then] QUERY`
- RATIOS contains exactly COUNT positive numeric atoms separated by colon/to (slash also allowed for adjacent pair notation). COUNT = two/2, three/3, four/4; omitted chained count means three. QUERY asks `find the SELECTOR number`, `the SELECTOR number [of the COUNT numbers] is`, or `the greater/lesser of the COUNT numbers is`; SELECTOR first/second/third/fourth/last/biggest/largest/smallest/greater/lesser. Explicit counts must agree. Unsupported clauses/units refuse.

Fixed TRAIN Problem-only evidence, independently calculated (no annotations read): train rows 10100: ratio2:3:5 total150 second=45; 9630: ratio2:3:4:5 total1344 biggest=480; 28312: ratio3:4 total21 greater=12; 28122: ratio5:1:4 total1000 last=400; 6897/9276/26441: chained2:3,5:8 total98 second=30; 859: chained5:3,4:9 total177 second=36. IDs only identify provenance here, never runtime branches.

Acceptance: full4475 correct increases, all14 baseline correct retained; sample baseline1/99/0 correct retained; all3116 synthetic status/value/unit unchanged; audit all newly answered dev cases, count every gold mismatch wrong, reject/fix introduced semantic errors. Baseline observed before edits: sample1right99refused0wrong; full14right4458refused3wrong (1229 unparseable golds unchanged). Baseline binary SHA256 80a2120912f95a8266a15d7e756c2cec145fd12ce4b12737e30e6d27e8fa57d2.

Scope clarification: the worker read `crates/quantities/dev/evaluate.py` and flagged its synthetic generation code under the broad wording of its worker brief. Coordinator review confirmed this is the repository-visible synthetic development adapter used by earlier authorized checks, not hidden/official-test generation or held-out data. The goal permits visible development tooling. Regression here uses the existing frozen JSONL; no held-out data or private logs were read. The original worker report is retained in its submission archive.

## Final result and exhaustive new-answer audit

Accepted numerical result: **14/4458/3 → 16/4456/3 right/refused/wrong out of4475**. Fixed first100 stays **1/99/0**, with the same baseline answer. All17 baseline full-dev answers (14 right and3 wrong) remain byte-equivalent at the answer-record level; no baseline-correct loss. All3116 frozen synthetic status/value/unit triples match exactly, preserving3056right (including762 expected refusals),60refused,0wrong. Synthetic aggregate is inherited from the supplied baseline plus exact output preservation, not a rerun of generating code.

Every newly answered case (zero-based official dev row; IDs used only in this audit):

| Row | Fully consumed question | Independent calculation | Candidate / gold | Outcome |
|---|---|---|---|---|
|882|the ratio of three numbers is 5 : 1 : 4 and their sum is 1000 . the last number of the three numbers is ?|1000 × 4 / (5+1+4) =400|400 /400, unit null|right|
|2454|the ratio of three numbers is 2 : 3 : 4 and their sum is 180 . the second number of the three numbers is ?|180 × 3 / (2+3+4) =60|60 /60, unit null|right|

No introduced gold mismatch or semantic error observed. Existing scored errors remain counted wrong: row454 exact80/3 vs gold26.67; row3795 exact300-mile conversion301752/625 km has unparseable gold; row4045 exact66.67% of804 gives1340067/2500 (536.0268) vs gold536. No metric correction or rounding applied. All1229 unparseable gold rows stay in the denominator.

## Exact changes and validation

- Added `src/ratio_total.rs`: one grounded positive-rational ratio-total representation, three fully anchored clause productions, checked arity/ordinal references and exact checked arithmetic. Direct ratios accept fractions; adjacent chain slots accept integer/decimal atoms, colon or slash separators. Greater/lesser queries require exactly two numbers; largest/smallest support2–4. Units and extra conditions refuse.
- `src/lib.rs` registers the private module; `src/parser.rs` invokes it through the shared library. MainCLI and standalone main source unchanged.
- Added3 focused integration tests:8 fixed train-derived arithmetic examples,4 independent successful examples,18 adversarial refusals. Existing2 percentage tests retained.
- Added reproduction shell script and frozen-output comparison helper; unchanged external evaluator receives Problem strings only. No train options, rationale, formulas, categories or programs inspected. Dev gold values are accessed solely by the unchanged evaluator and its audit outputs.
- Final crate fmt check, all-target clippy with warnings denied, all5 integration tests, standalone debug build, sample then full MathQA evaluation, and full3116 frozen synthetic comparison pass. Final command log is `TMPDIR/final-validation.txt`. Every cargo/eval command used inherited/exported ownedTMPDIR and existingCARGO_TARGET_DIR under systemd8GiB/0swap, setsid--wait, timeout600/kill10; cargo-j1, testthreads4.

Rejected/limited work: no second representation and no dev-driven grammar expansion. Initial candidate had the same16/4456/3 result. Review rejected ambiguous chained `2/3/4` and comparative greater/lesser over more than two numbers; both now refuse with fixtures. Initial clippy failed guarded direct indexing and explicit panic in a test helper; replaced with checked first-element access and assertion. A subsequent formatting check failed before rustfmt; final replay passes. No failed evaluation or semantic mismatch was hidden.

## Provenance, reproduction and limitations

Existing data only; **0 newly downloaded data bytes**, no new model. AllenAI MathQA parquet revision `fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, dataset-card revision `c4f1cc784c04c4957b50c97858f23893b633eea6`, Apache-2.0 per existing source manifest. Source URLs:
- https://huggingface.co/datasets/allenai/math_qa/resolve/fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6/default/train/0000.parquet
- https://huggingface.co/datasets/allenai/math_qa/resolve/fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6/default/validation/0000.parquet

Verified train9013733bytes SHA256 `c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`; validation1350141bytes SHA256 `488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`. Existing files read-only; train read via PyArrow `columns=['Problem']`. Officialtest never downloaded/read. No credentials, API keys, model calls, agents, services, commits or continuing jobs.

Reproduce from this uncommitted worktree (frozen baseline binary must already exist):

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/wordproblems-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/wordproblems-run/target
bash tools/quantities/wordproblems/reproduce.sh
```

The script sets the existing read-only PyArrow path and uses the unchanged MathQA evaluator; writes large outputs only underTMPDIR. `compare.py` runs both binaries on frozen synthetic JSONL and compares every status/value/unit, and audits every baseline/new dev answer. New runtime code has no Python dependency.

Limitations: small scoped dev gain, extensive remaining refusals. Both gained cases share training templates (row882 closely parallels train28122); this demonstrates bounded template generalization, not broad word-problem understanding. Adjacent-chain and rank variants passed train/independent fixtures but added no measured dev answers. Named entities, physical units, negative/zero-total ratios, whole-number feasibility, ratio differences, additional constraints and rounding requests remain unsupported. Checked intermediate overflow can refuse some mathematically representable results. The worker did not rebuild main talk; the coordinator subsequently verified it as recorded above. The development-tool scope clarification is recorded above. No new model configuration/authentication claim is made: no new LLM process was launched.

Final candidate binary SHA256 `2c260e83d08199487c7c194d385b4b91ccb5302ca06d301b289ced2702f4aa5b`.
Baseline binary SHA256 `80a2120912f95a8266a15d7e756c2cec145fd12ce4b12737e30e6d27e8fa57d2`.
Full question stream SHA256 `bf0c0a3d095234fe1b00707e0d0b748477225acf4de1cb633ed6095b66e425f2`.
First100 question stream SHA256 `1c42e4ea2b67b4e7783b5e10695bffd839c097739e32e6edf8cca6d3f25df269`.
Both synthetic canonical status/value/unit streams SHA256 `687aa81d37da226a8a3369195d2330868bb804dcc64f1a2956e10d7cf295cf2d`.

Evaluated source/script hashes (base742cc2a plus uncommitted changes; evaluator unchanged):

```text
aff54a913eb179476e084783e68241b449186483eb1f380cf16bc33d7ad31846  crates/quantities/src/lib.rs
ff89df837ad8eff1f838b2da06847a10f5b465cacd23ca325367c3a1ca56683d  crates/quantities/src/parser.rs
b08d5a838210944cd73a716f0f4d0f4329f2d6504aebeabe23d71428abb2446f  crates/quantities/src/ratio_total.rs
95a6a87eb0ecac3dfeb7c9ffbcc259bbe0c064605bfd133cc54762f8a46c4487  crates/quantities/tests/ratio_totals.rs
57d556afdf0270c7ba3829e90f5a8f56c9ff63cf3c6e7a5209971c5f3ac4b7fc  tools/quantities/wordproblems/compare.py
735c826503d52d02588b3c34490e9626389cab26032a4646eeb80605455524b4  tools/quantities/wordproblems/reproduce.sh
226f416ddff557bb752a41ae1e799235b0bcab72d2d07287ece12630db48f69d  tools/quantities/evaluate_mathqa.py
```

Retained artifact hashes under ownedTMPDIR:

```text
f62de5a364671d592d74dc765c7ea1538bca772a7dba2868c4a715f54db3c844  baseline-sample.json
28506f58961611bc5408241636195038e2e4c9b3f9cefb6c01789759147e00a2  baseline-full.json
13f16e93e2e713c95d4b9ce4ddd1660bee53844f7f1c0c919cc9fc049995b518  candidate-sample.json
3026350a39ca83903c1b49d76fe0e73de942b1165a9d9ecd1551766fe42cd7c8  candidate-full.json
6bad0a36f50977cfe0a0dbca912f5b3f1db293b24be23d290c479bf87ef86543  comparison.json
ab818fdee8b4b383c8e0439dae83d9136926c6a5b4558c66033dfff4dac895ab  baseline-synthetic-predictions.jsonl
ab818fdee8b4b383c8e0439dae83d9136926c6a5b4558c66033dfff4dac895ab  candidate-synthetic-predictions.jsonl
8fda8895da2f4922df5bb123b606b0bd3daa8d4fc792268ba09fea5d4463797d  final-validation.txt
```

Coordinator audit helper accepts optional `CANDIDATE_BINARY` for the integrated release binary; default worker reproduction still uses its debug binary. Comparison semantics are unchanged.
