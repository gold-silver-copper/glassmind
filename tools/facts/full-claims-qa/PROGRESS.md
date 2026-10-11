# QA calibration complete; promotion rejected

WORK=/home/gsc/.cache/glassmind/continuous-20261010.
Worktree WORK/full-claims-qa, branch continuous/full-claims-qa.
RUN=WORK/coordinator/full-claims-qa. Plan6fa9581, runnerc22d730 frozen before run.
Session85868 COMPLETE exit0; scopeglassmind-full-claims-qa terminal. 36.008seconds,
103604KiBprocessRSS,32587776sampledpeakbytes. Frozen resource gates pass.

Strict CAL baseline2right/1863refused/84wrong; candidate554/1365/30. Net552,
29newwrong fromrefused, no lostright => promotion REJECTED. No DEV or default
promotion. Wrong categories12anchor/7relation/10source-answerdisagreement.
See PARENT.md, summary.json and immutable original run artifacts.
NEXT integrate reporting with a separately labelled CAL score series. Then freeze
an entity-mention localization experiment on allowed TRAIN-FIT using this source;
no threshold tuning or relaxed retry of the rejected promotion gate.
