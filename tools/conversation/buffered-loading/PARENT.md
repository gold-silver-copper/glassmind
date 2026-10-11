# Buffering preserves behavior and reduces fresh-process startup

The talk JSON loader now wraps File in BufReader. It uses the same serde JSON
parser and model validation; no model format, weights, threshold or cache changes.
The installed serde_json1.0.152 IoRead implementation uses reader.bytes() and
explicitly documents that callers should buffer file input.

Five interleaved fresh-process runs with the identical25MB model and two fixed
inputs have median2.5866s before and0.09331s after, about27.7x faster in this local
probe. Other development jobs were active. This measures fresh processes on this
machine, not an isolated benchmark, cold filesystem cache or universal speedup.
Every measured response byte is identical.

All3,100 main-talk validation replies are byte-identical to the unbuffered routing
candidate, including proposal scores and margins. Ten malformed/missing model or
session cases preserve rejection status and output. An existing saved session
produces identical replies and persisted bytes after loading. Full story and
ordinary facts outputs, both490-check conversation suites, both134-check memory
suites and the remaining routing gates pass on the final buffered binary. The
final workspace checks and17conversation tests pass. See ../route-authority/parent
for full checks and exact source/binary identities.

The unfinished slow model suite was explicitly interrupted and preserved; it is
not a passing result. The final candidate completed every required suite in a new
output directory. Earlier completed measurements were not relabeled as buffered.
Raw timing/compatibility artifacts live under
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/model-load-review`.
No new tests duplicating the implementation were added; validation uses actual
process startup, failure behavior, persisted sessions and the existing full gates.
