use maud::{html, Markup};

use crate::data::lane::{all_lanes, lane_accent_gradient};
use crate::data::models::CompanyRow;
use crate::web::templates::{grade_pill, lane_chip};

/// Renders the filterable companies table at the bottom of the page.
///
/// Div+role grid markup (not `<table>`) so each row can carry an absolutely-
/// positioned lane accent stripe. ARIA roles preserve semantics.
pub(super) fn render_table(companies: &[CompanyRow], total_unfiltered: i64) -> Markup {
    html! {
        section.panel.no-pad {
            header.panel-head {
                h2 { "Companies" }
                span.panel-sub { (companies.len()) " of " (total_unfiltered) }
            }
            div.row-grid.row-grid-companies role="table" {
                div.row-header role="row" {
                    div.row-grade role="columnheader" { "Grade" }
                    div.row-lane  role="columnheader" { "Lanes" }
                    div.row-title role="columnheader" { "Company" }
                    div.row-status role="columnheader" { "Status" }
                    div.row-jobs  role="columnheader" { "Jobs" }
                    div.row-ats   role="columnheader" { "ATS" }
                }
                @for c in companies {
                    div.company-row.row-clickable role="row" data-detail=(format!("co-{}", c.id)) {
                        @if let Some(bg) = lane_accent_gradient(c.lanes.as_deref()) {
                            div.row-lane-accent style=(format!("background: {bg};")) {}
                        }
                        div.row-grade role="cell" { (grade_pill(c.grade.as_deref())) }
                        div.row-lane  role="cell" {
                            div.lane-chip-list {
                                @for lk in all_lanes(c.lanes.as_deref()).iter() {
                                    (lane_chip(lk))
                                }
                            }
                        }
                        div.row-title role="cell" {
                            a.row-title-text data-marquee=(c.name) href=(c.website) target="_blank" { (c.name) }
                        }
                        div.row-status role="cell" {
                            @let status_class = match c.status.as_str() {
                                "resolved" => "status-pill status-resolved",
                                "bespoke" => "status-pill status-bespoke",
                                "potential" => "status-pill status-potential",
                                "archived" => "status-pill status-archived",
                                _ => "status-pill",
                            };
                            span.(status_class) { (c.status) }
                        }
                        div.row-jobs role="cell" {
                            (c.job_count)
                            @if c.fit_count > 0 {
                                span.fit-marker { " · " (c.fit_count) "✓" }
                            }
                        }
                        // Bespoke companies have no ATS by definition — render
                        // an em dash rather than a noisy "BESPOKE" pill.
                        div.row-ats role="cell" {
                            @if c.status == "bespoke" {
                                span.row-ats-empty { "—" }
                            } @else {
                                (c.ats_provider.as_deref().unwrap_or("—"))
                            }
                        }
                    }
                }
            }
        }
    }
}
