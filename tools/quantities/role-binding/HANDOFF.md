# Rejected TRAIN-only role-binding experiment

Completed one fixed configuration, uncommitted, owned files only. **Continuation gate failed; do not promote this model.** No production crates, scorers, defaults, CLI, loop, shared manifests, historical prompts or evaluation definitions changed. No DEV/held-out/parent semantic cases read; no downloads, extra agents/model calls, services or commits. Base commit `3c0ad3dace4de37476808347f2012599abb63467`.

## Change and material distinction

The external Rust helper reuses byte-identical original proposal/binder/rational sources. The frozen operation head still consumes its original global hashed features. Only argument scoring changes: one 65,536-weight linear ranker scores every legal operand using the same candidate-relative feature map. There is no bank of weights indexed by input slot. Thus a mention's surface cues can transfer to another numeric position, unlike the rejected 42-way heads (which already had +/-3 slot context, rank and shapes).

Native numeric spans determine punctuation-delimited clauses. Each mention has explicit clause membership, following surface entity candidates, preceding predicate-context candidates, adjacent surface unit cues, clause words, shape and rank. Missing/ambiguous sets are explicit. The graph includes surface overlaps with the query clause and selected left operand, same-clause relations, usage in the prefix, and prior-step operation/recency/descendant-mention unions. Constants keep existing typed identities. Features are inspectable strings, deterministically hashed; side/operation and depth conjunctions share weights. No answer table, operator phrase whitelist, learned CAL vocabulary or runtime annotations.

These are **surface candidates**, not an entity/POS/dependency parser. In particular, predicate-context includes all preceding clause words; entity candidates include all following words before the next number/clause boundary; a unit cue may be a noun or another word. No claim that these recover true semantic roles. Pronoun/coreference and implicit relationships are unresolved. The clause permutation check establishes feature sharing, not synthetic accuracy.

One predeclared schedule: clean FIT alignments only, 10 epochs, fixed order, zero initialization, softmax SGD 0.20, inverse sqrt feature-count scaling. No tuning/early stopping/selection. Training completed in 8.623 seconds after one launch-only failure. Ranker 746,608 bytes; with the unchanged original model, 8,792,784 bytes (<16 MiB). All native/training/cargo processes inherited 8 GiB/zero-swap scopes and bounded timeouts. Original legal masks, numeric binding, rational arithmetic, greedy strict-greater ascending ties, STOP behavior and six-step horizon are unchanged. Oracle supplies gold full operation plan and length, predicted arguments and prefix, forced STOP at gold length (length six uses horizon); it is not production performance.

## Reconstruction and validation

Exactly 29,837 original question-group memberships: FIT 23,912 / CAL 5,925. Clean FIT 4,590 / CAL 1,182. All excluded IDs and reasons retained. Original parser is AST-extracted unchanged; historical adapter `main` is never run. All 29,837 gold native execution records and all 4,590 original FIT input records match. Full exclusion counts: [cleaning.json](reports/cleaning.json).

Before training, first 200 original TRAIN-CAL predictions from the original binary matched the reused free decoder exactly, including parsed scores and metadata (also byte-identical). The final helper passed the same check. Independent post-evaluation original-binary replay matches every clean FIT and CAL baseline output. Every baseline/candidate free/oracle proposed AST was independently executed by the original native executor; status, program, numbers and rational value match. The unchanged typed-label comparator scores saved predictions. Free inference ran in separate processes over files containing only JSON question strings; actual annotation-object ingress returns code 1 with zero predictions.

Four meaningful Rust checks pass: candidate-relative features follow a mention under clause/number-position permutation; missing/ambiguous relationships remain explicit; annotation objects cannot deserialize as free strings; illegal forward step references remain excluded. Final tests and formatting checks passed. Only dead-code warnings in copied original code. No giant suites or DEV evaluation.

## Complete results

All numeric outcomes use the original typed comparator. Clean labels are already annotation/numeric-agreement selected, so these are scoped TRAIN results. Every row is retained. All unscorable counts on these clean evaluations are zero.

| Split/model | Teacher op | Teacher left | Teacher right | Whole-row teacher all arguments |
|---|---:|---:|---:|---:|
| FIT baseline | 12354/15753 | 12886/15753 | 12857/15753 | 2258/4590 |
| FIT candidate | 12354/15753 | 11771/15753 | 12319/15753 | 1598/4590 |
| CAL baseline | 1912/4138 | 2197/4138 (53.09%) | 2499/4138 (60.39%) | 202/1182 |
| CAL candidate | 1912/4138 | 2370/4138 (57.27%) | 2540/4138 (61.38%) | 133/1182 |

Teacher left receives gold operation/prefix; teacher right also receives gold left. Op is nonSTOP gold-prefix accuracy. Whole-row counts require both arguments correct on every gold step.

| Split/model/mode | Exact AST | Numeric correct | Refused | Wrong | Whole-row all arguments |
|---|---:|---:|---:|---:|---:|
| FIT baseline free | 1775 | 1863 | 93 | 2634 | 1835 |
| FIT candidate free | 1279 | 1405 | 14 | 3171 | 1310 |
| FIT baseline gold-plan oracle | 2258 | 2334 | 93 | 2163 | 2258 |
| FIT candidate gold-plan oracle | 1598 | 1745 | 12 | 2833 | 1598 |
| CAL baseline free | 123 | 143 | 46 | 993 | 150 |
| CAL candidate free | 69 | 92 | 1 | 1089 | 89 |
| CAL baseline gold-plan oracle | 202 | 221 | 74 | 887 | 202 |
| CAL candidate gold-plan oracle | 133 | 170 | 5 | 1007 | 133 |

