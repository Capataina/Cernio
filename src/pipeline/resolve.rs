use std::sync::Arc;

use rusqlite::{params, Connection};
use tokio::sync::Semaphore;

use crate::ats::common::SlugProbeResult;
use crate::ats::{ashby, greenhouse, lever, smartrecruiters, workable};

// Note: Lever now has its own probe() function that tries both US and EU endpoints.

/// Generate slug candidates from a company name.
/// Tries common transformations that ATS providers use.
/// Intentionally generates many candidates — false probes are cheap (one HTTP 404),
/// while missing the right slug means the company never gets searched.
fn slug_candidates(name: &str) -> Vec<String> {
    let lower = name.to_lowercase();
    let mut candidates = Vec::new();

    // Exact lowercase with spaces removed.
    candidates.push(lower.replace(' ', ""));
    // Hyphenated.
    candidates.push(lower.replace(' ', "-"));
    // Underscored.
    candidates.push(lower.replace(' ', "_"));

    // Strip ALL punctuation and try (e.g. "D.E. Shaw" → "deshaw").
    let no_punct: String = lower.chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect();
    candidates.push(no_punct.replace(' ', ""));
    candidates.push(no_punct.replace(' ', "-"));

    // Strip common suffixes and try again.
    for suffix in &[
        " ltd", " limited", " inc", " incorporated", " corp", " corporation",
        " plc", " group", " holdings", " capital", " partners",
        " technologies", " technology", " tech", " labs", " laboratory",
        " ai", " hq", " io", " uk", " global", " systems", " software",
        " gmbh", " ag", " sa", " bv", " pty",
    ] {
        if let Some(stripped) = lower.strip_suffix(suffix) {
            candidates.push(stripped.replace(' ', ""));
            candidates.push(stripped.replace(' ', "-"));
        }
    }

    // Also try WITH common suffixes appended (XTX → xtxmarketstechnologies).
    let no_spaces = lower.replace(' ', "");
    candidates.push(format!("{no_spaces}technologies"));
    candidates.push(format!("{no_spaces}tech"));
    candidates.push(format!("{no_spaces}hq"));
    candidates.push(format!("{no_spaces}careers"));
    candidates.push(format!("{no_spaces}jobs"));

    // Try just the first word (e.g. "Helsing GmbH" → "helsing").
    if let Some(first) = lower.split_whitespace().next() {
        candidates.push(first.to_string());
    }

    // Try first two words hyphenated (e.g. "Tower Research Capital" → "tower-research").
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.len() >= 2 {
        candidates.push(format!("{}-{}", words[0], words[1]));
        candidates.push(format!("{}{}", words[0], words[1]));
    }

    // Try initials/acronym (e.g. "XTX Markets" → "xtx", "DRW" → "drw").
    if words.len() >= 2 {
        let initials: String = words.iter().map(|w| w.chars().next().unwrap_or('_')).collect();
        candidates.push(initials);
    }

    // Strip parenthetical suffixes: "Man Group (AHL)" → "mangroup", also try "ahl"
    if let Some(paren_pos) = lower.find('(') {
        let without_paren = lower[..paren_pos].trim();
        candidates.push(without_paren.replace(' ', ""));
        candidates.push(without_paren.replace(' ', "-"));
        // Also try the part inside parens
        if let Some(close) = lower.find(')') {
            let inside = lower[paren_pos + 1..close].trim();
            if !inside.is_empty() {
                candidates.push(inside.to_string());
            }
        }
    }

    // Strip slashes: "Refinitiv / LSEG" → "refinitiv", also try "lseg"
    if let Some(slash_pos) = lower.find('/') {
        let first_part = lower[..slash_pos].trim();
        candidates.push(first_part.replace(' ', ""));
        candidates.push(first_part.replace(' ', "-"));
        let second_part = lower[slash_pos + 1..].trim();
        if !second_part.is_empty() {
            candidates.push(second_part.replace(' ', ""));
            candidates.push(second_part.replace(' ', "-"));
        }
    }

    // Strip domain suffixes: "Copper.co" → "copper", "Modal.com" → "modal"
    for suffix in &[".co", ".io", ".ai", ".com", ".dev"] {
        if let Some(stripped) = lower.strip_suffix(suffix) {
            candidates.push(stripped.replace(' ', ""));
        }
    }

    // Deduplicate while preserving order.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()) && !c.is_empty());

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
        // Probe all fast providers in parallel for every slug candidate.
        // Don't stop early — companies can use multiple ATS providers
        // (e.g. Greenhouse for engineering, Workable for corporate).
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
    }

    // Probe SmartRecruiters separately — it returns HTTP 200 for any slug
    // (totalFound:0), so it's slower and noisier. Try all candidates but
    // stop after the first real hit.
    if !found_providers.contains("smartrecruiters") {
        for slug in &candidates {
            if let Some(result) = smartrecruiters::probe(client, slug).await {
                results.push(result);
                break;
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
