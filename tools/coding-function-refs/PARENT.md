# Function-reference rename with independent native validation

A bounded module/binding proof extends free-function rename to references in
imports, reexports, callbacks and ordinary assertion macros. The existing generic
route stays intact. Unclassified old-name tokens, shadowing, unknown macros,
collisions and unsupported package layouts refuse. Apply reindexes and reproves
the binding and exact spans before mutation, then uses existing native-test
verification and rollback. It does not infer identity from spelling alone.

The coordinator independently froze 16 cases before inspecting the candidate.
Baseline repairs eight; candidate repairs nine, retaining all eight. Seven cases
remain byte-identical refusals. Every actual edit matches a frozen exact source
mapping; all 16 fixtures compile and execute unchanged behavior tests before and
after. Aliases retain their local names; comments, strings, Unicode and CRLF
remain intact. The new parent case is the cfg(test) module's unique super glob.
The other simple import/callback cases already worked and are not counted as gains.

The supplied five-file, 2,923-byte public Rustbench workspace now supports the
canonical request `rename function parse_line to parse_entry`. Parent native
replay confirms exactly nine identifier edits across four Rust files, an unchanged
Cargo manifest, a read-only nine-span preview, passing original tests before and
after, and byte-identical executable output. All nine occurrences were inspected
independently; none is in a string or comment. Original terse/hurried/chatty wording
has separate language failures and is not claimed fixed by this backend.

The parent verified 13 archive entries (119,342 bytes), 3,361 retained artifacts
(521,827,063 bytes), 71 authored-source identities, 11 source identities and three
frozen identities. The original worker archive remains in `functionrefs-run/submitted`;
the separate coordinator archive is `functionrefs-run/parent-submitted`.
Root runtime copies exactly match the archived worker source. All original repository copies and retained artifacts were reverified before
removing the inactive worktree and temporary login link. Original manifest paths
map to the preserved archives after removal. Raw artifacts remain
under `/home/gsc/.cache/glassmind/continuous-20261010/functionrefs-run`.
The evaluated worker binary SHA256 is
`f57788db8058bde96f306a0b9de2a8d7a1645969f6bd2b17a35254ac53607cf9`.

Original worker results: 24/24 authored-plus-supplied cases, nine edits and 15
refusals; 12 supplemental preview/rollback/frontend contracts pass. World tests
and CLI edit tests pass. Parent integrated world tests pass (34 unit and four
isolation tests). Workspace fmt, all-target clippy, release build and strict four-case smoke pass.
The integrated release replays all 16 independent cases with identical source
hashes and outcomes to the worker binary. Integrated source commit is
`4effc66fb430f130ca8c97fb8eed586d28af8676`; release SHA256 is
`53d6e10aaa9d715eb1b8963c5cacaf4f41f06111f49e462161597bf91f7db2b9`.
The combined final language/backend release at source
`bb89964dd887b2b68ae01fb8f0a9a92a4901f490` passes full765:180 passes,585 refusals,
zero failures,all177 prior passes retained. The three gains require both layers;
this is not a separate backend-only score. See the
[language review and full comparison](../coding-rename-language/PARENT.md).

Existing adapter limitations remain explicit: exports 26/27 and AddField 14/15.
Ask/apply scores 15/16 against an unchanged expectation of refusal for an ordinary
assert_eq!(super::weigh(...)) reference. Parent inspected baseline and candidate:
the only changes are `weigh` to `measure` in the declaration and assertion, and
the original assertion still executes successfully. This is newly supported
behavior; no scorer was altered or failed expectation hidden.

The original [plan](PLAN.md), [handoff](HANDOFF.md), [experiment corrections](EXPERIMENTS.md)
and [independent identities](parent/identities.json) retain all details. Worker
corrected an authored expected-byte checker that mishandled an escaped newline
inside a Rust string and a hand-counted Unicode column; original failed artifacts
were retained. Parent exact-byte comparisons use separately authored expectations.
No new data/downloads/dependencies. Public source revision
`f79e71a2a26fdaf15f9bc55225a038b61e66fafb`, MIT OR Apache-2.0; parent probes MIT.
