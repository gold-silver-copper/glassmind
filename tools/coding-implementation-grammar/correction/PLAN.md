# Separate recognizing an implementation request from accepting its declaration

The archived implementation-grammar submission remains rejected: the native
parser accepts `fold_values needs a body, implement it` and `fold_values needs
implementing, implement it` although the frozen declaration grammar excludes
both. Independent checks otherwise pass 12 routing controls and 18 native cases,
including six implementations and twelve unchanged refusals. No DEV run occurred.

One bounded correction: represent an unsupported recognized declaration separately
from a declaration eligible to bind the implementation action. Keep both extra
forms as explicit refusals. Preserve all original accepted forms, all original
refusals and other-route boundaries. Do not change signature-derived candidates,
native verification, source eligibility, trial budgets or semantic scope.

Before editing, freeze the original source/plan identities. Add direct regression
cases for both rejected forms, with and without a trailing action, and retain
the original 62 parser contracts. Run the affected Rust tests, original 34 CLI
boundaries, independent 12 boundaries and 18 native cases. Require exact baseline
preservation and all six independent implementations. Recheck the fixed TRAIN
sample then full 60: at least three verified requests, all refusals unchanged.
Then run the original parent DEV gate for the first time: >=218/765 passes,
all 215 baseline passes retained, zero failures. No gate changes after scoring.

All original archives remain immutable. Use a fresh worktree based on root HEAD;
apply only candidate runtime files and preserve the accepted quantity prefix fix.
Complete source review, saved choice evidence, required workspace checks and
exact-head CI before any merge. Root integrates only after those gates pass.
