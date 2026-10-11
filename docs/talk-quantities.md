# Exact quantities in conversation

`glassmind talk` now returns exact arithmetic, units and calendar answers
through the existing quantities solver. Explicit memory and social responses
retain priority. A calculated response includes `kind`, `text`, `value`,
`unit` and `method` in JSON mode. Every request still advances conversation
memory exactly once, including last-message recall and persistent sessions.
Failed quantity parses preserve the existing refusal or intent response.

No quantity solver, intent weights or memory semantics changed. No new data
or runtime model calls are needed. The dependency is in the CLI orchestration
layer; the everyday memory crate stays independent of the quantities world.

## Evidence

Baseline: `f998e27`; candidate: the commit titled
`Answer exact quantity questions in the main conversation command`.
The score journal records its exact commit hash.

| Development check | Baseline | Candidate |
| --- | --- | --- |
| Fixed mixed conversation, no intent model | 18 right / 9 refused / 0 wrong | 27 / 0 / 0 |
| Same conversation, frozen CLINC model | 18 / 9 / 0 | 27 / 0 / 0 |
| Existing memory smoke | 41 right | 41 right |
| Full synthetic quantities through main `talk` | previously unavailable | 3056 right / 60 refused / 0 wrong |

The full quantities denominator is 3116; right includes 762 expected
refusals. These are synthetic development checks, not external or held-out
conversation performance. All existing quantity misses remain unsupported.
Workspace fmt, all-target clippy, release build and strict 4/4 fixture smoke
passed. No held-out items or evaluations were used.

## Reproduce

Run the existing capped `tools/check.sh` first to build. Then:

```sh
python3 tools/talk-dev.py target/release/glassmind --output scratch/talk-dev.json
python3 tools/talk-dev.py target/release/glassmind \
  --model "$HOME/.cache/glassmind/models/everyday/intent.json" \
  --output scratch/talk-model-dev.json
python3 crates/everyday/tools/smoke_memory.py target/release/glassmind \
  scratch/talk-memory --command talk
python3 tools/talk-quantity-dev.py target/release/glassmind \
  --output scratch/talk-quantity-dev.json \
  crates/quantities/dev/synthetic-dev.jsonl crates/quantities/dev/extended-dev.jsonl
```

The original before/after reports are retained at
`~/.cache/glassmind/continuous-20261010/coordinator/`. Parent token usage is
not available; the journal retains the prior known worker-token lower bound.
