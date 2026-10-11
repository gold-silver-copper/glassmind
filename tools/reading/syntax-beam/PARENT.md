# Bounded beam search rejected

The parent independently rescored all 2,001 retained EWT DEV predictions: LAS
19,423→17,879 and UAS20,540→19,051 out of25,148 words. All trees remain valid,
but both preregistered accuracy gates fail. No runtime changes are integrated.

The parent froze20 sentences/40 core edges before candidate inspection. The frozen
candidate yields25/40 correct edges versus36/40 baseline, and8/20 complete core
structures versus16/20. Every transition is in parent/independent.json. The first
invocation supplied a nonexistent model.json path and failed before predictions;
its logs remain external. The successful run uses the unchanged model.bin SHA
47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6.

Verified57 original rejected archive entries/58,265,881 bytes,29 parent snapshot
entries/82,097 bytes and69 raw artifact identities/109,454,301 bytes. Both runtime
source files have been rolled back and match the root checkout byte for byte.
Original source, binaries, rejected patch, full ledgers and failed preliminary
measurement attempt remain in the syntaxbeam-run directory. Published gold-form
and raw tokenizer results are unchanged; no default/parser/model gain is claimed.

The candidate's cumulative transition score is a poor search objective with these
fixed greedily trained weights. This rejects that specific width-four hypothesis.
A further search experiment requires a different objective or training hypothesis,
not a width sweep on this DEV result. No new data/downloads or held-out access.

Artifacts: /home/gsc/.cache/glassmind/continuous-20261010/syntaxbeam-run;
independent review: coordinator/syntaxbeam-review and syntaxbeam-independent.
