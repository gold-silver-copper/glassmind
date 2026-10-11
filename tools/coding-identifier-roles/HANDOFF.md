# Handoff — one bounded identifier-role batch, uncommitted

Locally accepted V2; coordinator must run the parent full765 gate before
integration (all174 prior correct retained, zero failures). No benchmark gain is
claimed. Base checkout fd01b9bdc2ff68402841577fb3510422e6e47ec6; supplied baseline
runtime1331ac715eadcf7b9169423e947cedf861eb42a3. No commit, agents, new model calls,
services, downloads, API keys, credentials, held-out inputs, hidden generators,
private historical logs or evaluation-definition edits. Original archived
implementationroutes HANDOFF was read only as rejection evidence. No rejected
implementation-family source or fixture assertions were reused.

## Exact changes

- `crates/english/src/lexicon.rs`: opt-in IDENTIFIER token class, preserving exact
  spelling, admitting known English words only when a bridge requests that class;
  rejects multiword/path/punctuated atoms and tokenizer-trimmed plain words.
- `crates/english/src/sem.rs`: separate IdentifierMention category.
- `crates/english-rust/src/identifier_roles.rs`: six read-only productions for
  explicit kind/name orderings, owner constraints, location targets and rename
  destinations. Two scoped tests check spelling, atomicity and retained original
  alternatives. No symbol-name special cases.
- `crates/english-rust/src/lib.rs`: register the new rules/module.
- New owned adapter/evidence files and `docs/coding-identifier-roles.md` only.
  Main CLI, verifier, shared manifests, scorers, historical prompts, README and
  other tracks are untouched. Cargo.lock unchanged.

## Baseline, hypothesis and measured validation

Before runtime edits, discovery showed the earlier `implement_again` failure
was actually the source `caller`: quoting it works. Known English destinations
`renamed` and `called` fail independently. Explicit-kind location questions also
miss known vocabulary names. Quoted controls15/15, including seven world-verified
renames, establish source support separately. Original native assertion tests
actually run before/after changes. PLAN and the42 authored contracts/evaluator
were frozen before editing; FROZEN.sha256 still verifies byte-for-byte.

| Check | Baseline | Final candidate |
| --- | ---: | ---: |
| Fixed local sample | 5/8 | 8/8 |
| Full frozen local contracts | 33/42 | 42/42 |
| Verified renames in full set | 10 | 14 |
| Baseline successes retained | — | 33/33 |
| Exact unchanged negative contracts | 7/7 | 7/7 |
| Original declared-type adapter | 35/35 | 35/35 |
| Original export adapter | 26/27 | 26/27 |
| Original ordinary/export regression | 8/8 | 8/8 |
| Original type-rename adapter | 9/9 | 9/9 |
| Original AddField per-case replay | 14/15 | 14/15 |

Five gained queries: function caller/renamed/called and field called/renamed.
Four gained edits: implement→renamed, weigh→called, Record.called→saved,
Record.renamed→read. Other underscore, quoted, Unicode, kind and collision cases
remain correct. Exact expected file contents retain assertions, comments, strings,
semantic references and unrelated bytes. Independent inventory audit found only
expected files plus native Cargo.lock. Every changed case has native-test evidence
and full before/after output in reports/comparison.json and candidate-v2-full.json.
No new wrong edits in these bounded contracts; no broad safety proof is claimed.

English2 + English-Rust40 tests pass (42 total including two new tests).
Affected fmt, all-target clippy (`-D warnings`) for English/bridge/CLI, and CLI debug
build pass. No release build or full765 run was performed. Reproduction script
assembles the commands actually run individually; a second consolidated replay
was not run unnecessarily.

Original regressions were reused unchanged. Export's `unsupported_type` expects
refusal although both versions correctly export Vec<u16>; original failure kept.
AddField's live-rollback fixture is edited by both versions, then the unchanged
adapter crashes unpacking expected=None. Its live-path-sensitive test does not
prevent the inherited edit under current check isolation. The full runner stops
there; additional original per-case invocations cover all15, preserving14 passes
and that failure. Final live-rollback source bytes match between versions. This
is a real inherited limitation, not a passing regression. No fix is attempted
outside ownership. Row predicates match across binaries; path-dependent prompt
strings are normalized only in the separate comparison report, not in scorers.

