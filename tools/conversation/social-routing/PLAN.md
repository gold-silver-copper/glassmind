# Prospective social routing, 2026-10-11
Base1331ac715eadcf7b9169423e947cedf861eb42a3. Before runtime editing.
Hypothesis: fully consumed compositional social forms may bypass unsupported
story-assertion invalidation, but a learned social label alone cannot. Probe
story operations immutably, then choose exactly one story transition. Typed
personal authority, calculations/facts, supported story operations retain priority.
No model/threshold/training/reasoner changes. Same production intent model.
TRAIN reviewed: first20 of each greeting/goodbye/thank_you/are_you_a_bot/
what_is_your_name/what_can_i_ask_you, each100 rows. greeting includes salutations,
wellbeing questions and nice-to-see-you; goodbye includes see/talk-to-you-later;
thank_you includes modified thanks and appreciation. Only bounded full forms,
not arbitrary prefix stripping or suppression of all unsupported assertions.

Frozen sample: first6 contracts (goodbye and hey-there × none/active/invalid).
Full: all contracts.json (41 sessions) both actual model and no-model, two
repeated social turns across persisted process restarts, last-utterance exact
recall, one increment per input, story state unchanged, memory source/focus,
bare correction, reset. Contrasts cover movements/unsupported assertions,
appended commands/assertions, negation, quoted speech, question/assertion,
priority for names/preferences/calculations/incomplete writes/external proposals.
Expected no-model unknown social remains ordinary refusal, never story write.
Learned supported social must be intent, literal Hello remains social.

Acceptance: all frozen checks; original supplemental plain6/6 model7/7;
original memory93+41 both modes; calc27/story47, facts36+16; sample then full
bAbI TRAIN-derived dev3987 with production intent model, unchanged3966correct,
18refused,3wrong; zero new wrong, audit changed responses and saved states.
Reject if old typed assertions lost or stale confident story answers introduced.
All failures preserved; no expectation edits after freeze. Parent independently
replays integrated binary and ordinary facts2821. No raw intent3100/native rerun.
No downloads. Authored contracts MIT OR Apache-2.0. CLINC revision828f8093932c8fe6ca7936c3d2e52903b1c523de CC-BY-3.0. bAbI v1.2 CC-BY-3.0 TRAIN only.
