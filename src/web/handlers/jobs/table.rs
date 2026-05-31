//! Filtered-jobs table + per-row decision buttons.
//!
//! Markup is div+role-based (not `<table>`) so each row is a CSS-grid
//! container that can host an absolutely-positioned lane accent stripe behind
//! the cells. ARIA roles preserve table semantics for assistive tech.

use maud::{html, Markup};

use crate::data::lane::lane_accent_gradient;
use crate::data::models::JobRow;
use crate::web::templates::{grade_pill, lane_chip_for};

pub(super) fn render_table(jobs: &[JobRow]) -> Markup {
    html! {
        section.panel.no-pad {
            header.panel-head {
                h2 { "Filtered jobs" }
                span.panel-sub { (jobs.len()) " matching · sorted by grade" }
            }
            div.row-grid.row-grid-jobs role="table" {
                div.row-header role="row" {
                    div.row-grade role="columnheader" { "Grade" }
                    div.row-lane role="columnheader" { "Lane" }
                    div.row-title role="columnheader" { "Title" }
                    div.row-company role="columnheader" { "Company" }
                    div.row-location role="columnheader" { "Location" }
                    div.row-actions role="columnheader" { "" }
                }
                @for j in jobs {
                    div.job-row.row-clickable role="row"
                        id=(format!("job-{}", j.id))
                        data-detail=(format!("job-{}", j.id)) {
                        @if let Some(bg) = lane_accent_gradient(j.lanes.as_deref()) {
                            div.row-lane-accent style=(format!("background: {bg};")) {}
                        }
                        div.row-grade    role="cell" { (grade_pill(j.grade.as_deref())) }
                        div.row-lane     role="cell" { (lane_chip_for(j.lanes.as_deref())) }
                        div.row-title    role="cell" { span.row-title-text data-marquee="" { (j.title) } }
                        div.row-company  role="cell" { (j.company_name) }
                        div.row-location role="cell" { (j.location.as_deref().unwrap_or("—")) }
                        div.row-actions  role="cell" {
                            (decision_buttons(j.id, j.decision.as_deref(), &j.url))
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn decision_buttons(id: i64, current: Option<&str>, url: &str) -> Markup {
    // Each button: if it represents the current decision, render in its "active"
    // tone — green (Applied), amber (Watching), red (Rejected) — with past-tense
    // label. Otherwise render in the dormant tone with imperative label.
    // The state IS the button colour + label; no trailing "current" text needed.
    let apply_class = if current == Some("applied") {
        "btn btn-applied"
    } else {
        "btn btn-apply"
    };
    let apply_label = if current == Some("applied") { "Applied" } else { "Apply" };
    let watch_class = if current == Some("watching") {
        "btn btn-watching"
    } else {
        "btn"
    };
    let watch_label = if current == Some("watching") { "Watching" } else { "Watch" };
    let reject_class = if current == Some("rejected") {
        "btn btn-rejected"
    } else {
        "btn"
    };
    let reject_label = if current == Some("rejected") { "Rejected" } else { "Reject" };
    html! {
        div.decision-buttons {
            a class=(apply_class) href=(url) target="_blank"
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"applied"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                (apply_label)
            }
            button class=(watch_class)
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"watching"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                (watch_label)
            }
            button class=(reject_class)
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"rejected"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                (reject_label)
            }
        }
    }
}
