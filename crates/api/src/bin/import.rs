use serde::Deserialize;
use sqlx::{
    QueryBuilder, Row,
    postgres::{PgPoolOptions, Postgres},
};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    error::Error,
    path::PathBuf,
};
use uuid::Uuid;

#[derive(Deserialize)]
struct Pair {
    pair_id: String,
    investor_a: String,
    investor_a_slug: String,
    investor_a_type: String,
    investor_a_domain: String,
    investor_b: String,
    investor_b_type: String,
    investor_b_domain: String,
    shared_deals: i32,
    first_year: i32,
    last_year: i32,
    top_shared_sector: String,
    top_shared_country: String,
    fliar_profile_url: String,
}
fn stable(name: &str) -> Uuid {
    let digest = format!(
        "{:x}",
        md5::compute(format!("venturegraph:fliar:{}", name.trim()))
    );
    Uuid::parse_str(&format!(
        "{}-{}-{}-{}-{}",
        &digest[0..8],
        &digest[8..12],
        &digest[12..16],
        &digest[16..20],
        &digest[20..32]
    ))
    .expect("md5 is a UUID shape")
}
fn slug(s: &str) -> String {
    let base = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!(
        "{}-{}",
        if base.is_empty() { "investor" } else { &base },
        &format!("{:x}", md5::compute(s.trim()))[..8]
    )
}
#[derive(Clone)]
struct InvestorRecord {
    id: Uuid,
    name: String,
    slug: String,
    kind: String,
    domain: String,
}

fn investor_record(
    name: &str,
    preferred_slug: Option<&str>,
    kind: &str,
    domain: &str,
) -> InvestorRecord {
    let normalized_type = match kind
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "vc" | "venture_capital_firm" | "venture_capital" => "venture_capital".to_owned(),
        "cvc" | "corporate_venture_capital" | "corporate_venture" => "corporate_venture".to_owned(),
        "angel_investor" | "angel" => "angel".to_owned(),
        "" => "unknown".to_owned(),
        other => other.to_owned(),
    };
    InvestorRecord {
        id: stable(name),
        name: name.trim().to_owned(),
        slug: preferred_slug
            .filter(|value| !value.is_empty())
            .map(slug)
            .unwrap_or_else(|| slug(name)),
        kind: normalized_type,
        domain: domain.trim().to_owned(),
    }
}

