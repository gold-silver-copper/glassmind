# Rate-clause composition rejected

The candidate fails the unchanged full DEV and independent language gates.
Do not integrate its runtime patch. Full4,475 v3 results are31 correct,4,440 refused,
4 wrong, against the approved29/4,443/3 and required34 correct with no new metric
errors. All32 old answer objects and raw JSON answer lines remain identical.
Legacy scoring separately reports30/4,440/5; neither evaluator was changed.

The parent independently rescored every saved prediction, verified row order and
question hashes, and recomputed all three new answers by subtracting known
individual rates from the joint rate, then inverting. The durations9,15,and20/19days
agree with the corresponding question texts under constant independent rates.
The last selected gold option is0.95days, numerically the remaining worker's rate
in jobs/day rather than its completion time. This apparent label discrepancy stays
counted as a metric error and does not rescue the failed coverage gate.

More decisively, the28 parent cases frozen before candidate inspection produce
only1 correct answer among16 solvable positives, versus0 for the baseline. All12
negative inputs refuse; no wrong answer is introduced on these probes. The frozen
gate required at least8 positives and preservation of negatives. The candidate's
65 authored cases and previous arithmetic regressions passed, but its clause
composition still generalizes poorly to these ordinary wording variations.
No candidate-informed retuning or third full inference was performed.

The worker froze40 initial cases and additional sets before corresponding grammar
changes, preserved three failed drafts and an abandoned state assumption, froze
source and adapter before DEV, saved all predictions before loading labels, and
archived the original candidate before rollback. The parent verified73 archive
entries2,412,764bytes;71 report entries2,400,251bytes;85 raw archive entries
32,080,445bytes;35 frozen identities and the candidate binary. Runtime quantity
files match the approved root bytes; the new module and test are absent there.
No quantity score gain or main-command change is published.

Candidate SHA256: `ff0f94d8dc72da238131a41fb3a76c6c5d764e0e13d02802aed466f78f5840b9`.
Full prediction SHA256: `905aa4df540cf65c743415dae8a5e03428ad9e132e4d12fdda6f0d2878b87229`.
See [original handoff](HANDOFF.md), [parent audit](parent/full-audit.json),
[independent cases](parent/cases.json), [candidate outcomes](parent/candidate.json),
and [verified identities](parent/identities.json). Source/rejected patch and all
failed logs remain unchanged. Large raw ledgers/predictions remain externally in
`/home/gsc/.cache/glassmind/continuous-20261010/rateclauses-run/submitted` and
`rejected`; original manifest-relative paths resolve there. Repository copies
omit those large raw files while retaining their manifests and summaries.

No downloads or official test access. MathQA revision
fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6, Apache-2.0; TRAIN9,013,733bytes and
DEV1,350,141bytes. Implementation inspection used TRAIN Problem only and did not
open the source card. Parent audit imported the unchanged frozen metrics. The
first parent raw-manifest check assumed absolute paths; resolving relative entries
against the original worker root completed verification without changing artifacts.

Two work-rate representations have now failed independent wording/coverage gates.
Do not add more templates against these probe sentences. A future attempt needs
new evidence for a more general way to bind clauses; prioritize other tracks now.
