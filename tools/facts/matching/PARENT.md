# Parent review: shared-subject fact matching

Accepted for integration: `Match facts:` now finds subjects satisfying **all two
or three explicit positive relationship conditions**. It resolves complete
endpoints before looking at edges, intersects full claim vectors before limiting
display to ten IDs, and supplies an exact directed witness for each condition.
It preserves personal memory and story state. This is a finite-store capability;
ordinary facts benchmark accuracy is unchanged.

The original worker submission was archived before inspection: 226 files,
85,040,204 bytes, including the Cargo.lock dependency-edge change, under
`~/.cache/glassmind/continuous-20261010/factmatch-run/terminal-originals`.
All copied hashes were verified. Original native binary SHA256:
`90861d7dfe781d45c7eccfe10253edefbe197e47fb4e2bbc57a14472205e7203`.
The worker's original separator failure and corrective evidence remain in
[evidence](evidence/reports/separator-original-failure.json); no original labels
were changed. Its [handoff](HANDOFF.md) distinguishes historical verification
fixture disagreements shared by the baseline from current contract results.

## Independent evidence

The [30 parent cases](parent/cases.json) and 49-entity graph were frozen before
candidate inspection, SHA256
`102d1cd1c75601a23e8e416fa3436989f313c596d5dbd10f9d14805bc9c2a92b`.
All 29 transportable standalone CLI cases pass with exact IDs, counts,
constraints and witnesses. All 30 pass against the native library, including NUL,
which cannot cross argv. No syntax adaptation or expected-result change was needed.
The cases cover all four predicates, two/three constraints, duplicates and order,
ambiguous aliases despite a unique graph answer, case collisions, edge-only IDs,
full claim vectors, unknown versus scoped empty, and unsupported full-input forms.
One graph intersects sets of 30 and 20 subjects to 11 before displaying ten.
The original PLAN prose said 19 for the second set; [audit.json](parent/audit.json)
records the correction without changing its frozen graph or expected arrays.

Eight real-index conjunctions all strictly reduce both input sets and match the
expected IDs. Sixteen directed witnesses were independently confirmed against
hash-verified original Wikidata response files. [Raw evidence](parent/real-audit/raw-edge-audit.json)
retains claim IDs, ranks and qualifier-property metadata; these unqualified index
edges do not establish present employment, citizenship or complete-world truth.
The 500-entity index was seeded by validation subject IDs, so this is scoped
capability evidence, not independent retrieval benchmark performance.

The parent implemented the explicit main conversation route. Its
[29 native conversation cases](parent/main-contracts/report.json) preserve the
standalone results and provenance. Another 26 state/recall checks pass with and
without the production intent model. Disabled bundles refuse explicitly;
malformed prefixed instructions cannot mutate names, preferences or stories.
Human output includes truncation, exact source links and snapshot limitations.
The multiline case is checked through the library and standalone CLI because
conversation input uses one utterance per line.

Unchanged legacy adapters confirm byte-identical standalone, facts-enabled talk
and default talk outputs on first200 and full2821 development queries. Ordinary
answer metrics remain **256 right /2545 refused /20 wrong**, joint metrics
**252 /2545 /24**. Existing wrong answers are retained and disclosed. Memory41/41
without intent, calculations27/27 and mixed story47/47 pass. The separate 26
state checks exercise the production intent model; they do not imply the entire
old memory suite is perfect with that model.

Affected native tests: 24 main CLI and 16 facts tests pass, including the parent
30-case library test. Workspace fmt, all-target clippy, release builds and strict
4/4 fixture smoke pass. Integrated main SHA256:
`562a774bc98583d487b2bbcabcb53b3b300bae767f8427c7f0491cc2652a07a4`.
The archived matching library source was integrated without changes; coordinator
additions are the main route, presentation and independent tests. Coding source
is unchanged from the independently evaluated ceiling-division batch.

## Scope and reproduction

[Usage and limits](../../../docs/fact-matching.md). No runtime LLM, new dataset,
new model, network query, index expansion, held-out inspection or scorer change.
The sha2 package was already locked; only its facts dependency edge was added.
Questions are limited to4096bytes, endpoints256bytes, raw conditions2..3 and one
complete parse. AND-containing aliases are explicitly unsupported. Exact QIDs may
refer to edge-only objects; an empty result describes only this store. Source
labels never determine links. Bundle hashes identify loaded bytes, not upstream
truth. Historical qualifications are not interpreted.

External artifacts live under `coordinator/factmatch-independent` (frozen cases,
real bundle and baselines), `coordinator/factmatch-audit` (raw-edge verification),
and `coordinator/factmatch-integrated` (native main, tests and regression results).
Use parent/run.py for standalone cases and parent/main-contracts.py for the
conversation route. `tools/facts/sets/regression.py --baseline .../main-baseline`
checks all six legacy output files. `tools/facts/chat/legacy.py` runs existing
memory, calculation and story fixtures. All builds use the established capped
systemd scope, persistent TMPDIR, timeout/process-group cleanup and -j1.
