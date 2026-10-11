# Frozen before implementation or dev scoring

Base da96f494d4a6865924cb0aa5eea138b029158fc2. Frozen model references-run/tmp/references/model.json.
H1: context-boundary rules learned from exact TRAIN state/text alignments can
select a verbatim span in the immediately preceding SYSTEM utterance, after
nonnegative USER acceptance, to fill missing schema input text slots.
No router/service/reference weights change. Existing filled slots preserved.

Settings: deterministic SHA256(dialogue_id) mod 5 == 0 calibration split;
remaining train conversations fit. Unicode alphanumeric/apostrophe/hyphen word
tokens plus separate punctuation, maximum 80 tokens per source and 16 tokens
per span, maximum 12 whitespace words. Left/right contexts 1..3 tokens with
BOS/EOS boundaries, no span-internal lexical features. Fit requires >=5 unique
source turns, >=5 distinct values, zero incorrect extracted spans. Calibration
requires >=3 unique source turns and zero incorrect spans. Final ambiguity
refuses all competing distinct values. Only immediately prior SYSTEM source;
new USER consumes eligibility, new SYSTEM replaces it. Negation/correction/
alternative markers refuse (no, not, never, instead, another, different,
don't, do not, or); applied to both user/source, conservative by design.
All train opportunities for schema input text slots on annotated service
frames count, including absent gold slot. No dev calibration or second setting.
Source provenance: source turn index, UTF-8 byte start/end, rule index.

Acceptance: sample >31/91, full >595/1119 full joint turns; multi-service
RideSharing exact states >0/265; preserve baseline-right where possible and
report every added wrong slot claim and all transitions. Unchanged 179/1119/1218
denominators and evaluate.py/compare.py; legacy202/257 exact89.17509727626459%
fuzzy. Four inherited wrong copy events /11 scored slot-turns remain known.
If criteria fail reject honestly; do not relax thresholds or metrics.

H1 rejected BEFORE dev: zero accepted rules (169 considered, 3659 proposed).
TRAIN diagnostic showed boundaries swallowing articles, time ranges and ride
qualifiers. H2 fixed before dev: retain every H1 setting and add a source span
shape constraint: first word starts uppercase or numeric; subsequent words
start uppercase/numeric or are generic connectors {of,the,in,and,on,at,de,des,
le,la,to,for}; internal punctuation limited to apostrophes, hyphens, ampersand,
period and parentheses. No lexical entity feature and no named-place exception.
This is conservative, case-sensitive proposal generation; raw output is still
verbatim. This is the final hypothesis in this batch; no dev-driven changes.
