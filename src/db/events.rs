//! Activity-event emitter helpers.
//!
//! Every DB mutation that should appear on the Activity timeline goes through
//! one of these helpers. The helpers cache subject_label / lane / grade at
//! emit time so events survive the underlying row's deletion (the whole point
//! of the append-only log).
//!
//! Trigger backstops in `schema.rs` catch any path that bypasses these helpers
//! and emit `raw.*` events; the dashboard / Activity view surface those raw
//! events under a distinct icon so missing-helper coverage is visible.

use rusqlite::{params, Connection};

/// Source tag — who emitted the event.
///
/// Use `tui` for TUI actions, `cli:<cmd>` for cernio subcommands, `skill:<name>`
/// for skill runs (the skill writes the event directly via its own SQL). The
/// log treats every source the same; the tag is for filtering.
pub fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Extract the primary lane from a JSON array string ("[\"hft\",\"ai-ml\"]" → "hft").
/// Returns None when input is None or unparseable.
pub fn primary_lane(lanes_json: Option<&str>) -> Option<String> {
    let raw = lanes_json?;
    let cleaned = raw.replace(['[', ']', '"'], "");
    let first = cleaned.split(',').next()?.trim();
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

/// Look up a job's cached label/lane/grade by id.
fn lookup_job(conn: &Connection, job_id: i64) -> Option<(String, Option<String>, Option<String>)> {
    conn.query_row(
        "SELECT j.title || ' — ' || c.name, j.lanes, j.grade
         FROM jobs j JOIN companies c ON c.id = j.company_id
         WHERE j.id = ?1",
        params![job_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        },
    )
    .ok()
    .map(|(label, lanes, grade)| (label, primary_lane(lanes.as_deref()), grade))
}

/// Look up a company's cached label/lane/grade by id.
fn lookup_company(
    conn: &Connection,
    company_id: i64,
) -> Option<(String, Option<String>, Option<String>)> {
    conn.query_row(
        "SELECT name, lanes, grade FROM companies WHERE id = ?1",
        params![company_id],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        },
    )
    .ok()
    .map(|(label, lanes, grade)| (label, primary_lane(lanes.as_deref()), grade))
}

/// Low-level insert. Most callers should prefer the typed helpers below.
#[allow(clippy::too_many_arguments)]
pub fn emit(
    conn: &Connection,
    event_type: &str,
    subject_type: &str,
    subject_id: Option<i64>,
    subject_label: Option<&str>,
    lane: Option<&str>,
    grade: Option<&str>,
    detail_json: Option<&str>,
    source: &str,
) {
    let _ = conn.execute(
        "INSERT INTO activity_events
            (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            now(),
            event_type,
            subject_type,
            subject_id,
            subject_label,
            lane,
            grade,
            detail_json,
            source,
        ],
    );
}

// ── Job events ────────────────────────────────────────────────────

pub fn job_deleted(conn: &Connection, job_id: i64, source: &str) {
    if let Some((label, lane, grade)) = lookup_job(conn, job_id) {
        emit(
            conn,
            "job.deleted",
            "job",
            Some(job_id),
            Some(&label),
            lane.as_deref(),
            grade.as_deref(),
            None,
            source,
        );
    }
}

pub fn job_pruned(conn: &Connection, job_id: i64, reason: &str, source: &str) {
    if let Some((label, lane, grade)) = lookup_job(conn, job_id) {
        let detail = serde_json::json!({ "reason": reason }).to_string();
        emit(
            conn,
            "job.pruned",
            "job",
            Some(job_id),
            Some(&label),
            lane.as_deref(),
            grade.as_deref(),
            Some(&detail),
            source,
        );
    }
}

pub fn job_archived(conn: &Connection, job_id: i64, source: &str) {
    if let Some((label, lane, grade)) = lookup_job(conn, job_id) {
        emit(
            conn,
            "job.archived",
            "job",
            Some(job_id),
            Some(&label),
            lane.as_deref(),
            grade.as_deref(),
            None,
            source,
        );
    }
}

pub fn job_grade_changed(
    conn: &Connection,
    job_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    source: &str,
) {
    if let Some((label, lane, _grade)) = lookup_job(conn, job_id) {
        let detail = serde_json::json!({ "from": from, "to": to }).to_string();
        emit(
            conn,
            "job.regraded",
            "job",
            Some(job_id),
            Some(&label),
            lane.as_deref(),
            to,
            Some(&detail),
            source,
        );
    }
}

// ── Decision events ───────────────────────────────────────────────

pub fn decision_recorded(conn: &Connection, job_id: i64, decision: &str, source: &str) {
    if let Some((label, lane, grade)) = lookup_job(conn, job_id) {
        let event_type = format!("decision.{decision}");
        emit(
            conn,
            &event_type,
            "job",
            Some(job_id),
            Some(&label),
            lane.as_deref(),
            grade.as_deref(),
            None,
            source,
        );
    }
}

// ── Company events ────────────────────────────────────────────────

pub fn company_archived(conn: &Connection, company_id: i64, source: &str) {
    if let Some((label, lane, grade)) = lookup_company(conn, company_id) {
        emit(
            conn,
            "company.archived",
            "company",
            Some(company_id),
            Some(&label),
            lane.as_deref(),
            grade.as_deref(),
            None,
            source,
        );
    }
}

pub fn company_unarchived(conn: &Connection, company_id: i64, source: &str) {
    if let Some((label, lane, grade)) = lookup_company(conn, company_id) {
        emit(
            conn,
            "company.unarchived",
            "company",
            Some(company_id),
            Some(&label),
            lane.as_deref(),
            grade.as_deref(),
            None,
            source,
        );
    }
}

// ── Batch / system events ─────────────────────────────────────────

pub fn batch(conn: &Connection, event_type: &str, count: i64, detail: serde_json::Value, source: &str) {
    let mut payload = detail;
    if let serde_json::Value::Object(ref mut map) = payload {
        map.insert("count".to_string(), serde_json::Value::from(count));
    }
    let s = payload.to_string();
    emit(conn, event_type, "batch", None, None, None, None, Some(&s), source);
}

pub fn system(conn: &Connection, event_type: &str, detail: serde_json::Value, source: &str) {
    let s = detail.to_string();
    emit(conn, event_type, "system", None, None, None, None, Some(&s), source);
}
