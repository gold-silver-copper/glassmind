# Exactness and bounds

For a fixed permitted root child, take the highest incoming edge of each nonroot
node. If acyclic, these independent upper bounds are attained simultaneously.
Otherwise contract the first discovered directed cycle C. An entering edge h->d
has adjusted score s(h,d)-s(parent[d],d); outgoing edges keep their score. Choose
the highest adjusted edge for each contracted endpoint pair and retain original
endpoints. Every expanded tree contributes the same omitted sum of selected C
edges. Replacing exactly the selected incoming edge of the chosen entry node
therefore preserves the objective comparison. Recursive optimality plus expansion
yields an optimal rooted arborescence. Each contraction reduces nodes strictly.

Enumerating each possible unique root child partitions every feasible single-root
tree. Maximum over these exact subproblems is exact. A one-root unconstrained
optimum already attains the unconstrained upper bound and can return immediately.
This is not root-edge removal or a post-decoding repair. Per-edge labels are
maximized before tree decoding because the objective decomposes by labeled arc.
Root edges admit only root label, all other edges exclude it.

Implementation finite input cap129 nodes, <=128 root subproblems, <=127 contraction
levels per subproblem. Simple dense recursive implementation has conservative
O(n^4) time with root enumeration and O(n^3) retained matrices in the worst case;
it does not claim the paper's optimized O(n^2) implementation. No pruning or
beam. Missing edges are Option values rather than infinities. Nonfinite matrix
entries/adjusted scores/total comparisons reject; final inference validates the
complete single-root tree independently. Tie resolution is deterministic ascending
node/label traversal with strict greater comparisons, not a claim of globally
lexicographically minimal optimal trees.

Native tests independently enumerate all (n+1)^n parent assignments for each of
500 generated matrices (100 at each n=1..5). They reject all non-trees, then compare
maximum score, without using contraction to construct the oracle. Integer negative
scores and ties avoid numerical tolerance. A hand-authored two-cycle graph forces
nested contraction; a second graph has four root children in its unconstrained
optimum. Both are checked against exhaustive enumeration. NaN and disconnected
inputs reject. This finite test set supports implementation correctness; the
contraction argument establishes the algorithm's general optimality.

Reference: McDonald, Pereira, Ribarov, Hajič (2005), Non-projective Dependency
Parsing using Spanning Tree Algorithms, Figure3, https://aclanthology.org/H05-1066.pdf.
Paper consulted as algorithm documentation only; no text/data copied into training.
