//! Filtered-jobs table + per-row decision buttons.

use maud::{html, Markup};

use crate::data::models::JobRow;
use crate::web::templates::{grade_pill, lane_chip_for};

pub(super) fn render_table(jobs: &[JobRow]) -> Markup {
    html! {
        section.panel.no-pad {
            header.panel-head {
                h2 { "Filtered jobs" }
                span.panel-sub { (jobs.len()) " matching · sorted by grade" }
            }
            table.job-table {
                thead {
                    tr {
                        th.col-grade { "Grade" }
                        th.col-lane  { "Lane" }
                        th.col-title { "Title" }
                        th.col-co    { "Company" }
                        th.col-loc   { "Location" }
                        th.col-actions { "" }
                    }
                }
                tbody {
                    @for j in jobs {
                        tr.job-row.row-clickable id=(format!("job-{}", j.id))
                            data-detail=(format!("job-{}", j.id)) {
                            td.col-grade { (grade_pill(j.grade.as_deref())) }
                            td.col-lane  { (lane_chip_for(j.lanes.as_deref())) }
                            td.col-title { div.job-title data-marquee="" { (j.title) } }
                            td.col-co    { (j.company_name) }
                            td.col-loc   { (j.location.as_deref().unwrap_or("—")) }
                            td.col-actions {
                                (decision_buttons(j.id, j.decision.as_deref(), &j.url))
                            }
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
