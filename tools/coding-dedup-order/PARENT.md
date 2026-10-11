# Independent review: typed dedup/sort order repairs

Full public RustBench DEV improves 209→212/765: all 209 previous passes retained,
553 refusals, zero failures. The three gains are request styles of one blog-tags
bug. The runner, denominator and baseline are unchanged. No hidden test items,
labels, generators or source contents were inspected. The worker's separate
27-row TRAIN probe improves 8→9, retaining every old outcome byte-for-byte.

Five frozen parent repair cases pass exactly: owned integer/String vectors,
explicitly typed integer/String locals, and a Unicode binding. Five alias,
intervening-effect, different-receiver and custom-method controls remain unchanged.
All twelve original fixtures compile and fail executed tests, with no timeout or
manifest changes. This tests actual behavioral repair rather than compile failures.

Both unresolved cases are refused. Two different dedup/sort choices each pass
the same public test but produce different vectors. The previous-extremum veto
case has two passing old choices and a passing new reorder alternative, with three
different outputs. The main CLI preserves the old complete two-passing refusal
evidence exactly and never tries the new family. All five alternatives were
independently executed before candidate inspection.

The parent reviewed namespace/type restrictions, exact byte-span swaps and
bounded native verification. Only adjacent zero-argument dedup/sort statements
on one uniquely bound standard owned Vec of total-order primitive/String elements
are proposed. Imports, aliases, custom/generic types, shadowing, nested flows,
intervening statements, macros and custom comparison methods are excluded.
Exactly one complete passing choice with saved evidence is required. Older
family ambiguity or incomplete verification blocks fallback. Existing legacy
proposal/undo/cargo/timeout code remains byte-identical. The old extremum helper
is shared with its original budget, priority, records and compiler-finding policy.

All 54 worker Rust tests and 29 authored native checks passed. Its initial clippy
slice-index warnings were fixed; a separately frozen raw-identifier shadowing
supplement added four checks. These original attempts are retained. No new
external data: RustBench f79e71a TRAIN, MIT OR Apache-2.0, 110 supplied files /
57265 bytes. New authored fixture bytes:10488+1342, repository license.

Original terminal archive:1182 files /173993430 bytes, independently hash-verified
before inspection. Frozen candidate:1137 entries /172975121 bytes verified.
Full-evaluated binary SHA
`dc5f5cccbc76d8749be40b083b003f7d6e2074c16c1bb0829c282ce645fcdb2d`.
Integrated world-rust sources match the frozen submission exactly. Native public
tests do not prove intended semantics; this remains a closed structural proposal
family rather than general method-order reasoning.

Compact fixtures, result comparisons, choice evidence reviews and adapters are
under `parent/`. Raw outputs and alternatives remain in
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/deduporder-independent`;
full results are in `coordinator/deduporder-full`, originals in
`deduporder-run/terminal-originals`. Reproduction uses those retained cache paths
and the capped `tools/rustbench-dev.sh` runner. No additional model calls or
native inference were used to change a failed expectation.

Final combined binary SHA
`b6b093c5a82e76ec08481b5e0c8739e6b9090590f5f06b79d8b41eeb04e8948d`.
All26 world-rust files match the full765 candidate. CLI/dialogue source matches
the independently evaluated initial integration, except main help text whose
previous bytes were reconstructed and hash-verified. Workspace fmt, all-target
clippy, release build and strict4/4 fixture smoke pass. Combined source manifest
and check log are retained in `coordinator/combined-deduporder-draftsession`.
