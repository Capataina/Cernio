use std::sync::Arc;

use rusqlite::{params, Connection};
use tokio::sync::Semaphore;

use crate::ats::common::AtsJob;
use crate::ats::{ashby, greenhouse, lever, smartrecruiters, workable, workday};
use crate::config::SearchFilters;

/// A resolved company with its portal info, ready for job fetching.
#[derive(Clone)]
struct SearchTarget {
    company_id: i64,
    company_name: String,
    portal_id: i64,
    provider: String,
    slug: String,
    ats_extra: Option<String>,
}

/// Result of fetching jobs for one company.
struct FetchResult {
    target: SearchTarget,
    jobs: Vec<AtsJob>,
}

/// Run the search pipeline for all resolved companies above the grade threshold.
/// Fetches from all companies in parallel, then filters and inserts sequentially.
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

    // Fetch all companies in parallel.
    let fetch_results = fetch_all_parallel(&targets).await;

    // Filter and insert sequentially (needs DB access).
    let mut total_fetched = 0u64;
    let mut total_after_location = 0u64;
    let mut total_after_exclusion = 0u64;
    let mut total_after_inclusion = 0u64;
    let mut total_new = 0u64;

    for result in &fetch_results {
        let fetched = result.jobs.len();
        total_fetched += fetched as u64;

        let after_location: Vec<_> = result
            .jobs
            .iter()
            .filter(|j| filters.passes_location(&result.target.provider, &j.all_locations))
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

        let new_jobs: Vec<_> = after_inclusion
            .into_iter()
            .filter(|j| !job_exists(conn, &j.url))
            .collect();
        let new_count = new_jobs.len();
        total_new += new_count as u64;

        let status = if new_count > 0 { "✓" } else { "·" };
        println!(
            "  {status} {:<30} {fetched:>4} fetched → {new_count:>3} new",
            result.target.company_name,
        );

        if !dry_run {
            for job in &new_jobs {
                insert_job(
                    conn,
                    result.target.company_id,
                    result.target.portal_id,
                    job,
                );
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
        .filter(|t| {
            t.company_name
                .to_lowercase()
                .contains(&company_name.to_lowercase())
        })
        .collect();

    if targets.is_empty() {
        println!("No resolved company matching '{company_name}' found above grade threshold.");
        return;
    }

    let client = crate::http::build_client();

    for target in &targets {
        println!("Searching {}...", target.company_name);
        let jobs = fetch_jobs(&client, &target.provider, &target.slug, target.ats_extra.as_deref()).await;
        println!("  Fetched {} jobs", jobs.len());

        let filtered: Vec<_> = jobs
            .iter()
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
            for job in filtered.iter().take(10) {
                println!("    - {}", job.title);
            }
            if filtered.len() > 10 {
                println!("    ... and {} more", filtered.len() - 10);
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

    let fetch_results = fetch_all_parallel(&targets).await;
    let mut total_new = 0u64;

    for result in &fetch_results {
        let filtered: Vec<_> = result
            .jobs
            .iter()
            .filter(|j| filters.passes_location(&result.target.provider, &j.all_locations))
            .filter(|j| filters.passes_exclusion(&j.title))
            .filter(|j| filters.passes_inclusion(&j.title))
            .filter(|j| !job_exists(conn, &j.url))
            .collect();

        let count = filtered.len();
        total_new += count as u64;
        let status = if count > 0 { "✓" } else { "·" };
        println!("  {status} {:<30} {count:>3} new", result.target.company_name);

        if !dry_run {
            for job in &filtered {
                insert_job(
                    conn,
                    result.target.company_id,
                    result.target.portal_id,
                    job,
                );
            }
        }
    }

    println!("\n  Total new: {total_new}");
    if dry_run {
        println!("  (dry run — nothing written)");
    }
}

// ── Parallel fetching ────────────────────────────────────────────

/// Fetch jobs from all targets in parallel, with a concurrency limit.
async fn fetch_all_parallel(targets: &[SearchTarget]) -> Vec<FetchResult> {
    let client = crate::http::build_client();
    let semaphore = Arc::new(Semaphore::new(8)); // Max 8 concurrent ATS fetches.
    let mut handles = Vec::new();

    for target in targets {
        let client = client.clone();
        let sem = semaphore.clone();
        let target = target.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            // Try up to 3 times — transient timeouts in parallel batches cause silent 0-job results.
            let mut jobs = fetch_jobs(&client, &target.provider, &target.slug, target.ats_extra.as_deref()).await;
            if jobs.is_empty() {
                // Retry after a brief delay.
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                jobs = fetch_jobs(&client, &target.provider, &target.slug, target.ats_extra.as_deref()).await;
            }
            if jobs.is_empty() {
                // Third attempt.
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                jobs = fetch_jobs(&client, &target.provider, &target.slug, target.ats_extra.as_deref()).await;
            }
            FetchResult { target, jobs }
        });

        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    // Sort by company name for clean output.
    results.sort_by(|a, b| a.target.company_name.cmp(&b.target.company_name));
    results
}

// ── Helpers ──────────────────────────────────────────────────────

fn get_search_targets(conn: &Connection, filters: &SearchFilters) -> Vec<SearchTarget> {
    let grades = filters.included_grades();
    let placeholders: Vec<String> = grades
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let sql = format!(
        "SELECT c.id, c.name, p.id, p.ats_provider, p.ats_slug, p.ats_extra
         FROM companies c
         JOIN company_portals p ON p.company_id = c.id
         WHERE c.status = 'resolved' AND c.grade IN ({})
         ORDER BY CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END, c.name, p.is_primary DESC",
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
            ats_extra: row.get(5)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

fn get_all_search_targets(conn: &Connection) -> Vec<SearchTarget> {
    conn.prepare(
        "SELECT c.id, c.name, p.id, p.ats_provider, p.ats_slug, p.ats_extra
         FROM companies c
         JOIN company_portals p ON p.company_id = c.id
         WHERE c.status = 'resolved'
         ORDER BY c.name, p.is_primary DESC",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(SearchTarget {
                company_id: row.get(0)?,
                company_name: row.get(1)?,
                portal_id: row.get(2)?,
                provider: row.get(3)?,
                slug: row.get(4)?,
                ats_extra: row.get(5)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}

async fn fetch_jobs(
    client: &reqwest::Client,
    provider: &str,
    slug: &str,
    ats_extra: Option<&str>,
) -> Vec<AtsJob> {
    let result: Result<Vec<AtsJob>, Box<dyn std::error::Error + Send + Sync>> = match provider {
        "greenhouse" => greenhouse::fetch_all_with_extra(client, slug, ats_extra)
            .await
            .map_err(|e| e.into()),
        "lever" => lever::fetch_all_with_extra(client, slug, ats_extra)
            .await
            .map(|postings| lever::normalise_postings(postings))
            .map_err(|e| e.into()),
        "ashby" => ashby::fetch_all(client, slug)
            .await
            .map_err(|e| e.into()),
        "workable" => workable::fetch_all(client, slug)
            .await
            .map_err(|e| e.into()),
        "smartrecruiters" => smartrecruiters::fetch_all(client, slug)
            .await
            .map_err(|e| e.into()),
        "workday" => {
            if let Some(extra) = ats_extra {
                workday::fetch_all_with_extra(client, slug, extra).await
            } else {
                Ok(Vec::new())
            }
        }
        _ => {
            eprintln!("    Unknown provider: {provider}");
            Ok(Vec::new())
        }
    };

    result.unwrap_or_else(|e| {
        eprintln!("    Error fetching from {provider}/{slug}: {e}");
        Vec::new()
    })
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
