use cernio::{ats, config, db, pipeline, telemetry, tui};
use db::Database;
use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("");

    // Initialise telemetry for any subcommand. The writer is a singleton,
    // so calling init twice is a no-op. We want TUI sessions and pipeline
    // runs alike to land in `~/Library/Application Support/cernio/telemetry/`.
    telemetry::init();
    cernio::tel!("cli_invoked", "subcommand": subcommand, "argv": &args[1..]);

    match args.get(1).map(|s| s.as_str()) {
        Some("tui") => cmd_tui(),
        Some("import") => cmd_import(&args),
        Some("resolve") => cmd_resolve(&args).await,
        Some("search") => cmd_search(&args).await,
        Some("clean") => cmd_clean(&args),
        Some("check") => cmd_check(&args).await,
        Some("format") => cmd_format(&args),
        Some("stats") => cmd_stats(),
        Some("pending") => cmd_pending(&args),
        Some("unarchive") => cmd_unarchive(&args),
        Some("lever-list") => cmd_lever_list(&args).await,
        Some("lever-detail") => cmd_lever_detail(&args).await,
        Some("db-status") => cmd_stats(),
        _ => print_usage(),
    }
}

// ── Pipeline commands ────────────────────────────────────────────

/// Import companies from potential.md into the database.
fn cmd_import(args: &[String]) {
    let db = open_db();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let path = get_flag_value(args, "--file")
        .unwrap_or_else(|| "companies/potential.md".to_string());

    pipeline::import::run(db.conn(), std::path::Path::new(&path), dry_run);
}

/// Resolve ATS portals for pending companies.
async fn cmd_resolve(args: &[String]) {
    let db = open_db();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    if let Some(company) = get_flag_value(args, "--company") {
        pipeline::resolve::run_single(db.conn(), &company, dry_run).await;
    } else {
        pipeline::resolve::run(db.conn(), dry_run).await;
    }
}

/// Search for jobs at resolved companies.
async fn cmd_search(args: &[String]) {
    let db = open_db();
    let prefs = config::Preferences::load();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    if let Some(company) = get_flag_value(args, "--company") {
        pipeline::search::run_single(db.conn(), &prefs.search_filters, &company, dry_run).await;
    } else if let Some(grade) = get_flag_value(args, "--grade") {
        pipeline::search::run_by_grade(db.conn(), &prefs.search_filters, &grade, dry_run).await;
    } else {
        pipeline::search::run(db.conn(), &prefs.search_filters, dry_run).await;
    }
}

/// Clean stale and low-grade entries.
fn cmd_clean(args: &[String]) {
    let db = open_db();
    let prefs = config::Preferences::load();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let jobs_only = args.iter().any(|a| a == "--jobs-only");

    pipeline::clean::run(db.conn(), &prefs.cleanup, dry_run, jobs_only);
}

/// Format job descriptions and fit assessments.
fn cmd_format(args: &[String]) {
    let db = open_db();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    pipeline::format::run(db.conn(), dry_run);
}

/// Run integrity checks.
async fn cmd_check(args: &[String]) {
    let db = open_db();
    let ats_only = args.iter().any(|a| a == "--ats-only");

    pipeline::check::run(db.conn(), ats_only).await;
}

/// Show database overview stats.
fn cmd_stats() {
    let db = open_db();
    let conn = db.conn();

    let total_companies: i64 = conn
        .query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))
        .unwrap_or(0);
    let resolved: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'resolved'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let bespoke: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'bespoke'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let potential: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'potential'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let archived: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'archived'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let total_jobs: i64 = conn
        .query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))
        .unwrap_or(0);
    let pending_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE grade IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let graded_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE grade IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    println!("── Cernio Stats ──\n");
    println!("  Companies:  {total_companies} total");
    println!("    resolved: {resolved}  bespoke: {bespoke}  potential: {potential}  archived: {archived}");
    println!();
    println!("  Jobs:       {total_jobs} total");
    println!("    pending:  {pending_jobs}  graded: {graded_jobs}");
    println!();

    // Lane-aware breakdown — added by the lane-based-relativity refactor
    // (cernio-full-refactor.md §6). Companies and jobs that carry the new
    // lanes column are surfaced here per lane. NULL-lanes rows (un-tagged)
    // get summed into the "(untagged)" row.
    let mut company_lanes: Vec<(String, i64)> = conn
        .prepare("SELECT COALESCE(lanes, '(untagged)') AS l, COUNT(*) FROM companies GROUP BY l ORDER BY 2 DESC")
        .ok()
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    company_lanes.retain(|(_l, c)| *c > 0);

    let mut job_lanes: Vec<(String, i64)> = conn
        .prepare("SELECT COALESCE(lanes, '(untagged)') AS l, COUNT(*) FROM jobs GROUP BY l ORDER BY 2 DESC")
        .ok()
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    job_lanes.retain(|(_l, c)| *c > 0);

    let sponsor_yes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE sponsors_uk = 'yes'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let sponsor_unknown: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE sponsors_uk = 'unknown' OR sponsors_uk IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if !company_lanes.is_empty() {
        println!("  Companies per lane:");
        for (l, c) in &company_lanes {
            println!("    {l:<32} {c}");
        }
        println!();
    }

    if !job_lanes.is_empty() {
        println!("  Jobs per lane:");
        for (l, c) in &job_lanes {
            println!("    {l:<32} {c}");
        }
        println!();
    }

    println!("  Sponsor status:");
    println!("    sponsors_uk='yes':    {sponsor_yes}");
    println!("    sponsors_uk=unknown:  {sponsor_unknown}");
}

