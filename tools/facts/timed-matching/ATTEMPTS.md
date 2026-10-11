# Preserved experiment history

- Before implementation: rejected truncating `Store::match_question` then date
  filtering. It violates complete totals and loses late qualifying subjects.
- Baseline: existing lib/matching/temporal tests passed; release question-only
  proof compiled from the original source before production edits. Full200/2821
  baseline predictions, native scores and six old CLI replies retained.
- First candidate: fixed8 then all51 frozen authored cases passed. All existing
  lib/matching/temporal tests, clippy and release build passed. `real-first/`
  retains original455-query outputs, independent expectations and raw checks.
  `current/` legacy replay passed byte-for-byte with baseline.
- Review refinement: retain TemporalIndex.store as a private field and expose
  only a crate-private immutable borrow; bound typed condition count before any
  scan. No date, interval, alias, graph, or display policy changed. Rebuilt and
  reran affected tests and full real query denominator; final `real/` is retained
  separately from `real-first/`.
- Supplemental tests add typed reason assertions, alias/ID duplicate provenance,
  reserved separator alias, record-order invariance and distinct citizenships.
  These are supplemental, not claimed as part of the preregistered51.
- A temporal proof binary was copied after the first candidate build; it is
  explicitly stored as `evidence/temporal-proof-first-candidate`, NOT baseline.
  Old temporal CLI replay uses the coordinator-provided original baseline binary.
- No contract, implementation, clippy or scoring failure required weakening an
  assertion. All original logs and command outputs are kept in RUN/evidence.
- Final per-invocation CLI replay completed within its600-second scope:56 old
  temporal cases,50 new matching cases, and all2821 ask outputs byte-identical.
  Both main modes also completed200/full2821 byte replay. No failed gates remain.
