# Stories in the main conversation

Start `glassmind talk --session conversation.json`. Give one statement or
question per line, for example:

```text
My name is Mira
Mary went to the kitchen.
Mary picked up the apple.
Calculate 2 + 3 * 4
Mary moved to the garden.
Where is the apple?
```

The final answer is `garden`. Names, preferences, greetings, calculations,
and last-message recall coexist with story facts. Restart with the same
session file to continue. `/reset` clears both kinds of memory.

Supported story forms come from the existing typed stories reasoner:
movement, taking/dropping/giving objects, explicit and uncertain locations,
spatial and size relations, kinds, colors, and the existing action model's
states. Questions can omit the final question mark. Explicit name and
preference statements retain priority over all other routing. A learned
intent proposal cannot bypass story invalidation or override typed reasoning.

JSON responses use `story_fact` for accepted assertions, `story_answer`
for deductions, `story_prediction` for learned or defeasible inferences,
and `refusal` when evidence is insufficient. Story replies contain `text`,
`answer` (null unless answered), and `reason`. Predictions also disclose their
basis in normal text; they are not described as logical certainty.

An unsupported assertion may change previously stated facts. It invalidates
all story answers, including after restart. The reply asks the user to
`/reset` and restate the facts; calculations and explicit personal memory
remain usable. Unknown questions and explicit service/calculation requests
leave facts alone. Compound messages, hedged actions and unknown states are
conservatively refused. Current-location, kind, color and supported-state
corrections accept explicit `Actually, ...`, `No, ...` or `Correction: ...`.
Other corrections invalidate the story; arbitrary retraction is unsupported.
Corrections are current-state updates, not edits of historical events.

## Implementation and bounds

The CLI now depends on the existing `glassmind-stories` crate. No changes to
that crate, reasoner rules, quantities, facts, world-rust or model weights
were needed. `everyday.rs` owns a session with flattened legacy memory fields
and a defaulted `stories` field, so old memory-only JSON still loads and the
existing memory persistence consumers retain their root-level fields.
`talk_stories.rs` wraps the existing world and replays its assertion journal.

The journal holds at most 512 assertions and 262,144 UTF-8 bytes. Capacity
exhaustion clears the journal and persists an invalidated flag; it never
silently evicts evidence. Questions and ordinary memory/calculation turns do
not consume this capacity. Existing 8,192-byte input-line limits still apply.
Persistence uses deterministic field/journal order and the existing atomic
rename. These limits apply to the new story state, not the pre-existing
personal-preference map. Session files are local trusted input, not an
untrusted network format. Concurrent writers to one session are unsupported.

## Experiment and evidence

Starting source: `e73f68c36e183b4060468cc6e752b8709f0662a0`.
Coordinator integration also routes unresolved personal-memory replies through
story invalidation when story context exists. This prevents an ambiguous
“Actually, it is the garden” correction from preserving a stale location,
while retaining the original clarification when no story exists. A story reply
also clears the previous personal-memory focus, so a later bare correction
cannot silently replace a name/preference from an earlier topic. Explicit
name/preference statements and questions establish that focus again. A focused
restart regression covers both cases. Exact evaluated
binary SHA-256 values are in `talk-stories-evidence/*.json`.

Hypothesis stated before implementation: typed story routing and a bounded
replay journal will add cross-turn and restart reasoning without regressing
explicit memory or calculations. Acceptance: improve the fixed mixed check
with no wrong answers, retain 41 memory checks, 27 mixed calculation checks,
and all 3,116 quantity outcomes; evaluate full established bAbI dev through
`talk` before handoff.

| Development check | Baseline | Candidate |
| --- | --- | --- |
| Fixed mixed dialogue (47 checks) | 22 right, 25 refused, 0 wrong | 47 right, 0 refused, 0 wrong |
| Same mixed dialogue with existing intent model | not run | 47 right, 0 refused, 0 wrong |
| bAbI sample (400) through talk | 0 right, 400 refused, 0 wrong | 397 right, 3 refused, 0 wrong |
| Full bAbI dev (3,987) through talk | 0 right, 3,987 refused, 0 wrong | 3,966 right, 18 refused, 3 wrong |
| Existing personal-memory smoke | supplied 41 right | 41 right |
| Existing mixed calculation dialogue, with/without intent model | supplied 27 right | 27 right each |
| Full synthetic quantity dev (3,116) through talk | supplied 3,056 right, 60 refused, 0 wrong | 3,056 right, 60 refused, 0 wrong |

Quantity right includes 762 expected refusals. A direct comparison against
the frozen baseline found zero changes in kind, value or unit across all
3,116 responses (`quantity-equivalence.json`). The mixed check includes
possession propagation, ambiguity, explicit correction, relations, prediction
labeling, reset, restart, taint persistence, legacy files, interleaving and
source-turn recall. These are same-author product checks, not independent
language generalization. The adapter imports the unchanged training-only
bAbI partition and normalization from `tools/stories/babi.py`. Runtime sees
only `/reset`, assertions and question text, never answers or task IDs.

