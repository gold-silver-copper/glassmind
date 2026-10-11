ACCEPTED bounded apostrophe tokenizer batch; uncommitted and ready for coordinator
review. Raw exact FORM sequences improve 1370 -> 1556 / 2001 (+186;
68.4658% -> 77.7611%, +9.2954 percentage points). Every baseline exact sentence
is preserved, exceeding the frozen >=1470 gate. This is a raw tokenization
result, not dependency accuracy, semantic understanding, or QA improvement.

Only production file changed: crates/reading/src/syntax.rs. Internal straight
and right-curly apostrophes join letter runs; productive terminal 's/'m/'d/
'll/'re/'ve and n't split into exact source slices, including stacked clitics.
Case and UTF8 bytes stay unchanged. Internal names stay together except lexical
ambiguities with clitic-shaped endings. Outer quotes, left-curly opening quotes,
numeric-adjacent apostrophes, other punctuation and whitespace behavior retain
scalar handling. Raw marker changes from raw_unicode_v1 to
raw_unicode_apostrophe_v2. Updated the existing contraction expectation and added
two focused Rust tests. All other changes are under tools/reading/raw-syntax.
No parser/trainer/model/provided_forms/QA/main-CLI/manifest changes; no retraining,
downloads, credentials, API keys, agents, new LLM calls, services or commits.

PLAN.md and 40 authored contrasts were hash-frozen before source changes
(pre-edit.sha256). Rules came only from official TRAIN boundaries: productive
clitics, internal names, and scalar feet/inches measurements. Selected one
candidate. Rejected before DEV: unrestricted apostrophe merging, a lexical
name/exception table, and retaining the scalar baseline. Numeric punctuation,
abbreviations, URLs, missing-apostrophe forms and hyphens were deferred. No
rejected implementation trials or post-DEV source/probe/evaluator changes.

FREEZE.json records 30 source/binary/probe/evaluator/input hashes, frozen at
2026-10-11T10:03:35.734697+00:00, before first candidate DEV inference. All 30
were reverified after evaluation. Fixed first100 raw DEV ran once, then full2001
raw DEV once, then supplied-form full2001 once. Authored baseline15/40 ->
candidate40/40; valid spans40/40 for each. No DEV exception patching.

| Metric | Baseline | Candidate |
| --- | ---: | ---: |
| First100 exact FORM sequences | 65 | 89 |
| Full2001 exact FORM sequences | 1370 | 1556 |
| Full mismatches | 631 | 445 |
| Raw tokens | 27745 | 27446 |
| Gold integer words (unchanged) | 25148 | 25148 |
| Predicted / valid source coverage and tree | 2001 | 2001 |
| New refusals / baseline exact losses | — | 0 / 0 |

All transitions: exact->exact1370, mismatch->exact186, exact->mismatch0,
mismatch->mismatch445. Of the remaining445, 62 changed FORM sequences and383
are unchanged. In total248 sequences change; all1753 unchanged sequences retain
identical forms/spans/UPOS/heads/deprels/status/model-version/validity fields,
with only the raw tokenizer marker changing. Every one of27446 candidate spans
is an exact UTF8 source slice, ordered and nonoverlapping; every nonwhitespace
source byte is covered once. Bounds hold and no sentence is refused.

reports/full-transitions.jsonl contains every official sentence in original
1-based order with text, gold/baseline/candidate forms, transition, statuses and
validity. reports/full-remaining-mismatches.jsonl contains all445 errors.
reports/remaining-analysis.json is post-evaluation descriptive edit-block
analysis, not an acceptance metric: repeated punctuation, ellipses, emoticons,
abbreviations, numbers and unmarked clitics remain prominent. For example55
edit blocks split an ellipsis and22 split a doubled hyphen. No rules were added
from these observations. Compound word/URL and corpus spelling mismatches also
remain. Numeric apostrophe spellings and ambiguous clitic-ending names are
conservative limitations; left-curly U+2018 is not treated as an apostrophe.

Duplicate accounting, without exclusions from the official2001 denominator:

| Subgroup definition | Sentences | Baseline exact | Candidate exact |
| --- | ---: | ---: | ---: |
| Gold FORM sequence seen in TRAIN | 68 | 61 | 62 |
| Gold FORM sequence not seen in TRAIN | 1933 | 1309 | 1494 |
| DEV text+FORM identity repeated within DEV | 133 | 107 | 107 |
| DEV text+FORM identity occurs once | 1868 | 1263 | 1449 |

