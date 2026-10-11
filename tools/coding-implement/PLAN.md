# Prospective experiment (2026-10-11)

Source baseline: 35d3ea80d63791af1625d5019077d6ea2cfa5f26. Coordinator-reported
full coding baseline 174 pass / 591 refused / 0 failed of 765; implement 0/60.
No worker full benchmark run. No private expected solutions, hidden assets,
generators or old private logs are inputs.

Hypothesis: a finite type-directed family of native standard-library pipelines,
selected only by unchanged executed project tests, implements at least three
independent authored functions without using identifier meaning or English
semantics. Existing passing tests are evidence, not a specification proof.

Frozen candidate family: one argument, native integer slices/vectors or &str;
scalar integer/usize/bool or Vec<integer>/String/Vec<String> returns. Enumerate
iteration with optional reverse and optional primitive predicate (positive,
negative, nonzero, even, odd), then type-compatible sum/count/collect/all/any;
strings use chars or split_whitespace with optional reverse and collect/count.
No constants from requests/tests, arbitrary algorithm templates or lookup tables.
At most 32 candidates; 75-second total search deadline, <=35 Cargo checks,
with <=10 seconds per check and 90-second proposal+apply budget. Refuse budget
exhaustion and more than one passing candidate (even if potentially equivalent);
this deliberately avoids claiming equivalence from an interpreter or finite probes.

Binding gate: single dependency-free library package, conventional src/lib.rs,
no production imports, aliases, structs, traits, macros, module graph or custom
attributes; primitive/container signature whitelist. Tests remain opaque and
byte-preserved. One unique free function with whole-body todo!/unimplemented!.
Require native executed failing baseline tests, then passing nonzero tests.
Stale source, disabled verification, compile failure, absent Cargo, no tests,
unsupported target or ambiguous candidates must leave original project bytes.

Freeze authored fixture definitions and hashes before evaluating any candidates.
Fixed sample: sum, reverse and whitespace reversal. Then full authored suite,
including signature-sharing behaviors, renamed/non-English names, conflicting and
insufficient tests, no tests, shadowing, malformed requests, unsupported bodies,
compile failure, stale apply and disabled rules. Acceptance >=3 independently
fully checked positive implementations, zero false-success or refusal mutations.
Parent alone runs unchanged full765 gate: >=177 passes, retain all174 prior
passes, zero failures and no same-source-equivalent regressions. A local pass
is only provisional pending that gate. Reject runtime patch if parent gate fails.

Only owned new modules, their exports, minimal do routing, tools/coding-implement
and docs/coding-implement.md. No shared manifests/dependencies/scorers/old tests.
Every Cargo call uses TMPDIR, existing CARGO_TARGET_DIR, systemd 8GiB/zero swap,
setsid --wait timeout <=600, -j1 except fmt, test threads4. No spawned agents,
LLM calls, credentials, services, commits or independently surviving jobs.
