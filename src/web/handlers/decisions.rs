//! /decisions tab — a focused view of your Watching / Applied / Interview /
//! Rejected job pipeline. Companion to the Dashboard, Jobs, and Activity
//! tabs; gives action a home of its own.
//!
//! Layout (2026-05-31 redesign):
//!   1. Decision funnel nav  — replaces the old KPI strip + segmented tabs.
//!      A single horizontal flow bar of stages (Watching → Applied → Interview
//!      → Rejected) plus an "All" pill on the right. Each segment is both
//!      stat-card and filter link; the active stage glows, inactive stages
//!      mute, connector arrows hint the flow direction.
//!   2. Next actions pane    — surfaces jobs currently in `watching` so the
//!      user is prompted to actually apply. Shows "watching since N days".
//!   3. All decisions list   — flat list grouped by day-headers (Today /
//!      Yesterday / weekday + date). Each row carries the lane-accent
//!      gradient stripe used elsewhere in the codebase.

use axum::extract::{Query, State};
use chrono::Datelike;
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::analytics;
use crate::data::lane::lane_accent_gradient;
use crate::web::templates::{
    grade_pill, lane_chip_for, lane_legend, page_with, PageAssets,
};
use crate::web::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct DecisionsQuery {
    #[serde(default)]
    pub kind: Option<String>,
}

