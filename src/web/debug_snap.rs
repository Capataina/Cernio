//! Debug screenshot tool — captures every web tab into /tmp/cernio-debug/<ts>/.
//!
//! Three capture modes per page, all run in one shot:
//!   1. `<page>/full.png`            — entire page, single image
//!   2. `<page>/viewport-NN.png`     — page sliced into viewport-tall bands
//!   3. `<page>/pane-NN-<label>.png` — each <section.panel> + KPI strip + filter
//!                                     bar + lane legend captured tight
//!
//! Optional temporal mode (`--temporal` on CLI / `?temporal=1` on the route)
//! captures every artefact twice with a 3-second gap. Useful for spotting
//! marquee animations, count-up tickers settling, etc. Output layout:
//!   `<page>/t0/...`  initial snapshot
//!   `<page>/t1/...`  3s later
//!
//! Triggered three ways:
//!   - POST /debug/snap-all                — from the floating button
//!   - POST /debug/snap-all?temporal=1     — temporal mode
//!   - CLI: `cernio snap [--port N] [--temporal]` — for headless testing

use axum::extract::{Host, Query, State};
use axum::response::Json;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams, Viewport,
};
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::web::AppState;

const CHROME_PATH: &str =
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

const PAGES: &[(&str, &str)] = &[
    ("dashboard", "/"),
    ("companies", "/companies"),
    ("jobs", "/jobs"),
    ("jobs-filtered", "/jobs?lane=hft"),
    ("companies-filtered", "/companies?lane=hft"),
    ("jobs-lanes", "/jobs?view=lanes"),
    ("decisions", "/decisions"),
    ("activity", "/activity"),
];

const VIEWPORT_WIDTH: u32 = 1600;
const VIEWPORT_HEIGHT: u32 = 1000;
const POST_NAV_SETTLE_MS: u64 = 1600;
const TEMPORAL_GAP_MS: u64 = 3000;

#[derive(Debug, Deserialize, Default)]
pub struct SnapQuery {
    #[serde(default)]
    pub temporal: Option<String>,
}

pub async fn snap_all(
    State(_state): State<Arc<AppState>>,
    Host(host): Host,
    Query(q): Query<SnapQuery>,
) -> Json<serde_json::Value> {
    let temporal = q
        .temporal
        .as_deref()
        .map(|v| matches!(v, "1" | "true" | "yes"))
        .unwrap_or(false);
    let started = Instant::now();
    match run_snap(&host, temporal).await {
        Ok((folder, files)) => Json(serde_json::json!({
            "ok": true,
            "folder": folder.to_string_lossy(),
            "files": files,
            "count": files.len(),
            "temporal": temporal,
            "elapsed_ms": started.elapsed().as_millis(),
        })),
        Err(e) => Json(serde_json::json!({
            "ok": false,
            "error": e,
            "elapsed_ms": started.elapsed().as_millis(),
        })),
    }
}

/// Public CLI entry point — called from main.rs `cernio snap`.
pub async fn run_cli(host: String, temporal: bool) -> Result<PathBuf, String> {
    let (folder, files) = run_snap(&host, temporal).await?;
    println!("\n→ wrote {} files", files.len());
    println!("→ {}", folder.display());
    Ok(folder)
}

async fn run_snap(host: &str, temporal: bool) -> Result<(PathBuf, Vec<String>), String> {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let root = PathBuf::from(format!("/tmp/cernio-debug/{ts}"));
    std::fs::create_dir_all(&root)
        .map_err(|e| format!("mkdir {}: {e}", root.display()))?;

    let config = BrowserConfig::builder()
        .chrome_executable(CHROME_PATH)
        .arg("--hide-scrollbars")
        .arg("--disable-gpu")
        .arg(format!("--window-size={VIEWPORT_WIDTH},{VIEWPORT_HEIGHT}"))
        .build()
        .map_err(|e| format!("browser config: {e}"))?;

    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| format!("launch chrome: {e}"))?;

    let handler_task = tokio::spawn(async move {
        while let Some(_ev) = handler.next().await {}
    });

    let mut written = Vec::new();
    let mut last_err: Option<String> = None;

    for (slug, path) in PAGES {
        let url = format!("http://{host}{path}");
        let page_folder = root.join(slug);
        if let Err(e) = std::fs::create_dir_all(&page_folder) {
            last_err = Some(format!("{slug}: mkdir: {e}"));
            continue;
        }
        match snap_page(&browser, &url, &page_folder, temporal).await {
            Ok(mut paths) => written.append(&mut paths),
            Err(e) => last_err = Some(format!("{slug}: {e}")),
        }
    }

    let _ = browser.close().await;
    handler_task.abort();

    if written.is_empty() {
        return Err(last_err.unwrap_or_else(|| "no pages captured".into()));
    }
    Ok((root, written))
}

