use axum::extract::State;
use axum::response::Json;
use rusqlite::Connection;
use std::sync::Arc;

use crate::data::lane::primary_lane;
use crate::data::queries;
use crate::web::AppState;

pub async fn stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return Json(serde_json::json!({ "error": "db" }));
    };
    let s = queries::fetch_stats(&conn);
    Json(serde_json::json!({
        "total_companies": s.total_companies,
        "total_jobs": s.total_jobs,
        "applied": s.applied_count,
        "watching": s.watching_count,
        "rejected": s.rejected_count,
        "by_grade": s.jobs_by_grade,
    }))
}

/// Lightweight search index for the Cmd-K palette. Returns active
/// (non-archived) companies and jobs as flat arrays. Held in browser memory.
pub async fn search_index(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return Json(serde_json::json!({ "error": "db" }));
    };

    // Companies: active only.
    let companies: Vec<serde_json::Value> = {
        let sql = "SELECT id, name, grade, lanes FROM companies
                   WHERE status != 'archived'
                   ORDER BY name COLLATE NOCASE";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Json(serde_json::json!({ "error": "prepare companies" })),
        };
        stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let grade: Option<String> = row.get(2)?;
            let lanes: Option<String> = row.get(3)?;
            Ok(serde_json::json!({
                "id": id,
                "name": name,
                "kind": "co",
                "lane": primary_lane(lanes.as_deref()),
                "grade": grade,
                "url": format!("/companies?detail=co-{id}"),
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    // Jobs: active only — exclude archived jobs and jobs of archived companies.
    let jobs: Vec<serde_json::Value> = {
        let sql = "SELECT j.id, j.title, c.name, j.grade, j.lanes
                   FROM jobs j JOIN companies c ON c.id = j.company_id
                   WHERE j.evaluation_status != 'archived'
                     AND c.status != 'archived'
                   ORDER BY c.name COLLATE NOCASE, j.title COLLATE NOCASE
                   LIMIT 1500";
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(_) => return Json(serde_json::json!({ "error": "prepare jobs" })),
        };
        stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let company: String = row.get(2)?;
            let grade: Option<String> = row.get(3)?;
            let lanes: Option<String> = row.get(4)?;
            Ok(serde_json::json!({
                "id": id,
                "name": format!("{title} — {company}"),
                "kind": "job",
                "lane": primary_lane(lanes.as_deref()),
                "grade": grade,
                "url": format!("/jobs?detail=job-{id}"),
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    };

    Json(serde_json::json!({
        "companies": companies,
        "jobs": jobs,
    }))
}
