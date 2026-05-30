//! Analytic aggregates for the web dashboard panes.
//!
//! Every function returns plain (label, count) shapes or small structs that
//! the web handlers serialise into JSON for ECharts. Keeping these here means
//! a future TUI analytics view can call them too.

use rusqlite::Connection;

use super::lane::LANE_KEYS;

/// Build a date axis covering the last N days, oldest → newest, inclusive
/// of today. Charts iterate over this and look up each day's value.
fn date_axis_n(n: i64) -> Vec<String> {
    let today = chrono::Local::now().date_naive();
    let start = today - chrono::Duration::days(n - 1);
    (0..n)
        .map(|i| (start + chrono::Duration::days(i)).format("%Y-%m-%d").to_string())
        .collect()
}

/// Build the canonical 30-day date axis (oldest → newest, inclusive of today).
/// Charts iterate over this and look up each day's value, defaulting to 0,
/// so sparse data renders as flat-zero with isolated peaks instead of
/// interpolated ramps across empty days.
fn date_axis_30d() -> Vec<String> {
    let today = chrono::Local::now().date_naive();
    let start = today - chrono::Duration::days(29);
    (0..30)
        .map(|i| (start + chrono::Duration::days(i)).format("%Y-%m-%d").to_string())
        .collect()
}

const JOB_GRADES: [&str; 6] = ["SS", "S", "A", "B", "C", "F"];
const COMPANY_GRADES: [&str; 4] = ["S", "A", "B", "C"];

/// Jobs × lane × grade with archived stacked alongside active.
/// Returns Vec<(lane_key, active_per_grade[6], archived_per_grade[6])>.
pub fn jobs_lane_grade_split(conn: &Connection) -> Vec<(String, [i64; 6], [i64; 6])> {
    let mut out = Vec::with_capacity(LANE_KEYS.len());
    for key in LANE_KEYS.iter() {
        let prefix = format!("[\"{key}\"%");
        let mut active = [0i64; 6];
        let mut archived = [0i64; 6];
        for (i, g) in JOB_GRADES.iter().enumerate() {
            active[i] = conn.query_row(
                "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND grade = ?2 AND evaluation_status != 'archived'",
                rusqlite::params![prefix, g], |r| r.get(0)).unwrap_or(0);
            archived[i] = conn.query_row(
                "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND grade = ?2 AND evaluation_status = 'archived'",
                rusqlite::params![prefix, g], |r| r.get(0)).unwrap_or(0);
        }
        out.push((key.to_string(), active, archived));
    }
    out
}

/// Companies per lane (active only). Multi-lane companies counted once per lane.
pub fn companies_per_lane(conn: &Connection) -> Vec<(String, i64)> {
    let mut out = Vec::with_capacity(LANE_KEYS.len());
    for key in LANE_KEYS.iter() {
        let prefix = format!("%\"{key}\"%");
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM companies WHERE lanes LIKE ?1 AND status != 'archived'",
            rusqlite::params![prefix], |r| r.get(0)).unwrap_or(0);
        out.push((key.to_string(), n));
    }
    out
}

/// Job-board health: every ATS provider + bespoke + potential + unresolved.
/// `unresolved` = status='resolved' but no portal row (shouldn't happen but flagged).
pub fn ats_health(conn: &Connection) -> Vec<(String, i64)> {
    let providers = ["greenhouse", "ashby", "lever", "workable", "smartrecruiters", "workday", "eightfold"];
    let mut out = Vec::new();
    for p in &providers {
        let n: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT company_id) FROM company_portals WHERE ats_provider = ?1",
            rusqlite::params![p], |r| r.get(0)).unwrap_or(0);
        out.push((p.to_string(), n));
    }
    let bespoke: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE status = 'bespoke'",
        [], |r| r.get(0)).unwrap_or(0);
    out.push(("bespoke".into(), bespoke));
    let potential: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE status = 'potential'",
        [], |r| r.get(0)).unwrap_or(0);
    out.push(("potential".into(), potential));
    out
}

