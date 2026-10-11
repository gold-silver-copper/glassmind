# Prospective incoming-edge experiment

Preregistered 2026-10-11 UTC, before code, fetch, or DEV access. One bounded batch
on source b569ac0eb0bab5a8b9d39cda904c816e77c08e63. Only new files under
 tools/facts/inverse-coverage are owned. Raw graphs and evidence stay in
/home/gsc/.cache/glassmind/continuous-20261010/inversefacts-run/tmp/inverse-coverage.

Hypothesis: independently fetched incoming nondeprecated item-valued Wikidata
statements for TRAIN-selected target/property pairs address missing actual graph
coverage while leaving the existing 999-entity alias linker unchanged.
Baseline answer-only 47 right / 2712 refused / 62 wrong; strict annotated entity +
relation + answer 2 right / 2712 refused / 107 wrong, total 2821. Earlier linker
experiments were rejected. Historical validation-seeded 500 is not comparable.

Freeze selection JSON before implementation/fetch/DEV: existing public TRAIN only;
first floor(.9*N) rows, excluding zero-based i%10==0. Read only first two TSV fields
(subject QID, inverse relation Rdigits); require subject in frozen999 index.
Count pairs, sort by descending frequency then SHA256(subject+'\t'+relation),
select top100 without backfill. Objects/questions/dev labels cannot select pairs
or generate graph claims. Hash TRAIN bytes without decoding unused fields.
Hash-verify reused base, selection, provenance, raw responses, bundle and model.

Consult official WDQS usage/policy/docs before requests. Primary endpoint only,
sequential identifying UA/gzip/timeouts, >=5s request-start spacing, <=110 requests,
<=600s total network phase, <=20MiB new decompressed bytes. Each pair query uses
actual p:P/ps:P statements, normal/preferred ranks (not truthy-only), item QIDs,
DISTINCT subject, ORDER BY subject LIMIT101. 101 results => TRUNCATED. Missing or
failed responses never imply empty knowledge. Stop first blocked queries and
repeated limits, honor Retry-After; no alternate endpoints or bypass. Keep raw
responses, exact query/time/hash/errors and revision availability. CC0 data.
A complete <101 response is only endpoint query snapshot coverage, not global
truth across temporal/qualified questions.

Append independently witnessed property-target edges only. New source entities
have EMPTY aliases. Existing aliases and existing claims must remain byte-for-byte
represented unchanged; append new properties/values without rewriting old data.
No TRAIN answer claims, alias changes, runtime/schema/scorer/model changes.

Freeze merged candidate/model/scripts before DEV. Fixed first200 then full2821,
unchanged official scorer and tools/facts/joint-audit.py, question-only runtime
inputs, annotations kept offline. Acceptance: joint correct >=7, both old joint
correct retained, no new joint wrong from old correct/refused, answer-only and
joint precision not worse; exact alias links unchanged; standalone/talk answer IDs
identical and relocated opt-in bundle works. Report all answer-only/joint
transitions, disagreements/new wrong. No post-DEV scope/rule/threshold tuning.
Reject => preserve raw evidence, no deployed/default new bundle.

If endpoint blocked: stop fetch, retain concrete blocker evidence, run meaningful
synthetic public-fixture tests for missing/truncated queries, independent edge
validation, exact base preservation, aliasless nodes, determinism and provenance
hashes. Mark experiment INCOMPLETE and claim no candidate scores.

No agents/LLM calls/keys/credentials/held-out/private logs/commits/services. No cargo
unless needed; every invocation requires TMPDIR and systemd-run --user --scope
-pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600,
-j1 (except fmt), test threads4. Existing CARGO_TARGET_DIR only. No /tmp storage.
