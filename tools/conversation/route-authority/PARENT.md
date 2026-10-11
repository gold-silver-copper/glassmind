# Parent decision: accepted after independent replay

The integrated route preserves all 1,303 previously correct learned proposals on
all 3,100 CLINC validation inputs and adds two. All 19 changed rows were audited:
two refusal-to-correct proposals and 17 refusal-to-refusal changes; the same 40
wrong accepted proposals remain inherited. Typed replies are ungraded by this
routing diagnostic, so it is not raw classifier accuracy or full conversation
correctness. The original social-routing candidate remains rejected separately.

Parent replay uses final runtime47b6b285b5cd23f0580600dcda3011689ab20da1, binary
bd7df8f882bf3fb2d1a5ac4c86ce77083e6305949953630030285256562747dd.
Both model modes pass the original490 checks, memory93+41, and goodbye6/7.
New boundaries pass116common checks plus6baseline-reply comparisons in each mode;
the extra comparisons are not presented as six coverage gains. The fixed samples
precede full runs. Calc27, mixed story47, fact sets36+16 and stale-answer2 pass.
All17 conversation tests and final workspace fmt/clippy/release/strict4smoke pass.

All47,625 actual-model story replies on3,987 questions are byte-identical to the
validated baseline:3966correct,18refused,3wrong. All2,821 ordinary fact questions,
with and without the facts bundle, preserve output bytes. Coding remains the
separately evaluated5f2859a:174passes,591refusals,0failures on765, every outcome
unchanged. Only the two talk modules changed afterward; no coding path changed.
The same model file, classifier, thresholds and weights are retained.

The original worker submission was archived before integration:442files, with
438artifact entries,25identities,30input identities and2frozen files verified.
Its exact runtime was integrated before the parent added buffering to the JSON
loader. See ../buffered-loading for the measured startup improvement. This
repository keeps a byte-identical selection of small original reports/adapters;
large streams and saved session files remain in the verified external archive.
The original artifacts.json indexes that full archive, not only these copies.

Execution limitations remain explicit. The worker's model wrapper exited2 after
all scoring commands because it was edited during execution; individual reports
and final bash syntax passed. One earlier boundary launch raced its prerequisite
and was resumed without a completed inference being repeated. Parent unbuffered
story inference completed, but a baseline-directory typo interrupted report
writing; the retained replies were scored offline with unchanged logic, without
another inference run. Parent then intentionally interrupted the slow unbuffered
model session run after15saved sessions when buffering proved faster. That partial
run is not counted as passing. Every required suite completed on the final buffered
binary; the later full replay is justified by the new loader change.

Unknown assertions still refuse conservatively. A newly introduced greeting-like
actor in coordinated movement can remain ambiguous; established actors retain
supported movement. This is finite routing, not general semantic understanding.

Original archive: `/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/submitted`.
Parent raw final artifacts: sibling `coordinator/buffered-route-review`; earlier
attempts: `coordinator/routeauthority-review`. Reports and exact identities are
in parent/. No held-out material, changed scorer or new model training was used.