/// Company-grade distribution (S/A/B/C + ungraded).
pub fn company_grade_distribution(conn: &Connection) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for g in &COMPANY_GRADES {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM companies WHERE grade = ?1 AND status != 'archived'",
            rusqlite::params![g], |r| r.get(0)).unwrap_or(0);
        out.push((g.to_string(), n));
    }
    let ungraded: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE grade IS NULL AND status != 'archived'",
        [], |r| r.get(0)).unwrap_or(0);
    out.push(("—".into(), ungraded));
    out
}

/// Geography buckets — London / London-adjacent / UK ex-London / Remote / International / Unknown.
pub fn geography_buckets(conn: &Connection) -> Vec<(String, i64)> {
    let sql = "
        SELECT
            CASE
                WHEN location IS NULL OR location = '' THEN 'Unknown'
                WHEN location LIKE '%Remote%' THEN 'Remote'
                WHEN location LIKE '%London%' AND (location LIKE '%New York%' OR location LIKE '%Singapore%' OR location LIKE '%global%' OR location LIKE '%Global%') THEN 'London + intl'
                WHEN location LIKE '%London%' THEN 'London'
                WHEN location LIKE '%Cambridge%' OR location LIKE '%Oxford%' OR location LIKE '%Bristol%' OR location LIKE '%Manchester%' OR location LIKE '%Edinburgh%' OR location LIKE '%Southampton%' OR location LIKE '%Birmingham%' OR location LIKE '%Leeds%' THEN 'UK ex-London'
                WHEN location LIKE '%UK%' THEN 'UK other'
                ELSE 'International'
            END AS bucket,
            COUNT(*) AS n
        FROM companies
        WHERE status != 'archived'
        GROUP BY bucket
        ORDER BY n DESC";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// UK sponsorship split.
pub fn sponsorship_split(conn: &Connection) -> Vec<(String, i64)> {
    let sql = "
        SELECT COALESCE(sponsors_uk, 'unknown') AS bucket, COUNT(*) AS n
        FROM companies WHERE status != 'archived'
        GROUP BY bucket
        ORDER BY n DESC";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Grade-freshness buckets for companies: 0-7d / 7-30d / 30-90d / 90d+ / ungraded.
pub fn grade_freshness_buckets(conn: &Connection) -> Vec<(String, i64)> {
    let sql = "
        SELECT bucket, COUNT(*) FROM (
            SELECT CASE
                WHEN graded_at IS NULL THEN 'ungraded'
                WHEN datetime(graded_at) >= datetime('now', '-7 days') THEN '0-7d'
                WHEN datetime(graded_at) >= datetime('now', '-30 days') THEN '7-30d'
                WHEN datetime(graded_at) >= datetime('now', '-90 days') THEN '30-90d'
                ELSE '90d+'
            END AS bucket
            FROM companies WHERE status != 'archived'
        ) GROUP BY bucket";
    let raw: Vec<(String, i64)> = conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    // Sort canonically.
    let order = ["0-7d", "7-30d", "30-90d", "90d+", "ungraded"];
    let mut sorted = Vec::with_capacity(5);
    for b in &order {
        let n = raw.iter().find(|(k, _)| k == b).map(|(_, n)| *n).unwrap_or(0);
        sorted.push((b.to_string(), n));
    }
    sorted
}

/// Top N companies by active job count.
pub fn top_companies_by_jobs(conn: &Connection, n: i64) -> Vec<(String, i64, i64)> {
    let sql = "
        SELECT c.name,
               COUNT(*) AS total,
               SUM(CASE WHEN j.grade IN ('SS','S','A') THEN 1 ELSE 0 END) AS strong
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.evaluation_status != 'archived' AND c.status != 'archived'
        GROUP BY c.id
        ORDER BY total DESC
        LIMIT ?1";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([n], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2).unwrap_or(0))))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Recently-discovered companies per day, last 30 days. Padded so missing
/// days render as zero rather than getting interpolated by ECharts.
pub fn companies_discovered_per_day(conn: &Connection) -> Vec<(String, i64)> {
    let sql = "
        SELECT DATE(discovered_at) AS d, COUNT(*) FROM companies
        WHERE datetime(discovered_at) >= datetime('now', '-30 days')
        GROUP BY d ORDER BY d ASC";
    let raw: Vec<(String, i64)> = conn
        .prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    let lookup: std::collections::HashMap<&str, i64> =
        raw.iter().map(|(d, n)| (d.as_str(), *n)).collect();
    date_axis_30d()
        .into_iter()
        .map(|d| {
            let n = lookup.get(d.as_str()).copied().unwrap_or(0);
            (d, n)
        })
        .collect()
}

/// Pipeline funnel — every stage is a COMPANY count so the funnel is
/// strictly monotone-decreasing. Mixing company-counts with job-counts
/// (e.g. "Jobs graded = 331") creates a diamond instead of a funnel because
/// 1 company can have many graded jobs.
pub fn pipeline_funnel(conn: &Connection) -> Vec<(String, i64)> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE status != 'archived'", [], |r| r.get(0)).unwrap_or(0);
    let resolved: i64 = conn.query_row(
        "SELECT COUNT(*) FROM companies WHERE status IN ('resolved', 'bespoke')", [], |r| r.get(0)).unwrap_or(0);
    let with_jobs: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT company_id) FROM jobs WHERE evaluation_status != 'archived'", [], |r| r.get(0)).unwrap_or(0);
    let with_graded: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT company_id) FROM jobs
         WHERE grade IS NOT NULL AND evaluation_status != 'archived'",
        [], |r| r.get(0)).unwrap_or(0);
    let with_decisions: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT j.company_id) FROM user_decisions ud
         JOIN jobs j ON j.id = ud.job_id", [], |r| r.get(0)).unwrap_or(0);
    vec![
        ("Companies".into(), total),
        ("Resolved + bespoke".into(), resolved),
        ("With active jobs".into(), with_jobs),
        ("With graded jobs".into(), with_graded),
        ("With decisions".into(), with_decisions),
    ]
}

