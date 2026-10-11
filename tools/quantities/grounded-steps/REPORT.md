# GSM8K TRAIN grounded-step data audit: rejected for continuation

One frozen conversion policy; no training, inference, DEV, official test, network downloads, or production changes. Base repository revision `2e44f256a95835e5836c78f04950c555db0a1c3f`; changes are uncommitted and limited to this new directory. No cargo invocation was needed.

**354 / 7473 (4.74%) uniquely grounded programs**, below the frozen **1000** threshold. All 354 executed through the unchanged original native executor, matching the exact final rational and every original number span. These are converted **gold-provided TRAIN programs**, not predictions. Numerical consistency and structural uniqueness do not establish English semantic correctness. Independent coordinator contracts were inaccessible and remain pending; the numerical coverage gate already fails.

## Results

| Indicator | All TRAIN | TRAIN-FIT | TRAIN-CAL |
|---|---:|---:|---:|
| All rows | 7473 | 5922 | 1551 |
| Annotation present | 7378 | 5848 | 1530 |
| Annotations parse | 7037 | 5578 | 1459 |
| All annotated arithmetic valid | 7037 | 5578 | 1459 |
| Last expression matches final answer | 6693 | 5292 | 1401 |
| Exact native binder accepted | 3874 | 3071 | 803 |
| Uniquely grounded typed + exact native execution | 354 | 290 | 64 |
| Ambiguity observed | 3025 | 2397 | 628 |

Stages are reported separately, not one monotonic funnel. All 7037 parsed annotation sequences have exact valid arithmetic; 344 do not end at the final answer. This does **not** establish that the dataset answer is wrong: the annotated sequence may omit a final calculation. We reject rather than complete or correct labels. Arithmetic/final consistency are not evaluated on 436 rows; ambiguity is not evaluated on 3852 rows with earlier binding/parsing/evaluation exclusions. Full per-stage true/false/not-evaluated counts are in the external JSON report.

| Exclusion (overlapping) | All | FIT | CAL |
|---|---:|---:|---:|
| ambiguous_leaf | 3025 | 2397 | 628 |
| cyclic_reference | 5 | 4 | 1 |
| impossible_forward_reference | 11 | 9 | 2 |
| missing_annotation | 95 | 74 | 21 |
| native_binder_refusal | 3599 | 2851 | 748 |
| overlong_program | 611 | 498 | 113 |
| unbound_leaf | 732 | 567 | 165 |
| unsupported_expression | 341 | 270 | 71 |
| wrong_final_value | 344 | 286 | 58 |

No supported row had wrong intermediate arithmetic, arithmetic overflow, division by zero, native gold-execution refusal, or span/value mismatch. Those refusals are exercised by authored contracts. Native binding failures: 3330 number-word refusals, 258 numeric spelling/boundary refusals, 7 separated/Unicode sign refusals, and 4 Unicode numeral refusals. No binder changes were made.

Ambiguity means more than one equal-valued **structural source**, even if all choices execute to the same number. All alternatives are retained per leaf; syntactic child expressions have fixed parent edges. Overlapping row-level causes: constant/input 2809; constant/prior-step 1385; repeated inputs 709; input/prior-step 473; repeated prior-step values 158. There are 2183 rows with ambiguity as their only recorded exclusion. Neither choosing the first source nor favoring question inputs is permitted.

| Emitted operator count | Rows | Accepted |
|---|---:|---:|
| 1 | 260 | 15 |
| 2 | 1687 | 173 |
| 3 | 1766 | 101 |
| 4 | 1315 | 34 |
| 5 | 879 | 25 |
| 6 | 519 | 6 |
| 7 | 317 | 0 |
| 8 | 163 | 0 |
| 9 | 76 | 0 |
| 10 | 34 | 0 |
| 11 | 11 | 0 |
| 12 | 7 | 0 |
| 13 | 1 | 0 |
| 14 | 1 | 0 |
| 15 | 1 | 0 |
| None | 436 | 0 |

| Native input count | Rows | Accepted |
|---|---:|---:|
| 0 | 1 | 0 |
| 1 | 96 | 34 |
| 2 | 605 | 109 |
| 3 | 1346 | 143 |
| 4 | 939 | 51 |
| 5 | 429 | 14 |
| 6 | 260 | 1 |
| 7 | 99 | 1 |
| 8 | 57 | 0 |
| 9 | 20 | 0 |
| 10 | 14 | 1 |
| 11 | 3 | 0 |
| 12 | 2 | 0 |
| 13 | 1 | 0 |
| 14 | 1 | 0 |
| 15 | 1 | 0 |
| None | 3599 | 0 |

`None` means unavailable after a prior exclusion, not a zero count.

| Operator | Rows containing operator | Accepted rows | All parsed nodes | Accepted nodes |
|---|---:|---:|---:|---:|
| add | 4257 | 208 | 7690 | 312 |
| divide | 3381 | 139 | 4551 | 169 |
| multiply | 5322 | 200 | 9922 | 259 |
| subtract | 3280 | 191 | 4332 | 221 |

FIT/CAL crossed with step count, input count, and operator presence are retained in `evidence-verification.json`. Operator row counts overlap; node totals count each emitted operation.

## Frozen policy and implementation

