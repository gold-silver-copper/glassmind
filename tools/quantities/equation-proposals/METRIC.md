# proposal-numeric-label-v1

This experiment has a distinct proposal metric. It is not ordinary solve accuracy.
The unchanged exact-label-comparison-v3 comparator is imported read-only from
`tools/quantities/typed-label-metric/metric.py`. A separate offline scoring phase,
after predictions and their manifests are saved, maps a `proposed` exact result to
the comparator's `answer` input sentinel with `unit=None`. That temporary dict is
never emitted by the Rust proposer and never enters solve/talk. DEV scoring reads
only options/correct; inference receives only question strings. The driver uses
separate column reads/phases in one Python process, not separate OS processes.

Every official DEV row remains in the 4475 denominator. Reported `wrong` and
`unscorable` are disjoint comparator outcomes; conservative wrong is their sum.
Explicit-unit labels without a predicted unit remain unscorable, even when the
coefficient agrees. Unsupported labels also remain unscorable. Refusals are a
separate count. Scalar coefficient agreement does not establish English entailment,
requested-unit correctness, or a physically meaningful interpretation of constants.

Selection considers only executable proposals. On ALL 5925 TRAIN calibration rows,
score ties are grouped by their exact stored floating-point score, and selected
coverage is maximized subject to >=90% exact-label precision and >=30 selected.
Unscorable selected proposals count wrong. No qualifying threshold means disabled;
zero selections have undefined precision, never 100% precision. These are benchmark
selection scores, not calibrated real-world confidence. Candidate and each lexical
baseline select independently. Both raw and selected comparisons use the stronger
baseline, never just the restricted bank.
