# Independent multiplicative repair contracts

Frozen before candidate inspection. Eight new repairs cover both operator directions,
signed/unsigned/float/nested-module/Unicode cases; all require the exact one-token
source change and native unchanged tests. Four negatives require no edit (opaque
macro, tests-only, two necessary simultaneous changes, division-by-zero candidate).
Legacy +/- repair must retain its exact patch. The final fixture admits two public
test-passing repairs with a defined divergent scalar witness: x/2+3 versus x*2-3
both return5 at4 but differ at2. Require ambiguity withdrawal with native evidence,
not a claimed uniquely correct repair. Baseline may accept the old first patch.
All14 original crates must build but fail their visible test. Freeze the originals;
do not relabel cases from candidate outcomes. Gate:8/8new exact native repairs,
4/4negatives unchanged, legacy exact repair retained, ambiguity withdrawal backed
by the native witness record. This is authored scope, not benchmark accuracy.
