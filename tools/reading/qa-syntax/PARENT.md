# Rejected: dependency features improve DEV scores but fail independent negation

The coordinator rejects this candidate. The frozen rule required no newly
selected clear predicate/negation mismatch. On a separately authored probe frozen
before inspecting the candidate, `Ava never hired Milo.` / `Whom did Ava hire?`
changes from refusal to selecting `Milo`. This is a direct negation failure.
The original worker's gate claim covers its own probes only; its handoff is
preserved unchanged and does not override this decision. No runtime source,
model, dependency, default behavior or main command is integrated.

Twenty independent probes contain 13 answerable and seven unsupported questions.
Raw correct positive proposals increase from 10 to 11; selected positive answers
remain six. Wrong selections remain four, but unsupported selections increase
from three to four. The conditional `If Zara wins, Omar receives a medal.` also
changes from refusal to answering who received a medal. The previous unsupported
`Nobody knows who opened the box.` selection becomes a refusal. Every original
input, prediction, reason and expected answer is retained in [parent](parent/).
These authored diagnostics establish concrete failures, not general accuracy.

An independent scorer recomputed all 10,570 retained official DEV predictions
and the complete TRAIN calibration frontier without importing the worker scorer.
Raw exact answers improve 2,532 to 3,057 and raw F1 34.518425% to 39.869573%.
Selected correct answers improve 815 to 943; selected wrong decrease 791 to 647;
refusals change 8,964 to 8,980. The aggregate gates pass. However, four previous
correct selections become wrong, 334 become refusals, and 476 old refusals become
wrong selections. All transitions agree with the original ledger. CAL chooses
166 selections, 83 correct, at margin 0.20622891187667847 with complete tie groups.
These scores are evidence for a representation, not permission to relax the gate.

The parent verified all 27 archived entries (227,761 bytes), 88 provenance
identities and 29 pre-DEV frozen identities. The archive includes the original
runtime patch, dependency changes, reports and source. The 20 native parent
predictions used evaluated binary `b0ad5d9655fe3f4b97804150446bca60761f8d406028be131424b398aa17c277`
and model `7cd2afb5fbb726f2bf09d98e54ae0e98807b178a7e49dcf85513be1f396315ad`.
All outputs remain explicitly unverified. Full inference was not needlessly
repeated; the independent audit reads frozen outputs and rederives their scores.
The original v1 full replay is byte-identical to its approved baseline.
Worker build/fmt/clippy, 14 tests and ten contracts passed; these do not repair
the semantic failure. Parent runtime is unchanged, so no new workspace/full
coding score is claimed. Original failed clippy output remains archived.

Raw data, models and evaluated binaries remain at
`/home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run`.
The original [preregistration](PREREGISTER.md), [handoff](HANDOFF.md),
[rejected patch](rejected.patch) and [independent full audit](parent/full-audit.json)
are retained. Original manifests contain worker paths; after worktree removal,
repository files map to `qasyntax-run/submitted`, preserving their exact bytes.
To reproduce the rejected implementation, apply the patch to a disposable
checkout at `078578b0ac350a1b3baa5af20b1a37d21d576d23` and follow the original
handoff with the retained baseline/data. The current build intentionally has no
`reading-qa-syntax` executable. Do not rerun training to audit this rejection.

Run the independent full audit from the repository root:

```sh
python3 -B tools/reading/qa-syntax/parent/full-audit.py \
  --source /home/gsc/.cache/glassmind/continuous-20261010/readingqa-source \
  --run /home/gsc/.cache/glassmind/continuous-20261010/qasyntax-run \
  --out /home/gsc/.cache/glassmind/continuous-20261010/coordinator/qasyntax-independent/full-audit.json
```

No new data was downloaded. Reused SQuAD TRAIN/DEV are 35,142,551 bytes at
revision `eee5fdbf62f8613a7812b03419e6b29617b74fd1`, CC-BY-SA-4.0; scorer MIT.
The frozen syntax model derives from UD English EWT r2.18 TRAIN, revision
`b7711cce01cdd4f5fcc0a8199b8a50d951b16c0c`, CC-BY-SA-4.0.
Authored parent probes and audit code are MIT. No official held-out data was used.
Further reading work needs an independently validated support/answerability
representation; no post-DEV threshold adjustment or hardcoded negation rescue.
