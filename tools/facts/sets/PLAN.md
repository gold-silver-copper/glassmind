# Prospective bounded fact sets — frozen before runtime edits
Base: 35d3ea80d63791af1625d5019077d6ea2cfa5f26. No benchmark gain claim.
Hypothesis: explicit set requests can expose genuine nonunique indexed claims
while preserving every legacy answer, JSON byte and refusal.

Grammar: trim outer whitespace; ASCII-case-insensitive exact `List answers to: `
or `Show answers to: ` prefix; remainder trimmed, one nonempty question ending
in exactly one `?`, at least three whitespace words, starting Who/Whom/Whose/
What/Which/Where/When/Why/How. No internal ?, !, semicolon, colon, newline or
control characters. Prefix-like `list answers`/`show answers` requests with
invalid syntax refuse; no conversational follow-up binding. Grammar checks shape,
not semantic truth/completeness; unchanged learned proposer and exact alias linker
operate on remainder only. No entity/relation mapping in parser.

Store::answer unchanged. New answer_set returns sorted unique IDs, first10,
accurate local total/returned/truncated. Forward dedup uses references, no copied
unbounded vector; inverse scans once with bounded output. Empty/unknown/ambiguous/
uncertain/unsupported refuse. No learned certainty or global completeness claim.

Explicit requests route before story/memory interpretation and intent fallback;
only conversational turn/last utterance/focus update, no beliefs. Default routing
unchanged. Missing bundle explicitly refuses. /reset retains loaded bundle.

Frozen authored contract matrix (fixture edges, not expected runtime answers):
forward multiple incl duplicate unsorted edges; inverse multiple; >10 forward;
>10 inverse; Unicode alias; missing labels; ambiguity; unknown alias; uncertain
relation; missing property; empty/malformed/multiple-question/trailing-action
prefix; unsupported followup; case-insensitive prefix; fresh/session/restart/reset;
interleaved name/preference/calculation/story and active-story external list.
Expected IDs independently computed directly from fixture edges in Python.
Baseline must refuse multi-answer questions and return no set responses.

Gates: all authored contracts pass; frozen first200 then full2821 standalone,
facts-enabled default questions and facts-disabled default talk byte-identical
baseline/candidate. Existing memory41/calculation27/story47 checks where feasible.
Small facts/CLI unit suites, targeted fmt/clippy/build. Real existing bundle example
with independently inspected published claim IDs. No new data/network/model tuning.
Record failures/rejected experiments, source/assets/binary hashes and exact commands.
