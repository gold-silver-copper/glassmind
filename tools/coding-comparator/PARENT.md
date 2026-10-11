# Independent review: record comparator direction

Full public RustBench DEV improves 212→215/765, retaining every old pass with
550 refusals and zero failures. The three gains are request styles of one arcade
leaderboard bug. Runner, denominator and comparison criteria are unchanged.

The five frozen independent repair cases pass exactly: unsigned integers with
String tie-breaking, signed integers, String, Unicode char/bindings and bool.
Four alias/import/custom-Ord/computed-receiver controls remain unchanged. All
eleven originals compile and fail executed tests, without timeouts or manifest
changes. Baseline repaired none of the five positive cases.

Both ambiguity cases remain refused. Two comparator alternatives pass the same
public test but produce different orders. The older dedup ambiguity has two
passing old alternatives and one passing comparator alternative, each producing
a distinct output. Five independent native executions establish those outcomes.
The candidate preserves the older two-passing refusal evidence byte-for-byte
and creates no comparator fallback evidence in that case. The new comparator
ambiguity records two complete passing choices and refuses.

The source review checks closed namespace/type resolution, unique bindings,
intrinsic total-order fields, pure same-field comparisons, and exact expression
span swaps. The entire cmp/then_with chain must be recognized. Tie-breaking and
all bytes outside the swapped operands are preserved. Imports, aliases, custom
Ord, opaque macros, effects, floats and unsupported type paths are excluded.
The new family follows all older repair families. Verification starts from the
original tree for each choice, requires exactly one complete passing choice and
saved evidence, and preserves the existing eight-choice/twenty-second limits.
An observation flag blocks only the comparator fallback after older compiler
findings/errors; older family priority and acceptance semantics are unchanged.

Worker TRAIN results improve 9→10/27 and fixed sample 2→3/4. All nine old complete
outcome files are byte-identical. Its 57 Rust tests and 30 native contracts pass.
The worker accidentally ran the full baseline before the sample and redundantly
repeated it; original outputs are retained. Candidate sample preceded full TRAIN.
A formatting failure was corrected before final tests/clippy/release. These are
public-test repair results, not proof of general intended ordering semantics.

No new external data: reused RustBench f79e71a TRAIN, MIT OR Apache-2.0, 110 files
and 57,265 bytes. New authored worker contract JSON is 14,912 bytes. No hidden
items, labels, generators or test contents were inspected by the development
worker or parent. Parent fixtures and their independent oracle were frozen
before candidate inspection.

Original archive: 1,684 files / 280,126,084 bytes, hash-verified before inspection.
Frozen submission: 1,639 files / 279,019,019 bytes, independently hash-verified.
Evaluated candidate binary SHA:
`2343c2ed6db780ee4b2be674800bfcb168f5acfb4e4c74f532ae2df4aab7ce38`.
Integrated world-rust files match that frozen source exactly.

Compact parent fixtures, adapters and reports are under `parent/`. Full native
evidence remains at
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/comparator-independent`;
original submission is in `comparator-run/terminal-originals` and the frozen
binary/source in `comparator-run/frozen-candidate` under the same work root.
The full 765-row DEV retention run and workspace gate pass. Final combined
binary SHA is `53c6cacac60e61cf90aba78d183c9134e574b3db56755141913392700c69dfff`.
Its world-rust source matches the frozen full765 candidate exactly. Its quantity
and main CLI sources match the independently evaluated comparison integration.
Workspace fmt, all-target clippy, release build and strict 4/4 smoke pass.
