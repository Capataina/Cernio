use rusqlite::{params, Connection};

use crate::ats::common::AtsJob;
use crate::ats::{ashby, greenhouse, lever, smartrecruiters, workable};
use crate::config::SearchFilters;

/// A resolved company with its portal info, ready for job fetching.
struct SearchTarget {
    company_id: i64,
    company_name: String,
    portal_id: i64,
    provider: String,
    slug: String,
}

/// Run the search pipeline for all resolved companies above the grade threshold.
pub async fn run(conn: &Connection, filters: &SearchFilters, dry_run: bool) {
    let targets = get_search_targets(conn, filters);

    if targets.is_empty() {
        println!("No resolved companies matching grade threshold.");
        return;
    }

    println!(
        "Searching {} companies (grade >= {})...\n",
        targets.len(),
        filters.min_company_grade
    );

    let client = reqwest::Client::new();
    let mut total_fetched = 0u64;
    let mut total_after_location = 0u64;
    let mut total_after_exclusion = 0u64;
    let mut total_after_inclusion = 0u64;
    let mut total_new = 0u64;

    for target in &targets {
        let jobs = fetch_jobs(&client, &target.provider, &target.slug).await;
        let fetched = jobs.len();
        total_fetched += fetched as u64;

        // Filter chain.
        let after_location: Vec<_> = jobs
            .into_iter()
            .filter(|j| filters.passes_location(&target.provider, &j.all_locations))
            .collect();
        total_after_location += after_location.len() as u64;

        let after_exclusion: Vec<_> = after_location
            .into_iter()
            .filter(|j| filters.passes_exclusion(&j.title))
            .collect();
        total_after_exclusion += after_exclusion.len() as u64;

        let after_inclusion: Vec<_> = after_exclusion
            .into_iter()
            .filter(|j| filters.passes_inclusion(&j.title))
            .collect();
        total_after_inclusion += after_inclusion.len() as u64;

        // Dedup against existing DB entries.
        let new_jobs: Vec<_> = after_inclusion
            .into_iter()
            .filter(|j| !job_exists(conn, &j.url))
            .collect();
        let new_count = new_jobs.len();
        total_new += new_count as u64;

        let status = if new_count > 0 { "✓" } else { "·" };
        println!(
            "  {status} {:<25} {fetched:>4} fetched → {new_count:>3} new",
            target.company_name,
        );

        if !dry_run {
            for job in &new_jobs {
                insert_job(conn, target.company_id, target.portal_id, job);
            }
        }
    }

    println!();
    println!("── Summary ──");
    println!("  Fetched:         {total_fetched:>6}");
    println!("  After location:  {total_after_location:>6}");
    println!("  After exclusion: {total_after_exclusion:>6}");
    println!("  After inclusion: {total_after_inclusion:>6}");
    println!("  New (inserted):  {total_new:>6}");

    if dry_run {
        println!("\n  (dry run — nothing was written to the database)");
    }
}

/// Run search for a single company.
pub async fn run_single(
    conn: &Connection,
    filters: &SearchFilters,
    company_name: &str,
    dry_run: bool,
) {
    let all_targets = get_search_targets(conn, filters);
    let targets: Vec<_> = all_targets
        .into_iter()
        .filter(|t| t.company_name.to_lowercase().contains(&company_name.to_lowercase()))
        .collect();

    if targets.is_empty() {
        println!("No resolved company matching '{company_name}' found above grade threshold.");
        return;
    }

    let client = reqwest::Client::new();

    for target in &targets {
        println!("Searching {}...", target.company_name);
        let jobs = fetch_jobs(&client, &target.provider, &target.slug).await;
        println!("  Fetched {} jobs", jobs.len());

        let filtered: Vec<_> = jobs
            .into_iter()
            .filter(|j| filters.passes_location(&target.provider, &j.all_locations))
            .filter(|j| filters.passes_exclusion(&j.title))
            .filter(|j| filters.passes_inclusion(&j.title))
            .filter(|j| !job_exists(conn, &j.url))
            .collect();

        println!("  {} new jobs after filtering", filtered.len());

        if !dry_run {
            for job in &filtered {
                insert_job(conn, target.company_id, target.portal_id, job);
            }
            println!("  Inserted into database.");
        } else {
            println!("  (dry run — not inserted)");
            for job in &filtered {
                println!("    - {}", job.title);
            }
        }
    }
}

