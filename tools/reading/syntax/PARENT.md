# Predicted dependencies in the main command

`glassmind syntax` now exposes the learned parser as explicit predictions:

```sh
./target/release/glassmind syntax \
  --model /home/gsc/.cache/glassmind/continuous-20261010/udsyntax-run/artifacts/model.bin \
  'Nora thanked Felix.'
```

Use `--json` for heads, dependency labels and exact UTF-8 source spans. Batch
`--input FILE` accepts JSONL rows containing only `text` or `forms`. Raw text is
bounded to 16 KiB and 128 tokens; it uses simple Unicode tokenization without
sentence segmentation or contraction expansion. Model and row loading are bounded.
These predictions do not authorize factual answers, memory updates or code actions.

The integrated runtime is `38fd9050d75a6499c306d802c8a779975cd74c23`; main release
SHA256 `226a97606e46889ac7d1005b513b13c9cb2c9343fd11c027752d3ec8dbd6dbf0`.
The frozen model is 12,554,724 bytes, SHA256
`47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6`.

Parent scoring independently checks every prediction against the pinned official
DEV source, without importing the worker scorer. On **25,148 gold-tokenized words**,
including punctuation, labeled attachment is **19,423/25,148 (77.23%)** and
unlabeled attachment **20,540/25,148 (81.68%)**. Every one of 2,001 emitted trees
passes independent root, range and cycle checks. Those checks establish structure,
not linguistic accuracy: 5,725 labeled edges and 307 root choices remain wrong.

All 68 TRAIN-duplicate DEV sentences remain in those totals. On the other 1,933
sentences and 25,011 words, the counts are 19,286 labeled and 20,403 unlabeled
matches (77.11% and 81.58%). Our earlier annotation-duplicate count used only
FORM/UPOS/HEAD/DEPREL, giving 68; the worker's stricter all-ten-columns count is
65. These definitions differ because the latter includes fields unused by inference.
The fixed previous-word chain gives 481 labeled/2,528 unlabeled matches. It is a
weak explicit comparator, not a competing parser or an older Glassmind score.

Parent-authored contrasts were frozen before inspecting candidate implementation.
Across thirteen sentences, both supplied-form and raw-text modes recover 68 of 70
requested core edges. Twelve sentences pass all requested edges in both modes.
“Who did Nora thank?” incorrectly attaches “Who” as a subject of “did”; the
expected object relation to “thank” is missing. The ambiguous prepositional phrase
has no asserted gold attachment. Exact UTF-8 spans pass. Original worker errors
on “Birds fly.” and apostrophe splitting remain unchanged. These small authored
checks are not a general raw-text benchmark or semantic verification.

The main command's complete 2,001-row output is byte-identical to frozen native
inference: SHA256 `059e5ca85b312f9ea692d01fcb30e67f6334fad002b704e686c59ecb7638710d`.
The first100, original authored/invalid-input cases and parent contrasts also
match byte-for-byte. All 16 new main-command contracts, all 14 existing reading
command contracts, all 12 reading tests, workspace fmt/all-target Clippy/release
and the strict four-case smoke gate pass. There was no second training run.
The coding implementation and its retained full765 result are unchanged by this
separate library and CLI addition; no repeated full coding evaluation is claimed.

Original 32 files/196,045 bytes were archived before integration. All 66 external
artifacts (60,359,758 bytes), five source entries, 27 repository entries and ten
freeze entries were hash-verified. All 31 source/report copies remained identical
after formatting. The inactive worktree and temporary login link were removed
only after repeating verification. Original worker paths in its unchanged manifests resolve to
the preserved archive; raw models/predictions remain external. The parent CLI
adapter and validation artifacts are separate additions. See
[identity audit](parent/identities.json), [independent scores](parent/score.json),
[contrasts](parent/independent.json) and [main contracts](parent/main-contracts.json).

UD English EWT r2.18 revision `b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c` is the
only new training source. TRAIN/DEV, README and license total 16,977,603 bytes;
the official test split was not downloaded. Code is MIT; model and derived corpus
artifacts retain CC-BY-SA-4.0 attribution and notices. See [NOTICE.md](NOTICE.md),
[source provenance](source-provenance.json) and [original handoff](HANDOFF.md).
Raw parent evidence is under
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/udsyntax-review` and
`udsyntax-independent`; original evidence is under `udsyntax-run`.
