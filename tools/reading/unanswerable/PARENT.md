# Independent review: rejected before candidate DEV

The parent independently reconstructed the 80 excluded article titles, the
96/32 FIT/CAL article selections, all 8,000/2,000 selected rows and their exact
context/question-only inputs. No title, context or question ID overlaps the
other partitions. All 29 provenance files were rehashed, and the rejected patch
reconstructed all eight frozen source, model and experiment artifacts.

Using the retained first-run weights and features, the parent recomputed every
official CAL span grade and all 1,975 threshold groups. No threshold passes.
Best precision retaining 100 correct answers is 100/220; at 60% precision there
are at most 48 correct answers. No candidate DEV run occurred. All 11,873 default
v1 outputs match the retained baseline byte-for-byte. [Full parent evidence](reports/parent-review.json).

The original 23-file submission (1,055,394 bytes) was archived before review.
No runtime changes are integrated. Failed weights use an unselected placeholder
threshold and must not be deployed. Predicate, role and negation failures remain.

The original audit expects its checkout beside the retained run directories.
For a saved-output audit after removal of the worker checkout, create a fresh
disposable checkout of its recorded source and copy this evidence directory:

```sh
WORK=/home/gsc/.cache/glassmind/continuous-20261010
git worktree add --detach "$WORK/unanswerable-audit" 80b9f636050accfbe6ca0fbdb0ecc2bc6ca531d9
cp -a tools/reading/unanswerable "$WORK/unanswerable-audit/tools/reading/"
(cd "$WORK/unanswerable-audit" && PYTHONDONTWRITEBYTECODE=1 python tools/reading/unanswerable/audit.py)
```

This reuses saved predictions; it does not retrain or repeat inference. The
original `reproduce.sh` is an experiment runner requiring the rejected patch in
an isolated checkout and a fresh run directory, not the saved-output audit.
