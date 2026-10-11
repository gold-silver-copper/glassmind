# Raw-text apostrophes and clitics in the main syntax command

Raw tokenization now matches the complete gold word sequence on **1,556/2,001**
EWT development sentences, up from1,370. All1,370 previous matches remain;445
sentences still mismatch. The denominator, capitalization, spelling and gold FORM
strings are unchanged. This is word segmentation accuracy, not dependency or
semantic understanding. No parser weights or supplied-form behavior changed.

Internal straight/right-curly apostrophes join letter runs; terminal English
clitics split into exact source slices, including stacked clitics. Case and UTF-8
bytes remain intact. Names retain internal apostrophes unless their spelling has
an ambiguous clitic ending. Outer quotes and other punctuation retain scalar
handling. Left-curly opening quotes, numeric punctuation, abbreviations, URLs,
unmarked clitics and sentence segmentation remain unsupported by this improvement.
The raw marker is now `raw_unicode_apostrophe_v2`; `provided_forms` is unchanged.

The parent independently recomputed all2,001 transitions:1,370 retained matches,
186 new matches,zero losses,445 continuing mismatches. All27,446 spans are ordered,
nonoverlapping exact UTF-8 slices and cover every non-whitespace source byte once.
Every emitted tree passes structural checks; this does not make its linguistic
edges correct. Among1,753 unchanged token sequences, all output fields except the
intentional tokenizer marker remain identical. Of445 remaining mismatches,62 have
changed token sequences and383 are unchanged; all are retained as errors.

All68 TRAIN-duplicate FORM sequences remain in the official total. Their exact
matches rise61->62; nonduplicate sentences rise1,309->1,494 out of1,933. The full
score excludes nothing. No new raw dependency alignment metric was invented;
gold-form output compatibility is checked separately. Existing parser/QA errors
and the gold-tokenized dependency scores remain unchanged.

Before candidate inspection, the parent froze20 linguistic and8 source-span
probes plus6 invalid inputs. The baseline gets0/20 linguistic cases; the candidate
gets20/20. All28 source-span cases and6 invalid refusals pass. The runner gained
an explicit standalone-adapter option to call the worker binary; cases and scoring
remained fixed. Main-command replay later matches every standalone output exactly.

Integrated source: `028ed241fbc7de684409ab9a577944daf0618088`.
Main SHA256: `981f8b100cd82ec9edfa23f2c2d379f2dfb17976e6b32ebab2c68d335c6efdb7`.
Raw full prediction SHA256: `4c7b1b65161ebb82bd4c5f1bb7a18646dba2b2d6c613f5986f6c0cd51dbe676a`.
Provided-form full SHA256: `059e5ca85b312f9ea692d01fcb30e67f6334fad002b704e686c59ecb7638710d`.
Model SHA256: `47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6`.

Validation:14 reading tests;17 main syntax contracts covering full raw/form outputs,
first100, authored probes, invalid inputs, source spans and CLI errors;14 unchanged
reading/QA command contracts; workspace fmt/all-target clippy/release/strict4smoke.
All2,001 supplied-form outputs and actual model serialization remain byte-identical.
The main command produces every frozen raw prediction byte for byte. There is no
coding runtime change; the previously validated180/765 result is retained without
repeating that unrelated full evaluation.

The parent verified30 archive entries1,665,136bytes,28 repository entries,
42 external artifacts49,131,913bytes and30 frozen identities before integration.
Original source, patch, reports and raw outputs remain at
`/home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/submitted` and
`candidate`. Large transition/error ledgers stay there; repository copies retain
their manifests. Unchanged source identities can be recovered from base commit
1ab5b03643b34fcca338bb1744758ed240b0ef0a after the worker checkout is removed.
All original/root copies, external artifacts and frozen identities were reverified
before removing the inactive worktree and temporary login link. Parent artifacts
are under `coordinator/rawsyntax-review` and `rawsyntax-independent`.

[Independent full audit](parent/full-audit.json), [probe results](parent/candidate.json),
[main contracts](parent/main-contracts.json), [identities](parent/integrated-identities.json)
and [original worker handoff](HANDOFF.md) contain reproduction evidence. The worker's
initial ACCEPTED wording means its own gates passed; parent acceptance follows the
separate checks above. No post-DEV retuning or new model training occurred.

Data: UD English EWT r2.18, revisionb7711cce01cdd4f5fcc0a8199b8a50d951b16c0c,
CC-BY-SA-4.0; TRAIN/DEV/README/license16,977,603 reused bytes,zero downloads.
No official test data was accessed. See [NOTICE.md](NOTICE.md) and provenance.
