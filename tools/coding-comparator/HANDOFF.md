# Comparator direction worker handoff

Candidate measured gain; uncommitted; awaiting parent independent contracts and
full765 retention gate. No full DEV was run or inspected by this worker.

Exact source base: 3d5748fd5d15646185f11b7ad67708cdc6ef6228.
Baseline binary SHA256:
b6b093c5a82e76ec08481b5e0c8739e6b9090590f5f06b79d8b41eeb04e8948d.
Final release binary SHA256:
2343c2ed6db780ee4b2be674800bfcb168f5acfb4e4c74f532ae2df4aab7ce38.

## Changes

Only crates/world-rust and new tools/coding-comparator are changed.
- New comparator.rs: closed source-local namespace/type/flow checks for
  Vec<local named struct>, direct mutable Vec arguments, explicitly typed locals,
  and unique &[Struct] argument .to_vec() locals. All struct fields must be owned
  intrinsic Ord integer widths, bool, char or String. Transparent nongeneric
  functions/structs, builtin derives and exact inline cfg(test) modules only.
- Recognize pure same-field cmp between the two sort_by closure parameters and
  an entirely recognized then_with chain. Propose swapping one pair of field
  expression spans, independently for each comparison. Preserve every other
  byte, tie-breaking expression, manifest and test. No name/comment/English
  direction rule, expected-patch lookup, runtime LLM or test-derived generation.
- checks.rs appends comparator after every older family, including dedup-order.
  Reuses independent original-tree native verification: <=8 choices / 20 seconds,
  complete checks, exactly one pass, and successfully saved evidence required.
  Comparator compiler findings are incomplete. A separate observation flag
  blocks only the new fallback after older compiler errors; older priority,
  budgets, returned patches and native-choice semantics remain unchanged.
- lib.rs registers the module. comparator_tests.rs and tools/coding-comparator
  hold public authored contracts, frozen plan, adapters, audit and reproduction.

## Measurement and validation

Same fixed TRAIN selection SHA256:
6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164.
Baseline 9 verified / 18 refused / 0 failed verification out of 27.
Candidate 10 verified / 17 refused / 0 failed verification out of 27.
All nine previous ENTIRE outcome files byte-identical, not merely same patches.
Fixed four-source sample 2/4 -> 3/4, zero failed verification.
Only additional verified source: public arcade leaderboard. Native evidence has
2 candidates, exactly 1 pass, incomplete=false; only primary operands reversed.
These are public native-test outcomes, not semantic correctness claims.

57 world-rust tests pass, including old native ambiguity cases. Thirty new
frozen native contracts: 19 unique repairs, 11 refusals (9 unsupported, 1 already
correct, 1 competing). All 29 failing originals compile and execute failing
public tests; already-correct original passes. Every accepted patched copy passes.
All 30 original source/test/manifest byte contents checked unchanged. Accepted
patches preserve outside bytes, ties, negative/repeated data, empty/singleton,
Unicode bindings/comments. Types cover all integer widths plus bool/char/String.

Actual competing alternatives both pass the same public tests and execute with
distinct outputs `2 1` and `1 2`; choice refuses. Actual old dedup ambiguity has
two passing alternatives plus a separately proved passing comparator repair;
undo_slip still refuses. Native controls also cover over-budget, invalid patches,
compiler incompleteness, and old extremum compiler-finding semantics unchanged
while the comparator-only veto is set. Save-failure behavior remains the existing
verifier's required saved-evidence check; no new save-failure fault injection.

Final fmt --all -- --check, world-rust all-target clippy -D warnings, and release
main build pass. Every cargo command used persistent TMPDIR, existing target,
8GiB / zero swap scope, setsid --wait, 600-second kill-after timeout, -j1 (fmt
excepted), and test-threads 4. Outer worker scope verified 8GiB / zero swap.
All owned command sessions completed; no agents, models, services, downloads,
credential reads, API keys, commits or lingering jobs were used/created.

## Sources and frozen evidence

Reused approved readonly public TRAIN only:
/home/gsc/.cache/glassmind/continuous-20261010/extremum-source
rustbench f79e71a, MIT OR Apache-2.0; 27 sources, exact 110 manifest files,
57,265 bytes (including sanitized request-kind inputs). Every manifest size/hash
revalidated. No source/selection/labels/test/scorer/evaluation definitions changed.
No external new data. New locally authored public contract JSON: 14,912 bytes,
SHA256 65693fe831012670b762b22e595e369a31abd218b836416beac09e0745664cae,
version frozen in this batch, project MIT OR Apache-2.0 license. cases.py is its
generator. The original case JSON is byte-identical to the pre-edit freeze.

RUN=/home/gsc/.cache/glassmind/continuous-20261010/comparator-run
RUN/frozen-candidate contains original-source (26 files / 365664 bytes),
final-source/crates/world-rust (28 files / 390507 bytes), baseline and final
binaries, unevaluated initial release, candidate.patch, plan, authored contracts,
all native choice records, native fixtures, command logs, baseline/candidate
sample/full TRAIN outcomes and verification logs, source manifest/data copy,
comparison.json, audit.json, and freeze-manifest.json with all artifact hashes.
See COMMANDS.md for exact invocation forms, compare.py for the exact outcome
byte gate, audit.py for local/source checks, freeze.py for archive reproduction.
The shared tools/coding-repair-dev.py adapter was used unchanged and is archived.

## Rejected attempts and limitations

The initial baseline command accidentally omitted --sample and ran all27 first.
A redundant baseline full invocation completed using cached observations before
an exact-process-group stop attempt found it already gone. Both runs are retained
(baseline-first-full and baseline-full). Correct fixed baseline sample was then
run. Thus requested baseline sample-first order and no-duplicate policy were not
fully met. Candidate sample DID precede candidate full. No input selection or
expectation was altered. See ATTEMPTS.md for this error and all failed checks.

One fmt check failed on wrapping after a guard addition, was corrected, and is
retained. No failed semantic experiment or fixture expectation was relabeled.
Review added comparator-only older-error observation and narrower function type
paths; tests/clippy/release were repeated after those source changes. The 30 main
native cases and shape/acceptance plan were frozen before production edits;
additional veto/incompleteness control bodies were written during implementation.

This is deliberately narrow source-local resolution, not whole-program analysis.
Imports, aliases, custom types/cmp/Ord/impls, unknown macros/attributes, qualified
or associated types, arbitrary generics/deref, shadowing, non-pure receivers,
floating comparisons, opaque comparator bodies and other chain constructs refuse.
No broad coverage or intended ascending/descending semantic claim is made.
Cached native observations are fingerprinted by unchanged source; independent
native baseline/variants were executed before those cached reruns.

Parent next action: archive this submission before inspection, run independent
contracts and full765 against the frozen final binary; require all212 retained,
zero failures, unchanged denominator765 before integration. Worker did not modify
main CLI, shared manifests, scorers, historical prompts or README files.