All1370 old exact cases remain exact in these partitions. No deduplication,
normalization, subset scoring or denominator changes. Raw dependency errors
were not given an alignment score: none was preregistered, and no new alignment
was invented after evaluation. Valid tree structure does not establish correct
linguistic edges. Supplied gold-form prediction bytes are the compatibility gate.

Validation completed:

- Reading package tests: 14 passed (12 library +2 probability), including40
  frozen contrasts,729 short UTF8/quote/mark/number boundary combinations,
  16KiB and128-token acceptance/rejection boundaries, NUL/DEL/C1 controls.
- Reading package fmt check and all-target clippy with -D warnings passed;
  release reading-syntax build passed. git diff --check passed.
- Original untouched tools/reading/syntax/contracts.py:16/16 input/span contracts,
  including annotation rejection and malformed-model NaN rejection.
- Original untouched model_check.rs: actual learned-model byte-exact serialization
  roundtrip; deterministic valid trees at1/2/3/127/128, refusal at129.
- Full2001 provided_forms predictions byte-identical to supplied baseline.
- Outer worker scope verified MemoryMax8589934592 and MemorySwapMax0. Every
  cargo command used explicit TMPDIR, systemd-run8GiB/0swap, setsid --wait,
  timeout600/kill-after10, -j1 except fmt, and test-threads4 for tests.
- Candidate main CLI was not rebuilt/tested; full workspace/main contracts remain
  coordinator-owned. No inference jobs or child sessions remain running.

Exact source/artifact identities:

- Base git source:1ab5b03643b34fcca338bb1744758ed240b0ef0a.
- Candidate syntax.rs:587209fd2279c49c64c00b9d1fa8c72443213d8adcb6c5f86b9d010574383c91.
- Candidate release reading-syntax:91a444107279f19d63e7f2f00042ac073ee70455ca7768d9a7823d7466c382c2 (12,189,568B).
- Frozen model:47d8d245b575ab01afa6c0ef7471fc8b8571aede62c106b8fbe50ec9ccc01db6 (12,554,724B).
- Full raw predictions:4c7b1b65161ebb82bd4c5f1bb7a18646dba2b2d6c613f5986f6c0cd51dbe676a.
- Baseline and candidate full forms predictions:059e5ca85b312f9ea692d01fcb30e67f6334fad002b704e686c59ecb7638710d.
- First100 raw predictions:2d5d5be078d39875dcfa1f72831c7c014afc7e692880fe414c4edee547c3b208.

Raw binary/model/inputs/predictions, source snapshots, patch and all command logs
remain in /home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/candidate.
The supplied baseline is its sibling baseline directory; it was not modified.
ARTIFACTS.json records external artifacts and repository evidence hashes/sizes.

Source attribution: UniversalDependencies/UD_English-EWT r2.18, revision
b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c; CC-BY-SA-4.0. Annotations/database
rights ©2013–2021 The Board of Trustees of The Leland Stanford Junior University;
underlying text rights as recorded in source README. See NOTICE.md and
source-provenance.json for original links and hashes. Reused TRAIN15,034,493B,
DEV1,898,574B, README24,522B, LICENSE20,014B, total16,977,603B already supplied.
Zero new downloaded data bytes. Derived reports are newly written evidence,
not new downloaded corpus data. No official TEST/held-out/private-log access.

Reproduce from this worktree with the retained baseline/source and existing target:

```bash
TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/tmp \
CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/target \
tools/reading/raw-syntax/reproduce.sh
```

The script performs scoped fmt/test/clippy/build, freezes hashes, evaluates
40 authored contrasts, first100, full2001, full2001 forms, original contracts and
actual model roundtrip. It writes a separate reproduction directory by default;
SOURCE/BASELINE/OUT may be overridden. The original batch ran these steps
individually, not by repeating the consolidated script. TRAIN inspection can
be reproduced independently without DEV access:

```bash
python3 tools/reading/raw-syntax/inspect_train.py \
 /home/gsc/.cache/glassmind/continuous-20261010/udsyntax-source/en_ewt-ud-train.conllu \
 /home/gsc/.cache/glassmind/continuous-20261010/rawsyntax-run/tmp/train-inspection.json
```

Coordinator should independently recompute raw metrics/contrasts and run its
main-command/workspace gates before integration. Acceptance here authorizes no
dependency-accuracy or understanding claim. Production patch remains applied
because all worker acceptance gates passed; nothing is committed.
