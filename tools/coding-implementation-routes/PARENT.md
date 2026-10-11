# Parent decision: rejected

The candidate remains rejected. Independent replay reproduced all 46 new
boundary results byte for byte: 36 pass, 10 fail. The implementation parser
matches 44 of 46 dispositions. Both dot-qualified field-name probes incorrectly
reserve an implementation request. No runtime change or full765 result is
accepted from this batch.

The frozen positive fixtures also contain unsupported baseline assumptions:
function/field name ambiguity, an unrecognized rename destination, a macro-bearing
source for declared types, and unsupported AddField wording. Those failures are
preserved, not rewritten into successes. Exploratory isolated rename probes in
the worker report are separate from the original gate.

Parent verified 65 archived files, 63 manifest entries, six frozen entries and
1,148 raw artifact identities before replay. Source and candidate binaries remain
in the original archive/run. The prior 20 authored, 34 parser, 34 CLI, six
environment and 3/60 TRAIN results are worker measurements, not newly repeated
parent checks. The three TRAIN successes cover one function in three styles.

Reproduction: run `boundaries.py` with the retained `glassmind-candidate` and
`parser-probe` under an 8GiB/0swap scope, persistent TMPDIR, explicit disk scratch,
setsid --wait and timeout --kill-after=10s 600. Expected exit is 1. Parent raw
results: `/home/gsc/.cache/glassmind/continuous-20261010/coordinator/implementationroutes-review`.
[Verification](parent/verification.json) records exact identities.

Three implementation-family experiments have now been rejected. The next coding
experiment changes scope to exact identifier roles and independently established
baseline behavior; it does not revive this candidate or weaken its gate.
