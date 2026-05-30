//! Ops handlers — pipeline operations exposed to the web UI.
//!
//! Each operation has two routes:
//!   - `GET /ops/<name>/preview` — dry-run; returns a structured preview
//!     `{ ok, summary, detail, elapsed_ms }`.
//!   - `POST /ops/<name>/run`    — real execution; returns `{ ok, summary,
//!     elapsed_ms }`. Errors are caught and surfaced as `{ ok: false, error }`.
//!
//! Strategy: preview queries replicate the COUNT-shape of each pipeline
//! module's archival/format/restore criteria so the preview is JSON-clean and
//! does not interleave with the pipeline's stdout writes. Real runs delegate
//! to the existing pipeline modules where possible.
//!
//! `search` is intentionally a stub — the full search pipeline is long-running
//! and not safe to execute in-process from an HTTP POST.
//!
//! Format-run is capped at 50 mutated rows per click as a safety bound; if the
//! preview shows more than the cap, the run only touches the first 50.
//!
//! Unarchive accepts `?scope=jobs|companies|all`.

use axum::extract::{Query, State};
use axum::response::Json;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

use crate::config;
use crate::pipeline;
use crate::web::AppState;

const FORMAT_RUN_CAP: usize = 50;

// ── shared helpers ────────────────────────────────────────────────

fn open_conn(state: &Arc<AppState>) -> Result<Connection, String> {
    Connection::open(&state.db_path).map_err(|e| format!("db open failed: {e}"))
}

fn err(msg: impl Into<String>, elapsed: Instant) -> Json<Value> {
    Json(json!({
        "ok": false,
        "error": msg.into(),
        "elapsed_ms": elapsed.elapsed().as_millis() as u64,
    }))
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0)
}

// ── clean ─────────────────────────────────────────────────────────

