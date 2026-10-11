# Coordinator validation: explicit bounded factual lists

Integrated the exact three worker source files at
`ac59ac82f8fa4cc154057c74dce29ed4edbee178`. Explicit list requests return up to ten
sorted unique IDs with local counts, truncation and provenance. Ordinary
single-answer behavior is preserved. No benchmark accuracy gain is claimed.

The coordinator verified 29 artifact identities and the frozen contracts before
integration. It repeated all 36 primary and 16 supplemental contracts, added seven
production-intent/session and exact-limit boundary checks, and passed 17 Rust tests,
workspace fmt/clippy/release and the four-turn fixture smoke. First200/full2821
standalone, ordinary facts-enabled talk and facts-disabled talk streams all
matched the retained baseline byte for byte. Independent scoring confirmed
256 right /2545 refused /20 wrong, and joint entity/relation/answer counts
252 /2545 /24. The three-answer real example was checked against cached published
Wikidata statement IDs.

Memory without the conversation intent model passes41/41 with and without a
facts bundle. Calculations27/27 and stories47/47 also pass in both modes with the
production intent model. Seven additional list/state checks use that production
model. A further production-model memory smoke failed on “Call me tomorrow”, which
is classified as `change_user_name` rather than refused. The coordinator preserved
that failure and compared all six original conversation streams and saved states
against the frozen baseline. They are byte-identical in both facts modes: both
versions match40/41 original expectations with the model. The personal name stays
unset. This inherited mismatch is recorded, not waived or reported as a passed
41/41 model-enabled test; the original smoke and its expectations are unchanged.

[Integrated validation](integrated/validation.json) links the scope and counts;
[inherited-memory comparison](integrated/production-memory-compatibility.json)
retains all four mode results. Raw parent artifacts are under
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/factsets-check/`.
Original25files/108,120bytes were archived, verified before integration and checked
again before removing the worker checkout. Its scope is inactive and login link
removed. No source downloads or model/index/scorer changes occurred.

The result lists finite snapshot edges selected by a learned relation proposal;
it is neither globally exhaustive nor a proof of question semantics. Historical
validation-seeded index scope and inherited factual errors remain documented.
