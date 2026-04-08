use rusqlite::{params, Connection};

use crate::ats::{ashby, greenhouse, smartrecruiters, workable};
use crate::ats::common::SlugProbeResult;
use crate::ats::lever;

/// Generate slug candidates from a company name.
/// Tries common transformations that ATS providers use.
fn slug_candidates(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();
    let mut candidates = Vec::new();

    // Exact lowercase with spaces removed.
    candidates.push(lower.replace(' ', ""));
    // Hyphenated.
    candidates.push(lower.replace(' ', "-"));
    // Underscored.
    candidates.push(lower.replace(' ', "_"));

    // Strip common suffixes and try again.
    for suffix in &[
        " ltd", " limited", " inc", " corp", " plc", " group",
        " technologies", " technology", " tech", " labs", " ai",
        " hq", " io",
    ] {
        if let Some(stripped) = lower.strip_suffix(suffix) {
            candidates.push(stripped.replace(' ', ""));
            candidates.push(stripped.replace(' ', "-"));
        }
    }

    // Also try WITH "technologies" appended (XTX → xtxmarketstechnologies).
    let no_spaces = lower.replace(' ', "");
    candidates.push(format!("{no_spaces}technologies"));

    // Try just the first word (e.g. "Helsing GmbH" → "helsing").
    if let Some(first) = lower.split_whitespace().next() {
        candidates.push(first.to_string());
    }

    // Deduplicate while preserving order.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()));

    candidates
}

/// Probe Lever for a slug (Lever has no dedicated probe endpoint).
async fn probe_lever(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    let postings = lever::fetch_all(client, slug).await.ok()?;
    if postings.is_empty() {
        return None;
    }
    Some(SlugProbeResult {
        provider: "lever",
        slug: slug.to_string(),
        job_count: postings.len(),
    })
}

/// Probe all ATS providers for a company name. Returns all hits.
async fn probe_all_providers(
    client: &reqwest::Client,
    name: &str,
) -> Vec<SlugProbeResult> {
    let candidates = slug_candidates(name);
    let mut results = Vec::new();

    // Track which providers we've already found to avoid duplicate slug probing.
    let mut found_providers = std::collections::HashSet::new();

    for slug in &candidates {
        // Probe all providers in parallel for each slug.
        let (gh, lv, ab, wk, sr) = tokio::join!(
            greenhouse::probe(client, slug),
            probe_lever(client, slug),
            ashby::probe(client, slug),
            workable::probe(client, slug),
            smartrecruiters::probe(client, slug),
        );

        for result in [gh, lv, ab, wk, sr].into_iter().flatten() {
            if found_providers.insert(result.provider.to_string()) {
                results.push(result);
            }
        }
    }

    results
}

/// Run the resolve pipeline for all pending companies.
pub async fn run(conn: &Connection, dry_run: bool) {
    let companies = get_pending_companies(conn);

    if companies.is_empty() {
        println!("No pending companies to resolve.");
        return;
    }

    println!("Resolving {} companies...\n", companies.len());

    let client = reqwest::Client::new();

    for (id, name, website) in &companies {
        print!("  {name:<30} ");

        let results = probe_all_providers(&client, name).await;

        if results.is_empty() {
            println!("  ✗ no ATS found — needs AI resolution");
            continue;
        }

        // Find the portal with the most jobs to mark as primary.
        let max_jobs = results.iter().map(|r| r.job_count).max().unwrap_or(0);

        for result in &results {
            let is_primary = result.job_count == max_jobs;
            println!(
                "  {} {} / {} ({} jobs{})",
                if is_primary { "✓" } else { "·" },
                result.provider,
                result.slug,
                result.job_count,
                if is_primary { ", primary" } else { "" }
            );

            if !dry_run {
                insert_portal(conn, *id, result, is_primary);
            }
        }

        if !dry_run {
            // Mark company as resolved.
            let _ = conn.execute(
                "UPDATE companies SET status = 'resolved' WHERE id = ?1",
                params![id],
            );
        }

        println!();
    }
}

/// Run resolve for a single company by name.
pub async fn run_single(conn: &Connection, company_name: &str, dry_run: bool) {
    let companies = get_pending_companies(conn);
    let matching: Vec<_> = companies
        .iter()
        .filter(|(_, name, _)| name.to_lowercase().contains(&company_name.to_lowercase()))
        .collect();

    if matching.is_empty() {
        println!("No pending company matching '{company_name}' found.");
        return;
    }

    let client = reqwest::Client::new();

    for (id, name, _website) in matching {
        print!("{name}: ");
        let results = probe_all_providers(&client, name).await;

        if results.is_empty() {
            println!("no ATS found — needs AI resolution");
            continue;
        }

        let max_jobs = results.iter().map(|r| r.job_count).max().unwrap_or(0);
        for result in &results {
            let is_primary = result.job_count == max_jobs;
            println!(
                "  {} {} / {} ({} jobs)",
                if is_primary { "✓" } else { "·" },
                result.provider,
                result.slug,
                result.job_count,
            );

            if !dry_run {
                insert_portal(conn, *id, result, is_primary);
            }
        }

        if !dry_run {
            let _ = conn.execute(
                "UPDATE companies SET status = 'resolved' WHERE id = ?1",
                params![id],
            );
        }
    }
}

fn get_pending_companies(conn: &Connection) -> Vec<(i64, String, String)> {
    conn.prepare(
        "SELECT id, name, website FROM companies WHERE status = 'potential' ORDER BY name",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}

fn insert_portal(conn: &Connection, company_id: i64, result: &SlugProbeResult, is_primary: bool) {
    let _ = conn.execute(
        "INSERT OR IGNORE INTO company_portals (company_id, ats_provider, ats_slug, is_primary, verified_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![
            company_id,
            result.provider,
            result.slug,
            is_primary as i32
        ],
    );
}
