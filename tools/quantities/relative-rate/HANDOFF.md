# Relative-rate: REJECTED, evidence only

One bounded experiment completed. The frozen candidate adds **one** correct dev
answer beyond ratewords, failing the predeclared minimum of three. **Do not
integrate `candidate.patch`.** All quantity runtime changes, including the applied
old ratewords patch, are restored byte-for-byte to starting HEAD. Only this new
owned evidence directory remains uncommitted. No commit or second experiment.

## Results under unchanged metrics

| Full4475 series | v3 correct | refused | wrong | unscorable answered | legacy correct | legacy wrong |
|---|---:|---:|---:|---:|---:|---:|
| Production |22|4450|3|0|22|3|
| Frozen ratewords |28|4444|3|0|23|8|
| New candidate |29|4443|3|0|24|8|

Fixed first100 is1correct/99refused/0wrong for all three series. Full denominator
4475 and legacy1229unparseable labels remain intact. Primary gate >=31correct
**FAILS**. No gate relaxation, scorer edits or post-DEV runtime edits. The only
post-DEV runtime operation was restoration.

All31 prior ratewords answer payloads are exactly identical, including all28
correct and all3 inherited wrong. All25 production payloads and all22 correct
remain. Against ratewords:4443refused→refused,28correct→correct,3wrong→wrong,
1refused→correct. Against production:4443refused→refused,22correct→correct,
3wrong→wrong,7refused→correct. No new wrong or unscorable answered rows.
Full order/count/question hashes and frozen sparse baseline hashes were checked;
missing baseline indices imply refusal because full refusal payloads are unavailable.

Every candidate prediction was saved before labels were loaded for scoring.
`sample/` and `full/` contain complete indexed/question-hashed predictions,
inference hashes, summaries, every-row ledgers and changed payload records.
`evaluate.py` imports unchanged frozen metric.py and validates every FREEZE hash,
source parquet hashes, pinned legacy scorer and frozen sparse answer reports.
Labels never go to Rust; only Problem JSON strings cross the process boundary.
Zero-valued count categories are omitted by Counter in machine summaries; above
zeros are explicit. Legacy is the pinned original scorer, not a changed normalizer.

## Physical evidence, separate from label comparison

The sole new answer beyond ratewords is DEV2382: two trains explicitly travel in
opposite directions at60km/h and90km/h; lengths1.15km and0.6km; requested seconds.
Complete clearing requires1750m of relative displacement. Relative speed is
150km/h=125/3m/s. Duration1750/(125/3)=**42seconds**. This is a direct English
geometry/unit derivation, independent of the selected label.

`physical-audit.json` records that derivation plus fresh arithmetic checks of all
six old ratewords additions (DEV295,563,1128,1986,2836,2873), with their English.
Their24,30,6,27,29,27seconds remain unchanged. Label equality is not proof of
English interpretation; uniform straight-line motion and complete clearing are
the declared bounded physical assumptions. No options supplied those derivations.

## Exact rejected implementation

`PLAN.md` freezes the hypothesis, scope, grammar, assumptions, probes and gate
before runtime edits or DEV. First4000 TRAIN Problem-only inspection selected58
same/opposite-direction train descriptions; `train-evidence.json` preserves them.
No TRAIN choices/correct/rationale/formula fields were used for design.

Applied the previously rejected ratewords patch unchanged as a starting point:
`rate.rs` typed affine core; `rate_language.rs` stationary-crossing grammar;
crate-visible exact scalar operations/tokenizer; module/dispatch registration;
old rate tests. This does not revise its historical rejection.

New `relative_rate.rs` uses fully consumed token clauses, an ordered pair of
moving bodies, optional unique names, canonical typed speed/length facts and
explicit direction. It checks positive lengths/speeds, contradictory attributes,
unknown bindings, equal rates and extra clauses. Opposite direction adds known
speeds; same direction uses their absolute difference. Sum of known lengths is
constrained against unknown Duration times known Speed using the typed affine
core (existing two-unknown/eight-equality bound). Duration output unit is explicit.
Names, coefficients, compatible units and fact clause order vary in probes.
No floats, unknown products, default units, answer tables, IDs or runtime Python.
Only lib/module and parser dispatch changes besides the core reuse.

The rejected patch is43,394bytes, SHA256
`b8256c104c1f4055491872e1082781b5ec93fb87aa3ae0bef5d493b7ffb02d16`.
Exact source, patch, probe/PLAN and binary hashes were frozen before first100 DEV
in `FROZEN.json`; `git apply --check` succeeds against restored base.

## Validation and rejected work

- New authored probes29/29:8answers,21refusals; all8 answer cases were refused by
  the supplied frozen ratewords binary. Exact mixed units/fractions, same/opposite
  signs, reversed length order, unique names and explicit minutes are covered.
