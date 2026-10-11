# Frozen work-rates plan (before implementation or DEV)

Source: 65e7240b074473ef761e6a81a64bbf64c2fb03f6. Hypothesis: exact signed
whole-job rates with fully consumed agent/time clauses and a single explicit
stage generalize across worker and tank problems, adding >=5 v3 correct DEV.
Baseline supplied: full4475 29correct/4443refused/3wrong, 32 answer payloads;
first100 1correct/99refused. Acceptance: >=34correct, all32 payloads identical,
no new wrong/unscorable, independent Problem-only audit of every new answer;
all3116 synthetic status/value/unit identical, existing50linear/55absolute/
38percentchange and available78parent cases preserved. Meaningful frozen
positive/adversarial probes all pass. If gate fails archive patch/binary/results
and restore runtime; no post-DEV rescue.

Grammar: 2..3 distinct arbitrary ASCII agent identifiers (optional worker,
machine, pipe, pump prefixes); individual completion declarations; coordinated
name/time lists (respectively or positional listing); named combined-time plus
individual declarations to infer one missing rate; explicit one initial stage
followed by named departure or named continuation. Queries: simultaneous full
completion; one named agent alone; remaining duration after a stage; total time
including stage. Whole work/job/task/piece of work or tank/cistern shared by
explicit same/the/it references or repeated conventional work noun. Exactly one
work kind per problem, no changed object. Fill positive, empty/drain negative;
fill query begins empty, drain query begins full; explicit unsupported partial
initial volume refused. Combined known fill time and named drain can compose.
One unknown rate only, exactly one combined constraint; redundant/conflicting
agent declarations refused. 2..3 agents only. Start stage can be one agent then
all, or all then one leaves; no implicit schedule. Stage may not exceed physical
0..1 capacity; if completion is reached before planned departure, total ends at
completion and remaining is zero. Negative net flow from empty is refused.
Hours/minutes/days/seconds convert exactly, query unit wins else first evidence
unit. Constant independent rates assumed; varying rates, efficiencies, ratios,
wages, partial jobs/initial fractions, cycles, symbolic times, multiple queries,
unknown references, unrelated numeric clauses and unsupported words refused.
No extracting arbitrary numbers or names, no gold/choices/runtime lookup.

Before DEV freeze source, binaries, adapter and probes. Run first100 then full4475
once, Problem-only inference persisted before loading options/correct for scoring.
Unchanged typed-label-metric/FREEZE identities and legacy functions. Supplied full
predictions replace obsolete sparse baseline only; denominator/order identical.
All jobs bounded8GiB/0swap,setsid--wait,timeout600/kill10; cargo -j1/tests4/TMPDIR.
No downloads, agents, model calls, commits, main edits or shared report edits.

Data access note: TRAIN inspection projected Problem only, first4000. An accidental
full source-card read displayed one unrelated annotated combinatorics example;
not used for grammar or implementation. No TRAIN annotations otherwise accessed.
