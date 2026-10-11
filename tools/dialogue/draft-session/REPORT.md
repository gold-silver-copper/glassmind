# Resumable provisional dialogue draft handoff

Implemented on source commit `49340ae528f1ea542ca5f4794bbc6cb8fe2cf670`, uncommitted.
Only `crates/dialogue`, this new tools directory, and the permitted Cargo.lock
entry changed. No model, runtime inference, policy, legacy binary source, main
CLI, scorer, shared manifest, or other crate changed. No downloads, fitting,
model calls, credentials, labels, held-out material, or external actions.

## Changes and parent API

`glassmind_dialogue::draft::DialogueBundle::load(directory)` reads fixed
`manifest.json`, `model.json`, `schema.json`; validates raw SHA256 hashes and
existing `MultiModel::validate`; retains private immutable model/schema in Arc.
The strict manifest contains exactly `version` (1), `model_sha256`,
`schema_sha256`, `source`, `license`, `scope`. Provenance strings must be
nonblank. Identity is SHA256 of UTF8
`glassmind-dialogue-bundle\n1\n<model_sha256>\n<schema_sha256>\n`.
Whitespace changes in either asset invalidate old history. Metadata-only
manifest formatting/provenance changes do not change identity. Hash checking
is content identification, not a signature or independent provenance proof.

`bundle.new_session()` needs no history. `append_user`, `append_system`, or
`append(InputTurn)` validate before mutation and advance the unchanged runtime
once. `snapshot()` is read-only, explicitly provisional and includes provenance,
identity, total/user counters, last speaker, and `draft`. `draft` is absent for
empty history and otherwise has the exact old prediction JSON shape, including
all inactive frames and offer/boundary source proposals. After SYSTEM it describes
current context/state; `snapshot.user_prediction()` returns None, so main should
only present a new USER prediction when this accessor returns Some.

`serialize_history()` returns bytes with only `{version,bundle_identity,turns}`.
`bundle.resume_json(&bytes)` validates all history before bounded replay;
`session.restore_json(&bytes)` atomically replaces an existing session only on
success. Predicted frames, slots, actions, annotations and other extra fields
cannot enter this history. `history()` exposes an immutable typed slice.
`reset()` clears session history/context/counters while keeping the shared bundle.
No file accesses/model parsing occur on append, show, reset or replay.
Parent owns persistence file writing and explicit main commands.

`crates/dialogue/examples/draft-session.rs` is a small native JSONL exercise:

```json
{"session":"a","command":{"op":"user","utterance":"Find a place"}}
{"session":"a","command":{"op":"system","utterance":"Go to Alpha?"}}
{"session":"a","command":{"op":"show"}}
{"session":"a","command":{"op":"save"}}
{"session":"a","command":{"op":"reset"}}
```

Restore uses `{"session":"a","command":{"op":"restore","history":SAVED_OBJECT}}`.
The example emits `{ok,result}` or `{ok,error}`, supports independent session
names, and exits at EOF. It is an exercise process, not a service.

## Measured validation

Before implementation: all six supplied asset hashes/byte counts verified;
old native first10 **91/91** full-prediction agreement and full179 **1119/1119**.
Frozen PLAN includes hypothesis, acceptance criterion, and authored contract spec.

After implementation, first10 then full179 API comparison passed every field:

- **1119/1119** USER payloads across **179** conversations.
- **179/179** midpoint resumes, all after SYSTEM, **517** subsequent USER outputs.
- Reverse conversation order: **1119/1119**; ten interleaved sessions: **91/91**.
- First conversation restarted at each of **20** boundaries, snapshot-identical.
- Compared **2266 inactive frames**, **11 offer proposals**, **2 boundary proposals**;
  one offer proposal refers to the SYSTEM turn immediately before a midpoint save.
- Authored UTF8 offer case explicitly preserves `École des Arts`, source turn 1,
  and output turn 2 after resume, with another session independently offering Beta.
- **14 Rust tests pass** (8 existing + 6 new grouped contract tests), including
  malformed/unknown fields, corrupt/missing/changed assets, actual 128/129-turn
  and 65536/65537-byte boundaries, UTF8 4096/4097, rejected-operation atomicity.
