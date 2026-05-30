//! /decisions tab — a focused view of your Watched / Applied / Interview /
//! Rejected job pipeline. Companion to the Dashboard, Jobs, and Activity
//! tabs; gives action a home of its own.

use axum::extract::{Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::analytics;
use crate::web::templates::{
    grade_pill, json_island, lane_chip_for, lane_legend, page_with, PageAssets,
};
use crate::web::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct DecisionsQuery {
    #[serde(default)]
    pub kind: Option<String>,
}

const KINDS: &[(&str, &str)] = &[
    ("all", "All"),
    ("watching", "Watching"),
    ("applied", "Applied"),
    ("interview", "Interview"),
    ("rejected", "Rejected"),
];

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<DecisionsQuery>,
) -> Html<String> {
    let kind = q.kind.as_deref().unwrap_or("all").to_string();
    let body = render(&state, &kind).await;
    Html(
        page_with(
            "Decisions",
            "decisions",
            PageAssets::css_js("/static/css/decisions.css", "/static/js/decisions.js"),
            body,
        )
        .into_string(),
    )
}

async fn render(state: &AppState, kind: &str) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let decisions = analytics::decisions_filtered(&conn, kind);
    let counts = analytics::decision_counts(&conn);
    let watching = counts.get("watching").copied().unwrap_or(0);
    let applied = counts.get("applied").copied().unwrap_or(0);
    let interview = counts.get("interview").copied().unwrap_or(0);
    let rejected = counts.get("rejected").copied().unwrap_or(0);
    let total = watching + applied + interview + rejected;

    // Sankey-ish funnel: total → applied → interview → offer (no data yet)
    let funnel_json = serde_json::json!({
        "items": [
            {"name": "Total decisions", "value": total},
            {"name": "Watching",        "value": watching},
            {"name": "Applied",         "value": applied},
            {"name": "Interview",       "value": interview},
        ]
    });

    html! {
        div.decisions-page {
            (lane_legend("decisions"))

            // KPI strip
            section.kpi-strip {
                div.kpi.kpi-watching {
                    div.kpi-label { "Watching" }
                    div.kpi-value data-ticker=(watching) { "0" }
                }
                div.kpi.kpi-applied {
                    div.kpi-label { "Applied" }
                    div.kpi-value data-ticker=(applied) { "0" }
                }
                div.kpi.kpi-interview {
                    div.kpi-label { "Interview" }
                    div.kpi-value data-ticker=(interview) { "0" }
                }
                div.kpi.kpi-rejected {
                    div.kpi-label { "Rejected" }
                    div.kpi-value data-ticker=(rejected) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Total decisions" }
                    div.kpi-value data-ticker=(total) { "0" }
                }
            }

            // Tabs across decision kinds
            div.decision-tabs {
                @for (k, label) in KINDS.iter() {
                    @let active = *k == kind;
                    @let class = if active { "decision-tab decision-tab-active" } else { "decision-tab" };
                    a class=(class) href=(format!("/decisions?kind={k}")) data-kind=(k) {
                        (label)
                        @let count = match *k {
                            "all" => total,
                            "watching" => watching,
                            "applied" => applied,
                            "interview" => interview,
                            "rejected" => rejected,
                            _ => 0,
                        };
                        span.decision-tab-count { (count) }
                    }
                }
            }

            // Paired row: funnel + recent activity sparkline
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { "Pipeline" }
                        span.panel-sub { "decisions by stage" }
                    }
                    div #chart-decisions-funnel .chart.chart-md {}
                    (json_island("decisions-funnel", &funnel_json))
                }
                section.panel {
                    header.panel-head {
                        h2 { "Recent decisions" }
                        span.panel-sub { "your last actions, newest first" }
                    }
                    div.decisions-feed {
                        @if decisions.is_empty() {
                            div.empty {
                                "No decisions yet. Go to Jobs and pick something — Apply / Watch / Reject."
                            }
                        }
                        @for (i, (decided_at, decision, title, company, lanes_json, grade, url, _id)) in decisions.iter().enumerate().take(8) {
                            @let _ = i;
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

            // Full list of decisions for the active kind
            section.panel.no-pad {
                header.panel-head {
                    h2 { "All " (kind) " decisions" }
                    span.panel-sub { (decisions.len()) " records" }
                }
                @if decisions.is_empty() {
                    div.empty { "No decisions in this view." }
                } @else {
                    div.decisions-list {
                        @for (decided_at, decision, title, company, lanes_json, grade, url, id) in &decisions {
                            div.decision-list-row data-detail=(format!("job-{id}")) {
                                span.decision-when { (short_time(decided_at)) }
                                span.decision-kind data-kind=(decision) { (decision.as_str()) }
                                (lane_chip_for(lanes_json.as_deref()))
                                (grade_pill(grade.as_deref()))
                                div.decision-text {
                                    div.decision-title { (title) }
                                    div.decision-co { (company) }
                                }
                                a.decision-open href=(url) target="_blank" title="open posting" { "↗" }
                            }
                        }
                    }
                }
            }
        }
    }
}

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