/// Decision velocity per day, last 30 days. Returns (date, applied, watching, rejected).
/// Pads every day in the 30-day window so the chart renders a continuous axis;
/// days with no decisions are (0, 0, 0) rather than missing.
pub fn decision_velocity_30d(conn: &Connection) -> Vec<(String, i64, i64, i64)> {
    let sql = "
        SELECT DATE(decided_at) AS d,
               SUM(CASE WHEN decision = 'applied' THEN 1 ELSE 0 END),
               SUM(CASE WHEN decision = 'watching' THEN 1 ELSE 0 END),
               SUM(CASE WHEN decision = 'rejected' THEN 1 ELSE 0 END)
        FROM user_decisions
        WHERE datetime(decided_at) >= datetime('now', '-30 days')
        GROUP BY d ORDER BY d ASC";
    let raw: Vec<(String, i64, i64, i64)> = conn
        .prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1).unwrap_or(0),
                    r.get::<_, i64>(2).unwrap_or(0),
                    r.get::<_, i64>(3).unwrap_or(0),
                ))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    let lookup: std::collections::HashMap<&str, &(String, i64, i64, i64)> =
        raw.iter().map(|t| (t.0.as_str(), t)).collect();
    date_axis_30d()
        .into_iter()
        .map(|d| match lookup.get(d.as_str()) {
            Some(t) => (d, t.1, t.2, t.3),
            None => (d, 0, 0, 0),
        })
        .collect()
}

