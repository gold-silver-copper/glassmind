# Provisional request drafts

`glassmind talk --dialogue-dir BUNDLE --session session.json` can retain a
provisional interpretation of a service request across restarts:

```text
Draft request: I need a taxi to Cedar Museum.
Service reply: How many passengers will travel?
Draft request: There are 3 passengers.
Show request
Reset request
```

Only the two explicit input commands enter the draft transcript. Ordinary
conversation continues to use personal memory, stories, calculations and facts.
Draft inputs do not update those stores, even when their payload resembles a
personal assertion or a story sentence. `Show request` leaves the draft unchanged;
`Reset request` clears only the draft and works without a bundle. `/reset` clears
the entire conversation. Command names are case-insensitive. Input text after the
colon and one optional separator space is retained exactly.

The current model covers RideSharing_1, Travel_1 and Weather_1. Output is a model
interpretation, with no booking, service execution or weather lookup. Persistence
adds usability, not accuracy: the existing selected SGD DEV result remains
610 right, 459 wrong and 50 refused out of 1,119 USER turns. All 1,119 native
prediction payloads are unchanged by the persistence API and main integration.

The bundle contains fixed `model.json`, `schema.json` and `manifest.json` files.
There is no automatic download. The manifest has exactly these fields:

```json
{
  "version": 1,
  "model_sha256": "SHA256 of the exact model.json bytes",
  "schema_sha256": "SHA256 of the exact schema.json bytes",
  "source": "Source and revision of the supplied model/schema",
  "license": "Applicable asset license",
  "scope": "Services and limitations of this model"
}
```

The loader validates both hashes, the existing model/schema compatibility rules
and nonempty provenance. A hash identifies bytes; it does not authenticate their
publisher. File limits are 64 MiB for the model, 1 MiB for the schema and 64 KiB
for the manifest. A declared invalid bundle fails startup.

`--json` replies use `kind: dialogue_draft`, `provisional: true`, a status and,
for accepted append/show commands, a snapshot with provenance and bundle identity.
The snapshot includes all service frames, active services and source proposals.
SYSTEM input records context; it is not a new USER prediction. Unsupported inputs
remain explicitly provisional even if a model proposes an intent or slot value.

The optional session field `request_draft` contains only a version, exact bundle
identity, and USER/SYSTEM utterances. Restoring a draft validates the whole history
and replays the unchanged model. Predicted frames, annotations or service actions
are rejected in saved histories. Changed model or schema bytes cannot silently
resume an old draft, even if only JSON formatting changed. A failed operation
preserves the saved draft; reset is the explicit way to begin under a new bundle.
In-process appends advance the model once, while show reads the current snapshot.

History is limited to 128 turns, 65,536 total UTF-8 utterance bytes and 4,096 bytes
per turn. Blank, NUL-containing and oversized utterances are rejected without
truncation. Unused drafts add no field to legacy session JSON.

The reused model and selected schema come from the existing SGD pipeline at
revision `e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0. Exact hashes,
source sizes, independent continuity checks and limitations are recorded in
[the parent review](../tools/dialogue/draft-session/PARENT.md).
