# Alias-aware verification revision, before implementation

Hypothesis: bounded enumeration of complete positive-question parses followed by
full endpoint resolution can accept predicate words inside names without using
edge truth to disambiguate language. Resolve endpoints before checking any claim.
An ambiguous endpoint remains a possible interpretation; never drop it to prefer
a uniquely named alternative. Multiple viable endpoint pairs must refuse even if
only one pair has a matching edge. Missing endpoints/edges remain Unknown. Bound
questions/endpoints as before; no runtime models, new data, index changes or main
integration in this prototype.

Require all original independent Supported cases and exact witnesses, no new wrong
Supported claims, all independent ambiguity/negation/extra-action exclusions, and
unchanged ordinary prediction bytes. Preserve original48 grades and all worker
67+5 labels verbatim; record any status or coverage changes individually. Original
strict-delimiter Refused cases may now resolve a unique endpoint pair or return
Unknown. Those are an explicit proposed grammar/contract revision, not proof that
the original contract passed. Do not integrate while contract tests remain failed.
