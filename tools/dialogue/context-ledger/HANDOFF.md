# Handoff — REJECTED, completed bounded experiment

One representation and one 8-role/8-rank averaged-perceptron schedule were fitted.
No CAL member passed. No candidate DEV inference, second representation, threshold
retuning, runtime integration, external action, commit, agent or model call occurred.
No accepted accuracy improvement is claimed. The unchanged production baseline is
still the supplied 610/1119 joint USER turns and 716/1218 frames on 179 DEV dialogues.

## Exact changes

- Added `crates/dialogue/examples/context-ledger.rs`, an isolated offline executable.
- Added `tools/dialogue/context-ledger/native/context_ledger_experiment.rs`, copied
  from the explicitly authorized native source and modified to deserialize ONLY
  the old frozen BIO boundary into a fresh model. Both old role/rank fields are
  skipped; new role/rank weights start at zero.
- Source labels use actual predicted SYSTEM spans, with NONE for absent/unaligned,
  ambiguous or out-of-label exact supervision. Exact duplicate annotations are
  deduplicated. FIT frequency >=20, top127 roles plus NONE; all negative rows retained.
- Added masked preceding actual USER text, masked local SYSTEM context and baseline
  predicted state features. No gold features, owner/router filter, service pair
  lexical rules, entity identity features, pretrained models or downloads.
- Added preparation, independent CAL audit, full linear-score reconstruction,
  first10 FIT independent boundary decoder parity, reproduction wrapper and reports.
- NONE never enters the role ledger. A separate bounded raw-span mask history is
  retained solely to mask available entity strings. Both histories cap128/32 SYSTEM;
  utterances4096bytes/128tokens, extracted spans16tokens, model16MiB.
- No tracked production dialogue file was changed (`git diff --exit-code --
  crates/dialogue` passed; the example is a NEW untracked file). No old model,
  README, manifest, scorer, main CLI, loop or historical report was edited.
  Pre-existing parent-directory deletions in this isolated worktree were untouched.

## Frozen CAL gate and complete result

The pre-implementation plan is `EXPERIMENT.md`. All 3174 TRAIN dialogues and all
34022 USER /34022 SYSTEM turns were retained. SHA256(id) mod5 split:2532 FIT/642 CAL.
CAL denominators are fixed at6830 USER turns and2505 supported-service gold frames.
The5137 additional unselected-service gold frames are retained in the dataset,
not scored against a model that has no corresponding output frame. USER turns with
no supported gold frame remain in the denominator; gold route is intersected with
supported schema services. This is a TRAIN diagnostic convention, not a DEV score.
All members use exactly this same convention and the unchanged upstream frame
metric. Baseline CAL:5803 joint and1478 strict frames;1073 individual erroneous
scored slot states are explicitly preserved in `artifacts/baseline-slot-errors.json`.

Required:>=30 new correct fills, zero newly wrong scored assertions, no lost
baseline-correct joint or frame outcomes. All changed scored slot-turns count,
including propagation. Baseline-supplied values take precedence and clear a held
entry; held overlay persists otherwise. Held/inactive changes receive no correctness
credit. Full candidate prediction files, original gold/state/input data, every
binding event and every changed slot-turn are retained for every grid member.

| Source margin | Rank vs NONE | Different-value gap | New correct | New wrong | Lost joint | Lost frames | Held/unscored |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 2 | 2 | 510 | 65 | 1 | 1 | 123 |
| 0 | 2 | 4 | 509 | 61 | 1 | 1 | 117 |
| 0 | 4 | 2 | 510 | 65 | 1 | 1 | 123 |
| 0 | 4 | 4 | 509 | 61 | 1 | 1 | 117 |
| 0 | 8 | 2 | 508 | 65 | 1 | 1 | 123 |
| 0 | 8 | 4 | 507 | 61 | 1 | 1 | 117 |
| 0 | 16 | 2 | 489 | 55 | 1 | 1 | 105 |
| 0 | 16 | 4 | 488 | 52 | 1 | 1 | 99 |
| 2 | 2 | 2 | 507 | 53 | 0 | 0 | 117 |
| 2 | 2 | 4 | 506 | 49 | 0 | 0 | 111 |
| 2 | 4 | 2 | 507 | 53 | 0 | 0 | 117 |
| 2 | 4 | 4 | 506 | 49 | 0 | 0 | 111 |
| 2 | 8 | 2 | 505 | 53 | 0 | 0 | 117 |
| 2 | 8 | 4 | 504 | 49 | 0 | 0 | 111 |
| 2 | 16 | 2 | 484 | 48 | 0 | 0 | 99 |
| 2 | 16 | 4 | 483 | 45 | 0 | 0 | 93 |

