# Test-verified function implementation

`glassmind do` can fill a selected whole-body `todo!()` or `unimplemented!()`
placeholder when existing tests identify exactly one implementation from its
bounded, signature-directed candidate family:

```sh
glassmind do ./my-library 'implement sum_even'
glassmind do ./my-library 'Please write the body of `sum_even` in src/lib.rs.'
glassmind do ./my-library 'sum_even is a stub, implement it'
```

The function signature supplies candidate operations over integer slices/Vecs
and strings: filtering, reversal, sums, counts, boolean reductions, character
sequences and word sequences with concatenation or single-space joining.
Word sequences can keep their order or reverse it. The request selects the target. Existing tests
verify candidates; doc comments and assertions do not choose an algorithm.
The result reports test verification and links to the saved evidence.

The current scope is a standalone dependency-free library with a unique free
function in `src/lib.rs`, an explicit supported signature and a whole-body
placeholder. Existing bodies, qualified owners, unsupported types, production
aliases/imports/macros, behavior constraints and compound actions refuse.
Quoted identifiers preserve words such as `write` and `it`; a function literally
named `function` also works as an unquoted target. The supported declaration is
`NAME is a stub` (or a `todo!()` equivalent); `needs a body` is not an accepted
declaration form.

The original tests must compile, execute, fail, and reach the selected target.
Every candidate is checked from the original package bytes. No tests, an
unexecuted target, compiler errors, timeout, zero passing candidates or multiple
passing candidates leave the package unchanged. A unique proposal is checked
again on the live package; failure restores the original files. Tests and other
source bytes remain intact. This establishes the recorded tests' behavior,
without proving all English requirements or behavior on untested inputs.

The full 765-request coding DEV run reaches **220 passes, 545 refusals and zero
failures**, retaining all 218 previous passes. The two new gains are styles of
the reverse-words task also present in the supplied TRAIN material. Ten new
authored checks cover both joining behaviors, Unicode, whitespace, existing
string operations and ambiguity; all 66 prior native checks remain passing.
These are separate from the benchmark score.
[Source, validation and limitations](../tools/coding-word-join/PARENT.md).
