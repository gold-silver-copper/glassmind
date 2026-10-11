# Joint entity, relation and answer audit

Answer-only scoring can reward the wrong claim when different subjects share
an answer, such as a country or gender. This separate diagnostic requires the
predicted subject, relation (including inverse direction), and answer to match
the public validation annotation exactly. It preserves all 2,821 rows and the
original refusals. The historical answer-only scorer remains unchanged.

| Frozen scope | Answer-only right / refused / wrong | Joint right / refused / wrong |
|---|---|---|
| Historical validation-subject-seeded 500 | 256 / 2545 / 20 | 252 / 2545 / 24 |
| Rejected TRAIN-selected 999 | 47 / 2712 / 62 | 2 / 2712 / 107 |

Four and 45 answer matches, respectively, had the wrong subject. This is not a
runtime regression or an improved predictor. The TRAIN expansion and subsequent
entity filter remain rejected under their original criteria. The diagnostic
does not retroactively change those acceptance decisions.

`joint-audit.py` uses frozen question-only predictions, verifies complete question
order and prediction coverage, and reconstructs the authoritative answer-only
totals before reporting joint counts. Reports include source hashes and every
coincidental match. Two tests check shared-answer errors and incomplete or
contradictory predictions. No training, inference or network requests are needed.

Exact annotated IDs do not prove current factual truth, global inverse uniqueness,
or satisfaction of every question constraint. Redirect normalization is absent;
annotation errors and Wikidata snapshot differences remain possible. The two
indices have different selection procedures and must not share a score series.

Reproduce from the repository root:

```bash
FACTS_DATA=/home/gsc/.cache/glassmind/data/simplequestions
AUDIT_RUN=/home/gsc/.cache/glassmind/continuous-20261010/linkquality-run/artifacts
python3 tools/facts/test-joint-audit.py
for scope in old500 new999; do
  python3 tools/facts/joint-audit.py \
    --labels "$FACTS_DATA/valid.tsv" --questions "$AUDIT_RUN/valid.questions" \
    --predictions "$AUDIT_RUN/$scope-valid-frozen.jsonl" \
    --official-totals "$AUDIT_RUN/$scope-base-2821.json" \
    --scope "$scope frozen diagnostic" --output "$AUDIT_RUN/$scope-joint-replay.json"
done
```

The unchanged frozen predictor source is `4a05e4cbd53209a1666f8335e2b974ab19aa6da1`.
See [historical scope](continuous/HANDOFF.md), [TRAIN snapshot provenance](trainindex/HANDOFF.md)
and [rejected filter](linkquality/HANDOFF.md). Raw data remains outside git;
this audit downloads no data and changes no licenses.

## Why linking alone cannot repair the rejected snapshot

`coverage-audit.py` uses annotated IDs only to examine knowledge coverage; it
does not generate predictions or select new entities. It validates full question
order and row coverage before joining frozen outputs. Among2821 validation rows,
only232 annotated entities exist in the999-entity snapshot. Of their232 relations,
228 are inverse and four forward. Even with annotated entity/relation supplied,
205 have a nonempty local answer set omitting the annotated answer,24 have no
claim, and only three have a unique matching claim. None has a nonunique set
containing the annotated answer.

The linker already finds193 of those232 entities, and190 predicted relation
labels match. Their outcomes are148 multiple-answer refusals,21 missing claims,
10 uncertain relations and11 accepted answers. Thus the two joint matches are
close to this snapshot's three unique matching claims; another alias filter
cannot address the dominant knowledge gap.

On19481 TRAIN rows,3031 entities are covered, including2293 inverse relations.
Their claim sets have555 unique gold matches,466 missing claims,1853 nonempty
sets omitting gold and157 nonunique sets containing gold. This supports examining
TRAIN-only entity selection and inverse-edge coverage before another linking
experiment. These oracle diagnostics are not model accuracy or global uniqueness.

[TRAIN report](../../docs/results/facts-train999-coverage-train.json) and
[validation report](../../docs/results/facts-train999-coverage-valid.json) include
input/tool hashes. Reproduce with `coverage-audit.py --index INDEX --labels
valid.tsv --questions valid.questions --predictions new999-valid-frozen.jsonl
--output REPORT`; add `--paired` for the retained TRAIN baseline/candidate file.
The index is `WORK/trainindex-run/tmp/trainindex/scope1000/index.json`; frozen
questions/predictions are in `WORK/linkquality-run/artifacts`. No runtime, index,
scorer, original acceptance criterion or historical metric changed.
