# Signature-directed composition of two integer sequences

Frozen before implementation. WORK/sequence-composition, branch
continuous/sequence-composition; external RUN=WORK/coordinator/sequence-composition,
where WORK=/home/gsc/.cache/glassmind/continuous-20261010.
No models, downloads or external workers.

The approved RustBench TRAIN placeholder for merging sorted sequences takes two
integer slices and returns a vector. The current generator rejects every two-argument
signature before producing candidates. Its documented task motivates adding a
reusable sequence-composition family, not a task-specific implementation or runtime
reading of names, requests, documentation, tests or expected results.

For exactly two plain immutable slice/owned Vec arguments of the same primitive
integer element type, returning Vec of that same type, generate these 16 distinct
semantic compositions:
- Concatenation in either argument order, each forward or reversed: four.
- Sorted concatenation, ascending or descending, retaining duplicates: two.
- Sorted set union, ascending or descending: two.
- Sorted intersection, ascending or descending: two.
- Sorted left-minus-right and right-minus-left differences, each direction: four.
- Sorted symmetric difference, ascending or descending: two.

Set operations remove duplicates. Use native standard-library integer ordering,
iterators and BTreeSet without arithmetic on element values. Mixed slice/Vec
containers are permitted; mismatched elements, other returns, mutable references,
receivers, destructuring, generics, unsafe/async/extern and additional arguments
remain unsupported. Preserve every one-argument candidate exactly. Argument names
only bind inputs; function names and body/request text never choose a composition.
Keep32candidate/90second budgets, exactly-one-survivor rule, opaque project-test
verification, target execution, final live check and rollback unchanged.

Before edits, freeze independent authored fixtures for all16 compositions, using
unsorted data, duplicates, overlaps/disjoint sets, empty sides, negative values,
integer boundaries and different container/element types. Include raw binding
names, weak-test ambiguity and unsupported/already-implemented cases. Existing
word-initial13 plus prior76 native checks must remain correct. Every accepted
proposal is independently recompiled; all other source/test bytes stay unchanged.

Baseline main executable SHA256
0d551879056cc5e0356b7e36adea1f274c2e0e30182122aa538dfb4b895dbc1b;
world implementation source remains0cc70e953702944b618c8804c19ff5b1e0e09b5ce3a87e35ed7f102e4301e456.
Reverify retained approved TRAIN source hashes. Fixed TRAIN sample contains all
three request styles for merge-sorted-unique, sum-even, reverse-words and initials;
then full60TRAIN. Require at least one new accepted implementation, all six prior
successful outputs byte-identical and no unintended source changes. Independently
recompile every accepted TRAIN output and inspect verifier evidence.

After native/TRAIN gates, all world tests and required workspace checks pass, run
the same full765DEV. Require all221previous passes, at least one gain and zero
failures. Report task/TRAIN overlap; this is not unseen-task generalization.
No official held-out items, hidden tests, generators or expected patches may be
inspected. Failed candidates retain original artifacts; do not weaken checks,
change parser rules, increase budgets or expand operators after DEV results.

Every cargo, including generated-project tests, uses8GiB/no swap,setsid--wait,
a finite timeout with descendant cleanup, private on-disk TMPDIR, at most4jobs
(and8project-wide), at most4testthreads. Runtime verification keeps its stricter
existing limits. Bound native/TRAIN/fullDEV separately using established wrappers.
