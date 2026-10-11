# Independent rejection: equivalent-reference operand supervision

All eight fixed gates fail. The unchanged original model gets143/1182 clean TRAIN
CAL numeric matches; matched singleton continuation A gets146, while formal-target
continuation B gets127. Gold-plan oracle results are221→225(A)/203(B). B loses27
previously correct free answers and changes29 refusals into wrong answers. No
model, quantity runtime, confidence policy or DEV score changes are accepted.

This was a supervision experiment, not another feature rename. Both arms begin
from the original weights, freeze the operation head, and continue six matched
epochs12–17 with the old SGD schedule and deterministic FIT rotation. The only
new targets arise from expanded symbolic tree equality with Add/Multiply child
commutation. Input slots remain distinct even if values coincide; Subtract/Divide
stay ordered. Association, cancellation and numerical equality never define labels.
The worker's complete design, denominators and limitations remain in REPORT.md.

The parent independently verified121 frozen files /150935819 bytes, after first
archiving146 original files /151366628 bytes before source/result inspection.
Original proposal.rs has only experiment/test includes appended; its feature,
decoder, masks and execution implementation remain byte-identical. Rational is
byte-identical. Original/A/B operation f32 bytes share SHA
`9f8445a807a39ce7478757cf92ac6816110ff2392d56666b38d4038b1fd8cb86`.
Both model files remain below16MiB. No fitting was repeated during parent review.

Before worker inspection the parent froze13 formal cases and3 invalid references,
then ran120 native arithmetic executions across65 assignment groups plus3 invalid
reference checks. These include distinct/equal values, negatives, zeros, decimals
and denominator failure. An independently implemented Fraction/tuple-tree oracle
constructed every legal target pair set for all19891 clean TRAIN steps. Later
review matched all59673 reported sets across original/A/B, every equivalence flag,
all5772 clean memberships and original free/oracle records. All29837 source rows
remain in the original split/exclusion accounting.

The parent compiled a wrapper around the archived private pair/gradient functions.
All13 formal sets and3 invalid references passed. Fifteen independent probability
cases check60 gradient coordinates against finite differences, including unequal
targets, tied scores and extreme probabilities; maximum error2.53e-9. Source
review confirms conditional right gradients use the same pre-update weights and
are averaged across valid left contexts, preserving one total step weight. Both
arms have identical six-epoch rates and total right-step weight15753 per epoch;
B has22797 contexts because its target sets expand7044 FIT steps.

A fresh question-only native B replay exactly reproduces all1182 CAL free records.
The original native executor independently reproduces2282 proposed CAL free/oracle
programs. The unchanged typed-label comparator independently reproduces all FIT/CAL
numeric totals and transitions, with the complete4590/1182 denominators. All eight
worker gates independently fail. Numeric equality still does not verify English
semantics; oracle results explicitly supply the gold operation plan and length.

The original equivalent-left baseline is2474/4138, not the2197 exact-order count.
B reaches2562 (+88, below125 required), while right declines2499→2487. No extra
right labels are admitted with gold left fixed in this corpus. Marginal equivalent
labels therefore do not demonstrate improved complete grounding. The matched
control's modest numeric gains do not authorize deployment or threshold selection.
Do not revive this fixed configuration by weakening gates or choosing a checkpoint.

No new external data: MathQA TRAIN revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6`, Apache-2.0,9013733 bytes,
SHA`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
Completed worker usage1454078 tokens, including1350912 cached input counted once;
root/interrupted usage is excluded. Original worker reports, training logs, native
tests, manifests and failed gates remain intact. There was one completed matched
experiment, no new failure-driven configuration or new DEV evaluation.

Compact parent reports/adapters are in parent/. Full native outputs, formal target
sets and parent probe source/binary are in
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/equivbinding-independent`.
Original submission is in `equivbinding-run/terminal-originals`; final models and
exact helper source are in `equivbinding-run/frozen-candidate`. The experiment's
reproduction scripts expect a disposable checkout under the existing WORK root
beside `groundingaudit-source`; parent adapters use their original cache layout.