Denominators are 4,590 FIT and 1,182 CAL per row. Free whole-row arguments requires equal length and exact operands, independent of operation correctness. Execution refusals count as missing programs in this metric.

Frozen continuation gates: left >=56.09% **passes**; right >=63.39% **fails**; oracle numeric >=251 **fails**; free numeric >=143 **fails**. Left improves 4.18 percentage points, right 0.99 points, while joint binding and numeric results regress. CAL free loses 69 previously correct rows, gains 18, and turns 45 prior refusals into wrong outputs. Reduced refusal is not an improvement. This rejects this configuration; it does not establish that all shared rankers fail.

[Full FIT report](reports/fit-report.json) and [full CAL report](reports/cal-report.json) contain every denominator, teacher counts by step/operator/input count, decoded argument counts by those same groups, row outcomes by gold length/operator/input count, whole-row metrics, lengths, refusal reasons, and baseline-to-candidate transitions. Row-level outcomes, independent native records, weights and all logs are external. Row-level operator groups overlap when a gold program contains several operator kinds; step-level groups partition their decision denominators.

## Provenance and evidence

RUN: `/home/gsc/.cache/glassmind/continuous-20261010/rolebinding-run`.
Terminal archive: `RUN/frozen-candidate/`, with `HANDOFF.md`, recursive SHA-256/byte manifest, source snapshot, source assets, executable, and all original evidence. Live `RUN/evidence` is also preserved. `pretraining-freeze.json`, `precal-freeze.json` and `plan.sha256` establish the frozen configuration, membership, scripts, weights, sources and model. `commands.log` records argv; logs record systemd scopes. No post-CAL representation or metric changes.

MathQA existing revision `fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0. TRAIN parquet 9,013,733 bytes, SHA-256 `c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`. Source card 7,438 bytes, SHA-256 `8051c9a1e42245600985ff1b890dfe3ecce804c47555d83ca7a02a2e3f4b4cc2`. **New downloaded data: zero bytes.** Full original assets and hashes are in `source-manifest.json`; original model SHA-256 `2b56181582960a862b2da422b0988cf3e20671949c80253bb5b0eaa2d49478e3`. Copied repository code retains MIT OR Apache-2.0 project licensing. Existing local PyArrow dependencies used; no dependency downloads.

Rejected/failed attempts: one experiment/configuration only. Initial training launch exited 127 because `/usr/bin/time` did not exist, before executing the trainer; preserved `train-launch-failure.log` and `train.log`. The same schedule ran using Bash `time` (`train-retry.log`). No weight/model search or CAL-driven adjustments. Resource caps are verified; peak RSS was not measured. No additional LLM calls or credential access; current session model/login selection is externally managed and not independently changed/verified here.

## Reproduction

Run from this worktree. `bounded.sh` sets persistent TMPDIR and the existing CARGO_TARGET_DIR, removes common API-key variables without reading them, logs argv, and always uses `systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`.

```bash
tools/quantities/role-binding/bounded.sh cargo build --release --offline --manifest-path tools/quantities/role-binding/Cargo.toml -j1
tools/quantities/role-binding/bounded.sh cargo test --offline --manifest-path tools/quantities/role-binding/Cargo.toml -j1 -- --test-threads 4
tools/quantities/role-binding/bounded.sh cargo fmt --manifest-path tools/quantities/role-binding/Cargo.toml -- --check --config skip_children=true
```

Exact original pipeline (already completed; do not overwrite populated evidence): `bounded.sh python3 prepare.py`; fixed native `train ranker.json < fit-gold.jsonl`; `python3 freeze.py`; `bounded.sh python3 evaluate.py fit`; `bounded.sh python3 evaluate.py cal`; `bounded.sh python3 verify.py`, all script paths under the owned folder. `prepare.py` and evaluation scripts intentionally contain the exact source/output paths. Replay safely into new files instead:

```bash
ROLE_RUN=/home/gsc/.cache/glassmind/continuous-20261010/rolebinding-run
ROLE_SRC=/home/gsc/.cache/glassmind/continuous-20261010/groundingaudit-source
ROLE_BIN="$ROLE_RUN/target/release/quantity-role-binding"
# Identical training schedule; output goes to a fresh path, never original weights.
tools/quantities/role-binding/bounded.sh "$ROLE_BIN" train "$ROLE_RUN/reproduced-ranker.json" < "$ROLE_RUN/evidence/fit-gold.jsonl"
cmp "$ROLE_RUN/reproduced-ranker.json" "$ROLE_RUN/evidence/ranker.json"
# Question-only independent free inference. Other modes: teacher, oracle, with *-gold.jsonl.
tools/quantities/role-binding/bounded.sh "$ROLE_BIN" free "$ROLE_SRC/model.json" "$ROLE_RUN/evidence/ranker.json" < "$ROLE_RUN/evidence/cal-questions.jsonl" > "$ROLE_RUN/reproduced-cal-free.jsonl"
cmp "$ROLE_RUN/reproduced-cal-free.jsonl" "$ROLE_RUN/evidence/cal-candidate-free.jsonl"
```

Coordinator alone may authorize any later DEV/semantic work. This batch recommends preserving the rejection and examining stronger defensible relationship extraction in a separately preregistered TRAIN experiment; no further configuration was attempted here.

Frozen candidate ranker SHA-256 `d726cf38216910edd00eacfa4f2a906a9e13886a5720c4ef761fe95219719cf8`; feature/scoring source SHA-256 `0938562a72c681b498057664093a43cb3ef3ce6e033d493a87f2a77860525694`.
