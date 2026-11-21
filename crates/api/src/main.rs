use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
};
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    net::SocketAddr,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct App {
    pool: PgPool,
}
#[derive(Serialize)]
struct Investor {
    id: Uuid,
    name: String,
    investor_type: String,
    domain: Option<String>,
    country: Option<String>,
}
#[derive(Serialize)]
struct Edge {
    source: Uuid,
    target: Uuid,
    weight: i32,
    first_year: i32,
    last_year: i32,
    sector: Option<String>,
}
#[derive(Serialize)]
struct Graph {
    dataset: String,
    temporal_capability: &'static str,
    nodes: Vec<Investor>,
    edges: Vec<Edge>,
}
#[derive(Deserialize)]
struct GraphQuery {
    year: Option<i32>,
    sector: Option<String>,
    limit: Option<i64>,
    dataset: Option<String>,
}
#[derive(Deserialize)]
struct TemporalQuery {
    year: Option<i32>,
    dataset: Option<String>,
}
#[derive(Serialize)]
struct PathResult {
    found: bool,
    path: Vec<Uuid>,
    hops: usize,
    mode: String,
    cost_definition: String,
    total_cost: f64,
    edge_weights: Vec<f64>,
}
#[derive(Serialize)]
struct InvestorPage {
    data: Vec<Investor>,
    page: i64,
    per_page: i64,
    total: i64,
    total_pages: i64,
}

#[derive(Deserialize)]
struct DiscoveryQuery {
    dataset: Option<String>,
    sector: Option<String>,
    country: Option<String>,
    investor_type: Option<String>,
    year: Option<i32>,
    min_shared_deals: Option<i32>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct EmergingQuery {
    dataset: Option<String>,
    since_year: Option<i32>,
    limit: Option<i64>,
}

async fn coinvestment_discovery(
    State(s): State<App>,
    Query(q): Query<DiscoveryQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset = q.dataset.as_deref().unwrap_or("demo");
    let source = source_id(dataset)?;
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    if q.min_shared_deals.is_some_and(|n| n < 0) {
        return Err((
            StatusCode::BAD_REQUEST,
            "min_shared_deals must be non-negative".into(),
        ));
    }
    let rows = sqlx::query("SELECT p.shared_deals,p.first_year,p.last_year,p.top_shared_sector,p.top_shared_country,a.id investor_a_id,a.canonical_name investor_a,b.id investor_b_id,b.canonical_name investor_b FROM aggregate_pairs p JOIN investors a ON a.id=p.investor_a_id JOIN investors b ON b.id=p.investor_b_id WHERE p.source_id=$1 AND ($2::text IS NULL OR p.top_shared_sector ILIKE $2) AND ($3::text IS NULL OR p.top_shared_country ILIKE $3) AND ($4::text IS NULL OR a.investor_type=$4 OR b.investor_type=$4) AND ($5::int IS NULL OR (p.first_year <= $5 AND p.last_year >= $5)) AND p.shared_deals >= $6 ORDER BY p.shared_deals DESC,a.canonical_name,b.canonical_name LIMIT $7")
        .bind(source)
        .bind(q.sector.as_ref().map(|v| format!("%{v}%")))
        .bind(q.country.as_ref().map(|v| format!("%{v}%")))
        .bind(q.investor_type.as_deref())
        .bind(q.year)
        .bind(q.min_shared_deals.unwrap_or(0))
        .bind(limit)
        .fetch_all(&s.pool).await.map_err(internal)?;
    let pairs = rows
        .into_iter()
        .map(|r| {
            Ok(serde_json::json!({
                "investor_a_id": r.try_get::<Uuid,_>("investor_a_id").map_err(internal)?,
                "investor_a": r.try_get::<String,_>("investor_a").map_err(internal)?,
                "investor_b_id": r.try_get::<Uuid,_>("investor_b_id").map_err(internal)?,
                "investor_b": r.try_get::<String,_>("investor_b").map_err(internal)?,
                "shared_deals": r.try_get::<i32,_>("shared_deals").map_err(internal)?,
                "first_year": r.try_get::<i32,_>("first_year").map_err(internal)?,
                "last_year": r.try_get::<i32,_>("last_year").map_err(internal)?,
                "sector": r.try_get::<Option<String>,_>("top_shared_sector").map_err(internal)?,
                "country": r.try_get::<Option<String>,_>("top_shared_country").map_err(internal)?,
            }))
        })
        .collect::<Result<Vec<_>, (StatusCode, String)>>()?;
    Ok(Json(
        serde_json::json!({"dataset":dataset,"evidence_mode":"aggregate_pair_observations","interpretation":"A year filter selects observed ranges containing that year; it does not establish an exact event in that year.","pairs":pairs}),
    ))
}

async fn emerging_investors(
    State(s): State<App>,
    Query(q): Query<EmergingQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset = q.dataset.as_deref().unwrap_or("demo");
    let source = source_id(dataset)?;
    let since_year = q.since_year.unwrap_or(2024);
    if !(1..=9999).contains(&since_year) {
        return Err((
            StatusCode::BAD_REQUEST,
            "since_year must be between 1 and 9999".into(),
        ));
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let rows = sqlx::query("SELECT i.id,i.canonical_name,i.investor_type,MIN(p.first_year)::int first_observed_year,COUNT(*) FILTER (WHERE p.first_year >= $2)::int recent_new_connections,COUNT(*)::int observed_connections,COALESCE(SUM(p.shared_deals),0)::bigint weighted_degree FROM investors i JOIN aggregate_pairs p ON (p.investor_a_id=i.id OR p.investor_b_id=i.id) WHERE p.source_id=$1 GROUP BY i.id,i.canonical_name,i.investor_type HAVING MIN(p.first_year) >= $2 OR COUNT(*) FILTER (WHERE p.first_year >= $2) > 0 ORDER BY recent_new_connections DESC,first_observed_year DESC,weighted_degree DESC,i.canonical_name LIMIT $3")
        .bind(source).bind(since_year).bind(limit).fetch_all(&s.pool).await.map_err(internal)?;
    let investors = rows.into_iter().map(|r| Ok(serde_json::json!({
        "id":r.try_get::<Uuid,_>("id").map_err(internal)?,
        "name":r.try_get::<String,_>("canonical_name").map_err(internal)?,
        "investor_type":r.try_get::<String,_>("investor_type").map_err(internal)?,
        "first_observed_year":r.try_get::<i32,_>("first_observed_year").map_err(internal)?,
        "recent_new_connections":r.try_get::<i32,_>("recent_new_connections").map_err(internal)?,
        "observed_connections":r.try_get::<i32,_>("observed_connections").map_err(internal)?,
        "weighted_degree":r.try_get::<i64,_>("weighted_degree").map_err(internal)?,
    }))).collect::<Result<Vec<_>,(StatusCode,String)>>()?;
    Ok(Json(
        serde_json::json!({"dataset":dataset,"since_year":since_year,"definition":"Ranks investors by the count of aggregate pair records whose first observed year is at or after since_year; tie breaks use latest first-observed year and weighted degree.","evidence_mode":"aggregate_pair_observations","interpretation":"This is a discovery heuristic based on source-reported first years, not a verified annual growth rate.","investors":investors}),
    ))
}

async fn health(State(s): State<App>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&s.pool).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status":"ok"}))),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status":"database_unavailable"})),
        ),
    }
}

