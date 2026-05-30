//! Dashboard tab — dense, Palantir/Bloomberg-flavoured dashboard-of-dashboards.
//!
//! Layout, top-down:
//!   1. KPI strip (Companies / Jobs / Strong / Applied / Watching / Pending).
//!   2. Lane legend bar.
//!   3. Action queue  +  Companies × lane donut  (paired row).
//!   4. Jobs × lane × grade stacked bar with archived overlay (full width).
//!   5. 7-day activity sparkline  +  Recent decisions (paired row).
//!   6. Top actionable jobs (full width footer).
//!
//! Charts are rendered through ECharts via the shared `bootEchart` helper in
//! `static/js/charts.js`; data is shipped to JS through `json_island` script
//! blocks so the handler stays declarative.
//!
//! Note: ATS provider health pane was removed in favour of the canonical
//! version on /companies. The dashboard focuses on "what should I do today?"
//! while /companies focuses on the universe-by-ATS shape.

use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use std::sync::Arc;

use crate::data::analytics;
use crate::data::lane::{lane_hex, lane_label, LANE_KEYS};
use crate::data::queries;
use crate::web::templates::{
    grade_pill, json_island, lane_chip_for, lane_legend, page_with, PageAssets,
};
use crate::web::AppState;

const GRADE_KEYS: [&str; 6] = ["SS", "S", "A", "B", "C", "F"];
const GRADE_COLORS: [&str; 6] = [
    "#c39df0", // SS
    "#7adf9a", // S
    "#7ea8ff", // A
    "#ffc94a", // B
    "#ff5c5c", // C
    "#4f5762", // F
];

pub async fn page(State(state): State<Arc<AppState>>) -> Html<String> {
    let body = render(&state).await;
    Html(
        page_with(
            "Dashboard",
            "dashboard",
            PageAssets::css_js("/static/css/dashboard.css", "/static/js/dashboard.js"),
            body,
        )
        .into_string(),
    )
}

