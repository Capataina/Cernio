use rusqlite::Connection;

use crate::ats::{ashby, greenhouse, lever, smartrecruiters, workable};

/// Full integrity report.
pub struct IntegrityReport {
    pub health: Vec<CheckResult>,
    pub completeness: Vec<CheckResult>,
    pub staleness: Vec<CheckResult>,
    pub recommendations: Vec<String>,
}

pub struct CheckResult {
    pub status: CheckStatus,
    pub message: String,
}

pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

/// Run all integrity checks and print the report.
pub async fn run(conn: &Connection, ats_only: bool) {
    let report = if ats_only {
        run_ats_only(conn).await
    } else {
        run_full(conn).await
    };

    print_report(&report);
}

async fn run_full(conn: &Connection) -> IntegrityReport {
    let mut health = Vec::new();
    let mut completeness = Vec::new();
    let mut staleness = Vec::new();
    let mut recommendations = Vec::new();

    // ── Health checks ────────────────────────────────────────

    // ATS slug verification.
    let (ats_ok, ats_total, ats_failed) = verify_ats_slugs(conn).await;
    if ats_failed.is_empty() {
        health.push(CheckResult {
            status: CheckStatus::Ok,
            message: format!("ATS slugs: {ats_ok}/{ats_total} verified"),
        });
    } else {
        health.push(CheckResult {
            status: CheckStatus::Error,
            message: format!(
                "ATS slugs: {ats_ok}/{ats_total} verified — {} failed: {}",
                ats_failed.len(),
                ats_failed.join(", ")
            ),
        });
        recommendations.push(format!(
            "Re-resolve failed ATS slugs: {}",
            ats_failed.join(", ")
        ));
    }

    // Orphaned decisions.
    let orphaned: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_decisions
             WHERE job_id NOT IN (SELECT id FROM jobs)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if orphaned == 0 {
        health.push(CheckResult {
            status: CheckStatus::Ok,
            message: "No orphaned decisions".to_string(),
        });
    } else {
        health.push(CheckResult {
            status: CheckStatus::Error,
            message: format!("{orphaned} orphaned decisions (pointing to deleted jobs)"),
        });
        recommendations.push("Run cernio clean --fix to remove orphaned decisions".to_string());
    }

    // Missing portal entries.
    let missing_portals: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies
             WHERE status = 'resolved'
             AND id NOT IN (SELECT company_id FROM company_portals)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if missing_portals == 0 {
        health.push(CheckResult {
            status: CheckStatus::Ok,
            message: "All resolved companies have portal entries".to_string(),
        });
    } else {
        health.push(CheckResult {
            status: CheckStatus::Error,
            message: format!("{missing_portals} resolved companies have no portal entries"),
        });
        recommendations.push("Run cernio resolve to fix missing portals".to_string());
    }

    // Duplicate companies (similar websites).
    let duplicates: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM (
                SELECT website, COUNT(*) as cnt FROM companies GROUP BY website HAVING cnt > 1
            )",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if duplicates == 0 {
        health.push(CheckResult {
            status: CheckStatus::Ok,
            message: "No duplicate companies".to_string(),
        });
    } else {
        health.push(CheckResult {
            status: CheckStatus::Warning,
            message: format!("{duplicates} duplicate company websites detected"),
        });
    }

    // ── Completeness checks ──────────────────────────────────

    let ungraded_companies: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE grade IS NULL AND status != 'archived'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    completeness.push(check_count(
        ungraded_companies,
        "companies ungraded",
        "Run grade-companies",
        &mut recommendations,
    ));

    let ungraded_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE grade IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    completeness.push(check_count(
        ungraded_jobs,
        "jobs pending grading",
        "Run grade-jobs",
        &mut recommendations,
    ));

    let missing_descriptions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE raw_description IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    completeness.push(check_count(
        missing_descriptions,
        "jobs missing descriptions",
        "Fetch descriptions for better grading quality",
        &mut recommendations,
    ));

    let missing_assessments: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE grade IS NOT NULL AND fit_assessment IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    completeness.push(check_count(
        missing_assessments,
        "graded jobs missing fit assessments",
        "Re-run grade-jobs with assessment writing",
        &mut recommendations,
    ));

    let unresolved: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'potential'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    completeness.push(check_count(
        unresolved,
        "companies unresolved",
        "Run cernio resolve",
        &mut recommendations,
    ));

    // ── Staleness checks ─────────────────────────────────────

    let stale_grades: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies
             WHERE graded_at IS NOT NULL
             AND graded_at < datetime('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stale_grades == 0 {
        staleness.push(CheckResult {
            status: CheckStatus::Ok,
            message: "All company grades are fresh (<30 days)".to_string(),
        });
    } else {
        staleness.push(CheckResult {
            status: CheckStatus::Warning,
            message: format!("{stale_grades} company grades older than 30 days"),
        });
        recommendations.push(format!("Re-grade {stale_grades} stale companies"));
    }

    let stale_jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs
             WHERE discovered_at < datetime('now', '-14 days')
             AND grade NOT IN ('SS', 'S')
             AND id NOT IN (SELECT job_id FROM user_decisions)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stale_jobs == 0 {
        staleness.push(CheckResult {
            status: CheckStatus::Ok,
            message: "No stale jobs (>14 days without decisions)".to_string(),
        });
    } else {
        staleness.push(CheckResult {
            status: CheckStatus::Warning,
            message: format!("{stale_jobs} stale jobs (>14 days, no decision)"),
        });
        recommendations.push("Run cernio clean to remove stale jobs".to_string());
    }

    IntegrityReport {
        health,
        completeness,
        staleness,
        recommendations,
    }
}

