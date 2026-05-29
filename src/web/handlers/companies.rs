use axum::extract::{Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Arc;

use crate::data::lane::{all_lanes, lane_hex, lane_label, LANE_KEYS};
use crate::data::queries;
use crate::web::templates::{grade_pill, lane_chip, lane_legend, page as chrome};
use crate::web::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct CompaniesQuery {
    pub layout: Option<String>,
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CompaniesQuery>,
) -> Html<String> {
    let layout_lanes = q.layout.as_deref() == Some("lanes");
    let body = render(&state, layout_lanes).await;
    Html(chrome("Companies", "companies", body).into_string())
}

async fn render(state: &AppState, layout_lanes: bool) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let companies = queries::fetch_companies(&conn, false);

    html! {
        div.companies-page {
            (lane_legend())

            div.layout-toggle {
                a.layout-tab[layout_lanes == false] href="/companies" { "Classic" }
                a.layout-tab[layout_lanes == true] href="/companies?layout=lanes" { "Lanes" }
            }

            @if layout_lanes {
                (render_lanes_kanban(&companies))
            } @else {
                (render_classic(&companies))
            }
        }
    }
}

fn render_classic(companies: &[crate::data::models::CompanyRow]) -> Markup {
    html! {
        section.panel.no-pad {
            table.company-table {
                thead {
                    tr {
                        th.col-grade { "Grade" }
                        th.col-lane  { "Lanes" }
                        th.col-co    { "Company" }
                        th.col-status { "Status" }
                        th.col-jobs  { "Jobs" }
                        th.col-ats   { "ATS" }
                    }
                }
                tbody {
                    @for c in companies {
                        tr.company-row {
                            td.col-grade { (grade_pill(c.grade.as_deref())) }
                            td.col-lane {
                                div.lane-chip-list {
                                    @for lk in all_lanes(c.lanes.as_deref()).iter() {
                                        (lane_chip(lk))
                                    }
                                }
                            }
                            td.col-co {
                                a href=(c.website) target="_blank" { (c.name) }
                            }
                            td.col-status { span.status-pill { (c.status) } }
                            td.col-jobs {
                                (c.job_count)
                                @if c.fit_count > 0 {
                                    span.fit-marker { " · " (c.fit_count) "✓" }
                                }
                            }
                            td.col-ats { (c.ats_provider.as_deref().unwrap_or("—")) }
                        }
                    }
                }
            }
        }
    }
}

fn render_lanes_kanban(companies: &[crate::data::models::CompanyRow]) -> Markup {
    // Bucket by lane.
    let mut per_lane: std::collections::HashMap<&str, Vec<&crate::data::models::CompanyRow>> =
        std::collections::HashMap::new();
    for c in companies {
        let lanes = all_lanes(c.lanes.as_deref());
        for lk in &lanes {
            if let Some(key) = LANE_KEYS.iter().find(|k| **k == lk) {
                per_lane.entry(*key).or_default().push(c);
            }
        }
    }

    html! {
        section.lane-kanban {
            @for key in LANE_KEYS.iter() {
                @let entries = per_lane.get(key).cloned().unwrap_or_default();
                div.lane-column style=(format!("--lane-color: {}", lane_hex(key))) {
                    header.lane-column-head {
                        span.lane-column-name { (lane_label(key)) }
                        span.lane-column-count { (entries.len()) }
                    }
                    div.lane-column-body {
                        @if entries.is_empty() {
                            div.empty { "—" }
                        }
                        @for c in &entries {
                            @let multi = all_lanes(c.lanes.as_deref()).len() > 1;
                            div.company-card {
                                div.company-card-row {
                                    (grade_pill(c.grade.as_deref()))
                                    div.company-card-name { (c.name) }
                                    @if multi { span.multi-marker title="multi-lane company" { "✦" } }
                                }
                                @if c.job_count > 0 {
                                    div.company-card-meta {
                                        (c.job_count) " jobs"
                                        @if c.fit_count > 0 { span.fit-marker { " · " (c.fit_count) " strong" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
