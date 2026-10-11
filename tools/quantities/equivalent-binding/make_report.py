"""Generate compact tables from immutable results; no scoring or model selection."""
import json,os
from pathlib import Path
ROOT=Path.cwd();R=Path(os.environ.get('EQUIV_RUN',str(ROOT.parent/'equivbinding-run/experiment')));O=ROOT/'tools/quantities/equivalent-binding'
x=json.loads((R/'results.json').read_text());v=json.loads((R/'independent-verification.json').read_text())
text='''# Rejected: equivalent-reference operand supervision

Completed one fixed TRAIN-only matched comparison. All eight continuation gates
fail. Arm B regresses CAL free numeric matches143→127 and oracle221→203; singleton
continuation A reaches146 and225. The small equivalent-left improvement does not
establish better complete programs. No production code/model, selection, deployment,
DEV/heldout access, download, agent, LLM call, threshold search or commit occurred.

Only new `tools/quantities/equivalent-binding/` adapters/reports are in the repository.
External evidence is `/home/gsc/.cache/glassmind/continuous-20261010/equivbinding-run/frozen-candidate`.
Base source commit `947e01559f428975592202dc280f6a5abc2de242`.

## Frozen design and actual change

[PLAN.md](PLAN.md) froze the hypothesis, membership policy, schedule, metrics and
gates before fitting. Its SHA256 is
`0c054fddfbd8d42d42a3c81ebc39ec1358236ebfe40f92bde66ca41732b06205`.
Both final arms start from the exact original model and continue SGD for epochs12–17,
rate `0.08/(1+0.2*epoch)`, original deterministic FIT rotation
`(k+epoch*997)%4590`. Rates:0.023529411764705882,0.022222222222222223,
0.021052631578947368,0.02,0.019047619047619046,0.01818181818181818.
No restart, seed, shuffle, early stop, intermediate model selection or tuning.

A trains singleton original operands; B marginalizes valid left labels, then trains
right sets conditioned on each valid left. Right contexts have uniform weight1/|V|
and their gradients are computed at the same pre-update weights and summed. Thus
one step has one total right-head weight in either arm. For legal softmax p and
valid set V, loss is `-log(sum_V p)`, score log-mass derivative is
`1[a in V]*p(a)/sum_V p - p(a)`. SGD adds this derivative times learning rate,
context weight and `1/sqrt(feature_count)` to each active feature weight. Target
logsumexp is stable; original distribution calculation is unchanged. Singleton
update is checked byte-for-byte against the original update.

Equivalent means strict expanded expression-tree equality after sorting only the
two Add/Multiply children. Inputs remain distinct indices even when equal-valued;
constants have exact normalized rational identities. Subtract/Divide remain ordered.
No associativity, cancellation, value evaluation, distributivity or English/answer
information defines labels. Prefix length remains<=6. This is not a complete
algebraic equivalence checker. Every original legal pair is enumerated, gold always
included. FIT7044/15753 steps and CAL1870/4138 steps have two pairs; all other steps
have one. No data step has more than two; duplicate-reference alternatives are also
covered by authored contracts. B uses22797 right contexts per epoch versus15753 A,
with exactly the same total right-head weight15753.

The 8192 hashed features, original number context/rank features, masks, constants,
operations, f32 weights/f64 scoring, strict-> greedy ties, six-step horizon and
Rational implementation are unchanged. No role-ranker features are used. Original
proposal.rs is copied externally byte-for-byte with two appended experiment/test
includes; Rational is byte-identical. Free inference still takes question strings
only. OP weight bytes are identical across original/A/B, SHA256 of little-endian
f32 bytes `9f8445a807a39ce7478757cf92ac6816110ff2392d56666b38d4038b1fd8cb86`.

## Corpus and denominators

All29837 official MathQA TRAIN rows are accounted for in original order. Reconstructed
split/group hashes and clean FIT inputs exactly match supplied assets. There are
9521 FIT groups and2346 CAL groups with zero normalized-question group overlap.
No assertion of absence of broader semantic overlap is made. Clean selection uses
original supported annotation, original native execution and numeric agreement.

| Partition | All rows | Clean | Unsupported annotation | Binding/execution refusal | Wrong label | Unscorable label |
|---|---:|---:|---:|---:|---:|---:|
| FIT |23912|4590|8522|3989|2470|4341|
| CAL |5925|1182|2129|985|645|984|

All metrics below retain these clean row denominators; operand decisions are15753
FIT and4138 CAL. No CAL learning occurs. This is a previously used, annotation-cleaned
TRAIN calibration subset, not fresh independent evaluation or whole-TRAIN accuracy.

## Full native results

Numeric means the unchanged typed-label comparator on native Rational execution.
ExactAST remains original annotated AST equality, including its operand order.
No equivalence-aware score reinterprets an old numeric error as product accuracy.

| Split | Arm | Free exactAST | Free numeric | Refused | Wrong | Oracle exactAST | Oracle numeric | Refused | Wrong |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
'''
for split in ['FIT','CAL']:
 for arm in ['original','A','B']:
  t=x['results'][arm][split]
  ks=['free_exactAST','free_correct','free_refused','free_wrong','oracle_exactAST','oracle_correct','oracle_refused','oracle_wrong']
  text+='|'+split+'|'+arm+'|'+'|'.join(str(t[k]) for k in ks)+'|\n'
