# Frozen temporal batch plan — 2026-10-11

Hypothesis: exact positive P108/P27 questions plus typed calendar dates can obtain
conservative evidence from a single bounded recorded Wikidata statement. Static
verification remains unchanged and provides no temporal guarantee. Acceptance:
all authored contracts, three distinct cached subjects with interior support and
boundary/outside unknown, all facts tests (including existing parent matching),
first200 then full2821 unchanged predictions and native Score bytes. Missing old
verify-at is capability absence, not measured accuracy. No independent benchmark.

Policy frozen before implementation: support iff latest possible start DAY <
requested DAY < earliest possible end DAY, from the SAME statement. Precision
9/10/11 denotes full year/month/day ranges even when subordinate digits are filled.
Never infer boundary inclusion, instants/timezones, current validity or negatives.
Reject dates outside exact ASCII YYYY-MM-DD, years0001..9999 and Gregorian dates.
Bounds require +YYYY-MM-DDT00:00:00Z, Gregorian Q1985727, precision9/10/11,
zero timezone/before/after. Zero subordinate month/day permitted only below
precision; filled fields must be structurally valid (day with zero month rejected).
Reject other calendars/precisions, BC/yearzero, malformed fields, open/nonvalue/
duplicate bounds, inverted possible intervals, deprecated ranks or extra qualifiers.
Overlapping uncertainty without strict interior is unknown. One eligible statement
suffices; duplicates identical collapse, conflicting same claim ID reject asset.
Stable selection sorts statement IDs. Alias resolution uses existing verification
before temporal lookup; no time/edge disambiguation.

Frozen resource caps: static index 32MiB; temporal JSON 16MiB; 100000 records;
individual serialized statement 64KiB; query4096bytes, endpoint256bytes. Offline
extractor: <=128 source files, each <=8MiB, <=64MiB total; manifest <=1MiB.
Schema version1, exact index SHA256, canonical identifiers/source URL/hash, raw
statement including ALL qualifiers retained. Rust derives ranges from raw values,
checks triple against mainsnak and static edge; no trusted precomputed ranges.
Metadata and index owned together by library object to prevent accidental rebinding.

Authored contracts are frozen in contracts.json BEFORE runtime edits; fixtures
are authored independently of extractor. Library and native CLI results must
agree including full witnesses. Loader rejection tests include index hash/schema/
wrongedge/conflictingduplicates/byte and record caps. Original failures retained.

Data: provided 20 historical raw Wikidata responses, CC0, partial500entity cache
seeded using historical validation subjects. No new retrieval or training. Legacy
SimpleQuestions revision df69267f1db4f85a71325e7e8c06e192bcaf0b0e, CC BY3;
Wikidata CC0. This sampling cannot establish independent retrieval or truth accuracy.

Primary documentation supplied by parent (no downloads):
https://www.wikidata.org/wiki/Help:Dates
https://doc.wikimedia.org/Wikibase/master/php/docs_topics_json.html
https://www.wikidata.org/wiki/Property:P580
https://www.wikidata.org/wiki/Property:P582
Help:Dates explicitly distinguishes numeric lower fields from precision and local
calendar dates from UTC instants; nonzero ambiguities rejected here.