async fn meta() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "VentureGraph API",
        "version": env!("CARGO_PKG_VERSION"),
        "api_version": "v1",
        "graph_algorithms": ["degree", "weighted_degree", "pagerank", "connected_components", "fewest_hops", "strongest_connection_route", "label_propagation", "local_clustering", "common_neighbors", "jaccard", "betweenness"],
    }))
}

async fn openapi_schema() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    serde_json::from_str(include_str!("../../../docs/openapi.json"))
        .map(Json)
        .map_err(internal)
}

async fn api_docs() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>VentureGraph API</title><style>body{font:16px system-ui;max-width:820px;margin:4rem auto;padding:0 1rem;background:#101512;color:#e6ece5}a{color:#c6e79a}code{color:#c6e79a}li{margin:.8rem 0}.note{color:#a9b5aa}</style><h1>VentureGraph API</h1><p>Versioned, read-only API for local graph exploration.</p><ul><li><a href="/api/v1/openapi.json">OpenAPI 3.1 JSON schema</a></li><li><a href="/health">Health check</a></li><li><a href="/api/v1/data-sources">Data source provenance</a></li><li><a href="/api/v1/graph?dataset=demo">Fictional demonstration graph</a></li></ul><p class="note">Fliar records are aggregate pair observations. They do not provide exact round dates or verified activity in each intervening year.</p></html>"#,
    )
}

