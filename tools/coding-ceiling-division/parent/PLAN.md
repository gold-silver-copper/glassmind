# Independent unsigned ceiling-division repair contracts

Frozen before candidate inspection:8positive crates,4negative crates,1ambiguous.
Allpositive arithmetic is computed independently with quotient+nonzero-remainder;
no n+d-1 intermediate, so allsix unsigned primitive maxima are covered. Zero
numerator/exact/inexact division and existingzero-denominatorguard, typedfields
andUnicodeparameters are required. The negativecustom Div has a matching
custom div_ceil method that could passpublictests; the new operator is restricted
to actual primitive unsigned arithmetic and must not call that custom method.

Gate:8/8nativepassing repairs with exactlythe one intended quotient changed;
all other source/tests/manifest bytes preserved. Accepted equivalent emitter forms:
fullyqualified core/std primitive div_ceil(left,right), or primitive receiver
(left).div_ceil(right); harmless outer parentheses/whitespace may vary. The type
is already pinned by each fixture. The adapter may parse/normalize only that
replacement expression, never broaden allowed edits or rewrite cases after outcomes.
Fournegativecases unchanged. Finalcase mustwithdraw with retained native witness:
ceil(x/2)+floor(x/3) andfloor(x/2)+ceil(x/3) bothreturn1at1 butdifferat2.
All13originals mustcompile andfail executedvisibleassertions, not merely failbuild.
These are authoredscope checks, not hiddenbenchmark or universalcorrectness proof.
