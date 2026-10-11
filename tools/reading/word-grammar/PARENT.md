# Coordinator review: word trigram remains rejected

The frozen candidate scored 734/1340 correct preferences, below the required
747. The original byte scorer remains at 707/1340. All 156 regressions and 183
new correct preferences are preserved; no accepted improvement is claimed.

The coordinator archived and verified all 26 original files (709,800 bytes),
checked all 14 frozen source/model/binary identities and 20 input identities,
and replayed both sample and full cost reports. All four replay files matched
byte for byte. The original runtime files were already restored; the owned
worktree was removed only after another archive comparison. Its scope is
inactive and the temporary login link is removed.

An independent reader scanned all 6,227,234 persisted count entries, checked
ordering and all three order totals, and computed Witten–Bell probabilities
without calling the Rust scorer. Twelve scores at fixed development positions
and four new empty/Unicode/whitespace/unknown-token inputs matched saved or Rust
outputs to 1e-10 bits. Every saved decision, finite cost, row order and group
was independently checked. See [parent-independent.json](parent-independent.json)
and [the count audit](parent-count-audit.py). The script uses retained external
model/binary paths, verifies their frozen hashes, and now also explicitly checks
the frozen development input hash; that input was separately verified before
the original audit. No retraining or repeated full candidate inference occurred.

This checks the reported arithmetic and decision, not grammatical truth or
semantic understanding. The tokenizer discards case/whitespace and merges unseen
spellings; these token costs are not byte likelihood. The historical reading
bits-per-byte value and its overlap limitation remain unchanged. The scoreboard
records the reproduced production byte baseline, not the rejected 734 result.

Raw parent evidence:
`/home/gsc/.cache/glassmind/continuous-20261010/coordinator/wordgrammar-review/`.
The first parent hash-check invocation did not resolve removed runtime paths
into the preserved source archive and stopped before scoring. Correcting that
path mapping verified all frozen identities; no inference/report was overwritten.
