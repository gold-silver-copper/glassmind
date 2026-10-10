Steering, now that the new direction is under way (prompts/general-semantic-model.md stays the working direction; this sets priorities for the next stretch).

Where things stand: PR #1 merged the foundation at loop round 28 (training 99.2%, validation 98.1%, held-out 72.0%, 59,391 tokens of code, 1,090 rules, 431M agent tokens). You're on `harness`, extending the code world with compiler errors and tests. Good. Keep that, with these priorities:

1. **The training score is saturated; the held-out gap is the real signal.** Training is at 99%, held-out at 72%, and the gap is phrasing style, not unfamiliar repos. More data in the same personas won't move it. The first ML models should attack this gap, and each should be judged by held-out gain:
   - a normalizer (typos, texting style, dropped words);
   - a category tagger;
   - a parse ranker that picks the reading people mean.

   Report, model by model, how much of the 72% → higher each one buys.

2. **Grow the harness, don't hand-write it.** Hand-write only the skeleton: the session, tool execution (read, search, edit, `cargo build`/`test`/`clippy`, `git`), structured compiler errors and test results as world items, and `--explain`. Then make harness tasks a new kind of training data, so the loop grows the rest the way it grew the grammar:
   - **Tasks come from real repos at fixed commits, with outcomes checked by tooling.** Break something and the task is to fix it. Remove a test and the task is to add one. Ask for a rename and check that it compiles and the tests pass.
   - **They're phrased in several styles, with a held-out style and held-out repos.**
   - **Scoring:** a broken edit or failing build costs like a wrong answer; asking or refusing costs little.

   Report how much of the harness the seed wrote and how much the loop grew, as the foundation report did.

3. **One shared trainer for small models, glassmind's own.** Build it once, in glassmind's own crates, in the spirit of the user's flexion (read its source for ideas; never depend on it or its tooling, vendor it, edit it or open PRs on it): data plus a short description go in; a small Rust model with calibrated confidences, a held-out report and an inspectable dump comes out. Prefer cheap, inspectable model families (decision lists or trees, linear and maximum-entropy models, n-grams, averaged perceptrons). Move to anything bigger only when measurement says it's worth it. Self-train on everything glassmind gets right.

4. **The module factory comes second, driven by failures.** Add a source when a failure group needs it, not speculatively. The first cases worth doing:
   - measurement units and quantities as types (show the type system catching a wrong conversion);
   - word forms from glassmind's own inflection model, trained from Wiktionary by glassmind's own trainer (not flexion or its tooling);
   - rustc's error index and the standard library docs as data the harness looks up.

5. **Extend the scorer; don't replace it.** Keep the QA score as it is, and add harness and model scores beside it, so the 28-round history stays comparable. The chart gets one line per task kind, held-out and training, against tokens.

6. **Next milestone: PR #2, when the harness does three kinds of coding task end to end on held-out repos.** For example, fixing a compiler error, renaming across a crate, and making a failing test pass. Each is scored, and the first ML model must show a measured held-out gain. Merge it when CI is green, then keep going toward the rest of `general-semantic-model.md`.

Keep the machine caps and the memory-capped loop slice. Keep the progress log current. Don't ask me anything, and keep going for as long as it improves.
