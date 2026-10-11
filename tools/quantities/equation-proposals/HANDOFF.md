# Rejected: finite TRAIN-supervised equation proposals

Completed one fixed fit and both declared lexical comparisons from base
`c9d10dc1a1348b757be3ea96db3ee600674e155d`. **Do not integrate the learned candidate.**
The raw model loses to lexical retrieval and no calibration selection threshold
qualifies. Runtime changes were archived and rolled back; the only remaining
uncommitted addition is this owned evidence directory. No commits were made.

## Exact changes and disposition

`rejected-runtime.patch` preserves the complete experimental implementation:
- New `crates/quantities/src/proposal.rs`: numeric literals with UTF-8 byte spans,
  a typed finite program graph, checked repository Rational arithmetic, validated
  sparse linear centroid model, and `Model::propose(question)` returning only
  `proposed`/`refusal`, with `semantics=unverified`, score and model provenance.
- Opt-in standalone `--propose-model MODEL QUESTION` / `--propose-model MODEL
  --jsonl` (JSON strings only), plus the library export. No shared main CLI or talk
  change. Default `solve()` and parser were unchanged throughout.
- Offline `equation_executor` example for annotation/baseline execution, and four
  meaningful boundary/executor/model tests. Neither Python nor retrieval is used
  by the runtime proposer.

The patch is review/replay evidence, not an accepted feature. `git apply --check`
passes against the restored base. All crate changes, including new files, were
removed only after their exact patch and source were retained externally.
`experiment.py`, `contracts.py`, `audit_saved.py`, `reproduce.sh`, frozen PLAN,
METRIC, reports, logs and manifests remain here. No existing scorer, fixture,
report, README, shared manifest, main CLI, loop or historical prompt changed.
No dependencies or Cargo.lock changes.

## Preregistered design and split

PLAN SHA256: `b11ffa00682002b50f82ff788aad3494949e7fee3d72f213ffaf6d14a6026303`.
Frozen before fitting/calibration/DEV. Normalize lowercase question whitespace,
replace ASCII digit runs with `#`, SHA256 whole normalized question, modulo5==0
calibration; other groups FIT. All official TRAIN rows participate in splitting.

| Quantity | FIT | Calibration |
|---|---:|---:|
| All rows | 23912 | 5925 |
| Disjoint normalized groups | 9521 | 2346 |
| Executable annotations in candidate bank | 5180 | 801 |
| Executable, exact-label-agreeing annotations in candidate bank | 3303 | 348 |

FIT cleaning retained 4590 rows across 1565 generic programs. Exclusions: 8522
unsupported annotations, 3989 numeric-binding/execution refusals, 2470 wrong
annotation results, 4341 unscorable annotation results. This is a supervision
filter, not solver accuracy. Candidate bank: >=3 cleaned FIT examples per class,
512-class cap, deterministic count/canonical-program ordering; 3303 examples fit
all retained classes. The other 1287 cleaned examples remain available to the
unrestricted baseline. No cherry-picked subset or second fit.

All 5925 calibration rows were inferred/scored, including unsupported groups.
3796 calibration annotations meet the parser family, 2811 execute, and 1182
have exact numeric-label agreement across the unrestricted annotation family.
The 348 candidate-bank figure is a post-fit descriptive coverage audit, not a
filter on calibration inference. Exact predicted-vs-annotated program agreement
was not measured. No DEV formula, rationale or category was read.

Model: binary word-unigram/bigram and numeric-slot-count features, FNV1a64 modulo
8192, FIT-only IDF, normalized TF-IDF document centroids normalized per class;
cosine argmax over the finite 512-program bank. No unseen-program synthesis.
The lexical baseline uses word-unigram TF-IDF cosine nearest cleaned FIT example,
with both all1565-program and restricted512-program banks. Its raw training
questions stay offline and are not a runtime asset. The model contains only IDF,
centroid weights, generic typed programs and provenance, no examples or IDs.

