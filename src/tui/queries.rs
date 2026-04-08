use rusqlite::Connection;

use super::app::{CompanyRow, DashboardStats, JobRow, TopMatch};

pub fn fetch_companies(conn: &Connection) -> Vec<CompanyRow> {
    let sql = "
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
        WHERE c.status != 'archived'
        ORDER BY
            CASE c.grade
                WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3
                WHEN 'C' THEN 4 ELSE 5
            END,
            c.name";

    let mut stmt = match conn.prepare(sql) {
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

pub fn fetch_jobs(conn: &Connection, company_filter: Option<i64>) -> Vec<JobRow> {
    let base = "
        SELECT j.id, j.company_id, c.name, j.title, j.url, j.location,
               j.remote_policy, j.posted_date, j.evaluation_status, j.fit_assessment,
               j.fit_score, j.grade, j.raw_description,
               (SELECT ud.decision FROM user_decisions ud
                WHERE ud.job_id = j.id ORDER BY ud.decided_at DESC LIMIT 1)
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.evaluation_status != 'archived'
        AND c.status != 'archived'";

    let order = "
        ORDER BY
            CASE j.grade
                WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6
                ELSE 7
            END,
            j.title";

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
            fit_score: row.get(10)?,
            grade: row.get(11)?,
            raw_description: row.get(12)?,
            decision: row.get(13)?,
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

fn fetch_top_matches(conn: &Connection) -> Vec<TopMatch> {
    conn.prepare(
        "SELECT j.title, c.name, j.grade
         FROM jobs j
         JOIN companies c ON c.id = j.company_id
         WHERE j.grade IN ('SS', 'S', 'A')
         ORDER BY
             CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 END,
             j.title
         LIMIT 10",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok(TopMatch {
                title: row.get(0)?,
                company: row.get(1)?,
                grade: row.get(2)?,
            })
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}