/// Show pending (ungraded) jobs.
///
/// Flags:
///   --count            Print only the count (no per-row listing)
///   --lane <key>       Filter to jobs/companies tagged with the given lane
///                      (e.g. `cernio pending --lane hft`)
///   --sponsor-only     Restrict to companies with sponsors_uk = 'yes'
///                      (default behaviour post-refactor, but the flag is
///                      retained for explicitness)
fn cmd_pending(args: &[String]) {
    let db = open_db();
    let conn = db.conn();

    let count_only = args.iter().any(|a| a == "--count");
    let sponsor_only = args.iter().any(|a| a == "--sponsor-only");

    // --lane <key> filter — added by the lane-based-relativity refactor
    // (cernio-full-refactor.md §6). Filters jobs whose `lanes` JSON array
    // contains the requested lane key.
    let lane_filter: Option<String> = args
        .iter()
        .position(|a| a == "--lane")
        .and_then(|i| args.get(i + 1).cloned());

    let lane_clause = match &lane_filter {
        Some(lane) => format!(" AND j.lanes LIKE '%\"{}\"%'", lane.replace("'", "''")),
        None => String::new(),
    };
    let sponsor_clause = if sponsor_only {
        " AND c.sponsors_uk = 'yes'"
    } else {
        ""
    };

    let pending: i64 = conn
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM jobs j
                 JOIN companies c ON c.id = j.company_id
                 WHERE j.grade IS NULL{lane_clause}{sponsor_clause}"
            ),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if count_only {
        println!("{pending}");
        return;
    }

    let filter_desc = match (&lane_filter, sponsor_only) {
        (Some(l), true) => format!(" (lane={l}, sponsor-only)"),
        (Some(l), false) => format!(" (lane={l})"),
        (None, true) => " (sponsor-only)".to_string(),
        (None, false) => String::new(),
    };
    println!("{pending} jobs pending grading{filter_desc}\n");

    if pending > 0 {
        let sql = format!(
            "SELECT j.title, c.name, c.grade, j.lanes
             FROM jobs j
             JOIN companies c ON c.id = j.company_id
             WHERE j.grade IS NULL{lane_clause}{sponsor_clause}
             ORDER BY
                 CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END,
                 c.name, j.title
             LIMIT 50"
        );
        let mut stmt = conn.prepare(&sql).expect("failed to prepare");

        let rows: Vec<(String, String, Option<String>, Option<String>)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("failed to query")
            .filter_map(|r| r.ok())
            .collect();

        for (title, company, grade, lanes) in &rows {
            let g = grade.as_deref().unwrap_or("—");
            let l = lanes.as_deref().unwrap_or("[]");
            println!("  [{g}] {title} @ {company} · {l}");
        }

        if pending > 50 {
            println!("  ... and {} more", pending - 50);
        }
    }
}

/// Unarchive previously archived jobs/companies so they can be re-evaluated.
/// Supports --grade flag to unarchive only jobs at a specific grade.
/// Resets discovered_at to now so the tiered archival timer restarts.
fn cmd_unarchive(args: &[String]) {
    let db = open_db();
    let conn = db.conn();

    let target = args.get(2).map(|s| s.as_str());
    let grade_filter = get_flag_value(args, "--grade");

    match target {
        Some("--jobs") => {
            let count = if let Some(grade) = &grade_filter {
                // Unarchive only jobs at a specific grade, preserving the grade and assessment.
                // Reset discovered_at so the tiered archival timer restarts.
                conn.execute(
                    "UPDATE jobs SET evaluation_status = CASE grade
                         WHEN 'SS' THEN 'strong_fit' WHEN 'S' THEN 'strong_fit'
                         WHEN 'A' THEN 'weak_fit' WHEN 'B' THEN 'weak_fit'
                         ELSE 'no_fit' END,
                     discovered_at = datetime('now'), archived_at = NULL
                     WHERE evaluation_status = 'archived' AND grade = ?1",
                    rusqlite::params![grade.to_uppercase()],
                )
                .unwrap_or(0)
            } else {
                // Unarchive all jobs — reset to pending for full re-grading.
                conn.execute(
                    "UPDATE jobs SET evaluation_status = 'pending', grade = NULL,
                     fit_assessment = NULL,
                     discovered_at = datetime('now'), archived_at = NULL
                     WHERE evaluation_status = 'archived'",
                    [],
                )
                .unwrap_or(0)
            };
            if let Some(g) = &grade_filter {
                println!("Unarchived {count} {}-graded jobs (timer reset).", g.to_uppercase());
            } else {
                println!("Unarchived {count} jobs (reset to pending for re-grading).");
            }
        }
        Some("--companies") => {
            let count = conn
                .execute(
                    "UPDATE companies SET status = 'resolved'
                     WHERE status = 'archived'
                     AND id IN (SELECT company_id FROM company_portals)",
                    [],
                )
                .unwrap_or(0);
            let bespoke = conn
                .execute(
                    "UPDATE companies SET status = 'bespoke'
                     WHERE status = 'archived'
                     AND id NOT IN (SELECT company_id FROM company_portals)",
                    [],
                )
                .unwrap_or(0);
            println!("Unarchived {count} companies (restored to resolved), {bespoke} (restored to bespoke).");
        }
        Some("--all") => {
            let jobs = conn
                .execute(
                    "UPDATE jobs SET evaluation_status = 'pending', grade = NULL,
                     fit_assessment = NULL,
                     discovered_at = datetime('now'), archived_at = NULL
                     WHERE evaluation_status = 'archived'",
                    [],
                )
                .unwrap_or(0);
            let companies = conn
                .execute(
                    "UPDATE companies SET status = 'resolved'
                     WHERE status = 'archived'
                     AND id IN (SELECT company_id FROM company_portals)",
                    [],
                )
                .unwrap_or(0);
            println!("Unarchived {jobs} jobs and {companies} companies.");
        }
        _ => {
            println!("Usage: cernio unarchive <--jobs|--companies|--all> [--grade G]");
            println!();
            println!("Restores archived entries so they can be re-evaluated.");
            println!("  --jobs                 Unarchive all archived jobs (reset to pending)");
            println!("  --jobs --grade A       Unarchive only A-graded jobs (preserves grade, resets timer)");
            println!("  --companies            Unarchive all archived companies");
            println!("  --all                  Unarchive everything");
        }
    }
}

