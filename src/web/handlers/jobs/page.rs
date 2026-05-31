//! Page entry-points: GET /jobs (renders the page) and POST /jobs/:id/decision.

use axum::extract::{Form, Path, Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::lane::{lane_hex, lane_label, LANE_KEYS};
use crate::data::models::{JobFilters, SortMode};
use crate::data::queries;
use crate::web::templates::{grade_pill, json_island, lane_legend, page_with, PageAssets};
use crate::web::AppState;

use super::charts::{
    decision_funnel_from, freshness_from, freshness_median_bucket, heatmap_cell_link,
    heatmap_from, lookup_funnel, top_companies_from, top_titles_from,
};
use super::filters::{axis_set, render_axis, render_lane_pie, set_href, view_href, AXES};
use super::lanes_view::render_lanes_view;
use super::table::{decision_buttons, render_table};

/// Multi-value chip query — every axis is a comma-separated list, empty/missing
/// means "no filter on this axis" (except `archive`, which defaults to `active`).
#[derive(Debug, Deserialize, Default)]
pub struct JobsQuery {
    pub lane: Option<String>,
    pub grade: Option<String>,
    pub decision: Option<String>,
    pub archive: Option<String>,
    pub ats: Option<String>,
    pub posted: Option<String>,
    pub sponsor: Option<String>,
    /// Single-value company-name filter (exact match on companies.name).
    /// Driven by click-to-filter from Top-companies rows; not part of the
    /// chip-strip axes, so it does not appear in `AXES`.
    pub company: Option<String>,
    /// Presentation mode for the filtered jobs list. `Some("lanes")` swaps the
    /// table for the 8-column lane-grouped grid; any other value (including
    /// `None` / `Some("table")`) renders the default table. Not part of `AXES`
    /// since it only controls rendering, not which jobs are included.
    pub view: Option<String>,
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<JobsQuery>,
) -> Html<String> {
    let body = render(&state, &q).await;
    Html(
        page_with(
            "Jobs",
            "jobs",
            PageAssets::css_js("/static/css/jobs.css", "/static/js/jobs.js"),
            body,
        )
        .into_string(),
    )
}

async fn render(state: &AppState, q: &JobsQuery) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    // Build JobFilters from query — JobFilters defaults set archive={active}
    // and evidence={jd,semantic}; we override per-axis from the query.
    let mut filters = JobFilters::default();
    filters.lanes = axis_set(q, &AXES[0]);
    filters.grades = axis_set(q, &AXES[1]);
    filters.decisions = axis_set(q, &AXES[2]);
    filters.archive = axis_set(q, &AXES[3]);
    let ats_set = axis_set(q, &AXES[4]);
    filters.ats = ats_set;
    filters.posted_within = axis_set(q, &AXES[5]);
    filters.sponsor = axis_set(q, &AXES[6]);

    // Optional click-driven company filter — resolve name → id, then pass to
    // fetch_jobs's existing `company_filter` Option<i64>. Empty / unknown names
    // collapse to None so the page still renders the universe.
    let company_id: Option<i64> = q.company.as_deref().and_then(|name| {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return None;
        }
        conn.query_row(
            "SELECT id FROM companies WHERE name = ?1 LIMIT 1",
            rusqlite::params![trimmed],
            |r| r.get::<_, i64>(0),
        )
        .ok()
    });
    let jobs = queries::fetch_jobs(&conn, company_id, &filters, SortMode::ByGrade);
    // Total jobs in the universe (for "X of Y" filter summary).
    let total_universe: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jobs j JOIN companies c ON c.id = j.company_id
             WHERE j.evaluation_status != 'archived' AND c.status != 'archived'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // ── Filter-scoped analytics (all computed from `jobs`) ────────────────
    let kpi_total = jobs.len() as i64;
    let kpi_strong = jobs
        .iter()
        .filter(|j| matches!(j.grade.as_deref(), Some("SS") | Some("S")))
        .count() as i64;
    let funnel = decision_funnel_from(&jobs);
    let kpi_untouched = lookup_funnel(&funnel, "untouched");
    let kpi_watching = lookup_funnel(&funnel, "watching");
    let kpi_applied = lookup_funnel(&funnel, "applied");
    let freshness = freshness_from(&jobs);
    let kpi_freshness_median = freshness_median_bucket(&freshness);
    let heatmap = heatmap_from(&jobs);
    let top_companies = top_companies_from(&jobs, 10);
    let top_titles = top_titles_from(&jobs, 10);

    let heatmap_max = heatmap
        .iter()
        .flat_map(|(_, row)| row.iter())
        .copied()
        .max()
        .unwrap_or(0);

    let freshness_colors_map = [
        ("0-7d", "#4ade80"),
        ("7-30d", "#7ea8ff"),
        ("30-90d", "#ffc94a"),
        ("90d+", "#ff8a5c"),
        ("unknown", "#4f5762"),
    ];
    let freshness_data = serde_json::json!({
        "labels": freshness.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
        "values": freshness.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
        "colors": freshness.iter().map(|(k, _)| {
            freshness_colors_map.iter().find(|(b, _)| *b == k.as_str())
                .map(|(_, c)| (*c).to_string()).unwrap_or_else(|| "#4f5762".to_string())
        }).collect::<Vec<_>>(),
    });

    let funnel_total: i64 = funnel.iter().map(|(_, n)| *n).sum();
    let companies_max = top_companies.iter().map(|(_, n)| *n).max().unwrap_or(0);
    let titles_max = top_titles.iter().map(|(_, n)| *n).max().unwrap_or(0);

    let any_active = AXES.iter().any(|a| {
        let s = axis_set(q, a);
        // Treat default archive={active} as "not active".
        if a.key == "archive" {
            !(s.len() == 1 && s.contains("active"))
        } else {
            !s.is_empty()
        }
    }) || q.company.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false);

    html! {
        div.jobs-page {
            (lane_legend("jobs"))

            // ── Chip strip filter (collapsible) ───────────────────────────
            @let summary_pairs: Vec<(&str, String)> = {
                let mut out: Vec<(&str, String)> = Vec::new();
                for axis in AXES.iter() {
                    let s = axis_set(q, axis);
                    // Skip default archive={active} from the summary.
                    if axis.key == "archive" && s.len() == 1 && s.contains("active") { continue; }
                    if s.is_empty() { continue; }
                    let mut vs: Vec<&str> = s.iter().map(|x| x.as_str()).collect();
                    vs.sort();
                    out.push((axis.key, vs.join("+")));
                }
                if let Some(c) = q.company.as_deref() {
                    if !c.trim().is_empty() { out.push(("company", c.trim().to_string())); }
                }
                out
            };
            @let is_lanes_view = matches!(q.view.as_deref(), Some("lanes"));
            div.filter-strip data-page="jobs" {
                div.filter-strip-head {
                    div.view-toggle.seg-group title="Switch between table and lane-column layout" {
                        a class="seg" href=(view_href(q, "table")) data-active=(!is_lanes_view) {
                            span.view-toggle-glyph { "⊞" } " Table"
                        }
                        a class="seg" href=(view_href(q, "lanes")) data-active=(is_lanes_view) {
                            span.view-toggle-glyph { "▦" } " Lanes"
                        }
                    }
                    div.filter-strip-summary {
                        @if summary_pairs.is_empty() {
                            span.fss-axis { "all filters · " }
                            span.fss-vals { "no axis active" }
                        }
                        @for (k, v) in &summary_pairs {
                            span.fss-axis { (k) ": " }
                            span.fss-vals { (v) }
                        }
                        span.fss-sep { "·" }
                        span.fss-count { (kpi_total) " of " (total_universe) " jobs" }
                    }
                    button.filter-strip-toggle type="button" aria-expanded="false" title="Toggle filters (f)" {
                        span.filter-strip-chevron { "▼" }
                        span { "filters" }
                        span.filter-strip-toggle-key { "f" }
                    }
                }
                div.filter-strip-body {
                    div.filter-body-grid {
                        // Left column — the lane pie (hero filter).
                        div.filter-pie-col {
                            (render_lane_pie(q))
                        }
                        // Right column — every non-lane axis stacked, with
                        // the summary/reset row pinned to the bottom.
                        div.filter-axes-col {
                            @for axis in AXES.iter().filter(|a| a.key != "lane") {
                                (render_axis(q, axis))
                            }
                            div.filter-summary-row {
                                div.filter-summary {
                                    span.filter-count { (kpi_total) }
                                    span.filter-count-total { " of " (total_universe) }
                                    span.filter-count-label {
                                        @if any_active { " filtered jobs" } @else { " jobs" }
                                    }
                                }
                                @if any_active {
                                    a.filter-reset href="/jobs" { "reset all" }
                                }
                            }
                        }
                    }
                }
            }

            // ── KPI strip ─────────────────────────────────────────────────
            section.kpi-strip {
                div.kpi {
                    div.kpi-label { "Filtered jobs" }
                    div.kpi-value data-ticker=(kpi_total) { "0" }
                }
                div.kpi {
                    div.kpi-label { "SS + S" }
                    div.kpi-value data-ticker=(kpi_strong) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Freshness median" }
                    div.kpi-value { (kpi_freshness_median) }
                }
                div.kpi {
                    div.kpi-label { "Untouched" }
                    div.kpi-value data-ticker=(kpi_untouched) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Watching" }
                    div.kpi-value data-ticker=(kpi_watching) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Applied" }
                    div.kpi-value data-ticker=(kpi_applied) { "0" }
                }
            }

            // ── Heatmap pane (grade × lane) ───────────────────────────────
            section.panel.heatmap {
                header.panel-head {
                    h2 { "Grade × Lane heatmap" }
                    span.panel-sub { "filtered jobs · cell shade scales to volume" }
                }
                div.heatmap-grid {
                    // Corner cell — no link.
                    div.heatmap-cell.heatmap-corner { "" }
                    // Column heads — lane → /jobs?lane=<key>.
                    @for key in LANE_KEYS.iter() {
                        a.heatmap-cell.heatmap-head.heatmap-cell-link
                            href=(set_href(&[("lane", key)]))
                            title=(lane_label(key))
                            style=(format!("--lane-color: {}", lane_hex(key))) {
                            (key)
                        }
                    }
                    @for (grade, row) in &heatmap {
                        // Row head — grade → /jobs?grade=<G>.
                        a.heatmap-cell.heatmap-row-head.heatmap-cell-link
                            href=(set_href(&[("grade", grade.as_str())])) {
                            (grade_pill(Some(grade.as_str())))
                        }
                        @for (li, n) in row.iter().enumerate() {
                            @let lane_key = LANE_KEYS[li];
                            (heatmap_cell_link(*n, heatmap_max, grade, lane_key))
                        }
                    }
                }
            }

            // ── Pair row: Freshness + Decision funnel ─────────────────────
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { "Posting freshness" }
                        span.panel-sub { "filtered jobs · by posted date" }
                    }
                    div id="chart-freshness" class="chart chart-sm" {}
                    (json_island("freshness", &freshness_data))
                }
                section.panel {
                    header.panel-head {
                        h2 { "Decision funnel" }
                        span.panel-sub { "filtered jobs · where each has landed" }
                    }
                    div.funnel {
                        @for (label, count) in &funnel {
                            @let pct = if funnel_total > 0 {
                                (*count as f64 * 100.0 / funnel_total as f64) as i32
                            } else { 0 };
                            // Decision-axis key — "untouched" maps to the
                            // "none" filter value, every other label matches
                            // its filter key 1:1.
                            @let dec_key = if label == "untouched" { "none" } else { label.as_str() };
                            a.funnel-row.row-clickable
                                href=(set_href(&[("decision", dec_key)]))
                                data-decision=(label) {
                                span.funnel-label { (label) }
                                div.funnel-bar-track {
                                    div.funnel-bar
                                        style=(format!("--target-scale: {}", pct as f64 / 100.0)) {}
                                }
                                span.funnel-num { (count) }
                                span.funnel-pct { (pct) "%" }
                            }
                        }
                    }
                }
            }

            // ── Pair row: Top companies + Top role titles ─────────────────
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { "Company concentration" }
                        span.panel-sub { "top 10 by filtered job count" }
                    }
                    div.top-list {
                        @if top_companies.is_empty() {
                            div.empty { "No jobs in current filter." }
                        }
                        @for (i, (name, total)) in top_companies.iter().enumerate() {
                            @let pct = if companies_max > 0 { *total * 100 / companies_max } else { 0 };
                            a.top-list-row.row-clickable
                                href=(set_href(&[("company", name.as_str())]))
                                title=(format!("Filter jobs by company: {name}")) {
                                span.top-list-rank { (i + 1) }
                                span.top-list-name { (name) }
                                div.top-list-bar {
                                    div.top-list-bar-fill style=(format!("--target-width: {pct}%")) {}
                                }
                                span.top-list-num { (total) }
                            }
                        }
                    }
                }
                section.panel {
                    header.panel-head {
                        h2 { "Top role titles" }
                        span.panel-sub { "filtered jobs · most common exact titles" }
                    }
                    div.top-list {
                        @if top_titles.is_empty() {
                            div.empty { "No jobs in current filter." }
                        }
                        @for (i, (title, count)) in top_titles.iter().enumerate() {
                            @let pct = if titles_max > 0 { *count * 100 / titles_max } else { 0 };
                            // Title-filter axis not yet wired; render as a
                            // hoverable row with a tooltip so the affordance is
                            // visible without a dead click target.
                            div.top-list-row
                                title=(format!("{title} (title filter coming)")) {
                                span.top-list-rank { (i + 1) }
                                span.top-list-name { (title) }
                                div.top-list-bar {
                                    div.top-list-bar-fill style=(format!("--target-width: {pct}%")) {}
                                }
                                span.top-list-num { (count) }
                            }
                        }
                    }
                }
            }

            // ── Filtered jobs list ────────────────────────────────────────
            // ?view=lanes swaps the table for an 8-column lane-grouped grid;
            // any other value (or absent) renders the default table.
            @if is_lanes_view {
                (render_lanes_view(&jobs))
            } @else {
                (render_table(&jobs))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DecisionForm {
    pub decision: String,
}

pub async fn decision(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Form(form): Form<DecisionForm>,
) -> Html<String> {
    let decision = form.decision.as_str();
    let allowed = matches!(decision, "applied" | "watching" | "rejected" | "interview");
    if !allowed {
        return Html(html! { div.error { "Invalid decision" } }.into_string());
    }

    if let Ok(conn) = Connection::open(&state.db_path) {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = conn.execute(
            "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, decision, now],
        );
        crate::db::events::decision_recorded(&conn, id, decision, "web");

        let url: String = conn
            .query_row(
                "SELECT url FROM jobs WHERE id = ?1",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        return Html(decision_buttons(id, Some(decision), &url).into_string());
    }

    Html(html! { div.error { "DB error" } }.into_string())
}
