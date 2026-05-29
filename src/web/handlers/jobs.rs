use axum::extract::{Form, Path, Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::lane::LANE_KEYS;
use crate::data::models::{JobFilters, SortMode};
use crate::data::queries;
use crate::web::templates::{grade_pill, lane_chip_for, lane_legend, page as chrome};
use crate::web::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct JobsQuery {
    pub lane: Option<String>,
    pub grade: Option<String>,
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<JobsQuery>,
) -> Html<String> {
    let body = render(&state, &q).await;
    Html(chrome("Jobs", "jobs", body).into_string())
}

async fn render(state: &AppState, q: &JobsQuery) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let mut filters = JobFilters::default();
    if let Some(lane) = &q.lane {
        if !lane.is_empty() { filters.lanes.insert(lane.clone()); }
    }
    if let Some(grade) = &q.grade {
        if !grade.is_empty() { filters.grades.insert(grade.clone()); }
    }

    let jobs = queries::fetch_jobs(&conn, None, &filters, SortMode::ByGrade);

    html! {
        div.jobs-page {
            (lane_legend())

            div.filter-bar {
                form method="get" action="/jobs" class="filter-form" {
                    label { "Lane: "
                        select name="lane" onchange="this.form.submit()" {
                            option value="" selected=(q.lane.as_deref().map(|s| s.is_empty()).unwrap_or(true)) { "All" }
                            @for key in LANE_KEYS.iter() {
                                option value=(key) selected=(q.lane.as_deref() == Some(*key)) { (key) }
                            }
                        }
                    }
                    label { "Grade: "
                        select name="grade" onchange="this.form.submit()" {
                            option value="" selected=(q.grade.as_deref().map(|s| s.is_empty()).unwrap_or(true)) { "All" }
                            @for g in ["SS", "S", "A", "B", "C", "F"].iter() {
                                option value=(g) selected=(q.grade.as_deref() == Some(*g)) { (g) }
                            }
                        }
                    }
                    span.filter-summary { (jobs.len()) " jobs" }
                }
            }

            section.panel.no-pad {
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
                        @for j in &jobs {
                            tr.job-row id=(format!("job-{}", j.id)) {
                                td.col-grade { (grade_pill(j.grade.as_deref())) }
                                td.col-lane  { (lane_chip_for(j.lanes.as_deref())) }
                                td.col-title { div.job-title { (j.title) } }
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

        // Pull the job's url so the new buttons block knows where Apply links.
        let url: String = conn.query_row(
            "SELECT url FROM jobs WHERE id = ?1",
            rusqlite::params![id], |r| r.get(0)).unwrap_or_default();
        return Html(decision_buttons(id, Some(decision), &url).into_string());
    }

    Html(html! { div.error { "DB error" } }.into_string())
}

fn decision_buttons(id: i64, current: Option<&str>, url: &str) -> Markup {
    let active = |kind: &str| if current == Some(kind) { "btn btn-active" } else { "btn" };
    html! {
        div.decision-buttons {
            a.btn.btn-apply href=(url) target="_blank"
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"applied"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Apply"
            }
            button.(active("watching"))
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"watching"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Watch"
            }
            button.(active("rejected"))
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"rejected"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Reject"
            }
            @if let Some(d) = current {
                span.decision-current { (d) }
            }
        }
    }
}