// ── Legacy commands (kept for compatibility) ─────────────────────

async fn cmd_lever_list(args: &[String]) {
    let Some(slug) = args.get(2) else {
        eprintln!("Usage: cernio lever-list <slug>");
        std::process::exit(1);
    };

    let client = reqwest::Client::new();
    match ats::lever::fetch_all(&client, slug).await {
        Ok(postings) => {
            let json = serde_json::to_string_pretty(&postings).expect("failed to serialise");
            println!("{json}");
        }
        Err(e) => {
            eprintln!("Error fetching from Lever: {e}");
            std::process::exit(1);
        }
    }
}

async fn cmd_lever_detail(args: &[String]) {
    let (Some(slug), Some(id)) = (args.get(2), args.get(3)) else {
        eprintln!("Usage: cernio lever-detail <slug> <job-id>");
        std::process::exit(1);
    };

    let client = reqwest::Client::new();
    match ats::lever::fetch_detail(&client, slug, id).await {
        Ok(detail) => {
            println!("# {}", detail.text);
            if let Some(loc) = &detail.categories.location {
                println!("Location: {loc}");
            }
            if let Some(desc) = &detail.description_plain {
                println!("\n{desc}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

// ── TUI ──────────────────────────────────────────────────────────

fn cmd_tui() {
    let db_path_string = db_path_from_env();
    let db_path = Path::new(&db_path_string);
    if !db_path.exists() {
        eprintln!("Database not found at {}", db_path.display());
        eprintln!("Run a session to populate the database first.");
        std::process::exit(1);
    }

    if let Err(e) = tui::run(db_path) {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}

// ── Helpers ──────────────────────────────────────────────────────

/// Resolve the database path, honouring the `CERNIO_DB_PATH` env var.
///
/// Defaults to `state/cernio.db` for normal operation. Tests override this
/// env var to point at a temporary per-test database so they never touch the
/// real one.
fn db_path_from_env() -> String {
    std::env::var("CERNIO_DB_PATH").unwrap_or_else(|_| "state/cernio.db".to_string())
}

fn open_db() -> Database {
    let db_path_string = db_path_from_env();
    let db_path = Path::new(&db_path_string);
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).expect("failed to create state/ directory");
        }
    }
    Database::open(db_path).expect("failed to open database")
}

fn get_flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn print_usage() {
    println!("Usage: cernio <command>\n");
    println!("Pipeline commands:");
    println!("  import [--file PATH] [--dry-run]         Import companies from potential.md into DB");
    println!("  resolve [--company NAME] [--dry-run]     Resolve ATS portals for pending companies");
    println!("  search [--company NAME] [--grade G] [--dry-run]  Search resolved companies for jobs");
    println!("  clean [--dry-run] [--jobs-only]          Archive stale/low-grade entries");
    println!("  unarchive <--jobs|--companies|--all>     Restore archived entries for re-evaluation");
    println!("  format [--dry-run]                        Format job descriptions and assessments");
    println!("  check [--ats-only]                       Run integrity checks");
    println!();
    println!("Info commands:");
    println!("  tui                                      Launch the interactive dashboard");
    println!("  stats                                    Show database overview");
    println!("  pending [--count]                        Show ungraded jobs");
    println!();
    println!("Legacy commands:");
    println!("  lever-list <slug>                        List jobs at a Lever company (JSON)");
    println!("  lever-detail <slug> <id>                 Fetch a Lever job description");
}
