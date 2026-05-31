use maud::{html, Markup};

use crate::data::lane::all_lanes;
use crate::data::models::CompanyRow;
use crate::web::templates::{grade_pill, lane_chip};

/// Renders the filterable companies table at the bottom of the page.
pub(super) fn render_table(companies: &[CompanyRow], total_unfiltered: i64) -> Markup {
    html! {
        section.panel.no-pad {
            header.panel-head {
                h2 { "Companies" }
                span.panel-sub { (companies.len()) " of " (total_unfiltered) }
            }
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
                        tr.company-row.row-clickable data-detail=(format!("co-{}", c.id)) {
                            td.col-grade { (grade_pill(c.grade.as_deref())) }
                            td.col-lane {
                                div.lane-chip-list {
                                    @for lk in all_lanes(c.lanes.as_deref()).iter() {
                                        (lane_chip(lk))
                                    }
                                }
                            }
                            td.col-co {
                                a data-marquee=(c.name) href=(c.website) target="_blank" { (c.name) }
                            }
                            td.col-status {
                                @let status_class = match c.status.as_str() {
                                    "resolved" => "status-pill status-resolved",
                                    "bespoke" => "status-pill status-bespoke",
                                    "potential" => "status-pill status-potential",
                                    "archived" => "status-pill status-archived",
                                    _ => "status-pill",
                                };
                                span.(status_class) { (c.status) }
                            }
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
