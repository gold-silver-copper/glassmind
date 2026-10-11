# Coordinator revision: retain ambiguity checks beside inline tests

Original five-line multiplicative submission fails original parent14case gate:
8/8newrepairs,1/1legacy,4/4negatives, but native_ambiguity changes '-' to '+'
instead of withdrawing. Originals and outputs remain immutable. No fixture relabel.
Source diagnosis: ambiguity::difference rejects any non-function root item,
including the sibling #[cfg(test)] mod tests. Worker used integration tests in a
separate file, so its ambiguity case passed. No benchmark regression was run yet.

Permit only inline root modules with exactly one #[cfg(test)] attribute as
nonparticipating siblings when locating a scalar function. Keep other module,
use/type/extern/macro declarations rejected; no changes to interpreter, sampling,
resource bounds, original tests or mandatory native witness confirmation.
Sibling module contents cannot introduce a binding into the root lexical scope;
root imports remain rejected, macro calls in expressions remain unsupported,
and native compilation/executed witnesses still validate actual Rust context.

Gate: rerun original14 unchanged fixtures, all8newexactnative,1legacy,4unchanged,
plus ambiguity withdrawal with two native outputs confirmed. New targeted unit
contrasts cover plain inline test module, ordinary module rejection, out-of-line
module rejection and extra module attributes rejection. Existing shadowed
primitive/native-context unit tests remain. Then full765 preserveall200passes
and0failures, workspace checks before merge. Retain original source/binary/test
failure and report this as a coordinator revision, not the original submission.
