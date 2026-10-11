# Rejected bounded implementation experiment

This batch is **evidence only**. The runtime remains at the starting revision;
`glassmind do PATH "implement NAME"` is not shipped by this experiment.
See `tools/coding-implement/HANDOFF.md` for exact artifacts and reproduction.

The archived candidate generated native standard-library pipelines solely from
one parsed argument and its return type. Integer slices/vectors supported
iteration, reversal, primitive filters, sum/count, collection and all/any.
Strings supported chars/split_whitespace with reversal and collection/count.
Function/parameter names served only target selection and binding; the generator
received no request, documentation, tests, benchmark identifiers or expected
patches. Original tests ran as opaque native checks.

The package gate conservatively required one conventional dependency-free
library, with no production imports, aliases, traits, macros or module graph.
Only a unique free function whose entire body was a bare todo!/unimplemented!
was eligible. Comments and unrelated bytes were retained. The baseline had to
execute at least one failing test. Every candidate was tried in scratch; multiple
passing candidates caused refusal, including potentially equivalent programs.
A unique survivor was explicitly only a test-verified proposal, never a proof
of intended semantics. Application checked package identity, source freshness,
rule switches and fresh nonempty tests, restoring original bytes on failure.

Frozen authored sample: 3/3 implementations. Full authored suite: 20/20 checks,
with six implementations and fourteen safe refusals. An extra absent-Cargo probe
refused without mutation. Scoped Rust tests: 76/76. These are engineering
fixtures, not benchmark results or an independent semantic generalization claim.

The frozen release gate required at least 177/765 passes, preserving the previous
174 and zero failures. A subsequent TRAIN request syntax audit found that the
candidate parser accepted **0/60** implementation requests. Terse requests quote
names with backticks; other styles need a broader, fully consumed language bridge.
The finite pipeline family is also much narrower than those requested algorithms.
No full benchmark was run, no gate was weakened, and runtime changes were archived
and reverted. Quoting alone would not establish the required coverage. Any attempt
to ship this API needs a new prospective experiment and the original parent gate.
