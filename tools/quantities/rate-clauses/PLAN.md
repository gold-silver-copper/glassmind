# Frozen plan, before implementation
Hypothesis: a bounded clause grammar feeding typed agent/group, work reference,
completion fact, simultaneous condition and query slots can compose independent
facts in either order without whole-question templates. Rate solver supports 2–3
agents and one inverse unknown; all facts refer to exactly one job/tank. Explicit
empty/full state sets tank workload/direction. Constant independent rates are a
stated modeling assumption, never a claim about arbitrary physical systems.
Baseline source bb89964, runtime unchanged from408d16b; v3 full4475 29 correct,
4443 refused,3 wrong,32 answered payloads to preserve byte-for-byte.
Acceptance >=34 correct, no new wrong/unscorable, all old payloads unchanged;
synthetic3116 and linear50/absolute55/percentchange38/parent78 preserved. Parent
unseen acceptance >=8 positives and all negatives safe is separate and pending.
Freeze authored cases before code, candidate/source/binary/adapter before DEV;
first100 then full4475 once. Inference reads Problem only, labels loaded after
inference. No post-DEV retuning. Independently audit every new answer.
Unsupported stages, quantities/capacities, initial partial volume, rate changes,
extra operations, unknown agents, conflicts, missing operands must refuse.
Rejected prior WORKRATES:33 correct <34, independent0/17 positives; sentence
patterns rejected. No previous probe list used. No downloads/model calls/agents.
Only TRAIN Problem inspected; no source card or other TRAIN columns read.
New authored values computed by rational reciprocal arithmetic, independent of
runtime. Tests include multiple realizations, reordered facts and negative scope.
If gates fail, archive exact candidate patch/source/binary/evidence with SHA256
before rollback; retain all failed attempts. One bounded batch, no tuning on DEV.
