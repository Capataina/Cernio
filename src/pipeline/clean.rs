use rusqlite::{params, Connection};

use crate::config::CleanupConfig;

/// Summary of what the cleanup would or did remove.
#[allow(dead_code)]
pub struct CleanupReport {
    pub jobs_removed: u64,
    pub jobs_by_grade: Vec<(String, u64)>,
    pub stale_jobs: u64,
    pub expired_archived: u64,
    pub companies_archived: u64,
    pub preserved_by_decision: u64,
    pub preserved_by_grade: u64,
}

/// Run the cleanup pipeline.
pub fn run(conn: &Connection, config: &CleanupConfig, dry_run: bool, jobs_only: bool) {
    let report = if dry_run {
        preview(conn, config, jobs_only)
    } else {
        execute(conn, config, jobs_only)
    };

    print_report(&report, dry_run);
}

/// Preview what would be removed without actually doing it.
fn preview(conn: &Connection, config: &CleanupConfig, jobs_only: bool) -> CleanupReport {
    let mut jobs_removed = 0u64;
    let mut jobs_by_grade = Vec::new();

    // Count jobs that would be archived by tiered rules.
    // SS=28d, S=21d, A=14d, B=7d, C/F=3d.
    let tier_rules: &[(&str, i64)] = &[
        ("SS", 28), ("S", 21), ("A", 14), ("B", 7), ("C", 3), ("F", 3),
    ];
    let mut stale = 0i64;
    for (grade, days) in tier_rules {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM jobs
                 WHERE grade = ?1
                 AND evaluation_status != 'archived'
                 AND datetime(discovered_at) < datetime('now', ?2)
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
                params![grade, format!("-{days} days")],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if count > 0 {
            jobs_by_grade.push((grade.to_string(), count as u64));
            jobs_removed += count as u64;
        }
        stale += count;
    }

    // Count archived jobs that would expire (14+ days in archive).
    let expired_archived: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs
             WHERE evaluation_status = 'archived'
             AND (
                 (archived_at IS NOT NULL AND datetime(archived_at) < datetime('now', '-14 days'))
                 OR (archived_at IS NULL AND datetime(discovered_at) < datetime('now', '-42 days'))
             )
             AND id NOT IN (SELECT job_id FROM user_decisions)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Count preserved by decision.
    let preserved_by_decision: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT j.id) FROM jobs j
             JOIN user_decisions ud ON ud.job_id = j.id
             WHERE j.grade IN (SELECT value FROM json_each(?1))
             OR datetime(j.discovered_at) < datetime('now', ?2)",
            params![
                serde_json::to_string(&config.remove_job_grades).unwrap_or_default(),
                format!("-{} days", config.stale_days)
            ],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Count companies to archive.
    let companies_archived = if jobs_only {
        0
    } else {
        count_archivable_companies(conn, config)
    };

    CleanupReport {
        jobs_removed,
        jobs_by_grade,
        stale_jobs: stale as u64,
        expired_archived: expired_archived as u64,
        companies_archived,
        preserved_by_decision: preserved_by_decision as u64,
        preserved_by_grade: 0,
    }
}

/// Execute the cleanup.
fn execute(conn: &Connection, config: &CleanupConfig, jobs_only: bool) -> CleanupReport {
    let mut jobs_removed = 0u64;
    let mut jobs_by_grade = Vec::new();

    // Tiered archival: each grade has a different active duration.
    // SS=28d, S=21d, A=14d, B=7d, C/F=3d.
    // Jobs with user decisions are always preserved.
    let tier_rules: &[(&str, i64)] = &[
        ("SS", 28),
        ("S", 21),
        ("A", 14),
        ("B", 7),
        ("C", 3),
        ("F", 3),
    ];

    let mut stale = 0u64;
    for (grade, days) in tier_rules {
        let count = conn
            .execute(
                "UPDATE jobs SET evaluation_status = 'archived', archived_at = datetime('now')
                 WHERE grade = ?1
                 AND evaluation_status != 'archived'
                 AND datetime(discovered_at) < datetime('now', ?2)
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
                params![grade, format!("-{days} days")],
            )
            .unwrap_or(0) as u64;
        if count > 0 {
            jobs_by_grade.push((grade.to_string(), count));
            jobs_removed += count;
        }
        stale += count;
    }

    // Delete archived jobs that have been in the archive for 14+ days.
    // Uses archived_at timestamp, not discovered_at.
    // Falls back to discovered_at + 42 days for jobs archived before migration 005.
    let expired_archived = conn
        .execute(
            "DELETE FROM jobs
             WHERE evaluation_status = 'archived'
             AND (
                 (archived_at IS NOT NULL AND datetime(archived_at) < datetime('now', '-14 days'))
                 OR (archived_at IS NULL AND datetime(discovered_at) < datetime('now', '-42 days'))
             )
             AND id NOT IN (SELECT job_id FROM user_decisions)",
            [],
        )
        .unwrap_or(0) as u64;

    // Archive companies.
    let companies_archived = if jobs_only {
        0
    } else {
        archive_companies(conn, config)
    };

    CleanupReport {
        jobs_removed,
        jobs_by_grade,
        stale_jobs: stale,
        expired_archived,
        companies_archived,
        preserved_by_decision: 0,
        preserved_by_grade: 0,
    }
}

fn count_archivable_companies(conn: &Connection, config: &CleanupConfig) -> u64 {
    let mut total = 0u64;
    for grade in &config.archive_company_grades {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM companies
                 WHERE grade = ?1 AND status != 'archived'",
                params![grade],
                |row| row.get(0),
            )
            .unwrap_or(0);
        total += count as u64;
    }
    total
}

fn archive_companies(conn: &Connection, config: &CleanupConfig) -> u64 {
    let mut total = 0u64;
    for grade in &config.archive_company_grades {
        let count = conn
            .execute(
                "UPDATE companies SET status = 'archived'
                 WHERE grade = ?1 AND status != 'archived'",
                params![grade],
            )
            .unwrap_or(0) as u64;
        total += count;
    }
    total
}

fn print_report(report: &CleanupReport, dry_run: bool) {
    let verb = if dry_run { "would archive" } else { "archived" };
    let verb_archive = if dry_run { "would archive" } else { "archived" };

    println!("── Cleanup Report ──\n");

    if report.jobs_by_grade.is_empty() && report.stale_jobs == 0 && report.companies_archived == 0 {
        println!("  Nothing to clean. Database is tidy.");
        return;
    }

    println!("  Jobs:");
    for (grade, count) in &report.jobs_by_grade {
        println!("    {verb} {count} {grade}-graded jobs");
    }
    if report.stale_jobs > 0 {
        println!("    {verb} {} stale jobs (>14 days)", report.stale_jobs);
    }
    if report.expired_archived > 0 {
        let verb_del = if dry_run { "would delete" } else { "deleted" };
        println!("    {verb_del} {} expired archived jobs (>42 days)", report.expired_archived);
    }
    if report.preserved_by_decision > 0 {
        println!(
            "    preserved {} (have user decisions)",
            report.preserved_by_decision
        );
    }

    if report.companies_archived > 0 {
        println!("\n  Companies:");
        println!(
            "    {verb_archive} {} companies",
            report.companies_archived
        );
    }

    println!(
        "\n  Total jobs {verb}: {}",
        report.jobs_removed + report.stale_jobs
    );
    if report.expired_archived > 0 {
        let verb_del = if dry_run { "would delete" } else { "deleted" };
        println!("  Total archived {verb_del}: {}", report.expired_archived);
    }

    if dry_run {
        println!("\n  (dry run — nothing was changed)");
    }
}
