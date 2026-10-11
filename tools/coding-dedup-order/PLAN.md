# Frozen before production edits

Hypothesis: on a directly resolved standard mutable Vec of total-order primitive
scalars or String, swapping adjacent standalone zero-argument dedup(); sort();
statements provides a reusable structural repair proposal. Native public tests
are not proof of intended semantics. No names, docs, requests or expected outputs
participate in proposal selection.

Acceptance: verify original TRAIN baseline 8/27; candidate 9/27 with all eight
old response/patch records identical, unchanged selection SHA
6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164, zero failed
verifications. Fixed sample first, then all27; full765 belongs to coordinator.
All authored original programs must compile and fail executed public tests.
Positive contracts cover numeric widths, String, owned mutable parameters,
explicitly typed locals including map/filter/collect, Unicode names, inline tests,
and empty/singleton/sorted/reverse/adjacent/nonadjacent duplicate inputs.
Negative contracts cover aliases/custom types/imports/generics/macros/attributes,
shadowing, differing receivers, intervening effects, comparator/dedup_by calls,
unresolved types. Exact byte identity outside the reordered statement pair.
Two independently test-passing vector reorderings must yield distinct native
outputs and end-to-end refusal with saved complete choice evidence.

Scope: root lib.rs/main.rs free functions in closed namespace, exact builtin
Vec<T>, T one of signed/unsigned integer widths, bool, char or String. Owned mut
parameter or explicit typed mut local established before pair. No references,
custom types, aliases/imports, generics, macros, non-doc attributes, nested items,
shadowed bindings or nested control-flow pairs. Inline exact cfg(test) modules
follow existing extremum namespace restrictions. Unknown initializer flow is
not inferred: local's explicit Vec<T> type is the anchor, compile/tests verify.

Append only after all old proposals including extrema. Reuse minimal bounded
choice helper retaining extrema budget/order/evidence. Each alternative starts
from original tree. <=8 candidates/20seconds, all checks complete, exactly one
passing candidate and saved evidence; multiple passes/errors/incomplete/saving
failure withdraw, never fall through an old withdrawn choice. Preserve old
argument swap/scalar ambiguity behavior. No scalar/loop semantic witness invented.

Validation: relevant world-rust tests including existing extremum and argument
swap contracts, fmt, all-target crate clippy, release binary build, fixed TRAIN
sample/full27 and exact response comparisons. Freeze source/diff/binary/inputs/
outputs/reports with hashes. No downloads, other model calls, agents, commits,
DEV/heldout/parent artifacts. Source: vetted TRAIN RustBench f79e71a,
MIT OR Apache-2.0, 110files/57265bytes; no new external data.
