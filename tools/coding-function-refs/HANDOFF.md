# Function reference rename — bounded batch accepted locally

Uncommitted worker changes only. Base commit `078578b0ac350a1b3baa5af20b1a37d21d576d23`.
Ownrun: `/home/gsc/.cache/glassmind/continuous-20261010/functionrefs-run`.
No agents, model calls, services, downloads, credentials/API keys, hidden material or commits. Session model cannot be independently attested from repository tools; no model was launched or changed. Outer scope verified MemoryMax8589934592 / MemorySwapMax0. Every cargo/native job used persistent TMPDIR, existing CARGO_TARGET_DIR, systemd scope8GiB/0swap, setsid--wait, timeout600/kill10, -j1 and tests4. All owned jobs completed before handoff.

## Exact runtime changes

- Register new `rename_functions.rs`; preserve the existing generic rename guard and all its success paths. Only a refused single-source free-function rename may use the new proof.
- Reuse existing Package/module/import utilities unchanged. Resolve target root identity, conventional inline/file modules, imports/reexports, explicit aliases, same-package extern spelling in the binary and uniquely resolved `super::*`.
- Mark only the unique declaration, resolved import leaf and resolved expression-path references (ordinary calls and function values). Explicit aliases keep their local spelling. All old-spelling tokens must be proven target spans; every other/unclassified occurrence refuses. No ExprPath role relabeling, text replacement, compatibility aliases, or changes to signatures/bodies/logic/tests beyond the requested identifier.
- Builtin assertion/output expression lists are parsed, with shadow imports, custom macros and macro definitions refused. Named formatting arguments and implicit target-name captures refuse. Module-name shadowing by patterns/type/const parameters, block-local items, methods/fields/homonyms, multiple target declarations, collisions, raw identifiers, unknown attrs/cfg/imports/layouts refuse in this fallback. Literal prose docs and builtin derives allowed; cfg(test) only through existing module graph mode. Unicode character columns and CRLF verified.
- At apply, this fallback reindexes and redoes the proof before writing, requires identical proposed spans, and uses existing verification/rollback with fresh native tests. Old generic edits retain Build verification. Same-width stale-binding mutation is covered by a new unit test.
- Four new Rust tests, authored adapter, supplemental contracts, regression/reproduction scripts and reports. No existing utility semantics, English/meaning/CLI/checks/scorers/other tracks/shared manifests changed. No dependencies or Cargo.lock changes.

## Measured result

Fixed sample6/6, then full24/24 = **9 verified edits +15 byte-preserving refusals**. This comprises eight independent authored positive repairs and the supplied canonical repair. Baseline:1 edit +23 refusals (16/24 meet frozen expectations); all old positive cases retained. Seven authored positive refusals and the canonical refusal become verified edits. See RESULTS.json; this is development coverage, not a fullbenchmark gain.

Canonical `rename function parse_line to parse_entry`: exactly9 identifier replacements across4 Rust files; Cargo.toml and every other original byte unchanged. Both original public parser assertions execute before/after, assertions unchanged except function identity. Binary executes before/after with identical output:

```
info=1 warn=1 error=1 malformed=1
first: 08:00:00 Info
error: database unreachable
```

Read-only preview returns exactly9 valid spans without edits. Three original requests remain unchanged: terse/hurried UnknownShape; chatty Command(Repair(Test)), no rename or source change. Frontend remains blocked.

Supplemental12/12 contracts: preview spans/bytes, all3 original routes/source bytes, and a native name-sensitive assertion that passes before renaming, fails after renaming, and triggers exact rollback. Native tests execute, not just compile. All8 authored positive fixtures also execute assertions; self-library binary output is compared. Unsupported negative fixtures are checked for zero edits, not all native-compiled.

Final source gates: world34 unit +4 check-isolation tests pass; CLI edit3 tests pass; scoped fmt check, all-target world clippy -Dwarnings and main debug build pass.

Final-binary regression adapters: type edits9/9, variants14/14, package15/15, field-owner16/16, metadata35/35, ordinary queries8/8. Inherited exports **26/27**, unchanged unsupported_type expected-refusal mismatch; inherited AddField **14/15**, unchanged live-rollback failure. Baseline reruns preserve these exact failures. Original ask/apply contract scorer **15/16 vs baseline16/16**: its unsupported_macro expected-refusal is newly safe supported `super::weigh` in assert_eq!; exact two-token rename and original executing assertion retained. All its prior positive contracts retained. Original scorer/expectations untouched, failure recorded rather than skipped.

Supplied full765 baseline177pass588refused0fail is coordinator evidence only. Worker did not mount/run full765. Parent must run independent adversarial identity probes and full765 preservation gate before publishing.

## Sources, licenses and frozen versions

Supplied Rustbench revision `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, MIT OR Apache-2.0. Verified every supplied SHA256/size against dev-inputs/provenance.json. **Five** Cargo/source files,2923 bytes; brief says six, actual manifest/tree list five. No task.toml/solutions/hidden contents accessed. Three supplied train-v3 requests +canonical and baseline traces are permitted inputs. No newly downloaded data (0 bytes).

Authored primary fixtures:23 cases,71 generated files (Cargo/source/prose),5652 bytes, repo MIT OR Apache-2.0; generator version frozen in FROZEN.sha256. Per-file hashes in ownrun/authored-source-manifest.json. Supplemental rollback fixture is independently authored in contracts.py. Runtime has no fixture IDs/expected answers/lookup tables or LLM.

Baseline binary SHA256 `226a97606e46889ac7d1005b513b13c9cb2c9343fd11c027752d3ec8dbd6dbf0`.
Final candidate SHA256 `f57788db8058bde96f306a0b9de2a8d7a1645969f6bd2b17a35254ac53607cf9`.
PLAN and independent fixture generator frozen before runtime editing; see EXPERIMENTS.md for two disclosed checker corrections and subsequent conservative safety refinements. No rejected runtime hypothesis was deployed. Initial checker failure and intermediate binaries/reports retained, no evidence deleted.

## Reproduction and artifacts

From repository root, with supplied persistent TMPDIR/CARGO_TARGET_DIR:

```
bash tools/coding-function-refs/reproduce.sh ../functionrefs-run ../functionrefs-run/replay-new
```

This runs baseline sample/full, scoped gates, candidate sample/full, contracts and existing adapters. Adapter exit status alone is insufficient; inspect JSON/status reports (existing askverify and AddField wrapper can exit0 with failed expectations). Full invocation commands are in reproduce.sh; every cargo command is wrapped as required.

Actual final evidence: ownrun/accepted-{sample,full,contracts}, regressions-accepted, regressions-baseline, world-tests-handoff.log, cli-tests-handoff.log, clippy-handoff.log, fmt-final.log, build-accepted.log. All original/repaired sources and native output logs retained alongside results. Canonical patch, final source patch, submitted files and raw-artifact SHA256/byte manifest retained in ownrun; small summary in RESULTS.json.

Limits: conservative closed subset, no arbitrary external imports, raw identifiers, general glob resolution, binding shadowing, custom macros, conditional function definitions or unusual Cargo layouts. Same-spelling other bindings are refused rather than renamed. Existing generic route deliberately unchanged, including its historical limitations. No claims for original frontend requests, broad semantic preservation beyond the proof/specified native observations, release build, fullworkspace clippy, or full765.