async fn data_sources(
    State(s): State<App>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT id,name,source_url,license,methodology,created_at FROM data_sources ORDER BY name",
    )
    .fetch_all(&s.pool)
    .await
    .map_err(internal)?;
    let sources = rows.into_iter().map(|row| {
        Ok(serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").map_err(internal)?,
            "name": row.try_get::<String, _>("name").map_err(internal)?,
            "source_url": row.try_get::<Option<String>, _>("source_url").map_err(internal)?,
            "license": row.try_get::<String, _>("license").map_err(internal)?,
            "methodology": row.try_get::<Option<String>, _>("methodology").map_err(internal)?,
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").map_err(internal)?,
        }))
    }).collect::<Result<Vec<_>, (StatusCode, String)>>()?;
    Ok(Json(sources))
}

async fn data_coverage(
    State(s): State<App>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let rows = sqlx::query("SELECT ds.id,ds.name,COUNT(DISTINCT p.id)::bigint pair_count,COUNT(DISTINCT r.id)::bigint dated_round_count,MIN(p.first_year) first_observed_year,MAX(p.last_year) last_observed_year,(SELECT COUNT(DISTINCT investor_id)::bigint FROM (SELECT investor_a_id investor_id FROM aggregate_pairs WHERE source_id=ds.id UNION SELECT investor_b_id FROM aggregate_pairs WHERE source_id=ds.id) i) investor_count FROM data_sources ds LEFT JOIN aggregate_pairs p ON p.source_id=ds.id LEFT JOIN funding_rounds r ON r.source_id=ds.id GROUP BY ds.id,ds.name ORDER BY ds.name")
        .fetch_all(&s.pool).await.map_err(internal)?;
    let coverage = rows.into_iter().map(|row| {
        Ok(serde_json::json!({
            "source_id": row.try_get::<Uuid, _>("id").map_err(internal)?,
            "source": row.try_get::<String, _>("name").map_err(internal)?,
            "investors": row.try_get::<i64, _>("investor_count").map_err(internal)?,
            "aggregate_pairs": row.try_get::<i64, _>("pair_count").map_err(internal)?,
            "dated_rounds": row.try_get::<i64, _>("dated_round_count").map_err(internal)?,
            "first_observed_year": row.try_get::<Option<i32>, _>("first_observed_year").map_err(internal)?,
            "last_observed_year": row.try_get::<Option<i32>, _>("last_observed_year").map_err(internal)?,
        }))
    }).collect::<Result<Vec<_>, (StatusCode, String)>>()?;
    Ok(Json(coverage))
}
async fn graph(
    State(s): State<App>,
    Query(q): Query<GraphQuery>,
) -> Result<Json<Graph>, (StatusCode, String)> {
    let limit = q.limit.unwrap_or(2000).clamp(1, 10000);
    let dataset = q.dataset.unwrap_or_else(|| "demo".into());
    let source = source_id(&dataset)?;
    let rows = sqlx::query("SELECT a.id aid,a.canonical_name an,a.investor_type at,a.website_domain ad,a.country ac,b.id bid,b.canonical_name bn,b.investor_type bt,b.website_domain bd,b.country bc,p.shared_deals,p.first_year,p.last_year,p.top_shared_sector FROM aggregate_pairs p JOIN investors a ON a.id=p.investor_a_id JOIN investors b ON b.id=p.investor_b_id WHERE p.source_id=$1 AND ($2::int IS NULL OR (p.first_year <= $2 AND p.last_year >= $2)) AND ($3::text IS NULL OR p.top_shared_sector = $3) ORDER BY p.shared_deals DESC,p.investor_a_id,p.investor_b_id LIMIT $4")
        .bind(source).bind(q.year).bind(q.sector).bind(limit).fetch_all(&s.pool).await.map_err(internal)?;
    let mut nodes: HashMap<Uuid, Investor> = HashMap::new();
    let mut edges = Vec::with_capacity(rows.len());
    for r in rows {
        let aid: Uuid = r.try_get("aid").map_err(internal)?;
        let bid: Uuid = r.try_get("bid").map_err(internal)?;
        nodes.entry(aid).or_insert(Investor {
            id: aid,
            name: r.try_get("an").map_err(internal)?,
            investor_type: r.try_get("at").map_err(internal)?,
            domain: r.try_get("ad").map_err(internal)?,
            country: r.try_get("ac").map_err(internal)?,
        });
        nodes.entry(bid).or_insert(Investor {
            id: bid,
            name: r.try_get("bn").map_err(internal)?,
            investor_type: r.try_get("bt").map_err(internal)?,
            domain: r.try_get("bd").map_err(internal)?,
            country: r.try_get("bc").map_err(internal)?,
        });
        edges.push(Edge {
            source: aid,
            target: bid,
            weight: r.try_get("shared_deals").map_err(internal)?,
            first_year: r.try_get("first_year").map_err(internal)?,
            last_year: r.try_get("last_year").map_err(internal)?,
            sector: r.try_get("top_shared_sector").map_err(internal)?,
        });
    }
    let mut graph_nodes: Vec<_> = nodes.into_values().collect();
    graph_nodes.sort_by_key(|node| node.id);
    Ok(Json(Graph {
        dataset,
        temporal_capability: "aggregate_observation_range",
        nodes: graph_nodes,
        edges,
    }))
}
async fn investors(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<InvestorPage>, (StatusCode, String)> {
    let term = q.get("q").cloned().unwrap_or_default();
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let parse_number = |name: &str, default: i64| -> Result<i64, (StatusCode, String)> {
        q.get(name).map_or(Ok(default), |value| {
            value.parse::<i64>().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("{name} must be an integer"),
                )
            })
        })
    };
    let page = parse_number("page", 1)?;
    if page < 1 {
        return Err((StatusCode::BAD_REQUEST, "page must be positive".into()));
    }
    let per_page = parse_number("per_page", 50)?.clamp(1, 200);
    let offset = (page - 1).saturating_mul(per_page);
    let investor_type = q.get("type").filter(|value| !value.is_empty());
    let country = q.get("country").filter(|value| !value.is_empty());
    let count_row = sqlx::query("SELECT COUNT(*)::bigint total FROM (SELECT investor_a_id investor_id FROM aggregate_pairs WHERE source_id=$1 UNION SELECT investor_b_id FROM aggregate_pairs WHERE source_id=$1) ids JOIN investors i ON i.id=ids.investor_id WHERE i.canonical_name ILIKE $2 AND ($3::text IS NULL OR i.investor_type=$3) AND ($4::text IS NULL OR i.country=$4)")
        .bind(source).bind(format!("%{term}%")).bind(investor_type).bind(country).fetch_one(&s.pool).await.map_err(internal)?;
    let total: i64 = count_row.try_get("total").map_err(internal)?;
    let rows=sqlx::query("SELECT i.id,i.canonical_name,i.investor_type,i.website_domain,i.country FROM (SELECT investor_a_id investor_id FROM aggregate_pairs WHERE source_id=$1 UNION SELECT investor_b_id FROM aggregate_pairs WHERE source_id=$1) ids JOIN investors i ON i.id=ids.investor_id WHERE i.canonical_name ILIKE $2 AND ($3::text IS NULL OR i.investor_type=$3) AND ($4::text IS NULL OR i.country=$4) ORDER BY i.canonical_name,i.id LIMIT $5 OFFSET $6")
        .bind(source).bind(format!("%{term}%")).bind(investor_type).bind(country).bind(per_page).bind(offset).fetch_all(&s.pool).await.map_err(internal)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(Investor {
            id: r.try_get("id").map_err(internal)?,
            name: r.try_get("canonical_name").map_err(internal)?,
            investor_type: r.try_get("investor_type").map_err(internal)?,
            domain: r.try_get("website_domain").map_err(internal)?,
            country: r.try_get("country").map_err(internal)?,
        });
    }
    Ok(Json(InvestorPage {
        data: out,
        page,
        per_page,
        total,
        total_pages: total / per_page + i64::from(total % per_page != 0),
    }))
}

