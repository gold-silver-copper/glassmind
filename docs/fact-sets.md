# Explicit local factual lists

Start `glassmind talk --facts-dir PATH` with an existing verified facts bundle.
Ask `List answers to: Who wrote TITLE?` or `Show answers to: Which works feature NAME?`.
These prefixes select list mode; the ordinary question still goes through the
existing learned relation model and exact alias linker. Unsupported questions
refuse. No web search or language model runs. Labels never become aliases.

Prefixes ignore ASCII case. The question must start with Who, Whom, Whose, What,
Which, Where, When, Why or How, have at least three words and one final question
mark. Embedded question marks, exclamation marks, semicolons, colons and control
characters refuse. Conversational follow-ups such as “And those?” do not bind to
previous entities. This checks request shape, not factual semantics; the learned
relation can still be wrong.

Replies show at most ten distinct IDs in lexical ID order, labels when available,
Wikidata links and the proposed property's direction. JSON `kind: "fact_set"`
adds `total_snapshot_matches`, `returned` and `truncated` to the existing fact
fields and source/model provenance. Missing labels are explicit. Counts refer
only to the loaded snapshot, never to every answer in the world. The list and
relation are suggestions, not logical certainty. Refusals return no IDs.

Explicit list requests work in fresh, persisted and active story sessions.
They advance the conversation turn and clear correction focus, without changing
personal beliefs or story facts. `/reset` clears the conversation but retains
the bundle. Ordinary memory, calculation, story and single-answer questions keep
their existing routing. Without a bundle, an explicit list request refuses.

`Store::answer_set(model, question)` exposes the bounded result directly;
`Store::answer` retains its existing single-answer semantics, including refusing
multiple local matches. No benchmark definition, model, index or score changes.
The unchanged bundle hash/provenance checks apply to list requests too.

See [prospective contracts](../tools/facts/sets/PLAN.md) and
[worker evidence](../tools/facts/sets/HANDOFF.md) for reproduction and limitations.
