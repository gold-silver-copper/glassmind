# Determined answers with unknown operands

Glassmind can answer a supported arithmetic question when the requested value
is fixed, even if individual numbers remain unknown:

```text
The sum of two numbers is 10. What is twice their sum?
20
```

Asking for the first number in that example still refuses. Likewise, knowing
that the difference between two numbers is 3 determines their absolute
difference, but does not determine the first number minus the second.

This works in `glassmind-quantities` and `glassmind talk`. The existing consumed
English grammar supports up to two unknown numbers, eight equalities and four
absolute differences. Exact rational arithmetic checks the possible values of
the requested expression. An ambiguous answer, inconsistent constraints or
arithmetic overflow refuses. Numbers are not assumed positive or integral.

The authored checks pass; MathQA DEV remains 29/4,475 with its previous answers
unchanged. This extends exact reasoning within the supported grammar, rather
than establishing broad word-problem coverage.
[Validation and limits](../tools/quantities/target-determination/PARENT.md).
