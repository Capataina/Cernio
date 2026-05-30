//! Web frontend — axum HTTP server serving an HTML interface over the same
//! SQLite DB the TUI reads. Boot via `cernio web`.
//!
//! Architecture: server-rendered HTML via maud, HTMX for inline writes, ECharts
//! for the dashboard charts. Binds to 127.0.0.1 only (no auth, local-only).

pub mod debug_snap;
mod handlers;
mod templates;

use axum::routing::{get, post};
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
}

pub async fn serve(db_path: &str, port: u16, open_browser: bool) -> std::io::Result<()> {
    let state = Arc::new(AppState {
        db_path: PathBuf::from(db_path),
    });

    let app = Router::new()
        .route("/", get(handlers::dashboard::page))
        .route("/jobs", get(handlers::jobs::page))
        .route("/jobs/:id/decision", post(handlers::jobs::decision))
        .route("/companies", get(handlers::companies::page))
        .route("/activity", get(handlers::activity::page))
        .route("/activity/group", post(handlers::activity::toggle_group))
        .route("/api/stats.json", get(handlers::api::stats))
        .route("/ops/clean/preview", get(handlers::ops::clean_preview))
        .route("/ops/clean/run", post(handlers::ops::clean_run))
        .route("/ops/format/preview", get(handlers::ops::format_preview))
        .route("/ops/format/run", post(handlers::ops::format_run))
        .route("/debug/snap-all", post(debug_snap::snap_all))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    let url = format!("http://{addr}");
    println!("→ cernio web running at {url}");
    println!("  Ctrl+C to stop.");

    if open_browser {
        let url_for_open = url.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(400));
            let _ = std::process::Command::new("open").arg(&url_for_open).spawn();
        });
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await
}
