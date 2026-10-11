Frozen before implementation, 2026-10-11.
Hypothesis: reversing one native Ord::cmp of pure fields within an ordinary
Vec<local named struct>::sort_by comparator can repair a direction mistake while
preserving every other comparison and byte outside the operand spans.
Baseline binary b6b093c5a82e76ec08481b5e0c8739e6b9090590f5f06b79d8b41eeb04e8948d:
TRAIN 9/27; fixed sample 2/4; full DEV 212/765 (parent only; never run here).

Recognized shapes: root lib/main source; closed namespace of transparent free
functions and named nongeneric structs; doc attributes and builtin derives only;
inline exact cfg(test) modules skipped. No imports, aliases, impls, macros,
external modules, opaque attributes, generic functions/types, custom fields,
qualified types or repeated bindings anywhere in a function. Vec receiver is a
unique mutable direct function argument, typed local, or local initialized by
.to_vec() on a unique &[LocalStruct] argument. Local struct fields must all be
owned intrinsic Ord primitives (signed/unsigned integer widths, bool, char) or
String. Closure has two unique untyped immutable bindings. Body is exact field
cmp or a lexicographic then_with chain of zero-argument pure closures. Each cmp
must compare the SAME declared field of the two distinct closure parameters.
Only one pair of field expressions is swapped per candidate. No whole-chain
reverse, no English/name/comment matching, no test-derived code generation.

Acceptance: <=8 alternatives/20s, independent original tree per choice, all
checks complete, exactly one passing choice and saved evidence; compiler errors
in the new family are incomplete. All older priority/budgets/patches retained;
older ambiguity/error/incomplete/save failure vetoes new fallback. Fixed TRAIN
sample first, then exact all27 input selection, gain>=1, zero failed verification,
all nine old entire outcome files byte-identical. World-rust tests including old
native ambiguity cases, fmt, all-target clippy, release main. No full DEV here.

Authored native contracts frozen next before production edits: equal primary /
distinct secondary; negative signed numbers; repeats, empty, singleton; Unicode
bindings/comments; primary and secondary repairs independently; direct Vec,
typed local and slice-to-Vec; already-correct comparator; unsupported aliases,
imports, custom cmp/Ord/types, generics, deref, macros, attributes, shadowing,
side-effectful receivers and floating fields. Two passing comparator alternatives
with differing actual native outputs must refuse. Two older dedup alternatives
plus one passing comparator must refuse due older veto. Repair originals must
compile and execute a failing test, patched copies pass; originals, tests and
manifests unchanged. Preserve failed experiments without changing expectations.

Source reused unchanged: rustbench f79e71a, MIT OR Apache-2.0,
/home/gsc/.cache/glassmind/continuous-20261010/extremum-source,
27 sources / 110 files / 57265 bytes including sanitized request-kind inputs.
No downloads, credentials, model calls, services, agents, commits or hidden data.