/// Run search filtered by company grade.
pub async fn run_by_grade(
    conn: &Connection,
    filters: &SearchFilters,
    grade: &str,
    dry_run: bool,
) {
    let all_targets = get_all_search_targets(conn);
    let targets: Vec<_> = all_targets
        .into_iter()
        .filter(|t| {
            let company_grade: Option<String> = conn
                .query_row(
                    "SELECT grade FROM companies WHERE id = ?1",
                    params![t.company_id],
                    |row| row.get(0),
                )
                .ok()
                .flatten();
            company_grade.as_deref() == Some(grade)
        })
        .collect();

    if targets.is_empty() {
        println!("No resolved companies with grade {grade}.");
        return;
    }

    println!("Searching {} {grade}-tier companies...\n", targets.len());

    let client = reqwest::Client::new();
    let mut total_new = 0u64;

    for target in &targets {
        let jobs = fetch_jobs(&client, &target.provider, &target.slug).await;
        let filtered: Vec<_> = jobs
            .into_iter()
            .filter(|j| filters.passes_location(&target.provider, &j.all_locations))
            .filter(|j| filters.passes_exclusion(&j.title))
            .filter(|j| filters.passes_inclusion(&j.title))
            .filter(|j| !job_exists(conn, &j.url))
            .collect();

        let count = filtered.len();
        total_new += count as u64;
        let status = if count > 0 { "✓" } else { "·" };
        println!("  {status} {:<25} {count:>3} new", target.company_name);

        if !dry_run {
            for job in &filtered {
                insert_job(conn, target.company_id, target.portal_id, job);
            }
        }
    }

    println!("\n  Total new: {total_new}");
    if dry_run {
        println!("  (dry run — nothing written)");
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn get_search_targets(conn: &Connection, filters: &SearchFilters) -> Vec<SearchTarget> {
    let grades = filters.included_grades();
    let placeholders: Vec<String> = grades.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "SELECT c.id, c.name, p.id, p.ats_provider, p.ats_slug
         FROM companies c
         JOIN company_portals p ON p.company_id = c.id AND p.is_primary = 1
         WHERE c.status = 'resolved' AND c.grade IN ({})
         ORDER BY CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END, c.name",
        placeholders.join(", ")
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = grades
        .into_iter()
        .map(|g| Box::new(g.to_string()) as Box<dyn rusqlite::types::ToSql>)
        .collect();

    stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(SearchTarget {
            company_id: row.get(0)?,
            company_name: row.get(1)?,
            portal_id: row.get(2)?,
            provider: row.get(3)?,
            slug: row.get(4)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

fn get_all_search_targets(conn: &Connection) -> Vec<SearchTarget> {
    conn.prepare(
        "SELECT c.id, c.name, p.id, p.ats_provider, p.ats_slug
         FROM companies c
         JOIN company_portals p ON p.company_id = c.id AND p.is_primary = 1
         WHERE c.status = 'resolved'
         ORDER BY c.name",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(SearchTarget {
                company_id: row.get(0)?,
                company_name: row.get(1)?,
                portal_id: row.get(2)?,
                provider: row.get(3)?,
                slug: row.get(4)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}

async fn fetch_jobs(client: &reqwest::Client, provider: &str, slug: &str) -> Vec<AtsJob> {
    let result = match provider {
        "greenhouse" => greenhouse::fetch_all(client, slug).await,
        "lever" => fetch_lever_normalised(client, slug).await,
        "ashby" => ashby::fetch_all(client, slug).await,
        "workable" => workable::fetch_all(client, slug).await,
        "smartrecruiters" => smartrecruiters::fetch_all(client, slug).await,
        _ => {
            eprintln!("    Unknown provider: {provider}");
            return Vec::new();
        }
    };

    result.unwrap_or_else(|e| {
        eprintln!("    Error fetching from {provider}/{slug}: {e}");
        Vec::new()
    })
}

/// Adapter: convert Lever's native types to AtsJob.
async fn fetch_lever_normalised(
    client: &reqwest::Client,
    slug: &str,
) -> Result<Vec<AtsJob>, reqwest::Error> {
    let postings = lever::fetch_all(client, slug).await?;
    Ok(postings
        .into_iter()
        .map(|p| {
            let mut all_locations = Vec::new();
            if let Some(loc) = &p.categories.location {
                all_locations.push(loc.clone());
            }

            AtsJob {
                external_id: p.id,
                title: p.text,
                url: p.hosted_url.unwrap_or_default(),
                location: p.categories.location,
                all_locations,
                remote_policy: p.workplace_type,
                posted_date: p
                    .created_at
                    .map(|ts| {
                        chrono::DateTime::from_timestamp_millis(ts as i64)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default()
                    }),
                description: None,
            }
        })
        .collect())
}

fn job_exists(conn: &Connection, url: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM jobs WHERE url = ?1",
        params![url],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
    .unwrap_or(false)
}

fn insert_job(conn: &Connection, company_id: i64, portal_id: i64, job: &AtsJob) {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let _ = conn.execute(
        "INSERT OR IGNORE INTO jobs (company_id, portal_id, title, url, location, remote_policy, posted_date, raw_description, evaluation_status, discovered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9)",
        params![
            company_id,
            portal_id,
            job.title,
            job.url,
            job.location,
            job.remote_policy,
            job.posted_date,
            job.description,
            now,
        ],
    );
}
