# Historical-offer candidate diagnostic

The first inference found 87 initial missing ride destinations; 83 last appeared
earlier than the immediately preceding SYSTEM turn. Run the unchanged approved
OfferModel::propose over at most the last 32 SYSTEM turns, using the current
utterance and only predicted-active services with missing text slots. Keep
all proposals and provenance; do not apply them to session state or select a
winner. Count exact gold availability, conflicts, and incorrect unique values
offline. This measures candidate coverage, not a deployable retrieval policy.
