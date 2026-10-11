# TRAIN-selected finite Wikidata index: rejected expansion

Coordinator review independently verified 101 provenance hashes, recomputed
the 15,778-row TRAIN scope selection, checked nested index identity, recomputed
all 2,821 outcome transitions and checked standalone/talk answer-ID equality
for each scope. All five offline preparation tests pass. The recorded fetch
stayed within its limits. [Parent review](reports/parent-review.json) records
these checks; no second network or complete inference run is claimed.

The original twelve worker files (238,431 bytes) were archived and hash-verified
before integration and again before removing the completed worktree. The
temporary login link was removed. Raw responses, experimental bundles and all
predictions remain under the run root below. No new index becomes the default.
The original worker handoff follows.

One bounded batch, no commit. Only `tools/facts/trainindex/` is new. Source
`4a05e4cbd53209a1666f8335e2b974ab19aa6da1`; no runtime Rust, CLI, scorer, model,
shared manifest, historical index, README, or other track was changed. No agents,
LLM calls, credentials, API keys, held-out data, hidden tests, private logs,
services, or persistent background jobs were used. All owned subprocesses ended.

**Reject default replacement.** Expanded correct answers increased by three, but
answered precision fell by 0.0271406728, exceeding the preregistered 0.02 allowance.
Retain both bundles as opt-in experimental evidence. The historical 500-entity
validation-seeded series remains untouched; none of the numbers below compare
against that series. This is TRAIN-scoped finite retrieval, not open-world QA.

## Selection and independence audit

[PLAN.md](PLAN.md) was written before validation access and before fetch. Exact
interpretation of “first 90% TRAIN”: first `floor(0.9*N)` rows, then exclude
zero-based `i % 10 == 0`, the frozen relation model's calibration membership.
This is explicitly a prefix AND a calibration exclusion, not all 90% fit rows.

- TRAIN: 19,481 rows; prefix ends at exclusive index 17,532.
- Eligible fit rows: 15,778; calibration excluded inside prefix: 1,754.
- Final TRAIN tail excluded: 1,949 rows. Original model calibration total: 1,949.
- Count only subject QIDs; order descending frequency, then ascending SHA256(QID).
  Freeze 1,000 distinct QIDs and nested first100 before fetching.
- TRAIN relations, objects and questions never become claims or aliases.
- Frozen selection JSON SHA256:
  `77208912a8fbf86efc3aafa707495c6c5f1ecde2f07390d9a37d2ece637f40af`.
- Newline-terminated QID-prefix hashes: first100
  `e4af71e5d6added53d8fc58d8c7122e41101c5d0a6fb733452239282edb638e9`;
  all1000 `4adfd2617a09e84d9ad97cbed28f44b18f55a4291dc9a24424242bcc8692c65a`.

28 exact selected entities were reused from existing raw Wikidata responses.
Only selected IDs become searchable entities; no historical index was imported.
Baseline has **100/100** IDs; expanded has **999/1000**. Missing `Q5967378`
redirects to `Q9326077`; it was omitted, not substituted or backfilled. The first
100 entities and their claims are identical in both indexes.

## Development results (unchanged scorer)

Acceptance required more right answers, answered precision no worse by >0.02,
exact standalone/talk answer-ID equivalence and a working relocated bundle.
The fixed first200 evaluation ran before full2821, without subsequent tuning.

| Scope | Dev rows | Right | Refused | Wrong | Answered precision |
|---|---:|---:|---:|---:|---:|
| Declared first100, actual100 | 200 | 3 | 193 | 4 | 42.8571% |
| Declared top1000, actual999 | 200 | 3 | 191 | 6 | 33.3333% |
| Declared first100, actual100 | 2821 | 44 | 2725 | 52 | 45.8333% |
| Declared top1000, actual999 | 2821 | 47 | 2712 | 62 | 43.1193% |

Full transition matrix (every row retained):

| Before → after | Count |
|---|---:|
| right → right | 44 |
| right → refused / wrong | 0 / 0 |
| refused → right | 2 |
| refused → refused | 2705 |
| refused → wrong | 18 |
| wrong → right | 1 |
| wrong → refused | 7 |
| wrong → wrong | 44 |

