# Relative-rate preregistration — before runtime edits or DEV

Base HEAD 336d5f9ee5ff70b208d7e2b78ee0b582c6f04a2a. Production quantities
687f baseline:22 correct/4450 refused/3 wrong; frozen ratewords v3:28/4444/3.
Ratewords remains historically rejected; applying its exact preserved patch is
only a starting point for this new experiment, not an acceptance claim.

Hypothesis: dimension-checked relative velocity and complete crossing of two
explicitly bound moving extended bodies adds >=3 correct beyond known six.
TRAIN design access: Problem column only, first4000 public MathQA TRAIN rows;
deterministic same/opposite direction + train filter, saved train-evidence.json.
No answer columns, official test, hidden code, test generators or private logs.

Frozen scope: one complete passing event for two trains, each with known positive
length and speed. Collective ordered pairs bind first/second; named entities
bind uniquely, no third entity. Explicit opposite directions adds speeds;
explicit same direction uses absolute speed difference. Equal speeds refuse in
both forms as required. Complete passing traverses sum of lengths, assuming
uniform straight-line motion throughout the explicitly described event. No
start delay/gap, acceleration, speed changes, driver/person targets or implied
direction. Query explicitly requests duration units (seconds/minutes/hours).
No default units. Unknown lengths/speeds, extra events, incomplete clauses,
contradictory rates, ambiguous names and referents refuse. Optional parallel
tracks describes geometry, not extra evidence or inferred direction.

Grammar: fully consumed clauses: two trains [named X and Y] [are] moving/running
[in explicit direction] [at/with speed(s) of] two explicit speed quantities
[respectively] [in explicit direction] [on parallel tracks/lines]; their lengths
are two explicit length quantities respectively. Lengths may precede speeds,
or occur after two trains as two quantities long. Named single-train length and
speed declarations permitted with explicit joint direction clause. Query: time
[taken by slower/faster train] to cross/pass [the other/each other/the faster or
slower train] in explicit duration unit; or in how many UNIT will they cross
one another. Unknown words/clauses do not get discarded. Numeric coefficients,
units, entity names and clause order are variables, no digit search/IDs/tables.
Core uses checked rational typed Length/Duration/Speed affine expressions,
<=2 unknowns and existing bounded equalities, no nonlinear unknown products.
Old scalar/ratio/absolute/inventory/date/percentage paths remain unchanged.

Independent pre-DEV authored probes: opposite100m+200m at36+72km/h=>10s;
same100m+200m at72-36km/h=>30s; mixed1km+500m at1km/min+5m/s=>900/13s;
reordered named Cedar/Maple with45m+75m at7+3m/s=>12s; fractional values,
minutes conversion; refusals for equal rates, missing direction/unit/extent,
negative values, conflicting speeds/directions, duplicate names, third entity,
station/person crossing, start delay and changing speed. Old ratewords43 probes
and parent known six remain required. Probe expectations frozen before DEV.

Acceptance: full4475 under UNCHANGED exact-label-v3 >=31correct, no new wrong or
unscorable answered rows versus frozen31 ratewords answers, all28 correct retained,
no production22correct loss. All3116 synthetic status/value/unit triples identical;
oldlinear50/absolute55 checks and new semantic probes pass. First100 then full4475;
freeze exact source/binary before either DEV run, no post-full retuning. Save
complete predictions before scoring. Validate frozen metric hashes, full question
hash/order/count and baseline sparse31/25records (missing indices imply refusal).
Also report unchanged legacy metric with full4475/1229unparseable denominator.
Independently derive every newly answered English problem separately from labels.
Physical derivations do not make label equality proof. No gate relaxation.

On failure preserve exact patch/source/binaries/results/hashmanifest, then restore
ALL quantity runtime to current base including old ratewords patch. Evidence only.
Final scoped fmt/clippy/tests and mainCLI build; parent owns integration regressions.
No commits, agents, model calls, credentials, new data/deps or background services.
All cargo: disk TMPDIR, existing CARGO_TARGET_DIR, systemd scope8GiB/0swap,
setsid --wait timeout --kill-after=10s 600; -j1 except fmt; testthreads4.
