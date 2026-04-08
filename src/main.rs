mod ats;
mod config;
mod db;
mod pipeline;
mod tui;

use db::Database;
use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("tui") => cmd_tui(),
        Some("resolve") => cmd_resolve(&args).await,
        Some("search") => cmd_search(&args).await,
        Some("clean") => cmd_clean(&args),
        Some("check") => cmd_check(&args).await,
        Some("stats") => cmd_stats(),
        Some("pending") => cmd_pending(&args),
        Some("lever-list") => cmd_lever_list(&args).await,
        Some("lever-detail") => cmd_lever_detail(&args).await,
        Some("db-status") => cmd_stats(),
        _ => print_usage(),
    }
}

// ── Pipeline commands ────────────────────────────────────────────

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
}

/// Show pending (ungraded) jobs.
fn cmd_pending(args: &[String]) {
    let db = open_db();
    let conn = db.conn();

    let count_only = args.iter().any(|a| a == "--count");

    let pending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE grade IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if count_only {
        println!("{pending}");
        return;
    }

    println!("{pending} jobs pending grading\n");

    if pending > 0 {
        let mut stmt = conn
            .prepare(
                "SELECT j.title, c.name, c.grade
                 FROM jobs j
                 JOIN companies c ON c.id = j.company_id
                 WHERE j.grade IS NULL
                 ORDER BY
                     CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END,
                     c.name, j.title
                 LIMIT 50",
            )
            .expect("failed to prepare");

        let rows: Vec<(String, String, Option<String>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("failed to query")
            .filter_map(|r| r.ok())
            .collect();

        for (title, company, grade) in &rows {
            let g = grade.as_deref().unwrap_or("—");
            println!("  [{g}] {title} @ {company}");
        }

        if pending > 50 {
            println!("  ... and {} more", pending - 50);
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
    let db_path = Path::new("state/cernio.db");
    if !db_path.exists() {
        eprintln!("Database not found at state/cernio.db");
        eprintln!("Run a session to populate the database first.");
        std::process::exit(1);
    }

    if let Err(e) = tui::run(db_path) {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn open_db() -> Database {
    let db_path = Path::new("state/cernio.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create state/ directory");
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
    println!("  resolve [--company NAME] [--dry-run]     Resolve ATS portals for pending companies");
    println!("  search [--company NAME] [--grade G] [--dry-run]  Search resolved companies for jobs");
    println!("  clean [--dry-run] [--jobs-only]          Remove stale/low-grade entries");
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