async fn investor_detail(
    State(s): State<App>,
    Path(raw_id): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<Investor>, (StatusCode, String)> {
    let id = Uuid::parse_str(&raw_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "id must be a UUID".into()))?;
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let row = sqlx::query("SELECT i.id,i.canonical_name,i.investor_type,i.website_domain,i.country FROM investors i WHERE i.id=$1 AND EXISTS(SELECT 1 FROM aggregate_pairs p WHERE p.source_id=$2 AND (p.investor_a_id=i.id OR p.investor_b_id=i.id))")
        .bind(id).bind(source).fetch_optional(&s.pool).await.map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "investor not found in selected dataset".into()))?;
    Ok(Json(Investor {
        id: row.try_get("id").map_err(internal)?,
        name: row.try_get("canonical_name").map_err(internal)?,
        investor_type: row.try_get("investor_type").map_err(internal)?,
        domain: row.try_get("website_domain").map_err(internal)?,
        country: row.try_get("country").map_err(internal)?,
    }))
}

async fn investor_partners(
    State(s): State<App>,
    Path(raw_id): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let id = Uuid::parse_str(&raw_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "id must be a UUID".into()))?;
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let limit = q
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 200);
    let rows = sqlx::query("SELECT partner.id,partner.canonical_name,partner.investor_type,p.shared_deals,p.first_year,p.last_year,p.top_shared_sector FROM aggregate_pairs p JOIN investors partner ON partner.id=CASE WHEN p.investor_a_id=$2 THEN p.investor_b_id ELSE p.investor_a_id END WHERE p.source_id=$1 AND (p.investor_a_id=$2 OR p.investor_b_id=$2) ORDER BY p.shared_deals DESC,partner.canonical_name LIMIT $3")
        .bind(source).bind(id).bind(limit).fetch_all(&s.pool).await.map_err(internal)?;
    rows.into_iter().map(|row| Ok(serde_json::json!({
        "investor": Investor {
            id: row.try_get("id").map_err(internal)?,
            name: row.try_get("canonical_name").map_err(internal)?,
            investor_type: row.try_get("investor_type").map_err(internal)?,
            domain: None,
            country: None,
        },
        "shared_deals": row.try_get::<i32, _>("shared_deals").map_err(internal)?,
        "first_year": row.try_get::<i32, _>("first_year").map_err(internal)?,
        "last_year": row.try_get::<i32, _>("last_year").map_err(internal)?,
        "top_shared_sector": row.try_get::<Option<String>, _>("top_shared_sector").map_err(internal)?,
    }))).collect::<Result<Vec<_>, (StatusCode, String)>>().map(Json)
}
async fn temporal(
    State(s): State<App>,
    Query(q): Query<TemporalQuery>,
) -> Result<Json<Graph>, (StatusCode, String)> {
    let year = q
        .year
        .ok_or((StatusCode::BAD_REQUEST, "year is required".into()))?;
    let dataset = q.dataset.unwrap_or_else(|| "demo".into());
    if dataset != "demo" {
        return Err((
            StatusCode::BAD_REQUEST,
            "exact dated events are available only for demo data".into(),
        ));
    }
    let source = source_id(&dataset)?;
    let rows = sqlx::query("SELECT a.id aid,a.canonical_name an,a.investor_type at,a.website_domain ad,a.country ac,b.id bid,b.canonical_name bn,b.investor_type bt,b.website_domain bd,b.country bc,COUNT(DISTINCT r.id)::int shared_deals,NULL::text sector FROM funding_rounds r JOIN round_participants pa ON pa.round_id=r.id JOIN round_participants pb ON pb.round_id=r.id AND pa.investor_id < pb.investor_id JOIN investors a ON a.id=pa.investor_id JOIN investors b ON b.id=pb.investor_id WHERE r.source_id=$1 AND EXTRACT(YEAR FROM r.announced_at)=$2 GROUP BY a.id,a.canonical_name,a.investor_type,a.website_domain,a.country,b.id,b.canonical_name,b.investor_type,b.website_domain,b.country ORDER BY a.id,b.id")
        .bind(source).bind(year).fetch_all(&s.pool).await.map_err(internal)?;
    let mut nodes = HashMap::new();
    let mut edges = Vec::with_capacity(rows.len());
    for r in rows {
        let aid: Uuid = r.try_get("aid").map_err(internal)?;
        let bid: Uuid = r.try_get("bid").map_err(internal)?;
        for (id, name, kind, domain, country) in [
            (
                aid,
                r.try_get::<String, _>("an").map_err(internal)?,
                r.try_get::<String, _>("at").map_err(internal)?,
                r.try_get("ad").map_err(internal)?,
                r.try_get("ac").map_err(internal)?,
            ),
            (
                bid,
                r.try_get::<String, _>("bn").map_err(internal)?,
                r.try_get::<String, _>("bt").map_err(internal)?,
                r.try_get("bd").map_err(internal)?,
                r.try_get("bc").map_err(internal)?,
            ),
        ] {
            nodes.entry(id).or_insert(Investor {
                id,
                name,
                investor_type: kind,
                domain,
                country,
            });
        }
        edges.push(Edge {
            source: aid,
            target: bid,
            weight: r.try_get("shared_deals").map_err(internal)?,
            first_year: year,
            last_year: year,
            sector: r.try_get("sector").map_err(internal)?,
        });
    }
    let mut graph_nodes: Vec<_> = nodes.into_values().collect();
    graph_nodes.sort_by_key(|node| node.id);
    Ok(Json(Graph {
        dataset: format!("demo-dated-{year}"),
        temporal_capability: "synthetic_dated_events",
        nodes: graph_nodes,
        edges,
    }))
}

