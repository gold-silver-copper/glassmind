# Repair scope revision accepted by full development coverage

The bounded speech-act filter removes Repair readings for explicit negation,
retraction, deferred permission, quoted/reported instructions, explanation requests
and competing operations. Complete supported status questions become typed checks.
It applies to ordinary interpretation, guesses and the learned coding fallback.
Legacy affirmative and implicit-report inference remains available. The old blanket
rejection of “without” is removed so behavior-preservation constraints can work.
No repair operator, model weights, training data or benchmark definitions changed.

The worker's original source and executable remain archived and rejected: fresh
independent24 cases exposed three unrequested native edits and one missing status
meaning. Its original30 suite missed one status meaning. The coordinator added
source-preservation retraction, conditional “once” clauses, negated request/permission
predicates, and status ability/test-suite vocabulary. Thirteen contrasts were frozen
before that revision; they are post-discovery regression cases, not blind validation.
Original worker32+9, old68, public legacy15, TRAIN172 and parent30+24 labels remain.

Both parent native suites now pass the documented gates:16/16 exact positive repairs
with original assertions passing, no unrequested edits or negative command previews,
all10 typed status queries, and initial ask--apply remains read-only. Original30
status proxy v1 remains5/6; the previously documented v2 Problems-vs-Passes correction
is6/6. Original baseline/candidate scores and the corrected proxy are retained.
TRAIN typing159→167/172, old68 34→63/68 and legacy15/15 are unchanged from the worker's
final typing coverage, with no baseline successes lost.

Full fixed coding DEV is **200/765 pass,565 refused,0 failed**, preserving all180
previous passes. All20 gains are chatty variants of existing compiler-repair tasks.
Full raw results are external at WORK/coordinator/repairveto-full/results.jsonl,
SHA256 cf7a8a25c7d90d53e02ada1ca8fb66c7a7e2307b6416c91f506321e6927357ac.
The unchanged runner is tools/rustbench-dev.sh.
No official held-out run was used to select this change.

Evaluated frozen release65750260c5db5a5838704cde8cd6cd17f16d11980e53005644d5df3cd7015f44,
sourcef53651abb4c96c7be2885eeac1e52adbaa643a583c4a8297f783a0aec004d8e6,
at WORK/coordinator/repairveto-review/revision/frozen. Worker branch base6a346c3;
source copied exactly into the coordinator checkout after full comparison. Scoped
22CLI+40English tests, fmt/clippy/release passed before inference. Combined-root
checks and release integration are recorded separately before publication.

This is measured scope improvement, not a general proof of English authorization.
Original old68 still has five typing mismatches, including underspecified commands
and implicit failure reports. Original native rows54,55,57 still edit under their
unchanged negative labels. Their original outcomes remain visible; no all68-pass or
universal no-unrequested-edit claim is made. An earlier strict positive whitelist
lost127/180 successes and was rejected; those artifacts remain separate.
