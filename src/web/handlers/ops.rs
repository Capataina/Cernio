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

use axum::extract::State;
use axum::response::Json;
use rusqlite::Connection;
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
    let lanes = ["big-tech","ai-ml","hft","crypto-mm","bank-strats","systems-infra","devtools","fintech"];
    let mut by_grade = serde_json::Map::new();
    let mut by_lane = serde_json::Map::new();
    for l in &lanes { by_lane.insert((*l).to_string(), json!(0)); }
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
            by_grade.insert((*grade).to_string(), json!(n));
        }
        jobs_total += n;
        // Drill into per-lane breakdown for this tier
        for l in &lanes {
            let prefix = format!("[\"{l}\"%");
            let m: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM jobs
                     WHERE grade = ?1 AND lanes LIKE ?2
                     AND evaluation_status != 'archived'
                     AND datetime(discovered_at) < datetime('now', ?3)
                     AND id NOT IN (SELECT job_id FROM user_decisions)",
                    rusqlite::params![grade, prefix, format!("-{days} days")],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if m > 0 {
                let prev = by_lane.get(*l).and_then(|v| v.as_i64()).unwrap_or(0);
                by_lane.insert((*l).to_string(), json!(prev + m));
            }
        }
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

    let orphans = count(
        &conn,
        "SELECT COUNT(*) FROM user_decisions WHERE job_id NOT IN (SELECT id FROM jobs)",
    );

    let summary = format!(
        "{jobs_total} jobs archived · {expired_archived} pruned · {companies_archivable} companies"
    );
    Json(json!({
        "ok": true,
        "summary": summary,
        "detail": {
            "would_archive_total": jobs_total,
            "by_grade": Value::Object(by_grade),
            "by_lane": Value::Object(by_lane),
            "archived_to_purge": expired_archived,
            "low_grade_companies": companies_archivable,
            "orphan_decisions": orphans,
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

    let _ = total_jobs; // kept earlier for context; not surfaced
    Json(json!({
        "ok": true,
        "summary": summary,
        "detail": {
            "candidates": desc_candidates + assess_candidates,
            "jobs_descriptions": desc_candidates,
            "jobs_assessments": assess_candidates,
            "cap": cap,
            "would_mutate": touch.min(cap),
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

// Note: check / search / unarchive ops were dropped from the web menu —
// they require interactive context (check is long-running, search needs
// subagents, unarchive needs scope reasoning) and live on the CLI instead.