async fn temporal_diff(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parse_year = |key: &str| -> Result<i32, (StatusCode, String)> {
        let year = q
            .get(key)
            .and_then(|value| value.parse::<i32>().ok())
            .ok_or((StatusCode::BAD_REQUEST, format!("{key} must be a year")))?;
        if !(1900..=2100).contains(&year) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("{key} must be between 1900 and 2100"),
            ));
        }
        Ok(year)
    };
    let year_a = parse_year("year_a")?;
    let year_b = parse_year("year_b")?;
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    if dataset != "demo" {
        return Err((StatusCode::BAD_REQUEST, "exact period comparison requires dated round events; the selected aggregate source cannot answer this query".into()));
    }
    let source = source_id(dataset)?;
    let pairs_a = event_pairs_for_year(&s.pool, source, year_a).await?;
    let pairs_b = event_pairs_for_year(&s.pool, source, year_b).await?;
    let mut added: Vec<_> = pairs_b.difference(&pairs_a).copied().collect();
    let mut removed: Vec<_> = pairs_a.difference(&pairs_b).copied().collect();
    let mut retained: Vec<_> = pairs_a.intersection(&pairs_b).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();
    retained.sort_unstable();
    Ok(Json(serde_json::json!({
        "dataset": dataset,
        "evidence_mode": "synthetic_dated_events",
        "period_a": year_a,
        "period_b": year_b,
        "newly_observed": added,
        "absent_in_period_b": removed,
        "retained": retained,
        "interpretation": "A relationship absent from period B was not observed in that period; this does not establish that a business or legal relationship ended.",
    })))
}

