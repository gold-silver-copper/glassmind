# HANDOFF — REJECTED, evidence retained; runtime restored

One bounded batch at `412f11eaedbf4d69e4d084a94cfe283762a515ed`.
No commit, agent, model call, service, download, API key, hidden asset, benchmark
run, scorer change, expectation edit, shared manifest or other worker file change.
Both original rejected archives remain untouched. The archived patch was reused
only after freezing PLAN and contracts. This experiment does not accept either
older rejection retroactively.

## Decision and measurements

Reject under the unchanged new all-contracts gate. The exact candidate is useful
for review in `rejected.patch` and `rejected-source/`, but must not be integrated
as accepted runtime. Source was archived and verified before restoration.

| Scope | Baseline / candidate |
| --- | --- |
| Original TRAIN fixed sample | 0/1 / 1/1 test-verified |
| Full permitted original TRAIN | candidate3/60 verified,57 unchanged refusals |
| Original authored sample/full | candidate3/3;20/20 (6 implementations14 refusals) |
| Original parser / actual CLI | candidate34/34 /34/34 |
| Original six environment cases | candidate6/6 |
| New frozen boundary CLI gate | baseline36/46; candidate36/46 |
| New exact parser dispositions | candidate44/46, including target/TMPDIR-unset runs |
| Supplemental isolated query/rename | baseline4/4; candidate4/4 |
| Scoped Rust tests | 81 passed: CLI11, English39, world27, isolation4 |
| fmt check / affected all-target clippy / debug build | passed |
| Parent full765 / release | not run; local rejection |

TRAIN successes are original `rb-implement-sum-even` chatty,hurried,terse requests,
**one distinct function, three styles**. Original public tests ran opaquely and
independent cargo test passed. Every byte outside the placeholder and other files
was preserved. No expected patches, hidden tests or solution values were used.
Parser-only TRAIN coverage23/60 is not behavioral correctness. The trusted parent
174/591/0 result is unchanged; no177-pass claim or full retention claim is made.

The old strict CLI failure `repair x` now passes unchanged at the isolated native
baseline. Its old33/34 rejection stays rejected. All old fixtures and adapters
are byte-identical to the original submitted impllanguage archive; see
`reports/original-identities.json`. No evaluator predicates were relaxed.

## Failure analysis and rejected attempts

The new gate was frozen before runtime editing, but its positive assumptions
were faulty. Defining a function and field both named `implement` causes inherited
query-kind/rename ambiguity. The `implement_again` destination is not understood
by the inherited language. The authored macro-bearing source is unsupported for
declared types; the chosen AddField wording is unsupported. These five positives
fail in both ordinary and unset environments on baseline and candidate. They stay
in the gate, with original results, rather than being deleted or rewritten.

Additionally, the candidate's punctuation reservation sees the dot in
`Ledger.implement` as a sentence boundary: expected UNRECOGNIZED, actual REFUSED,
in both environments. CLI declared-type priority prevents interception there,
but the exact parser contract fails. This is a real remaining language-boundary
bug. No follow-up threshold change or second runtime candidate was attempted.

Supplemental isolated probes were added after observing the frozen fixture
failures, so are explicitly exploratory, not replacements. First versions used
`renamed`, which the baseline language did not parse:2/4. A separate preserved
script uses `calculate`:4/4 baseline and candidate, including actual successful
renames of both `implement` and `implemented`. Original scripts/results remain.
The supplemental runner's exit status is not a verdict: inspect its four `ok`
fields; `isolated.py` returns0 even for its two failed rename assertions. None of
these auxiliary results count toward the frozen gate. Successful original-name
baseline operations were thus demonstrated only after editing; that ordering
requirement was not fully satisfied, another reason this is not accepted.

## Exact changes and identities

World `implement.rs` is byte-identical in its entirety to the original impllanguage
submission (`reports/family.json`), including the candidate family, native tests,
finite exhaustion, multi-survivor refusal, budgets, rules, staleness and rollback.
Names/docs/English/test values never become code-generation features. The English
module replaces global implement/implemented word occurrence reservation with
leading action/negation, passive modal, declarative stub and punctuation/conjunction
clause checks. Parsing of accepted requests remains fully consumed. Quoted Unicode,
politeness, bounded pronouns and doc-reference caveats remain unchanged.
The same minimal16-line do hook follows answer-file and declared-type handlers,
before AddField and generic rename/repair; library exports only otherwise.

