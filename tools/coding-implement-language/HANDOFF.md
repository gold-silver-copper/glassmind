# Handoff — REJECTED, runtime restored

One bounded prospective batch at base `2be77e092f5337d9d10d5bd024b3d0a35ef66b5a`.
No commit, agent, model call, service, dependency, download, held-out access or
scorer/manifest/history edit. The previous implementation candidate remains
**REJECTED, 0/60 request coverage**. Its original archive was read-only and the
patch was applied only after this experiment's PLAN. No retroactive acceptance.

**Decision: reject under the unchanged all-contracts gate.** New parser contracts
pass34/34, but the stronger actual CLI/no-environment suite passes33/34. Its sole
failure is `repair x`: implementation parsing correctly returns no implementation
action, then the pre-existing generic repair route writes Cargo.lock and target
artifacts. Original source/manifest bytes remain unchanged. The archived baseline
reproduces this failure (`reports/archived-wrong-action-results.json`). Fixing that
route is outside this worker's ownership of implementation modules; changing the
contract after observing it would weaken the gate. The failed check is preserved,
not silently dropped or recast as a success. Runtime files were archived and hash
verified before restoration; only this tools directory and the new doc remain.

## Measured outcomes

| Scope | Result |
| --- | --- |
| Archived candidate authored fixed sample | 3/3 implementations |
| Archived candidate original TRAIN fixed sample | 0/1 (quoted identifier rejected) |
| Archived candidate without CARGO_TARGET_DIR, bare request | Refused unchanged, required caller target |
| New fixed original TRAIN sample, both target/TMPDIR unset | 1/1 test-verified |
| Full permitted TRAIN implementation requests | **3/60 test-verified;57/60 refused unchanged** |
| Original authored fixed sample / full suite | 3/3; **20/20**,6 implementations14 safe refusals |
| New pure parser contracts | 34/34 |
| New full CLI contracts with target/TMPDIR unset | **33/34**,9 implementations24 unchanged refusals1 artifact-changing wrong-action case |
| Additional environment checks | 6/6:2 implementations4 unchanged refusals |
| Parser coverage alone | **23/60**,not code correctness |
| Scoped Rust tests | 76/76 (10 CLI,39 English-Rust,27 world-Rust) |
| fmt check / affected all-target clippy / debug build | Passed |
| Release / parent full765 | Not run: local strict gate failed |

Three distinct original TRAIN requests are `train/rb-implement-sum-even/` **chatty,
hurried,terse**, each `#0`: **one distinct function, three styles**, not three
algorithms. Each fresh source copy retained original public tests, passed the
finite-family native search and fresh live test, then passed independent cargo
test. Every byte outside the placeholder was preserved. This is original public
test verification, **not benchmark or hidden correctness**. No >=177 result or
retention of all174 parent passes is claimed. Trusted parent174/591/0of765 remains
unchanged. Parent alone owns any future full765 evaluation.

## Exact implementation archived

- English `implement.rs`: balanced quoted/bare Unicode identifiers, polite direct
  actions, single declarative stub target and bounded pronoun resolution, exact
  src/lib.rs location, fully consumed generic doc references. Behavioral suffixes,
  negation, multiple targets, malformed quotes and unsupported paths refuse.
  Recognized unsupported action/stub requests cannot fall through into generic do.
  The existing AddField tokenizer is private and unwraps quoted constituents, so
  this module uses a small cursor retaining quote boundaries; no shared parser edit.
- World `implement.rs`: reuse the archived signature-directed API unchanged except
  native environment setup. Missing target/TMPDIR uses owned disk scratch/cache;
  validation rejects relative/traversal,/tmp/symlink-to-/tmp and live/scratch source
  paths before creating directories. Each proposal has its own namespace under
  the caller's existing target cache, reused by every candidate and live check.
  Cached outputs are retained; no unrelated cache entries are deleted.
- Exports in the two libraries; minimal16-line CLI do route. Output explicitly
  says tests pass while English constraints/doc comments are not proved.
- **Candidate family is byte-identical to archive**, hash in `reports/source.json`.
  Names, English, docs and test assertion values never become proposer features.
  Nonempty original tests, source snapshots/bindings, rules, staleness, rollback,
  multiple-survivor refusal and finite search budget remain intact.

Final five exact source copies: `rejected-source/`; hashes/bytes:
`reports/validation.json`. Patch `rejected.patch`,37091bytes,
SHA256 `cfea6bd0c4abd8ac4819663dbc13fdc41483aedecc8298b4842398a05328c4a0`.
Final binary retained at
`/home/gsc/.cache/glassmind/continuous-20261010/impllanguage-run/glassmind-final`,
126989920bytes,SHA256
`ef5ba7d56a22814f620c322b0205e6a13234c7903aa7325c1eabc1c8096b1f5d`.
Archived baseline and intermediate binary hashes are also in validation.json.
`MANIFEST.sha256` covers submitted files; `reports/raw-artifacts.json` covers owned
raw projects, original test logs, proposal evidence and binaries, excluding target
build caches. No credentials/private logs are included.

