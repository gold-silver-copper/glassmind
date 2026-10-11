# Frozen raw apostrophe experiment

Base source 1ab5b03643b34fcca338bb1744758ed240b0ef0a; baseline official DEV
1370/2001 exact integer-word FORM sequences, 631 mismatches, 27745 raw tokens,
25148 gold words, 2001 predicted with exact source spans. Model remains
47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6.

Hypothesis: productive apostrophe segmentation alone recovers at least 100 whole
sentences without losing any baseline exact sentence. Acceptance >=1470/2001,
all1370 retained, every nonwhitespace source byte covered exactly once in ordered
contiguous UTF8 spans, same16KiB/128token bounds, no new DEV refusal, all frozen
authored contrasts pass; full2001 provided_forms predictions byte-identical and
model serialization byte-identical. This is raw tokenization, not LAS/understanding.

Rules selected from official TRAIN only, before production edits:
1. Existing alphanumeric/combining-mark runs and scalar punctuation stay default.
2. Straight U+0027 and right-curly U+2019 apostrophes may join a run only between
   letters (a combining mark attached to a preceding letter also qualifies on
   the left). Numeric-adjacent apostrophes remain separate; no feet/inches merge.
3. Split terminal ASCII-case-insensitive 's/'m/'d/'ll/'re/'ve off a nonempty
   run; split terminal n't with the n/N included in that suffix, leaving a
   nonempty stem. Apply right-to-left for compound clitics. Preserve original
   case/apostrophe bytes and all marks; no canonical expansion.
4. Other internal apostrophes stay inside the word, including names. Outer
   quotes, doubled quotes, lone apostrophes and terminal plural possessive
   apostrophes remain scalar punctuation. Left-curly U+2018 remains punctuation.
5. No whitespace crossing. No source normalization. No parser/model/form changes.
   Raw marker becomes raw_unicode_apostrophe_v2.

TRAIN evidence: 's904, n't645, 'm197, 'll148, 've124, 're101, 'd53; curly
’s40/n’t10. MWT examples can't -> ca/n't (65), won't -> wo/n't (34),
O'Connell/O'Rourke and other internal-name apostrophes are single words.
Search for scalar-tokenized internal apostrophe word sequences found only
numeric measurements (5'2,5'4,5'9,5'10,5'11,14'2), motivating numeric guard.
Token counts/examples are descriptive TRAIN evidence, not a runtime lookup.

Rejected alternatives before DEV: all-apostrophe merging (breaks clitics and
measurements); splitting every apostrophe (baseline); lexical name/exception
lists (unproductive); numeric punctuation/abbreviations, missing-apostrophe
expansion, URLs, hyphen rules (deferred to separate experiments). Ambiguous
suffix-bearing names and annotation inconsistencies are acknowledged limitations.

Freeze PLAN and authored.jsonl before source edits. Freeze candidate source,
binary, probes, evaluator and baseline hashes before first candidate DEV.
Run fixed first100 raw DEV, then full2001 once, no postDEV source/rule/contrast
patching. Report all transitions and remaining mismatches. Report TRAIN exact
FORM duplicate subgroup separately; retain all2001 in primary denominator.
Gold-form full replay uses supplied forms and approved model. Original adapter
contracts, reading tests/clippy/fmt/build; coordinator owns main/workspace checks.
If gate fails preserve patch/source/binary/report/manifest then restore runtime.
No retraining/downloads/agents/commits/background services/hidden material.

Source: UniversalDependencies/UD_English-EWT r2.18,
b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c, CC-BY-SA-4.0. Annotations ©2013–2021
The Board of Trustees of The Leland Stanford Junior University; underlying text
rights as recorded in source README. Reuse read-only supplied TRAIN15,034,493B,
DEV1,898,574B, README24,522B, LICENSE20,014B; zero new downloaded data bytes.
