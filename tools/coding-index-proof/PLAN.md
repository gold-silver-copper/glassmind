# Frozen plan (2026-10-11, before runtime edits)
Hypothesis: exact compiler E0277 slice-index diagnostics plus a bounded straight-line
AST proof permit a minimal signed index cast, retaining original one-based behavior.
Baseline source b27dabe0aeacda2a86acc520450f8df3e59f7c47; supplied frozen binary
20594291e08e216596645ac21355eca216f8cadfb12cd20e66d7fd71350634f6.
Coordinator full DEV baseline 174 pass /591 refuse /0 fail; no repeat/full access.

Supported: conventional src/lib.rs crate; production top-level free functions only
(test modules exempt); doc attributes only. Exactly one immutable signed parameter
(i8/i16/i32/i64), one immutable shared slice parameter with primitive elements or
&str. No generics, aliases, custom types, mutation, shadowing, closures, macros,
loops, joins, unknown attributes, or unmodeled calls. Sequential exiting if guards,
or OR of lower/upper rejection conditions, establish lower >=0/1 and upper rank
<slice.len()/<=slice.len(). Reversed comparison operands supported. Exits return
None; one optional immutable local = rank, rank-1, rank+0. Final Some(slice[index])
or Some(slice[index].to_string()) only. Exactly one index, diagnostic same source,
file and line; exact old span text retained. Require upper bound appropriate to
offset and lower sufficient for arithmetic definedness. No new behavior branches.

Target proof: after AST proof, use existing copy/apply/check machinery to compile
a temporary expression containing a constant array equality proving usize::BITS
>= signed parameter width in that exact native build. The temporary proof is never
proposed. No host-width inference. Reject explicit local/ancestor cargo configs,
custom crate targets/build scripts/manifest options outside bounded package fields.
Final existing build verification and all prior ambiguity paths remain intact.

Fixed first sample: supplied terse request, one zero-based i16, unsafe absent lower.
Full independently authored frozen cases: positive one-based i8/i16/i32/i64,
zero-based i32/i16, names and reversed comparisons, split guards, rank-1 / rank+0 /
direct index. Negatives: missing/partial bounds, wrong >=/> boundary, AND,
non-exiting branches, mutation, shadowing, overflow, negative/wrapping, generics,
custom index/type, macros, unknown/cfg attributes, nested guards/joins, multiple
indices, unsupported calls/aliases and source out of scope.
Expected sources generated and saved before running solver; native original E0277
retained and independent native tests check negative/zero/valid/last/outside/extreme
inputs against one-/zero-based semantics. All original source bytes except exact
index edit preserved, original tests unchanged.
Acceptance: >=1/3 supplied requests and >=5 authored positives minimal exact repair
plus native behavior; zero accepted negatives; scoped tests/fmt/clippy/main build
and relevant existing adapters. Preserve inherited adapter failures. Reject and
restore owned runtime if gate fails, preserving candidate patch/binary/reports.
Locally authored data license MIT OR Apache-2.0; no downloads/dependencies/model calls.
