# Native confirmation of repair ambiguity

Integrated source `e03437a81f2d047babad5e122aec99a88b1c56af`, binary SHA256
`b53e5f21899189d7e51d8a8ce87aeab561ea92e3196d949cb6ff20d37661e133`.
The original worker submission (7 files,87,499 bytes) is archived and hash-verified
under `WORK/repairambiguity2-run/submitted`; HANDOFF/results.json describe that
original proposal, before this parent correction.

The isolated interpreter cannot establish surrounding Rust bindings. An included
file using `u8` can inherit `type u8 = i8`. The functions `x <= 127` and `x == x`
disagree at128 as u8 functions, but128 is invalid in the i8 context; both functions
agree for every i8 input. A file outside the module graph also cannot supply
evidence just because its package builds successfully.

Both proposed outputs must now be confirmed by executed native Rust probes in
the original scratch package. Generated assertions validate interpreter outputs;
they do not choose an intended correct answer. Original assertions stay unchanged
and both candidates must already pass them. Probes use inferred argument types
and emit distinct execution markers. Successful cargo without a marker is not
evidence. Confirmation shares the10-second extra-search deadline and existing
memory/process caps. Unsupported interpretation, failed confirmation or exhausted
budget preserves the old first-candidate policy; none proves correctness.

Independent parent validation:

- 25 world-Rust tests pass, including exhaustive u8/i8 comparisons, native
  binding/execution checks, ambiguous versus constrained repair, original
  source/test preservation and bounded timeout behavior.
- Fresh sample2verified/2refused and full27:5verified/22refused. Every verdict
  and all five remaining accepted patches match the worker. All108 original
  fixture hashes match preparation provenance.
- Both exact repairs again pass unchanged public integration tests. A separate
  native replay confirms their differing outputs on the retained input.
- Workspace fmt, all-target clippy, release and strict4/4 smoke pass.
- Full765 dev:174passes,591refusals,0failures. All174 previous passes remain;
  exactly three failing styles become refusals. All other762 outcomes match.
  This improves reliability, not repair coverage.

[Integrated evidence](integrated/validation.json) and
[full comparison](../../docs/results/coding-ambiguity.json) retain exact hashes.
No held-out evaluation, expected implementation, hidden test or generator content
was inspected. No new external data, models or dependencies.

Reproduction uses the unchanged adapters in `integrated/replay.sh`; choose fresh
output paths for deliberate sample/full27 and native-witness replay. Checks:

```bash
WORK=/home/gsc/.cache/glassmind/continuous-20261010
TMPDIR="$WORK/coordinator/ambiguity-check/tmp" CARGO_BUILD_JOBS=1 tools/check.sh
TMPDIR="$WORK/coordinator/ambiguity-check/tmp" \
  GLASSMIND_SCRATCH="$WORK/coordinator/ambiguity-check/new-tests" \
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 \
  cargo test -p glassmind-world-rust -j1 -- --test-threads 4
```

Only the documented bounded scalar subset/candidate window is checked. A surviving
repair is not proved correct. Original first-candidate search and file-copy latency
retain their prior limits; this batch bounds added search and native confirmation.
