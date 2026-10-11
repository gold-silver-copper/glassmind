# Frozen child-feature experiment
Base: 6a346c3080a5dd94d7a6bc7dae32af7f6d969798. One TRAIN fit; no DEV tuning.
Hypothesis: second-outermost attached-child history supplies missing attachment cues.
Legacy baseline full2001: LAS19423, UAS20540 of25148 (including punctuation).
Prior rejected decoders: unnormalized width4 LAS17879; legal-logsoftmax width4 LAS19428/UAS20547. Neither retried.

Keep every legacy feature, tagger, hash bucket, static oracle, ordered 2 tag /3 parser epochs and deterministic greedy decoder unchanged.
For top stack positions p=0,1 and side d=left,right, first is min/max attached index;
second is second-smallest index<head / second-largest index>head. Maintain two indices
per side with constant work per attachment, bounded by128 tokens, no node cloning.
Append exactly eight features per (p,d), namespaces 100+16*p+8*d through +7:
0 [word(second)]; 1 [tag(second)]; 2 [label(second)];
3 [headtag,tag(second),label(second)];
4 [tag(first),tag(second)]; 5 [headtag,label(first),label(second)];
6 [headtag,tag(first),tag(second)];
7 [other-stack-tag,headtag,tag(first),tag(second)].
Missing child word/tag/label are explicitly0; real tag IDs +2, label IDs+1;
root/head sentinel stays legacy1. No directional valency or other new features.

Version ewt-arc-standard-children-v2 selected only from strict validated header;
legacy VERSION, train API and CLI default remain v1. Explicit train_with_child_features
API and standalone train --child-features enable v2. Unknown header versions reject.
Limits remain128 tokens/16KiB text/128KiB JSONL row/128MiB loader; model <32MiB.
Runtime consumes forms/text only; annotations only in TRAIN trainer.

Freeze source, binaries, model, adapters, probes and unchanged parent audit before
candidate first100 then full2001 once. Acceptance LAS>=19673 and UAS>=20540 of25148,
all2001 emitted valid trees; authored and independent parent old-correct core edges
retained. Report all gold-edge and sentence transitions, duplicate68/137 separately
with no exclusions. Parent supplement not accessed and remains outstanding.
Compatibility: v1 model byte-exact load/save and2001 forms/raw byte replay against
provided outputs; both versions roundtrip, corrupt header/weights/dimensions rejected;
native tests including40 raw contrasts and16 invalid/span contracts, fmt/clippy/release.
One full fit only. On failure archive exact source/model/binary/patch/reports and verify
hashes before reverting owned runtime files. Preserve evidence, never weaken gates.
Source existing read-only EWT r2.18 b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c,
CC-BY-SA-4.0 TRAIN15034493 bytes DEV1898574. No downloads or official TEST access.
