# Prospective social routing

Main talk can preserve its story history across supported social turns. The
existing intent model remains a proposal; the router requires an entire social
form, including any supported modifier, before bypassing assertion handling.
The grammar covers salutations, short farewells, modified thanks, wellbeing
questions and nice-to-see/speak/talk forms. Without a model, existing literal
social replies remain available and other recognized social forms retain the
ordinary no-model refusal without corrupting story state.

Typed personal memory, calculations and facts retain their existing authority.
The story adapter now creates a non-mutating plan before the router chooses
whether to apply it. Supported story operations retain priority. A complete
social prefix followed by an explicit clause boundary is refused and invalidates
story state, rather than becoming a coordinated story actor. Unconsumed text
after a complete social prefix also invalidates when no supported story
operation consumes the turn, including a question-shaped prefix. An unconsumed
learned social proposal cannot authorize a social reply through the final
fallback. Unsupported assertions retain invalidation; social turns never heal
an already-invalid context. Explicit reset is still required.

This is deliberately bounded syntax, not general conversation understanding.
It does not execute external services, train a classifier, or alter story
reasoning. See [the frozen plan](../tools/conversation/social-routing/PLAN.md)
and [handoff](../tools/conversation/social-routing/HANDOFF.md) for exact evidence,
known limitations, hashes and reproduction commands. No independent held-out
performance is claimed.