- Package fmt check, all-target clippy with warnings denied, release bins/examples pass.
- Rebuilt legacy `dialogue-multi` full prediction file is **405228 bytes**, byte-for-byte
  identical to frozen baseline SHA256
  `8887d88afbf18462d10a70574be030df8a6d33071a5c201fa84dc1f99efacc8d`.

Single local full driver run: 5.440 seconds for 10323 mixed JSONL requests,
including model load, parsing, IPC, show/save/reset, prediction and replay.
Mixed request median 0.0151 ms; maximum 164.99 ms includes initial load. These
are practical observations, **not isolated inference latency or a speedup claim**.
Largest midpoint history observed: 1465 JSON bytes (Python UTF8 encoding).

No semantic accuracy measured or improved: retain the existing limited joint
**610 right / 459 wrong / 50 refused of 1119**. Agreement with old predictions
is solely a compatibility result. No rejected implementation experiments.
One baseline preparation filename collision overwrote its already-consumed
first10 input; distinct input/output names were restored and baseline repeated
before implementation. Original issue and correction are in `failures.txt`.

## Source, license, bytes, limits

No new external data. Reused supplied SGD revision
`e852981ae34990f4358979625854259302feaa78`, declared **CC-BY-SA-4.0**:
model **32174720** bytes, schema **3707**, utterance-only DEV input **199152**,
frozen predictions **405228**, license **20139**. Supplied old executable
**16840656** bytes. Six exact SHA256 digests are in `source-checks.json`.
Model SHA `fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88`;
schema SHA `b021ae923dbeb7782b99ac6628e65b1b48f3cca2892fb62ce8c77b472324e089`;
input SHA `bb74aadd451f1e4ff709c27ad2c6c6502274a9fc8259479bd34b718576b13004`.
Authored Rust fixtures are synthetic contract data under the crate's existing
MIT OR Apache-2.0 license; fixture evidence counts are not measured fitting results.
SHA2 0.10.9 uses the existing offline cache/lock version; only dialogue's
local dependency and its lock dependency list were added.

Limits: model 64MiB, schema 1MiB, manifest 64KiB; history JSON 512KiB;
1..4096 UTF8 bytes per utterance, trimmed-nonempty, NUL-free; at most128 total
USER+SYSTEM turns and65536 total utterance bytes. Reject rather than truncate.
Exact input bytes retained. Replay is linear in accepted history and model work.
Scope remains the accepted three services. No bookings/network/action execution.
Main integration and independent parent/full cross-track checks remain parent work.

## Reproduction and evidence

From this checkout, with WORK pointing to the supplied external artifact root:

```bash
export WORK=/home/gsc/.cache/glassmind/continuous-20261010
export TMPDIR="$WORK/draftsession-run/tmp"
export CARGO_TARGET_DIR="$WORK/draftsession-run/target"
python3 tools/dialogue/draft-session/baseline.py --source "$WORK/draftsession-source" --run "$WORK/draftsession-run/baseline-reproduction"
tools/dialogue/draft-session/cargo.sh fmt -p glassmind-dialogue -- --check
tools/dialogue/draft-session/cargo.sh test --offline -p glassmind-dialogue -j1 -- --test-threads 4
tools/dialogue/draft-session/cargo.sh clippy --offline -p glassmind-dialogue --all-targets -j1 -- -D warnings
tools/dialogue/draft-session/cargo.sh build --offline --release -p glassmind-dialogue --bins --examples -j1
for count in 10 179; do
  timeout --kill-after=10s 600 python3 tools/dialogue/draft-session/evaluate.py --source "$WORK/draftsession-source" --run "$WORK/draftsession-run/reproduction$count" --binary "$CARGO_TARGET_DIR/release/examples/draft-session" --count "$count"
done
```

Every cargo invocation uses the wrapper's 8GiB/zero-swap user scope,
`setsid --wait`, 600-second timeout and 10-second kill grace, persistent TMPDIR,
-j1 (fmt excepted), four test threads. Outer worker scope also verified 8GiB/0swap.
No background process remains from validation.

`$WORK/draftsession-run/frozen-candidate` retains original/final source, complete
diff including new files, old/new executables, source assets, baseline and parity
reports/streams, commands, failures, and a SHA256/byte inventory. Compact
machine-readable reports are copied alongside this report. No commits made.
