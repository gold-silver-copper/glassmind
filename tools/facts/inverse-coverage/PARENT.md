# Coordinator review: rejected partial inverse graph

The candidate adds 1,817 new claims and 1,735 aliasless nodes but leaves every
one of 2,821 standalone predictions byte-identical. Answer-only scores remain
47 right / 2,712 refused / 62 wrong; strict entity/relation/answer agreement
remains 2 / 2,712 / 107. The original minimum of seven joint-correct answers
was not met. No new bundle becomes a default or an accepted replacement.

Acquisition returned 18 sets of 101 subjects, all truncated, before request 19
timed out. The remaining 81 frozen pairs were not queried. The coordinator
verified 19 request starts, at least five seconds apart, 291.154209 seconds
elapsed and 1,117,488 decompressed bytes. Both earlier local parser failures
(case in statement IDs and an assumed lexical ordering) are preserved. No
network request was repeated during coordinator review.

Independent checks verified 130 artifact identities, all 20 frozen identities,
and the complete TRAIN-only selection. An independent reconstruction from raw
statement/property/target/rank bindings reproduced the candidate graph exactly.
All 999 original records remain unchanged; every new node has empty aliases.
The 1,818 witnessed edges include one already present claim.

The coordinator reran all seven offline tests, independently recomputed sample
and full answer-only/joint totals from frozen predictions, and verified all
standalone/talk answer-ID agreements and byte-identical relocation outputs.
No model inference or network fetch was repeated. Original ten-file evidence
(77,808 bytes) was archived and checked against both copies before removing the
restored worktree. The scope is inactive and the login link is removed.
See [parent-review.json](parent-review.json) and [parent-audit.py](parent-audit.py).
The audit can resolve removed worker-owned files from its preserved submission
and original Rust sources from Git at the recorded baseline commit.

The [post-evaluation coverage diagnostic](coverage-diagnostic.json) explains
why these added edges cannot improve singleton answers: all 18 queried pairs
already had between two and 67 matching subjects. They match 80 development
predictions, 76 already refused for multiple answers and four for uncertain
relation selection. Adding subjects cannot make these existing sets unique.
This diagnostic did not change selection, scoring or the rejection decision.

The broader incoming-edge hypothesis remains incomplete. These are primary
endpoint graph snapshots with truncation, absent revision identifiers and
unresolved temporal/qualifier semantics; they do not establish global coverage
or factual uniqueness. A separate explicit set-answer capability is a new
product hypothesis, not a reinterpretation of this failed benchmark gate.
