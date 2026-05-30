use axum::extract::{Form, Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::activity::{group_timeline, ActivityGroup};
use crate::data::analytics;
use crate::data::models::ActivityEntry;
use crate::data::queries;
use crate::web::templates::{json_island, lane_chip_for, page_with, PageAssets};
use crate::web::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct ActivityQuery {
    #[serde(default)]
    pub window: Option<String>,
    #[serde(default)]
    pub raw: Option<String>,
}

/// Parse `?window=7d|30d|90d` into a day-count. Unknown / missing → 30.
fn parse_window(raw: Option<&str>) -> i64 {
    match raw.unwrap_or("30d") {
        "7d" => 7,
        "90d" => 90,
        _ => 30,
    }
}

fn parse_show_raw(raw: Option<&str>) -> bool {
    matches!(raw, Some("1") | Some("true") | Some("yes"))
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ActivityQuery>,
) -> Html<String> {
    let days = parse_window(q.window.as_deref());
    let show_raw = parse_show_raw(q.raw.as_deref());
    let body = render(&state, days, show_raw).await;
    Html(
        page_with(
            "Activity",
            "activity",
            PageAssets::css_js("/static/css/activity.css", "/static/js/activity.js"),
            body,
        )
        .into_string(),
    )
}

async fn render(state: &AppState, days: i64, show_raw: bool) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let timeline = queries::fetch_activity_timeline_opts(&conn, show_raw, 2000);
    let groups = group_timeline(&timeline);

    // ── KPIs ──
    let (events_24h, events_7d, decisions_30d, most_active_source) =
        analytics::activity_kpis(&conn);

    // ── Windowed events by source (stacked area) ──
    let activity = analytics::events_by_source_window(&conn, days);
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
        "window": format!("{days}d"),
    });

    // ── Event-type breakdown (horizontal bar) ──
    let event_types = analytics::event_type_breakdown_window(&conn, days);
    let et_labels: Vec<String> = event_types.iter().map(|(k, _)| event_label(k)).collect();
    let et_values: Vec<i64> = event_types.iter().map(|(_, v)| *v).collect();
    let event_types_json = serde_json::json!({
        "labels": et_labels,
        "values": et_values,
        "window": format!("{days}d"),
    });

    // ── Decision velocity (stacked bar) ──
    let velocity = analytics::decision_velocity_30d(&conn);
    let v_dates: Vec<String> = velocity.iter().map(|(d, _, _, _)| d.clone()).collect();
    let v_applied: Vec<i64> = velocity.iter().map(|(_, a, _, _)| *a).collect();
    let v_watching: Vec<i64> = velocity.iter().map(|(_, _, w, _)| *w).collect();
    let v_rejected: Vec<i64> = velocity.iter().map(|(_, _, _, r)| *r).collect();
    let velocity_json = serde_json::json!({
        "dates": v_dates,
        "applied": v_applied,
        "watching": v_watching,
        "rejected": v_rejected,
    });

    html! {
        div.activity-page {
            div.activity-head {
                div.activity-head-left {
                    span.dot {}
                    h1 { "Activity" }
                }
                span.activity-sub {
                    (groups.len()) " groups · " (timeline.len()) " events"
                }
            }

            // KPI strip
            section.kpi-strip {
                div.kpi {
                    div.kpi-label { "Events · 24h" }
                    div.kpi-value data-ticker=(events_24h) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Events · 7d" }
                    div.kpi-value data-ticker=(events_7d) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Decisions · 30d" }
                    div.kpi-value data-ticker=(decisions_30d) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Top source · 30d" }
                    div.kpi-value.kpi-value-text { (most_active_source) }
                }
            }

            // Timeframe toggle (controls both panels below)
            div.timeframe-bar {
                span.timeframe-label { "Window" }
                (timeframe_toggle(days, show_raw))
                (raw_toggle(days, show_raw))
            }

            // Windowed sparkline + event-type breakdown
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { (format!("{days}-day events by source")) }
                        span.panel-sub { "stacked area · cli / tui / web / script / trigger / other" }
                    }
                    div #chart-activity-30d-full .chart.chart-md {}
                    (json_island("activity-30d-full", &activity_json))
                }
                section.panel {
                    header.panel-head {
                        h2 { "Event types" }
                        span.panel-sub { (format!("top 12 · last {days} days")) }
                    }
                    div #chart-event-types .chart.chart-md {}
                    (json_island("event-types", &event_types_json))
                }
            }

            // Decision velocity (full width)
            section.panel {
                header.panel-head {
                    h2 { "Decision velocity" }
                    span.panel-sub { "applied · watching · rejected per day, last 30 days" }
                }
                div #chart-velocity .chart.chart-sm {}
                (json_island("velocity", &velocity_json))
            }

            // Event log (full width, inside panel for live-wire)
            section.panel.activity-log-panel {
                header.panel-head {
                    h2 { "Event log" }
                    span.panel-sub { "append-only timeline · grouped by burst" }
                }
                div.activity-feed {
                    @if groups.is_empty() {
                        div.empty { "No activity yet." }
                    }
                    @let mut current_date: Option<String> = None;
                    @for group in &groups {
                        @let date = group.date().to_string();
                        @if current_date.as_deref() != Some(date.as_str()) {
                            @let _ = current_date = Some(date.clone());
                            div.day-header { (format_date(&date)) }
                        }
                        (render_group(group))
                    }
                }
            }
        }
    }
}

