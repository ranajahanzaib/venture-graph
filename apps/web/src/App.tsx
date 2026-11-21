import { useQuery } from "@tanstack/react-query";
import {
  Activity,
  ArrowUpRight,
  ChevronDown,
  Download,
  GitFork,
  LayoutDashboard,
  Network,
  Search,
  Settings,
  SlidersHorizontal,
  Users,
  Waypoints,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Link, useLocation } from "react-router-dom";
import GraphView from "./GraphView";
import { formatCount } from "./lib/format";
type Investor = {
  id: string;
  name: string;
  investor_type: string;
  domain?: string | null;
  country?: string | null;
};
type Edge = {
  source: string;
  target: string;
  weight: number;
  first_year: number;
  last_year: number;
  sector?: string | null;
};
type Graph = {
  dataset: string;
  temporal_capability: string;
  nodes: Investor[];
  edges: Edge[];
};
type Centrality = {
  investors: Array<{
    investor_id: string;
    degree: number;
    weighted_degree: number;
    pagerank: number;
    betweenness: number;
    clustering_coefficient: number;
  }>;
};
const API = import.meta.env.VITE_API_URL ?? "http://localhost:8080";
const fetcher = async <T,>(url: string): Promise<T> => {
  const r = await fetch(url);
  if (!r.ok) throw Error("API request failed");
  return r.json();
};
export default function App() {
  const location = useLocation();
  const [query, setQuery] = useState("");
  const [year, setYear] = useState("");
  const [dataset, setDataset] = useState("demo");
  const [sector, setSector] = useState("");
  const [sectorOptions, setSectorOptions] = useState<string[]>([]);
  const [selected, setSelected] = useState<Investor | null>(null);
  const graph = useQuery({
    queryKey: ["graph", year, dataset, sector],
    queryFn: () =>
      fetcher<Graph>(
        dataset === "demo" && year
          ? `${API}/api/v1/temporal?dataset=demo&year=${year}`
          : `${API}/api/v1/graph?dataset=${dataset}&limit=1500${year ? `&year=${year}` : ""}${sector ? `&sector=${encodeURIComponent(sector)}` : ""}`,
      ),
  });
  const metrics = useQuery({
    queryKey: ["metrics", dataset],
    queryFn: () =>
      fetcher<{
        investors: number;
        relationships: number;
        dated_rounds: number;
      }>(`${API}/api/v1/metrics?dataset=${dataset}`),
  });
  const centrality = useQuery({
    queryKey: ["centrality", dataset],
    queryFn: () =>
      fetcher<Centrality>(
        `${API}/api/v1/analytics/centrality?dataset=${dataset}`,
      ),
  });
  const topInvestor = centrality.data?.investors[0];
  const data = graph.data;
  const topInvestorName = data?.nodes.find(
    (node) => node.id === topInvestor?.investor_id,
  )?.name;
  const shown = useMemo(
    () =>
      data?.nodes
        .filter((n) => n.name.toLowerCase().includes(query.toLowerCase()))
        .slice(0, 8) ?? [],
    [data, query],
  );
  useEffect(() => {
    if (!sector && data)
      setSectorOptions(
        Array.from(
          new Set(
            data.edges
              .map((edge) => edge.sector)
              .filter((value): value is string => Boolean(value)),
          ),
        ).sort(),
      );
  }, [data, sector]);
  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandmark">
            <Network size={20} />
          </div>
          <span>
            venture<span className="lime">graph</span>
          </span>
        </div>
        <div className="workspace">
          <div className="avatar">VG</div>
          <div>
            <b>VentureGraph</b>
            <small>Workspace</small>
          </div>
          <ChevronDown size={15} />
        </div>
        <div className="nav-label">WORKSPACE</div>
        <nav>
          <Link className={location.pathname === "/" ? "active" : ""} to="/">
            <LayoutDashboard size={17} />
            Overview
          </Link>
          <Link
            className={location.pathname === "/explorer" ? "active" : ""}
            to="/explorer"
          >
            <Waypoints size={17} />
            Network explorer
          </Link>
          <Link
            className={location.pathname === "/investors" ? "active" : ""}
            to="/investors"
          >
            <Users size={17} />
            Investors{" "}
            <span className="nav-count">{format(metrics.data?.investors)}</span>
          </Link>
          <Link
            className={location.pathname === "/communities" ? "active" : ""}
            to="/communities"
          >
            <GitFork size={17} />
            Communities
          </Link>
          <Link
            className={location.pathname === "/timeline" ? "active" : ""}
            to="/timeline"
          >
            <Activity size={17} />
            Temporal explorer
          </Link>
          <Link
            className={location.pathname === "/paths" ? "active" : ""}
            to="/paths"
          >
            <Network size={17} />
            Relationship finder
          </Link>
        </nav>
        <div className="nav-label lower">ANALYZE</div>
        <nav>
          <Link
            className={location.pathname === "/analytics" ? "active" : ""}
            to="/analytics"
          >
            <Activity size={17} />
            Market signals
          </Link>
          <Link
            className={location.pathname === "/compare" ? "active" : ""}
            to="/compare"
          >
            <SlidersHorizontal size={17} />
            Compare sectors
          </Link>
        </nav>
        <div className="sidebar-bottom">
          <div className="plan-card">
            <div className="plan-icon">
              <Network size={15} />
            </div>
            <b>Data coverage</b>
            <span>
              {dataset === "demo"
                ? "Fictional demonstration dataset"
                : "Fliar aggregate pair sample"}
            </span>
            <div className="progress">
              <i />
            </div>
            <small>
              {dataset === "demo" ? "Synthetic round data" : "2019 — Jun 2026"}
            </small>
          </div>
          <Link className="settings" to="/settings">
            <Settings size={16} />
            Settings
          </Link>
          <div className="profile">
            <div className="avatar user">VG</div>
            <div>
              <b>Demo workspace</b>
              <small>Local session</small>
            </div>
            <ChevronDown size={15} />
          </div>
        </div>
      </aside>
      <main className="main">
        <header>
          <div className="crumb">
            Workspace <span>/</span> <b>{pageTitle(location.pathname)}</b>
          </div>
          <div className="header-right">
            <span className="status">
              <i /> Data synced
            </span>
            <div className="user-chip">VG</div>
          </div>
        </header>
        <section className="content">
          {location.pathname === "/" ? (
            <>
              <div className="welcome">
                <div>
                  <div className="eyebrow">
                    <span /> INVESTOR NETWORK INTELLIGENCE
                  </div>
                  <h1>
                    See the connections
                    <br />
                    behind the <em>capital.</em>
                  </h1>
                  <p>
                    Explore co-investment relationships and uncover the
                    communities shaping the venture landscape.
                  </p>
                </div>
                <div className="welcome-mark">
                  <div className="orbit orbit-a" />
                  <div className="orbit orbit-b" />
                  <div className="orb-center">
                    <Network size={27} />
                  </div>
                  <i className="dot d1" />
                  <i className="dot d2" />
                  <i className="dot d3" />
                  <i className="dot d4" />
                </div>
              </div>
              <div className="stats">
                <Stat
                  title="Investors tracked"
                  value={format(metrics.data?.investors)}
                  note="Across the ecosystem"
                  icon={<Users />}
                />
                <Stat
                  title="Co-investment ties"
                  value={format(metrics.data?.relationships)}
                  note="Observed pair relationships"
                  icon={<GitFork />}
                />
                <Stat
                  title="Observation window"
                  value={dataset === "demo" ? "2021 — 2024" : "2019 — 2026"}
                  note={
                    dataset === "demo"
                      ? "Synthetic demo event years"
                      : "Aggregate source coverage"
                  }
                  icon={<Activity />}
                />
                <Stat
                  title="Dated demo rounds"
                  value={format(metrics.data?.dated_rounds)}
                  note={
                    dataset === "demo"
                      ? "Fictional event records"
                      : "Not provided by this source"
                  }
                  icon={<Waypoints />}
                />
              </div>
              <div className="section-head">
                <div>
                  <h2>Network landscape</h2>
                  <p>
                    Explore the strongest observed co-investment relationships
                  </p>
                </div>
                <div className="controls">
                  <div className="searchbox">
                    <Search size={15} />
                    <input
                      value={query}
                      onChange={(e) => setQuery(e.target.value)}
                      placeholder="Search investors..."
                    />
                    {query && shown.length > 0 && (
                      <div className="suggestions">
                        {shown.map((n) => (
                          <button
                            key={n.id}
                            onClick={() => {
                              setSelected(n);
                              setQuery(n.name);
                            }}
                          >
                            {n.name}
                            <span>{n.investor_type}</span>
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                  <select
                    value={dataset}
                    onChange={(e) => {
                      setDataset(e.target.value);
                      setYear("");
                      setSector("");
                    }}
                  >
                    <option value="demo">Fictional demo</option>
                    <option value="fliar">Fliar sample</option>
                  </select>
                  <select
                    value={year}
                    onChange={(e) => {
                      setYear(e.target.value);
                      if (dataset === "demo") setSector("");
                    }}
                  >
                    <option value="">All years</option>
                    {Array.from({ length: 8 }, (_, i) => 2019 + i).map((y) => (
                      <option key={y}>{y}</option>
                    ))}
                  </select>
                  <select
                    aria-label="Filter by top shared sector"
                    value={sector}
                    disabled={dataset === "demo" && Boolean(year)}
                    onChange={(e) => setSector(e.target.value)}
                  >
                    <option value="">All sectors</option>
                    {sectorOptions.map((value) => (
                      <option key={value}>{value}</option>
                    ))}
                  </select>
                  <button
                    className="export"
                    onClick={() => data && download(data)}
                  >
                    <Download size={15} /> Export
                  </button>
                </div>
              </div>
              <div className="canvas-card">
                <div className="canvas-top">
                  <div className="legend">
                    <span>
                      <i className="legend-dot green" />
                      Investor
                    </span>
                    <span>
                      <i className="legend-dot yellow" />
                      Selected investor
                    </span>
                    <span>
                      <i className="legend-line" />
                      Co-investment tie
                    </span>
                  </div>
                  <div className="canvas-actions">
                    <span>
                      {data
                        ? `${data.nodes.length.toLocaleString()} nodes · ${data.edges.length.toLocaleString()} ties`
                        : "Loading network"}
                    </span>
                  </div>
                </div>
                {graph.isError ? (
                  <div className="empty">
                    <Network size={30} />
                    <b>Couldn’t reach the graph API</b>
                    <span>
                      Start the API and PostgreSQL, then reload this page.
                    </span>
                  </div>
                ) : graph.isPending ? (
                  <div className="empty">
                    <Network size={30} />
                    <b>Loading network</b>
                  </div>
                ) : data?.edges.length === 0 ? (
                  <div className="empty">
                    <Network size={30} />
                    <b>No relationships found for this dataset</b>
                    <span>
                      Import the Fliar sample to explore its aggregate network.
                    </span>
                  </div>
                ) : (
                  <GraphView
                    data={data}
                    selected={selected}
                    onSelect={setSelected}
                  />
                )}
                <div className="canvas-foot">
                  <span>
                    <i /> Live network view
                  </span>
                  <span>Edge width represents shared deal count</span>
                </div>
              </div>
              <div className="bottom-grid">
                <div className="insight-card">
                  <div className="card-title">
                    <div>
                      <span className="mini-icon">
                        <Activity size={15} />
                      </span>
                      <h3>Network signals</h3>
                    </div>
                    <Link to="/analytics">
                      View analysis <ArrowUpRight size={14} />
                    </Link>
                  </div>
                  <div className="signal">
                    <div className="signal-icon">
                      <GitFork size={17} />
                    </div>
                    <div>
                      <b>
                        Highest PageRank
                        {centrality.isPending ? " · calculating" : ""}
                      </b>
                      <span>
                        {centrality.isError
                          ? "Centrality is temporarily unavailable."
                          : topInvestor
                            ? `${topInvestorName ?? "Investor"} ranks highest by weighted PageRank (${topInvestor.pagerank.toFixed(3)}).`
                            : "No connected investors are present in this dataset."}
                      </span>
                    </div>
                    <span className="signal-tag">GRAPH</span>
                  </div>
                  <div className="signal">
                    <div className="signal-icon gold">
                      <Activity size={17} />
                    </div>
                    <div>
                      <b>Temporal coverage</b>
                      <span>
                        {dataset === "demo"
                          ? "Fictional dated rounds are seeded for event level timeline exploration."
                          : "Year filtering shows observation ranges, not exact dated funding rounds."}
                      </span>
                    </div>
                    <span className="signal-tag gold-tag">DATA NOTE</span>
                  </div>
                </div>
                <div className="source-card">
                  <div className="card-title">
                    <div>
                      <span className="mini-icon yellow-bg">
                        <Waypoints size={15} />
                      </span>
                      <h3>Dataset provenance</h3>
                    </div>
                    <Link to="/settings">
                      Details <ArrowUpRight size={14} />
                    </Link>
                  </div>
                  <div className="source-name">
                    <div className="source-logo">F</div>
                    <div>
                      <b>
                        {dataset === "demo"
                          ? "Fictional demo dataset"
                          : "Fliar co-investment sample"}
                      </b>
                      <span>
                        {dataset === "demo"
                          ? "Synthetic records · CC0-1.0"
                          : "Aggregate pair observations · CC BY-NC 4.0"}
                      </span>
                    </div>
                    <i className="source-live" />
                  </div>
                  <div className="source-note">
                    {dataset === "demo"
                      ? "Fictional demo data is separate from the real world sample. Round level events are synthetic illustrations."
                      : "This source has no dated round-level records. Year filters represent observed ranges, not exact event dates."}
                  </div>
                </div>
              </div>
            </>
          ) : (
            <ProductPage
              path={location.pathname}
              dataset={dataset}
              graph={data}
            />
          )}
        </section>
      </main>
    </div>
  );
}
function Stat({
  title,
  value,
  note,
  icon,
}: {
  title: string;
  value: string;
  note: string;
  icon: React.ReactNode;
}) {
  return (
    <div className="stat">
      <div className="stat-icon">{icon}</div>
      <span>{title}</span>
      <b>{value}</b>
      <small>{note}</small>
    </div>
  );
}
function format(x?: number) {
  return formatCount(x);
}
function pageTitle(path: string) {
  return (
    (
      {
        "/": "Overview",
        "/explorer": "Network explorer",
        "/investors": "Investors",
        "/communities": "Communities",
        "/analytics": "Market signals",
        "/timeline": "Temporal explorer",
        "/paths": "Relationship finder",
        "/compare": "Sector comparison",
        "/settings": "Settings",
      } as Record<string, string>
    )[path] ??
    (path.startsWith("/investors/") ? "Investor profile" : "VentureGraph")
  );
}
function ProductPage({
  path,
  dataset,
  graph,
}: {
  path: string;
  dataset: string;
  graph?: Graph;
}) {
  if (path === "/explorer") return <ExplorerPage graph={graph} />;
  if (path === "/investors") return <InvestorDirectory dataset={dataset} />;
  if (path.startsWith("/investors/"))
    return <InvestorProfile id={path.split("/")[2]} dataset={dataset} />;
  if (path === "/communities")
    return <CommunityPage dataset={dataset} graph={graph} />;
  if (path === "/analytics")
    return <AnalyticsPage dataset={dataset} graph={graph} />;
  if (path === "/timeline") return <TimelinePage dataset={dataset} />;
  if (path === "/paths") return <PathPage dataset={dataset} graph={graph} />;
  if (path === "/compare")
    return <ComparePage dataset={dataset} graph={graph} />;
  if (path === "/settings") return <SettingsPage />;
  return (
    <div className="product-page">
      <h1>Page not found</h1>
      <p>Choose a VentureGraph section from the navigation.</p>
      <Link to="/">Return to overview</Link>
    </div>
  );
}
function ProductHeader({
  eyebrow,
  title,
  description,
}: {
  eyebrow: string;
  title: string;
  description: string;
}) {
  return (
    <div className="product-header">
      <div className="eyebrow">
        <span />
        {eyebrow}
      </div>
      <h1>{title}</h1>
      <p>{description}</p>
    </div>
  );
}
function ExplorerPage({ graph }: { graph?: Graph }) {
  const [selected, setSelected] = useState<Investor | null>(null);
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="NETWORK EXPLORER"
        title="Explore investor connections"
        description="The graph shows the selected dataset projection. Edge width encodes observed shared-deal counts."
      />
      {graph?.edges.length ? (
        <div className="canvas-card product-canvas">
          <div className="canvas-top">
            <span className="legend">
              {graph.nodes.length} investors · {graph.edges.length}{" "}
              relationships
            </span>
          </div>
          <GraphView data={graph} selected={selected} onSelect={setSelected} />
        </div>
      ) : (
        <p className="empty-state">
          No relationships are available for this dataset yet.
        </p>
      )}
    </div>
  );
}
type InvestorPageResponse = {
  data: Investor[];
  page: number;
  per_page: number;
  total: number;
  total_pages: number;
};
function InvestorDirectory({ dataset }: { dataset: string }) {
  const [query, setQuery] = useState("");
  const [type, setType] = useState("");
  const [country, setCountry] = useState("");
  const [page, setPage] = useState(1);
  const result = useQuery({
    queryKey: ["investor-directory", dataset, query, type, country, page],
    queryFn: () =>
      fetcher<InvestorPageResponse>(
        `${API}/api/v1/investors?dataset=${dataset}&q=${encodeURIComponent(query)}&type=${encodeURIComponent(type)}&country=${encodeURIComponent(country)}&page=${page}&per_page=12`,
      ),
  });
  const investors = result.data?.data ?? [];
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="INVESTOR DIRECTORY"
        title="Investors"
        description="Search and filter investors across the selected dataset, independently of the rendered graph subset."
      />
      <div className="toolbar-panel">
        <input
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            setPage(1);
          }}
          placeholder="Search investor names"
        />
        <select
          aria-label="Investor type"
          value={type}
          onChange={(event) => {
            setType(event.target.value);
            setPage(1);
          }}
        >
          <option value="">All investor types</option>
          <option value="venture_capital">Venture capital</option>
          <option value="corporate_venture">Corporate venture</option>
          <option value="angel">Angel</option>
          <option value="unknown">Unknown</option>
        </select>
        <input
          aria-label="Country"
          value={country}
          onChange={(event) => {
            setCountry(event.target.value);
            setPage(1);
          }}
          placeholder="Country"
        />
        <span>{result.data?.total ?? "—"} results</span>
      </div>
      <div className="data-list">
        {investors.map((investor) => (
          <Link
            className="data-row"
            to={`/investors/${investor.id}`}
            key={investor.id}
          >
            <span className="avatar">
              {investor.name.slice(0, 1).toUpperCase()}
            </span>
            <span>
              <b>{investor.name}</b>
              <small>
                {investor.investor_type}
                {investor.country ? ` · ${investor.country}` : ""}
              </small>
            </span>
            <ArrowUpRight size={15} />
          </Link>
        ))}
        {!result.isPending && !investors.length && (
          <p className="empty-state">
            No investors match these filters in the selected dataset.
          </p>
        )}
      </div>
      <div className="toolbar-panel">
        <span>
          Page {page} of {result.data?.total_pages ?? 1}
        </span>
        <button disabled={page <= 1} onClick={() => setPage(page - 1)}>
          Previous
        </button>
        <button
          disabled={page >= (result.data?.total_pages ?? 1)}
          onClick={() => setPage(page + 1)}
        >
          Next
        </button>
      </div>
      {result.isError && (
        <p className="empty-state">Investor data could not be loaded.</p>
      )}
    </div>
  );
}
type Partner = {
  investor: Investor;
  shared_deals: number;
  first_year: number;
  last_year: number;
  top_shared_sector?: string | null;
};
function InvestorProfile({ id, dataset }: { id: string; dataset: string }) {
  const profile = useQuery({
    queryKey: ["investor-profile", dataset, id],
    queryFn: () =>
      fetcher<Investor>(`${API}/api/v1/investors/${id}?dataset=${dataset}`),
  });
  const partners = useQuery({
    queryKey: ["investor-partners", dataset, id],
    queryFn: () =>
      fetcher<Partner[]>(
        `${API}/api/v1/investors/${id}/partners?dataset=${dataset}`,
      ),
  });
  const centrality = useQuery({
    queryKey: ["centrality", dataset],
    queryFn: () =>
      fetcher<Centrality>(
        `${API}/api/v1/analytics/centrality?dataset=${dataset}`,
      ),
  });
  const investor = profile.data;
  const metrics = centrality.data?.investors.find(
    (item) => item.investor_id === id,
  );
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="INVESTOR PROFILE"
        title={
          investor?.name ??
          (profile.isPending ? "Loading investor…" : "Investor unavailable")
        }
        description={
          investor
            ? `${investor.investor_type}${investor.domain ? ` · ${investor.domain}` : ""}`
            : "Investor information is unavailable for this dataset."
        }
      />
      {investor && (
        <>
          <div className="stats">
            <Stat
              title="Connections"
              value={format(metrics?.degree)}
              note="Distinct observed partners"
              icon={<Users />}
            />
            <Stat
              title="Weighted degree"
              value={format(metrics?.weighted_degree)}
              note="Sum of shared deals"
              icon={<GitFork />}
            />
            <Stat
              title="PageRank"
              value={metrics?.pagerank.toFixed(3) ?? "—"}
              note="Weighted network rank"
              icon={<Activity />}
            />
            <Stat
              title="Partners shown"
              value={format(partners.data?.length)}
              note="Top observed partners"
              icon={<Waypoints />}
            />
          </div>
          <h2>Co-investment partners</h2>
          <div className="data-list">
            {partners.data?.map((item) => (
              <Link
                className="data-row"
                to={`/investors/${item.investor.id}`}
                key={item.investor.id}
              >
                <span>
                  <b>{item.investor.name}</b>
                  <small>
                    {item.shared_deals} shared deals · observed{" "}
                    {item.first_year}–{item.last_year}
                    {item.top_shared_sector
                      ? ` · ${item.top_shared_sector}`
                      : ""}
                  </small>
                </span>
                <ArrowUpRight size={14} />
              </Link>
            ))}
          </div>
          {partners.isError && (
            <p className="empty-state">Partner data could not be loaded.</p>
          )}
        </>
      )}
      {profile.isError && (
        <p className="empty-state">
          This investor does not appear in the selected dataset.
        </p>
      )}
    </div>
  );
}
type CommunityResponse = {
  algorithm: string;
  interpretation: string;
  communities: Array<{ id: string; size: number; members: string[] }>;
};
function CommunityPage({ dataset, graph }: { dataset: string; graph?: Graph }) {
  const result = useQuery({
    queryKey: ["communities", dataset],
    queryFn: () =>
      fetcher<CommunityResponse>(
        `${API}/api/v1/graph/communities?dataset=${dataset}`,
      ),
  });
  const names = new Map(
    (graph?.nodes ?? []).map((investor) => [investor.id, investor.name]),
  );
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="COMMUNITY ANALYSIS"
        title="Investor communities"
        description="Deterministic weighted label propagation creates a heuristic partition from co-investment ties."
      />
      <p className="method-note">
        {result.data?.interpretation ?? "Loading community method and results…"}
      </p>
      <div className="community-grid">
        {result.data?.communities.map((group, index) => (
          <article className="community-card" key={group.id}>
            <div className="card-title">
              <h2>Community {index + 1}</h2>
              <span>
                {group.size} {group.size === 1 ? "investor" : "investors"}
              </span>
            </div>
            <div className="data-list">
              {group.members.map((id) => (
                <Link className="data-row" to={`/investors/${id}`} key={id}>
                  <b>{names.get(id) ?? id}</b>
                  <ArrowUpRight size={14} />
                </Link>
              ))}
            </div>
          </article>
        ))}
      </div>
      {result.isError && (
        <p className="empty-state">Community data could not be loaded.</p>
      )}
    </div>
  );
}
type DiscoveryPair = {
  investor_a_id: string;
  investor_a: string;
  investor_b_id: string;
  investor_b: string;
  shared_deals: number;
  first_year: number;
  last_year: number;
  sector?: string | null;
  country?: string | null;
};
type EmergingInvestor = {
  id: string;
  name: string;
  first_observed_year: number;
  recent_new_connections: number;
  weighted_degree: number;
};
function AnalyticsPage({ dataset, graph }: { dataset: string; graph?: Graph }) {
  const [sector, setSector] = useState("");
  const [country, setCountry] = useState("");
  const [type, setType] = useState("");
  const [year, setYear] = useState("");
  const [minimum, setMinimum] = useState("");
  const result = useQuery({
    queryKey: ["centrality", dataset],
    queryFn: () =>
      fetcher<Centrality>(
        `${API}/api/v1/analytics/centrality?dataset=${dataset}`,
      ),
  });
  const pairQuery = new URLSearchParams({ dataset, limit: "20" });
  if (sector) pairQuery.set("sector", sector);
  if (country) pairQuery.set("country", country);
  if (type) pairQuery.set("investor_type", type);
  if (year) pairQuery.set("year", year);
  if (minimum) pairQuery.set("min_shared_deals", minimum);
  const pairs = useQuery({
    queryKey: [
      "co-investment-discovery",
      dataset,
      sector,
      country,
      type,
      year,
      minimum,
    ],
    queryFn: () =>
      fetcher<{ pairs: DiscoveryPair[] }>(
        `${API}/api/v1/analytics/co-investments?${pairQuery}`,
      ),
  });
  const emerging = useQuery({
    queryKey: ["emerging-investors", dataset],
    queryFn: () =>
      fetcher<{
        definition: string;
        interpretation: string;
        investors: EmergingInvestor[];
      }>(
        `${API}/api/v1/analytics/emerging?dataset=${dataset}&since_year=2024&limit=20`,
      ),
  });
  const names = new Map(
    (graph?.nodes ?? []).map((investor) => [investor.id, investor.name]),
  );
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="NETWORK ANALYTICS"
        title="Market signals"
        description="Compare centrality, high weight co-investment relationships, and recent source-reported observations."
      />
      <h2>Centrality rankings</h2>
      <div className="metric-note">
        PageRank uses shared-deal weighted transitions. Betweenness is
        unweighted Brandes centrality. Metrics cover the selected full graph.
      </div>
      <div className="data-list">
        {result.data?.investors.map((item, index) => (
          <div className="data-row rank-row" key={item.investor_id}>
            <span className="rank">{String(index + 1).padStart(2, "0")}</span>
            <span>
              <b>{names.get(item.investor_id) ?? item.investor_id}</b>
              <small>
                Degree {item.degree} · Weighted degree {item.weighted_degree} ·
                Betweenness {item.betweenness.toFixed(3)} · Clustering{" "}
                {item.clustering_coefficient.toFixed(2)}
              </small>
            </span>
            <strong>{item.pagerank.toFixed(3)}</strong>
          </div>
        ))}
      </div>
      {result.isError && (
        <p className="empty-state">Centrality could not be loaded.</p>
      )}
      <h2>Top co-investment pairs</h2>
      <div className="toolbar-panel">
        <input
          aria-label="Sector filter"
          value={sector}
          onChange={(event) => setSector(event.target.value)}
          placeholder="Sector"
        />
        <input
          aria-label="Country filter"
          value={country}
          onChange={(event) => setCountry(event.target.value)}
          placeholder="Country"
        />
        <select
          aria-label="Investor type filter"
          value={type}
          onChange={(event) => setType(event.target.value)}
        >
          <option value="">Any investor type</option>
          <option value="venture_capital">Venture capital</option>
          <option value="corporate_venture">Corporate venture</option>
          <option value="angel">Angel</option>
          <option value="growth_equity">Growth equity</option>
        </select>
        <input
          aria-label="Observed year filter"
          type="number"
          value={year}
          onChange={(event) => setYear(event.target.value)}
          placeholder="Observed year"
        />
        <input
          aria-label="Minimum shared deals"
          type="number"
          min="0"
          value={minimum}
          onChange={(event) => setMinimum(event.target.value)}
          placeholder="Min deals"
        />
      </div>
      <p className="method-note">
        Year filters match source-reported observation ranges; they do not
        assert an exact event date.
      </p>
      <div className="data-list">
        {pairs.data?.pairs.map((pair, index) => (
          <div
            className="data-row rank-row"
            key={`${pair.investor_a_id}-${pair.investor_b_id}`}
          >
            <span className="rank">{String(index + 1).padStart(2, "0")}</span>
            <span>
              <b>
                {pair.investor_a} · {pair.investor_b}
              </b>
              <small>
                {pair.shared_deals} shared deals · {pair.first_year}–
                {pair.last_year}
                {pair.sector ? ` · ${pair.sector}` : ""}
              </small>
            </span>
            <strong>{pair.country ?? "—"}</strong>
          </div>
        ))}
      </div>
      {pairs.isError && (
        <p className="empty-state">
          Co-investment rankings could not be loaded.
        </p>
      )}
      {pairs.data?.pairs.length === 0 && (
        <p className="empty-state">No observed pairs match these filters.</p>
      )}
      <h2>Recent first observations</h2>
      <p className="method-note">
        {emerging.data?.interpretation ??
          "Ranks pair records whose source-reported first year is 2024 or later. This is not a verified annual growth rate."}
      </p>
      <div className="data-list">
        {emerging.data?.investors.map((investor, index) => (
          <Link
            className="data-row rank-row"
            key={investor.id}
            to={`/investors/${investor.id}`}
          >
            <span className="rank">{String(index + 1).padStart(2, "0")}</span>
            <span>
              <b>{investor.name}</b>
              <small>
                First observed {investor.first_observed_year} ·{" "}
                {investor.recent_new_connections} recent first-observed pair
                records
              </small>
            </span>
            <strong>WD {investor.weighted_degree}</strong>
          </Link>
        ))}
      </div>
      {emerging.isError && (
        <p className="empty-state">
          Emerging investor rankings could not be loaded.
        </p>
      )}
    </div>
  );
}
function PathPage({ dataset, graph }: { dataset: string; graph?: Graph }) {
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [mode, setMode] = useState("hops");
  const result = useQuery({
    queryKey: ["path", dataset, from, to, mode],
    enabled: Boolean(from && to && from !== to),
    queryFn: () =>
      fetcher<{
        found: boolean;
        path: string[];
        hops: number;
        mode: string;
        total_cost: number;
        cost_definition: string;
        edge_weights: number[];
      }>(
        `${API}/api/v1/path?from=${from}&to=${to}&mode=${mode}&dataset=${dataset}`,
      ),
  });
  const names = new Map(
    (graph?.nodes ?? []).map((investor) => [investor.id, investor.name]),
  );
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="RELATIONSHIP FINDER"
        title="Find an investor route"
        description="Compare fewest hops with the strongest connection route. Each mode optimizes a different cost."
      />
      <div className="toolbar-panel">
        <label>
          From
          <select
            value={from}
            onChange={(event) => setFrom(event.target.value)}
          >
            <option value="">Select investor</option>
            {graph?.nodes.map((investor) => (
              <option key={investor.id} value={investor.id}>
                {investor.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          To
          <select value={to} onChange={(event) => setTo(event.target.value)}>
            <option value="">Select investor</option>
            {graph?.nodes.map((investor) => (
              <option key={investor.id} value={investor.id}>
                {investor.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          Mode
          <select
            value={mode}
            onChange={(event) => setMode(event.target.value)}
          >
            <option value="hops">Fewest hops</option>
            <option value="strongest">Strongest connection</option>
          </select>
        </label>
      </div>
      {result.data && (
        <div className="result-panel">
          {result.data.found ? (
            <>
              <h2>
                {result.data.path
                  .map((id) => names.get(id) ?? id)
                  .join("  →  ")}
              </h2>
              <p>
                {result.data.hops} hops · route cost{" "}
                {result.data.total_cost.toFixed(3)} ·{" "}
                {result.data.cost_definition}
              </p>
              <p>
                Shared deals on each tie: {result.data.edge_weights.join(" · ")}
              </p>
            </>
          ) : (
            <p>
              No connection path exists between these investors in this dataset.
            </p>
          )}
        </div>
      )}
      {result.isError && (
        <p className="empty-state">
          Path calculation failed. Check that both investors belong to this
          dataset.
        </p>
      )}
    </div>
  );
}
type DiffResponse = {
  period_a: number;
  period_b: number;
  evidence_mode: string;
  newly_observed: Array<[string, string]>;
  absent_in_period_b: Array<[string, string]>;
  retained: Array<[string, string]>;
  interpretation: string;
};
function TimelinePage({ dataset }: { dataset: string }) {
  const [yearA, setYearA] = useState("2021");
  const [yearB, setYearB] = useState("2024");
  const diff = useQuery({
    queryKey: ["temporal-diff", dataset, yearA, yearB],
    enabled: dataset === "demo",
    queryFn: () =>
      fetcher<DiffResponse>(
        `${API}/api/v1/temporal/diff?year_a=${yearA}&year_b=${yearB}&dataset=${dataset}`,
      ),
  });
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="TEMPORAL EXPLORER"
        title="Compare observed activity"
        description="Dated synthetic demo rounds support exact event-year comparisons. Aggregate sources cannot establish activity in an exact year."
      />
      <div className="toolbar-panel">
        <label>
          Period A
          <select
            value={yearA}
            onChange={(event) => setYearA(event.target.value)}
          >
            {[2021, 2022, 2023, 2024].map((year) => (
              <option key={year}>{year}</option>
            ))}
          </select>
        </label>
        <label>
          Period B
          <select
            value={yearB}
            onChange={(event) => setYearB(event.target.value)}
          >
            {[2021, 2022, 2023, 2024].map((year) => (
              <option key={year}>{year}</option>
            ))}
          </select>
        </label>
        <span>
          Dataset:{" "}
          {dataset === "demo"
            ? "dated synthetic events"
            : "aggregate observations"}
        </span>
      </div>
      {dataset === "demo" ? (
        <>
          {diff.data && (
            <>
              <p className="method-note">{diff.data.interpretation}</p>
              <div className="stats">
                <Stat
                  title="New in period B"
                  value={format(diff.data.newly_observed.length)}
                  note="Observed in B, not A"
                  icon={<GitFork />}
                />
                <Stat
                  title="Absent in period B"
                  value={format(diff.data.absent_in_period_b.length)}
                  note="Observed in A, not B"
                  icon={<Activity />}
                />
                <Stat
                  title="Retained"
                  value={format(diff.data.retained.length)}
                  note="Observed in both periods"
                  icon={<Waypoints />}
                />
              </div>
            </>
          )}
          {diff.isError && (
            <p className="empty-state">Temporal comparison failed.</p>
          )}
        </>
      ) : (
        <p className="method-note">
          This aggregate source provides first and last observed years only. It
          cannot verify year-by-year activity or answer this exact comparison.
        </p>
      )}
    </div>
  );
}
function ComparePage({ dataset, graph }: { dataset: string; graph?: Graph }) {
  const sectors = Array.from(
    new Set(
      (graph?.edges ?? [])
        .map((edge) => edge.sector)
        .filter((sector): sector is string => Boolean(sector)),
    ),
  ).sort();
  const [left, setLeft] = useState("");
  const [right, setRight] = useState("");
  const a = useQuery({
    queryKey: ["sector-graph", dataset, left],
    enabled: Boolean(left),
    queryFn: () =>
      fetcher<Graph>(
        `${API}/api/v1/graph?dataset=${dataset}&sector=${encodeURIComponent(left)}&limit=1500`,
      ),
  });
  const b = useQuery({
    queryKey: ["sector-graph", dataset, right],
    enabled: Boolean(right),
    queryFn: () =>
      fetcher<Graph>(
        `${API}/api/v1/graph?dataset=${dataset}&sector=${encodeURIComponent(right)}&limit=1500`,
      ),
  });
  const compare = (name: string, result?: Graph) => (
    <article className="community-card">
      <h2>{name || "Choose a sector"}</h2>
      <p>
        {result
          ? `${result.nodes.length} investors · ${result.edges.length} observed pairs`
          : "Select a sector to calculate its subgraph."}
      </p>
      <p className="method-note">Filters by the pair's top shared sector.</p>
    </article>
  );
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="SECTOR COMPARISON"
        title="Compare pair sectors"
        description="Sector membership reflects each pair’s dominant shared sector in the source data."
      />
      <div className="toolbar-panel">
        <label>
          Sector A
          <select
            value={left}
            onChange={(event) => setLeft(event.target.value)}
          >
            <option value="">Select sector</option>
            {sectors.map((sector) => (
              <option key={sector}>{sector}</option>
            ))}
          </select>
        </label>
        <label>
          Sector B
          <select
            value={right}
            onChange={(event) => setRight(event.target.value)}
          >
            <option value="">Select sector</option>
            {sectors.map((sector) => (
              <option key={sector}>{sector}</option>
            ))}
          </select>
        </label>
      </div>
      <div className="community-grid">
        {compare(left, a.data)}
        {compare(right, b.data)}
      </div>
    </div>
  );
}
function SettingsPage() {
  const sources = useQuery({
    queryKey: ["data-sources"],
    queryFn: () =>
      fetcher<
        Array<{
          id: string;
          name: string;
          license: string;
          source_url?: string;
          methodology?: string;
        }>
      >(`${API}/api/v1/data-sources`),
  });
  const coverage = useQuery({
    queryKey: ["data-coverage"],
    queryFn: () =>
      fetcher<
        Array<{
          source_id: string;
          source: string;
          investors: number;
          aggregate_pairs: number;
          dated_rounds: number;
          first_observed_year?: number;
          last_observed_year?: number;
        }>
      >(`${API}/api/v1/data-coverage`),
  });
  return (
    <div className="product-page">
      <ProductHeader
        eyebrow="DATA & PREFERENCES"
        title="Sources and coverage"
        description="Review source licensing and distinguish aggregate observations from dated event data."
      />
      {sources.data?.map((source) => {
        const item = coverage.data?.find(
          (coverageItem) => coverageItem.source_id === source.id,
        );
        return (
          <article className="source-card settings-source" key={source.id}>
            <div className="card-title">
              <h2>{source.name}</h2>
              <span>{source.license}</span>
            </div>
            <p>{source.methodology}</p>
            <p>
              {item
                ? `${item.investors} investors · ${item.aggregate_pairs} aggregate pairs · ${item.dated_rounds} dated rounds`
                : "Coverage unavailable"}
            </p>
            {source.source_url && (
              <a href={source.source_url} target="_blank" rel="noreferrer">
                Source documentation <ArrowUpRight size={14} />
              </a>
            )}
          </article>
        );
      })}
      {sources.isError && (
        <p className="empty-state">Source information could not be loaded.</p>
      )}
    </div>
  );
}
function download(g: Graph) {
  const blob = new Blob([JSON.stringify(g, null, 2)], {
    type: "application/json",
  });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = "venturegraph-filtered-network.json";
  a.click();
  URL.revokeObjectURL(a.href);
}
