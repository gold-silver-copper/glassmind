# Prompt: grow glassmind fast, in coding and in conversation

You continue work on glassmind (`~/code/glassmind`, GitHub
`gold-silver-copper/glassmind`, branch `harness`). This prompt is your
working direction from now on.

Read for context, but don't edit these; they are history:
- `prompts/glassmind.md` (the full brief);
- `prompts/handoff-2026-10-10.md` (where the last session stopped);
- `~/code/notes/projects/glassmind/progress.md`.

Where they disagree with this prompt, this one wins. Commit this prompt
first, with this paragraph as the reason.

**The user's ask:** make glassmind as good as possible, as fast as possible,
at coding **and** at basic conversation, working on many benchmarks in
parallel, with minimum ceremony and minimum testing. Your judgment about
what moves the scores beats any plan written here. Change anything,
including this prompt, when evidence says so.

## Where things stand (2026-10-10 night, commit `4cd72a3`)

| set | right | wrong |
|---|---|---|
| training, `train-v3` (7,557 turns of questions about code) | 89.4% | 0 |
| unseen-style selection set v2 (711) | 37.8% (3% with the grammar alone) | 1 |
| rustbench dev, first run | 7.8% pass | 0 |
| rustbench held-out | 12% pass, nearly all the rest refused | 0 |

**The main lesson so far:** grammar grown on training failures reaches 99%
on the styles it was grown on and 3% on a new one. The gains came from
models that read (the request tagger in `crates/models`), with typed code
checking what they propose. Keep building that way.

**Coding today** works only where rustc prints a ready-made suggestion
(`rust.remedy.compiler_suggestions`). Edits, failing tests and "implement"
are at 0%. `build_status` and `fix_build` are near 0 because a guessed
action is always asked about first.

**The loop is stopped** (a `STOP` file in `~/.cache/glassmind/loop/state`).
Its config focuses growth on `harness` and `rustbench` failures, runs 8
evaluations at once, and merges commits you push to `harness` at each round
boundary (`upstream`), so you never stop it for a hand change.

```sh
cd ~/code/glassmind && tools/loop-service.sh start ~/.cache/glassmind/loop/state ~/.cache/glassmind/loop/bin/glassmind-loop
# progress: tail -f ~/.cache/glassmind/loop/state/logs/loop.log
# stop at a round boundary: touch ~/.cache/glassmind/loop/state/STOP
```

## Fixed: the only rules

- **Machine safety.** Every cargo run on generated, broken or agent-written
  code goes in a memory-capped scope with its own process group, killed as a
  group on timeout:

  ```
  systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0 setsid ...
  ```

  A broken test once reached 82 GB.
  - The loop runs in `glassmind.slice`.
  - At most 4 agents at once; `-j 4` per run and `-j 8` overall;
    `--test-threads 4`.
  - Never write to `/tmp` (set `TMPDIR` for every cargo run).
  - Never `pkill -f`; use exact PIDs.
  - Check free memory before big jobs: the user's other projects share the
    machine.
- **Honest scores.** Agents never read held-out material, and you never read
  held-out items, only totals.
- **No LLM inside glassmind at run time.** The one allowed learned code
  model is a code-only proposer:
  - it never sees English;
  - it only proposes candidates;
  - types and tests decide.
- **Opus 5.5 only**, through `claude -p --model claude-opus-5-5
  --no-session-persistence` on the user's login. Never an API key, never
  `~/.claude/.credentials.json`.
- **Never depend on `~/code/flexion`.** Read it for ideas only.
- **No panics in non-test code**, enforced by the workspace clippy lints.

Everything else is yours to decide.

## Mode: grow fast

- **Act, then verify.** When cargo, tests or a world lookup can check a
  result:
  - make the change or give the answer;
  - run the check;
  - keep it if it passes, undo it if it doesn't.

  Ask or refuse only when nothing can check it. A failed check still costs a
  wrong answer, so "never wrong" stays honest without refusing everything.
- **Minimum testing.**
  - CI runs fmt, clippy, build and one smoke run.
  - Add a unit test only when the same bug comes back.
  - No full-set evaluations per change. Steer on fixed samples of about
    5 minutes; run a track's full dev split before pushing a batch.
