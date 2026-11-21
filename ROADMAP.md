# Roadmap

VentureGraph is an actively developed investor network explorer. This roadmap describes the current product scope and areas under consideration; it does not promise delivery dates.

## Available today

- Source-separated fictional demo and optional aggregate Fliar datasets
- Investor directory, profiles, partner relationships, and source coverage
- Weighted co-investment graph, centrality rankings, and path finding
- Deterministic label-propagation community heuristic
- Pair discovery filters and transparent recent-observation rankings
- Dated event comparison for fictional demo records
- Rust CSV importer and read-only REST API with OpenAPI documentation
- Local Docker Compose development stack

## Areas for future work

- Portfolio membership ingestion and verified portfolio-overlap analysis
- Authenticated administrative imports and import management
- Broader filter and selection state shared across application views
- Temporal playback and additional event-level data ingestion
- Graph computation caching and rate limiting
- Browser end-to-end coverage
- Production deployment configuration and operational guidance

## Data and interpretation

The built-in graph and dated demo events are fictional. The optional Fliar sample contains aggregate pair observations, not dated rounds or full portfolio membership. Its CC BY-NC 4.0 license restricts use to non-commercial purposes under the license terms. Community assignments and emerging-investor rankings are heuristics, not verified classifications or growth rates.