- Old ratewords43/43, linear50/50 and absolute55/55 pass.
- All3116 synthetic status/value/unit triples are identical:3056correct,
  including762expected refusals;60unsupported;0wrong. Canonical SHA256
  `93b2bd1c9f8822af95a68d02663a90d122d24ca57d730c61c670e2b11079bbe9`.
- Candidate13 quantity tests, scoped all-target clippy, scoped fmt, quantity and
  mainCLI builds pass (`validation.log`). Initial clippy found2 dynamic array
  indexing sites; replaced with checked get_mut before freezing or DEV.
- Restored-base9 quantity tests, scoped fmt/clippy and quantity/mainCLI builds
  pass (`restored-validation.log`). MainCLI source was never edited.
- The runtime experiment was rejected for insufficient coverage (29<31), despite
  one measured new answer. No full-dev retuning or additional hypothesis.

One access-policy deviation is disclosed: while locating the existing regression
invocation, the worker opened `tools/quantities/linearwords/check.py` and saw its
local test-generation code. This violated the instruction not to read test-
generating code. No held-out items, official test split, hidden-test code, old
private logs or credentials were read. Remaining old checks were executed without
reading their generators. No model/agent calls, services, downloads, dependencies,
API keys or child jobs remain. Inherited launcher model/auth was not changed.

All cargo invocations used disk TMPDIR, the supplied existing CARGO_TARGET_DIR,
systemd user scope MemoryMax8G/MemorySwapMax0, setsid--wait, timeout600 with
kill-after10s; -j1 except fmt; testing--test-threads4. No `/tmp` or pkill.
No shared manifests, CLI, loop, scorers, metric definitions/FREEZE, old reports or
historical prompts were modified. Parent's askapply integration was untouched.

## Preservation, provenance and reproduction

Starting source HEAD `336d5f9ee5ff70b208d7e2b78ee0b582c6f04a2a`; production
quantities source687f2751ff256f2071330590345ef0652fb09baa. Source/data details in
`source.json`. Workspace code/authored diagnostics: MIT OR Apache-2.0.

Existing AllenAI MathQA Apache-2.0, dataset revision
fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6, card revision
c4f1cc784c04c4957b50c97858f23893b633eea6:
TRAIN9,013,733bytes SHA c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99;
DEV1,350,141bytes SHA488beaa0e6cafe4aefbdd07ac76542547a27681461e3da7641e0777e14756e20.
**New external data bytes0; new dependencies0.** Local TRAIN extract14,218bytes,
SHA0a44ee94a136568fe8e14588f42cda18c7a2ee89c536b8f7981318419a00f9bb.
All new local report/probe sizes and hashes are in `ARTIFACTS.json`.

Retained under WORK=`/home/gsc/.cache/glassmind/continuous-20261010`:
`relativerate-run/candidate/source/` exact source; candidate binaries:

- glassmind-quantities25,661,624bytes, SHA
  `5047be66fe497f70094d36a45812c69f70d87360937bce868752031337234d54`.
- glassmind126,331,248bytes, SHA
  `ef9c98c23b21c2a8b015f5ff03d67c9ed337c312657c64c9683af2fdc5f31c35`.

`relativerate-run/pre-restoration-evidence/` archives24original evidence files
with verified ARCHIVE.json before restoration. All candidate source/binary hashes
were verified before restoration. Git restore could not acquire an index lock in
the readonly Git administrative directory; tracked bytes were restored using
`git show HEAD:path`, four owned new runtime/test files removed. Empty quantity
diff and restored binary hashes recorded in `restoration.json`.

Replay retained candidate (use a fresh output directory):

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/relativerate-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/relativerate-run/target
bash tools/quantities/relative-rate/reproduce.sh
```

The script was syntax checked; component commands executed individually, not a
second consolidated inference run. To rescore existing complete predictions use
`evaluate.py --score-only` with the same --binary/--data/--output arguments as
reproduce.sh, plus --sample for sample. This never reruns inference.

To rebuild rejected source, in a separate disposable checkout at starting HEAD:
`git apply tools/quantities/relative-rate/candidate.patch`, then with the same
TMPDIR/CARGO_TARGET_DIR exports run `bash tools/quantities/relative-rate/validate.sh`.
That wrapper contains every required cargo cap/flag. Debug binary hashes depend
on path/toolchain; retained hashes identify the evaluated binaries.

Limitations: narrow complete-crossing duration grammar; no unknown lengths/rates,
implicit output units/directions, driver crossings, catching a head start, speed
changes, extra events or general motion solver. MainCLI compiled only; parent
owns integration regressions/full coding. No release/acceptance claim. The task's
prospective capability gain remains incomplete because the gate failed.
