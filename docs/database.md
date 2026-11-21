# Database

The initial SQLx migration creates source provenance, investors and aliases, startups, dated rounds and participants, aggregate pairs, import runs and row errors, portfolio memberships, metric/community cache tables, and saved views. UUID primary keys, foreign keys, undirected canonical pair order, range checks, and uniqueness prevent malformed and duplicate edges. Funding amounts use `numeric(24,4)`. The migration also inserts a small explicitly fictional graph and fictional dated rounds.

Pair year indexes support source, year, endpoint, and sector lookups. Import files remain local and outside version control.