Rejected experiment: the first candidate passed mixed and bAbI checks but
accepted “Mary might have moved to the loft” and “Mary is gone” without
invalidating the old location, and ignored the assertion after a question
in a compound message. All three returned the stale location. The exact
outputs are retained in `rejected-initial-routing.json`. The final guard
rejects these cases and persists invalidation; focused Rust tests cover them,
replay/pronoun determinism and the 512-assertion limit. This is a conservative
routing correction, not a change to benchmark definitions.

Limitations: the inherited reasoner still has three wrong bAbI dev answers;
this integration does not fix its reasoning rules. Restricted English,
short entity phrases (at most three words) and conservative global
invalidation are intentional. Arbitrary names, prose, hedges, quotation,
implicit corrections and complex contradictions are not generally solved.
No held-out evaluation or independent dialogue-quality claim is made.

## Source and provenance

No new downloaded or generated training data: **0 bytes**. The unchanged
embedded action model is `tools/stories/action-model.json`, **979 bytes**,
SHA-256 `8b4ce4359a10e31948dba8cfa4b32c378b206487c04a987176aea2a56c8e0bf8`.
Its established provenance records 5,015 training stories and no QA labels.
It is a small association model, not an LLM or answer table; no retraining
or runtime network/model service is used.

The reused evaluation source is Facebook bAbI tasks 1–20 v1.2, English 1k
**training members only**, **CC-BY-3.0**, 2,947,923 training bytes across 20
files. The established SHA-256 episode split uses seed `glassmind-stories-v1`.
The source archive pin is
`84f5296ab9a1ad0dc9464e08c491d65cd08830fca3acae9ab86f75e0fb81573c`;
source/member hashes and attribution are in `tools/stories/source.json`,
and model provenance in `tools/stories/action-model.provenance.json`.
No official test member, hidden test/generator, private old log, credential,
API key, or LLM was accessed. Code and the authored adapter use the existing
repository MIT OR Apache-2.0 license. The active model was inherited from
the coordinator; this worker did not change model or authentication settings.

## Reproduction

Use the existing target directory and an on-disk temporary directory. Every
cargo invocation must use the capped scope and process-group wrapper:

```sh
export TMPDIR=/home/gsc/.cache/glassmind/continuous-20261010/storychat-run/tmp
export CARGO_TARGET_DIR=/home/gsc/.cache/glassmind/continuous-20261010/storychat-run/target
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 cargo build -p glassmind -j1
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 cargo test -p glassmind -j1 \
  talk_stories::tests -- --test-threads 4
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 cargo clippy -p glassmind --all-targets -j1 -- -D warnings
systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 \
  setsid --wait timeout --kill-after=10s 600 cargo fmt --all --check

python3 tools/talk-story-dev.py "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/mixed.json"
python3 tools/talk-story-dev.py "$CARGO_TARGET_DIR/debug/glassmind" \
  --model "$HOME/.cache/glassmind/models/everyday/intent.json" --output "$TMPDIR/mixed-model.json"
python3 tools/talk-story-dev.py "$CARGO_TARGET_DIR/debug/glassmind" --mode babi \
  --data "$HOME/.cache/glassmind/data/babi/en" --output "$TMPDIR/sample.json"
python3 tools/talk-story-dev.py "$CARGO_TARGET_DIR/debug/glassmind" --mode babi --sample 0 \
  --data "$HOME/.cache/glassmind/data/babi/en" --output "$TMPDIR/full.json"
python3 tools/talk-dev.py "$CARGO_TARGET_DIR/debug/glassmind" --output "$TMPDIR/calculations.json"
python3 tools/talk-dev.py "$CARGO_TARGET_DIR/debug/glassmind" \
  --model "$HOME/.cache/glassmind/models/everyday/intent.json" --output "$TMPDIR/calculations-model.json"
python3 crates/everyday/tools/smoke_memory.py "$CARGO_TARGET_DIR/debug/glassmind" "$TMPDIR/memory" --command talk
python3 tools/talk-quantity-dev.py "$CARGO_TARGET_DIR/debug/glassmind" \
  --output "$TMPDIR/quantities.json" crates/quantities/dev/synthetic-dev.jsonl crates/quantities/dev/extended-dev.jsonl
```

To reproduce the baseline, build the starting commit in a separate checkout
with the same wrapper and run this adapter against that binary. A frozen
baseline binary remains at `$TMPDIR/glassmind-story-baseline` for the current
coordinator session. No agents, services or background jobs are left running.
Worker evidence above is retained separately from coordinator reproduction.
Final integrated reports are in `talk-stories-evidence/integrated/`.
