# Token endpoints — REJECTED before DEV

One bounded experiment completed at base **d59cb121d0c6e0276141283c872e9cd25cb3275e**.
**No rule qualifies. Do not deploy the candidate keys. No capability improvement
is claimed.** There was no second representation, loosened gate, threshold
search, refit, runtime extension implementation, or candidate DEV evaluation.

Every tracked `crates/dialogue` file is byte-identical to HEAD. The temporary
offline diagnostic binary source was preserved, then removed from the crate.
Only new `tools/dialogue/token-endpoints/` remains uncommitted. No commits,
shared manifests, CLI, scorers, historical prompts, existing adapters/models/
reports, other tracks, or README files changed.

## Frozen hypothesis and measured rejection

[PLAN.md](PLAN.md) froze the hypothesis, representation, algorithm, independent
contrasts and acceptance before runtime edits or evaluation. A TRAIN-supported
one/two-token contiguous extension of an existing reference-copy proposal might
repair source truncation without changing source states, routing, intent, offers
or punctuation rules. Most-recent whole exact occurrence only; no older fallback.

One shape key uses the last two source-token shapes, next two token shapes and
two intervening gaps. Upper/lower/uncased-letter/digit runs have length capped3;
punctuation is literal. Bounds:4096 UTF8 bytes/256 tokens per utterance,12 source
tokens,64 bytes per shape,3 gap symbols,extension1or2. No entity names, service
IDs, lexical dictionaries or DEV values in keys. Existing punctuation restoration
has priority. Unknown/conflicting keys preserve the original copy.

Fixed selected TRAIN:3174 conversations/34022 USER turns. SHA256(dialogue_id)
integer mod5==0 gives2532 FIT/642 CAL. All TRAIN-schema noncategorical input
annotations on existing USER/SYSTEM turns were allowed. FIT gold prefixes
generated1583 candidate keys/1635 key-length hypotheses from26847 rows;
158 unaligned spans and1057 nonlatest prefixes were excluded from discovery.
All candidate conversation IDs were independently checked to be FIT-only.

The diagnostic then replayed **all3174 actual conversations**, using only
speaker/utterance input and frozen approved v2. It reconstructed the existing
reference call site from prior predicted states and unchanged per-service
advance, then asserted that every reconstructed reference equals the actual
value inserted by `MultiModel::advance`. No gold prefix constructs CAL inputs.

| Actual frozen TRAIN evidence | FIT | CAL |
|---|---:|---:|
| Existing reference proposals |134|31|
| Generated endpoint hypotheses, including conflicting lengths |46|4|
| Endpoint matches |0|0|
| Endpoint mismatches |46|4|
| Of mismatches, missing source annotation |1|1|
| Qualifying rules |0|0|

All165 references are retained in [audit.jsonl](reports/audit.jsonl), including
82 without bounded token context,35 unknown keys,44 single-length candidate
rows,3 conflicting-key rows (both lengths audited), and1 unchanged punctuation
correction. The47 candidate-bearing rows generate50 hypotheses. No failures or
missing annotations were filtered out of the denominator.

Each rule required>=5 distinct actual FIT conversations,>=3 distinct actual
CAL conversations, zero mismatches, and>=30 distinct CAL instances collectively.
There are **zero qualifying rules and zero qualifying CAL instances**. Full
per-rule rejection reasons are in [gate.json](reports/gate.json). No accepted
runtime model was constructed.

A post-rejection diagnostic explains the evidence gap: excluding the one
punctuation-restored copy,129 FIT+29 CAL original values already end exactly at
their matching source annotation;4 FIT+2 CAL have no matching source annotation.
There is **no annotated truncation among these actual reference proposals**.
This is evidence about the frozen TRAIN replay, not a general claim that source
truncation does not occur. Artificial gold-prefix support does not substitute
for actual predicted-source validation. See [source-endpoints.json](reports/source-endpoints.json).

Frozen classifiers already used all TRAIN including CAL. This was new-rule
empirical validation, **not independent probability calibration**. Source
provenance alone is not referent correctness; all-O semantic freshness remains
unsolved. Nothing here supports changing that separate mechanism.

## Exact changes and validation

- `experiment.py`: bounded FIT discovery and actual-proposal endpoint audit;
  all candidates, rule-level failures and raw proposal rows retained.
- `diagnostic.rs` / `diagnostic.patch`: exact temporary Rust diagnostic source,
  archived outside the crate after rejection. The patch applies cleanly to base.
  It adds a new offline binary only; no production function was edited.
- `prepare.py`, `test_experiment.py`, `reproduce.sh`: utterance-only TRAIN input,
  five authored contrasts and bounded reproduction. Reports retain split IDs,
  original logs, source/model freezes and all165 raw frozen proposals.

Release diagnostic build passes. All165 reference reconstruction assertions
pass. Five authored tests pass: new multiword names,1/2-token suffixes, Unicode
byte slices, punctuation boundaries, whole/case matching, repeated/latest and
correction occurrences, no older fallback, input bounds, complete values,
missing/conflicting annotations. These validate the offline mechanism, not
trained-rule accuracy. All eight unchanged Rust library tests pass, including
annotation rejection, source-state persistence, reset and ambiguity behavior.
A separate annotation-bearing diagnostic input exits1 with `unknown field
frames`, before creating output. Reproduction preparation is byte-identical;
shell syntax and diff checks pass. No full workspace suite was run.

