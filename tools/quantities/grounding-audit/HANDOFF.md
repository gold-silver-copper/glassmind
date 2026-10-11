# Rejected compositional model: TRAIN-only grounding audit

Completed one bounded diagnostic batch. No production acceptance, model fitting, feature/threshold/decoder changes, DEV access, downloads, extra LLM calls, agents, services or commits. Prior rejection and disabled calibration selection stand.

The strongest finding is **poor planning and binding even with correct context**, with additional autoregressive error propagation. Operation prediction excluding STOP is the weakest local metric (46.21% CAL), but this audit cannot assign a unique causal primary failure between planning and binding. It rules out a prefix-only explanation. Gold operation plans increase numeric matches by only 78/1182 (6.60 percentage points), leaving 961 rows without a match.

Suggested next experiment, not implemented: replace global hashed classification with explicit quantity-role and relation representations, supervised on TRAIN operand alignments and operation relations. Evaluate operation and argument heads separately under the same conditioning audit before proposing a solver. The 20–30 point FIT/CAL gaps and failures with gold prefixes justify changing representation/supervision; another decoder or threshold adjustment does not address these observations.

## Frozen corpus and exclusions

[PLAN.md](PLAN.md) and exact external membership were frozen before diagnostic results. All 29,837 TRAIN rows remain accounted for. The normalized-question SHA256 grouping replaces digit runs with `#` after lowercase/whitespace normalization; hash modulo five selects CAL when zero. Reconstructed split equals the retained split exactly: 9,521 FIT groups, 2,346 CAL groups, zero overlap. Clean FIT inputs match all retained input records, in order; 1,565 distinct FIT programs.

| TRAIN partition | All rows | Clean | Unsupported annotation | Binding/execution refusal | Wrong numeric label | Unscorable label |
|---|---:|---:|---:|---:|---:|---:|
| FIT | 23,912 | 4,590 | 8,522 | 3,989 | 2,470 | 4,341 |
| CAL | 5,925 | 1,182 | 2,129 | 985 | 645 | 984 |

Clean means the original bounded parser supports the annotation, original native gold execution proposes a value, and the unchanged typed-label comparator grades it correct. These are conditional clean subsets, not whole-TRAIN performance or ordinary generalization estimates.

## Free and oracle-plan results

| Mode | Split | Exact annotated program | Native numeric-label match | Refused/invalid arithmetic |
|---|---|---:|---:|---:|
| Frozen free decoder | FIT | 1775/4590 (38.67%) | 1863/4590 (40.59%) | 93 |
| Gold full operation plan; model arguments | FIT | 2258/4590 (49.19%) | 2334/4590 (50.85%) | 93 |
| Frozen free decoder | CAL | 123/1182 (10.41%) | 143/1182 (12.10%) | 46 |
| Gold full operation plan; model arguments | CAL | 202/1182 (17.09%) | 221/1182 (18.70%) | 74 |

All free refusals were division by zero. Oracle FIT has 92 division-by-zero and one rational-overflow refusal; CAL has 74 division-by-zero refusals. Syntax/forward-reference invalid counts are zero. Wrong counts: free FIT 2634, CAL 993; oracle FIT 2163, CAL 887. No clean output is unscorable. Oracle mode supplies the entire annotated operation sequence AND length; it forces STOP for 4124 FIT/1027 CAL rows and horizon termination for 466 FIT/155 CAL rows. It does not predict a plan or its termination.

## Conditional decisions and FIT-only majority controls

Each cell gives correct decisions/decisions (accuracy); final columns give rows with every decision of that head correct. Teacher operation uses gold prefix; left additionally uses gold operation; right additionally uses gold left. Whole-row denominators always include all clean rows.

| Condition/head | FIT top-1 | CAL top-1 | FIT whole rows | CAL whole rows |
|---|---|---|---:|---:|
| Teacher operation, including STOP | 16153/19877 (81.26%) | 2765/5165 (53.53%) | 2450/4590 | 188/1182 |
| Teacher left | 12886/15753 (81.80%) | 2197/4138 (53.09%) | 2878/4590 | 311/1182 |
| Teacher right | 12857/15753 (81.62%) | 2499/4138 (60.39%) | 2866/4590 | 351/1182 |
| Oracle-plan autoregressive left | 12406/15753 (78.75%) | 2008/4138 (48.53%) | 2809/4590 | 301/1182 |
| Oracle-plan autoregressive right | 12301/15753 (78.09%) | 2116/4138 (51.14%) | 2691/4590 | 273/1182 |
| FIT-majority operation by step | 6863/19877 (34.53%) | 1708/5165 (33.07%) | 37/4590 | 1/1182 |
| FIT-majority left by gold operation | 4623/15753 (29.35%) | 1286/4138 (31.08%) | 511/4590 | 115/1182 |
| FIT-majority right by gold operation | 3988/15753 (25.32%) | 1040/4138 (25.13%) | 186/4590 | 37/1182 |