## Provenance, freeze and rejected attempts

PLAN, original fixture/check adapter, independently authored language contracts
and permitted TRAIN input provenance are frozen in `FROZEN.sha256`; all verify.
Original fixture3433bytes,SHA256
`7284edba6c1fcc4b55f432069c3c74a30f69d781c558985b73c9b4f0b267aa40`.
New authored language contracts1287bytes,SHA256
`01f4ffc2936ff5b754c342b12198de4ed55fe973c96433314030986c4998b98d`.
Both locally authored code/fixtures use workspace **MIT OR Apache-2.0** license.
The original check adapter is copied unmodified under original/. The new evaluator
changes only its import location and independent cargo timeout30→600/outer wait40→620;
fixture expectations and scoring predicates are unchanged.

Existing TRAIN inputs:60 requests,5070 UTF-8 text bytes; projected kind/split/identity
metadata only, never expected patches. Existing production snapshots:20 projects,
40 allowed Cargo/src files,17648bytes. Exact version is their frozen byte hashes
in train-input-provenance.json; inherited benchmark pin
f79e71a2a26fdaf15f9bc55225a038b61e66fafb was not independently reverified. No upstream
license was present in the inspected allowlist, so none is inferred. Raw public
project sources remain outside git. No new downloaded data/model bytes.
Only production signatures/whole-body placeholder forms were displayed; public
tests were copied/hashed/executed opaquely, never mined to invent bodies.

Initial new run reached3/60, but two unsupported TRAIN forms fell through to the
old route and created artifacts. Added syntactic intent recognition; final60/60
byte-preservation/acceptance checks pass. Initial concurrent fixture verification
also ran the wrong same-named Cargo binary from a shared target: initial CLI27/34.
Added per-proposal cache ownership and reran actual suites serially; final33/34.
Original unsuccessful results/binaries/logs remain. The first clippy run rejected
a collapsible if; corrected, final all-target clippy passes. No family widening,
contract changes or benchmark scoring changes. Initial intermediate source was
not separately frozen; its binary/raw results remain; only final source is claimed
fully reproducible.

## Reproduction and limits

From repository root, retained binaries can replay without applying the patch:

```bash
RUN=/home/gsc/.cache/glassmind/continuous-20261010/impllanguage-run
export CARGO_TARGET_DIR="$RUN/target" TMPDIR="$RUN/tmp"
bash tools/coding-implement-language/reproduce.sh "$RUN/replay-new"
# Expected exit1 at final CLI contract check, with all results retained.
```

The wrapper was assembled from the individual commands actually run; no second
whole consolidated replay is claimed. To rebuild only in a disposable checkout of
the base revision, apply this **rejected** patch, keep the existing target and run:

```bash
git apply tools/coding-implement-language/rejected.patch
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo fmt --all -- --check
TMPDIR="$TMPDIR" GLASSMIND_SCRATCH="$RUN/rebuild-scratch" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo test --offline -p glassmind-world-rust -p glassmind-english-rust -p glassmind -j1 -- --test-threads 4
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo clippy --offline -p glassmind-world-rust -p glassmind-english-rust -p glassmind --all-targets -j1 -- -D warnings
TMPDIR="$TMPDIR" systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 cargo build --offline -p glassmind -j1
```

Native candidate checks use8GiB/0swap,setsid --wait,600s outer timeout plus unchanged
10s trial/75s search/90s application budgets,kill-after10s,-j1,testthreads4,disk TMPDIR.
Historical archived-runner replay and incidental inherited generic fallthrough used
those old runners' shorter timeout wrappers, not the new literal600s invocation;
this deviation is disclosed rather than claiming retroactive wrapper compliance.
The shared worktree git index is read-only: git restore could not acquire its
index lock. After archiving, restored the three owned tracked files byte-for-byte
with git show HEAD:path and removed only the two owned new runtime modules.
No surviving owned process/service or unrelated deletion. The outer worker scope
was8GiB/0swap. No source/build caches were globally cleared.

The grammar is bounded and conservative; generic doc references do not establish
doc entailment. Most original algorithms are outside the unchanged family. Per-
proposal target reuse trades disk growth for isolation; no cache eviction policy
was added. Filesystem races, spoofed test output, crash-atomic restore, disk-full
restore and adversarial process-tree timeout behavior were not fault-injected.
No default HOME/scratch-unset end-to-end run was performed (owned scratch supplied).
No claim about full765, hidden tests, broad coding correctness or accepted runtime.
