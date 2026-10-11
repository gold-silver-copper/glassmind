# Frozen experiment: reference token endpoint completion

Base d59cb121d0c6e0276141283c872e9cd25cb3275e. Frozen before runtime edits,
rule fitting, actual TRAIN prediction, or any DEV evaluation.

Hypothesis: shape-supported extension by one or two adjacent tokens can repair
a truncated existing reference-copy value. This does not repair all-O source
freshness or establish referent correctness. Baseline supplied: 610 correct,
459 wrong, 50 refused /1119 USER turns; 716/1218 frames, 179 conversations.
Frozen v2 model SHA256 fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88.

Initial bounded TRAIN inspection: 3174 conversations, 34022 USER and 34022
SYSTEM turns; 12037 USER and 19309 SYSTEM annotated noncategorical input spans;
24245 multiword spans across both speakers. SHA256(id) integer mod5==0 gives
642 CAL and 2532 FIT. Use all TRAIN-schema noncategorical input slots, all
existing USER/SYSTEM spans in the selected conversations. No other data.

## One fixed representation and algorithm

Tokens: Unicode alphanumeric runs, permitting an internal apostrophe, curly
apostrophe, hyphen or period only when followed by another alphanumeric run.
Underscore is a separator. Shape encodes maximal upper/lower/uncased-letter/digit
runs as U/L/A/D, each run length capped at 3; internal punctuation is literal.
Key is the last TWO source-token shapes (left padded EMPTY), following TWO
token shapes (right padded END), plus the two intervening gaps. Gaps encode
whitespace as W and literal punctuation; runs of whitespace collapse; max 3
gap symbols. Each shape max64 bytes; utterance max4096 UTF8 bytes /256 tokens;
source max12 tokens. No words, entity names, service IDs, slot names, lexicons,
or dev-specific conditions in keys. No alternate representation search.

FIT gold spans may generate candidate keys: remove exactly the last one or two
tokens from a span with aligned token endpoints, retaining a nonempty prefix.
The resulting value must locate this exact occurrence as the latest whole exact
occurrence in its history. This is only candidate discovery, not acceptance.
Conflicting extension lengths for one key reject that key permanently.

Audit actual frozen reference proposals on ALL selected TRAIN conversations:
replay speaker/utterance-only inputs through the frozen MultiModel; snapshot
prior states before each USER; rerun unchanged individual service advance on
that prior state to identify missing target slots; call unchanged reference
resolver with actual prior predicted states/history, only for routed services.
This duplicates the existing reference call site without changing inference.
Existing punctuation restoration has priority: if it fires, token completion
does not run. Locate latest case-sensitive whole exact occurrence; never try
an older occurrence when this one has unknown context or no matching rule.
Candidate extends to the end of token 1 or 2 following the original value,
as one contiguous source slice. A missing/incorrect/ambiguous annotation is
an endpoint mismatch. Exact supervision requires a source-service/source-slot
annotation at the same source turn/start and exactly the proposed end; all
available matching source annotations must agree. Span provenance is not
referent correctness. Source annotations enter only offline audit.

Every generated actual proposal is retained in the audit, including mismatches
and missing annotations. FIT-discovery rows cannot count as actual CAL rows.
Each accepted rule requires >=5 distinct actual FIT conversations and >=3
distinct actual CAL conversations, zero endpoint mismatches in both splits,
and collectively >=30 distinct CAL proposal instances after those rule gates.
No gate loosening, alternate keys, fallback thresholds or whole-TRAIN refit.
Existing classifiers used all TRAIN including CAL: this is empirical validation
of new rules, NOT independent probability calibration.

Independent contrasts before gate reporting: unseen multiword names and Unicode
byte slices; one/two-token suffixes; punctuation; latest ambiguous occurrence;
newer failed occurrence preventing older fallback; case/whole-token negatives;
conflicting/missing annotations counted wrong; exact already-complete value.
These test the offline mechanism, not trained-model generalization.

If TRAIN gate fails: reject before DEV, retain candidates/raw predictions/audit
and exact source/model hashes. Restore every owned runtime edit. No need to
re-evaluate unchanged production inference. Record all seven inherited errors
from provided audit as unchanged, including distinction from held states.

If gate passes: implement optional bounded validated version3 token model with
genuine v1/v2 compatibility, freeze model/source, run unchanged evaluate.py and
compare.py in existing metrics venv: fixed first10 then full179. Preserve all610
correct joint, >=716 frames, zero newly wrong changed assertions, and strict
joint/frame gain or correction of an old wrong assertion. Audit every changed
assertion and all seven inherited errors. No post-full patch/tuning. Test reset,
reversed order, annotation rejection, all-v2 optional-absent byte identity and
legacy45 byte identity (202/257). Runtime accepts speaker/utterance only.

Resources: no new dependencies/downloads/agents/services/LLM calls/commits;
all native cargo/train jobs scoped 8GiB/0swap, setsid --wait, timeout600/kill10,
cargo -j1 except fmt, tests --test-threads4. Existing target and own on-disk
TMPDIR/GLASSMIND_SCRATCH. One bounded experiment, approximately20 minutes.