Every member is rejected:45–65 new wrong assertions. Some members additionally
lose one previously correct joint/frame. No member was selected, including those
that preserve all prior correct frames. Held/unscored changes range93–123 and are
not counted as correct fills. No further experiments were attempted.

## Source supervision and reconstruction evidence

34223 actual frozen-boundary source spans;27241 FIT and6982 CAL.128 competing
classes include NONE. FIT has1912 NONE targets (1597 absent/unaligned,315 outside
label vocabulary); CAL has663 (562 absent/unaligned,101 outside vocabulary).
No ambiguous multiple-role exact annotations occurred in this dataset, but their
NONE behavior is explicitly tested. FIT predicts2734 NONE and CAL predicts876.
CAL source role-or-NONE agreement with exact-span supervision:5138/6982;
4742 positive predictions match annotated roles,1364 positive predictions do not.
This remains insufficient for the zero-wrong reference gate. Missing source
annotations do not prove semantic absence, so these are supervision-agreement
counts rather than complete semantic precision/recall.

All7086 opportunities are retained:5654 FIT/1432 CAL, including1324 empty and3955
negative opportunities across both groups. All46789 candidates remain in raw
proposals. Model role owners cover25 services; valid sources were not restricted
to the three routed services. All22912 native CAL decisions (including refusals)
were independently reproduced. Every other-valued candidate competes regardless
of its source margin. NONE has fixed rank score0; source NONE is a trained class.

`reconstruction.py` independently recomputed all4380544 source class scores and
all46789 rank scores, verified exact original source bytes/UTF-8 span endpoints,
preceding actual USER utterances, feature hashes, FIT frequency ordering/cap and
non-NONE candidate provenance. The separate Python boundary decoder matched all
69 extracted spans on104 SYSTEM turns of the fixed first10 FIT dialogues.
Original baseline inputs/prediction USER indices match all3174 sorted conversations.
Original frozen boundary numerical weights match exactly; both package manifests
were independently SHA256/size verified.

## Validation, failures and limitations

PASS:7 native tests (UTF-8 provenance, deterministic capacity/age eviction, reset,
strict annotation rejection, finite weights, source masking/actual USER lexical
context, NONE supervision/ledger exclusion, exhaustive two-candidate ambiguity,
low-margin competitor veto and same-value alternatives). PASS: final formatting,
all-target affected-package clippy with -D warnings, release example and complete
package release. Logs are under `artifacts/`.

Preserved failures: initial clippy rejected an eight-argument helper. Its history
arguments were grouped with no feature change, then all checks passed. Initial CAL
audit asserted every TRAIN gold frame belonged to selected schema; it stopped
before candidate metrics. The original script and failure log are retained. The
fix limits frame scoring to available schema outputs while keeping every USER
turn and all raw gold; no upstream metric code, fitting data, fitted weights,
representation, schedule or grid was changed. This scoring convention is explicit
above. It does not rescue any candidate: every member fails on new wrong assertions.

Old baseline weights saw all TRAIN, and prior diagnostics inspected FIT and CAL.
This is empirical new-component validation, never unseen calibration. Annotation
absence creates potentially false-negative NONE supervision. Finite label cap and
hash collisions remain representation limitations. The source owner count does
not imply correct role inference. Rank supervision uses offline exact case-folded
value alignment; the unchanged strict noncategorical evaluation is case-sensitive.

