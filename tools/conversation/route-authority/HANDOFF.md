# Route authority handoff

Status: required behavioral acceptance PASS; uncommitted candidate for parent replay.
All candidate checks completed on the one exact candidate binary. Two runner errors are
retained and explained below; no clean consolidated wrapper execution is claimed.
One bounded runtime candidate, uncommitted. The original social-routing submission
is still REJECTED and remains untouched in the read-only archive.

Base fd01b9bdc2ff68402841577fb3510422e6e47ec6, runtime1331ac7. Only owned
runtime files changed: crates/cli/src/everyday.rs and talk_stories.rs. New adapter
and evidence live here; documentation is docs/conversation-route-authority.md.
No main CLI, shared manifests, loop, scorers, classifier, weights, thresholds,
README, historical prompts or evaluation definitions changed. No commit.

## Exact implementation

Reused only the archived two Rust files after preregistration, including their
read-only StoryPlan / single apply transition and bounded TRAIN-derived social
recognition. Removed the archived blanket veto of learned greeting/goodbye/thanks
proposals that reach ordinary fallback. Typed story mutation/invalidation occurs
before that fallback. Unknown social form alone is not unsafe-assertion evidence.

Retained explicit appended-clause and unconsumed-prefix guards. Supported
coordinated movement can retain ownership when its greeting-shaped leading actor
was already established in world events. This uses story state and the existing
safe event parser, not the proposed label, a name whitelist, or new social words.
Explicit punctuation, unsupported actions, negation and unknown suffixes keep
existing refusal/invalidation behavior. New-actor ambiguity is disclosed below.

## Frozen evidence and results

PLAN.md and boundaries.py were SHA256-frozen before runtime edits. The new18
sessions cover ordinary questions, unknown social question shapes, negation,
appended commands/updates, source turns, correction focus, restart persistence,
single-actor and coordinated movement with an actor named Hello. A fixed subset
ran before each full authored run. Baseline observations preceded implementation;
model baseline completed during the build, with no code changes in response.

Exact baseline: 8cde5dc486025e05932db946ba4accb7b5401fcd27c530bc8539eb4dca5bb78b.
Exact candidate: ec8b6734916d64fc50b163a04dd42dd071f7c2b51168b96dcd44a359de7c5270.
Every candidate executable check uses ../routeauthority-run/glassmind-candidate.
Source was frozen before build; no test-only edit or rebuild happened afterward.
Final everyday.rs SHA256 e2118b62c7aea715c9c80f8f5008abbd95fb0b5a4ee79dd44c496ff37a4502b8;
final talk_stories.rs SHA256 945070eaed3b360bf773b57073a5a8fe81e3edb15ea2d1716477726fb9840a5d.
Runtime patch SHA256 407c93a5c63c29ccef4443a8979f9442d282d2821ecbdc993551a2a051b15a08.
Unit test executables have their own exact hashes in identities.json.

- Original fixed6 social sessions: plain118/126 ->126/126; model106/126 ->126/126.
- Original full490: plain490/490 and model490/490 PASS.
- Original memory93+41: plain134/134 and model134/134 PASS.
- Goodbye supplemental: plain6/6 and model7/7 PASS.
- New independent full boundaries: plain114/116 ->116/116, plus6/6 exact baseline
  question-reply retention checks. Model113/116 ->116/116, plus6/6 exact reply-retention checks.
  Fixed boundary samples: baseline56/56 each; candidate59/59 each (three additional
  exact reply-retention checks). Counts are separated rather than claiming that
  added comparison checks are gains.
- Supplemental unchanged stale-answer gate: baseline0/2 ->candidate2/2.
- Calc27/27, story47/47, factset36/36 +session16/16 PASS.
- Unchanged full3100 MAIN talk diagnostic:1303 ->1305 correct learned proposals;
  all1303 retained, two gains, zero newly wrong accepted proposals. All19 changed
  rows audited offline: two correct social gains and17 refusal-reason-only changes.
  Typed replies are ungraded. All40 inherited wrong accepted proposals retain
  exactly their prior row indices; they remain inherited errors.
- Fixed bAbI story sample400 questions/4847 turns:397correct/3refused, byte-identical.
  Full3987/47625 turns:3966correct/18refused/3wrong, raw bytes identical, SHA256
  8df8189e8022cf436aa1aaf6e0aaeaabced18cd3b01b41faedb4f1d19010b480.
- Scoped release Rust tests:3 everyday +4 story adapter +4 memory PASS.
  Release build, affected all-target clippy -D warnings, fmt --check and diff check
  PASS. No giant suite, no workspace-wide unrelated tests.

