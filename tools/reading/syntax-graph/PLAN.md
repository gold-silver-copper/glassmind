# Frozen graph experiment
Hypothesis: globally trained first-order labeled arc scores with exact single-root
nonprojective decoding improve LAS >=19673 and UAS >=20540 /25148, all2001 valid,
zero refusals, and preserve independent parent core edges. Baseline19423/20540;
rejected raw beam17879, normalized19428, children19443 LAS. No second fit/tuning.

Model ewt-labeled-graph-v2: exactly2^21 scalar hash weights, existing FNV/mixing
function, namespace2000 unlabeled and3000+label*64 labeled. Approved v1 tagger
payload and sorted tags/labels copied exactly. Three ordered TRAIN epochs only.
One sentence step (including correct sentences); gold-minus-predicted whole-tree
unit update, average pre-update weight vectors across all sentence steps, f64
accumulation then f32 final weights. No loss augmentation, shuffling or pruning.
All valid <=128-token TRAIN trees including nonprojective are eligible.

Arc features: word=lowercase FNV; tag=predicted tag+2, root word/tag=1,
boundary=0; direction=head<dependent; distance buckets1,2,3,4,5(5..7),6(8..15),7(16+).
Unlabeled inventory ordered: bias; htag; dtag; hword; dword; htag,dtag;
hword,dword; hword,dtag; htag,dword; hword,htag; dword,dtag;
then each preceding feature duplicated with direction,distance suffix;
then hleft,htag,dtag; hright,htag,dtag; htag,dleft,dtag;
htag,dright,dtag; hleft,htag,dleft,dtag; htag,hright,dtag,dright;
all six context features carry direction,distance.
Labeled inventory: bias; htag; dtag; htag,dtag; hword; dword;
hword,dtag; htag,dword; htag,dtag,direction; htag,dtag,direction,distance;
hword,direction; dword,direction. Sum shared unlabeled plus labeled scores.
Only root label on root arc, never root label elsewhere.

Decoder: finite f64 matrices <=129 nodes; missing edges explicit Option.
Chu-Liu-Edmonds recursively contracts first discovered cycle, adjusts entering
edge by subtracting selected incoming score; stores original endpoints for exact
expansion. Ascending nodes/head enumeration; strict greater comparisons retain
first tie. Enumerate root child1..n, constrain only that root edge, keep first
best total. Unconstrained one-root optimum can return immediately (already exact).
Validate final tree; reject nonfinite input/arithmetic, no invalid fallback.

Before DEV freeze source/model/binary/probes/adapter/unchanged scorer/hashes.
Exactly first100 then full2001; duplicates retained. Full edge/sentence transitions,
raw validity/spans, v1 supplied-output parity, model roundtrips/corruption/caps,
exhaustive small-matrix decoder comparisons, fmt/clippy/release. No hidden/TEST
access, downloads, agents, runtime LLM, commits or shared production integration.
Reference: McDonald et al2005 https://aclanthology.org/H05-1066.pdf algorithm3;
single-root enumeration is our explicit additional exact constraint.
