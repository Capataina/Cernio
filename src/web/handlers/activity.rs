use axum::extract::{Form, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::activity::{group_timeline, ActivityGroup};
use crate::data::models::ActivityEntry;
use crate::data::queries;
use crate::web::templates::{lane_chip_for, page as chrome};
use crate::web::AppState;

pub async fn page(State(state): State<Arc<AppState>>) -> Html<String> {
    let body = render(&state).await;
    Html(chrome("Activity", "activity", body).into_string())
}

async fn render(state: &AppState) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let timeline = queries::fetch_activity_timeline(&conn);
    let groups = group_timeline(&timeline);

    html! {
        div.activity-page {
            div.activity-head {
                h1 { "Activity" }
                span.activity-sub { (groups.len()) " groups · " (timeline.len()) " events" }
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
            span.activity-subject { (entry.subject_label) }
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
