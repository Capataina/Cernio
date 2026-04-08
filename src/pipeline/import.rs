use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

/// A company parsed from potential.md.
struct PotentialCompany {
    name: String,
    website: String,
    what_they_do: String,
    why_relevant: String,
    source: String,
    sector: String,
}

/// Parse potential.md and bulk-insert companies into the DB.
/// Deduplicates by website — companies already in the DB are skipped.
pub fn run(conn: &Connection, path: &Path, dry_run: bool) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {e}", path.display());
            return;
        }
    };

    let companies = parse_potential_md(&content);

    if companies.is_empty() {
        println!("No companies found in {}", path.display());
        return;
    }

    println!("Parsed {} companies from {}\n", companies.len(), path.display());

    let mut inserted = 0u64;
    let mut skipped_duplicate = 0u64;
    let mut skipped_invalid = 0u64;

    for company in &companies {
        // Validate: must have website and name at minimum.
        if company.website.is_empty() || company.name.is_empty() {
            println!("  ✗ {:<30} missing name or website — skipped", company.name);
            skipped_invalid += 1;
            continue;
        }

        // Normalise website for dedup (strip trailing slash).
        let website = company.website.trim_end_matches('/');

        // Check if already in DB.
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM companies WHERE website = ?1 OR website = ?2",
                params![website, format!("{}/", website)],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if exists {
            println!("  · {:<30} already in DB — skipped", company.name);
            skipped_duplicate += 1;
            continue;
        }

        if dry_run {
            println!("  + {:<30} {} [{}]", company.name, website, company.sector);
            inserted += 1;
            continue;
        }

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let source = format!("discovery: {}", company.source);

        let result = conn.execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, status, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'potential', ?6, ?7)",
            params![
                company.name,
                website,
                company.what_they_do,
                source,
                now,
                company.why_relevant,
                now,
            ],
        );

        match result {
            Ok(_) => {
                println!("  ✓ {:<30} inserted [{}]", company.name, company.sector);
                inserted += 1;
            }
            Err(e) => {
                println!("  ✗ {:<30} insert failed: {e}", company.name);
                skipped_invalid += 1;
            }
        }
    }

    println!();
    println!("── Summary ──");
    println!("  Inserted:  {inserted}");
    println!("  Duplicate: {skipped_duplicate}");
    println!("  Invalid:   {skipped_invalid}");

    if dry_run {
        println!("\n  (dry run — nothing was written to the database)");
    } else if inserted > 0 {
        // Clear the potential file after successful import.
        let header = "# Potential Companies\n\n> Imported to database. File cleared automatically after import.\n";
        match fs::write(path, header) {
            Ok(_) => println!("  Cleared {} after import", path.display()),
            Err(e) => eprintln!("  Warning: could not clear {}: {e}", path.display()),
        }
    }
}

/// Parse the potential.md markdown format into structured companies.
///
/// Expected format:
/// ```markdown
/// ## Sector Name
///
/// ### Company Name
/// - **Website**: https://...
/// - **What they do**: ...
/// - **Why relevant**: ...
/// - **Source**: ...
/// ```
fn parse_potential_md(content: &str) -> Vec<PotentialCompany> {
    let mut companies = Vec::new();
    let mut current_sector = String::new();
    let mut current_name = String::new();
    let mut current_website = String::new();
    let mut current_what = String::new();
    let mut current_why = String::new();
    let mut current_source = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Sector header (## ...).
        if let Some(sector) = trimmed.strip_prefix("## ") {
            // Flush any pending company.
            flush_company(
                &mut companies,
                &current_name,
                &current_website,
                &current_what,
                &current_why,
                &current_source,
                &current_sector,
            );
            current_sector = sector.to_string();
            current_name.clear();
            continue;
        }

        // Company header (### ...).
        if let Some(name) = trimmed.strip_prefix("### ") {
            // Flush previous company.
            flush_company(
                &mut companies,
                &current_name,
                &current_website,
                &current_what,
                &current_why,
                &current_source,
                &current_sector,
            );
            current_name = name.to_string();
            current_website.clear();
            current_what.clear();
            current_why.clear();
            current_source.clear();
            continue;
        }

        // Field lines.
        if let Some(val) = extract_field(trimmed, "Website") {
            current_website = val;
        } else if let Some(val) = extract_field(trimmed, "What they do") {
            current_what = val;
        } else if let Some(val) = extract_field(trimmed, "Why relevant") {
            current_why = val;
        } else if let Some(val) = extract_field(trimmed, "Source") {
            current_source = val;
        }
    }

    // Flush last company.
    flush_company(
        &mut companies,
        &current_name,
        &current_website,
        &current_what,
        &current_why,
        &current_source,
        &current_sector,
    );

    companies
}

fn extract_field(line: &str, field: &str) -> Option<String> {
    // Match "- **Field**: value" or "- **Field:** value".
    let pattern1 = format!("- **{field}**: ");
    let pattern2 = format!("- **{field}:** ");

    if let Some(val) = line.strip_prefix(&pattern1) {
        Some(val.to_string())
    } else if let Some(val) = line.strip_prefix(&pattern2) {
        Some(val.to_string())
    } else {
        None
    }
}

fn flush_company(
    companies: &mut Vec<PotentialCompany>,
    name: &str,
    website: &str,
    what: &str,
    why: &str,
    source: &str,
    sector: &str,
) {
    if name.is_empty() {
        return;
    }
    companies.push(PotentialCompany {
        name: name.to_string(),
        website: website.to_string(),
        what_they_do: if what.is_empty() {
            format!("(no description — needs research)")
        } else {
            what.to_string()
        },
        why_relevant: if why.is_empty() {
            format!("Discovered in sector: {sector}")
        } else {
            why.to_string()
        },
        source: if source.is_empty() {
            "discovery".to_string()
        } else {
            source.to_string()
        },
        sector: sector.to_string(),
    });
}