`PLAN.md` and `contracts.json` were frozen before conversion. The parser is a bounded recursive-descent grammar for exact `+ - * /`, unsigned decimals, and parentheses. No unary expression signs, variables, eval/exec, symbolic solver, simplification, gold-derived constants, or text-derived entity roles. Arithmetic mirrors the supplied checked i128 rational operations. All annotation trees expand in order by left-first postorder, with a 1..6-step acceptance horizon. All literal alternatives include native question occurrences, previously completed expression nodes, and the exact 20 native constants. Unbound/future/cyclic references are excluded. Full alternatives use a compact Cartesian representation; ambiguity is never resolved by choosing a first match.

Question text is unchanged for native calls. Lowercase/whitespace-collapsed questions with ASCII digit runs replaced by `#` are SHA256-grouped; full-digest modulo 5 == 0 is CAL. 7472 normalized groups, no group crosses FIT/CAL. Both subsets are official TRAIN and no independent generalization claims apply.

`audit.py` produces one record for every source row, membership hashes, question-only binding requests/outputs, separately named gold-provided execution requests/outputs plus row mappings, and all exclusion counts. `verify_evidence.py` checks memberships, question isolation, UTF-8 spans, every candidate set, accepted execution links, and the fixed sample/full match. `reproduce.sh` runs the bounded workflow without compilation.

## Validation and preserved failures

- Frozen authored contracts: 19/19 passed initially and 19/19 with final converter source. They include all requested positive/collision/refusal cases, nested expressions, overflow, and UTF-8 spans. These are self-contracts, not coordinator contracts.
- Fixed first 200 official TRAIN rows: 5 uniquely grounded exact programs. Full 7473: 354. All first-200 row records and native binding outputs match the full run exactly. No conversion-policy changes followed sample inspection.
- All 7473 question-only requests match source questions byte-for-byte; all 354 accepted requests/outputs match programs, final values, and complete native number records. Every recorded leaf candidate set was independently reconstructed by the evidence checker.
- Evidence checker initially used Unicode `str.splitlines()`, which incorrectly split a JSON string containing a Unicode line separator. Original checker and reproduced traceback are frozen as `verify-evidence-original-failed.py` and `verify-original-failure.log`. Fixed to iterate physical JSONL lines; recount passes. Converter records and corpus outputs were unchanged.
- A pre-corpus span-check correction allowed the original binder's ASCII `+` numeric literal when validating spans; annotation grammar remained unchanged. Initial contract source is retained separately. No failed experiment or native refusal was discarded.
- Rejected experiment: this sole frozen GSM8K conversion pipeline fails the >=1000 coverage gate. No retuning, second configuration, or CAL search was attempted. Historical MathQA failures in PLAN are supplied context, not a GSM8K baseline.

## Provenance and reproduction

Source: `openai/grade-school-math`, revision `3101c7d5072418e28b9008a6636bde82a006892c`, MIT (OpenAI copyright 2021). Official `train.jsonl`: **7473 rows, 4166206 bytes**, SHA256 `17f347dc51477c50d4efb83959dbb7c56297aba886e5544ee2aaed3024813465`. LICENSE: 1062 bytes, SHA256 `86bbb73e855821d7c401912fd4bf82e34313e6e3b6fd6f909f2b6cc9e209a53b`. No new download or data redistribution in the repository.

Original executor: 16919992 bytes, SHA256 `40e6a6e975cf8474231ac0e3bcc2e3df5c1ce8196dead0b9159586aa5935ea6f`. All six supplied file hashes and byte lengths plus the supplied manifest hash were verified before and after corpus execution; exact original source and binary are unchanged. Full native source hashes are retained in `full/report.json`. No parent artifacts, hidden contracts, official test, or old private logs were read.

External evidence: `/home/gsc/.cache/glassmind/continuous-20261010/groundedsteps-run/frozen-candidate`. It contains the policy/contract freeze hashes, converter snapshot, complete per-row records, sample/full reports, native requests/outputs, commands, source verification, preserved checker failure, and a SHA256/byte manifest for every artifact other than that manifest itself. Large data and outputs remain outside the repository.

From the repository root (choose a fresh external output directory):

```bash
bash tools/quantities/grounded-steps/reproduce.sh \
  /home/gsc/.cache/glassmind/continuous-20261010/groundedsteps-source \
  /home/gsc/.cache/glassmind/continuous-20261010/groundedsteps-run/reproduction
```

Each invocation uses `systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600`. The wrapper sets persistent `TMPDIR`; no cargo was invoked. The supplied binary is required; no build, runtime model, model calls, services, or background jobs are added.

## Recommendation and limitations

Reject continuation of this supervision pipeline under the frozen gate. The strongest measured bottlenecks are unchanged number-word refusal and unresolved source identity, especially native-constant collisions. A materially different next direction would be separately authorized, explicitly span- and step-reference-annotated supervision with human-checked semantic links and independently specified number-word support—not another operand-ranker configuration or a value-priority rule. Those changes require their own task and gates; none is implemented here.

The 354 records remain useful auditable numerical examples, but are not semantically proven and do not meet the readiness threshold. Conservative grammar, native binder restrictions, all-annotation expansion, and the six-step horizon intentionally limit coverage. Independent coordinator contracts and semantic review remain incomplete. No production model or solver accuracy improvement is claimed.