**DEV acceptance was not reached.** First10/full179 candidate runs using
evaluate.py/compare.py would violate the failed TRAIN gate; neither was run.
The supplied baseline remains610right/459wrong/50refused over1119 USER turns,
716/1218 frames,179 conversations/54 multi-service. No new denominator or metric.
Optional-absent v2, reversed DEV and legacy45 replays were not repeated, because
no production source/model change survives. Prior legacy202/257 is inherited
evidence, not a new measurement. Version/schema extension was never implemented;
existing version validation stays untouched.

[inherited-audit.json](reports/inherited-audit.json) carries the explicit seven
still-wrong baseline slot-turns:17_00022 turns6/8/10 and17_00038 turns6/8/10/12.
All remain unchanged; no repaired assertion is claimed. It separately preserves
four previously corrected punctuation assertions and six held-unscored states;
held states receive no accuracy credit. Changed assertions/new wrong assertions
are empty by source/model identity, **not by a new candidate DEV replay**.

## Reproduction and retained evidence

From a clean checkout at the base above, with provided baseline and allowed SGD
mounts, run:

```bash
export TOKEN_ENDPOINT_RUN=/home/gsc/.cache/glassmind/continuous-20261010/tokenendpoints-run
export CARGO_TARGET_DIR="$TOKEN_ENDPOINT_RUN/target"
bash tools/dialogue/token-endpoints/reproduce.sh
```

The script temporarily adds the archived diagnostic source, removes it on exit,
rebuilds, replays TRAIN, and compares discovery/audit/gate/raw-trace bytes.
It sets own-run TMPDIR/GLASSMIND_SCRATCH and scopes native jobs with8GiB/0swap,
`setsid --wait timeout --kill-after=10s 600`; cargo uses `-j1`. Constituent
commands ran in this worker; the consolidated script was syntax checked,
**not rerun end-to-end**. Original raw input and evaluated binary remain in
`$TOKEN_ENDPOINT_RUN/scratch/`. No candidate/source cleanup preceded preservation.

The additional validation command was:

```bash
TMPDIR="$TOKEN_ENDPOINT_RUN/tmp" GLASSMIND_SCRATCH="$TOKEN_ENDPOINT_RUN/scratch" \
CARGO_TARGET_DIR="$TOKEN_ENDPOINT_RUN/target" \
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
setsid --wait timeout --kill-after=10s 600 \
cargo test --offline -p glassmind-dialogue --lib -j1 -- --test-threads 4
```

| Artifact | Bytes | SHA256 |
|---|---:|---|
| Frozen approved v2 model |32174720|`fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88`|
| Supplied full DEV predictions |405228|`8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d`|
| Supplied first10 predictions |34694|`91afdda73ca8e79afcc1a51d204fc15d1c8fb26efbfbec5b4e69b9cbeafd6761`|
| Actual TRAIN reference trace |24545|`567b67eef3158118953f65bb7678acabebb5899ab776f0306615fb039b14e96b`|
| Complete proposal audit |61416|`32b2d2f07ef98ca96a1865c6b68f955631334e83ccf2a31bd8c82178a92cb895`|
| Rejected candidate keys |665890|`b5facf38efc56b117f45a6380ab2896e4880573f8573ec527fd9ee3dd3f2f20e`|
| Gate report |799941|`6fa2bfc6091c43a8ddb328a2d1b2cbfbd4ced9f2af451e1572936ceefc0e2683`|
| Diagnostic source patch |3953|`d5d5d919861fd4441feb87e776d9a6b72cc32a1cc9dce1132a328d5e252202a9`|
| Evaluated diagnostic binary |10825112|`d9f7446991852139f91c4c1e1d0e8bdb3aacdb4e910fd9fd46c2d13d2dd1c6f0`|

All tracked crate files match HEAD; canonical sorted file-identity JSON digest
`4ff78d2a0e8b34a334b1e0a2f0f9f4cd8cba161f1de95529e3049624a223bfa7` is detailed in
[restored-source.json](reports/restored-source.json). Model, prediction, scorer,
input, binary and owned-file identities are in [artifacts.json](reports/artifacts.json).

## Data and limitations

New external data/downloads: **0 bytes**. Reused SGD revision
`e852981ae34990f4358979625854259302feaa78`, **CC-BY-SA-4.0**:
selected TRAIN45291003bytes, full TRAIN schema75764bytes, selected TRAIN schema
3707bytes, license20139bytes. [data-provenance.json](reports/data-provenance.json)
records their exact hashes. Derived data-bearing reports retain CC-BY-SA-4.0.
No dependencies, pretrained models, runtime LLMs or answer lookup tables added.

Outer scope verified8GiB/0swap. Every cargo/native replay used the required
limits and persistent own-run storage; all jobs completed. No agents, services,
new model calls, credentials/API keys, downloads, `/tmp`, hidden/test/held-out
material, old private logs, commits, or surviving owned jobs. The host's model/
ChatGPT login configuration cannot be independently attested by this worker;
no alternate model/authentication was invoked. Parent independent review is
required before publishing this negative result; there is no useful candidate
to independently retrain or deploy from this batch.
