# Contextual source roles rejected after independent review

The new source representation fails every frozen calibration setting. Across
all 16 settings it adds 483–510 correct scored slot-turn fills but also **45–65
newly wrong assertions**. Some settings lose one previously correct joint/frame
outcome. The strictest setting still introduces 45 wrong assertions. No setting
was selected, no candidate DEV evaluation ran, and production dialogue code and
model remain unchanged at 610/1119 joint and 716/1218 frames.

This experiment reused only the earlier FIT boundary model. Fresh role and
reference rankers were trained with one eight-epoch schedule each. The role
classifier adds preceding USER text and learns a competing NONE class from
actual predicted spans. Valid source roles from other services remain available;
there is no old-router owner filter. NONE supervision includes missing or
incomplete annotations, so agreement with those labels is not complete semantic
precision or recall. CAL is empirical component validation, not unseen data.

The parent independently verified:

- All 87 members of the worker's archive and both immutable source packages.
  The separate parent archive contains 129 files/468,238,087 bytes, preserved
  and verified before review or modification.
- All 3,174 utterance-only runtime inputs and frozen baseline prediction indices
  against original TRAIN, including the fixed 642-conversation CAL split.
- Every one of 34,223 reported source spans, its exact original annotation join,
  preceding USER turn, role-or-NONE label, and FIT-only 128-label vocabulary.
- All 4,380,544 source class scores and 46,789 candidate rank scores, every
  candidate's source bytes/UTF-8 endpoints and non-NONE role provenance, and
  all 22,912 native CAL decisions. Maximum f64-versus-native-f32 reconstruction
  errors were 0.000126 and 0.000179 respectively.
- All 34,223 source feature contexts reconstructed from original text, the bounded
  predicted-span mask history and frozen predicted state. No annotation values
  were used to reconstruct the features.
- Every grid's complete persistent overlay and original-gold strict frame/joint
  outcomes. All reported new-correct/new-wrong counts, held changes, losses and
  initial-event counts match. Baseline CAL remains 5803 joint / 1478 frames among
  6830 USER turns / 2505 supported gold frames; unselected turns remain accounted.

This audit checks all reported source contexts and scores; it does not independently
retrain the weights or rerun the full boundary decoder. The worker's separate
first 10 FIT decoder check and seven native tests remain worker evidence.
The calibration overlay does not feed its new values back into the old models;
it establishes rejection of a prerequisite, not integrated runtime accuracy.
The sixteen masked, parent-authored conversations were never run against this
candidate because that prerequisite failed. Their fixtures and baseline remain
unchanged outside the worker's accessible directories. The command log shows
no access to their contents.

The original failed clippy invocation and the audit's initial unselected-frame
assertion failure are preserved. The corrected audit scores only supported
schema frames while retaining every USER turn and raw gold frame; the parent
reconstructs that same, already-established TRAIN convention directly from the
original annotations. No fit, model, threshold, representation or DEV score was
changed to rescue the result. The rejection remains explicit.
The original command log and its preserved patch contain trailing spaces;
their bytes are retained as evidence rather than silently reformatted.

After review, all 129 parent-archive files were reverified, all 32 tracked dialogue
files matched production byte-for-byte, and the owned worktree and temporary
authentication link were removed. Its scope is inactive and PID 539613 is absent.
Original evidence remains in
`~/.cache/glassmind/continuous-20261010/contextledger-run/terminal-originals`;
independent scripts/reports are in `coordinator/contextledger-independent` and
copied under `parent/` for review. Large weights, native executables, predictions
and archives remain external. The rejected example is retained under `native/`,
not installed as a production crate example.

SGD revision `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0;
no downloads or additional teacher calls. Worker usage: 1,749,044 input tokens,
including 1,671,936 cached within that total, plus 31,934 output = 1,780,978.
Known completed-worker goal usage is 201,926,299, cached input counted once;
parent and interrupted usage remain excluded. After two failed SYSTEM-role
representations, move to another track before proposing another grounding model.
