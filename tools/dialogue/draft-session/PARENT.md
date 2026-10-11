# Independent review: resumable provisional dialogue drafts

The unchanged three-service tracker now supports a bounded immutable bundle,
input-only saved history, replay, efficient in-process append and read-only show.
The main `talk --dialogue-dir BUNDLE` route exposes Draft request, Service reply,
Show request and Reset request commands. All output is provisional; no service
action or external factual answer is performed. This is a usability change,
not an accuracy improvement: retain 610 right / 459 wrong / 50 refused of 1119.

The parent froze six authored streams, main state-isolation contracts and loader
limits before inspecting the worker. The old native predictor supplied 13 USER
snapshots. Source review found the existing model, router and inference unchanged.
History accepts only version, bundle identity and typed USER/SYSTEM utterances.
The identity binds exact raw model/schema hashes and manifest version; replay
validates the complete input before mutation. Stored predictions cannot enter it.

Independent native checks passed:

- All 13 authored USER predictions, nine process restart boundaries and read-only
  show/reset checks; exact transcript bytes preserved.
- All 15 invalid bundles rejected, including valid JSON over each real file cap,
  incompatible schema, changed hashes, unknown fields and invalid model version.
- Ten malformed history categories refused atomically, including annotation-bearing
  turns, wrong identities, bad roles, NUL, UTF-8 size, turn and total byte limits.
- Exact positive boundaries at 128 turns, 4096 single bytes and 65536 total bytes;
  rejected appends leave history and snapshots unchanged.
- Two same-semantics raw asset changes cannot resume old history.
- All 1119 predictions across 179 DEV conversations match while interleaved, with
  179 midpoint restores. These are compatibility checks, not semantic gold.

Main integration passed 46 counted commands and 34 state checks in each of two
intent modes (92/68 total), with a fresh process for every command. USER payloads
match the old native predictions. Personal/story fields remain unchanged except
the ordinary turn bookkeeping and cleared correction focus. Ordinary memory,
story, calculation and fact messages never enter draft history. Missing bundles,
changed identities, malformed commands and annotated saved history refuse without
replacing it. Reset works without a bundle and recovers from invalid history.
Unused drafts leave legacy session JSON unchanged. A declared corrupt bundle
fails startup without modifying the session. Human output labels interpretations.
The full main dialogue replay separately matches all 1119 USER payloads.

Main clippy and release builds pass. Existing memory41, calculation27, story47,
production memory-authority93 and original memory41 checks pass. First200 and full
2821 standalone, facts-enabled and default-talk outputs remain byte-identical.
The worker additionally passed14 Rust tests, 179 SYSTEM-boundary restores,
reverse-order replay, interleaving, and a legacy full prediction byte comparison.
Worker baseline preparation initially reused a filename; corrected distinct paths
and a repeated baseline preceded implementation. Its original failure is retained.

Worker original submission:179 archived files /94263022 bytes, hash-verified before
inspection. Frozen inventory:116 files /93364413 bytes independently verified.
Original native example SHA654e6897ade4751364ced3d80ff4a131e434a953ef5e5a7eaedb71122959de0b.
Initial main SHA c6b9db4312edabde8edfa07fd9a74ce7b6c4ac1b4542dd6ae913d4b72388a310.
All reports identify their evaluated binary. No new data downloads or fitting.
Model32174720 bytes SHAfac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88;
schema3707 bytes SHAb021ae923dbeb7782b99ac6628e65b1b48f3cca2892fb62ce8c77b472324e089.
SGD source revision e852981ae34990f4358979625854259302feaa78, CC-BY-SA-4.0.

Reproduction scripts and compact reports are in `parent/`. Their original cache
layout is `/home/gsc/.cache/glassmind/continuous-20261010/coordinator/`
`draftsession-independent`; native inputs, prepared cap fixtures, raw streams,
frozen executables and all original worker evidence remain outside git. Main
artifacts are in `coordinator/draftsession-integrated` and original worker files
in `draftsession-run/terminal-originals`. Limits and commands are documented in
`docs/dialogue-drafts.md`. The snapshot's source metadata is descriptive; hashes
identify exact content rather than certifying its publisher.

Final combined binary SHA
`b6b093c5a82e76ec08481b5e0c8739e6b9090590f5f06b79d8b41eeb04e8948d`.
All26 world-rust files match the full765 candidate. CLI/dialogue source matches
the independently evaluated initial integration, except main help text whose
previous bytes were reconstructed and hash-verified. Workspace fmt, all-target
clippy, release build and strict4/4 fixture smoke pass. Combined source manifest
and check log are retained in `coordinator/combined-deduporder-draftsession`.
