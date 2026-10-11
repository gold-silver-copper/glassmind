1. Baseline full TRAIN: 7 verified / 20 refused; exact expected selected-input SHA.
2. Initial world-rust legacy suite passed (49 existing tests across unit/integration).
3. First authored native run failed at custom_deref: initial fixture did not compile
   because &Items is not IntoIterator and its tuple constructor field was private.
   Retained contracts.json is the original frozen fixture. contracts-native.json
   fixes ONLY fixture compilability: public constructor field, explicit &**xs loop.
   The failing assertion and refusal expectation are unchanged. First 19 native
   contracts before this fixture passed; full rerun required. This is a fixture
   construction failure, not evidence for a successful repair.
4. Static review rejected ::core::primitive::<type>::MIN/MAX qualification: a
   dependency named core could shadow that crate path. Final replacement uses
   <resolved_primitive>::MIN/MAX, which relies only on the already-proven primitive
   type, with local/root aliases excluded. Original planned expected patches remain
   in contracts.json; revised exact patches in contracts-native.json. Assertions
   and test inputs remain identical. This narrows an environmental assumption.
