# Reserved separator audit (before corrective edit)

The frozen grammar excludes AND inside endpoints. Review found that an indexed
alias such as "Harbor and works for Studio" could nonetheless be silently split
into shorter endpoints. Freeze the additional check: if any full normalized alias
spans a reserved separator within endpoint positions, return UnsupportedGrammar,
regardless of edges, alias ambiguity or case acceptance. This does not add grammar
alternatives. There remain exactly one permitted parse and at most three forbidden
alias-span checks (2..3 clauses), each <=256 bytes, before claims. Retain original
45 contracts unchanged. New independent contrast: add the alias above as Q120 to
the authored graph and query "Who was born in Harbor and works for Studio and is
a citizen of Republic?". Expected refused, with and without all graph claims.
The unsplit predicate-word alias "Born in Bay" must remain supported.