/// Events per day per source, windowed to the last N days.
/// Returns (date, cli, tui, web, script, trigger, other).
pub fn events_by_source_window(conn: &Connection, days: i64) -> Vec<(String, [i64; 6])> {
    // Exclude raw.* events too — they're SQL-trigger fallout from the
    // migration backfill (the trigger itself records source='trigger', not
    // 'backfill', so the source-only filter doesn't catch them).
    let sql = format!(
        "SELECT DATE(occurred_at) AS d, source, COUNT(*)
         FROM activity_events
         WHERE datetime(occurred_at) >= datetime('now', '-{days} days')
           AND event_type NOT LIKE 'raw.%'
           AND source NOT LIKE '%backfill%'
           AND source NOT LIKE '%migration%'
         GROUP BY d, source ORDER BY d ASC"
    );
    let raw: Vec<(String, String, i64)> = conn
        .prepare(&sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    let mut map: std::collections::BTreeMap<String, [i64; 6]> = std::collections::BTreeMap::new();
    for (d, source, n) in raw {
        let entry = map.entry(d).or_insert([0; 6]);
        let idx = match source.as_str() {
            s if s.starts_with("cli") => 0,
            "tui" => 1,
            "web" => 2,
            s if s.contains("script") => 3,
            "trigger" => 4,
            _ => 5,
        };
        entry[idx] += n;
    }
    date_axis_n(days)
        .into_iter()
        .map(|d| {
            let row = map.get(&d).copied().unwrap_or([0; 6]);
            (d, row)
        })
        .collect()
}

/// Back-compat for existing callers — defaults to 30 days.
pub fn events_by_source_30d(conn: &Connection) -> Vec<(String, [i64; 6])> {
    events_by_source_window(conn, 30)
}

/// Activity KPI snapshot for the Activity page header.
/// Returns (events_24h, events_7d, decisions_30d, most_active_source).
pub fn activity_kpis(conn: &Connection) -> (i64, i64, i64, String) {
    let events_24h: i64 = conn.query_row(
        "SELECT COUNT(*) FROM activity_events WHERE datetime(occurred_at) >= datetime('now', '-1 day')",
        [], |r| r.get(0)).unwrap_or(0);
    let events_7d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM activity_events WHERE datetime(occurred_at) >= datetime('now', '-7 days')",
        [], |r| r.get(0)).unwrap_or(0);
    let decisions_30d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM user_decisions WHERE datetime(decided_at) >= datetime('now', '-30 days')",
        [], |r| r.get(0)).unwrap_or(0);
    let most_active_source: String = conn.query_row(
        "SELECT source FROM activity_events
           WHERE datetime(occurred_at) >= datetime('now', '-30 days')
             AND event_type NOT LIKE 'raw.%'
             AND source NOT LIKE '%backfill%'
             AND source NOT LIKE '%migration%'
           GROUP BY source ORDER BY COUNT(*) DESC LIMIT 1",
        [], |r| r.get(0)).unwrap_or_else(|_| "—".to_string());
    (events_24h, events_7d, decisions_30d, most_active_source)
}

/// Event-type breakdown windowed to N days. Excludes raw.* triggers and
/// one-shot migration/backfill scripts.
pub fn event_type_breakdown_window(conn: &Connection, days: i64) -> Vec<(String, i64)> {
    let sql = format!(
        "SELECT event_type, COUNT(*) FROM activity_events
         WHERE datetime(occurred_at) >= datetime('now', '-{days} days')
           AND event_type NOT LIKE 'raw.%'
           AND source NOT LIKE '%backfill%'
           AND source NOT LIKE '%migration%'
         GROUP BY event_type ORDER BY COUNT(*) DESC LIMIT 12"
    );
    conn.prepare(&sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Back-compat for existing callers — defaults to 30 days.
pub fn event_type_breakdown_30d(conn: &Connection) -> Vec<(String, i64)> {
    event_type_breakdown_window(conn, 30)
}

/// Decision funnel: untouched / watching / applied / rejected / interview.
pub fn decision_funnel(conn: &Connection) -> Vec<(String, i64)> {
    let active_jobs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM jobs WHERE evaluation_status != 'archived'", [], |r| r.get(0)).unwrap_or(0);
    let with_decision: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT job_id) FROM user_decisions", [], |r| r.get(0)).unwrap_or(0);
    let untouched = active_jobs - with_decision;
    let mut out = vec![("untouched".to_string(), untouched.max(0))];
    for d in &["watching", "applied", "interview", "rejected"] {
        let n: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = ?1",
            rusqlite::params![d], |r| r.get(0)).unwrap_or(0);
        out.push((d.to_string(), n));
    }
    out
}