/// Capture one page. If `temporal` is true, runs the whole capture suite twice
/// (3s apart) into `t0/` and `t1/` subfolders; otherwise writes flat into
/// `page_folder` directly.
async fn snap_page(
    browser: &Browser,
    url: &str,
    page_folder: &Path,
    temporal: bool,
) -> Result<Vec<String>, String> {
    let page = browser
        .new_page("about:blank")
        .await
        .map_err(|e| format!("new_page: {e}"))?;

    page.execute(
        SetDeviceMetricsOverrideParams::builder()
            .width(VIEWPORT_WIDTH as i64)
            .height(VIEWPORT_HEIGHT as i64)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
            .unwrap(),
    )
    .await
    .map_err(|e| format!("set viewport: {e}"))?;

    page.goto(url).await.map_err(|e| format!("goto: {e}"))?;
    page.wait_for_navigation()
        .await
        .map_err(|e| format!("wait_for_navigation: {e}"))?;

    tokio::time::sleep(Duration::from_millis(POST_NAV_SETTLE_MS)).await;

    // Hide UI elements that would clutter every screenshot.
    let _ = page
        .evaluate(
            r#"
            (function () {
              const ids = ['snap-all', 'snap-toast', 'ambient-canvas'];
              for (const id of ids) {
                const el = document.getElementById(id);
                if (el) el.style.display = 'none';
              }
            })();
            "#,
        )
        .await;

    let mut all_paths = Vec::new();

    if temporal {
        let t0 = page_folder.join("t0");
        let t1 = page_folder.join("t1");
        std::fs::create_dir_all(&t0).map_err(|e| format!("mkdir t0: {e}"))?;
        std::fs::create_dir_all(&t1).map_err(|e| format!("mkdir t1: {e}"))?;

        let mut paths_t0 = capture_suite(&page, &t0).await?;
        all_paths.append(&mut paths_t0);

        tokio::time::sleep(Duration::from_millis(TEMPORAL_GAP_MS)).await;

        let mut paths_t1 = capture_suite(&page, &t1).await?;
        all_paths.append(&mut paths_t1);
    } else {
        let mut paths = capture_suite(&page, page_folder).await?;
        all_paths.append(&mut paths);
    }

    let _ = page.close().await;
    Ok(all_paths)
}

/// Run the full capture suite (full + viewport slices + per-pane) into `into`.
async fn capture_suite(page: &Page, into: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();

    // 1. Full-page PNG.
    let full_bytes = page
        .screenshot(
            CaptureScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .capture_beyond_viewport(true)
                .from_surface(true)
                .build(),
        )
        .await
        .map_err(|e| format!("screenshot full: {e}"))?;
    let full_path = into.join("full.png");
    std::fs::write(&full_path, &full_bytes)
        .map_err(|e| format!("write full: {e}"))?;
    out.push(full_path.to_string_lossy().into_owned());

    // 2. Viewport slices — divide the full document into VIEWPORT_HEIGHT bands.
    let page_height = match page
        .evaluate("Math.max(document.documentElement.scrollHeight, document.body.scrollHeight)")
        .await
    {
        Ok(v) => v.into_value::<f64>().unwrap_or(VIEWPORT_HEIGHT as f64),
        Err(_) => VIEWPORT_HEIGHT as f64,
    };
    let n_slices = ((page_height / VIEWPORT_HEIGHT as f64).ceil() as u32).max(1);
    for i in 0..n_slices {
        let y = (i * VIEWPORT_HEIGHT) as f64;
        let remaining = page_height - y;
        let h = remaining.min(VIEWPORT_HEIGHT as f64);
        if h < 40.0 {
            continue;
        }
        let clip = Viewport {
            x: 0.0,
            y,
            width: VIEWPORT_WIDTH as f64,
            height: h,
            scale: 1.0,
        };
        let bytes = match page
            .screenshot(
                CaptureScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .capture_beyond_viewport(true)
                    .from_surface(true)
                    .clip(clip)
                    .build(),
            )
            .await
        {
            Ok(b) => b,
            Err(_) => continue,
        };
        let name = format!("viewport-{:02}.png", i + 1);
        let p = into.join(&name);
        if std::fs::write(&p, &bytes).is_ok() {
            out.push(p.to_string_lossy().into_owned());
        }
    }

    // 3. Per-pane PNGs.
    let panel_rects: Vec<PanelRect> = match page
        .evaluate(
            r#"
            (function () {
              const out = [];
              // Cover every standalone bordered card AND structural strips.
              const selectors = [
                'section.panel',
                '.pane',
                '.kpi-strip',
                '.filter-bar',
                '.filter-strip',
                '.lane-legend',
                '.heatmap',
                '.funnel',
              ];
              const seen = new Set();
              for (const sel of selectors) {
                document.querySelectorAll(sel).forEach((p) => {
                  if (seen.has(p)) return;
                  seen.add(p);
                  const r = p.getBoundingClientRect();
                  const x = r.left + window.scrollX;
                  const y = r.top + window.scrollY;
                  let label = sel.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '');
                  const h2 = p.querySelector('.panel-head h2');
                  if (h2) {
                    label = h2.textContent.trim().toLowerCase()
                      .replace(/[^a-z0-9]+/g, '-')
                      .replace(/^-+|-+$/g, '')
                      .slice(0, 40);
                  }
                  out.push({ label, x, y, w: r.width, h: r.height });
                });
              }
              // Sort top-to-bottom so file numbering matches reading order.
              out.sort((a, b) => a.y - b.y);
              return out;
            })();
            "#,
        )
        .await
    {
        Ok(v) => v.into_value::<Vec<PanelRect>>().unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    for (i, rect) in panel_rects.iter().enumerate() {
        if rect.w < 40.0 || rect.h < 40.0 {
            continue;
        }
        let clip = Viewport {
            x: rect.x,
            y: rect.y,
            width: rect.w,
            height: rect.h,
            scale: 1.0,
        };
        let bytes = match page
            .screenshot(
                CaptureScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .capture_beyond_viewport(true)
                    .from_surface(true)
                    .clip(clip)
                    .build(),
            )
            .await
        {
            Ok(b) => b,
            Err(_) => continue,
        };
        let name = format!("pane-{:02}-{}.png", i + 1, rect.label);
        let p = into.join(&name);
        if std::fs::write(&p, &bytes).is_ok() {
            out.push(p.to_string_lossy().into_owned());
        }
    }

    Ok(out)
}

#[derive(Debug, Deserialize)]
struct PanelRect {
    label: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