- **Skip for now:**
  - the human-written final set (benchmarks' own test splits are the check);
  - timing reports;
  - compression rounds (switch them off; code size doesn't matter yet);
  - grammar growth on training-style QA failures;
  - PR conditions.

  Push to `harness` often, and merge to `main` with `gh pr merge --merge`
  whenever CI is green.
- **Never block on waits.** Start long jobs in the background, get notified,
  and do other work meanwhile.
- **Parallelize.** Run 3–4 subagents in their own worktrees on separate
  tracks or features at once, plus the loop on whichever track has the most
  checkable failures. Merge their work as it lands.
- **Dependencies are fine:** rust-analyzer's crates or binary, syn,
  datasets, anything useful except flexion. Record each data source's
  version, size and license in `data/README.md`.

## The scoreboard: tracks in parallel

Each track has:
- an adapter that turns its items into glassmind turns with checkable
  answers, scored right / refusal / wrong as now (BLiMP and reading are
  scored by their own measure);
- a fixed dev sample to steer by;
- a held-out split scored every few hours, totals only.

One chart, one line per track, against hours and Opus tokens.

| Track | Benchmark | What it checks |
|---|---|---|
| Coding | rustbench (`~/code/rustbench`; dev to steer, held-out to check) | fixes, edits, failing tests, small implementations, explaining, asking |
| Stories and reasoning | bAbI, 20 tasks | who has what, where things are, counting, yes/no, simple deduction, "it" |
| Everyday requests | CLINC150 | the intent of everyday requests; its out-of-scope class counts as a right refusal |
| Multi-turn dialogue | Schema-Guided Dialogue (or MultiWOZ) | tracking what the user wants across turns |
| Facts | SimpleQuestions over Wikidata | "where was X born?", through an indexed Wikidata store |
| Numbers, units, dates | SVAMP / MAWPS, plus generated unit and date questions | arithmetic from English; units as types |
| Grammar | BLiMP (also in `~/code/sememe/data/blimp`) | prefers the grammatical sentence of each minimal pair |
| Reading real English | BabyLM (`~/code/sememe/data/babylm`) | the share of held-out sentences glassmind reads into a meaning; bits per byte once it has a probability model of text |
| Its own chat | generated small talk with checkable facts | names, preferences, follow-ups, corrections, "what did I just say" |

Find more benchmarks when a track needs them.

**Where effort goes:**
- Each new piece of work goes to the track with the best recent gain per
  hour, but every track gets a minimum share so none is ignored.
- A change may cost one track a little if another gains more. Undo anything
  that drops a track a lot.
- QA about code is no longer a headline. Keep it as one more track.

## Highest-payoff work, roughly in order

Coding:

1. **Act-then-verify in the harness.** This unblocks build status and build
   fixing.
2. **Use the repo's state to understand the request.** A broken build,
   failing tests or recent changes decide what "fix it" means, whatever the
   wording.
3. **Edits through rust-analyzer's assists:** rename (including structs and
   enum variants), add a derive or a field, change a signature, fill in
   match arms or struct fields. The request picks an assist and a target;
   cargo checks.
4. **Fixes beyond rustc's suggestions.** Mine broken/fixed pairs from
   rust-lang/rust `tests/ui` (`.fixed` files) and clippy's tests, plus
   build-fixing commits in open-source repos. Learn ranked fix templates per
   error code.
5. **Make failing tests pass.** Find the likely bug from what the failing
   tests run, read the assertion values, try small changes; hidden tests
   decide. Learn from real bug fixes, not from the task generator's own
   breakages.
6. **Small "implement" tasks.** Type-directed search over std compositions,
   ranked by a model of likely Rust code; tests decide.

Conversation:

1. **The tracks' adapters and a world for each:**
   - a story world for bAbI (people, places, objects, events over time);
   - a conversation memory for chat and dialogue;
   - a facts store for Wikidata;
   - typed quantities, units and dates for word problems.
2. **Models that read, shared across tracks:**
   - the request tagger generalized beyond code requests;
   - a parse ranker;
   - entity linking and "it";
   - intent with out-of-scope.
3. **Ideas from `~/code/sememe`** (read it; reuse its downloaded data; never
   depend on its crates):
   - **Rule probabilities learned from raw text by EM:** parse BabyLM,
     count rule use, re-weight, repeat. The parse ranker then comes from
     millions of real sentences, not from glassmind's own requests.
   - **Opus as a teacher, in waves.** It labels real sentences in
     glassmind's own meaning types, which trains the tagger and the parse
     ranker. Show the teacher how glassmind already reads the known words,
     or its labels contradict each other (sememe found 287 of 1,291
     inconsistent before it did this).
   - **Judge on a sample, confirm on everything affected:** sememe uses a
     4,000-document sample for big changes, then re-checks all affected
     documents.

## Reporting

- Every few hours, add a five-line note to
  `~/code/notes/projects/glassmind/progress.md`:
  - the scoreboard (dev and held-out per track);
  - what changed;
  - what's next.

  Write it so a fresh session can pick up from it alone.
- At each merge to `main`, update the chart and the README's numbers.
- When the session ends, stop every process, service, worktree and
  background shell you started.
