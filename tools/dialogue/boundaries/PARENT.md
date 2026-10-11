# Independent boundary integration

Evaluated source `38ef238c560d11f0627adb14a946ceb935bc4288`, including the
production dialogue changes and a reproduction-path correction. Original worker
submission is archived with per-file SHA256 hashes under
`~/.cache/glassmind/continuous-20261010/boundaries-run/submitted/`.

The coordinator independently fit the boundary rules again from the retained
TRAIN data, ran the fixed first sample and full dev, audited changed claims,
replayed both the frozen multi-service baseline and legacy single-service model,
and verified source provenance and inference isolation. Eight dialogue unit tests
and workspace fmt, all-target clippy, release build and strict 4/4 fixture smoke
passed. No evaluator changed and no held-out evaluation ran.

Full dev confirms **610 right, 459 wrong, 50 refused / 1,119 USER turns**, from
609/460/50; exact frames improve 712 to 716 / 1,218. Every previously correct
joint turn and frame remains correct. Four scored slot-turns from two incorrect
copies are fixed, with no new wrong scored claims. Six other changed held states
are unscored. The remaining two inherited events still produce seven wrong
scored slot-turns. Routing, intents and offers are unchanged. This does not fix
source-state truncation or semantic freshness.

Independent model, boundary weights and full prediction bytes exactly match the
worker's candidate. Frozen baseline and legacy prediction bytes are also
unchanged; the legacy score remains 202/257 exact and 89.17509727626459% fuzzy.
Reversed conversation order is identical and annotation-bearing input is rejected.

| Artifact | SHA256 |
|---|---|
| Model | `fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88` |
| Boundary weights | `e807be35af0d5264d687279b71db1c6d2f4f48edb5d4238f2b87f6b0ea5a5e01` |
| Full predictions | `8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d` |

[Integrated artifacts](integrated/artifacts.json), [full report](integrated/full.json),
[transitions](integrated/transitions.json), and [claim audit](integrated/audit.json)
retain exact identities and outcomes. Training uses gold TRAIN endpoints with
fixed fit/calibration conversations, not predicted-source calibration. The
USER+SYSTEM/all-TRAIN-schema amendment was recorded before dev evaluation.

Reproduce with `reproduce.sh` and persistent TMPDIR/CARGO_TARGET_DIR as described
in [the handoff](HANDOFF.md). The parent ran the workspace gate first, then an
otherwise identical copy of that script excluding its already-completed
fmt/clippy/build steps. Exact script and logs remain at
`~/.cache/glassmind/continuous-20261010/coordinator/boundaries-check/`.
