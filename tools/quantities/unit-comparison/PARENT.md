# Independent review: exact dimensional comparisons

The frozen parent suite passes 24 substantive comparisons and 17 refusals. A
separate Fraction/SI oracle verifies every exact conversion witness, dimension,
ordering, predicate/truth and left/right/equal selection. All 24 direct API
results equal their parsed counterparts. Cases cover six dimensions, affine
temperatures, negative and fractional magnitudes, raw-number order reversals,
false predicates, equal selections, malformed inputs and checked overflow.

Source review confirms reuse of existing Unit scale/offset definitions, checked
rational arithmetic, private validated operands, direct-query dimension checks,
full input consumption and bounded comparison transport. No runtime floating
point, language model, context binding or duplicate unit definitions. Legacy
scalar Answer/solve/parser are unchanged. Mathematical signed magnitudes are
allowed; no new physical plausibility policy is claimed. Checked intermediate
overflow may refuse otherwise representable answers.

Main conversation tests pass 79 commands and 79 state-preservation checks in
each of plain and intent-model modes: 158 comparisons total, with a process
restart after each. Successful natural/explicit requests equal native results;
malformed quantitative stems and explicit prefixes remain in the comparison
route. Personal memory, story and saved draft state are preserved; only turn,
last-message and correction-focus bookkeeping changes. Recall, calculation and
draft checks pass afterward. Both real facts and dialogue bundles are loaded
during these checks. Human false/equal/right-choice rendering includes exact
witnesses. No comparison field is added to saved sessions.

Fresh native scalar AND main conversation outputs are byte-identical to the
baseline for sample200, full4475 DEV and synthetic3116. The latter's main score
remains 3056 right (including 762 expected refusals), 60 unsupported, zero wrong.
Legacy memory41, calculations27 and stories47 pass; production memory authority93
and original41 pass. All2821 ordinary factual results remain byte-identical in
standalone, facts-enabled talk and default talk, after the fixed200 sample.
These are authored capability/compatibility checks, not a MathQA accuracy gain.

Worker64 authored cases (32 comparisons/32 expected refusals), 19 package tests,
five transport checks, fmt, all-target clippy and release passed. Initial clippy
failure on enum size was corrected by boxing the successful response; original
source/log retained. No new external data or training. Existing question-only
MathQA DEV revision fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6 is Apache-2.0;
authored cases and source are MIT OR Apache-2.0. No labels were mounted or used.

Original archive:206 files/50,970,674 bytes, verified before inspection.
Frozen submission:126 files/49,345,825 bytes, independently hash-verified.
Native SHA `a135ff47690658ed9f85e88dc3cea145ac6e2c1b33d624ee8fff9aec71f800e5`.
Final combined main SHA
`53c6cacac60e61cf90aba78d183c9134e574b3db56755141913392700c69dfff` equals the
independently evaluated integration binary byte-for-byte. Quantity source matches
the frozen worker source exactly. Workspace fmt, all-target clippy, release and
strict4/4 fixture smoke pass.

Compact oracle, contracts, adapters and reports are under `parent/`. Complete raw
artifacts remain under `/home/gsc/.cache/glassmind/continuous-20261010` in
`unitcompare-run/terminal-originals`, `unitcompare-run/frozen-candidate`,
`coordinator/unitcompare-independent`, `coordinator/unitcompare-integrated` and
`coordinator/combined-comparator-unitcompare`. The worker used 909,785 known
tokens, including cached input once; parent usage is not included.
