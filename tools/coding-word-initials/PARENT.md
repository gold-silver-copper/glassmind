# Word-initial compositions pass the frozen gates

Functions with an `&str -> String` signature can now receive a test-verified
implementation that takes the first character of each whitespace-separated word,
optionally keeps only Unicode alphabetic initials, applies Unicode upper/lower
case conversion, and preserves or reverses word order. All 12 compositions join
the six existing string candidates. The generator sees only the signature; it
does not use names, requests, documentation or test outputs to choose a body.

Full public coding DEV passes its frozen gate: **221/765**, **544 refused**,
**zero failures**, retaining **all 220 prior passes**. The sole gain is
`implement/initials` in terse style, also present in the supplied TRAIN material.
No unseen-task generalization is claimed. [Full comparison](full765/comparison.json)
records the unchanged denominator, exact transition and artifact hashes.

Twelve new authored behaviors cover every composition, mixed case, Unicode
uppercase expansion, non-Latin letters, punctuation/numeric/emoji starts, varied
whitespace, empty input and a single word. A separate weak case admits four
passing candidates and correctly refuses without changing source. These 13 new
checks and all 76 existing native checks pass; the latter include 18 cases
repeated with the target environment unset. All 30 retained source outcomes
from the original implementation and word-joining fixtures are byte-identical.
All 17 accepted authored string outputs were independently recompiled, and their
failed baseline, target execution, unique survivor and live recheck verified.

The fixed nine-request TRAIN sample accepts six. Full TRAIN improves from 5/60
to 6/60, preserving all five previous successful source outputs byte-for-byte.
All six accepted final outputs pass independent native recompilation and the
complete verifier-evidence review. The 43 retained source files were hash-checked
before use. No official held-out material, hidden tests or expected patches
were inspected. No new datasets, model workers or teacher calls were used.

All 59 Rust world tests pass, as do workspace formatting, all-target clippy,
release build and the strict four-turn fixture smoke. The only runtime change
is 18 lines in `crates/world-rust/src/implementation.rs`; review of all 196
crate source/manifest/lock files confirms that scope. The 32-candidate and
90-second budgets, exactly-one-survivor rule, target execution, live check and
rollback remain unchanged. Tests establish only the behavior they exercise.

Runtime commit `23e7525`; evaluated binary SHA256
`efb48471993e9fb63181a7f171ffef428ef8938e24f29af20f5192a59d5b101b`.
Full DEV result SHA256
`f9554a39072279faad89ec977f12cacb6f4d6afe9e4e23551b59b4a3b9e476b8`.
RustBench revision `f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, MIT OR Apache-2.0.
[Pre-DEV evidence](pre-dev.json) and [TRAIN comparison](train-comparison.json)
are pinned before the full DEV result. Original artifacts remain under
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/word-initials`.
Known completed-worker usage remains 201,926,299 tokens, cached input counted
once; parent/interrupted usage is excluded. The continuous goal remains active.
