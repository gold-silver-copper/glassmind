# Prospective implementation grammar experiment, 2026-10-11

Hypothesis: fully consumed role-aware action/target grammar preserves PR43
identifier routing while selecting only a Rust target for the old finite native
family. Prior three rejections remain rejected. Baseline source 3f7ce8f;
supplied PR43 binary and all 40 public source files verified against input manifest.

Acceptance (unchanged after observation): all 34 authored boundary outputs and
all-file before/after hashes identical to measured current baseline, including
unsupported AddField phrasing and ambiguous rename as unchanged refusals. All
62 frozen parser contracts (9 accepted, 19 explicit refused, 34 other-route
entries; duplicate requests remain) must match. All 20 native contracts pass:
6 implementations, 14 unchanged refusals including genuinely two passing
str->usize alternatives (chars vs words, distinct on "ab") and live rollback.
Six positive original packages must compile and execute failing tests before edits.
Fixed sample first three native positives plus all boundaries, then full authored.
Environment tests set/unset target, persistent disk TMPDIR; no refusal artifacts.
Then first three styles of rb-implement-sum-even (chosen from old retained TRAIN
report), then full 60 exactly once. At least three native-verified TRAIN requests,
all refusals unchanged. No DEV/full765 worker run. Parent alone requires >=218,
all215 prior passes retained, zero failure and unchanged denominator.

Accepted grammar:
  request := greeting? (polite? action target location? suffix? terminal?
             | declaration separator polite? action bound_target? suffix? terminal?)
  action := implement | write ('the body of')? | fill in | fill it in
  target := bare Rust identifier | balanced backtick/single/double quoted identifier
  location := in src/lib.rs
  declaration := ('there\'s a')? target function? location?
                  (is | that's | =) still? just? a? (todo!() | `todo!()` | stub | todo) still?
  suffix := (per|following) (the|its)? doc comment | so it does what its doc comment says
  polite := please/pls and/or could/would/can you, optional please/pls
  greeting := hi there, | hello! | hi! | hey! | hey,
  terminal := . | ?; optional trailing please/pls before terminal
Bound pronouns have exactly one declaration antecedent and must agree with an
explicit target. Qualified owner names and paths are lexically indivisible,
recognized but unsupported for synthesis. Quoted constituents never act as verbs.
No negation, compound actions, behavioral suffix, extra sentence, arbitrary path,
malformed quote or unresolved pronoun is accepted. Action classification occurs
only at grammatical heads/declarations or explicit unquoted clause boundaries,
never substring scans or reservations of identifier spellings.

Native generator: copy reviewed old candidates(sig) byte-for-byte, unchanged
catalog. Correctness audit adds executed-target sentinel probe, explicit compiler
error rejection, checked external scratch root and saved unique-choice evidence.
Tests only verify candidates. Neither requests, docstrings, identifiers nor
assertion values choose algorithms. Every candidate starts from original bytes.
Nonempty executed failure, full bounded enumeration, unique survivor, package/rule/
staleness checks and verified live apply/rollback remain required. Wording stays
"test-verified", no specification proof claim.

Archive source, binaries, complete patch/new files, commands, failures, evidence,
manifest before restoration if any gate fails. fmt, core English/roundtrip,
world/CLI meaningful tests, affected all-target clippy and release are required.
No agent/model call/download/commit. MIT OR Apache-2.0 authored fixtures;
RustBench f79e71a supplied data MIT OR Apache-2.0, exact bytes in input manifest.