No DEV inference was permitted after gate failure; consequently no DEV gain,
reverse-conversation runtime test or45-dialogue legacy candidate replay is claimed.
No runtime/bundle integration was attempted and no production rollback was needed.
The runtime reset tests concern the isolated ledger only. Rejection/correction is
learned from USER lexical evidence, with no manually added veto: CAL wrong assertions
show this is insufficient for deployment. No action executor or persistent user
memory exists in this experiment.

## Source/version/license/bytes

No newly downloaded data (0bytes). Source SGD revision
`e852981ae34990f4358979625854259302feaa78`, CC-BY-SA-4.0. Derived annotation artifacts
retain CC-BY-SA-4.0 (`DATA-LICENSE.txt`). Code MIT OR Apache-2.0; unchanged upstream
metric Apache-2.0. Source checkout `d84ba870465562456fb389805d3703ee12d41f23`.
Supplied read-only `contextledger-source`:3 manifest payloads/20369681bytes.
Supplied read-only `entityledger-source`:11 manifest payloads/120818396bytes.
Their exact per-file source IDs, hashes and sizes are in `artifacts/source-verification.json`.
Rejected source model SHA256:
`661c80b75a808994427a53591c651cb3d1ddc2fc1e0624503d6b70e4ca09b0e0`.
Approved unchanged v2 source model SHA256:
`fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88`.

| Derived artifact | Bytes | SHA256 |
|---|---:|---|
| `dataset.json` | 29431429 | `e24fc279a53a4f8b78062d2dfc39a45e8df69fc14de608faae52549f8a731faa` |
| `model.json` | 4081031 | `bfaa28ad936fee501145484db17ef4fe7d1676c71c8ef41dc23822b10a994385` |
| `context-ledger` | 11924568 | `e0218edb68e40a927807baf74239ec4d1e04007bff0ca65a543edd7f64fdc966` |
| `source-spans.jsonl` | 148845810 | `ab5cecdf683e376204b487506320f4a74993b54ceea57fa17ece1725bd4fc1f4` |
| `proposals.jsonl` | 96660872 | `31a250e06a955ffc99c7a5eb51ff3ecce793f7976aee73a2d35612ffe98799b0` |
| `decisions.jsonl` | 2349914 | `b973cb993a59b42377c36dc5bd31cdeef1a12fb63ac56f95193a6cab10a9fdc5` |

## Reproduction and archive

Run from the supplied worktree with its existing `CARGO_TARGET_DIR` and on-disk
`TMPDIR`. The complete bounded fitting/check commands are in `run-checks.sh` and
`commands.log`; every Cargo invocation is capped8GiB/0swap, `setsid --wait`,
`timeout --kill-after=10s 600`, `-j1` except fmt, test threads4. The outer scope was
also independently verified capped8GiB/0swap. Use the mounted read-only metrics
interpreter. No credentials are required or read.

```sh
python3 tools/dialogue/context-ledger/prepare.py
bash tools/dialogue/context-ledger/run-checks.sh
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 /home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/metrics-venv/bin/python tools/dialogue/context-ledger/audit.py
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 /home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/metrics-venv/bin/python tools/dialogue/context-ledger/reconstruct.py
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout --kill-after=10s 600 /home/gsc/.cache/glassmind/continuous-20261010/dialogue-run/tmp/metrics-venv/bin/python tools/dialogue/context-ledger/boundary-parity.py
```

The historical mechanical-construction script under `originals/` is retained for
provenance, not to be rerun over final source. `originals/` also preserves the
original authorized native code and the initially failing audit implementation.
All final source, new example, complete raw models/traces/predictions/counts/golds,
logs/failures, license, source patch and this handoff are preserved in
`archive.tar.gz`; `archive-manifest.json` records every member hash/size and
`archive-verification.json` confirms byte-for-byte verification from the archive.
Nothing was committed. Coordinator may retain this as a rejected offline
experiment; no runtime change is proposed. No owned job remains running at handoff.