async fn render(state: &AppState) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let stats = queries::fetch_stats(&conn);
    let strong = stats
        .jobs_by_grade
        .iter()
        .filter(|(g, _)| g == "SS" || g == "S")
        .map(|(_, c)| *c)
        .sum::<i64>();
    let pending_eval = stats
        .jobs_by_eval
        .iter()
        .filter(|(e, _)| e == "pending")
        .map(|(_, c)| *c)
        .sum::<i64>();

    // ── Action queue (what needs attention right now) ──
    let action_queue = analytics::action_queue(&conn);

    // ── Companies × lane donut ──
    let companies_lane = analytics::companies_per_lane(&conn);
    let companies_lane_json = serde_json::json!({
        "items": companies_lane.iter()
            .map(|(k, n)| serde_json::json!({
                "label": lane_label(k),
                "value": n,
                "color": lane_hex(k),
            }))
            .collect::<Vec<_>>(),
    });

    // ── Jobs × lane × grade with archived overlay ──
    let lane_grade = analytics::jobs_lane_grade_split(&conn);
    let lane_grade_json = serde_json::json!({
        "lanes": LANE_KEYS.iter().map(|k| lane_label(k)).collect::<Vec<_>>(),
        "lane_keys": LANE_KEYS,
        "grades": GRADE_KEYS,
        "grade_colors": GRADE_COLORS,
        "active": lane_grade.iter()
            .map(|(_, a, _)| a.to_vec())
            .collect::<Vec<_>>(),
        "archived": lane_grade.iter()
            .map(|(_, _, a)| a.to_vec())
            .collect::<Vec<_>>(),
    });

    // ── 7-day events by source ──
    let activity = analytics::events_by_source_window(&conn, 7);
    let dates: Vec<String> = activity.iter().map(|(d, _)| d.clone()).collect();
    let sources = ["cli", "tui", "web", "script", "trigger", "other"];
    let source_colors = ["#7adf9a", "#7ea8ff", "#ff5c5c", "#ffc94a", "#aab3bf", "#4f5762"];
    let mut series: Vec<Vec<i64>> = vec![Vec::with_capacity(activity.len()); sources.len()];
    for (_, row) in &activity {
        for (i, n) in row.iter().enumerate() {
            series[i].push(*n);
        }
    }
    let activity_json = serde_json::json!({
        "dates": dates,
        "sources": sources,
        "colors": source_colors,
        "series": series,
    });

    // ── Recent decisions ──
    let recent_decisions = analytics::recent_decisions(&conn, 6);

    // ── Top actionable jobs ──
    let actionable = analytics::top_actionable_jobs(&conn, 8);

    html! {
        div.dashboard {
            // 1. KPI strip
            section.kpi-strip {
                div.kpi {
                    div.kpi-label { "Companies" }
                    div.kpi-value data-ticker=(stats.total_companies) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Jobs" }
                    div.kpi-value data-ticker=(stats.total_jobs) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Strong (SS+S)" }
                    div.kpi-value data-ticker=(strong) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Applied" }
                    div.kpi-value data-ticker=(stats.applied_count) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Watching" }
                    div.kpi-value data-ticker=(stats.watching_count) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Pending eval" }
                    div.kpi-value data-ticker=(pending_eval) { "0" }
                }
            }

            // 2. Lane legend
            (lane_legend())

            // 3. Action queue + companies × lane donut
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { "Action queue" }
                        span.panel-sub { "what needs attention right now · click to filter" }
                    }
                    div.action-queue {
                        @for (i, (label, count, url)) in action_queue.iter().enumerate() {
                            a.action-item href=(url) data-count=(count) {
                                span.action-rank { (i + 1) }
                                span.action-label { (label) }
                                span.action-count { (count) }
                            }
                        }
                    }
                }
                section.panel {
                    header.panel-head {
                        h2 { "Companies × lane" }
                        span.panel-sub { "active universe by primary lane" }
                    }
                    div #chart-companies-lane .chart.chart-donut {}
                    (json_island("companies-lane", &companies_lane_json))
                }
            }

            // 4. Jobs × lane × grade with archived overlay (full width)
            section.panel {
                header.panel-head {
                    h2 { "Jobs × lane × grade" }
                    span.panel-sub { "stacked by grade · hatched layer = archived · click cell to filter" }
                }
                div #chart-jobs-lane-grade .chart.chart-md {}
                (json_island("jobs-lane-grade", &lane_grade_json))
            }

            // 5. 7-day activity + recent decisions
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { "7-day activity" }
                        span.panel-sub { "events per day, stacked by source" }
                    }
                    div #chart-activity-7d .chart.chart-md {}
                    (json_island("activity-7d", &activity_json))
                }
                section.panel {
                    header.panel-head {
                        h2 { "Recent decisions" }
                        span.panel-sub { "your last applies / watches / rejects" }
                    }
                    div.decisions-feed {
                        @if recent_decisions.is_empty() {
                            div.empty { "No decisions yet — Apply / Watch / Reject a job to start." }
                        }
                        @for (decided_at, decision, title, company, lanes_json, grade, url, _id) in &recent_decisions {
                            a.decision-row href=(url) target="_blank" data-decision=(decision) {
                                span.decision-when { (short_time(decided_at)) }
                                span.decision-kind data-kind=(decision) { (decision.as_str()) }
                                (lane_chip_for(lanes_json.as_deref()))
                                (grade_pill(grade.as_deref()))
                                span.decision-title { (title) }
                                span.decision-co { (company) }
                            }
                        }
                    }
                }
            }

            // 6. Top actionable jobs footer
            section.panel {
                header.panel-head {
                    h2 { "Top actionable jobs" }
                    span.panel-sub { "SS + S · no decision yet · most recent first" }
                }
                div.top-roles {
                    @if actionable.is_empty() {
                        div.empty { "No actionable jobs — every SS/S has a decision." }
                    }
                    @for (id, grade, title, lanes_json, _location, company, url) in &actionable {
                        div.role-row {
                            (lane_chip_for(lanes_json.as_deref()))
                            (grade_pill(Some(grade.as_str())))
                            div.role-text {
                                div.role-title data-marquee=(title) { (title) }
                                div.role-company { (company) }
                            }
                            a.role-apply href=(url) target="_blank"
                                hx-post=(format!("/jobs/{id}/decision"))
                                hx-vals=(r#"{"decision":"applied"}"#)
                                hx-swap="outerHTML" {
                                "Apply"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render a decided_at timestamp as a short relative-time label.
fn short_time(ts: &str) -> String {
    let parsed = chrono::DateTime::parse_from_rfc3339(ts)
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
            .map(|n| n.and_utc().fixed_offset()));
    let Ok(dt) = parsed else { return ts.to_string(); };
    let now = chrono::Utc::now();
    let delta = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
    let mins = delta.num_minutes();
    if mins < 1 { "just now".into() }
    else if mins < 60 { format!("{mins}m") }
    else if mins < 1440 { format!("{}h", mins / 60) }
    else { format!("{}d", mins / 1440) }
}