/// Grade × lane heatmap (jobs). Returns Vec<(grade, [count per lane; 8])>.
pub fn job_grade_lane_heatmap(conn: &Connection, include_archived: bool) -> Vec<(String, Vec<i64>)> {
    let archive_clause = if include_archived { "" } else { " AND evaluation_status != 'archived'" };
    let mut out = Vec::new();
    for g in &JOB_GRADES {
        let mut row = Vec::with_capacity(LANE_KEYS.len());
        for lk in LANE_KEYS.iter() {
            let prefix = format!("[\"{lk}\"%");
            let sql = format!(
                "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND grade = ?2{archive_clause}");
            let n: i64 = conn.query_row(&sql, rusqlite::params![prefix, g], |r| r.get(0)).unwrap_or(0);
            row.push(n);
        }
        out.push((g.to_string(), row));
    }
    out
}

/// Freshness histogram by posted_date (active jobs only).
pub fn job_freshness_histogram(conn: &Connection) -> Vec<(String, i64)> {
    let sql = "
        SELECT bucket, COUNT(*) FROM (
            SELECT CASE
                WHEN posted_date IS NULL THEN 'unknown'
                WHEN datetime(posted_date) >= datetime('now', '-7 days') THEN '0-7d'
                WHEN datetime(posted_date) >= datetime('now', '-30 days') THEN '7-30d'
                WHEN datetime(posted_date) >= datetime('now', '-90 days') THEN '30-90d'
                ELSE '90d+'
            END AS bucket
            FROM jobs WHERE evaluation_status != 'archived'
        ) GROUP BY bucket";
    let raw: Vec<(String, i64)> = conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    let order = ["0-7d", "7-30d", "30-90d", "90d+", "unknown"];
    order.iter()
        .map(|b| {
            let n = raw.iter().find(|(k, _)| k == b).map(|(_, n)| *n).unwrap_or(0);
            (b.to_string(), n)
        })
        .collect()
}

