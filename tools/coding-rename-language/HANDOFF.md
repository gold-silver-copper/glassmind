# Rename language handoff — 2026-10-11

One bounded batch, ready for independent language review and the parent's full
coding development run. No commit. Baseline source:
`8e12be3f884720a9ba6065fbec49e274d37589c4`.

## Changes

- New `crates/english-rust/src/rename_requests.rs`: lossless identifier tokens,
  fully consumed rename clauses, explicit source/destination binding, supported
  kind/owner preservation, global coverage and behavior-preserving clauses.
  Conversational descriptions have a bounded grammar; arbitrary prose is refused.
- One module export in `crates/english-rust/src/lib.rs`.
- `crates/cli/src/coding.rs`: shared ask/do routing, typed `Action::Rename`,
  compatibility with genuine existing readings, and refusal of incompatible
  fallback/partial guesses. Exact old alternatives retain ambiguity. Old
  free-text remarks require an additional bounded compatibility proof.
  Four new tests cover frozen cases, old alternatives, 27 legacy forms and
  additional refusal probes. Runtime integration is about 80 lines; the rest
  of the CLI diff is tests.
- This adapter directory contains the frozen plan/cases, replay and native
  verification scripts, reports, provenance and reproduction script.

No backend, checks, models, main CLI, evaluation definitions, shared manifests,
README, historical prompt or other capability code changed. No new dependencies.
No agents, model calls, credentials, API keys, downloads or held-out access.

## Measured result

Fixed supplied sample first, then all 38 affected requests (3 supplied + 35
frozen authored). `reports/comparison.json` compares typed meanings, not replies.

| Measurement | Baseline | Candidate |
|---|---:|---:|
| Supplied correct rename meanings | 0/3 | 3/3 |
| All positive correct rename meanings | 5/15 | 15/15 |
| Negative cases exposing Command readings | 13/21 | 0/21 |
| Expected test-repair controls | 1 repair, 1 refusal | unchanged |

The 12 authored positive cases all pass. The 21 frozen negatives and 14
additional refusal probes expose no Command or mutating guess. Baseline negative
counts include incomplete Rename readings; they do **not** mean 13 applied edits.
The frozen corpus has **37/38 exact expected coverage**, not 38: the authored
expectation for `fix the test named rename_foo` was overoptimistic. The baseline
already refuses it, and this patch deliberately preserves that refusal. The
frozen case and its expectation were not changed. The unit test checks baseline
preservation for both unrelated repair controls rather than adding repair syntax.

All 38 ask/do meaning lists match exactly; all supplied `.rs` hashes remain
unchanged. All three supplied requests bind exact `parse_line` → `parse_entry`.
The conversational request preserves explicit `Some(Function)`; bare-name
requests retain `None`, then resolve to the same function. None becomes Repair.
**The unchanged backend refuses all three supplied-workspace renames.** There is
no claim to repair that backend or pass the underlying coding task.

Native independent fixture: three syntax forms × ask --apply/do = **6/6**.
Before/after executable output and authored unit tests agree; exact source checks
preserve comments, string literals and unrelated code while changing definition,
caller and test reference. Existing public native edit regression: **9/9**.
All **17 CLI + 40 English-Rust tests** pass, including existing rename/identifier
and ask/apply tests. Fmt check, scoped clippy with warnings denied, build and
`git diff --check` pass. Reproduction script was assembled from commands run
individually; no second complete script replay is claimed.

## Rejected intermediate choices

Failure logs are retained, not overwritten:

- Retaining all zero-edit old readings admitted incomplete conversational
  interpretations that discarded the explicit destination. Replaced by
  full-consumption compatibility checks (`authored-initial.log`).
- Requiring every legacy reading to be complete lost eight old conversational
  forms. Added bounded legacy validation while preserving their original
  alternatives (`legacy-initial.log`, `authored-third.log`, `authored-fourth.log`).
- Checking only absence of the grammar TEXT class missed `except tests`, parsed
  as a named-check remark. Now reject unvalidated Remark/Topic/Musing categories
  too (`authored-fifth.log`, `rejected-exact-scope.txt`).
- A quoted diagnostic about the word rename received an unrelated old reading.
  Explicit diagnostic introductions now fence quoted rename mentions; ordinary
  repair names containing rename remain unaffected (`quoted-probes.log`).

## Provenance and frozen artifacts

No external data was added. Authored cases: **35 rows / 2,731 bytes**, dated
2026-10-11, under this repository's MIT OR Apache-2.0 license. Existing public
English regression extraction: **27 rows / 1,630 bytes**, from the baseline
source revision above, same license. Independent native source: **322 bytes**;
its authored generator: **3,080 bytes**. Supplied requests: **658 bytes**.

Supplied public workspace revision:
`f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, MIT OR Apache-2.0, **2,923 bytes**.
The supplied provenance manifest and actual mount enumerate **five files**
(Cargo.toml and four Rust files), despite the brief saying six. All five original
hashes verified; no missing file was sought. Full provenance is in
`reports/provenance.json` and `reports/supplied-provenance.json`.

Approved baseline binary SHA256:
`226a97606e46889ac7d1005b513b13c9cb2c9343fd11c027752d3ec8dbd6dbf0`.
Frozen candidate (debug build, 129,837,728 bytes):
`../renameenglish-run/candidate/glassmind`, SHA256
`9cba58d1363d3983e0377ca13e818522b0db245058d0351491fe6ed809efa5ff`.

Raw full traces remain in `../renameenglish-run/baseline-full` and
`../renameenglish-run/frozen-full`; native artifacts in `frozen-native` and
`edits-regression`. Source/report snapshot and hashes are at
`../renameenglish-run/frozen-source` and `frozen-source-manifest.json`.
The earlier candidate is separately retained as `before-quoted-fence.glassmind`.

## Reproduction and limits

From the worktree, run:

```sh
bash tools/coding-rename-language/reproduce.sh
```

`RUN_ROOT` and `REPLAY_ROOT` can redirect retained inputs/output. Every cargo and
native replay uses persistent TMPDIR, the existing CARGO_TARGET_DIR, systemd
8-GiB/zero-swap scope, `setsid --wait`, 600-second timeout and 10-second kill grace;
cargo uses `-j1` and tests use `--test-threads 4`. The worker's outer scope was
verified at 8 GiB/zero swap. All owned subprocesses are terminal; no job is left
running. Failed iteration exit codes are retained in their logs.

This is bounded syntax, not unrestricted English understanding. Unsupported
kinds, descriptions, conditional edits and extra operations refuse. Identifier
validation accepts alphabetic/alphanumeric Unicode plus underscores and raw
prefixes; it is not a complete Rust XID validator. Backend source binding and
rename execution remain unchanged. The helper is integrated in CodingLanguage,
not the generic English parser used independently by library callers.

**Parent full765 run and independent adversarial probes remain required.** No
benchmark mount was available or accessed, and preserving ~177/765 successes and
zero failures is not claimed from these scoped tests. Combine with a separately
accepted backend only after independent review.