Teacher all-heads-correct rows: FIT 1775/4590, CAL 123/1182. Teacher both-arguments-correct rows: FIT 2258/4590, CAL 202/1182; these necessarily equal oracle exact programs by deterministic prefix induction. Majority all-heads-correct: zero in each split. Baselines fit only 4590 FIT rows; no CAL targets enter counts.

STOP is relatively easy: FIT 3799/4124 (92.12%), CAL 853/1027 (83.06%). Removing STOP gives operation accuracy FIT 12354/15753 (78.42%), CAL 1912/4138 (46.21%). There is no operation decision at depth six, matching original training/decoder behavior. At the first step, before any prefix errors, CAL operation/left/right are 628/1182 (53.13%), 705/1182 (59.64%), and 751/1182 (63.54%).

[strata.json](strata.json) provides decision numerators, denominators, accuracy, distinct row denominators and whole-row correctness within each head/step/operator/input-count stratum. Operator IDs 0/1/2/3/4 mean add/subtract/multiply/divide/STOP. [summary.json](summary.json) contains aggregate counts and selfchecks.

## Exactness, scope and limitations

The Rust helper includes the original proposal and rational modules. Original proposal code changes only by appended diagnostic/test includes, recorded in external `instrumentation.diff`; no numerical/features/mask/decoder edits. The original three heads have 5/42/42 actions and 8192 features. Legal operands are current input indices, 20 constants, and strictly earlier steps. Scores retain f32 weights, original ordered f64 accumulation and normalization; ties use strict `>` and the first legal action, with no tolerance. Majority also uses first-in-mask ties.

Original native replay agrees with all 200 retained first-CAL predictions in every parsed field, including scores and numeric results. Helper agrees exactly with that replay. Original native gold execution agrees with all 29,837 retained records. Further checks match all 4,590 original native free FIT predictions and all 1,182 clean retained CAL predictions to helper outputs. CAL question order matches TRAIN split membership. Final build replay also agrees on all 200 rows. These are exact comparisons, not approximate Python ports.

Three authored native tests pass: legal masks/stable ties, operation conditioning of both argument heads, and forward-reference rejection. Corpus-wide assertions check every target/prediction legal, STOP/horizon denominators, explicit oracle mode, changed argument features under perturbed gold operation, all rows present, and FIT-only baseline fitting. Raw base features take only question/native spans. Free mode accepts question strings only; gold enters only the explicitly named TRAIN oracle routine.

Exact AST errors include commuted/equivalent programs. Numeric matches do not establish English semantic correctness. The right-operand teacher-to-autoregressive gap combines previous-prefix corruption with current predicted-left errors; it cannot isolate those two causes. CAL is an existing TRAIN calibration subset, not fresh independent evaluation. Cleaning is annotation/numeric-agreement selection and covers only 5772/29837 rows. No DEV or parent semantic checks were accessed or repeated. No failed audit attempt or alternative model experiment occurred; build logs and original/initial/final outputs are retained. Adding row statistics completed the already frozen stratum metrics without changing scoring. Original production source hashes and supplied source/binary hashes are unchanged.

## Provenance and reproduction

MathQA revision `fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6` as supplied by coordinator; Apache-2.0 per mounted source card. No new data downloaded. Mounted TRAIN is 9,013,733 bytes, SHA256 `c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.

Model: 8,046,176 bytes, SHA256 `2b56181582960a862b2da422b0988cf3e20671949c80253bb5b0eaa2d49478e3`. Original proposal source SHA256 `f341f213d9ca351c29160af02edab3fd34b81f8ab56ef8327673162069a1eaa3`; original decoder binary SHA256 `7f8f6f8658cedde3590fbf7dd2e462cb88f505e3f0dcc22e6249dd1f9079c0d3`; executor SHA256 `40e6a6e975cf8474231ac0e3bcc2e3df5c1ce8196dead0b9159586aa5935ea6f`. Full mounted manifest is verified before and after work. Comparator SHA256 `24f1d65ed2f1785b940d7c674565dd9b49c168e498f57c23cbd6c8c2c6eab6f0`.

From this worktree, with the supplied mounts and existing target directory, choose a fresh persistent directory:

```sh
AUDIT_RUN=/home/gsc/.cache/glassmind/continuous-20261010/groundingaudit-run/replay-new \
CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/groundingaudit-run/target \
bash tools/quantities/grounding-audit/reproduce.sh
```

Every cargo/native invocation is bounded by systemd 8 GiB/no swap, `setsid --wait timeout --kill-after=10s 600`; cargo uses `-j1`, tests use `--test-threads 4`, and TMPDIR is persistent. No training occurs. Full per-row data, original assets, helper source/binaries, command record, logs and outputs are frozen under `../groundingaudit-run/frozen-candidate`; its SHA256 manifest is referenced by `freeze-receipt.json`. All owned job handles completed. Only new `tools/quantities/grounding-audit/` files are uncommitted in the worktree.
