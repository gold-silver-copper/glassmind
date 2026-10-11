The initial baseline command omitted --sample and completed all27 first (9/27).
Its original output is retained under baseline-first-full. A redundant full
command was started before noticing the count; it completed using shared cached
observations before the exact-process-group stop attempt (no process remained).
Its output is baseline-full. This violated baseline sample-first ordering and
unnecessarily duplicated the full adapter invocation. The correct fixed sample
was then run with the existing content-fingerprinted cache. No expectations or
input selection changed. Candidate sample must precede candidate full.

No proposal/fixture expectation was changed in response to a failed test.
After initial implementation review, added a separate comparator-only veto
observation for compiler findings in older families. The old native choice
semantics remain unchanged; a new native control explicitly verifies this.
Review also tightened function type-path syntax to refuse qualified/associated
paths and unknown generic constructors. Repeated world-rust tests and clippy
because production changed; no TRAIN candidate evaluation had run yet. An fmt
check caught line wrapping in that addition; its failed output is retained as
fmt-final.log, then cargo fmt corrected whitespace. Initial and final native
expectations are unchanged. No learned/heuristic or broader family was tried.