/// Render the [7D] [30D] [90D] toggle as a row of hyperlinks. The active
/// chip is marked with `timeframe-chip-active` so CSS can highlight it; every
/// chip points at `/activity?window=Xd` so default browser navigation handles
/// the round-trip (no JS required).
fn timeframe_toggle(active_days: i64, show_raw: bool) -> Markup {
    let options: [(i64, &str); 3] = [(7, "7D"), (30, "30D"), (90, "90D")];
    let raw_qs = if show_raw { "&raw=1" } else { "" };
    html! {
        div.timeframe-toggle {
            @for (days, label) in options.iter() {
                @let is_active = *days == active_days;
                @let class = if is_active { "timeframe-chip timeframe-chip-active" } else { "timeframe-chip" };
                a.(class) href=(format!("/activity?window={days}d{raw_qs}")) { (label) }
            }
        }
    }
}

/// "Show raw events" toggle — segmented control on/off.
fn raw_toggle(days: i64, show_raw: bool) -> Markup {
    html! {
        div class="seg-group" {
            a class="seg" data-active=(!show_raw)
              href=(format!("/activity?window={days}d")) { "Real events" }
            a class="seg" data-active=(show_raw)
              href=(format!("/activity?window={days}d&raw=1")) { "+ raw triggers" }
        }
    }
}

fn render_group(group: &ActivityGroup) -> Markup {
    match group {
        ActivityGroup::Single(e) => render_entry(e, false),
        ActivityGroup::Collapsed { event_type, source, first_at, last_at, entries, .. } => {
            let time_first = if first_at.len() >= 16 { &first_at[11..16] } else { "" };
            let time_last = if last_at.len() >= 16 { &last_at[11..16] } else { "" };
            let time_str = if time_first == time_last { time_first.to_string() } else { format!("{time_first}–{time_last}") };
            html! {
                details.activity-group {
                    summary.activity-group-summary {
                        span.activity-time { (time_str) }
                        span.activity-icon { (icon_for(event_type)) }
                        span.activity-count { (entries.len()) }
                        span.activity-label { (event_label(event_type)) }
                        @if source != "tui" {
                            span.activity-source { (source) }
                        }
                    }
                    div.activity-children {
                        @for entry in entries {
                            (render_entry(entry, true))
                        }
                    }
                }
            }
        }
    }
}

fn render_entry(entry: &ActivityEntry, is_child: bool) -> Markup {
    let time = if entry.occurred_at.len() >= 16 { &entry.occurred_at[11..16] } else { "" };
    let class = if is_child { "activity-row activity-child" } else { "activity-row" };
    html! {
        div.(class) {
            span.activity-time { (time) }
            (lane_chip_for(entry.lane.as_deref()))
            span.activity-icon { (icon_for(&entry.event_type)) }
            span.activity-event-label { (event_label(&entry.event_type)) }
            span.activity-subject data-marquee=(entry.subject_label) { (entry.subject_label) }
            @if let Some(g) = &entry.grade {
                span.grade-pill.(format!("grade-{}", g.to_lowercase())) { (g) }
            }
            @if entry.source != "tui" {
                span.activity-source { (entry.source) }
            }
        }
    }
}

fn icon_for(event_type: &str) -> &'static str {
    match event_type {
        "job.added" | "company.added" => "+",
        "job.deleted" | "job.pruned" | "company.deleted" => "−",
        "job.archived" | "company.archived" => "□",
        "company.unarchived" => "↻",
        "job.graded" | "company.graded" => "★",
        "job.regraded" => "↺",
        "decision.applied" => "✓",
        "decision.watching" => "◉",
        "decision.rejected" => "✗",
        "decision.interview" => "→",
        "search.ran" => "⟳",
        t if t.starts_with("raw.") => "?",
        _ => "·",
    }
}

fn event_label(event_type: &str) -> String {
    match event_type {
        "job.added" => "job added".into(),
        "job.deleted" => "job deleted".into(),
        "job.pruned" => "job pruned".into(),
        "job.archived" => "job archived".into(),
        "job.graded" => "job graded".into(),
        "job.regraded" => "job regraded".into(),
        "company.added" => "company added".into(),
        "company.deleted" => "company deleted".into(),
        "company.archived" => "company archived".into(),
        "company.unarchived" => "company unarchived".into(),
        "company.graded" => "company graded".into(),
        "decision.applied" => "applied".into(),
        "decision.watching" => "watching".into(),
        "decision.rejected" => "rejected".into(),
        "decision.interview" => "interview".into(),
        "search.ran" => "search".into(),
        "raw.job.inserted" => "raw insert (job)".into(),
        "raw.job.deleted" => "raw delete (job)".into(),
        "raw.company.inserted" => "raw insert (company)".into(),
        "raw.company.deleted" => "raw delete (company)".into(),
        other => other.to_string(),
    }
}

fn format_date(date_str: &str) -> String {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map(|d| d.format("%A, %b %-d, %Y").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}

#[derive(Debug, Deserialize)]
pub struct ToggleGroupForm {
    pub _key: String,
}

/// Placeholder for stateful group toggling (HTMX uses <details> client-side
/// for now; this endpoint exists for future persistent state).
pub async fn toggle_group(
    State(_state): State<Arc<AppState>>,
    Form(_form): Form<ToggleGroupForm>,
) -> Html<String> {
    Html(String::from("ok"))
}
