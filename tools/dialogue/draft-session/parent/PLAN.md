# Resumable provisional dialogue drafts

Gap: validated three-service model is only accessible through a batch binary,
with ephemeral MultiSession. It cannot resume a draft across process restarts or
participate in main talk sessions. Add a bounded, hash-bound replayable transcript
API and bundle loader; coordinator will integrate explicit main talk prefixes.
This is new persistence/usability, not an accuracy gain or booking/external action.
Existing610/1119 joint correct,459wrong50refused remains the measured limitation.

Freeze unchanged modelSHAfac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88
and fullpredictionSHA8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d.
Same first10/full179 utterance-only inputs. Require exact snapshot parity on1119
USER turns, including active/inactive frames, offer andboundary proposals, plus
restart/split/reversed/interleaved isolation. No model/weights/decoder changes.
No new data, retraining, thresholdsearch, scorer or held-out evaluation.

Bundle hashes exact raw model/schema bytes; required source/license/scope. Typed
history stores only USER/SYSTEM utterances and bundle identity, not predicted
states, annotations, frames, actions, API results or gold labels. Bounded loading,
128turn/64KiB total utterance limit,4096bytes/utterance, rejectempty/NUL. Validate
before mutating. Unknown serialized fields, badversion/hash or malformed history
refuse; wrong bundle cannot silently reinterpret old history. Explicit reset
clears only draft state. Snapshot is always provisional; never a factual answer
or confirmation of an external transaction. No runtime network/modelcall.

Coordinator main integration: opt-in --dialogue-dir, `Draft request: TEXT` USER,
`Service reply: TEXT` SYSTEM, `Show request`, `Reset request`. Reserve exact command
shapes before ordinary state interpreters. Preserve all memory/story fields,
clear correctionfocus; normal turns do not enter draft transcript. Main old JSON
must staybyte-identical when draftunused. Persisttranscriptwithmain --session;
missing bundle or mismatch refuses without mutation. Model/schema assets immutable.
Freeze main contract cases separately before implementation. Parentowns mainCLI.
