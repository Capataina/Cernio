use std::sync::Arc;

use rusqlite::{params, Connection};
use tokio::sync::Semaphore;

use crate::ats::common::SlugProbeResult;
use crate::ats::{ashby, greenhouse, lever, smartrecruiters, workable};

// Note: Lever now has its own probe() function that tries both US and EU endpoints.

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

    // Strip parenthetical suffixes: "Man Group (AHL)" → "mangroup"
    if let Some(paren_pos) = lower.find('(') {
        let without_paren = lower[..paren_pos].trim();
        candidates.push(without_paren.replace(' ', ""));
        candidates.push(without_paren.replace(' ', "-"));
    }

    // Strip slashes: "Refinitiv / LSEG" → "refinitiv"
    if let Some(slash_pos) = lower.find('/') {
        let first_part = lower[..slash_pos].trim();
        candidates.push(first_part.replace(' ', ""));
        candidates.push(first_part.replace(' ', "-"));
    }

    // Strip ".co" suffix: "Copper.co" → "copper"
    if let Some(stripped) = lower.strip_suffix(".co") {
        candidates.push(stripped.replace(' ', ""));
    }

    // Deduplicate while preserving order.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()));

    candidates
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
        // Probe fast providers in parallel first (Greenhouse, Lever, Ashby, Workable).
        // SmartRecruiters is probed separately because it returns 200 for ANY slug,
        // making it slow and noisy. Only probe it if we haven't found anything yet.
        let (gh, lv, ab, wk) = tokio::join!(
            greenhouse::probe(client, slug),
            lever::probe(client, slug),
            ashby::probe(client, slug),
            workable::probe(client, slug),
        );

        for result in [gh, lv, ab, wk].into_iter().flatten() {
            if found_providers.insert(result.provider.to_string()) {
                results.push(result);
            }
        }

        // If we've found all 4 fast providers, stop early.
        if found_providers.len() >= 4 {
            break;
        }
    }

    // Only probe SmartRecruiters if we found nothing on faster providers.
    // SmartRecruiters returns HTTP 200 for any slug (totalFound:0), so every
    // candidate generates a response — very slow for no value when we already
    // have a hit.
    if results.is_empty() {
        for slug in &candidates {
            if let Some(result) = smartrecruiters::probe(client, slug).await {
                results.push(result);
                break; // One SR hit is enough.
            }
        }
    }

    results
}

/// Result for a single company's resolution attempt.
struct ResolveResult {
    id: i64,
    name: String,
    portals: Vec<SlugProbeResult>,
}

/// Run the resolve pipeline for all pending companies — concurrently.
pub async fn run(conn: &Connection, dry_run: bool) {
    let companies = get_pending_companies(conn);

    if companies.is_empty() {
        println!("No pending companies to resolve.");
        return;
    }

    println!("Resolving {} companies (parallel)...\n", companies.len());

    let client = crate::http::build_client();
    let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent companies.

    // Spawn all probe tasks concurrently, with retry to handle flaky responses.
    let mut handles = Vec::new();

    for (id, name, _website) in companies {
        let client = client.clone();
        let sem = semaphore.clone();
        let name_clone = name.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let portals = probe_all_providers(&client, &name_clone).await;
            ResolveResult {
                id,
                name: name_clone,
                portals,
            }
        });

        handles.push(handle);
    }

    // Collect all results.
    let mut results: Vec<ResolveResult> = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    // Sort by name for clean output.
    results.sort_by(|a, b| a.name.cmp(&b.name));

    // Print results and write to DB.
    let mut resolved_count = 0u64;
    let mut unresolved_count = 0u64;

    for result in &results {
        if result.portals.is_empty() {
            println!("  ✗ {:<35} no ATS found", result.name);
            unresolved_count += 1;
            continue;
        }

        let max_jobs = result.portals.iter().map(|r| r.job_count).max().unwrap_or(0);

        for portal in &result.portals {
            let is_primary = portal.job_count == max_jobs;
            println!(
                "  {} {:<35} {} / {} ({} jobs{})",
                if is_primary { "✓" } else { "·" },
                result.name,
                portal.provider,
                portal.slug,
                portal.job_count,
                if is_primary { ", primary" } else { "" }
            );

            if !dry_run {
                insert_portal(conn, result.id, portal, is_primary);
            }
        }

        if !dry_run {
            let _ = conn.execute(
                "UPDATE companies SET status = 'resolved' WHERE id = ?1",
                params![result.id],
            );
        }

        resolved_count += 1;
    }

    println!();
    println!("── Summary ──");
    println!("  Resolved:   {resolved_count}");
    println!("  Unresolved: {unresolved_count} (need AI fallback)");

    if dry_run {
        println!("\n  (dry run — nothing was written to the database)");
    }
}

/// Run resolve for a single company by name.
pub async fn run_single(conn: &Connection, company_name: &str, dry_run: bool) {
    let companies = get_pending_companies(conn);
    let matching: Vec<_> = companies
        .into_iter()
        .filter(|(_, name, _)| name.to_lowercase().contains(&company_name.to_lowercase()))
        .collect();

    if matching.is_empty() {
        println!("No pending company matching '{company_name}' found.");
        return;
    }

    let client = crate::http::build_client();

    for (id, name, _website) in &matching {
        print!("  {name}: ");
        let results = probe_all_providers(&client, name).await;

        if results.is_empty() {
            println!("no ATS found — needs AI resolution");
            continue;
        }

        println!();
        let max_jobs = results.iter().map(|r| r.job_count).max().unwrap_or(0);
        for result in &results {
            let is_primary = result.job_count == max_jobs;
            println!(
                "    {} {} / {} ({} jobs)",
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