async fn flush_batch(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    import_id: Uuid,
    source_id: Uuid,
    rows: Vec<(i64, Pair)>,
    mut errors: Vec<(i64, String)>,
) -> Result<(i64, i64), Box<dyn Error>> {
    let mut investors = HashMap::<Uuid, InvestorRecord>::new();
    let mut external_pairs = BTreeMap::<String, (Uuid, Uuid, Pair)>::new();
    let mut accepted = 0_i64;
    for (row_number, pair) in rows {
        let valid = !pair.pair_id.trim().is_empty()
            && !pair.investor_a.trim().is_empty()
            && !pair.investor_b.trim().is_empty()
            && pair.shared_deals >= 0
            && (1..=9999).contains(&pair.first_year)
            && pair.first_year <= pair.last_year
            && (1..=9999).contains(&pair.last_year);
        if !valid {
            errors.push((row_number, "required values invalid".into()));
            continue;
        }
        let a_record = investor_record(
            &pair.investor_a,
            Some(&pair.investor_a_slug),
            &pair.investor_a_type,
            &pair.investor_a_domain,
        );
        let b_record = investor_record(
            &pair.investor_b,
            None,
            &pair.investor_b_type,
            &pair.investor_b_domain,
        );
        if a_record.id == b_record.id {
            errors.push((row_number, "self-pair rejected".into()));
            continue;
        }
        let (a, b) = if a_record.id < b_record.id {
            (a_record.id, b_record.id)
        } else {
            (b_record.id, a_record.id)
        };
        investors.insert(a_record.id, a_record);
        investors.insert(b_record.id, b_record);
        external_pairs.insert(pair.pair_id.clone(), (a, b, pair));
        accepted += 1;
    }
    let mut pairs = BTreeMap::new();
    for (_, (a, b, pair)) in external_pairs {
        pairs.insert((a, b), pair);
    }
    if !investors.is_empty() {
        let mut query = QueryBuilder::<Postgres>::new(
            "INSERT INTO investors(id,canonical_name,slug,investor_type,website_domain) ",
        );
        query.push_values(investors.values(), |mut row, item| {
            row.push_bind(item.id)
                .push_bind(&item.name)
                .push_bind(&item.slug)
                .push_bind(&item.kind)
                .push_bind(if item.domain.is_empty() {
                    None
                } else {
                    Some(item.domain.as_str())
                });
        });
        query.push(" ON CONFLICT(id) DO UPDATE SET investor_type=EXCLUDED.investor_type,website_domain=COALESCE(EXCLUDED.website_domain,investors.website_domain),updated_at=now()");
        query.build().execute(&mut **tx).await?;
    }
    if !pairs.is_empty() {
        let delete_a: Vec<_> = pairs.keys().map(|(a, _)| *a).collect();
        let delete_b: Vec<_> = pairs.keys().map(|(_, b)| *b).collect();
        let delete_sources = vec![source_id; pairs.len()];
        let delete_external_ids: Vec<_> = pairs.values().map(|pair| pair.pair_id.clone()).collect();
        sqlx::query("DELETE FROM aggregate_pairs old USING unnest($1::uuid[],$2::uuid[],$3::uuid[],$4::text[]) AS incoming(investor_a_id,investor_b_id,source_id,external_pair_id) WHERE old.source_id=incoming.source_id AND old.external_pair_id=incoming.external_pair_id AND (old.investor_a_id,old.investor_b_id) IS DISTINCT FROM (incoming.investor_a_id,incoming.investor_b_id)")
            .bind(delete_a).bind(delete_b).bind(delete_sources).bind(delete_external_ids)
            .execute(&mut **tx).await?;
        let mut query = QueryBuilder::<Postgres>::new(
            "INSERT INTO aggregate_pairs(investor_a_id,investor_b_id,shared_deals,first_year,last_year,top_shared_sector,top_shared_country,profile_url,source_id,external_pair_id) ",
        );
        query.push_values(pairs.values(), |mut row, pair| {
            let a = stable(&pair.investor_a);
            let b = stable(&pair.investor_b);
            let (a, b) = if a < b { (a, b) } else { (b, a) };
            row.push_bind(a)
                .push_bind(b)
                .push_bind(pair.shared_deals)
                .push_bind(pair.first_year)
                .push_bind(pair.last_year)
                .push_bind(if pair.top_shared_sector.is_empty() {
                    None
                } else {
                    Some(pair.top_shared_sector.as_str())
                })
                .push_bind(if pair.top_shared_country.is_empty() {
                    None
                } else {
                    Some(pair.top_shared_country.as_str())
                })
                .push_bind(if pair.fliar_profile_url.is_empty() {
                    None
                } else {
                    Some(pair.fliar_profile_url.as_str())
                })
                .push_bind(source_id)
                .push_bind(&pair.pair_id);
        });
        query.push(" ON CONFLICT (investor_a_id,investor_b_id,source_id) DO UPDATE SET shared_deals=EXCLUDED.shared_deals,first_year=EXCLUDED.first_year,last_year=EXCLUDED.last_year,top_shared_sector=EXCLUDED.top_shared_sector,top_shared_country=EXCLUDED.top_shared_country,profile_url=EXCLUDED.profile_url,external_pair_id=EXCLUDED.external_pair_id");
        query.build().execute(&mut **tx).await?;
    }
    if !errors.is_empty() {
        let mut query = QueryBuilder::<Postgres>::new(
            "INSERT INTO import_errors(import_id,row_number,message) ",
        );
        query.push_values(errors.iter(), |mut row, (row_number, message)| {
            row.push_bind(import_id)
                .push_bind(*row_number)
                .push_bind(message);
        });
        query.build().execute(&mut **tx).await?;
    }
    Ok((accepted, errors.len() as i64))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let argument = env::args_os()
        .nth(1)
        .ok_or("usage: venturegraph-import <sample.csv> (use --help for details)")?;
    if argument == "--help" || argument == "-h" {
        println!(
            "VentureGraph Fliar aggregate CSV importer\n\nUsage:\n  venturegraph-import <sample.csv>\n\nReads the published Fliar pair sample incrementally in batches of 500 and stores it in DATABASE_URL. Obtain the CSV from https://fliarbi.com/datasets/investor-coinvestment-network/ . The source is CC BY-NC 4.0."
        );
        return Ok(());
    }
    let path = PathBuf::from(argument);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    let source = Uuid::parse_str("30000000-0000-4000-8000-000000000001")?;
    sqlx::query("INSERT INTO data_sources(id,name,source_url,license,methodology) VALUES($1,'Fliar VC Co-Investment Network sample','https://fliarbi.com/datasets/investor-coinvestment-network/','CC BY-NC 4.0','25% fixed pair sample; edge observations aggregate shared funding rounds from Jan 2019 through Jun 2026.') ON CONFLICT(id) DO NOTHING").bind(source).execute(&pool).await?;
    let source_version = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("local-csv");
    let import:Uuid=sqlx::query("INSERT INTO data_imports(source_id,dataset_name,source_version) VALUES($1,'Fliar sample',$2) RETURNING id").bind(source).bind(source_version).fetch_one(&pool).await?.try_get("id")?;
    let mut tx = pool.begin().await?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;
    let mut seen = 0_i64;
    let mut imported = 0_i64;
    let mut rejected = 0_i64;
    let mut batch_rows = Vec::with_capacity(500);
    let mut batch_errors = Vec::with_capacity(500);
    for (offset, result) in reader.deserialize::<Pair>().enumerate() {
        seen += 1;
        let row_no = (offset + 2) as i64;
        match result {
            Ok(pair) => batch_rows.push((row_no, pair)),
            Err(error) => batch_errors.push((row_no, error.to_string())),
        }
        if batch_rows.len() + batch_errors.len() >= 500 {
            let (batch_imported, batch_rejected) = flush_batch(
                &mut tx,
                import,
                source,
                std::mem::take(&mut batch_rows),
                std::mem::take(&mut batch_errors),
            )
            .await?;
            imported += batch_imported;
            rejected += batch_rejected;
        }
        if seen % 1000 == 0 {
            eprintln!("processed {seen} rows ({imported} imported, {rejected} rejected)");
        }
    }
    let (batch_imported, batch_rejected) =
        flush_batch(&mut tx, import, source, batch_rows, batch_errors).await?;
    imported += batch_imported;
    rejected += batch_rejected;
    tx.commit().await?;
    sqlx::query("UPDATE data_imports SET finished_at=now(),rows_seen=$2,rows_imported=$3,rows_rejected=$4,status='completed' WHERE id=$1").bind(import).bind(seen).bind(imported).bind(rejected).execute(&pool).await?;
    println!("Import {import}: {seen} rows seen, {imported} imported, {rejected} rejected");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fliar_sample_header_deserializes_incrementally() {
        let csv = "pair_id,investor_a,investor_a_slug,investor_a_type,investor_a_domain,investor_b,investor_b_type,investor_b_domain,shared_deals,first_year,last_year,top_shared_sector,top_shared_country,fliar_profile_url\npair-123,Alpha Fund,alpha-fund,VC,alpha.example,Beta Fund,CVC,beta.example,3,2021,2024,Fintech,United States,https://example.test/profile\n";
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let pair: Pair = reader.deserialize().next().unwrap().unwrap();
        assert_eq!(pair.pair_id, "pair-123");
        assert_eq!(pair.investor_a, "Alpha Fund");
        assert_eq!(pair.shared_deals, 3);
        assert_eq!(pair.last_year, 2024);
    }

    #[test]
    fn investor_identifiers_are_stable_for_exact_source_names() {
        assert_eq!(stable("Alpha Fund"), stable("Alpha Fund"));
        assert_ne!(stable("Alpha Fund"), stable("Alpha fund"));
    }

    #[test]
    fn investor_types_are_normalized_without_merging_names() {
        assert_eq!(
            investor_record("Alpha Fund", None, "VC", "").kind,
            "venture_capital"
        );
        assert_eq!(
            investor_record("Alpha Fund", None, "CVC", "").kind,
            "corporate_venture"
        );
        assert_ne!(stable("Alpha Fund"), stable("Alpha fund"));
    }
}
