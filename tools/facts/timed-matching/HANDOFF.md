# Timed shared-subject matching handoff

Base: `a716079f6713e190fe543d7ecf6599ce7c14369d`, uncommitted changes only.
Ownership: `crates/facts` and new `tools/facts/timed-matching/` files only.
Coordinator retains main conversation route, shared manifests and scoreboard.

Implemented `TemporalIndex::{match_at_str, match_at, match_constraints_at}` and
public `TemporalMatchResult`, `TemporalMatchRefusalReason`, `TemporalSubjectMatch`.
The standalone facts binary accepts:

```
glassmind-facts match-at --index INDEX --temporal TEMPORAL --date YYYY-MM-DD 'Who worked for Q2 and was a citizen of Q3?'
```

The typed APIs take the existing `CalendarDate` and either the existing Who
question grammar or `&[MatchConstraint]`. Only P108/P27 are accepted; birth/death
are typed refusals. Shared helpers now perform endpoint resolution and complete
ordered static intersection. Both old verification and new matching use the
same original-claim interval/witness helper. TemporalIndex retains a private
owned Store; no external replacement/mutable accessor exists. No English
question construction, learned fallback, runtime table or data-policy change.
All aliases resolve independently before graph/interval evidence. Every condition
requires one eligible original statement; complete dated count precedes limit10.
The shared result date applies to every witness. Zero counts mean no complete
recorded interval match in this snapshot, never global absence.

Validation completed:

- Original lib/matching/temporal tests passed before production edits. Baseline
  has no match-at command. First8 then all51 preregistered authored contracts
  passed, including full witness equality against independent raw fixtures.
  Supplemental typed reasons, alias-ID provenance, order and distinct citizenship
  tests passed. Existing56 temporal contracts, matching contracts, loader/cap tests
  and library tests passed; fmt/clippy/release facts and main builds passed.
- All308 metadata records independently equal original statements in20 hash-verified
  cached raw responses.20 claims are schema-eligible. Preregistered derivation
  yields24 distinct and18 duplicate condition pairs:455 queries. First8 and all455
  exact independent-reference comparisons pass.47 nonzero results/55 witnesses;
  among324 distinct-condition queries,8 nonzero queries cover4 pairs and2 subjects.
  All queries, expectations, original per-record hashes and results are retained.
- First200 and full2821 question-only legacy prediction bytes and native score
  bytes match baseline. Original ordinary metric remains256 right/2545 refused/
  20 wrong; unchanged joint-audit.py gives252/2545/24. Both default and facts-enabled
  main conversation bytes match baseline on first200 and full2821.
- Old temporal CLI and new matching CLI authored cases pass, with NUL confined to
  library tests because exec arguments cannot contain NUL. Full per-process standalone ask replay also passed all2821 byte comparisons
  (including the first200 checkpoint); RUN/evidence/cli/report.json records56
  old temporal,50 new matching and2821 ask cases.

No new fetched data or trained model. New authored development data:
`contracts.json`,51 cases,209104 bytes, SHA256
`f0c0eb703ef632a0569bf288ff92d1cd18b219815cb0225f0657fd02bb58034b`;
source/version is frozen `freeze.py` plus `freeze.sha256`, under repository
MIT OR Apache-2.0 terms. These synthetic IDs/intervals are test fixtures, not
claims about the corresponding real-world entities.

Reused data: Wikidata CC0,20 cached responses from the existing500-validation-
subject-seeded snapshot. Index287236 bytes, SHA256
`923fda815c0a132e11fded9b79dc8dd42e22faee15dbbc7b9a9a45d426a1c185`;
original temporal268153 bytes, SHA256
`55097953db9b0f041ec4a7c2b5a7513586b1db9f572856cb84aa8610f484ee3d`.
Public SimpleQuestions dev2821 rows/173187 bytes, CC BY3.0, source
`askplatypus/wikidata-simplequestions` revision
`df69267f1db4f85a71325e7e8c06e192bcaf0b0e`.
Unchanged relation model35773124 bytes, SHA256
`db0b175aa5caa78d7f7f6b63dd830e0dc7a2fab39f8259f223239a17db89ada6`.
All raw file bytes/hashes and remaining data identities are in the report manifest.

Reproduction: from this worktree run
`bash tools/facts/timed-matching/reproduce.sh`.
`COMMANDS.md` gives the original sequence; `scoped.sh` enforces every cargo/native
validation's8GiB/0swap scope, setsid --wait,600-second timeout, persistent TMPDIR,
existing target; cargo uses-j1 and tests4threads. Compiler rustc1.99.0
(b940084d7,2026-09-28). Source and binary hashes are frozen in
`RUN/frozen-candidate/manifest.json`, with source copies, binaries, patch and reports.
RUN=`/home/gsc/.cache/glassmind/continuous-20261010/timedmatch-run`.

Limitations: finite historical validation-seeded cache, not independent retrieval
or global/current truth. Only existing conservative Gregorian precision9/10/11
bounded claims qualify. Main date-qualified conjunction conversation route is
intentionally coordinator work; this batch adds standalone/library behavior.
Worst-case latency over maximum-size assets is not benchmarked. Requested worker
configuration is Astra fast/ChatGPT; no model calls, API-key use, credential reads,
network fetches, additional agents or services were performed. Runtime model
selection is controlled by the launcher, not attested by these source artifacts.
No held-out material or parent artifact directories were inspected. No commits.