Candidate binary retained at WORK/implementationroutes-run/glassmind-candidate:
127584192bytes, SHA256
`1966eb5735d79540432335ac67438cc79336c3d370155764ecc878bc322ca4ca`.
Patch38549bytes, SHA256
`c6e6e9d10c3a6660ffd1862409bac1d824a8760fea4213ef4195c343453216e9`.
Exact five source, parser binary, baseline binary and patch identities are in
`reports/identities.json`. FROZEN.sha256 covers PLAN, original20 fixtures,
34 contracts,6 environment definitions,23 new boundary rows (46 environment
executions), and original TRAIN provenance. MANIFEST.sha256 covers submitted
artifacts. `reports/raw-identities.json` identifies1148 owned raw input/output/log
files outside disposable native caches. All actual evaluation result JSON files
are copied into reports; full runtime evidence remains in WORK/implementationroutes-run.

## Provenance and limitations

WORK=/home/gsc/.cache/glassmind/continuous-20261010. Baseline binary source1331ac7,
worktree412f11e; SHA2568cde5dc486025e05932db946ba4accb7b5401fcd27c530bc8539eb4dca5bb78b.
New local code/contracts use workspace MIT OR Apache-2.0, dated2026-10-11;
exact individual source versions/bytes/hashes are in data-provenance.json.
No downloaded data or model bytes. Original fixture3433bytes;
original language contracts1287bytes; new authored boundary source and supplementary
fixtures have exact lengths in that provenance manifest.
Existing TRAIN inputs:60 requests,5070UTF-8 request bytes;20 production snapshots,
40 Cargo/src files,17648bytes in inherited frozen provenance. The benchmark pin
f79e71a2a26fdaf15f9bc55225a038b61e66fafb and absence of an allowlisted source license
are inherited, not independently reverified or assigned a license. The source
snapshots were copied/hashed/run opaquely, not mined for assertions or bodies.
All60 copied original input snapshots and request metadata match the frozen
archive exactly (`reports/provenance-verification.json`). Only request metadata
was displayed. No additional production-body study occurred.

Every direct/new nested cargo ran under persistent disk TMPDIR,8GiB/0swap scope,
setsid --wait, outer timeout kill-after10s600,-j1,testthreads4 (fmt excepted).
Outer worker cgroup was verified8589934592bytes,swap0. Explicit owned scratch was
supplied to test subprocesses, including unset target/TMPDIR cases. Existing target
was retained; native namespaces stay beneath its cache or the owned disk scratch.
Inherited checks.rs has an additional inner deadline timeout with kill-after1s;
it was not changed under the ownership restriction. Its outer600s wrapper has
kill-after10s. Thus no claim that every inherited inner grace equals10s is made.
No default-HOME scratch run, hostile filesystem race/fake-test-output/crash-atomic
restore/disk-full fault injection, or broad arbitrary-English proof is claimed.
Reservation of actions after conjunctions also needs broader query-list scrutiny.
No unchecked live edits or relaxed assertions. Runtime restoration was bytewise
against HEAD after archiving only owned files. No owned process survives handoff.

## Reproduction

From repository root, retained candidate replay (expected final exit1):

```bash
RUN=/home/gsc/.cache/glassmind/continuous-20261010/implementationroutes-run
export TMPDIR="$RUN/tmp" CARGO_TARGET_DIR="$RUN/target"
bash tools/coding-implementation-routes/reproduce.sh "$RUN/replay-new"
```

The replay wrapper assembles commands run separately; no second consolidated
replay is claimed. To rebuild in a disposable412f11e checkout only:

```bash
git apply tools/coding-implementation-routes/rejected.patch
export GLASSMIND_SCRATCH="$RUN/rebuild-scratch"
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt --all -- --check
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test --offline -p glassmind-world-rust -p glassmind-english-rust -p glassmind -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy --offline -p glassmind-world-rust -p glassmind-english-rust -p glassmind --all-targets -j1 -- -D warnings
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build --offline -p glassmind -j1
```
