# API

The data endpoints are read-only. Versioned API routes use `/api/v1`; database
health checks are available at `/health` and `/ready`. The interactive API
landing page is at `/docs`, and the machine-readable OpenAPI 3.1 schema is at
`/api/v1/openapi.json`.

| Route                                                                                 | Behavior                                                                                             |
| ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `GET /health`, `GET /ready`                                                           | Database readiness                                                                                   |
| `GET /api/v1/meta`                                                                    | API version and algorithm capabilities                                                               |
| `GET /api/v1/data-sources`                                                            | Source licenses, methodology, and provenance                                                         |
| `GET /api/v1/data-coverage`                                                           | Per-source investor, pair, round, and year coverage                                                  |
| `GET /api/v1/graph?dataset=demo&year=2024&limit=1000`                                 | Source-scoped weighted graph; dataset is `demo` or `fliar`; `year` uses inclusive observed ranges    |
| `GET /api/v1/temporal?dataset=demo&year=2024`                                         | Dated round co-investment snapshot for fictional demo events only                                    |
| `GET /api/v1/temporal/diff?dataset=demo&year_a=2021&year_b=2025`                      | New, absent, and retained observed pair edges in dated demo events                                   |
| `GET /api/v1/investors?q=North&page=1&per_page=50&type=venture_capital&country=US`    | Source-aware investor search with pagination, type, and country filters                              |
| `GET /api/v1/investors/{id}?dataset=demo`                                             | Stable-ID profile for an investor in the selected dataset                                            |
| `GET /api/v1/investors/{id}/partners?dataset=demo&limit=50`                           | Top observed partner edges with weights and observation years                                        |
| `GET /api/v1/metrics?dataset=demo`                                                    | Selected source counts                                                                               |
| `GET /api/v1/path?from=<uuid>&to=<uuid>&mode=hops`                                    | Fewest-hop or strongest-connection route                                                             |
| `GET /api/v1/graph/communities?dataset=demo`                                          | Weighted label-propagation heuristic communities                                                     |
| `GET /api/v1/communities`                                                             | Connected components, explicitly distinguished from communities                                      |
| `GET /api/v1/graph/similarity?a=<uuid>&b=<uuid>`                                      | Common neighbors and Jaccard similarity                                                              |
| `GET /api/v1/analytics/centrality?dataset=demo`                                       | Full selected graph's degree, PageRank, betweenness, and clustering metrics                          |
| `GET /api/v1/analytics/co-investments?dataset=demo&sector=Fintech&min_shared_deals=2` | Highest-weight co-investment pairs with source range and sector/country/type/year filters            |
| `GET /api/v1/analytics/emerging?dataset=demo&since_year=2024`                         | Investors with recent source-reported first observations, using a disclosed aggregate-data heuristic |

The investor list response contains `data`, `page`, `per_page`, `total`, and `total_pages`. `per_page` is clamped to 1–200. Search is case-insensitive; investor type is an exact normalized type filter, and country matches the investor country field when it is populated. Profile and partner routes require the investor to be present in the selected source. The partner limit is clamped to 1–200.

# Analysis endpoint semantics

`GET /ready` checks PostgreSQL connectivity. `GET /api/v1/meta` reports the API
and supported computation names. `GET /api/v1/data-sources` returns source
provenance and license records; `GET /api/v1/data-coverage` returns counts and
year bounds separately for each source.

`GET /api/v1/analytics/centrality?dataset=demo|fliar` returns degree, weighted
degree, weighted PageRank, normalized unweighted betweenness, and local
clustering coefficient for each investor represented by an aggregate pair in
the selected dataset. Betweenness is exact up to 5,000 nodes and uses 256
evenly spaced deterministic sources above that threshold; the response labels
the method. The scope is the full selected dataset graph, not the rendered
visualization subset.

`GET /api/v1/analytics/co-investments?dataset=demo|fliar` ranks pair records by
shared-deal count and accepts optional `sector`, `country`, `investor_type`,
`year`, `min_shared_deals`, and `limit` filters. The year filter selects an
aggregate observed range containing that year; it does not assert an exact
deal date. Sector and country are substring matches against source pair fields.

`GET /api/v1/analytics/emerging?dataset=demo|fliar&since_year=2024` ranks
investors by pair records whose source-reported first year is at or after the
threshold. This is a discovery heuristic, not an annual growth rate; the
response carries the definition and evidence interpretation.

`GET /api/v1/graph/similarity?a=<investor-uuid>&b=<investor-uuid>&dataset=demo|fliar`
returns common neighbors and Jaccard similarity of the two investors' graph
neighborhoods. Both investors must occur in the selected dataset graph.

`GET /api/v1/path?from=<investor-uuid>&to=<investor-uuid>&dataset=demo|fliar&mode=strongest|hops`
supports both fewest-hop BFS (`mode=hops`) and strongest-connection Dijkstra
(`mode=strongest`, the default). The response includes per-edge shared-deal
weights, route cost, and its definition. Strongest-route cost is the sum of
reciprocal shared-deal counts and is not a hop count.

`GET /api/v1/temporal/diff?year_a=2021&year_b=2025&dataset=demo` compares
observed co-investment pairs in two years of dated demo events and separates
new, absent, and retained edges. Aggregate sources are rejected for this query
because their first/last observation bounds cannot establish exact annual
activity. “Absent” means not observed in the later period, not that a business
relationship ended.

`GET /api/v1/graph/communities?dataset=demo|fliar` returns weighted label
propagation groups and labels them as a heuristic partition. The separate
`GET /api/v1/communities` endpoint continues to return connected components.