Post-implementation owner audit: baseline4/8 → candidate6/8. The two gains are
location queries with Change/Ledger owning same-named fields. Both rename
expectations fail on both versions, including quoted control; retain failures,
exclude from frozen gain. Fully quoted ambiguous-field rename yields explicit
source ambiguity in both binaries. Bare unknown/negated/multiple-action forms
remain unchanged. Baseline frozen ambiguous-with-unquoted-destination is a parse
failure; frozen two-functions becomes an explicit ambiguity refusal in V2.
These are different outcomes, not interchangeable evidence.

## Rejected experiments and limitations

V1 broad Mention promotion passed42/42 local contracts but failed three original
English-Rust tests (37/40): count suffix, field-type qualifier and vague rename
interpretations changed. Rejected source, tests, partial tracked patch and binary
are retained; do not integrate V1. REFINEMENT.md records
V2's narrower hypothesis before its edit. The gate/scorer was never changed.
A first V2 compile failed due to Question enum-name shadowing; the compile log is
retained, then the import alias was corrected. No runtime source change followed
successful V2 validation other than rustfmt.

Final scope requires an explicit kind for the new rename route. Bare
`rename implement to renamed` still requires quoting or adding `function`.
Unquoted lexical owner names may likewise require an explicit kind. Duplicate
field owners can remain unsupported even when a query resolves correctly.
No arbitrary Rust syntax, raw identifiers, combining-character normalization,
malformed-quote policy, general conversational normalization or synthesis added.
Existing alternative readings are retained in scoped tests, not proven for all
English. Parent765 and coordinator source review remain outstanding.

## Identities, data and reproduction

RUN=/home/gsc/.cache/glassmind/continuous-20261010/identifierroles-run
Baseline binary SHA256:
8cde5dc486025e05932db946ba4accb7b5401fcd27c530bc8539eb4dca5bb78b
Final `RUN/glassmind-candidate-v2`:126675832 bytes, SHA256:
aaf8887135081b3462c0051b6f3750390ca5df56a631eab25fb46005a0b83d7a
Rejected `RUN/glassmind-rejected-v1` SHA256:
372f7b5d51c5c247332663d5ce8654f00896c3a3f2149897229f619fda37de5e
Final runtime-only candidate.patch:9222 bytes, SHA256:
c843cd41ef360d9ffa3d0fc95f3735299b97c4c99bc7e54ee229ac08598ec268
Per-source and original adapter identities: reports/identities.json.
Frozen source/fixture hashes: FROZEN.sha256; complete owned files: MANIFEST.sha256.
Raw workspaces/native logs remain under RUN; retained result JSON is in reports/.

New data is locally authored public development material, identifier-roles-v1,
2026-10-11, MIT OR Apache-2.0. contracts.json40404 bytes; generator3040 bytes;
42 expanded fixture snapshots11387 bytes, request text1496 UTF-8 bytes.
No downloaded data/model bytes. Exact source, version, bytes and SHA256 are in
data-provenance.json. Supplemental owner audit is explicitly post-implementation.

From the checkout root (fresh output directory):

```bash
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/identifierroles-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/identifierroles-run/target
bash tools/coding-identifier-roles/reproduce.sh \
  /home/gsc/.cache/glassmind/continuous-20261010/identifierroles-run/replay
```

All cargo and CLI mutation processes run under 8GiB/0swap systemd scopes,
setsid --wait, timeout --kill-after=10s600; cargo-j1 and tests4 (fmt excepted).
TMPDIR is supplied persistent Btrfs, never /tmp; explicit owned disk
GLASSMIND_SCRATCH is set on every mutation/test run. CARGO_TARGET_DIR is retained.
Inherited verifier uses an additional inner deadline with kill-after1s; unchanged.
Outer worker memory.max8589934592 and swap.max0 were checked. No child jobs,
background services or model processes survive this handoff. No host model or
login configuration was changed; no authentication file was opened.
