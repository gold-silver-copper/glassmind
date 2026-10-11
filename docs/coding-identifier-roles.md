# Coding identifier roles

This local development batch adds exact identifier readings in explicit Rust
kind/name positions used by location queries and renames. For example,
`where is function caller?`, `rename function weigh to called`, and
`rename field renamed on Record to read` now work. The unchanged world verifier
still resolves targets, rejects collisions, applies edits, and checks builds.
Independent native assertion tests and exact source comparisons verify mutations.

`IDENTIFIER` is an opt-in English lexical class, preserving token spelling rather
than excluding words already in the English vocabulary. `IdentifierMention`
keeps these readings separate from ordinary mentions. Six Rust bridge productions
compose kinds, names, owners, location targets, and rename destinations. Existing
rules remain present. There is no name-specific logic, runtime model or synthesis.

The frozen authored gate improves from 33/42 to 42/42: five new location answers
and four new verified renames. All 33 baseline successes remain. All seven negative
contracts preserve the exact source; 14 candidate mutations pass native tests,
retain original assertions/comments/string literals, and change only expected
identifier references. These are local development results, not benchmark gains.
The fixed eight-case sample improves 5/8 to 8/8. Forty-two scoped English tests,
affected all-target clippy, fmt and the CLI debug build pass.

An initial broad `Mention` implementation passed the new gate but failed three
existing language tests; it is rejected and retained in `rejected-v1/`. The final
separate constituent avoids those regressions. Bare vocabulary source names and
bare-source renames to vocabulary destinations still may require quotes or an
explicit kind. This does not add arbitrary Rust identifier syntax, raw names or
Unicode normalization. It retains the tokenizer's existing quoted-name behavior.

The original declared-type35, ordinary/export8, and type-rename9 adapters retain
all results. The original export27 retains its one obsolete `unsupported_type`
expectation. The original AddField15 retains an inherited live-rollback failure:
both binaries edit, after which the original adapter raises TypeError. Individual
case runs retain 14/15 passes. No assertion or scorer was changed.

A post-implementation owner audit adds two successful location answers with
lexical type `Change` and duplicate field names. Its two rename expectations
fail on both binaries, including the quoted control: source support is limited,
not evidence of a parser regression. A fully quoted ambiguous-field request is
refused by both as source ambiguity. This audit is excluded from the frozen gain.

See [HANDOFF](../tools/coding-identifier-roles/HANDOFF.md), including hashes,
provenance, reproduction commands, rejected evidence and parent-gate limitations.
