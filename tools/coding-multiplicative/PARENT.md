# Multiplication/division repairs and ambiguity beside unit tests

Accepted coordinator revision: the repair search now proposes binary multiplication
and division swaps after every existing candidate, across all files and lines.
It applies a candidate only after the unchanged checks pass. Independent authored
cases demonstrate eight new exact repairs; full765 development outcomes remain
200pass/565refused/0fail. This is added repair scope, not a benchmark score gain or
proof that passing tests establishes the intended behavior.

## Original submission and the independent failure

Before inspection, parent archived709 files/79,969,637bytes and verified their
SHA256 identities atWORK/multiplicative-run/terminal-originals, where WORK is
`/home/gsc/.cache/glassmind/continuous-20261010`. The original binary/source,
worker reports, tests, logs and all native evidence remain immutable.
Original binary:deec095b2261610795dd4ebbe71851d378b698710d5135256a574f4668192f06.

The worker's eight authored new repairs and six negatives passed; its public
TRAIN sample/full27 remained2/4 and5/27 verified, with no changed verdict or patch.
These are public native checks, not hidden benchmark correctness. Worker39tests,
fmt/clippy/release evidence is retained, including the initial wrong-package build
failure. HANDOFF describes that original submission, not this coordinator revision.

Parent froze14 independent crates before inspection. All14 compiled and failed
executed original assertions. Original candidate repaired8/8 new and1/1 legacy,
left4/4 negatives unchanged, but accepted an ambiguous repair. The same function
`x / 2 - 3` can become `x / 2 + 3` or `x * 2 - 3`; both return5 at4 and disagree
elsewhere. Worker used a separate integration-test file. Parent used an inline
`#[cfg(test)]` module, which caused the old ambiguity parser to skip the whole file.
The original submission fails the frozen parent gate; no expectation was relabeled.

## Coordinator correction and evidence

After recording a revision plan and original hashes, parent allowed only inline
sibling modules with exactly one `#[cfg(test)]` attribute beside an analyzed scalar
function. Such a module has its own lexical scope. Other root modules, imports,
type declarations and unsupported attributes retain the existing rejection.
Native compilation and executed witnesses still check actual package bindings
and both outputs. Interpreter, input sampling, search limits and original tests
are unchanged. The correction does not treat sampled agreement as equivalence.

The unchanged14 parent fixtures now pass:8/8 new and1/1 legacy exact source repairs
with native tests,4/4 negative crates unchanged, and ambiguity withdrawn with native
evidence. Signed/unsigned integers, floats, production modules, Unicode spans,
macros, test-only operations, multi-edit refusal and zero-divisor refusal are covered.
A separate replay rebuilds each proposed program with its unchanged visible tests,
then independently confirms both differing outputs on the retained witness input.
Original source, tests and manifest remain unchanged when ambiguity withdraws.

Revision validation passes40 world-rust tests (35library+4isolation+1scope test),
fmt, all-target crate clippy and release main build. After copying exact frozen
source to the root checkout, workspace fmt/all-target clippy/release and strict
4/4 fixture smoke pass. The original worker TRAIN run is not relabeled as a new
coordinator TRAIN run; the full main-CLI development evaluation below covers the
final source. No extra dependency or runtime model was introduced.

Full765 via unchanged tools/rustbench-dev.sh:200pass/565refused/0fail. Every one of
the previous200passes remains, all765 outcomes are identical, and there are no
new failures. All prior per-kind scores remain, including fix-test15/81. No hidden
test, heldout item, expected patch or hidden generator was inspected. Full results
SHA c2f9fbd3aba3f11558e9872973b6fb0aa4d3714d7ce9c79d5f3a95e58b9625f6.

## Exact artifacts and limits

Evaluated frozen revision release:
825df340f06a81024d4620fee92cd5cea7a0813abd675cf3f2cfe59a15f63cc6,
WORK/coordinator/multiplicative-independent/revision/frozen/glassmind.
Integrated root release:
7a3d98dfb4eb6428d9d43ceb4cbdb963d20750034d0da223ffaa21dbe9781d81,
WORK/coordinator/multiplicative-integrated/glassmind.
The four changed source/test files match the frozen revision byte-for-byte;
parent/integrated/manifest.json records their hashes. Full765 used the frozen
revision executable, not an unmeasured claim about the integrated binary.

Parent fixtures, original/revised results, plans, comparison adapter, native witness
report and build logs are under parent/. The independent native adapter is run
under a capped scope with a fresh output directory; native subprocesses also use
8GiB/0swap, setsid--wait, bounded timeouts, on-disk TMPDIR, cargo1/testthreads4.
Original full worker evidence and both coordinator release binaries remain external.

Only AST binary */ expressions are new proposals; dereferences, types, assignment
operators, strings, macros and protected test code are not rewritten. Existing
candidate order is preserved globally. Search remains bounded, so new candidates
can be untried after the old budget is exhausted. Ambiguity detection remains a
bounded scalar subset: floats, complex bodies and other module forms are not
proved unambiguous. Untested division by zero, overflow and unintended semantics
are not ruled out by a passing test suite. No broad correctness claim is made.

No external data/model downloads. Original worker fixture/source provenance is in
HANDOFF; parent fixtures are authored development checks. Completed worker usage
2,279,238input (including2,198,144cached)+21,878output=2,301,116. After both quantities
experiments, cumulative completed worker usage is161,484,271; parent/interrupted
usage remains excluded and cached input is counted once.
