# Proof boundary
The compiler must report E0277 for a signed slice index on the exact source,
crate-root file, and line. Its message's signed width must match the parameter.
Only one proved index may match; otherwise no proposal. AST spans identify the
exact replaced expression, and the existing apply machinery checks old bytes.

A function has two immutable, uniquely named parameters: a known signed primitive
and an immutable reference to a slice of known primitives or &str. Production
crate-root items are functions; no imports, custom types, aliases, macros, or
production modules can change primitive, slice, or Option name resolution. Functions
named Some or None also refuse. The only modeled calls in the selected function
are slice.len(), Some(...), and optionally the indexed value's to_string().

Sequential ifs must have no else and exactly `return None` in the rejecting branch.
Supported comparisons are normalized for reversed operands. An OR is processed
left-to-right; each rejection predicate's negation is true on fall-through. AND
and nested control-flow are unsupported. A lower bound must precede a cast-based
upper bound. No intervening statement is permitted except a single immutable
non-shadowing local. Parameters and locals cannot be mutated or captured. All
body attributes refuse. Source is capped at 64KiB, statements at 32, boolean
condition recursion at 16. Other expression shapes refuse.

For r of signed width w, the surviving path proves r>=0 or r>=1. The only index
arithmetic is r, r+0, or r-1. For subtraction r>=1 proves definedness and a
nonnegative result; addition of zero is defined throughout the signed range.
The upper guard proves r<len or r<=len. Zero-based indexing requires the strict
bound; one-based r-1 is strictly less than len under either bound. Empty slices
therefore cannot reach an accepted index.

A separate scratch build replaces ONLY the index with:
`{ let _: [(); 1] = [(); (::core::primitive::usize::BITS >= w) as usize]; (index) as usize }`
This array type equality must typecheck in the selected function on the actual
Cargo compilation target. It proves w<=usize::BITS, so nonnegative r and its
proved subtraction result are exactly representable, and the ORIGINAL guard's
cast also cannot truncate. No host sizeof inference, check branch, unwrap, panic,
new default, or new error handling is involved. The array witness is discarded;
only the cast is proposed. The proof check uses the existing public run_in API
with cache=None, so environment-insensitive cached successes cannot establish width.
Existing copy, isolation, compiler execution, ambiguity and cache implementations
are unchanged. Fresh existing build verification still follows the proposal;
compiler-suggestion iteration is disabled for this new atomic repair only.

The conventional library manifest is required; local/ancestor Cargo configuration,
build scripts, custom targets and other manifest fields refuse. An environment or
Cargo-home target selection is checked by the native array witness itself. This
proves the configuration actually checked, not every future compilation target.
Cross-compilers were not installed or exercised. Changing a target after proposing
or accepting code is outside the guarantee, as is concurrent external source or
compiler-toolchain mutation. The helper does not claim general dataflow analysis.
