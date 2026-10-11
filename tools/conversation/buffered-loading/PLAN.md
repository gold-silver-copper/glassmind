# Buffer JSON loading without changing models or routing

Fresh-process discovery on combined runtime c751ebb / f16beba7: five interleaved
runs of `/reset` then `Hello` have median startup 0.00191s without the production
model and 2.61433s with it. Output bytes are identical for that literal greeting.
Concurrent development jobs were active; this is an exploratory measurement,
not an isolated benchmark or a claimed speedup yet. Exact timings and model hash
are in the parent baseline artifact.

The main CLI's generic load helper passes File directly to serde_json::from_reader,
while the standalone everyday CLI already uses BufReader. Hypothesis: wrapping
the main reader in BufReader removes per-byte file reads, substantially reducing
startup latency while preserving the JSON parser, models, validation and routing.
No new format, cache, training, threshold, dependency or serialization semantics.

Acceptance: materially lower fresh-process median in the same interleaved probe,
identical production-model values/proposals and full3100 main-talk replies versus
the frozen pre-buffering binary, unchanged malformed/missing-model rejection and
session loading, and all integrated conversation gates on the final actual binary.
The already-complete full765 coding result is unaffected by this talk-local loader.
Full story3987 and ordinary facts2821 outputs must remain byte-identical. Keep
all old candidate and partial run evidence. The unbuffered parent model session
run may be explicitly interrupted and replaced by full final-binary replay; never
claim its unfinished checks passed. Source manifests and exact evaluated commit
will distinguish the binaries. All cargo retains existing caps and disk TMPDIR.
