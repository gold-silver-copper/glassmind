# Preregistered boundary batch

Base source 2621cf42b33d32b228f3decbe68988917eec6991. Frozen offers model
SHA256 deddfaac2f596c5090aeb65bf77d24470d1d44f61d2aad69469792cd26232781.
Baseline full: 609 right / 460 wrong / 50 refused of 1119 USER turns;
712/1218 exact frames; first ten conversations 35/91 joint, 48/108 frames.
Four inherited wrong reference events persist for eleven scored slot-turns.

TRAIN inspection: USER-span punctuation opportunities overwhelmingly exclude
sentence punctuation (e.g. final alphabetic length>=3 plus period: 601 fit,
141 calibration exclusions). Dotted single-letter shape plus period includes
punctuation in 12 USER fit spans, but has no USER calibration evidence.
SYSTEM spans add 61 fit and 11 calibration occurrences for that shape.
These are raw occurrences, not independent conversation support. No dev labels
were inspected to choose these representations. Prior handoffs' disclosed dev
errors are known; they are not runtime rules or a calibration source.

H1: optional train-only punctuation boundary restoration at reference-copy time
reduces copied-slot errors without changing classifiers, routing, offers, source
states, or suppressing transfers. Features: final whitespace word with alphabetic
and numeric runs anonymized to A/D plus length capped at three, literal other
characters; next maximal punctuation run (1..3 chars). The learned output is a
prefix length of that run. Public USER and SYSTEM TRAIN span annotations supply
positive AND negative endpoints. Noncategorical input slots only. Candidate keys
come from fit only. SHA256(dialogue_id) integer mod5==0 is calibration, all others
fit (same split as offers). Freeze before dev: at least five distinct fit and
three distinct calibration conversations, zero mismatched endpoint lengths,
nonempty suffix, bounded shape<=64 chars. No entity values or service/slot IDs
in boundary weights. Sparse value diversity will be reported, not hidden.

Runtime only modifies a reference proposal after existing validation. It looks
at the most recent exact whole-value occurrence in prior utterances, restores
only punctuation literally adjacent in that source, and emits exact byte
provenance. Ambiguity or absent evidence leaves the existing proposal unchanged.
No freshness correction is claimed; stale source predictions remain possible.
Optional submodel has explicit version 1; absent keeps frozen behavior.

Acceptance: fixed first ten dev conversations then full179/1119/1218 with
unchanged scorer; retain all609 baseline-correct joint turns, ideally all712
frames, and reduce actual incorrect copied claims OR add correct coverage with
zero new incorrect slot claims. Exhaustive changed-slot audit includes held
unscored states; track all four inherited events. Legacy202/257 exact and
89.17509727626459% fuzzy unchanged byte-for-byte. Annotation rejection and
conversation reverse-order/reset isolation required. If gates fail, reject
candidate honestly without dev-tuning another rule.

Rejected before implementation: indiscriminate punctuation append (TRAIN
negative evidence); USER-only learned punctuation (no calibration support).
Deferred: contiguous BIO decoder and semantic correction freshness are separate
hypotheses; changing frozen decoding would broaden risk and violate this batch.

## TRAIN-only schema amendment, before any dev prediction/evaluation

Selected-service schema H1 fit rejects the abbreviation rule:12 fit conversations,
0 calibration. It retains only a closing-parenthesis rule (9fit/4cal), insufficient
to test the source error hypothesis. Inspection found the initially counted SYSTEM
calibration spans belong to OTHER services in the SAME selected TRAIN conversations.
The selection retains their public annotations, but selected schema excluded them.
Retain that rejected training report. Fit H1b with the full public TRAIN schema,
using all noncategorical input slots in the unchanged3174 conversations. Features,
5fit/3cal thresholds, SHA split, runtime scope and acceptance remain unchanged.
This is a training-only scope choice, before any candidate dev outputs or labels.
No new conversations/data are added; system-span evidence is explicitly disclosed.