Bounds: 4096 question bytes, 16 numeric slots, 1..6 steps; declared 20 exact
constants (see PLAN), no pi or approximate conversion constants. Model loader
bound is 32MiB (33,554,432 bytes); the actual model is 1,151,545 bytes, below both
32MB and 32MiB. A strict decimal-32MB interpretation would require tightening
that loader limit before any future integration. Input spelling checks are
conservative; number words, Unicode numerals/signs, scientific notation,
malformed numeric boundaries, invalid refs, overflow and zero division refuse.
Fraction components retain separate slots. Arithmetic validity never proves
English entailment or unit correctness.

## Results: distinct proposal-numeric-label-v1 metric

See METRIC.md: the frozen comparator is unchanged, but its offline sentinel
mapping is explicitly a proposal metric. All4475 rows remain in the denominator;
unscorable results count wrong conservatively. Sample is first200 by question
SHA256 order. Its output shape was checked before full inference; label scoring
of both saved sets occurred afterwards. No sample-driven edits or tuning.

| Full4475 raw | Correct | Refused | Wrong | Unscorable | Conservative wrong |
|---|---:|---:|---:|---:|---:|
| Candidate | 499 | 1878 | 1179 | 919 | 2098 |
| Unrestricted lexical | 663 | 1747 | 1132 | 933 | 2065 |
| Restricted lexical | 518 | 1850 | 1183 | 924 | 2107 |

Sample200 candidate22/83/48/47; unrestricted32/76/44/48;
restricted22/82/50/46 (correct/refused/wrong/unscorable).

| All5925 calibration raw | Correct | Refused | Wrong | Unscorable |
|---|---:|---:|---:|---:|
| Candidate | 376 | 2411 | 1941 | 1197 |
| Unrestricted lexical | 499 | 2288 | 1910 | 1228 |
| Restricted lexical | 374 | 2378 | 1977 | 1196 |

Independent selection thresholds: **all disabled**. Best attainable calibration
precision with >=30 selections was candidate111/211=52.6066%, unrestricted227/524
=43.3206%, restricted50/98=51.0204%; none reaches90%. These descriptive frontiers
are not substituted thresholds. Full DEV selected results for every method:
0 selected,0 correct,0 wrong,4475 unselected,precision undefined.

Gate1 FAIL: 499 meets250 but trails the stronger baseline663 by164; it needed688
for the required +25. It also trails restricted baseline518. Gate2 FAIL: no valid
calibration threshold, zero selected correct rather than>=50, no +10 gain, and
undefined rather than>=80% precision. Gate3 PASS as detailed below. Gate4 remains
for independent parent review; no aggregate result can waive bounds/semantic
issues. Six new authored semantic contrasts obtained **0/6** numeric matches:
four executable wrong proposals and two missing-slot refusals, with identical
predictions across addition/subtraction and pack-count/usable-count contrasts.

## Validation and preserved baseline

16 scoped tests pass (12 existing +4 new); crate fmt, all-target clippy with
`-D warnings`, release bins/examples build and `git diff --check` pass. Initial
clippy found seven static-regex-expect/indexing warnings; fixed before the fit,
with first and corrected logs retained. This was one model setting, not a search.
CLI probes verified annotation-object rejection, oversized JSONL/input/slot
refusals, Unicode byte-span retention and explicitly unverified status. Saved
source/model/binary/prediction manifests verified after completion.

Default solve JSON is byte-identical to supplied baseline executable
`5a94dba0b893fafed21906c2150e51de7fab94c67cadefba10008b2a187e4b9f`:
- sample200 output SHA `dc6b9f8d10bcf64c23295e666e67edd63f10616ad9068b9b34458a87aa0a85bf`
- full4475 SHA `5cf77234ace536a7f147c3a1bc4ac832a1052e2ac50c05ea232a1ce4a175eb98`
- synthetic3116 SHA `ab818fdee8b4b383c8e0439dae83d9136926c6a5b4558c66033dfff4dac895ab`

Old full score remains **29 correct /4443 refused /3 wrong**. Comparison used
whole JSONL bytes, stronger than just status/value/unit. No regression waiver.
No workspace-wide suite or parent integration is claimed.

## Frozen artifacts and provenance

