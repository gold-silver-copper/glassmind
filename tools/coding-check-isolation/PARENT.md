# Coordinator review: native checks leave live projects unchanged

The coordinator replayed all 16 authored cases using the frozen submitted binary.
Every outcome, before/after source hash and changed-path set matches the worker.
All diagnostic responses match after masking only OS-assigned panic thread IDs.
Passing/failing tests retain their findings; each actual compiler repair changes
only its intended source file. Existing hidden files and build artifacts remain.

The original 21 files (332,251 bytes) and all 21 recorded provenance entries were
verified before integration. Source review confirms unique scratch copies and
unchanged copy/freshness rules, native compiler commands, repair rules, rollback
and ambiguity logic. Result-cache generation is advanced to avoid old findings.
The archived submission was reverified against repository copies before removing
the worker tree and temporary login link; its scope is inactive.

Parent workspace fmt, all-target clippy, release build and strict four-fixture
smoke pass. All 29 world-Rust tests pass on the integrated source, including four
new integration groups. The frozen release executable and source commit are in
[parent/evaluated.json](parent/evaluated.json). Full765 passes: all 174 previous passes retained, 591 refusals and zero failures,
with every individual outcome unchanged. See [full comparison](parent/full-comparison.json).
This gate evaluates coding runtime `77bc246`. Subsequent memory-only changes in
`1331ac7` do not touch coding paths; the combined workspace checks also pass.

The worker's ordinary-HOME checks ran inside its launcher's masked cache mount,
which is a temporary filesystem. Those native TMPDIR paths therefore do not
establish persistent-disk compliance, despite their path spelling. The parent
replay uses the actual home-cache Btrfs filesystem; integrated tests and full
coding evaluation use explicit owned scratch paths on persistent disk. Preserve
this worker-run limitation rather than claiming retroactive compliance.

The dev runner now uses the required ten-second shutdown grace instead of five
seconds. Its 80/100/120/3600-second execution limits, dev split and scoring are
unchanged. Earlier completed benchmark runs had the older grace period.

This is normal Cargo artifact isolation, not an OS sandbox. Relocating a package
can change behavior when it depends on external relative paths, hidden/ancestor
configuration or absolute paths. Such unsupported topology may refuse; build
scripts retain caller authority. Explicit caller cache paths inside a live tree
remain outside the no-live-artifact contract. See the original scope document.

Raw parent artifacts:
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/checkisolation-review/`
and `checkisolation-full/`. Rejected implementation-language experiments remain
rejected; this independent change does not alter their tests or decisions.