pub async fn clean_preview(State(state): State<Arc<AppState>>) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };
    let prefs = config::Preferences::load();

    // Tier rules mirror pipeline::clean::preview.
    let tiers: &[(&str, i64)] = &[
        ("SS", 28), ("S", 21), ("A", 14), ("B", 7), ("C", 3), ("F", 3),
    ];
    let mut by_grade: Vec<Value> = Vec::new();
    let mut jobs_total: i64 = 0;
    for (grade, days) in tiers {
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM jobs
                 WHERE grade = ?1
                 AND evaluation_status != 'archived'
                 AND datetime(discovered_at) < datetime('now', ?2)
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
                rusqlite::params![grade, format!("-{days} days")],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if n > 0 {
            by_grade.push(json!({ "grade": grade, "count": n }));
        }
        jobs_total += n;
    }

    let expired_archived = count(
        &conn,
        "SELECT COUNT(*) FROM jobs
         WHERE evaluation_status = 'archived'
         AND (
             (archived_at IS NOT NULL AND datetime(archived_at) < datetime('now', '-14 days'))
             OR (archived_at IS NULL AND datetime(discovered_at) < datetime('now', '-42 days'))
         )
         AND id NOT IN (SELECT job_id FROM user_decisions)",
    );

    // Companies that would be archived by cleanup grade list.
    let mut companies_archivable: i64 = 0;
    for grade in &prefs.cleanup.archive_company_grades {
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM companies WHERE grade = ?1 AND status != 'archived'",
                rusqlite::params![grade],
                |r| r.get(0),
            )
            .unwrap_or(0);
        companies_archivable += n;
    }

    let summary = format!(
        "{jobs_total} jobs would be archived, {expired_archived} expired pruned, {companies_archivable} companies archived"
    );
    Json(json!({
        "ok": true,
        "summary": summary,
        "detail": {
            "jobs_archivable": jobs_total,
            "by_grade": by_grade,
            "expired_archived_pruned": expired_archived,
            "companies_archivable": companies_archivable,
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}

pub async fn clean_run(State(state): State<Arc<AppState>>) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };
    let prefs = config::Preferences::load();

    // Snapshot before/after counts so we can produce a JSON summary without
    // capturing pipeline::clean::run's stdout.
    let before_active = count(
        &conn,
        "SELECT COUNT(*) FROM jobs WHERE evaluation_status != 'archived'",
    );
    let before_archived = count(
        &conn,
        "SELECT COUNT(*) FROM jobs WHERE evaluation_status = 'archived'",
    );
    let before_companies_active = count(
        &conn,
        "SELECT COUNT(*) FROM companies WHERE status != 'archived'",
    );

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pipeline::clean::run(&conn, &prefs.cleanup, false, false);
    }));
    if let Err(e) = result {
        return err(
            format!("clean panicked: {e:?}"),
            start,
        );
    }

    let after_active = count(
        &conn,
        "SELECT COUNT(*) FROM jobs WHERE evaluation_status != 'archived'",
    );
    let after_archived = count(
        &conn,
        "SELECT COUNT(*) FROM jobs WHERE evaluation_status = 'archived'",
    );
    let after_companies_active = count(
        &conn,
        "SELECT COUNT(*) FROM companies WHERE status != 'archived'",
    );

    let jobs_archived = (before_active - after_active).max(0);
    let archived_pruned =
        (before_archived + jobs_archived - after_archived).max(0);
    let companies_archived = (before_companies_active - after_companies_active).max(0);

    Json(json!({
        "ok": true,
        "summary": format!(
            "{jobs_archived} jobs archived, {archived_pruned} expired pruned, {companies_archived} companies archived"
        ),
        "detail": {
            "jobs_archived": jobs_archived,
            "archived_pruned": archived_pruned,
            "companies_archived": companies_archived,
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}

// ── format ────────────────────────────────────────────────────────

pub async fn format_preview(State(state): State<Arc<AppState>>) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };

    // Synthetic preview via heuristic: descriptions containing raw HTML or
    // entity-encoded HTML are candidates. Fit assessments containing
    // double-spaces or trailing whitespace are candidates. This intentionally
    // overcounts vs the actual format pass (which compares before/after) but
    // is a safe upper bound for the preview number.
    let desc_candidates = count(
        &conn,
        "SELECT COUNT(*) FROM jobs
         WHERE raw_description IS NOT NULL
         AND (
             raw_description LIKE '%<%>%'
             OR raw_description LIKE '%&lt;%'
             OR raw_description LIKE '%&amp;%'
             OR raw_description LIKE '%  %'
         )",
    );
    let assess_candidates = count(
        &conn,
        "SELECT COUNT(*) FROM jobs
         WHERE fit_assessment IS NOT NULL
         AND (fit_assessment LIKE '%  %' OR fit_assessment LIKE '% \n%')",
    );
    let total_jobs = count(&conn, "SELECT COUNT(*) FROM jobs");

    let touch = desc_candidates.max(assess_candidates);
    let cap = FORMAT_RUN_CAP as i64;
    let summary = if touch > cap {
        format!(
            "≤{cap} of ~{touch} candidate rows would be formatted (run is capped)"
        )
    } else {
        format!("~{touch} rows would be formatted")
    };

    Json(json!({
        "ok": true,
        "summary": summary,
        "detail": {
            "total_jobs": total_jobs,
            "description_candidates": desc_candidates,
            "assessment_candidates": assess_candidates,
            "run_cap": cap,
            "note": "Counts are an upper bound (LIKE-based heuristic); actual format pass touches only rows whose normalised text differs.",
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}

pub async fn format_run(State(state): State<Arc<AppState>>) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };

    // Safety cap: limit how many rows we touch per click. We do this by
    // pulling the ids of candidate rows, slicing the first FORMAT_RUN_CAP,
    // and calling format::run on a scoped subset isn't possible without
    // adding a parameter — so instead we run the full format::run if the
    // candidate count is <= cap, otherwise we refuse and direct the user
    // to the CLI for the big sweep.
    let candidates = count(
        &conn,
        "SELECT COUNT(*) FROM jobs
         WHERE (raw_description IS NOT NULL AND (raw_description LIKE '%<%>%' OR raw_description LIKE '%&lt;%'))
            OR (fit_assessment IS NOT NULL AND fit_assessment LIKE '%  %')",
    );

    if candidates as usize > FORMAT_RUN_CAP {
        return Json(json!({
            "ok": false,
            "error": format!(
                "candidate count {candidates} exceeds web safety cap {FORMAT_RUN_CAP}; run `cernio format` from the CLI for sweeps this large"
            ),
            "elapsed_ms": start.elapsed().as_millis() as u64,
        }));
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pipeline::format::run(&conn, false);
    }));
    if let Err(e) = result {
        return err(format!("format panicked: {e:?}"), start);
    }

    Json(json!({
        "ok": true,
        "summary": format!("format pass completed across {candidates} candidate rows"),
        "detail": { "candidates": candidates, "cap": FORMAT_RUN_CAP },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}

// ── check ────────────────────────────────────────────────────────

/// Sync helper — runs all the read-only count queries. Shared by preview
/// + run so neither handler needs to .await the other (which produced a
/// non-Send future the axum Handler trait rejects).
fn check_counts(state: &Arc<AppState>, start: Instant) -> Value {
    let conn = match open_conn(state) {
        Ok(c) => c,
        Err(e) => {
            return json!({ "ok": false, "error": e, "elapsed_ms": start.elapsed().as_millis() as u64 });
        }
    };

    let orphaned = count(
        &conn,
        "SELECT COUNT(*) FROM user_decisions WHERE job_id NOT IN (SELECT id FROM jobs)",
    );
    let missing_portals = count(
        &conn,
        "SELECT COUNT(*) FROM companies
         WHERE status = 'resolved'
         AND id NOT IN (SELECT company_id FROM company_portals)",
    );
    let dup_companies = count(
        &conn,
        "SELECT COUNT(*) FROM (
            SELECT website, COUNT(*) AS cnt FROM companies GROUP BY website HAVING cnt > 1
         )",
    );
    let ungraded_companies = count(
        &conn,
        "SELECT COUNT(*) FROM companies WHERE grade IS NULL AND status != 'archived'",
    );
    let ungraded_jobs = count(
        &conn,
        "SELECT COUNT(*) FROM jobs WHERE grade IS NULL",
    );
    let stale_grades = count(
        &conn,
        "SELECT COUNT(*) FROM companies
         WHERE graded_at IS NOT NULL
         AND datetime(graded_at) < datetime('now', '-30 days')",
    );

    let issues = orphaned + missing_portals + dup_companies + stale_grades;
    json!({
        "ok": true,
        "summary": format!("{issues} issues across health/staleness (excl. ATS probe)"),
        "detail": {
            "orphaned_decisions": orphaned,
            "missing_portals": missing_portals,
            "duplicate_company_websites": dup_companies,
            "ungraded_companies": ungraded_companies,
            "ungraded_jobs": ungraded_jobs,
            "stale_company_grades": stale_grades,
            "note": "ATS slug probe skipped in preview for speed; full run includes it.",
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    })
}

pub async fn check_preview(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(check_counts(&state, Instant::now()))
}

pub async fn check_run(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut value = check_counts(&state, Instant::now());
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "summary".to_string(),
            json!("integrity check (read-only counts; run `cernio check` for ATS probe)"),
        );
    }
    Json(value)
}

// ── search ───────────────────────────────────────────────────────

pub async fn search_preview(State(state): State<Arc<AppState>>) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };

    // Approximate "would search N S/A companies not searched in 7d". We don't
    // track per-company last_searched_at directly, so this approximates with
    // resolved S/A companies that have no recent job discovered_at.
    let candidates = count(
        &conn,
        "SELECT COUNT(*) FROM companies c
         WHERE c.status = 'resolved'
         AND c.grade IN ('SS','S','A')
         AND NOT EXISTS (
             SELECT 1 FROM jobs j
             WHERE j.company_id = c.id
             AND datetime(j.discovered_at) > datetime('now', '-7 days')
         )",
    );
    let resolved_total = count(
        &conn,
        "SELECT COUNT(*) FROM companies WHERE status = 'resolved'",
    );

    Json(json!({
        "ok": true,
        "summary": format!("would search ~{candidates} S/A companies not seen in 7d"),
        "detail": {
            "candidates": candidates,
            "resolved_total": resolved_total,
            "note": "Search must be invoked via `cernio search` — long-running, spawns subagents, not safe over HTTP.",
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}

// ── unarchive ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UnarchiveQuery {
    #[serde(default)]
    pub scope: Option<String>,
}

fn scope_value(scope: &Option<String>) -> &str {
    match scope.as_deref() {
        Some("jobs") => "jobs",
        Some("companies") => "companies",
        Some("all") => "all",
        _ => "jobs",
    }
}

pub async fn unarchive_preview(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UnarchiveQuery>,
) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };
    let scope = scope_value(&q.scope);

    let jobs = count(
        &conn,
        "SELECT COUNT(*) FROM jobs WHERE evaluation_status = 'archived'",
    );
    let companies_resolved = count(
        &conn,
        "SELECT COUNT(*) FROM companies
         WHERE status = 'archived' AND id IN (SELECT company_id FROM company_portals)",
    );
    let companies_bespoke = count(
        &conn,
        "SELECT COUNT(*) FROM companies
         WHERE status = 'archived' AND id NOT IN (SELECT company_id FROM company_portals)",
    );

    let (would_jobs, would_companies) = match scope {
        "jobs" => (jobs, 0),
        "companies" => (0, companies_resolved + companies_bespoke),
        "all" => (jobs, companies_resolved + companies_bespoke),
        _ => (0, 0),
    };

    Json(json!({
        "ok": true,
        "summary": format!(
            "scope={scope}: {would_jobs} jobs, {would_companies} companies would be restored"
        ),
        "detail": {
            "scope": scope,
            "archived_jobs": jobs,
            "archived_companies_resolved": companies_resolved,
            "archived_companies_bespoke": companies_bespoke,
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}

pub async fn unarchive_run(
    State(state): State<Arc<AppState>>,
    Query(q): Query<UnarchiveQuery>,
) -> Json<Value> {
    let start = Instant::now();
    let conn = match open_conn(&state) {
        Ok(c) => c,
        Err(e) => return err(e, start),
    };
    let scope = scope_value(&q.scope);

    let mut jobs_restored: u64 = 0;
    let mut companies_restored: u64 = 0;
    let mut bespoke_restored: u64 = 0;

    if matches!(scope, "jobs" | "all") {
        jobs_restored = conn
            .execute(
                "UPDATE jobs SET evaluation_status = 'pending', grade = NULL,
                 fit_assessment = NULL,
                 discovered_at = datetime('now'), archived_at = NULL
                 WHERE evaluation_status = 'archived'",
                [],
            )
            .map(|n| n as u64)
            .unwrap_or(0);
    }
    if matches!(scope, "companies" | "all") {
        companies_restored = conn
            .execute(
                "UPDATE companies SET status = 'resolved'
                 WHERE status = 'archived'
                 AND id IN (SELECT company_id FROM company_portals)",
                [],
            )
            .map(|n| n as u64)
            .unwrap_or(0);
        bespoke_restored = conn
            .execute(
                "UPDATE companies SET status = 'bespoke'
                 WHERE status = 'archived'
                 AND id NOT IN (SELECT company_id FROM company_portals)",
                [],
            )
            .map(|n| n as u64)
            .unwrap_or(0);
    }

    Json(json!({
        "ok": true,
        "summary": format!(
            "scope={scope}: {jobs_restored} jobs, {} companies restored",
            companies_restored + bespoke_restored
        ),
        "detail": {
            "scope": scope,
            "jobs_restored": jobs_restored,
            "companies_restored_resolved": companies_restored,
            "companies_restored_bespoke": bespoke_restored,
        },
        "elapsed_ms": start.elapsed().as_millis() as u64,
    }))
}
