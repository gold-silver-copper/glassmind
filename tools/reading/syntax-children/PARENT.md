# Child-history training representation rejected

The parent independently rescored all2,001 retained DEV predictions against the
unchanged EWT audit: LAS19,423→19,443 (+20) and UAS20,540→20,515 (−25) over25,148
integer words including punctuation. Both frozen corpus gates fail. All trees are
valid and no requests refuse. Independently recomputed transitions include789
previously correct labeled edges lost,809 gained, and50 exact labeled trees lost.
All68 TRAIN-duplicate sentences/137 words remain included and correct.

The parent froze20 modifier/question/word-order sentences with40 core edges before
candidate inspection. Baseline31/40 and15/20 complete becomes30/40 and14/20: one old
correct object is assigned nsubj under the final adverb, with no gains. The sentence
is “The nurse thanked the young doctor yesterday.” This independent preservation
gate also fails. Worker-authored82→84/94 core edges do not override it. No training
or candidate inference was repeated; only retained full outputs were rescored.

Verified original73-file archive/98,513,691bytes, parent28 entries/78,662bytes and
20 pre-DEV frozen identities/42,739,823bytes. Both candidate model versions' roundtrip,
unchanged tagger payload and v1 full-form/raw byte compatibility remain useful
validation evidence; they are not evidence of improved accuracy. Original source,
model, binary, commands, failures and complete ledgers remain external under
/home/gsc/.cache/glassmind/continuous-20261010/syntaxchildren-run. Parent audit is in
coordinator/syntaxchildren-review; frozen probes in syntaxchildren-independent.

Both runtime files were restored byte-identically. No model or source is integrated.
There was one full TRAIN fit with fixed2 tagger/3 parser epochs and no DEV tuning.
Source is pinned EWT r2.18 b7711cce,CC-BY-SA-4.0; no new downloads or official TEST.
Two search objectives and this representation experiment failed useful improvement
gates. Move to another conversation track before another parser experiment.
