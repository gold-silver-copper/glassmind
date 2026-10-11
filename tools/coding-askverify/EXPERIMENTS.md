# Baseline and rejected approaches

Frozen runtime baseline: 1/4 sample, 6/16 full authored contracts. Broken and
multifile packages were written without verification, disabling rust.apply.changes
was ignored, JSON had trailing plain text, successful session retained Proposal.
Raw original-run.py and initial results remain in the worker artifact directory.

Fixture preflight: an attributed test module and a macro containing a function
reference are outside existing rename support. Retained that original fixture
and its assert_eq! unchanged as two refusal regressions and a field success case.
Added a separate bounded, unattributed public package for struct/function cases.
No runtime proposal checks weakened.

Harness correction BEFORE first candidate execution: two plain-text checks had
assumed the phrase "build passed", which isn't the existing language's Applied
rendering. The existing grammar says "I made ... No problems with the build."
The corrected check requires BOTH the actual applied statement and the verified
build statement. Baseline rerun retained; original adapter/results preserved.
This corrects a wording bug, not the acceptance contract or underlying assertions.

Rejected implementation options: retain direct writer plus add CLI cargo runner
(duplicates verification and misses rule/rollback ownership); synthesize a yes
turn (alters user/session history and may apply a different pending action).
One runtime candidate: existing World::apply plus replacement of actual turn reply.
