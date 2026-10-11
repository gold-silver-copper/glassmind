# Frozen plan (before implementation, 2026-10-11)
Base a716079f6713e190fe543d7ecf6599ce7c14369d. One bounded batch.
Hypothesis: compose full shared-subject intersection with existing single-claim
strict temporal witnesses to add date-qualified positive conjunction retrieval.
No model calls, downloads, labels for query selection, or semantic policy changes.

Grammar: exact existing Who grammar, 2..3 raw positive constraints, duplicate
normalization unchanged; only P108/P27 supported. Parse the complete question
before alias resolution; reject unsupported properties even if another alias is
missing. Resolve every alias before touching edges or intervals. Typed conditions
use the same endpoint rules. Exact Gregorian CalendarDate and all existing
precision/rank/qualifier rules remain unchanged. Immutable hash-bound index.
Each normalized condition requires one original eligible statement, with
latest(start) < requested day < earliest(end); never combine separate claims.
Deterministic lexical subjects and existing lexical lowercase claim-ID choice.
Count complete dated intersection before display limit 10. Zero describes only
recorded interval matches, never global absence. Every displayed witness retains
edge, source, claim ID, raw bounds, ranges, raw hash and index hash. Result date
applies to every witness. Subject edge resolution is explicit_id (selected ID).

Acceptance: independent authored fixtures cover late matches after ten static
nonmatches, >10 dated matches, distinct condition intervals, 2/3 constraints,
same-property distinct objects, duplicates, ambiguous aliases, unsupported grammar,
invalid dates, all boundary/month/year/leap/qualifier cases, split claims and
claim-order determinism. Assert full expected witnesses, not status alone.
Baseline/current old APIs first. Fixed first 8 authored and real queries before
full finite runs. Independently reconstruct all temporal records from original
hash-verified raw responses; reference dates use Python Gregorian calendar.
Real query denominator: all unordered distinct eligible (property,object) pairs
co-occurring on any subject (including nonoverlapping intervals), plus duplicate
pairs for every eligible edge. For each pair use sorted union of each claim's
latest-start and earliest-end days and those days +/- one day, plus 2000-01-01.
No answer-dependent date/query selection. Full expected intersections over raw
statements, exact witness comparison. Report distinct-pair coverage honestly.
Legacy: first200 then full2821 questions byte replay and unchanged native score;
old verify-at/match/verify/ask CLI outputs compared baseline/current. Main route
is coordinator-owned and not implemented here.

Rejected approach before experiment: filtering already-truncated match_question
would lose later valid matches and produce wrong totals; not implemented.
Any failed attempts retained; no changed denominators or weakened assertions.
All cargo/native jobs use 8GiB/0swap systemd scope, setsid --wait timeout600,
persistent TMPDIR, existing CARGO_TARGET_DIR, -j1 and test threads4. No commits.
