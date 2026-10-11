# Original submission rejected; independent native checks pass

The original candidate remains rejected. Its declaration recognizer accepts
`fold_values needs a body, implement it` and `fold_values needs implementing,
implement it`, despite neither declaration being in the frozen accepted grammar.
The coordinator independently reproduced both acceptances with the retained
native parser. The same probe preserves an accepted stub declaration, an ordinary
query containing `write`, and refusal of a negated implementation request.
No full 765-task development run was performed for this rejected candidate.

The coordinator also ran previously frozen independent contracts against the
release binary. All 12 query/rename controls preserve exact stdout, stderr,
exit status and project file hashes. All 18 native cases pass: six verified
implementations, one ambiguous case refused, and eleven other unchanged controls.
The six successes cover two different sums with identical signatures, odd-item
reversal, Unicode character reversal, word sequences and character counts.
Their saved native evidence independently confirms all candidates were tried,
exactly one passed, the target was executed, and live tests passed after applying.
No assertions, manifests or unrelated source bytes changed. These contracts
establish bounded test-verified behavior, not general implementation correctness.

Original worker evidence reports 3/60 verified TRAIN requests, all three styles
of one function, with 57 unchanged refusals. This parent did not repeat that
TRAIN inference for the rejected candidate. The worker's 62 parser, 34 CLI and
20 native enumerated contracts did not catch the extra accepted productions;
the later freeze audit did. All prior implementation experiments remain rejected.

Original source, reports, results and patch were preserved before modification:
4,565 entries / 1,223,261,648 bytes, including six tool symlinks preserved as
link metadata without dereferencing. The first archive attempt stopped on a
systemd-run symlink; it was resumed without changing worker files. The separate
frozen candidate manifest was independently verified: 2,240 files / 612,259,441
bytes. Raw archives remain under
`~/.cache/glassmind/continuous-20261010/implgrammar-run`.
Release SHA256:
`1822949badd4ab902f630f9e6b092fd8081cff4b588ad966047182655f88caf1`.

The worker used 5,170,049 input tokens (5,032,704 cached, counted within input)
and 55,220 output tokens: 5,225,269 total. No new download or additional model
call was made during parent review. Source remains RustBench TRAIN revision
`f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, MIT OR Apache-2.0.

A separate coordinator correction subsequently passed the original acceptance
gate at 218/765, retaining all 215 old passes with zero failures, and was
integrated. It distinguishes unsupported declarations from accepted bindings
and preserves a literal `function` identifier. See the
[correction review](correction/PARENT.md). This original submission and its
historical evidence remain rejected.