External frozen directory:
`/home/gsc/.cache/glassmind/continuous-20261010/equationproposals-run/frozen-candidate`.
Large assets remain outside git. Source and executable manifests precede scoring;
prescore manifests cover every prediction file. Original manifests are retained
unchanged; supplemental submission manifest covers subsequently added audit/tests.

| Artifact | SHA256 |
|---|---|
| Model | `5594aebe9ab9f29cb4e017a380a9119c3603bb2e4e85e272337f028c93f2abc6` |
| Proposal executable | `f7b728e30ccdcce5f1b99b1d185f197e13fa6d2914cfe5b866376d3ac678fa4e` |
| Offline executor | `d5f54ab5d311736aa6863509f2cbb5d6c007f934845cd98b92bc94809702f24a` |
| Split membership/group hashes | `b16bce2866326a9fe20438f88fca0631d11d62019a75fb2189c9dca6e01411e6` |
| Prefit source/binary manifest | `5168fdb0ba5fca95b40b843715e6b821ed794a0d7858ae8fe9540ad0a34cce3f` |
| Full candidate predictions | `1464b543b3fee58994021d2a2a925d34819e408a5bfaa7b0b14ba2fe14610fb6` |
| Full unrestricted lexical | `6ed3d8acba275e3432364977d5ae9a32d4474f71c4f2d8ad91c61677fde5734b` |
| Full restricted lexical | `99b3163344eecd8620560d44f7b480258e8dea20744f813cd58e72c21cb3ed77` |
| Rejected runtime patch | `17178663a04dc1eff9546138ace59649a408c2cdac01d4fe8f663573920b6d09` |

Existing MathQA read-only source: revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0 per supplied source-card.
TRAIN29837:9,013,733bytes SHA
`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
Validation4475:1,350,141bytes SHA
`488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20`.
Source-card7,438bytes SHA
`8051c9a1e42245600985ff1b890dfe3ecce804c47555d83ca7a02a2e3f4b4cc2`.
Total10,371,312 existing source bytes; **new downloads/data/dependency installs0**.
Offline Python used existing read-only PyArrow23.0.1 and standard library, no numpy.
Derived model provenance inherits the supplied Apache-2.0 TRAIN source; no claim
of a separately licensed pretrained model. Runtime code uses existing Rust deps.

## Reproduction and limits

For evidence-only verification, set the variables below with PROPOSAL_RUN pointing
to the retained frozen-candidate, then run capped `audit_saved.py`. For an explicit
research replay, use an isolated checkout at the base, apply the rejected patch,
and choose a NEW empty external PROPOSAL_RUN sibling of the supplied baseline/:

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/equationproposals-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/equationproposals-run/target
export PROPOSAL_SOURCE=/home/gsc/.cache/glassmind/continuous-20261010/equation-source
export PROPOSAL_RUN=/home/gsc/.cache/glassmind/continuous-20261010/equationproposals-run/replay-new
export PYTHONPATH=/home/gsc/.cache/glassmind/continuous-20261010/quantities-run/tmp/quantities-deps
git apply tools/quantities/equation-proposals/rejected-runtime.patch
bash tools/quantities/equation-proposals/reproduce.sh
```

Every cargo invocation uses persistent TMPDIR, existing CARGO_TARGET_DIR,
`systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout
--kill-after=10s 600`, `-j1` except fmt, and tests `--test-threads 4`. Training,
inference, contracts and saved audit use the same cap/timeout. Outer scope
MemoryMax8589934592/MemorySwapMax0 was confirmed. The single fit/full evaluation
completed; no owned jobs remain. No agents, services, LLM calls, credentials,
API keys, hidden/test items or old private logs were used. Inherited model/login
selection cannot be independently attested or switched by this worker.

The experiment is rejected, not deployed. Program-bank coverage and lexical
features miss argument roles, direction, negation, and units; number-word refusal
also limits coverage. Exact coefficient matching is not English correctness.
Official DEV can contain related questions to TRAIN; only the requested TRAIN
fit/calibration grouping was enforced. No representation rescue, threshold
weakening, second fit, second full proposal inference or DEV-label tuning occurred.
