# Experiments

Baseline authored full preview: 23/24 positive remedies, 4/12 correct read-only
status results, 10/32 negative refusals; 30 unauthorized remedy previews among
44 status/negative cases. Native nine-case sample: 21/27 source/native expectations
met, with six unauthorized applied repairs. Original raw outputs retained in ownrun.

Revision one: 68/68 new speech-act bridge checks passed, but existing frozen rename
control FAILED for `fix the test named rename_foo`: baseline refuses this unsupported
named-test target; candidate invented a global Test repair. This is a real regression,
not a test failure to relabel. Archived binary, source, patch, and test log under
ownrun/revision-one before changing parser. Removed the new `named` target syntax;
retained existing `for function` qualifier behavior. Existing tests unchanged.

Correction to the early baseline sample description: six failed expectations
comprised THREE unauthorized do edits and THREE unchanged initial ask --apply
requests. Initial ask --apply does not auto-apply a new repair, in either binary.
The frozen PLAN expected otherwise. We retain its failed expectations unchanged;
separate confirm.py tests preview + affirmative yes --apply in a persisted session.
This is not a claim that the original frozen acceptance gate passed.

The first 68-case candidate had 24/24 positive typed requests and zero unauthorized
repair readings, but only 6/15 existing public diagnostic repair phrasings worked
through the bridge. Rejected that restricted coverage. Added bounded failure
statements, diagnostic messages/error codes, source-name slots, and example paths.
No free-text tail is discarded. First context expansion temporarily lost three
project/compile cases because the noun was consumed as a source modifier. Preserved
failure logs; removed compile from that modifier lookahead. A quoted-only location
slot lost the legacy `in heck` example; replaced with a complete identifier slot.
All 15 public examples and additional contextual negatives now pass helper tests.
Bare source modifiers before a check noun require a determiner so hypothetical
introducers such as `Suppose build ...` cannot be parsed as a source name.

Native probe setup defect: concurrent fixtures initially shared the package name
repair_probe and target directory. Two native checks falsely reused a passing
artifact while exact source bytes remained broken. All raw results are preserved;
those runs are not native evidence for the final candidate. New fixture names
include a SHA256-derived identity of their full path, while retaining the supplied
CARGO_TARGET_DIR. Final evidence is only ownrun/verified-* against the immutable
ownrun/frozen-candidate/glassmind. Earlier runs against a rebuilt target binary
are interim evidence, not final exact-binary validation.