/// Funnel stages in flow order. `all` is rendered separately as a trailing
/// pill — it is the no-filter view, not a pipeline stage.
const FUNNEL_STAGES: &[(&str, &str, &str)] = &[
    // (key, label, decision-kind colour token suffix used in CSS)
    ("watching",  "Watching",  "watching"),
    ("applied",   "Applied",   "applied"),
    ("interview", "Interview", "interview"),
    ("rejected",  "Rejected",  "rejected"),
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

    // Width-flex for each segment: proportional to count, with a floor so
    // zero-count stages remain visibly clickable. The "All" pill is fixed.
    let max_count = [watching, applied, interview, rejected]
        .into_iter()
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    // Pull the full watching list (independent of `kind` filter) so the
    // Next-Actions pane stays useful even when the user is browsing
    // "Applied" or "Rejected".
    let watching_jobs = analytics::decisions_filtered(&conn, "watching");

    html! {
        div.decisions-page {
            (lane_legend("decisions"))

            // ── Funnel nav ─────────────────────────────────────────────
            // Replaces the old KPI strip + segmented tab row in one
            // component. Stages are clickable filter anchors; the active
            // stage gets full saturation + glow, others mute to ~40%.
            nav.decision-funnel-nav aria-label="Decision pipeline" {
                @for (i, (k, label, kind_token)) in FUNNEL_STAGES.iter().enumerate() {
                    @let count = match *k {
                        "watching"  => watching,
                        "applied"   => applied,
                        "interview" => interview,
                        "rejected"  => rejected,
                        _ => 0,
                    };
                    @let active = kind == *k;
                    // Flex grow ratio: 0.6 floor + (count / max) so non-zero
                    // stages dominate visually but zero stages remain visible.
                    @let grow = 0.6 + (count as f32 / max_count) * 1.6;
                    @let cls = if active {
                        "decision-funnel-stage is-active"
                    } else {
                        "decision-funnel-stage"
                    };
                    a class=(cls)
                      style=(format!("flex-grow: {:.3};", grow))
                      href=(format!("/decisions?kind={k}"))
                      data-kind=(kind_token)
                      title=(format!("Click to filter — {label}: {count}")) {
                        span.decision-funnel-label { (label) }
                        span.decision-funnel-count data-ticker=(count) { (count) }
                    }
                    @if i + 1 < FUNNEL_STAGES.len() {
                        span.decision-funnel-arrow aria-hidden="true" { "→" }
                    }
                }
                // Trailing "All" pill — the no-filter view.
                @let all_active = kind == "all";
                @let all_cls = if all_active {
                    "decision-funnel-all is-active"
                } else {
                    "decision-funnel-all"
                };
                a class=(all_cls) href="/decisions?kind=all" title="Show every decision" {
                    span.decision-funnel-all-label { "All" }
                    span.decision-funnel-all-count data-ticker=(total) { (total) }
                }
            }

            // ── Next actions pane ─────────────────────────────────────
            // Watching = decided, not yet acted on. This pane is the
            // explicit prompt to convert intent into application.
            section.panel.no-pad.next-actions-panel {
                header.panel-head {
                    h2 { "Next actions" }
                    span.panel-sub {
                        @if watching_jobs.is_empty() {
                            "no jobs currently watching"
                        } @else {
                            (format!("{} watching — apply or let go", watching_jobs.len()))
                        }
                    }
                }
                @if watching_jobs.is_empty() {
                    div.empty {
                        "Nothing on the watch list. "
                        a href="/jobs?decision=none" { "Browse jobs →" }
                    }
                } @else {
                    div.next-actions-list {
                        @for (decided_at, _decision, title, company, lanes_json, grade, url, id)
                            in watching_jobs.iter().take(6)
                        {
                            @let days = days_since(decided_at);
                            @let urgency_class = urgency_class_for(days);
                            div class=(format!("next-action-row {urgency_class}"))
                                data-detail=(format!("job-{id}")) {
                                @if let Some(bg) = lane_accent_gradient(lanes_json.as_deref()) {
                                    div.row-lane-accent style=(format!("background: {bg};")) {}
                                }
                                div.next-action-meta {
                                    (grade_pill(grade.as_deref()))
                                    (lane_chip_for(lanes_json.as_deref()))
                                }
                                div.next-action-text {
                                    div.next-action-title { (title) }
                                    div.next-action-co { (company) }
                                }
                                div.next-action-since {
                                    span.next-action-since-num { (days) }
                                    span.next-action-since-label {
                                        @if days == 1 { "day watching" } @else { "days watching" }
                                    }
                                }
                                a.next-action-open href=(url) target="_blank" title="open posting"
                                    onclick="event.stopPropagation();" { "↗" }
                            }
                        }
                    }
                    @if watching_jobs.len() > 6 {
                        a.next-actions-more href="/decisions?kind=watching" {
                            "See all " (watching_jobs.len()) " watching →"
                        }
                    }
                }
            }

            // ── All decisions list (grouped by day) ────────────────────
            section.panel.no-pad {
                header.panel-head {
                    h2 {
                        @if kind == "all" { "All decisions" }
                        @else { (capitalize(kind)) " decisions" }
                    }
                    span.panel-sub { (decisions.len()) " records" }
                }
                @if decisions.is_empty() {
                    div.empty {
                        "No decisions in this view. "
                        a href="/jobs" { "Go to Jobs →" }
                    }
                } @else {
                    // Pre-compute day boundaries so the loop only renders;
                    // Maud's @let is a binding, not a mutable cell.
                    @let day_marks: Vec<bool> = {
                        let mut prev = String::new();
                        decisions.iter().map(|d| {
                            let key = day_bucket_key(&d.0);
                            let is_new = key != prev;
                            prev = key;
                            is_new
                        }).collect()
                    };
                    div.decisions-list {
                        @for (i, (decided_at, decision, title, company, lanes_json, grade, url, id)) in decisions.iter().enumerate() {
                            @if day_marks[i] {
                                div.day-header { (day_bucket_label(decided_at)) }
                            }
                            div.decision-list-row data-detail=(format!("job-{id}")) {
                                @if let Some(bg) = lane_accent_gradient(lanes_json.as_deref()) {
                                    div.row-lane-accent style=(format!("background: {bg};")) {}
                                }
                                span.decision-when { (short_time(decided_at)) }
                                span.decision-kind data-kind=(decision) { (decision.as_str()) }
                                (lane_chip_for(lanes_json.as_deref()))
                                (grade_pill(grade.as_deref()))
                                div.decision-text {
                                    div.decision-title { (title) }
                                    div.decision-co { (company) }
                                }
                                a.decision-open href=(url) target="_blank" title="open posting"
                                    onclick="event.stopPropagation();" { "↗" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render a relative-time short string ("just now", "12m", "3h", "2d").
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

/// Stable bucket key for day-grouping (YYYY-MM-DD in UTC). Falls back to the
/// raw timestamp when parsing fails so an unparseable row still groups with
/// itself instead of merging into a neighbour.
fn day_bucket_key(ts: &str) -> String {
    parse_ts(ts)
        .map(|dt| dt.date_naive().to_string())
        .unwrap_or_else(|| ts.to_string())
}

/// Human label for the day-header: "Today" / "Yesterday" / "Mon 28 May" /
/// "Sun 27 May 2024" when older than the current year.
fn day_bucket_label(ts: &str) -> String {
    let Some(dt) = parse_ts(ts) else { return ts.to_string(); };
    let today = chrono::Utc::now().date_naive();
    let date = dt.date_naive();
    let days = (today - date).num_days();
    match days {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        2..=6 => date.format("%a %-d %b").to_string(),
        _ if date.year_ce() == today.year_ce() => date.format("%a %-d %b").to_string(),
        _ => date.format("%a %-d %b %Y").to_string(),
    }
}

/// Days elapsed since a timestamp (UTC, floor). Returns 0 for future / same-day.
fn days_since(ts: &str) -> i64 {
    let Some(dt) = parse_ts(ts) else { return 0; };
    let delta = chrono::Utc::now().signed_duration_since(dt);
    delta.num_days().max(0)
}

/// Map watching age to a visual urgency class. 0–2d calm, 3–6d nudge,
/// 7d+ stale (the longer you watch without applying, the louder the row).
fn urgency_class_for(days: i64) -> &'static str {
    match days {
        0..=2 => "urgency-fresh",
        3..=6 => "urgency-warm",
        _ => "urgency-stale",
    }
}

fn parse_ts(ts: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                .map(|n| n.and_utc())
                .ok()
        })
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

