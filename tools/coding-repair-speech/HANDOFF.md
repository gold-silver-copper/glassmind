# Repair speech-act batch — review candidate, not published

Base checkout `1ab5b03643b34fcca338bb1744758ed240b0ef0a`; runtime baseline
bb89964, supplied release SHA256
`f5bda2d3d73085eea1790bcd8d34a767d40e381e61e5b9c7fb2efdb332f311a6`.
Final debug candidate SHA256
`f8699eb506931014ce833046d86018f18b5cfc0f0c06f708518d2816e705264f`.
No commits, downloads, model calls, agents, credentials, evaluation-definition,
backend, CLI entry, rename-helper, model-weight, or shared-manifest changes.

## Changes

* New `crates/english-rust/src/repair_requests.rs`: bounded token/cursor parser
  returning Repair or Status with Build/Test kind. Fully consumes auxiliary
  questions, direct/polite requests, failure context and bounded diagnostic
  clauses. Quotes retain their data role; quoted identifiers require identifier
  syntax. Conditions, cancellation, explanatory questions, and additional
  operations cannot be discarded to authorize a repair. Limits: 4096 bytes and
  128 tokens.
* One module export in English/Rust. `crates/cli/src/coding.rs` applies the same
  authority boundary to all understand/guess results and gates learned repair
  proposals. Existing rename routing body and helper are unchanged. Intent
  probability cannot authorize an edit. Supported status questions become typed
  Ask(Passes(Build/Test)); positive requests remain typed Command(Repair(...)).
* New authored development tests, probes, raw-result summaries and reproduction
  script only under `tools/coding-repair-speech`. Original supplied files remain
  untouched in `../repairspeech-run/dev-inputs`.

## Validation and acceptance limits

PLAN and 68 cases were hashed before runtime edits: 24 positive requests,
12 status questions, 32 adversarial/unsupported cases. Baseline preview results:
23/24 positive remedies, 4/12 status answers, 10/32 nonmutating negative results;
30 unauthorized remedy previews in 44 status/negative inputs.

Final tests: 20 CLI tests and 42 English/Rust tests pass, including all original
19 CLI +40 English/Rust tests. Frozen 68-case tests assert typed meanings and
absence of Repair in guesses. Additional tests preserve all 15 public legacy
repair phrasings and reject contextual/hypothetical variants. Build, fmt and
package all-target clippy with warnings denied pass. Logs are retained here.

The original frozen native acceptance is NOT fully satisfied: its assumption
that a new repair request with `ask --apply` immediately applies was wrong.
Both binaries instead retain a proposal until affirmative confirmation. These
failures remain failures in every raw report; no expected label was changed.
Separate confirmation probes exercise preview + `yes --apply` using a session.
See final observations and results below for actual behaviors by protocol.

Coordinator full765 preservation and independent review are still REQUIRED.
This worker did not read benchmark definitions or run that evaluation. No claim
that all coding baseline passes are preserved or that this candidate is ready to
publish. Unsupported prose outside the bounded clauses refuses repair; additional
unseen conversational forms may lose coverage. Source-name clauses identify a
workspace-wide check, as existing Repair semantics do, not an individual-test edit.
Only Build/Test repair authority is gated; other existing action kinds are preserved.

## Evidence, reproduction, provenance

All run paths below are relative to
`/home/gsc/.cache/glassmind/continuous-20261010/repairspeech-run`.
Only `verified-*` runs use the immutable `frozen-candidate/glassmind` with unique
fixture package identities. Earlier candidate/final directories are retained
interim runs, not final exact-binary evidence. `EXPERIMENTS.md` records the rejected
first candidate, unsupported named-test regression, context-parser corrections,
incorrect initial apply expectation, and native shared-target fixture interference.

Run `bash tools/coding-repair-speech/reproduce.sh` with exported TMPDIR,
CARGO_TARGET_DIR, REPAIR_RUN and BASELINE_BINARY. It contains all validation and
probe commands. In this batch TMPDIR and CARGO_TARGET_DIR were the supplied
`../repairspeech-run/tmp` and `../repairspeech-run/target`. Every cargo call,
including native fixture checks, used an 8GiB/zero-swap user scope, setsid --wait,
timeout600/kill-after10s, -j1 (except fmt), and test threads4. The outer worker
scope also reported MemoryMax8589934592 / MemorySwapMax0.

No new external data. `cases.tsv` is locally authored development material under
this repository's MIT OR Apache-2.0 license. `public-legacy.tsv` copies 15 existing
public authored test strings from English/Rust at the base source version.
`supplied.tsv` transcribes the 14 user-supplied development cases; no separate
redistribution license is inferred. Exact bytes and SHA256 values, plus source,
binary, probe and raw result identities, are recorded in `HASHES.json`.

## Final measured results

* Authored full: 68 requests ×3 modes =204 fresh broken workspaces. All132
  negative/status executions preserve exact source and remain natively broken;
  all36 status executions return the correct check kind. No unauthorized Repair.
* All24 positive do executions make exactly `7u8` → `7u32` and pass the original
  `assert_eq!(number(),7)` test. All48 positive initial ask/ask --apply executions
  retain the original broken source and return repair proposals. Thus the original
  frozen native expectation is **180/204**, with24 apply-expectation failures,
  explicitly not204/204. Native before checks fail in all204 workspaces.
* Supplied diagnostic: 14×3 =42 executions; all36 negative/status executions
  unchanged and natively broken. Both positive do executions repair correctly.
  Original expectation40/42; two initial-apply expectation failures retained.
* Public legacy: 15×3 =45 executions; all15 do executions exact repair/native pass,
  all30 initial preview/apply requests unchanged. Baseline15/15 preview meanings
  preserved. Original expectation30/45; fifteen initial-apply failures retained.
* Explicit session confirmation:2/2 build/test previews unchanged, followed by
  exact repaired source and native assertion success after `yes --apply`.
* All204 authored fixture manifests unchanged; only the authored src/lib.rs exists
  in each source directory. Raw JSON retains exact original/after strings, SHA256,
  CLI output, native stdout/stderr and exit codes. Supplied/legacy audits also
  retain manifest/source-inventory observations.

The final source, binary and evidence are retained for coordinator review. No
worker-owned jobs remain at handoff. No rollback or commit performed.

## New data byte counts

* `cases.tsv`: 2889 bytes, 68 development cases.
* `public-legacy.tsv`: 1065 bytes, 15 development cases.
* `supplied.tsv`: 793 bytes, 14 development cases.