async fn event_pairs_for_year(
    pool: &PgPool,
    source: Uuid,
    year: i32,
) -> Result<HashSet<(Uuid, Uuid)>, (StatusCode, String)> {
    let rows = sqlx::query("SELECT pa.investor_id investor_a_id,pb.investor_id investor_b_id FROM funding_rounds r JOIN round_participants pa ON pa.round_id=r.id JOIN round_participants pb ON pb.round_id=r.id AND pa.investor_id < pb.investor_id WHERE r.source_id=$1 AND EXTRACT(YEAR FROM r.announced_at)=$2 GROUP BY pa.investor_id,pb.investor_id")
        .bind(source).bind(year).fetch_all(pool).await.map_err(internal)?;
    rows.into_iter()
        .map(|row| {
            Ok((
                row.try_get("investor_a_id").map_err(internal)?,
                row.try_get("investor_b_id").map_err(internal)?,
            ))
        })
        .collect()
}
async fn metrics(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let row=sqlx::query("SELECT (SELECT COUNT(DISTINCT investor_id) FROM (SELECT investor_a_id investor_id FROM aggregate_pairs WHERE source_id=$1 UNION SELECT investor_b_id FROM aggregate_pairs WHERE source_id=$1) i) investors,(SELECT COUNT(*) FROM aggregate_pairs WHERE source_id=$1) relationships,(SELECT COUNT(DISTINCT r.id) FROM funding_rounds r WHERE r.source_id=$1) dated_rounds").bind(source).fetch_one(&s.pool).await.map_err(internal)?;
    Ok(Json(
        serde_json::json!({"investors":row.try_get::<i64,_>("investors").map_err(internal)?,"relationships":row.try_get::<i64,_>("relationships").map_err(internal)?,"dated_rounds":row.try_get::<i64,_>("dated_rounds").map_err(internal)?}),
    ))
}
async fn path(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<PathResult>, (StatusCode, String)> {
    let Some(from) = q.get("from").and_then(|x| Uuid::parse_str(x).ok()) else {
        return Err((StatusCode::BAD_REQUEST, "from must be a UUID".into()));
    };
    let Some(to) = q.get("to").and_then(|x| Uuid::parse_str(x).ok()) else {
        return Err((StatusCode::BAD_REQUEST, "to must be a UUID".into()));
    };
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let mode = q.get("mode").map(String::as_str).unwrap_or("strongest");
    if mode != "strongest" && mode != "hops" {
        return Err((
            StatusCode::BAD_REQUEST,
            "mode must be strongest or hops".into(),
        ));
    }
    let source = source_id(dataset)?;
    let (graph, ids) = load_investor_graph(&s.pool, source).await?;
    let (Some(&a), Some(&b)) = (ids.get(&from), ids.get(&to)) else {
        return Ok(Json(PathResult {
            found: false,
            path: vec![],
            hops: 0,
            mode: mode.into(),
            cost_definition: path_cost_definition(mode).into(),
            total_cost: 0.0,
            edge_weights: vec![],
        }));
    };
    let route = if mode == "hops" {
        venturegraph_graph::shortest_path(&graph, a, b)
    } else {
        venturegraph_graph::strongest_connection_path(&graph, a, b)
    };
    let Some(route) = route else {
        return Ok(Json(PathResult {
            found: false,
            path: vec![],
            hops: 0,
            mode: mode.into(),
            cost_definition: path_cost_definition(mode).into(),
            total_cost: 0.0,
            edge_weights: vec![],
        }));
    };
    let path: Vec<Uuid> = route.into_iter().map(|n| graph[n]).collect();
    let edge_weights = route_edge_weights(&graph, &path, &ids);
    let total_cost = if mode == "hops" {
        edge_weights.len() as f64
    } else {
        edge_weights
            .iter()
            .map(|weight| 1.0 / weight.max(1.0))
            .sum()
    };
    Ok(Json(PathResult {
        found: true,
        path: path.clone(),
        hops: path.len().saturating_sub(1),
        mode: mode.into(),
        cost_definition: path_cost_definition(mode).into(),
        total_cost,
        edge_weights,
    }))
}

fn path_cost_definition(mode: &str) -> &'static str {
    if mode == "hops" {
        "each relationship costs 1"
    } else {
        "each relationship costs 1 / shared_deals"
    }
}

