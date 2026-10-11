# Frozen rename language batch — 2026-10-11

Source baseline 8e12be3f884720a9ba6065fbec49e274d37589c4; approved binary SHA256
226a97606e46889ac7d1005b513b13c9cb2c9343fd11c027752d3ec8dbd6dbf0.
Hypothesis: fully consumed compositional rename instructions can preserve explicit
identifier bindings, target kinds, scope and behavior constraints without being
misread as test repair. No backend changes, learned models or arbitrary prose removal.

Supported extension: courtesy; imperative rename with optional supported kind and
optional ownership, exact source identifier, to/arrow, exact destination;
everywhere plus enumerated definition/callers/tests coverage; keep tests and
preserve behavior clauses. Conversational form binds a named function and a
bounded present-tense description to “I'd like it called DEST instead”, followed
by an explicit rename-it-everywhere request. Context/description noun phrases use
bounded grammatical vocabularies; unfamiliar descriptions refuse. Explicit scope
examples after everywhere do not narrow it. Unicode, case, backtick and raw
identifier contents retained. No implicit target-kind conversion.

Recognized incomplete, negated, conditional, diagnostic-only, conflicting or
unsupported rename instructions must not fall through to repair or partial
rename guesses. Quoted mentions and identifiers containing rename are not rename
commands. Existing genuine grammar readings and ambiguity must be retained where
compatible; no new route may silently resolve an old ambiguity.

Acceptance: all three supplied requests typed Rename(parse_line,parse_entry),
no Repair; >=8 distinct authored positive extensions; negative controls produce
no wrong/mutating action. Shared ask/do bridge and guesses tested. Existing English
rename/identifier tests and CLI intent/apply regressions pass. Independent simple
native edit preserves behavior/comments/strings/unrelated code. Supplied workspace
may still refuse in unchanged backend. Fixed sample then full authored dev.
Parent must run full765 and retain ~177 successes/zero fail before integration.

Frozen authored cases in cases.tsv, written before implementation. Existing tests
are additional unchanged controls. No official/held-out evaluation access.