text+='''
Oracle explicitly supplies the full gold operation sequence AND length, uses
predicted operands/prefix, and forces termination. Each arm forces STOP on4124 FIT/
1027 CAL rows and horizon-six on466 FIT/155 CAL rows. This is not inference performance.
All clean outputs are numerically scorable; wrong+refused+correct equals every row.

Teacher left uses gold prefix/op. Right uses gold prefix/op AND gold left. Equivalent
left is membership among all valid lefts; equivalent right is membership in the valid
right set conditioned on gold left. The unchanged denominator includes every step.

| Split | Arm | Left exact | Right exact | Left equivalent | Right equivalent | Whole-row exact grounding | Whole-row equivalent gold-context grounding | Whole-row paired equivalent grounding |
|---|---|---:|---:|---:|---:|---:|---:|---:|
'''
for split in ['FIT','CAL']:
 for arm in ['original','A','B']:
  t=x['results'][arm][split];ks=['left_exact','right_exact','left_equiv','right_equiv','whole_exact_grounding','whole_equiv_gold_context_grounding','whole_pair_equiv']
  text+='|'+split+'|'+arm+'|'+'|'.join(str(t[k]) for k in ks)+'|\n'
text+='''
Whole-row gold-context equivalent means every marginal teacher decision passes;
it is not necessarily a mutually compatible pair. Paired equivalent uses predicted
left and right conditioned on that left, still with the gold previous prefix/op.
Per-head whole-row counts and every event/valid pair are in external records.
Operation teacher non-STOP counts remain12354/15753 FIT and1912/4138 CAL in all arms.
Right equivalence does not expand the gold-left target in this corpus, so its exact
and equivalent scores coincide; the formal policy was not relaxed to create gain.

## Frozen gates: all fail

| CAL gate | Required | Observed B | Result |
|---|---:|---:|---|
| Free numeric minimum |168/1182|127/1182|FAIL|
| Oracle numeric minimum |251/1182|203/1182|FAIL|
| Free beats A |greater than146|127|FAIL|
| Oracle beats A |greater than225|203|FAIL|
| Equivalent left gain over original |at least125/4138|88/4138 (+2.1266pp)|FAIL|
| Equivalent left gain over A |at least125/4138|88/4138 (+2.1266pp)|FAIL|
| Equivalent right gain over original |at least125/4138|-12/4138 (-0.2900pp)|FAIL|
| Equivalent right gain over A |at least125/4138|-21/4138 (-0.5075pp)|FAIL|

Three percentage points requires an integer gain of125 at denominator4138. There
are no selected answers or deployed changes; all old-correct cases remain protected
from this experiment becoming a confident product answer. Raw losses nevertheless
matter and are recorded exhaustively by TRAIN index externally:

| Arm | Split | Mode | Original-correct losses | Of those refused | Refusal→wrong |
|---|---|---|---:|---:|---:|
'''
for arm in ['A','B']:
 for split in ['FIT','CAL']:
  for mode in ['free','oracle']:
   p=f'{arm}/{split}/{mode}/';c=x['transition_counts'];ref=c.get(p+'correct->refused',0)
   text+=f'|{arm}|{split}|{mode}|{c.get(p+"correct->wrong",0)+ref}|{ref}|{c.get(p+"refused->wrong",0)}|\n'
