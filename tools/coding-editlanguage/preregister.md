# AddField clause composition preregistration

Base cfa343e9948a61699061230f720d15f77b81ca52, 2026-10-10.
Hypothesis: source-preserving lexical constituents + typed declaration, constructor
scope and default clauses, unified only after full consumption, improve English
coverage over whole-sentence matching without accepting competing commands.
Implement only english-rust/add_fields.rs and new diagnostic adapters/reports.
Reuse AddField API. Existing English Token spans are useful precedent, but its
normalization strips call parentheses/quotes, unsuitable for scalar expressions.
No shared helper until there is a second consumer.

Acceptance: fixed terse sample first, then all FIVE provided original dev
requests; >2 verified edits, preserve both baseline successes, zero wrong edits.
Test independent names/types/literals, clause order, polite/contraction forms,
negative compounds, conflicting values/owners, unknown clauses, exact literal
spelling, unrelated Option fallback, shared queries/exports. Restore source for
each request; preserve original public tests and behavior, verify new values.
Frozen binary baseline SHA256 5f76302ccde394ebf74e2b7d7bc0030ab12f0f34a2915b1f1618d68ba17cd11a.
Given baseline 2/5; first replay with relative output paths produced package graph
refusals (0/2); rerun absolute paths before evaluating parser changes.

Bounded grammar: public declaration variants, owner prepositions, universal and
new-plus-other-literals scopes, explicit scalar defaults, standalone polite
asides. Unknown motivational/context prose remains unsupported rather than being
trimmed. No entities, filenames, task identities or expected values in grammar.
No runtime/world/CLI/scorer edits, training, downloads, agents or LLM calls.
