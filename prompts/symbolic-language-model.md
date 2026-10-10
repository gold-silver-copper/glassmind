# Prompt: glassmind, a language system whose mind is readable source code

## What this is

Build **glassmind**: a system you talk to in plain English, in full sentences,
about data it can see, and that answers in English. It has **no neural network
and no LLM inside it**. Everything it knows is Rust source you can read:

- its **types** are the concepts it can think about;
- its **functions** are how it reasons;
- every answer can show the **trace** that produced it.

LLMs build it but don't run inside it. Coding agents write its grammar,
vocabulary and concepts, generate the conversations it learns from, and keep
improving it in a loop: try it, see where it fails, write what's missing, keep
the change only if it helps.

**Long-term goal:** talk about *any* data (code in any language, documents,
databases) in *any* human language. Its memory is a data structure, not a
token window, so its context has no fixed limit. This PR builds the
foundation for that: the architecture working end to end on one small world,
in English, grown by the loop.

You own this fully: architecture, names, crates, formats, tests and CLI are
yours to decide. The design below is the intended direction, not a spec. If
you find something better, do it and explain why in the final report. No
prototypes: the PR delivers a working system and a working loop.

## The architecture: three layers

1. **Meaning layer (the ontology).** Typed, language-neutral meanings:
   things, kinds, properties, relations, events, actions, quantities,
   comparison, negation, time, questions, reference ("it", "that one", "the
   struct we just changed"). Every sentence the system understands becomes a
   value of these types. No meaning stored as free strings. If a concept has
   no type, the system can't think it. This layer knows neither English nor
   any particular data.
2. **Languages.** Each human language has its own crate: tokenizer, words,
   word forms, grammar, and the generator that turns meanings back into
   sentences. The same grammar should both read and write, if that works out.
   English is the only language in this PR. Its crate must not leak into the
   meaning layer: name meaning types after concepts, not English words, and
   keep English-only distinctions (articles, English's tense set, word order)
   in the English crate. Where languages divide meaning differently, the
   meaning leaves the choice open.
3. **Worlds.** Each kind of data is a world with its own crate. It defines its
   kinds of things and relations (for code: struct, field, function, "calls"),
   how to look things up by name or description, what actions are possible,
   and how to index a source. A language adds its own words for a world's
   kinds; a world never adds grammar.

**The build enforces the separation:** the meaning layer depends on no
language and no world, and the English crate depends on no world. Check it
from the crate dependency graph in CI.

## The first world

**Rust source code.** The user points glassmind at one or more Rust repos and
asks about them: "what calls `spawn_wave`?", "which structs have a `Velocity`
field?", "what does `apply_damage` do?", "rename `hp` to `health` on
`Player`". Start with questions and one or two simple edits. Choose about 3–5
kinds of request that are common, small, and have answers that can be checked
exactly. Keep the world small and honest, like SHRDLU's blocks world, rather
than broad and shallow.

Rust tooling is not language and is allowed in the world: `syn` for parsing
and indexing, `cargo metadata`, `git`. Edits are proposed as a diff, in the
system's own words ("I'll rename `Player::hp` to `health` in 14 places"), and
applied only with `--apply`. It never commits.

## How it talks

- **Input is any text.** It understands it, or says exactly where and why it
  didn't:
  - unknown word;
  - unknown sentence shape;
  - something named that doesn't exist;
  - an action on the wrong kind of thing;
  - an ambiguous reference, listing the candidates.

  When it stops understanding partway, it marks the spot:
  `which structs have ▮the bouncy thing`.
- **Guided input.** The grammar knows what can come next, so the interactive
  mode offers it on Tab, including real names from the world (after "rename",
  the actual items). That requires a parser that works left to right on
  unfinished sentences. Build it that way from the start.
- **Several readings.** Keep every reading that survives the grammar and the
  world's checks:
  - if they all lead to the same result, answer;
  - if they differ on a question, answer using stated preference rules and
    say what was assumed ("By `it` I assume you mean `Player`");
  - if they differ on an edit, ask which one was meant. Never edit on a
    guess.
- **Wrong is worse than "I don't know".** A confident wrong answer costs far
  more than an honest gap, in scoring and in the loop.
- **Conversation and context** are typed data, not a token window:
  - the sources it was given;
  - what has been said, including facts the user stated and corrections;
  - what it did and why, so it can answer "why did you change that?";
  - which source the conversation is about right now.

  Sessions are saved to disk. "Show context" displays all of it, and "forget
  that" removes things.
- **`--explain`** shows the whole trace: the words, the parse, the meaning
  value, how references were resolved, which readings were dropped and why,
  what it did, and how the reply was built. For every rule that fired, it
  also shows which loop round added it.
- **Deterministic:** the same sources, conversation and input give the same
  output, byte for byte.

## Start from nothing

- Don't use or read for ideas the user's language crates (`english`,
  `english-core`, `english-phrase`, `english-ccg`, `church-slavonic`,
  `ruthenian`, anything in `~/code/linguistics/` or `~/code/english/`).
- No outside language data (Wiktionary, WordNet, dictionaries, word lists,
  treebanks, corpora) and no NLP or ML crates.
- **Understanding goes through the grammar.** No keyword spotting, regexes
  over whole requests, fuzzy matching, or one special case per sentence. Only
  the tokenizer reads raw text, and the grammar is built from a small, fixed
  set of building blocks.
- **The seed.** You write a small seed by hand: the tokenizer, the grammar's
  building blocks, the first meaning types, and just enough words and rules
  for one kind of request to work end to end. Mark it as seed. Everything
  after it comes from the loop. Report how big the seed is next to what the
  loop grew; if the seed does most of the work, the loop hasn't been tested.

## Data: generated, task first

There is no fixed corpus. LLMs generate the conversations, working backwards
from tasks so every label is exact:

1. Pick a real task in a real repo, e.g. "find all callers of
   `spawn_wave`" in one of the user's Rust repos at a fixed commit (copy or
   check out repos under `~/.cache/`; never modify anything in `~/code/`).
2. Compute the correct answer with tooling (`syn`, `git`, and rust-analyzer
   where needed), never with glassmind's own code.
3. Have an LLM phrase the request in many styles: terse, chatty, with typos,
   indirect, referring back to earlier turns, inside a longer conversation.

**Guard against "AI English".** If the same kind of model writes the requests
and the system, glassmind can learn to understand only the clean, regular way
LLMs write.
- Use several models and phrasing prompts.
- Keep one generator, and some repos, completely out of training: the
  **held-out set** is written by that generator, about those repos.
- Make the generators imitate how people actually type: short, lowercase,
  missing words, typos.

**No input from the user.** The work needs nothing from the user: no
hand-written requests, no labels, no reviews. Everything is generated and
checked by tooling and agents. The held-out generator is therefore the only
check against "AI English", so keep it as different from the training
generators as you can (another model, different prompts, a different style),
and say plainly in the report that no human-written requests were tested.

## The loop

A real program in the repo that runs as a detached, resumable service. It
alternates two kinds of round:

- **Growth rounds.**
  1. Run the system on the training conversations.
  2. Group the failures (same unknown word, same sentence shape, same missing
     concept).
  3. Start headless coding agents (`claude -p`), each in its own git worktree,
     each with one failure group, a few examples, the relevant slice of the
     grammar and types, and an earlier accepted fix as a worked example.

  Agents may change anything, but they put each piece of knowledge where it
  belongs: English-specific things in the English crate, data-specific things
  in the world, and only language-neutral meaning in the meaning layer.
  Changes to the shared core (grammar building blocks, core meaning types,
  words like "the", "every", "it") run one at a time.
- **Compression rounds.** The codebase is not append-only. Find rules that
  never fire, rules that fire for only one request (memorizing), and
  near-duplicates. Agents merge, generalize and rename, and the loop also
  tries switching each rule off by its ID and deletes it if nothing gets
  worse. Every rule therefore needs a stable ID and must be switchable off at
  runtime.

**Accepting a change.** The loop minimizes failures on the training set plus
a fixed price per unit of code size. Size is counted in syntax tokens:
identifiers count 1 whatever their length, and comments count 0, so names and
docs are free. A refusal costs less the further the request got
(understanding the words but not the sentence is progress). A wrong answer
costs much more than any refusal. A change is accepted only if:

1. it builds, and clippy, tests and the dependency check pass;
2. it lowers that total;
3. **no task that was right or honestly refused becomes wrong;**
4. the held-out score doesn't drop;
5. the change doesn't put language knowledge in as keyword matching or
   special cases, or English or code knowledge into the meaning layer. An LLM
   reviewer can veto a change for this, but never adds to the score.

Every rule and type the loop adds gets a real name and a doc comment written
from its actual uses. Loop agents read this code too, and unnamed code makes
them worse at using it.

**How long:** the user has authorized running the loop as long as it keeps
improving. Don't ask about time or token cost. Stop when the held-out score
hasn't improved for many rounds. If you hit usage limits, wait and resume.
Machine caps, to protect the machine: at most 4 agents at once, cargo with
`-j 8` and `--test-threads 8`. Give loop agents their own Claude Code config
directory (`CLAUDE_CONFIG_DIR`), so their sessions don't mix into the user's
history, and make sure they can't read the held-out set.

## Research

`references/` holds the related work, indexed in `references/README.md`. Read
the index, and the papers that bear on a decision before making it. The most
relevant:

- **SHRDLU**: the closest ancestor, and why it never grew;
- **Grammatical Framework**: one typed meaning layer with a grammar per
  language, and an incremental parser that powers completion;
- **DreamCoder and LILO**: growing code by alternating solving and
  compressing;
- **FunSearch, AlphaEvolve and Voyager**: LLM agents improving code against an
  automatic scorer;
- **Cyc and The Bitter Lesson**: the case against this whole approach.

They are ideas to learn from, not code or data to reuse.

## What the PR delivers

- The three layers, the Rust world, the conversation and context features,
  guided input, `--explain`, and the loop. Plus a CLI with an interactive
  mode and a machine-readable mode.
- The loop actually run for as long as it helps.
- **The main result:** a chart of training and held-out scores against tokens
  spent, with code size on the same timeline. Does the held-out score keep
  rising as the loop spends more, or level off? Say it plainly.
- A short readability check: number of rules, how often each is reused, and
  code size against coverage. Coverage should grow faster than size.

**Build order:**
1. Data generation and the scorer, checked by hand on a sample: everything
   after this is measured with it.
2. The seed, end to end on one kind of request.
3. The loop.

## How you work

- Never write to `/tmp` (it's RAM-backed). Use `~/.cache/claude-tmp/` or a
  gitignored folder in the repo.
- Non-test code must not panic: no `unwrap`, `expect`, indexing or `panic!`,
  enforced with workspace clippy `deny` lints.
- Few, meaningful tests, and no snapshot or exact-output tests (replies
  change as the system grows). One useful property: a sentence the system
  generates from a meaning should parse back to the same meaning. CI runs
  without any generated data: use a small hand-written fixture, and a stubbed
  agent for one loop step.
- Keep a progress log at `~/code/notes/projects/glassmind/progress.md`
  (done, current numbers, decisions and why, next), updated after every
  milestone so the work can resume if interrupted.
- When finished, stop every process, service, tmux session, worktree and
  background shell you started.

## Repo and Git

- `~/code/glassmind/` exists and holds this prompt and `references/`. Its
  `.gitignore` keeps the downloaded papers out of git (most can't be
  redistributed); only `references/README.md` is committed.
- Create `gold-silver-copper/glassmind` on GitHub. Push the skeleton, this
  prompt and CI to `main`, do the work on a branch, open one PR, and merge it
  with `gh pr merge --merge` once CI is green. Then check out the latest
  `main`.
- Don't publish to crates.io or create tags or releases.

## Final report

- Repo URL and PR link.
- The main result chart and its plain answer, and the gap between the
  training generators' score and the held-out generator's score.
- The seed's size against what the loop grew, and the readability numbers.
- How it works, with one real `--explain` trace.
- What limited growth, and what should come next: the next world, the first
  other language, or reading documents.
- How you checked it works in practice, and copy-pasteable commands starting
  with `cd ~/code/glassmind` to:
  - ask a question about a repo;
  - try Tab completion;
  - continue a saved conversation;
  - propose and apply an edit;
  - run one round of the loop.
