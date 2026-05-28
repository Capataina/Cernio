use rusqlite::Connection;

use super::app::{ActivityEntry, CompanyRow, DashboardStats, JobRow, PipelineCard, SortMode, TopMatch};

pub fn fetch_companies(conn: &Connection, show_archived: bool) -> Vec<CompanyRow> {
    let archive_filter = if show_archived { "" } else { "WHERE c.status != 'archived'" };
    let sql = format!("
        SELECT c.id, c.name, c.website, c.what_they_do, c.status, c.location,
               c.sector_tags, c.grade, c.grade_reasoning, c.why_relevant, c.careers_url,
               (SELECT p.ats_provider FROM company_portals p
                WHERE p.company_id = c.id AND p.is_primary = 1 LIMIT 1),
               (SELECT p.ats_slug FROM company_portals p
                WHERE p.company_id = c.id AND p.is_primary = 1 LIMIT 1),
               (SELECT COUNT(*) FROM jobs j WHERE j.company_id = c.id),
               (SELECT COUNT(*) FROM jobs j WHERE j.company_id = c.id
                AND j.grade IN ('SS', 'S', 'A'))
        FROM companies c
        {archive_filter}
        ORDER BY
            CASE c.grade
                WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3
                WHEN 'C' THEN 4 ELSE 5
            END,
            c.name");

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([], |row| {
        Ok(CompanyRow {
            id: row.get(0)?,
            name: row.get(1)?,
            website: row.get(2)?,
            what_they_do: row.get(3)?,
            status: row.get(4)?,
            location: row.get(5)?,
            sector_tags: row.get(6)?,
            grade: row.get(7)?,
            grade_reasoning: row.get(8)?,
            why_relevant: row.get(9)?,
            careers_url: row.get(10)?,
            ats_provider: row.get(11)?,
            ats_slug: row.get(12)?,
            job_count: row.get(13)?,
            fit_count: row.get(14)?,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

pub fn fetch_jobs(
    conn: &Connection,
    company_filter: Option<i64>,
    filters: &crate::tui::app::JobFilters,
    sort_mode: SortMode,
) -> Vec<JobRow> {
    // Build per-axis WHERE fragments. Empty axis = no filter on that axis.

    // ── Grade axis ──
    let grade_filter = if filters.grades.is_empty() {
        String::new()
    } else {
        let mut parts = Vec::new();
        let mut graded: Vec<String> = filters
            .grades
            .iter()
            .filter(|g| g.as_str() != "?")
            .map(|g| format!("'{}'", g.replace('\'', "''")))
            .collect();
        graded.sort();
        if !graded.is_empty() {
            parts.push(format!("j.grade IN ({})", graded.join(", ")));
        }
        if filters.grades.contains("?") {
            parts.push("j.grade IS NULL".to_string());
        }
        format!(" AND ({})", parts.join(" OR "))
    };

    // ── ATS axis ──
    let ats_filter = if filters.ats.is_empty() {
        String::new()
    } else {
        let mut parts = Vec::new();
        let mut providers: Vec<String> = filters
            .ats
            .iter()
            .filter(|a| a.as_str() != "bespoke")
            .map(|a| format!("'{}'", a.replace('\'', "''")))
            .collect();
        providers.sort();
        if !providers.is_empty() {
            parts.push(format!(
                "j.id IN (SELECT j2.id FROM jobs j2 JOIN company_portals cp \
                 ON cp.company_id = j2.company_id AND cp.is_primary = 1 \
                 WHERE cp.ats_provider IN ({}))",
                providers.join(", ")
            ));
        }
        if filters.ats.contains("bespoke") {
            parts.push("c.status = 'bespoke'".to_string());
        }
        format!(" AND ({})", parts.join(" OR "))
    };

    // ── Decision axis ──
    let decision_filter = if filters.decisions.is_empty() {
        String::new()
    } else {
        let mut parts = Vec::new();
        let mut decided: Vec<String> = filters
            .decisions
            .iter()
            .filter(|d| d.as_str() != "none")
            .map(|d| format!("'{}'", d.replace('\'', "''")))
            .collect();
        decided.sort();
        if !decided.is_empty() {
            parts.push(format!(
                "j.id IN (SELECT job_id FROM user_decisions WHERE decision IN ({}))",
                decided.join(", ")
            ));
        }
        if filters.decisions.contains("none") {
            parts.push(
                "j.id NOT IN (SELECT job_id FROM user_decisions)".to_string(),
            );
        }
        format!(" AND ({})", parts.join(" OR "))
    };

    // ── Package axis ──
    // The application_packages table was dropped in migration_008 as part of the
    // lane-based-relativity refactor (prepare-applications skill removed). The
    // axis is kept in the Filters struct for API stability but is now a no-op.
    let package_filter: String = {
        let _ = &filters.package;
        String::new()
    };

    // ── Archive axis ──
    let archive_filter = match (
        filters.archive.contains("archived"),
        filters.archive.contains("active"),
    ) {
        (true, false) => {
            " AND (j.evaluation_status = 'archived' OR c.status = 'archived')".to_string()
        }
        (false, true) => {
            " AND j.evaluation_status != 'archived' AND c.status != 'archived'".to_string()
        }
        _ => String::new(),
    };

    // ── Evidence axis ──
    // Empty set = no filter (show all). Otherwise show only rows whose
    // evidence_basis is in the set. NULL is treated as 'jd' for back-compat
    // with rows graded before evidence_basis existed.
    let evidence_filter = if filters.evidence.is_empty() {
        String::new()
    } else {
        let mut vals: Vec<&str> = filters.evidence.iter().map(|s| s.as_str()).collect();
        vals.sort();
        let quoted = vals
            .iter()
            .map(|v| format!("'{}'", v))
            .collect::<Vec<_>>()
            .join(", ");
        let mut clause = format!(" AND j.evidence_basis IN ({quoted})");
        if filters.evidence.contains("jd") {
            // Treat legacy NULL rows as 'jd' so pre-migration grades remain visible.
            clause = format!(" AND (j.evidence_basis IN ({quoted}) OR j.evidence_basis IS NULL)");
        }
        clause
    };

    let base = format!("
        SELECT j.id, j.company_id, c.name, j.title, j.url, j.location,
               j.remote_policy, j.posted_date, j.evaluation_status, j.fit_assessment,
               j.grade, j.evidence_basis, j.raw_description,
               (SELECT ud.decision FROM user_decisions ud
                WHERE ud.job_id = j.id ORDER BY ud.decided_at DESC LIMIT 1),
               0 AS has_package
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE 1=1{archive_filter}{grade_filter}{ats_filter}{decision_filter}{package_filter}{evidence_filter}");

    let order = match sort_mode {
        SortMode::ByGrade => "
            ORDER BY
                CASE j.grade
                    WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                    WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6
                    ELSE 7
                END,
                j.title",
        SortMode::ByCompany => "
            ORDER BY c.name, j.title",
        SortMode::ByDate => "
            ORDER BY j.posted_date DESC NULLS LAST, j.title",
        SortMode::ByLocation => "
            ORDER BY j.location, j.title",
    };

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<JobRow> {
        Ok(JobRow {
            id: row.get(0)?,
            company_id: row.get(1)?,
            company_name: row.get(2)?,
            title: row.get(3)?,
            url: row.get(4)?,
            location: row.get(5)?,
            remote_policy: row.get(6)?,
            posted_date: row.get(7)?,
            evaluation_status: row.get(8)?,
            fit_assessment: row.get(9)?,
            grade: row.get(10)?,
            evidence_basis: row.get(11)?,
            raw_description: row.get(12)?,
            decision: row.get(13)?,
            has_package: row.get::<_, bool>(14).unwrap_or(false),
        })
    };

    if let Some(id) = company_filter {
        let sql = format!("{base} AND j.company_id = ?1 {order}");
        conn.prepare(&sql)
            .and_then(|mut s| {
                s.query_map([id], map_row)
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default()
    } else {
        let sql = format!("{base} {order}");
        conn.prepare(&sql)
            .and_then(|mut s| {
                s.query_map([], map_row)
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default()
    }
}

#[allow(dead_code)]
pub fn fetch_total_job_count(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM jobs
         WHERE evaluation_status != 'archived'
         AND company_id IN (SELECT id FROM companies WHERE status != 'archived')",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

pub fn fetch_stats(conn: &Connection) -> DashboardStats {
    let total_companies: i64 = conn
        .query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))
        .unwrap_or(0);

    let companies_by_grade = query_groups(
        conn,
        "SELECT COALESCE(grade, '—'), COUNT(*) FROM companies
         GROUP BY grade
         ORDER BY CASE grade
             WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3
             WHEN 'C' THEN 4 ELSE 5
         END",
    );

    let companies_by_status = query_groups(
        conn,
        "SELECT status, COUNT(*) FROM companies GROUP BY status ORDER BY status",
    );

    let total_jobs: i64 = conn
        .query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))
        .unwrap_or(0);

    let jobs_by_eval = query_groups(
        conn,
        "SELECT evaluation_status, COUNT(*) FROM jobs
         GROUP BY evaluation_status
         ORDER BY CASE evaluation_status
             WHEN 'strong_fit' THEN 1 WHEN 'weak_fit' THEN 2
             WHEN 'evaluating' THEN 3 WHEN 'pending' THEN 4
             WHEN 'no_fit' THEN 5
         END",
    );

    let jobs_by_grade = query_groups(
        conn,
        "SELECT COALESCE(grade, '—'), COUNT(*) FROM jobs
         GROUP BY grade
         ORDER BY CASE grade
             WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
             WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6
             ELSE 7
         END",
    );

    let ats_coverage = query_groups(
        conn,
        "SELECT ats_provider, COUNT(DISTINCT company_id) FROM company_portals
         GROUP BY ats_provider
         ORDER BY COUNT(DISTINCT company_id) DESC",
    );

    let pending_companies: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'potential'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let bespoke_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'bespoke'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let archived_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE status = 'archived'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let applied_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = 'applied'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let watching_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = 'watching'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let rejected_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = 'rejected'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let bespoke_searchable: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies
             WHERE status = 'bespoke'
             AND grade IN ('S', 'A')
             AND (last_searched_at IS NULL
                  OR last_searched_at < datetime('now', '-7 days'))",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let needs_description: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs
             WHERE grade IN ('SS', 'S', 'A')
             AND evaluation_status != 'archived'
             AND (raw_description IS NULL OR LENGTH(raw_description) < 50)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let top_matches = fetch_top_matches(conn);

    DashboardStats {
        total_companies,
        companies_by_grade,
        companies_by_status,
        total_jobs,
        jobs_by_eval,
        jobs_by_grade,
        ats_coverage,
        top_matches,
        pending_companies,
        bespoke_count,
        archived_count,
        applied_count,
        watching_count,
        rejected_count,
        bespoke_searchable,
        needs_description,
    }
}

fn query_groups(conn: &Connection, sql: &str) -> Vec<(String, i64)> {
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Fetch pipeline cards grouped by decision (watching, applied, interview).
pub fn fetch_pipeline_cards(
    conn: &Connection,
) -> (Vec<PipelineCard>, Vec<PipelineCard>, Vec<PipelineCard>) {
    let fetch = |decision: &str| -> Vec<PipelineCard> {
        let sql = "
            SELECT j.id, j.title, c.name, j.grade
            FROM jobs j
            JOIN companies c ON c.id = j.company_id
            JOIN (
                SELECT job_id, decision
                FROM user_decisions
                WHERE id IN (
                    SELECT MAX(id) FROM user_decisions GROUP BY job_id
                )
            ) ud ON ud.job_id = j.id
            WHERE ud.decision = ?1
            AND j.evaluation_status != 'archived'
            ORDER BY
                CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                    WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6 ELSE 7 END,
                j.title";

        conn.prepare(sql)
            .and_then(|mut stmt| {
                stmt.query_map([decision], |row| {
                    Ok(PipelineCard {
                        job_id: row.get(0)?,
                        title: row.get(1)?,
                        company: row.get(2)?,
                        grade: row.get(3)?,
                    })
                })
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default()
    };

    (fetch("watching"), fetch("applied"), fetch("interview"))
}

/// Fetch activity data for the contribution heatmap (last 84 days).
///
/// Returns `(date_string, action_type)` pairs aggregated across four sources:
/// user decisions, company searches, company grading, and job discovery.
pub fn fetch_activity_data(conn: &Connection) -> Vec<(String, String)> {
    let sql = "
        SELECT DATE(decided_at) AS d, decision FROM user_decisions
        WHERE datetime(decided_at) >= datetime('now', '-84 days')
        UNION ALL
        SELECT DATE(last_searched_at), 'searched' FROM companies
        WHERE last_searched_at IS NOT NULL AND datetime(last_searched_at) >= datetime('now', '-84 days')
        UNION ALL
        SELECT DATE(graded_at), 'graded' FROM companies
        WHERE graded_at IS NOT NULL AND datetime(graded_at) >= datetime('now', '-84 days')
        UNION ALL
        SELECT DATE(discovered_at), 'discovered' FROM jobs
        WHERE discovered_at IS NOT NULL AND datetime(discovered_at) >= datetime('now', '-84 days')
    ";

    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Fetch the most recent search timestamp.
pub fn fetch_last_search_at(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT MAX(last_searched_at) FROM companies",
        [],
        |r| r.get(0),
    )
    .unwrap_or(None)
}

/// Fetch the most recent grading timestamp.
pub fn fetch_last_graded_at(conn: &Connection) -> Option<String> {
    conn.query_row(
        "SELECT MAX(graded_at) FROM companies",
        [],
        |r| r.get(0),
    )
    .unwrap_or(None)
}

/// Fetch top companies by SS+S+A job count.
pub fn fetch_top_companies_by_hits(conn: &Connection) -> Vec<(String, i64)> {
    let sql = "
        SELECT c.name, COUNT(*) AS cnt
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.grade IN ('SS', 'S', 'A')
        AND j.evaluation_status != 'archived'
        AND c.status != 'archived'
        GROUP BY c.id
        ORDER BY cnt DESC
        LIMIT 5
    ";

    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Fetch a rich activity timeline for the Activity tab.
///
/// Returns individual decision entries (with job title + company) and
/// aggregated search/grading/discovery events, ordered by date descending.
pub fn fetch_activity_timeline(conn: &Connection) -> Vec<ActivityEntry> {
    let sql = "
        SELECT d AS date, action, detail FROM (
            -- Individual user decisions with job + company detail
            SELECT DATE(ud.decided_at) AS d, ud.decision AS action,
                   j.title || ' — ' || c.name AS detail,
                   ud.decided_at AS sort_ts,
                   1 AS priority
            FROM user_decisions ud
            JOIN jobs j ON j.id = ud.job_id
            JOIN companies c ON c.id = j.company_id

            UNION ALL

            -- Aggregated search events per day
            SELECT DATE(last_searched_at) AS d, 'searched' AS action,
                   CAST(COUNT(*) AS TEXT) || ' companies searched' AS detail,
                   MAX(last_searched_at) AS sort_ts,
                   3 AS priority
            FROM companies
            WHERE last_searched_at IS NOT NULL
            GROUP BY DATE(last_searched_at)

            UNION ALL

            -- Aggregated grading events per day
            SELECT DATE(graded_at) AS d, 'graded' AS action,
                   CAST(COUNT(*) AS TEXT) || ' companies graded' AS detail,
                   MAX(graded_at) AS sort_ts,
                   4 AS priority
            FROM companies
            WHERE graded_at IS NOT NULL
            GROUP BY DATE(graded_at)

            UNION ALL

            -- Aggregated discovery events per day
            SELECT DATE(discovered_at) AS d, 'discovered' AS action,
                   CAST(COUNT(*) AS TEXT) || ' jobs discovered' AS detail,
                   MAX(discovered_at) AS sort_ts,
                   5 AS priority
            FROM jobs
            WHERE discovered_at IS NOT NULL
            GROUP BY DATE(discovered_at)
        )
        ORDER BY date DESC, priority ASC
        LIMIT 200
    ";

    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |row| {
                Ok(ActivityEntry {
                    date: row.get(0)?,
                    action: row.get(1)?,
                    detail: row.get(2)?,
                })
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Fetch counts of new jobs, companies, and decisions since last session (~12 hours ago).
pub fn fetch_new_since_session(conn: &Connection) -> (i64, i64, i64) {
    let jobs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE datetime(discovered_at) >= datetime('now', '-12 hours')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let companies: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM companies WHERE datetime(discovered_at) >= datetime('now', '-12 hours')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let decisions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_decisions WHERE decided_at >= datetime('now', '-12 hours')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    (jobs, companies, decisions)
}

fn fetch_top_matches(conn: &Connection) -> Vec<TopMatch> {
    conn.prepare(
        "SELECT j.title, c.name, j.grade, COALESCE(j.location, c.location)
         FROM jobs j
         JOIN companies c ON c.id = j.company_id
         WHERE j.grade IN ('SS', 'S', 'A')
         AND j.evaluation_status != 'archived'
         ORDER BY
             CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 END,
             j.title",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(TopMatch {
                title: row.get(0)?,
                company: row.get(1)?,
                grade: row.get(2)?,
                location: row.get(3)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}
