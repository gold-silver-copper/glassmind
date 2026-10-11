Supplement frozen before candidate execution, after runtime change: strengthen two edge cases
without changing the original sixteen fixtures or their acceptance threshold.
1. variable-zero-divisor: 8*x, tests require f(0)=0 and f(2)=4. Swap to division
passes the nonzero input but panics at zero; must refuse and preserve the tree.
2. compile-failure-then-success: an earlier production Only*4 expression has Mul
but no Div implementation. A later x/4 needs multiplication; skip the compile
failure, repair only the latter expression, pass multiple signed/zero inputs.
These are supplemental diagnostics, excluded from the original 8-positive count.