async fn run_ats_only(conn: &Connection) -> IntegrityReport {
    let (ok, total, failed) = verify_ats_slugs(conn).await;
    let health = vec![if failed.is_empty() {
        CheckResult {
            status: CheckStatus::Ok,
            message: format!("ATS slugs: {ok}/{total} verified"),
        }
    } else {
        CheckResult {
            status: CheckStatus::Error,
            message: format!(
                "ATS slugs: {ok}/{total} verified — failed: {}",
                failed.join(", ")
            ),
        }
    }];

    IntegrityReport {
        health,
        completeness: Vec::new(),
        staleness: Vec::new(),
        recommendations: if failed.is_empty() {
            Vec::new()
        } else {
            vec![format!("Re-resolve: {}", failed.join(", "))]
        },
    }
}

async fn verify_ats_slugs(conn: &Connection) -> (usize, usize, Vec<String>) {
    let portals: Vec<(String, String, String)> = conn
        .prepare(
            "SELECT c.name, p.ats_provider, p.ats_slug
             FROM company_portals p
             JOIN companies c ON c.id = p.company_id
             WHERE p.is_primary = 1",
        )
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    let total = portals.len();
    let mut ok = 0usize;
    let mut failed = Vec::new();

    let client = reqwest::Client::new();

    for (name, provider, slug) in &portals {
        let alive = match provider.as_str() {
            "greenhouse" => greenhouse::probe(&client, slug).await.is_some(),
            "lever" => lever::fetch_all(&client, slug).await.map(|v| !v.is_empty()).unwrap_or(false),
            "ashby" => ashby::probe(&client, slug).await.is_some(),
            "workable" => workable::probe(&client, slug).await.is_some(),
            "smartrecruiters" => smartrecruiters::probe(&client, slug).await.is_some(),
            _ => true, // Unknown providers get a pass.
        };

        if alive {
            ok += 1;
        } else {
            failed.push(format!("{name} ({provider}/{slug})"));
        }
    }

    (ok, total, failed)
}

fn check_count(
    count: i64,
    label: &str,
    recommendation: &str,
    recommendations: &mut Vec<String>,
) -> CheckResult {
    if count == 0 {
        CheckResult {
            status: CheckStatus::Ok,
            message: format!("No {label}"),
        }
    } else {
        recommendations.push(format!("{recommendation} ({count} {label})"));
        CheckResult {
            status: CheckStatus::Warning,
            message: format!("{count} {label}"),
        }
    }
}

fn print_report(report: &IntegrityReport) {
    println!("┌─ Integrity Report ─────────────────────────────────────────┐");
    println!("│                                                             │");

    if !report.health.is_empty() {
        println!("│  ── Health ──                                               │");
        for check in &report.health {
            let icon = match check.status {
                CheckStatus::Ok => "✓",
                CheckStatus::Warning => "⚠",
                CheckStatus::Error => "✗",
            };
            println!("│  {icon}  {:<55}│", check.message);
        }
        println!("│                                                             │");
    }

    if !report.completeness.is_empty() {
        println!("│  ── Completeness ──                                         │");
        for check in &report.completeness {
            let icon = match check.status {
                CheckStatus::Ok => "✓",
                CheckStatus::Warning => "⚠",
                CheckStatus::Error => "✗",
            };
            println!("│  {icon}  {:<55}│", check.message);
        }
        println!("│                                                             │");
    }

    if !report.staleness.is_empty() {
        println!("│  ── Staleness ──                                            │");
        for check in &report.staleness {
            let icon = match check.status {
                CheckStatus::Ok => "✓",
                CheckStatus::Warning => "⚠",
                CheckStatus::Error => "✗",
            };
            println!("│  {icon}  {:<55}│", check.message);
        }
        println!("│                                                             │");
    }

    if !report.recommendations.is_empty() {
        println!("│  ── Recommended Actions ──                                  │");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("│  {}.  {:<53}│", i + 1, rec);
        }
        println!("│                                                             │");
    }

    println!("└─────────────────────────────────────────────────────────────┘");
}
