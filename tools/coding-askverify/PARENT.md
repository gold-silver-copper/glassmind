# Verified application through `ask --apply`

`ask --apply` now routes rename proposals through the same typed world operation
as `do`, including rule checks, build verification and rollback. It renders the
result after application: JSON is one final outcome, failures produce a refusal
and nonzero exit, and saved history records the actual result. Successful
application clears the pending proposal; nested assumptions remain visible.
Ordinary previews and the existing eligibility of `--apply` are preserved.

Parent validation at source `336d5f9`:

- All 20 authored CLI contracts pass, compared with 7 on the frozen baseline.
  Six non-application outputs are byte-identical.
- Three additional parent probes pass: disabling verification rolls back,
  missing Cargo manifests prevent application, and a Unicode type rename works.
- Ten CLI tests and the original renamed fixture's behavioral test pass.
  Workspace format, all-target clippy, release build and all four smoke cases pass.
- Memory 41/41, calculations 27/27 and mixed stories 47/47 remain correct.
- All **765 coding dev outcomes are identical**: 174 pass, 591 refused, 0 failed.
  Every previous pass is preserved; this is a reliability change, not a new
  benchmark-coverage gain.

[Validation](integrated/validation.json), [complete outcome comparison](integrated/dev-regression.json),
[CLI contracts](integrated/full.json), [session and routing checks](integrated/supplement.json),
and [fresh parent probes](integrated/independent.json) retain the evidence.
The first no-manifest probe accidentally used the reserved unquoted word `count`
as a new name, producing a parser refusal before application. The corrected
request used `measure`; both the authoring mistake and correction are disclosed
in the parent report. No implementation change was needed.

The original worker's 36 files (278,578 bytes) were archived and hash-verified
before integration and again before worktree removal. All 11 original public
fixture files and the evaluated source hashes were rechecked. The parent release
binary SHA256 is
`3dbbef685a95a397cae7c928630999e854115699e053f65722a933489168e873`;
full-dev result SHA256 is
`8218bc544bd292f897d4a9ff7365b50b8314e13b12b6cee8d696a8718a2e1da1`.
Raw parent runs remain under
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/askverify-{check,full}`.

This preserves the world's existing build-verification semantics for renames;
it does not prove arbitrary behavioral equivalence. Crash atomicity, filesystem
restoration failures and session-save I/O failures were not fault-injected. A
failed application may leave an older proposal retryable under existing session
semantics. No runtime LLM, dependency, dataset, scorer or world-verifier change.
