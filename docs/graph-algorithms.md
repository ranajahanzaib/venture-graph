# Graph algorithms

Reusable algorithms live in `crates/graph` (`venturegraph-graph`) and operate on
an undirected weighted investor graph independently of HTTP handlers. Current
operations include degree, weighted degree, weighted-transition PageRank,
connected components, fewest-hop BFS, strongest-route Dijkstra, local clustering,
common neighbors, Jaccard similarity, bounded neighborhoods, density, exact
unweighted Brandes betweenness, and deterministic synchronous label propagation.

Label propagation is a heuristic partition whose labels depend on stable node
insertion order and deterministic tie breaking. It is not a ground-truth
community classification. Betweenness is exact for graphs up to 5,000 nodes;
larger graphs use an evenly spaced deterministic sample of 256 sources. Both
methods are unweighted and normalized for ranking in this projection. Weighted
degree sums shared-deal counts. Strongest-route cost is the sum of
`1 / max(shared_deals, 1)` and must not be described as fewest hops.

The API exposes selected full-dataset calculations at
`GET /api/v1/analytics/centrality` and `GET /api/v1/graph/similarity?a=<uuid>&b=<uuid>`.
Centrality responses include the dataset scope and betweenness method used.
