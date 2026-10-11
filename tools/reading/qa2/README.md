# SQuAD2 public-dev baseline, no new model

The frozen v1 reader was evaluated first on200 questions and then all11873
public SQuAD2.0 DEV questions. Runtime saw context/question only. This is a new
scope and must not share a score series with SQuAD1.1. No model was fitted,
calibrated or changed. [Source/version/license/hashes](source.json),
[complete frozen baseline evidence](baseline.json), [reproduction](baseline.py).

Full11873:433 correct substantive answers,5140 correct expected refusals,
1236 wrong substantive answers and5064 unsupported answerable questions.
Selected substantive precision is25.9437%. Official normalized selected EM is
46.94685%, F147.43594%; raw proposal EM11.74935%, F117.08871%.
There are5928 answerable and5945 unanswerable questions. Always refusing scores
5945 official exact matches, exceeding this reader's5574. Thus overall accuracy
alone cannot justify future work: meaningful substantive coverage must also pass.

One wrong substantive prediction, `A`, normalizes to empty and receives official
credit on an unanswerable item. Official scoring remains unchanged. The separate
`substantive_exact_or_correct_expected_refusal` diagnostic counts5573 correct,
5064 unsupported and1236 wrong;5573 includes5140 correct expected refusals.
This distinction prevents presenting a wrong nonempty prediction as a justified
refusal. All rows and official results are preserved.

The first diagnostic assertion caught this normalization coincidence after both
inference runs completed. Classification was corrected separately and the exact
saved predictions were rescored, with no repeated inference. Reproduction checks
frozen identities and input order before reusing any complete saved predictions;
changed binaries require a new run directory. Raw inputs/predictions/statuses stay
under `WORK/coordinator/qa2-baseline`; the script uses the established local paths.

New external bytes:46,494,161, from pinned SQuAD-explorer revision
`eee5fdbf62f8613a7812b03419e6b29617b74fd1`. TRAIN130319 questions includes43498
unanswerable; DEV11873 includes5945. Both files are CC-BY-SA-4.0. The existing
pinned official scorer and MIT license file were reused. No official test,
example predictions or leaderboard model outputs were downloaded.
[Official dataset description](https://rajpurkar.github.io/SQuAD-explorer/).

Next experiment should use genuine TRAIN unanswerable supervision with frozen,
article-disjoint fit/calibration and explicit substantive-coverage criteria.
It must outperform an always-refuse comparator without disguising refusals as
new answers. Current v1 behavior/model remains the trusted implementation;
all responses are unverified proposals, never automatic stored facts.