fn route_edge_weights(
    graph: &venturegraph_graph::InvestorGraph,
    path: &[Uuid],
    ids: &HashMap<Uuid, NodeIndex>,
) -> Vec<f64> {
    path.windows(2)
        .filter_map(|pair| {
            let a = ids.get(&pair[0])?;
            let b = ids.get(&pair[1])?;
            let edge = graph.find_edge(*a, *b)?;
            graph.edge_weight(edge).copied()
        })
        .collect()
}
async fn communities(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let (graph, _) = load_investor_graph(&s.pool, source).await?;
    let groups = venturegraph_graph::connected_components(&graph);
    Ok(Json(
        serde_json::json!({"algorithm":"connected_components","interpretation":"components are not inferred communities","communities":groups}),
    ))
}

async fn centrality(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let (graph, indices) = load_investor_graph(&s.pool, source).await?;
    let pageranks = venturegraph_graph::pagerank(&graph, 0.85, 100, 1e-10);
    let (betweenness, betweenness_method) = if graph.node_count() > 5_000 {
        (
            venturegraph_graph::sampled_betweenness_centrality(&graph, 256, true),
            "deterministic 256-source Brandes approximation, normalized",
        )
    } else {
        (
            venturegraph_graph::betweenness_centrality(&graph, true),
            "exact unweighted Brandes centrality, normalized",
        )
    };
    let mut results: Vec<_> = indices
        .into_iter()
        .map(|(id, index)| {
            serde_json::json!({
                "investor_id": id,
                "degree": venturegraph_graph::degree(&graph, index),
                "weighted_degree": venturegraph_graph::weighted_degree(&graph, index),
                "pagerank": pageranks[index.index()],
                "betweenness": betweenness[index.index()],
                "clustering_coefficient": venturegraph_graph::local_clustering_coefficient(&graph, index),
            })
        })
        .collect();
    results.sort_by(|a, b| {
        b["pagerank"]
            .as_f64()
            .unwrap_or_default()
            .total_cmp(&a["pagerank"].as_f64().unwrap_or_default())
            .then_with(|| a["investor_id"].as_str().cmp(&b["investor_id"].as_str()))
    });
    Ok(Json(serde_json::json!({
        "dataset": dataset,
        "scope": "full selected dataset graph",
        "betweenness": betweenness_method,
        "pagerank": "weighted transitions, damping 0.85, max 100 iterations",
        "investors": results,
    })))
}

async fn detected_communities(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let (graph, _) = load_investor_graph(&s.pool, source).await?;
    let assignments = venturegraph_graph::label_propagation(&graph, 100);
    let mut groups: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for (investor, community) in assignments {
        groups.entry(community).or_default().push(investor);
    }
    let communities: Vec<_> = groups
        .into_iter()
        .map(|(id, mut members)| {
            members.sort();
            serde_json::json!({"id": id, "size": members.len(), "members": members})
        })
        .collect();
    Ok(Json(serde_json::json!({
        "dataset": dataset,
        "algorithm": "deterministic_weighted_label_propagation",
        "interpretation": "Heuristic graph partition based on neighboring labels and shared-deal weights; not a ground-truth classification.",
        "communities": communities,
    })))
}

