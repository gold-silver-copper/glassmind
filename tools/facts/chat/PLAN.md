# Preregistration — 2026-10-11
Base 05e80930ef2b06e8e054ece3ab4cc69666d19b98. One bounded integration batch.
Hypothesis: optional immutable facts context, routed after typed memory/calculation
and established story context, exposes exactly the frozen accepted fact IDs in
fresh sessions, while clearing memory correction focus prevents cross-domain edits.
Display-only English labels improve readability without changing predictions.
Baseline: standalone frozen facts first200 12 right/188 refused/0 wrong;
full2821 256 right/2545 refused/20 wrong. Current talk has no facts route.
Acceptance: first200 before full2821; every standalone accepted ID set preserved;
default and opt-in memory41, calculations27, mixedstory47 retained; additional
mixed checks include unknown facts, explicit memory override, name/preference
correction focus after fact answer/refusal, reset/restart/legacy, partial config
and mismatched assets. Runtime gets only question text. No new factual coverage.
Closed scope: 500 validation-subject-seeded entities, not open-world retrieval.
No scorer/linker/weights/index changes. No held-out access or agents/LLMs.
Labels only: existing index subject/object IDs; cached English first; official
API at most50/request, sequential >=1s, <=12000IDs, <=10MiB new raw bytes,
<=600s wall, retry-after/maxlag/backoff, stop repeated failures. All new data
in own persistent TMPDIR. Source/raw hashes and coverage recorded.
Rejected experiments and measured results will be in HANDOFF.md.
