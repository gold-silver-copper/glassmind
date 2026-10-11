# Official documentation inspected before network acquisition

Consulted 2026-10-11 UTC using web read tools, before the first WDQS request:

- [WDQS user manual](https://www.mediawiki.org/wiki/Wikidata_Query_Service/User_Manual#Query_limits):
  60-second query deadline; processing and error quotas; HTTP429/Retry-After;
  identifying User-Agent. GET and JSON supported. We use sequential requests,
  >=5-second start spacing, socket timeout45s, total alarm600s, <=110 requests,
  <=20MiB decompressed data. Stop entirely on first failure; no retry thus no
  Retry-After violation. Redirects are refused, not followed to another endpoint.
- [User-Agent policy](https://foundation.wikimedia.org/wiki/Policy:Wikimedia_Foundation_User-Agent_Policy):
  identify application and project contact URL; no browser impersonation.
- [RDF format](https://www.mediawiki.org/wiki/Wikibase/Indexing/RDF_Dump_Format):
  p:/ps: statement paths retain normal/preferred ranks; wdt: truthy projection
  is inappropriate here. EntityData metadata may contain schema:version; OPTIONAL
  retrieval records absence as null. One eligible statement and rank are sampled
  independently per distinct subject, sufficient to witness the property-target
  relation, not to reconstruct all statement GUIDs/qualifiers.
- [Graph split](https://www.wikidata.org/wiki/Wikidata:SPARQL_query_service/WDQS_graph_split),
  page revision2542638311: primary endpoint contains main graph, excluding scholarly
  graph since2025-05-09. No federation/alternate endpoint used. Even an untruncated
  set is only main-endpoint query snapshot coverage, not full Wikidata uniqueness.
- Structured Wikidata data is CC0-1.0; policy/manual prose is not copied into the
  dataset. Documentation tool reads are separate from the bounded dataset fetch.
  One mistaken Wikimedia_Query_Service manual URL returned an internal error;
  the correct Wikidata_Query_Service page was read successfully. No dataset query
  was made through the web tool.