text+='''
## Validation, provenance and reproduction

Original first200 CAL outputs exactly match every retained parsed field through
both original native binary and helper. Original full clean CAL output records also
match retained outputs. Original FIT/CAL free/oracle/teacher totals reproduce all
frozen baselines. Each arm's fixed first200 clean-CAL diagnostic sample exactly
matches its full5772-row run. Four native tests pass: formal targets, finite-difference
and singleton gradients, native Rational transformations/definedness, and legal
mask/tie/singleton-update parity. Independent Fraction arithmetic agrees with32
native executor cases including negative/zero assignments and denominator failure.
Independent tuple-tree code re-enumerates every legal pair on all19891 gold steps
and verifies every original/A/B target set and equivalence label. Original source
bytes, operation weights, and every supplied manifest hash pass final verification.

Only one A/B experiment was attempted; neither arm is incomplete. No alternative
hypothesis, threshold or checkpoint was tried. No failed build/test/replay occurred;
compiler dead-code warnings from reused original modules are retained in logs.
The previously rejected finite-bank, sequence/feature and role-binding models and
GSM value-equality conversion remain closed. This supervision hypothesis is rejected.

No new external data. Reused MathQA TRAIN revision
`fafb9f7ee5b9ec4da9499f9c4177a4c91389f2d6` (coordinator-recorded revision), Apache-2.0
per supplied source card;9013733 bytes, SHA256
`c16335ea4f7c9a8da44ccec52146d29e040582d2c11ca712fcfa2dd0ee964a99`.
Original model8046176 bytes, SHA256
`2b56181582960a862b2da422b0988cf3e20671949c80253bb5b0eaa2d49478e3`.
Comparator unchanged SHA256
`24f1d65ed2f1785b940d7c674565dd9b49c168e498f57c23cbd6c8c2c6eab6f0`.
A model8017709 bytes; B8021487 bytes, both below16MiB. Training wall times4.22s A/
5.92s B, each under600s,8GiB,0swap; the full measured native pipeline took about30s.
All cargo used persistent TMPDIR, existing CARGO_TARGET_DIR, systemd8GiB/0swap,
setsid --wait,600s timeout,-j1 and test threads4. Outer worker scope also8GiB/0swap.
No background process is left running.

Reproduce from this checkout and supplied readonly source/dependencies, using a fresh
persistent destination (the existing experiment refuses overwritten model artifacts):

```sh
EQUIV_RUN=/home/gsc/.cache/glassmind/continuous-20261010/equivbinding-run/replay-new \\
TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/equivbinding-run/tmp \\
CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/equivbinding-run/target \\
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \\
setsid --wait timeout --kill-after=10s 1800 \\
bash tools/quantities/equivalent-binding/reproduce.sh
```

The frozen directory includes original supplied files/binaries/data/model, exact
compiled helper source and binary, Cargo.lock, adapter source, patch, plans,
membership, per-arm model weights, commands, logs, checks, full native records,
loss transitions, results and a complete per-file SHA256/byte manifest. A compact
freeze receipt is beside this report. No edits after final freeze are permitted
without a separately preserved revision. Numeric equality still does not prove
English semantics; no DEV run is authorized even on a passing outcome.
'''
(O/'REPORT.md').write_text(text)
for name in ['results.json','corpus-frozen.json','independent-verification.json']:
 (O/name).write_bytes((R/name).read_bytes())
