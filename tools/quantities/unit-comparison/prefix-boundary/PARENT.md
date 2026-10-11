# Explicit prefix preserves the comparison payload limit

The old route rejected valid4096-byte questions when prefixed with
`Compare quantities: ` because its separator space became part of the payload.
The route now consumes at most one optional ASCII space after the colon, matching
the command's spelling. The4096-byte limit still applies before whitespace
normalization;4097-byte inputs and NUL still refuse.

The independently executed baseline passes12/14 boundary cases and fails both
at-limit explicit requests. The revised main passes14/14 in each of plain and
intent-model modes, including ASCII and multibyte Unicode whitespace. All158
existing main comparison/state contracts pass again with process restarts, real
facts/draft bundles, memory/story preservation, and false/equal human output.
Quantity library and native solver source are unchanged. Workspace fmt,
all-target clippy, release build and strict4/4 smoke pass.

Evaluated main SHA:
`931528b065c361a9580a090392497b9e060a894cf5f1edb824ccb0132836eda4`.
Original baseline failure, fixed results, contracts and check log are preserved
here. Full input/output streams and binary remain in
`~/.cache/glassmind/continuous-20261010/coordinator/unitcompare-prefix-boundary`.
This is a route-boundary fix; no benchmark accuracy change is claimed.
