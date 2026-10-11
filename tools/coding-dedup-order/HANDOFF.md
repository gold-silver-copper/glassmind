# Dedup/order worker handoff

Measured TRAIN improvement: **8/27 -> 9/27**, 18 refusals, **0 failed
verifications**. All eight prior entire outcome files are byte-identical.
The original fixed sample remains 2/4. Selection SHA is unchanged:
`6eea206a106aa053c7e458e5975f7a4e2c4c2b19a7a066510ad454f2985e6164`.
The one new repair swaps only the adjacent dedup/sort statements in public
TRAIN blog-tags. No names, comments, prompts or expected outputs enter proposals.

Changes:
- `crates/world-rust/src/dedup_order.rs`: root-file AST/type checks for a unique
  mutable binding anchored by owned `Vec<T>` parameter or explicit local type;
  primitive total-order elements/String, including Unicode identifiers and
  canonical raw-identifier shadow detection. Adjacent standalone zero-argument
  dedup/sort only. Exact source-span swap preserves everything outside the pair.
- `checks.rs`: append proposals after every old family including extremum; extract
  its existing bounded native-choice helper, preserving its budget, ordering and
  record format. Each alternative starts from the original tree. New family has
  <=8 candidates, 20-second deadline, requires complete native checks, exactly one
  pass and saved evidence. Errors, compiler findings, incomplete choices and two
  passes refuse. Old withdrawn/incomplete choices never fall through.
- `lib.rs`: register module. `dedup_order_tests.rs`: public native contracts,
  exact patch/manifest/test preservation, ambiguity, candidate cap, application
  and compilation errors, and older-extremum withdrawal blocking a passing new
  candidate. Legacy proposal/undo, cargo and run_in_until functions byte-identical.
- This tools directory contains frozen plans, authored fixtures, replay/comparison,
  audit/freeze adapters and reproduction commands. No other ownership changed.

Validation on final source:
- 54 world-rust unit/integration tests pass, including existing extremum native,
  argument-swap structural/namespace, scalar ambiguity, priority and isolation.
- 29 authored native cases: 13 accepted, 15 excluded, one two-passing refusal.
  Every original compiles and fails an executed test. Six numeric widths,
  owned/explicit local/String pipeline, Unicode/raw shadowing, bool/char,
  inline cfg(test) assertions, empty/singleton/sorted/reversed/adjacent and
  nonadjacent duplicates. Exact byte checks preserve the rest of the program.
- New vector alternatives independently pass, yielding respectively
  `[1, 2] [1, 2, 2]` and `[1, 2, 2] [1, 2]`. Main CLI refuses with complete saved
  choice.json (two successful checks, two passes, incomplete=false).
- Old public authored argument-swap ambiguity also independently passes both
  alternatives, yields differing tuples, and is refused by main CLI with its
  existing choice evidence. No scalar/loop witness was invented.
- `cargo fmt --all -- --check`, crate all-target clippy with `-D warnings`, release
  main binary build, fixed TRAIN sample and full27 replay all pass. All cargo
  calls use persistent TMPDIR, supplied target, 8GiB/0swap scopes, setsid --wait,
  600-second outer timeout, jobs1 and testing threads4. Outer worker cap verified.
- First clippy attempt rejected four slice-index expressions; fixed with slice
  destructuring. A separately frozen review supplement fixed raw-name shadow
  counting and added owned String/bool/char. All earlier attempts/logs retained.

Source/provenance: base HEAD `c0efc7acc6d7575076cd4f2a5566c7e0681a0c45`.
Provided baseline binary SHA
`4087feb06e9e72f87674be8ddba20bf119cd14537b9c73ae8ff993cad8b10552`.
Candidate binary SHA
`dc5f5cccbc76d8749be40b083b003f7d6e2074c16c1bb0829c282ce645fcdb2d`.
Existing readonly public TRAIN source is RustBench `f79e71a`, MIT OR Apache-2.0,
110 files/57,265 bytes; every supplied manifest hash was verified before and after.
No external data added or downloaded. New authored fixture data: cases.json
10,488 bytes and supplement.json 1,342 bytes, under project MIT OR Apache-2.0.
Original25 were frozen before production edits; supplement4 separately before
its review fix. No existing cases, evaluation definitions or labels changed.

Evidence root:
`/home/gsc/.cache/glassmind/continuous-20261010/deduporder-run/frozen-candidate`.
Contains original/final source, complete diff including new files, both binaries,
inputs/reports/output records/native alternatives/choice evidence/logs, and
verified per-file `hashes.json`. Full TRAIN report SHA:
`207813c94e99416e7efe24f928c35fec68b30b9b732a72de4217eddd9fafca7e`.
Comparison SHA:
`61b35bcc73a17c8e803b3f20cab5d539357f61ef510e447608ecb6a277e769a0`.
Main-CLI ambiguity report SHA:
`9f1e92ea893af9eeceb1ba4a968c76c9e11c14cd3f2672df4e09122ced54cc38`.
Exact reproduction commands are in [COMMANDS.md](COMMANDS.md); attempts are in
[ATTEMPTS.md](ATTEMPTS.md). Native caches reuse only exact content fingerprints.

Limitations: public native tests are not proof of intended semantics. This is a
closed source-local structural proposal family, not whole-program intent analysis,
unknown-flow inference or general method-order reasoning. Aliases/imports/custom
or generic types, macros/attributes, nested/control-flow pairs, reference receivers,
shadowing, intervening effects and custom comparison/dedup methods stay excluded.
Root src/lib.rs/src/main.rs only; no generalized include/module resolution.
The 27-row probe is TRAIN, not an independent generalization score. No DEV,
held-out content, private logs, parent artifacts, model calls or credentials read.
No commits and no owned jobs remain running at handoff.

Remaining coordinator gate: independently review/integrate these uncommitted
changes, then full765 against the frozen 209 verified/556 refused/0 failed result
`fd8400d01239435fbdf43e89ed295ee8d32d69478af953234d2453906df3262c`,
retaining every old repair and zero failed verifications. Worker did not run it.
