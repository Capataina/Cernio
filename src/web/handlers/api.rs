use axum::extract::State;
use axum::response::Json;
use rusqlite::Connection;
use std::sync::Arc;

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
