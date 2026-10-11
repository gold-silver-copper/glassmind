# Growing glassmind: a guide for loop agents

`prompts/grow-fast.md` is the active direction and overrides older rules here.
Use Codex GPT-6 Astra fast through the logged-in ChatGPT account; never API
keys or Claude. Compression is off. Measure a fixed dev sample during
iteration, then the affected track's full dev split before pushing a batch.

You are improving glassmind, a system that understands English questions and
commands about Rust code with **no neural network inside**. Everything it
knows is Rust source: its types are the concepts it can think about, its
functions are how it reasons. You are given a group of training turns it gets
wrong or refuses; make it handle them, in a way that generalizes.

## The layers, and where knowledge goes

| crate | what belongs there | must not contain |
|---|---|---|
| `crates/meaning` | language-neutral, data-neutral meaning types: questions, descriptions, conditions, things, references, actions, replies; the `Language` and `World` interfaces | English words or categories, Rust-code concepts |
| `crates/grammar` | the parser/generator building blocks (Earley parser, `Rule`, `Sym`, `Sem`) | anything language- or world-specific |
| `crates/english` | English tokenizer, categories (`Cat`), phrase values (`Sem`), grammar rules for English sentence shapes, function words ("the", "every", "it", "where") and replies | anything about Rust code |
| `crates/world-rust` | Rust items, kinds (`RustKind`), relations (`RustRelation`), indexing with `syn`, lookup, answering meanings | English |
| `crates/english-rust` | the English words for Rust kinds and relations ("struct", "fields", "calls") | grammar rules, sentence shapes |
| `crates/mind` | dialogue: readings, reference resolution, preferences, sessions, traces | English, Rust |
| `crates/cli` | turning replies into the machine-readable answer format (`answer.rs`) and the commands | knowledge |

`cargo run -p glassmind-depcheck` enforces the dependency directions.

Name meaning types after concepts (`Locate`, `Possession`, `Count`), not
after English words or grammatical categories. If English and other languages
would divide a meaning differently, leave the choice open in the meaning.

## How understanding works

1. `english::tokens::tokenize` splits the text (the only code that reads raw text).
2. `grammar::Grammar::parse` (Earley) reads the tokens with every enabled rule,
   building `english::Sem` values bottom-up, and keeps every reading.
3. Each reading's `Sem::Utterance` is a `meaning::Utterance`.
4. `mind` resolves names (`Thing::Named`) to items with `World::lookup`, asks
   the world to answer each resolved reading, drops readings the world
   rejects, and picks among the rest with stated preference rules.
5. The reply (`meaning::Reply`) is spoken by the same grammar run backwards
   (`Grammar::generate` uses each rule's split function).
6. `cli/src/answer.rs` turns the reply into the JSON `Response` the scorer
   checks (`crates/answer`, specified in `docs/tasks.md`).

A rule is `Rule::new(RuleInfo::new(id, round), lhs, rhs, sem)`. `Sem::Both(build,
split)` reads and speaks; `.read_only()` marks forms the system should accept
but never say (sloppy or rare input). Single words use `english::word(...)`.

## Coding tasks (the harness)

Some training turns are coding tasks on a broken copy of a small crate
(`kind` `build_status`, `fix_build`, `test_status`, `fix_tests`): "does it
build?", "fix the build", "which tests fail?", "make the tests pass". The
Rust world runs the crate's own tools for them (`crates/world-rust/src/checks.rs`):
`cargo check`, `cargo test`, `clippy`, whose reports become items of kind
`Problem` and `Test`. A remedy is a set of changes; the scorer applies it to
a scratch copy and runs the check, so **a remedy that does not make the
check pass costs like a wrong answer**, while offering none, or saying some
problems would remain, is an honest refusal. Today the only remedies are the
compiler's own suggestions; making failing tests pass needs more (finding
the code a failing test exercises and trying changes there, always verified
by running the tests).

Checks are slow, so their results are cached by the files' contents. Scratch copies never go to `/tmp` (it is in memory on this machine). Set
these when you run `eval` on coding tasks, and score with `--sources`:

