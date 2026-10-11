# Typed personal-memory authority

A learned intent is a proposal. Only the explicit personal-memory parser can
supply a value for a name or preference write. An accepted `change_user_name`
proposal without a typed name now refuses the operation and explains how to
state the name explicitly. It retains the actual model proposal for inspection;
its `accepted` field describes classifier acceptance, not write authorization.
The classifier, model weights and thresholds are unchanged.

`Conversation::respond_with_authority` returns a reply plus a flag identifying
an explicit personal-memory response. The existing `respond` API and reply JSON
remain compatible. The main talk router honors an authoritative typed refusal
before trying other domains. This matters for incomplete statements: sending
one to the story parser previously invalidated otherwise empty story context.

A correction must have a currently focused, existing personal value. Unrelated
fallbacks, social turns, last-message recalls and unsuccessful personal recalls
clear focus without deleting beliefs or correction history. A successful typed
recall or explicit write establishes focus again. On reading old session state,
a focus pointing to an absent value is invalidated. Unicode spelling, source
turns, correction provenance, exact last utterances and reset remain supported.

The established calculation contract deliberately preserves personal focus
across a successfully solved calculation. This is an existing compatibility
exception, retained to keep the original calc27 expectations unchanged. Story
and explicit fact-set turns continue to clear personal focus. Explicit fact-set
routing still precedes memory and is unchanged.

With no story context, a learned personal read can retrieve typed memory when
there is no supported story operation. The router checks a fresh story parser
before accepting that read, so an unsupported personal question cannot create
invalid story state. Existing story context and supported typed story operations
keep precedence. Other learned intents still describe the recognized request
and clearly say no external service is connected.

The frozen experiment, authored contracts, baseline/candidate outputs, hashes
and reproduction script are in `tools/conversation/memory-authority`. This is
bounded product development coverage, not independent dialogue accuracy. No
runtime LLM, new training, downloaded data, phrase blacklist or answer table
is introduced. Explicit statement parsing retains its existing narrow grammar;
this work does not claim to understand arbitrary names or preference syntax.

Measured: the original memory smoke improves from 40/41 to 41/41 with the actual
production intent model and remains 41/41 without it. All 93 preregistered new
checks pass in both modes; calc27/story47 results remain unchanged. A later
supplemental review probe is kept separately: bare `goodbye` is still intercepted
by the main talk story parser (baseline and candidate identical). The standalone
everyday chat recognizes the learned social intent and clears correction focus.
General social/story routing remains incomplete; the supplemental failure was
not removed, relabeled or used to broaden this patch. See HANDOFF.md for totals.