async fn similarity(
    State(s): State<App>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parse = |key: &str| {
        q.get(key)
            .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("{key} must be a UUID")))
            .and_then(|value| {
                Uuid::parse_str(value)
                    .map_err(|_| (StatusCode::BAD_REQUEST, format!("{key} must be a UUID")))
            })
    };
    let a = parse("a")?;
    let b = parse("b")?;
    let dataset = q.get("dataset").map(String::as_str).unwrap_or("demo");
    let source = source_id(dataset)?;
    let (graph, indices) = load_investor_graph(&s.pool, source).await?;
    let (Some(&a_index), Some(&b_index)) = (indices.get(&a), indices.get(&b)) else {
        return Err((
            StatusCode::NOT_FOUND,
            "investor not present in dataset graph".into(),
        ));
    };
    Ok(Json(serde_json::json!({
        "dataset": dataset,
        "investor_a": a,
        "investor_b": b,
        "common_neighbors": venturegraph_graph::common_neighbors(&graph, a_index, b_index),
        "jaccard_similarity": venturegraph_graph::jaccard_similarity(&graph, a_index, b_index),
    })))
}

async fn load_investor_graph(
    pool: &PgPool,
    source: Uuid,
) -> Result<(venturegraph_graph::InvestorGraph, HashMap<Uuid, NodeIndex>), (StatusCode, String)> {
    let rows = sqlx::query(
        "SELECT investor_a_id,investor_b_id,shared_deals FROM aggregate_pairs WHERE source_id=$1 ORDER BY investor_a_id,investor_b_id",
    )
    .bind(source)
    .fetch_all(pool)
    .await
    .map_err(internal)?;
    let mut graph = venturegraph_graph::InvestorGraph::default();
    let mut indices = HashMap::new();
    for row in rows {
        let a: Uuid = row.try_get("investor_a_id").map_err(internal)?;
        let b: Uuid = row.try_get("investor_b_id").map_err(internal)?;
        let weight: i32 = row.try_get("shared_deals").map_err(internal)?;
        let a_index = *indices.entry(a).or_insert_with(|| graph.add_node(a));
        let b_index = *indices.entry(b).or_insert_with(|| graph.add_node(b));
        graph.add_edge(a_index, b_index, f64::from(weight));
    }
    Ok((graph, indices))
}
fn source_id(dataset: &str) -> Result<Uuid, (StatusCode, String)> {
    match dataset {
        "demo" => Ok(Uuid::parse_str("00000000-0000-4000-8000-000000000001")
            .expect("constant UUID is valid")),
        "fliar" => Ok(Uuid::parse_str("30000000-0000-4000-8000-000000000001")
            .expect("constant UUID is valid")),
        _ => Err((
            StatusCode::BAD_REQUEST,
            "dataset must be demo or fliar".into(),
        )),
    }
}
fn cors_layer() -> CorsLayer {
    let configured = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:15173,http://localhost:5173".into());
    let mut origins = Vec::new();
    for origin in configured
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        match HeaderValue::try_from(origin) {
            Ok(value) => origins.push(value),
            Err(error) => tracing::warn!(%origin, %error, "ignoring invalid CORS origin"),
        }
    }
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET])
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!(error=%e,"request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "request failed".into())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(health))
        .route("/docs", get(api_docs))
        .route("/api/v1/openapi.json", get(openapi_schema))
        .route("/api/v1/meta", get(meta))
        .route("/api/v1/data-sources", get(data_sources))
        .route("/api/v1/data-coverage", get(data_coverage))
        .route("/api/v1/graph", get(graph))
        .route("/api/v1/temporal", get(temporal))
        .route("/api/v1/temporal/diff", get(temporal_diff))
        .route("/api/v1/investors", get(investors))
        .route("/api/v1/investors/{id}", get(investor_detail))
        .route("/api/v1/investors/{id}/partners", get(investor_partners))
        .route("/api/v1/metrics", get(metrics))
        .route("/api/v1/path", get(path))
        .route("/api/v1/communities", get(communities))
        .route("/api/v1/analytics/centrality", get(centrality))
        .route(
            "/api/v1/analytics/co-investments",
            get(coinvestment_discovery),
        )
        .route("/api/v1/analytics/emerging", get(emerging_investors))
        .route("/api/v1/graph/similarity", get(similarity))
        .route("/api/v1/graph/communities", get(detected_communities))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(App { pool });
    let addr: SocketAddr = std::env::var("API_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr,"VentureGraph API listening");
    axum::serve(listener, app).await?;
    Ok(())
}
