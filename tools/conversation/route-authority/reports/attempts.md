# Attempts

One runtime candidate, frozen before its single release build. No source edits
or rebuild followed dev inspection. The original archived candidate remains
rejected for losing three correct learned proposals (1303 ->1302 overall).
This revision removes its blanket fallback veto and adds a state-derived actor
exception for supported coordinated movement; no social grammar expansion.

Validation orchestration race: the candidate model fixed boundary sample passed,
but the next full boundary invocation attempted to read the still-running full
baseline's records.json. It failed before invoking any candidate test process.
The raw traceback remains in gates-boundaries.log; its empty output directory is
retained as candidate-boundary-model-unstarted. Resume only the unstarted full
comparison after the baseline finishes, then the supplemental stale-answer gate.
No full behavioral run is repeated and no expectation is modified.

Early baseline boundary invocations used supplied persistent TMPDIR but did not
explicitly export GLASSMIND_SCRATCH; these invoke only talk/session operations,
with explicit session paths in owned reports. All cargo and subsequent gates
explicitly export the owned disk scratch. No generated code or scratch build was
triggered by those early baseline turns. No /tmp storage was requested.

The first plain candidate sample output directory was renamed from candidate-first-sample to candidate-boundary-sample without changing its contents. It was not rerun.

The model gate shell exited2 after all its five scorer commands completed
successfully (sample126, full490, memory93+41, goodbye7). Its log ends with an
unexpected-EOF diagnostic. The runner file had been edited in place during that
long-running shell to add configurable output paths and avoid a duplicate plain
sample in a different branch; Bash had already parsed the original case body.
The diagnostic is consistent with a stale end-of-file offset when Bash
resumed reading the edited file. The current final script passes bash -n. No evaluator or runtime source
was edited, and none of those full runs was repeated. check_gates.py independently
asserts every completed report and frozen identity; it returns PASS. The wrapper
exit is an execution limitation, not claimed to be a clean consolidated run.
