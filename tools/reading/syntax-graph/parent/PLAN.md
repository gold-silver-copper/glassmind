# Frozen graph parser parent contracts
24 independently authored sentences/52 core labeled edges: modifiers, questions,
negation, auxiliaries, passives, coordination, relative clauses and Unicode names.
No old-correct core edge may be lost; full fixed corpus LAS>=19673 and UAS>=20540
remain mandatory independently. Forms/text inference must emit valid bounded
single-root trees and exact source spans. Semantic labels remain predictions.
Twelve small arbitrary score matrices have independently enumerated exact maximum
single-root tree scores, including negative scores and ties. Candidate decoder
must match the optimum with legal trees; do not require a particular tied tree.
Full DEV retained outputs will be independently rescored with unchanged audit,
including all2001 sentences/25148 words and68 duplicate sentences/137 words.
Legacy model/forms/raw behavior must remain byte-identical. No parent cases are
available to worker. Decoder adapter can change interface only after source review;
fixed expectations and acceptance gates cannot change.
