# Frozen plan — 2026-10-11, before implementation

Hypothesis: typed conjunctions of two or three positive directed relationships
can enumerate shared subjects from the entire finite Store with exact witnesses.
Baseline: verify accepts one explicit subject-edge, answer_set one learned relation;
there is no match command. Its absence is a capability gap, not baseline accuracy.
No benchmark accuracy gain is claimed; preserve ordinary answer256/refused2545/
wrong20 and joint252/refused2545/wrong24 on full2821 dev.

Grammar (ASCII case-insensitive keywords, exact single spaces inside grammar):
QUESTION := spaces 'Who ' CLAUSE ' and ' CLAUSE [' and ' CLAUSE] '?' spaces
CLAUSE := 'was born in ' ENDPOINT | 'died in ' ENDPOINT |
          'is a citizen of ' ENDPOINT | 'was a citizen of ' ENDPOINT |
          'works for ' ENDPOINT | 'worked for ' ENDPOINT
ENDPOINT := full verification-compatible alias or explicit canonical QID.
All separators are reserved, including inside aliases: aliases containing AND/OR/
negation/conditionals are unsupported, not truncated or resolved using edges.
Predicate words elsewhere inside aliases are retained in full. Exactly one complete
parse alternative is possible (hard cap 1); raw condition count 2..3 before dedup.
No learned fallback, pronoun context, temporal qualifiers or current validity.
Explicitly reject temporal/comparison/instruction words: currently, current, now,
today, yesterday, tomorrow, before, after, since, until, during, when, while,
more, less, than, latest, still, only, always, ever, ignore, instead, please.
Also reject verification's reserved endpoint words and malformed QID-like inputs.
Question <=4096 bytes before allocation; endpoints <=256 bytes; control chars
rejected; Unicode alphanumeric aliases preserved. Leading/trailing ASCII spaces
around whole question allowed; endpoints cannot have exterior whitespace.

API: typed predicate enum and constraints, immutable Store matching method; resolve
ALL endpoints before reading claims, with missing/ambiguous endpoint => typed
unknown. IDs override label collisions and may occur only as edge objects. Entire
claim vectors are searched; intersect before truncation. Dedup constraints and
subjects, sort subject IDs lexically, report full snapshot total/returned/truncated,
display at most10. Each displayed subject has all unique exact directed witnesses
and source URLs. Empty is scoped no matches, never global false. Caller attaches
verified bundle provenance; CLI adds SHA256 of actual index bytes and scope.
Do not change Store.validate or old Verification/Answer/AnswerSet semantics.

Frozen authored graph/contracts cover all four predicates, two/three constraints,
order/duplicate invariance, >10 output and >10 objects, disjoint/empty graphs,
missing/ambiguous endpoints despite unique graph matches, QID/label collisions,
case collisions/acronyms, Unicode, predicate words in aliases, unsupported syntax,
controls, malformed IDs and size/count bounds. Exact expected IDs and witness
conditions are frozen before implementation in contracts.json and graph.json.
Acceptance: every CLI contract passes, typed API tests pass, existing facts tests
and available public verification contracts pass; sample200 then full2821 legacy
predictions and native score bytes unchanged using original adapters/metrics.
Real-index proof must strictly reduce BOTH single-filter sets with exact edges.
Freeze code/binaries/patch/contracts/reports under run/frozen-candidate. Preserve
original failures; no official heldout, parent fixtures, old logs, data changes,
scorer changes, services, model calls, agents, downloads, commits or main integration.
Pronoun tokens he, she, it, they, him, her, them, we, us, you, me, my, our,
their, his, its are unsupported endpoints, even if they happen to be aliases.
Malformed QID-like means q/Q followed by a digit or hyphen, or bare q/Q; these
must satisfy verification's canonical QID rules. Other names beginning Q allowed.