The full CLINC audit and full story replay each ran once for this candidate;
no post-dev runtime edits. The exact unchanged coordinator intent-routing.py was
called directly. Public original scorers and frozen expectations were imported
or invoked unchanged, not rewritten. Source and fixture hashes were rechecked.

## Rejected experiments and execution limitations

The original archived candidate was rejected by the parent:1303 ->1302 correct
learned proposals, three losses/two gains. This revision does not rehabilitate or
rewrite that record. No alternate runtime candidates or model tuning were tried.

One launcher raced the unfinished model baseline file and stopped before any
full model boundary case ran. Its traceback and empty output directory are kept;
the missing comparison was resumed after baseline completion. See reports/attempts.md.
This is an orchestration failure, not a failed or repeated behavioral full run.
The model-gate shell also exited2 with an EOF error after all scorer commands
finished successfully: its wrapper was edited in place during the long run to
add configurable output plumbing. Final bash -n passes; all individual reports
and identities independently pass check_gates.py. No full run was repeated.
The raw nonzero wrapper result is retained, not represented as a clean run.
Early baseline-only talk probes omitted an explicit GLASSMIND_SCRATCH export;
they used owned explicit session paths and supplied persistent TMPDIR, and never
ran generated-code builds. All cargo and subsequent gates exported owned disk
scratch. This deviation is retained rather than obscured.

## Data provenance

Zero downloads/new model calls/training/credentials/API keys/agents/services.
The enclosing systemd scope verified MemoryMax8589934592, MemorySwapMax0.
Every cargo invocation explicitly set supplied persistent TMPDIR, used the existing
CARGO_TARGET_DIR and systemd-run --user --scope -pMemoryMax=8G -pMemorySwapMax=0
setsid --wait timeout --kill-after=10s 600; -j1 except fmt, tests --test-threads 4.
All owned jobs are complete; no agents, services or background jobs remain.

CLINC150, Larson et al.2019, clinc/oos-eval revision
828f8093932c8fe6ca7936c3d2e52903b1c523de, CC-BY-3.0: existing extracted
TRAIN15000/1,165,484 bytes; VAL3100/239,360 bytes. Original TRAIN evidence was
read before editing; no DEV-derived vocabulary or model threshold was introduced.
No upstream test file or held-out item was read. SHA256s:
TRAIN f27778e9c16448d040d9c364a1b369493711865419613a277270a54781ea87c2;
VAL39afa094ba8495e9b8b30595394937bdd7b5b8e6dcdba71a9a380047759c88f9.
Production intent model:25,524,518 bytes, SHA256
 a84fd59c3e49db08300f61ee33f6d73e64a8ea18109102b01685333b95936408.
bAbI Facebook tasks1–20 v1.2 English1k, CC-BY-3.0:20 TRAIN members,
2,947,923 bytes. Existing public tools/stories/babi.py dev episode split,
normalization and scorer unchanged. All20 member hashes verified against the
original archive and retained in verified-inputs.json. No official test members.
New independent authored boundary contracts are embedded in boundaries.py,
version1,3,747 bytes, repository MIT OR Apache-2.0; exact hash in artifacts.json.
No new external dataset. Derived raw CLINC records retain the same provenance.

## Reproduction and identities

Run from this worktree with the same read-only dataset/archive mounts:

```bash
export ROUTE_REPORTS=/home/gsc/.cache/glassmind/continuous-20261010/routeauthority-run/fresh-replay
bash tools/conversation/route-authority/reproduce.sh
```

The output directory must not exist. reproduce.sh assembles the executed commands
in dependency order; the consolidated script was not redundantly rerun. Actual
runs overlapped independent gates. validate.sh records each capped cargo command;
gates.sh contains exact direct scorer commands. reports/*log retain outputs,
reports/runtime.patch and source.sha256 freeze runtime source; identities.json
records all binaries, unit harnesses and relevant evaluator identities.
verified-inputs.json records original model/data/archive checksums. artifacts.json
indexes owned evidence. The command to replay just the unchanged full routing audit:

```bash
python3 ../coordinator/socialrouting-review/intent-routing.py ../routeauthority-run/glassmind-candidate NEW_OUTPUT --baseline ../coordinator/socialrouting-review/intent-baseline
```

Limitations: this is finite social routing and retention of learned proposals,
not general semantic correctness or raw classifier accuracy. Unknown assertions
can still be conservatively refused. A newly introduced greeting-named actor in
coordinated movement remains ambiguous and can refuse; its recorded baseline
acceptance is not relabeled unsafe. Previously established actors are preserved.
Full ordinary facts2821 and independent integrated replay belong to the parent
and were not run here. No publishing or deployment is claimed.
