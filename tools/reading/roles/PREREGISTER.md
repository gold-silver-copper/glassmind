# One frozen anchor-alignment ranker experiment

Before any implementation or DEV evaluation, 2026-10-10.
Base eb22519f20325e72549a103cf99180ea0dbc2ba3; baseline model
b7dc4618b96c26ea9dc91880d1b7644336c9ff253f22bab036583473479f7ed5.
Inspected only first14 original FIT examples: matching content on the wrong side
of a proposed span cannot be represented by v1's symmetric overlap/proximity.
Examples include meninges separating skull from brain, merge-sort inventor,
tourism jobs and seed potato location. No manually encoded answers or predicates.

Hypothesis: a reusable hashed anchor-alignment family improves proposal ranking.
Append to v1 features: conservative suffix-stem content matches; sentence stem
coverage; per matched anchor candidate side and bucketed distance crossed with
question position (first/middle/last), wh, question-head, adjacent auxiliary and
anchor morphology; nearest left/right matched-anchor order; question/head and
span-boundary lexical conjunctions; local question/anchor negation agreement.
Lexical features are hashed general text, never lookup tables. Same65536 weights,
version2 optional Model API. v1 code path remains byte identical. No parser,
pretrained model, embedding, runtime annotation, runtime LLM or new data.

One training run: original exact4000 FIT rows/64 article hashes; original1000 CAL
rows/next16 hashes; five sequential passes; same passive-aggressive averaged
optimizer/rate cap0.1; same top3 sentences/10token candidate bound, 64KiB context,
2048 tokens/2KiB question. No hyperparameter search or dev retuning. JSON<=2MiB.
CAL selection unchanged: maximum coverage with >=50% normalized exact, >=30
accepts, ties grouped. Freeze model hash before first200 SHA(ID), then all10570.
Use official normalized EM/F1 imported unchanged from tools/reading/qa/run.py;
assert all IDs/counts; baseline frozen v1, never weak lexical baseline.

Accept only raw exact>=2638 (>=24.954588%), raw F1>=35.518425%, selected correct
>=815 and selected precision>=53%. Report wrong/refusal/coverage and every raw
and selected transition, especially oldcorrectlost/newcorrect/newwrong.
Any failure rejects and restores runtime while retaining patch/evidence.

Independent probes frozen before training/dev: role reversal subject and object
(Nora thanked Felix / Felix thanked Nora); matched/mismatched predicate
(Nora demolished vs painted bridge; ask when demolished); negation positive vs
never demolished; place vs person answer kind (Mira met Soren in Oslo); numeric
kind (three crates); Unicode (Élodie visited Zürich); unsupported relation/date
(Lena owns bicycle vs bought bicycle in2012). All positive counterparts included.
Report every failure; synthetic probes confer no benchmark or entailment claim.
All outputs unverified proposals; no automatic factual use.

Prior rejected experiment: support's48-feature postselector, distinct expanded
FIT/CAL and90% gate, failed calibration/role/negation. This experiment instead
changes candidate ranking on the original v1 data and retains its gate.