Sample transitions: right→right3, refused→refused191, refused→wrong2,
wrong→wrong4; all other cells zero. No denominator exclusions. Full entity-link
correct totals150→193; relation top1 correct2664 unchanged. All2821 standalone
and talk answer-ID sets agree exactly for each scope; likewise both samples.
Each runtime inference process receives only question text, with fresh talk
context per question. Offline annotations are separate; unchanged `score`
independently verifies totals. Full per-row records, questions, predictions and
hashes are retained outside git and indexed by [reports/evidence.json](reports/evidence.json).

All **18 newly wrong** rows are scrutinized in that report:

- Inverse locally unique outputs:154,666,747,849,1143,1572,1661,2357,2371,2551.
  Nine link the annotated subject;2357 picks Paris, France instead of the
  annotated Paris locality. Some answers are real alternative members of a
  broad category, but exact benchmark answers still count wrong. Additional
  wording constraints can also be missed (album vs band, pastor vs footballer).
- Wrong-subject forward links:190,618,721,761,861,2359,2678,2757. Examples:
  “Monster Movie” links Monster, “Eat Pray Love” links Love, “wake-up” links Jan
  Wyck through the actual alias “Wake”, and “Rudy Carpenter” links a different
  Carpenter. No aliases were patched in response to dev failures.
- No earlier-correct losses. Six old wrong inverse answers become refusals
  because expanded local results are multiple; one more wrong answer becomes
  an alias-ambiguity refusal. Local uniqueness is not global uniqueness.
- Three new scored-correct rows:572 (Mera Shikar country),1780 (James Develin
  citizenship),2190 (George Butler gender). **2190 links the wrong person**
  (George Edmund Butler), coincidentally sharing the expected gender. This is
  recorded as scorer-right, not claimed as a verified subject-level improvement.

## Exact changes and checks

- `prepare.py`: deterministic subject-only selection; sequential bounded public
  fetch; exact-QID extraction; offline replay bound to recorded raw hashes;
  nested indexes and index-bound display labels. Incomplete/error replies create
  no claims. Rebuild refuses modified raw contents and ignores added cache files.
- `evaluate.py`: new-series adapter invoking existing score unchanged; separate
  question-only standalone/talk streams; complete outcomes and hazards.
- `check_bundle.py`: relocation, four corruption/schema rejection checks and five
  independent fetched-fact prompts fixed before dev access.
- `test_prepare.py`: five focused offline tests: calibration/tail/non-subject
  exclusion, rank/qualifier semantics, exact-QID cache filtering, incomplete
  response handling, frozen cache membership/content checking.
- `audit.py`, `reproduce.sh`, preregistration and reports provide replay/evidence.

Release build passed with existing target, TMPDIR, `-j1`, 8GiB/0swap scope and
600-second process-group timeout. Five Python tests passed; shell syntax passed.
No giant suite or unrelated Rust test suite was run. Rebuilding all four
index/label files is byte-identical. Bundles were made with unchanged
`tools/facts/chat/package.py`. Relocated talk output is byte-identical from a
new working directory. Index/model corruption, labels binding mismatch and
unsupported manifest version each reject loading.

Independent end-to-end probes: Diane Keaton birthplace→Los Angeles;
Steven Soderbergh birthplace→Atlanta; Thomas Edison citizenship→United States;
Jean Cocteau death place→Q549750; all correct. “What is the capital of Goa?”
refuses due to uncertain learned relation. **4/5**, separate from benchmark score.

Rejected experiments: the only scoring experiment was frozen100→1000, rejected
above. During replay hardening, initially freezing only claim-bearing cache files
lost two baseline/nine expanded display labels; a separate frozen display-cache
ledger restored the original bytes. No scored predictions changed. No entity
reselection, model tuning, claim-filter changes or second expansion was tried.

## Sources, bytes, provenance and limitations

SimpleQuestions source `askplatypus/wikidata-simplequestions`, revision
`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`, **CC BY 3.0**. Existing TRAIN
1,193,868 bytes (SHA256 `25c5994c4e3a4dad39bacf3c457b3ff4cf7b199630eb78045a0b478e4fda145b`)
and valid173,187 bytes (SHA256 `1dffa967f15f2cb73ba7a50d3545f9585ef76b8722322e569cf54c9f9a1a9a48`)
were reused. No SimpleQuestions download or test access. Frozen model unchanged;
its existing provenance records17,532fit/1,949calibration rows, not this smaller
scope-selection prefix. No new model or dependency.

