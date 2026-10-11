# Parent integration review

V2 supports explicit Rust names that are also English words, without promoting
those words to unrestricted noun phrases. The first candidate is still rejected:
it passed the new local contracts but broke three existing grammar tests. V2
uses a separate category consumed only in explicit location and rename roles.

Parent source review found no symbol-name exceptions or new mutation machinery.
The existing world still resolves entities, proposes exact changes, validates
and rolls back edits. Known limitations remain: bare ambiguous names require
clarification; owner-qualified queries can work where the conservative field
rename engine still refuses; raw identifiers and malformed-quote policy are
outside this change. A separate worker is investigating field ownership.

The original 136-file submission was archived before integration. Parent verified
129 manifest entries, four frozen files and 12 source/binary/adapter identities.
All four source files were integrated byte for byte. The independent parent
underscore-call tokenizer fix is a separate commit and remains included.

Independent integrated checks reproduce the worker's full 42 outcomes, exact
files and reply text: 33/42 → 42/42, retaining all 33 previous successes. Five
queries and four verified renames improve; there are 14 verified renames and
seven unchanged negative contracts. The parent also reran the fixed eight-case
sample first. All eight independent spelling probes remain safe, with seven
actual edits now passing their unchanged native behavior tests. The original
spelling-only candidate had six edits; the explicit-name grammar adds a correct
rename that previously refused. This is added coverage, not a relabeled failure.

All 43 English tests and workspace fmt/clippy/release/strict4smoke pass. Full
1,340-pair grammar and 120-document reading parse reports are byte-identical to
the baseline: grammar1good/3bad/1336ties, reading1/120 parsed. These are weak
legacy parser measurements, separate from byte probability and passage QA; no
new likelihood or QA evaluation is claimed. Full765 is complete on coding commit
5f2859a730f6d20a0ac64ca9a7551821ed123e86 and binary
ca120ec676a8979d9116ef10380e58cd6f7f3d033fc97ade968a7d510c5b1bd7.
It retains all 174 previous passes, with 591 refusals and zero failures; all
765 outcomes are unchanged. Both coding changes meet the integration gate.

Original export26/27 and AddField14/15 mismatches remain in the worker evidence.
The latter has a live-path-sensitive rollback fixture that fails under inherited
native source relocation; this grammar change does not repair or worsen it.
No scorer or assertion was altered. Parent did not repeat every unaffected
adapter: it replayed the new contracts, native behavior, full parser inputs,
workspace checks and the full coding split.

Raw parent artifacts: `/home/gsc/.cache/glassmind/continuous-20261010/coordinator/identifierroles-review`.
Original archive: sibling `identifierroles-run/submitted`. Original worker binary
is aaf8887135081b3462c0051b6f3750390ca5df56a631eab25fb46005a0b83d7a.
The inactive worker's worktree and login link were removed only after another
archive/repository-copy verification. All data in the new authored contracts is
local development material under the workspace license; no hidden data or new
model training was involved.
