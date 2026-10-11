# Data provenance

No new corpus downloads or external training sources. Existing read-only
UniversalDependencies/UD_English-EWT r2.18 revision
b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c, CC-BY-SA-4.0:
https://github.com/UniversalDependencies/UD_English-EWT/tree/b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c

TRAIN en_ewt-ud-train.conllu 15034493 bytes, SHA256
 d68e06122a702464c613076523d56740f047e5bbe89dd90ec32737e04d952143
DEV en_ewt-ud-dev.conllu 1898574 bytes, SHA256
 39239e0a60db3ae68f4b7036189f11b6692741d10ff8240dd91f74f2760d90f8
LICENSE.txt 20014 bytes, SHA256
 b3d1b0f4c6ae151f7eb78738f46ebd5ee140f8a7f76501ba26af123140d35ae7

Only TRAIN used in fitting. DEV used once fixed first100 then full2001 after freeze.
All integer words including punctuation and68 TRAIN duplicate sentences retained.
Official TEST/held-out data/hidden generators/old private logs not accessed.
Parent PARENT/HANDOFF summaries read as requested; parent case inputs not opened.

New authored development probes: authored.jsonl and forms-only projection
 authored-inputs.jsonl, created for this experiment under MIT. Exact bytes and
SHA256 in pre-edit.json. They are development evidence, not training examples,
runtime lookup tables or independent parent tests. Decoder test matrices are
independently authored development tests and a deterministic public generator.