/// Top job titles by frequency (normalised loosely — exact match).
pub fn top_job_titles(conn: &Connection, n: i64) -> Vec<(String, i64)> {
    let sql = "
        SELECT title, COUNT(*) FROM jobs
        WHERE evaluation_status != 'archived'
        GROUP BY title
        ORDER BY COUNT(*) DESC
        LIMIT ?1";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([n], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Action queue — actionable counts for the dashboard.
/// Returns a list of (label, count, filter_url) — each is a thing the user
/// could do something about right now. Clicking the URL pre-filters the
/// relevant page to the corresponding subset.
pub fn action_queue(conn: &Connection) -> Vec<(String, i64, String)> {
    let pending_grade_companies = conn.query_row(
        "SELECT COUNT(*) FROM companies
         WHERE grade IS NULL AND status NOT IN ('archived', 'potential')",
        [], |r| r.get(0)).unwrap_or(0);

    let needs_search = conn.query_row(
        "SELECT COUNT(*) FROM companies c
         WHERE c.status = 'resolved'
         AND c.grade IN ('S','A')
         AND NOT EXISTS (
             SELECT 1 FROM jobs j
             WHERE j.company_id = c.id
             AND datetime(j.discovered_at) > datetime('now', '-14 days')
         )",
        [], |r| r.get(0)).unwrap_or(0);

    let pending_grade_jobs = conn.query_row(
        "SELECT COUNT(*) FROM jobs WHERE grade IS NULL AND evaluation_status != 'archived'",
        [], |r| r.get(0)).unwrap_or(0);

    let needs_decision = conn.query_row(
        "SELECT COUNT(*) FROM jobs j
         JOIN companies c ON c.id = j.company_id
         WHERE j.grade IN ('SS','S')
         AND j.evaluation_status != 'archived'
         AND c.status != 'archived'
         AND j.id NOT IN (SELECT job_id FROM user_decisions)",
        [], |r| r.get(0)).unwrap_or(0);

    let watching_to_followup = conn.query_row(
        "SELECT COUNT(DISTINCT job_id) FROM user_decisions
         WHERE decision = 'watching'
         AND datetime(decided_at) < datetime('now', '-7 days')",
        [], |r| r.get(0)).unwrap_or(0);

    vec![
        ("Companies need grading".to_string(), pending_grade_companies,
         "/companies?grade=?".to_string()),
        ("Companies need search".to_string(), needs_search,
         "/companies?grade=S%2CA&status=resolved&has_jobs=no".to_string()),
        ("Jobs need grading".to_string(), pending_grade_jobs,
         "/jobs?grade=%3F".to_string()),
        ("SS/S need decision".to_string(), needs_decision,
         "/jobs?grade=SS%2CS&decision=none".to_string()),
        ("Watching > 7d, no follow-up".to_string(), watching_to_followup,
         "/jobs?decision=watching".to_string()),
    ]
}

/// All decisions, optionally filtered by kind ("applied" | "watching" |
/// "interview" | "rejected" | "all"). Returns most-recent first.
/// Returns (decided_at, decision, title, company, lanes_json, grade, url, job_id).
pub fn decisions_filtered(conn: &Connection, kind: &str)
    -> Vec<(String, String, String, String, Option<String>, Option<String>, String, i64)>
{
    let where_clause = match kind {
        "all" | "" => String::new(),
        k => format!(" WHERE ud.decision = '{}'", k.replace('\'', "''")),
    };
    let sql = format!(
        "SELECT ud.decided_at, ud.decision, j.title, c.name, j.lanes, j.grade, j.url, j.id
         FROM user_decisions ud
         JOIN jobs j ON j.id = ud.job_id
         JOIN companies c ON c.id = j.company_id
         {where_clause}
         ORDER BY ud.decided_at DESC");
    conn.prepare(&sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| {
                Ok((
                    r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
                    r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?,
                ))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Decision counts grouped by kind.
pub fn decision_counts(conn: &Connection) -> std::collections::HashMap<String, i64> {
    let sql = "SELECT decision, COUNT(DISTINCT job_id) FROM user_decisions GROUP BY decision";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Recent user decisions (last N), most recent first.
/// Returns (decided_at, decision, title, company, lanes_json, grade, url, job_id).
pub fn recent_decisions(conn: &Connection, n: i64)
    -> Vec<(String, String, String, String, Option<String>, Option<String>, String, i64)>
{
    let sql = "
        SELECT ud.decided_at, ud.decision, j.title, c.name, j.lanes, j.grade, j.url, j.id
        FROM user_decisions ud
        JOIN jobs j ON j.id = ud.job_id
        JOIN companies c ON c.id = j.company_id
        ORDER BY ud.decided_at DESC
        LIMIT ?1";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([n], |r| {
                Ok((
                    r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
                    r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?,
                ))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Recent SS/S ungraded-decision jobs (top of funnel — what to act on next).
/// Returns (id, grade, title, lanes_json, location, company_name, url).
pub fn top_actionable_jobs(
    conn: &Connection,
    n: i64,
) -> Vec<(i64, String, String, Option<String>, Option<String>, String, String)> {
    let sql = "
        SELECT j.id, j.grade, j.title, j.lanes, j.location, c.name, j.url
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.grade IN ('SS', 'S')
          AND j.evaluation_status != 'archived'
          AND c.status != 'archived'
          AND j.id NOT IN (SELECT job_id FROM user_decisions)
        ORDER BY CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 END, j.discovered_at DESC
        LIMIT ?1";
    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([n], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}
