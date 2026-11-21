CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE data_sources (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), name text NOT NULL, source_url text, license text NOT NULL, methodology text, created_at timestamptz NOT NULL DEFAULT now());
CREATE TABLE investors (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), canonical_name text NOT NULL, slug text NOT NULL UNIQUE, investor_type text NOT NULL DEFAULT 'unknown', website_domain text, country text, description text, metadata jsonb NOT NULL DEFAULT '{}', created_at timestamptz NOT NULL DEFAULT now(), updated_at timestamptz NOT NULL DEFAULT now());
CREATE INDEX investors_name_idx ON investors USING gin (to_tsvector('simple', canonical_name));
CREATE TABLE investor_aliases (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), investor_id uuid NOT NULL REFERENCES investors ON DELETE CASCADE, alias text NOT NULL, source text NOT NULL, UNIQUE(investor_id,alias,source));
CREATE TABLE startups (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), name text NOT NULL, slug text NOT NULL UNIQUE, sector text, country text, website text, metadata jsonb NOT NULL DEFAULT '{}', created_at timestamptz NOT NULL DEFAULT now());
CREATE TABLE funding_rounds (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), startup_id uuid NOT NULL REFERENCES startups ON DELETE CASCADE, round_type text, announced_at date NOT NULL, amount numeric(24,4), currency char(3), source_id uuid REFERENCES data_sources, metadata jsonb NOT NULL DEFAULT '{}', UNIQUE(startup_id,announced_at,round_type));
CREATE INDEX funding_rounds_date_idx ON funding_rounds(announced_at);
CREATE TABLE round_participants (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), round_id uuid NOT NULL REFERENCES funding_rounds ON DELETE CASCADE, investor_id uuid NOT NULL REFERENCES investors, participation_role text NOT NULL DEFAULT 'investor', UNIQUE(round_id,investor_id));
CREATE TABLE aggregate_pairs (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), investor_a_id uuid NOT NULL REFERENCES investors, investor_b_id uuid NOT NULL REFERENCES investors, shared_deals integer NOT NULL CHECK(shared_deals >= 0), first_year integer NOT NULL, last_year integer NOT NULL, top_shared_sector text, top_shared_country text, profile_url text, source_id uuid REFERENCES data_sources, external_pair_id text, CHECK(investor_a_id < investor_b_id), CHECK(first_year <= last_year), UNIQUE(source_id,external_pair_id), UNIQUE(investor_a_id,investor_b_id,source_id));
CREATE INDEX pair_a_idx ON aggregate_pairs(investor_a_id,first_year,last_year);
CREATE INDEX pair_b_idx ON aggregate_pairs(investor_b_id,first_year,last_year);
CREATE INDEX pair_sector_idx ON aggregate_pairs(top_shared_sector);
CREATE TABLE data_imports (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), source_id uuid NOT NULL REFERENCES data_sources, dataset_name text NOT NULL, source_version text, started_at timestamptz NOT NULL DEFAULT now(), finished_at timestamptz, rows_seen bigint NOT NULL DEFAULT 0, rows_imported bigint NOT NULL DEFAULT 0, rows_rejected bigint NOT NULL DEFAULT 0, status text NOT NULL DEFAULT 'running');
CREATE TABLE import_errors (id bigserial PRIMARY KEY, import_id uuid NOT NULL REFERENCES data_imports ON DELETE CASCADE, row_number bigint NOT NULL, message text NOT NULL, raw_record jsonb);
CREATE TABLE portfolio_memberships (investor_id uuid NOT NULL REFERENCES investors, startup_id uuid NOT NULL REFERENCES startups, source_id uuid REFERENCES data_sources, observed_at date, PRIMARY KEY(investor_id,startup_id,source_id));
CREATE TABLE graph_metrics (investor_id uuid NOT NULL REFERENCES investors, dataset_id uuid, computed_at timestamptz NOT NULL DEFAULT now(), algorithm_version text NOT NULL, metrics jsonb NOT NULL, PRIMARY KEY(investor_id,algorithm_version));
CREATE TABLE community_assignments (investor_id uuid NOT NULL REFERENCES investors, dataset_id uuid, algorithm_version text NOT NULL, community_id text NOT NULL, PRIMARY KEY(investor_id,algorithm_version));
CREATE TABLE saved_graph_views (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), name text NOT NULL, filters jsonb NOT NULL DEFAULT '{}', created_at timestamptz NOT NULL DEFAULT now());
INSERT INTO data_sources(id,name,source_url,license,methodology) VALUES ('00000000-0000-4000-8000-000000000001','VentureGraph fictional demo',NULL,'CC0-1.0','Synthetic illustrative graph; no real investor or startup data.') ON CONFLICT DO NOTHING;
INSERT INTO investors(id,canonical_name,slug,investor_type,website_domain) VALUES
('10000000-0000-4000-8000-000000000001','Northstar Ventures','northstar-ventures','venture_capital',NULL),
('10000000-0000-4000-8000-000000000002','Juniper Capital','juniper-capital','venture_capital',NULL),
('10000000-0000-4000-8000-000000000003','Commonwealth Seed Fund','commonwealth-seed','venture_capital',NULL),
('10000000-0000-4000-8000-000000000004','Tandem Growth Partners','tandem-growth','growth_equity',NULL),
('10000000-0000-4000-8000-000000000005','Meridian Labs Fund','meridian-labs','venture_capital',NULL),
('10000000-0000-4000-8000-000000000006','Cedar Grove Ventures','cedar-grove','venture_capital',NULL),
('10000000-0000-4000-8000-000000000007','Blue Peak Capital','blue-peak','venture_capital',NULL),
('10000000-0000-4000-8000-000000000008','Orbit Founders Fund','orbit-founders','venture_capital',NULL) ON CONFLICT (id) DO NOTHING;
INSERT INTO aggregate_pairs(investor_a_id,investor_b_id,shared_deals,first_year,last_year,top_shared_sector,top_shared_country,source_id,external_pair_id) VALUES
('10000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000002',12,2019,2025,'Enterprise software','United States','00000000-0000-4000-8000-000000000001','demo-1'),
('10000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000003',8,2020,2026,'Climate technology','United States','00000000-0000-4000-8000-000000000001','demo-2'),
('10000000-0000-4000-8000-000000000002','10000000-0000-4000-8000-000000000004',6,2021,2025,'Fintech','United Kingdom','00000000-0000-4000-8000-000000000001','demo-3'),
('10000000-0000-4000-8000-000000000003','10000000-0000-4000-8000-000000000005',5,2022,2026,'Health technology','United States','00000000-0000-4000-8000-000000000001','demo-4'),
('10000000-0000-4000-8000-000000000005','10000000-0000-4000-8000-000000000006',9,2019,2024,'Enterprise software','Canada','00000000-0000-4000-8000-000000000001','demo-5'),
('10000000-0000-4000-8000-000000000004','10000000-0000-4000-8000-000000000007',4,2023,2026,'Fintech','United States','00000000-0000-4000-8000-000000000001','demo-6'),
('10000000-0000-4000-8000-000000000006','10000000-0000-4000-8000-000000000008',3,2024,2026,'Climate technology','Australia','00000000-0000-4000-8000-000000000001','demo-7'),
('10000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000004',7,2020,2024,'Enterprise software','United Kingdom','00000000-0000-4000-8000-000000000001','demo-8') ON CONFLICT DO NOTHING;
INSERT INTO startups(id,name,slug,sector,country,metadata) VALUES
('20000000-0000-4000-8000-000000000001','Example Lattice','example-lattice','Enterprise software','United States','{"synthetic":true}'),
('20000000-0000-4000-8000-000000000002','Example Verdant','example-verdant','Climate technology','United States','{"synthetic":true}') ON CONFLICT DO NOTHING;
INSERT INTO funding_rounds(startup_id,round_type,announced_at,amount,currency,source_id,metadata) VALUES
('20000000-0000-4000-8000-000000000001','Seed','2021-04-12',2500000,'USD','00000000-0000-4000-8000-000000000001','{"synthetic":true}'),
('20000000-0000-4000-8000-000000000001','Series A','2023-09-05',12000000,'USD','00000000-0000-4000-8000-000000000001','{"synthetic":true}'),
('20000000-0000-4000-8000-000000000002','Seed','2024-02-20',4000000,'USD','00000000-0000-4000-8000-000000000001','{"synthetic":true}') ON CONFLICT DO NOTHING;
INSERT INTO round_participants(round_id,investor_id) SELECT r.id,i.id FROM funding_rounds r CROSS JOIN investors i WHERE (r.startup_id='20000000-0000-4000-8000-000000000001' AND i.id IN ('10000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000002')) OR (r.startup_id='20000000-0000-4000-8000-000000000002' AND i.id IN ('10000000-0000-4000-8000-000000000003','10000000-0000-4000-8000-000000000005')) ON CONFLICT DO NOTHING;
