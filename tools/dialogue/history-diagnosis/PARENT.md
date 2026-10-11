# TRAIN diagnosis: missing destinations need new candidate evidence

The approved dialogue model gets 787 of 1,171 USER turns completely correct on
all 144 complete TRAIN conversations containing only its three supported
services. Routing and intents are correct throughout this subset. Of 384
incorrect turns, 382 lack RideSharing destination values and two have incorrect
Travel locations. The fixed first-ten sample passed 53/53 turns before the full
subset was run. These are in-sample diagnostics, not a new DEV score or model gain.

The missing destinations begin at 87 state updates and persist for another 295
turns. Case-insensitive text matching finds all 87 values in prior utterances:
83 last occur earlier than the immediately preceding SYSTEM turn, four in that
immediate turn. This establishes text availability only, not a correct extractor
or proof of the user's intended referent.

A native probe calls the unchanged approved OfferModel on up to the last 32
SYSTEM turns, for predicted-active services with missing text slots. It does
not apply any proposal. Across the whole subset it produces only two rows,
both correct elsewhere, and **no proposal for any of the 87 initial missing
destinations**. Every emitted UTF-8 span was independently checked against the
source utterance. Merely extending the existing offer history cannot address
the observed failures. A subsequent experiment needs new candidate evidence,
such as learned system-offer entities plus a separate reference-selection model.
No such model has been fit or accepted here.

Runtime input contains only speaker and utterance; the unchanged upstream
scorer and offline diagnostics inspect TRAIN annotations after inference. The
approved model SHA256 remains
`fac911dc687d8b31d6ef48ef5d200827ee417f85cd25b832bb1a709855bbbd88`.
Production source, model weights and DEV scores are unchanged. No held-out
material, new download or development-time model call was used.

Source: SGD revision `e852981ae34990f4358979625854259302feaa78`,
CC-BY-SA-4.0. The source TRAIN file has 3,174 selected-service conversations;
exactly 144 qualify for this complete-conversation diagnostic, and all
1,171 USER turns / 1,323 gold frames are retained.
The official metric remains the pinned Apache-2.0 upstream implementation.
Data-derived artifacts retain CC-BY-SA-4.0; new diagnostic code uses the workspace
MIT OR Apache-2.0 license.

Exact source, binary and input identities are in freeze.json and
probe-source-freeze.json. Native build used an 8 GiB, zero-swap systemd scope,
setsid --wait, a timeout, one Cargo job, offline dependencies and persistent
TMPDIR. Both inference runs and the historical-offer probe were bounded likewise.
The probe Cargo path targets this checkout; it is a local diagnostic harness.
Raw TRAIN inputs, predictions, per-error details and binaries remain at
`~/.cache/glassmind/continuous-20261010/coordinator/dialogue-train-diagnosis`.
Run the retained diagnose.py with the existing metrics interpreter and
history-audit.py with Python to reproduce the aggregate joins; they do not fit
or alter a model. No additional workspace gate is needed for this evidence-only
directory.
