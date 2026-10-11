# Public-pilot QA: calibration complete, replacement rejected

Frozen plan366b058; evaluator7645195. Full1949-row CAL completed in6.542s.
Strict baseline2/1863/84 vs publicpilot2/1940/7 (right/refused/wrong).
Zero net correct gain;7 newly wrong cases;2 old correct answers safely refused.
Gate fails. No DEV inference, model/index edits, downloads or new model workers.
Read PARENT.md and summary.json. All original per-row artifacts remain outside git.

Correct anchor links153->843; native eligible alias coverage1946/1949. Offline
annotated-relation sets:1642 missing,296 nonempty withoutgold,6 uniquegold,2
nonunique includinggold. All7 newly wrong cases have correct anchors/relations
but a different local singleton inverse answer. The2 lost answers lackforward
claims in the firstshard. This confirms the claim-coverage bottleneck.

WORK=/home/gsc/.cache/glassmind/continuous-20261010.
Run WORK/coordinator/wikidata-qa; source worktree WORK/wikidata-qa.
Session14150 COMPLETE; no workers/auth mounts. All input hashes passed.
Runtime remains PR49, merged663819d70f9bd2f220ba4d5452d4665597b2639e,
reviewed9e5bdcc; CI38159805008 passed94s. Public graph remains optional/read-only.

Next inspect the retained immutable claims metadata and design bounded broader
claim ingestion, reusing the native name index. Do not select sources using CAL
IDs or fetch other85 files until the new acquisition/resource plan is frozen.
Do not repeat this failed QA run or advance it to DEV without a new candidate.
