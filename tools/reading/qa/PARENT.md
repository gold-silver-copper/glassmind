# Passage proposals in the main command

Evaluated source `2a968ec`. The parent retained the worker's span ranker as an
explicitly unverified proposal mechanism. It does not supply factual answers to
`talk`, persist claims or assert that source occurrence proves a relation.

```sh
cd /home/gsc/code/glassmind
target/release/glassmind read \
  --model tools/reading/qa/artifacts/model.json \
  --context passage.txt 'Who visited Zürich?'
```

With passage `Élodie visited Zürich in 2018.`, the output labels Élodie as a
**candidate answer (unverified)** and prints the exact source passage. `--explain`
adds byte offsets and ranking scores, explicitly not probabilities. `--json`
returns typed prediction/refusal records. `--input FILE` accepts JSONL containing
only `context` and `question`; IDs, titles and answer annotations reject.

File/input limits are 64 KiB for a passage, 4 MiB for a model and 512 KiB per
JSONL row. Invalid models fail; finite weights that overflow score or margin
arithmetic produce a refusal. These guards preserve all measured predictions.

## Independent verification

The coordinator rebuilt the workspace, fit the same 4,000 training questions
again, selected the threshold on the same 1,000 calibration questions, and replayed
sample 200 and full 10,570 dev questions. Trained model bytes and every baseline
and candidate prediction byte exactly match the worker. All 10,570 predictions
through the main command also match the crate binary byte-for-byte.

| Measure | Lexical baseline | Learned ranker |
|---|---:|---:|
| Raw proposal normalized EM | 12.393567% | 23.954588% |
| Raw proposal token F1 | 19.001517% | 34.518425% |
| Selected exact / refused / wrong | 1,305 / 108 / 9,157 | 815 / 8,964 / 791 |
| Selected normalized EM | 12.346263% | 7.710501% |

The selected predictions are only **50.7472% exact** at 15.1939% coverage. Of the
baseline's selected correct answers, 67 become selected errors and 976 become
refusals. Raw proposals lose 641 previous exact matches while adding 1,863.
The unsupported-relation probe still returns an incorrect candidate; five of
six independent behavioral probes work. Neither extraction nor the threshold
establishes entailment, and SQuAD 1.1 does not test genuine unanswerables. These
limits prohibit treating this component as automatic factual conversation.

Workspace fmt, all-target clippy, release build and strict 4/4 fixture smoke pass.
Six QA tests and two existing byte-model tests pass. Fourteen main-command
contract checks pass, including strict annotation rejection, Unicode offsets,
bounded input, invalid models and arithmetic overflow. These are interface checks,
not 14 proofs of semantic accuracy. Cross-command checks retain all three metadata
exports and eight ordinary-name/explanation regressions.

[Parent validation](integrated/validation.json), [full metrics](integrated/full.json),
[behavioral probes](integrated/independent.json), and
[main-command contracts](integrated/main-contracts.json) retain the evidence.
The original worker source is archived under
`~/.cache/glassmind/continuous-20261010/readingqa-run/submitted/`; raw artifacts
are preserved outside git in `readingqa-run/artifacts/` with a hash manifest.
No held-out evaluation ran and no scoring definition changed.

## Reproduction

The exact parent script/logs are at
`~/.cache/glassmind/continuous-20261010/coordinator/readingqa-check/`.
It ran the workspace gate first, then scoped tests, independent fit/calibration,
sample/full scoring, behavioral probes and full main-command equivalence.
`SQUAD_SOURCE` selects the retained pinned source; `QA_ARTIFACTS_DIR` and
`QA_REPORT_DIR` place independent output outside the checkout. Metric function
ASTs are the unchanged, hash-pinned official functions. Every expected question
ID is required before scoring, including refusals; no denominator is reduced.

Model SHA256: `b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5`.
Main binary SHA256: `8659839583857656c1fdf2d0f22e4fd57de245db84778f9ab5b4d754de62af41`.
Full predictions SHA256: `37db70ae0c949a89427a707c37812a666d9507a2192bcb75ed3e4145d3a598a7`.