Wikidata structured data: **CC0-1.0**, public official API. Parent-checked
[Data access](https://www.wikidata.org/wiki/Wikidata:Data_access) and
[Licensing](https://www.wikidata.org/wiki/Wikidata:Licensing). Identifying UA,
gzip request, maxlag5, sequential≤25 IDs/request, minimum spacing2.000031seconds.
Fetch2026-10-11 05:30:08.863176–05:31:25.551019UTC: **39 requests,
76.6877seconds, 67,094,727 new decompressed bytes**. No retry/rate-limit failures.
Within80requests/600seconds/80MiB. Fourteen reused claim-source cache files total
20,475,085bytes; all20 existing raw cache files total27,696,032bytes supply only
eligible display labels. Their other entities never enter the index.

**Numeric Wikidata revisions are unavailable for this batch:** the original
request omitted `props=info`, and the reused raw cache also lacks revision fields.
Exact response SHA256, byte counts and new request timestamps are retained in
[fetch.json](reports/fetch.json); old fetch timestamps are unavailable, with
filesystem mtime explicitly distinguished. [rebuild.json](reports/rebuild.json)
records revision fields as null rather than inventing them. Final reusable fetcher
now requests `info` and binds selection hash at fetch time; these bookkeeping
changes were not network-retested. No second fetch was made to disguise the gap.
The initial ledger lacks the new selection-binding field; the preregistered
selection hash and frozen prefix hashes above bind this run's evidence. The
separate display cache ledger was frozen during replay hardening after dev;
replay confirms original index/label bytes unchanged. Content-addressed cached
rebuild works; this is not a promise that live refetch reproduces a snapshot.

Inherited extraction skips deprecated statements, includes all nondeprecated
entity-value QIDs, and unions normal/preferred ranks. Temporal qualifiers,
references and preferred-rank truth semantics are not applied. Historical and
current values can coexist and cause refusals; inverse uniqueness is only local.
The relation model and alias linker are frozen and demonstrably fallible. Labels
are display-only and incomplete; missing labels remain IDs.

## Retained assets and exact identities

Run root: `/home/gsc/.cache/glassmind/continuous-20261010/trainindex-run/tmp/trainindex`.
Raw responses, selection, indexes, bundles, complete transitions and predictions
remain there, outside git. Reports record every retained artifact hash/size and
all runtime/adapter source hashes. Index100:89,148bytes; index999:650,045bytes.

```text
source commit 4a05e4cbd53209a1666f8335e2b974ab19aa6da1
model (35,773,124 bytes) db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6
index100 ecd5c8095fcfbcd03e8b6ea5e6373c393937f93fddfab1cc14174954a8c93cb7
index999 142f62f8abeec20f0b01bb5361b5e536507d21781c77773e96ca87091aed5791
facts binary 076f5f89d620d4e6c4521d45799f85c585a9c6fb54004e74540c5c5ad37b4a3d
question-only predictor 1e5a4af21b05024c85c6e3173f782cd6d62c3a5ed27259fe642aeb30c56eb433
talk binary d199ecfbe0d52692fe35d058de523ba4883a7c99e90679f5ea933c180784a1f1
unchanged historical500 index 923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185
```

## Reproduction

From this worktree, with retained raw assets (default replay is offline):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/trainindex-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/trainindex-run/target
bash tools/facts/trainindex/reproduce.sh
python3 tools/facts/trainindex/audit.py --run "$TMPDIR/trainindex" --output tools/facts/trainindex/reports
"$CARGO_TARGET_DIR/release/glassmind" talk --facts-dir "$TMPDIR/trainindex/bundle1000"
```

The script runs exact selection verification, cached rebuild, capped release
build, focused tests, unchanged packaging, relocated-bundle checks, fixedfirst200
and full2821 evaluations. No cargo subprocess exists in Python adapters. Its
cargo command is exactly:

```bash
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 cargo build --release \
  -p glassmind -p glassmind-facts --bins --example continuous_predictions -j1
```

For a genuinely new live snapshot only, choose a new `RUN` and explicitly set
`FETCH=1`; do not use that to reproduce these exact data or reset this batch's
network budget. Fetch refuses an existing ledger. Existing default/historical
facts behavior remains unaffected; coordinator handles integration and any
independent validation. This batch does not meet expansion acceptance, and
numeric revision provenance is incomplete as disclosed above.