```sh
export GLASSMIND_CARGO_TARGET=$HOME/.cache/glassmind/check/target GLASSMIND_CHECK_CACHE=$HOME/.cache/glassmind/check/results GLASSMIND_SCRATCH=$HOME/.cache/glassmind/check/scratch
./target/release/glassmind eval --sources $SOURCES --inputs $DATA/group-inputs.jsonl > group-out.jsonl
./target/release/glassmind-loop score --labels $DATA/group-labels.jsonl --outcomes group-out.jsonl --sources $SOURCES
# talk to a broken crate yourself:
./target/release/glassmind code $SOURCES/<task source>
```

## Rules for every change

- **Any method is allowed** if it measurably helps: grammar rules, fuzzy or
  learned matching of words and names, probabilities, outside data, a new
  formalism. Prefer what generalizes to phrasings you have not seen: a hidden
  held-out set, written by people with other habits about other
  repositories, measures that (it decides nothing; a hidden validation set
  of other repositories guards acceptance). Never fake answers (no tables of particular sentences or
  names with their answers).
- **Every new rule** (grammar rule, word, world relation, preference) gets a
  stable unique dotted ID (`en.question.where_defined`,
  `en.rust.noun.structs`, `rust.relation.has_field`, `mind.prefer.…`), the
  round number you were given (`RuleInfo::new(id, ROUND)`), a real name, and
  a doc comment written from its actual uses. World and mind rules must also
  be listed in their crate's `RULES` and be switchable off through
  `Switches` (look at how existing ones do it).
- **Act, then verify.** A proposed action can proceed when a compiler, tests
  or world lookup can check its result. Roll back failed changes. Ask when
  ambiguity has no reliable check; never claim a failed check as success.
- **Every reply must have words.** A reply the language cannot say costs
  0.5 extra, and a change that leaves more replies without words is
  rejected.
- **Wrong is far worse than "I don't know".** A wrong answer costs 10; a
  refusal at most 1. Do not make a turn answer unless the answer is right by
  `docs/tasks.md`. Do not turn a right or honest refusal into a wrong answer:
  such a change is rejected outright.
- **Size can be priced** (currently disabled): every syntax token of non-test code in the mind crates
  costs a fixed price in failure units (your brief says how much) (identifiers count 1 whatever their length;
  comments and docs are free). Prefer general rules that cover many turns
  over many specific ones. Delete what you make redundant.
- Non-test code must not panic: no `unwrap`, `expect`, indexing or `panic!`
  (clippy denies them).
- Few, meaningful tests; no snapshot or exact-output tests.
- Keep `docs/tasks.md` semantics: they are the contract with the oracle.

## Checking your work

Paths are given in your brief (`$SOURCES`, `$DATA`). Other agents build at
the same time: always use `-j 4` and `--test-threads 4`.

```sh
tools/check.sh # capped fmt, clippy, build and one fixture smoke
# Every additional cargo run needs a memory-capped scope, TMPDIR,
# an isolated process group and a timeout that kills the group.
# For example:
TMPDIR=$HOME/.cache/glassmind/check-tmp systemd-run --user --scope \
  -pMemoryMax=8G -pMemorySwapMax=0 setsid --wait timeout -k 10s 600 \
  cargo build --release -j 1 -p glassmind
cargo clippy --workspace --all-targets -j 4 -- -D warnings
cargo run -q -p glassmind-depcheck
# your group's turns:
./target/release/glassmind eval --sources $SOURCES --inputs $DATA/group-inputs.jsonl > group-out.jsonl
./target/release/glassmind-loop score --labels $DATA/group-labels.jsonl --outcomes group-out.jsonl
# the affected track's full dev split before pushing a batch:
./target/release/glassmind eval --sources $SOURCES --inputs $DATA/train-inputs.jsonl > train-out.jsonl
./target/release/glassmind-loop compare --labels $DATA/train-labels.jsonl --before $DATA/baseline-outcomes.jsonl --after train-out.jsonl
# one sentence, with the full trace:
./target/release/glassmind ask --explain -s tds=$SOURCES/tds "which structs have a field named hp"
```

Leave your changes uncommitted in the working tree; the loop commits accepted
changes. Finish with a short summary of what you changed and why.
