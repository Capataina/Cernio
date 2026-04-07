mod ats;
mod db;
mod tui;

use db::Database;
use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("tui") => cmd_tui(),
        Some("lever-list") => cmd_lever_list(&args).await,
        Some("lever-detail") => cmd_lever_detail(&args).await,
        Some("db-status") => cmd_db_status(),
        _ => print_usage(),
    }
}

/// List all jobs at a Lever company. Outputs JSON.
/// Usage: cernio lever-list <slug>
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

/// Fetch full details for a single Lever posting. Outputs JSON.
/// Usage: cernio lever-detail <slug> <job-id>
async fn cmd_lever_detail(args: &[String]) {
    let (Some(slug), Some(id)) = (args.get(2), args.get(3)) else {
        eprintln!("Usage: cernio lever-detail <slug> <job-id>");
        std::process::exit(1);
    };

    let client = reqwest::Client::new();
    match ats::lever::fetch_detail(&client, slug, id).await {
        Ok(detail) => {
            // Print structured summary, not raw JSON, for readability.
            println!("# {}", detail.text);
            println!();
            if let Some(loc) = &detail.categories.location {
                println!("Location: {loc}");
            }
            if let Some(dept) = &detail.categories.department {
                println!("Department: {dept}");
            }
            if let Some(team) = &detail.categories.team {
                println!("Team: {team}");
            }
            if let Some(wt) = &detail.workplace_type {
                println!("Workplace: {wt}");
            }
            println!();

            if let Some(desc) = &detail.description_plain {
                println!("## Description\n");
                println!("{desc}");
            }

            if let Some(lists) = &detail.lists {
                for list in lists {
                    println!("\n## {}\n", list.text);
                    println!("{}", ats::lever::strip_html(&list.content));
                }
            }

            if let Some(additional) = &detail.additional_plain {
                if !additional.is_empty() {
                    println!("\n## Additional\n");
                    println!("{additional}");
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching detail: {e}");
            std::process::exit(1);
        }
    }
}

/// Show database status.
fn cmd_db_status() {
    let db_path = Path::new("state/cernio.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create state/ directory");
    }

    let db = Database::open(db_path).expect("failed to open database");
    let count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))
        .expect("failed to query");
    println!("Cernio database: {count} companies in universe.");
}

/// Launch the interactive TUI dashboard.
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

fn print_usage() {
    println!("Usage: cernio <command>");
    println!();
    println!("Commands:");
    println!("  tui                         Launch the interactive dashboard");
    println!("  lever-list <slug>           List all jobs at a Lever company (JSON)");
    println!("  lever-detail <slug> <id>    Fetch full job description from Lever");
    println!("  db-status                   Show database status");
}
