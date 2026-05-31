use axum::extract::{Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use std::sync::Arc;

use crate::data::queries;
use crate::web::templates::{json_island, lane_legend, page_with, PageAssets};
use crate::web::AppState;

use super::charts;
use super::filters::{filter_companies, grade_color, render_filter_strip};
use super::table::render_table;

pub use super::filters::CompaniesQuery;

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CompaniesQuery>,
) -> Html<String> {
    let body = render(&state, &q).await;
    Html(
        page_with(
            "Companies",
            "companies",
            PageAssets::css_js("/static/css/companies.css", "/static/js/companies.js"),
            body,
        )
        .into_string(),
    )
}

async fn render(state: &AppState, q: &CompaniesQuery) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    let all_companies = queries::fetch_companies(&conn, false);
    let total_unfiltered = all_companies.len() as i64;
    let companies = filter_companies(&all_companies, q);

    // ── KPI counts (over filtered set) ──
    let kpis = charts::kpis(&companies);

    // ── Companies × lane donut (filtered) ──
    let companies_lane_json = charts::companies_lane_json(&companies);

    // ── Grade distribution (filtered) ──
    let (grade_dist, grade_max) = charts::grade_distribution(&companies);

    // ── ATS / job-board health (filtered) ──
    let ats_json = charts::ats_health_json(&companies);

    // ── Geography (filtered) ──
    let geography_json = charts::geography_json(&companies);

    // ── Sponsorship split (filtered) ──
    let (sponsorship_rows, sponsorship_max) = charts::sponsorship_rows(&companies);

    // ── Grade freshness (filtered) ──
    let (freshness, freshness_max) = charts::freshness_rows(&conn, &companies);

    // ── Top companies by job count (filtered) ──
    let (top_companies, top_max) = charts::top_companies(&companies);

    // ── Discovered timeline (filtered) ──
    let discovered_json = charts::discovered_json(&conn, &companies);

    let filter_strip_markup = render_filter_strip(q, kpis.total_companies, total_unfiltered);

    html! {
        div.companies-page {
            // Lane legend
            (lane_legend("companies"))

            // Filter strip
            (filter_strip_markup)

            // KPI strip
            section.kpi-strip {
                div.kpi {
                    div.kpi-label { "Companies" }
                    div.kpi-value data-ticker=(kpis.total_companies) { "0" }
                }
                div.kpi {
                    div.kpi-label { "S-tier" }
                    div.kpi-value data-ticker=(kpis.s_tier) { "0" }
                }
                div.kpi {
                    div.kpi-label { "A-tier" }
                    div.kpi-value data-ticker=(kpis.a_tier) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Bespoke" }
                    div.kpi-value data-ticker=(kpis.bespoke) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Potential" }
                    div.kpi-value data-ticker=(kpis.potential) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Sponsors UK" }
                    div.kpi-value data-ticker=(kpis.sponsors_yes) { "0" }
                }
            }

            // Lane donut + Grade distribution
            div.grid-2 {
                section.panel {
                    header.panel-head {
                        h2 { "Companies × lane" }
                        span.panel-sub { "filtered universe by lane" }
                    }
                    div #chart-companies-lane-large .chart.chart-donut {}
                    (json_island("companies-lane-large", &companies_lane_json))
                }
                section.panel.no-pad {
                    header.panel-head {
                        h2 { "Grade distribution" }
                        span.panel-sub { "companies × grade · filtered" }
                    }
                    div.bar-list {
                        @for (g, n) in &grade_dist {
                            @let pct = if grade_max > 0 { (n * 100 / grade_max) as i32 } else { 0 };
                            div.bar-list-row {
                                span.bar-list-label { (g) }
                                div.bar-list-bar {
                                    div.bar-list-fill
                                        style=(format!("--target-width: {pct}%; background: {}; width: {pct}%", grade_color(g))) {}
                                }
                                span.bar-list-num { (n) }
                            }
                        }
                    }
                }
            }

            // ATS / job-board health (full width)
            section.panel {
                header.panel-head {
                    h2 { "ATS / job-board health" }
                    span.panel-sub { "filtered companies per provider + bespoke + potential" }
                }
                div #chart-ats-health-companies .chart.chart-md {}
                (json_island("ats-health-companies", &ats_json))
            }

            // Geography + Sponsorship + Freshness
            div.grid-3 {
                section.panel {
                    header.panel-head {
                        h2 { "Geography" }
                        span.panel-sub { "filtered companies by region bucket" }
                    }
                    div #chart-geography .chart.chart-sm {}
                    (json_island("geography", &geography_json))
                }
                section.panel.no-pad {
                    header.panel-head {
                        h2 { "Sponsorship" }
                        span.panel-sub { "UK Skilled Worker visa support" }
                    }
                    div.bar-list {
                        @for (label, n, color) in &sponsorship_rows {
                            @let pct = (*n * 100 / sponsorship_max) as i32;
                            div.bar-list-row {
                                span.bar-list-label { (label) }
                                div.bar-list-bar {
                                    div.bar-list-fill
                                        style=(format!("--target-width: {pct}%; background: {color}; width: {pct}%")) {}
                                }
                                span.bar-list-num { (n) }
                            }
                        }
                    }
                }
                section.panel.no-pad {
                    header.panel-head {
                        h2 { "Grade freshness" }
                        span.panel-sub { "since grading run" }
                    }
                    div.bar-list {
                        @for (bucket, n) in &freshness {
                            @let pct = (n * 100 / freshness_max) as i32;
                            @let color = if bucket == "ungraded" { "var(--text-5)" } else { "var(--accent)" };
                            div.bar-list-row {
                                span.bar-list-label { (bucket) }
                                div.bar-list-bar {
                                    div.bar-list-fill
                                        style=(format!("--target-width: {pct}%; background: {color}; width: {pct}%")) {}
                                }
                                span.bar-list-num { (n) }
                            }
                        }
                    }
                }
            }

            // Top companies + Discovered timeline
            div.grid-2 {
                section.panel.no-pad {
                    header.panel-head {
                        h2 { "Top companies by job count" }
                        span.panel-sub { "filtered · top 10" }
                    }
                    div.top-list {
                        @if top_companies.is_empty() {
                            div.empty { "No companies with active jobs." }
                        }
                        @for (i, (cid, name, total, strong)) in top_companies.iter().enumerate() {
                            @let pct = (total * 100 / top_max) as i32;
                            // Detail-drawer URL — the drawer system auto-opens
                            // from `?detail=co-<id>` when its JS is present;
                            // otherwise the page renders normally with the
                            // param dangling, preserving the click affordance.
                            a.top-list-row.row-clickable
                                href=(format!("/companies?detail=co-{cid}"))
                                title=(format!("Open {name} details")) {
                                span.top-list-rank { (i + 1) }
                                span.top-list-name data-marquee=(name) { (name) }
                                div.top-list-bar {
                                    div.top-list-bar-fill
                                        style=(format!("--target-width: {pct}%; width: {pct}%")) {}
                                }
                                span.top-list-num {
                                    (total)
                                    @if *strong > 0 {
                                        span.fit-marker { " · " (strong) "✓" }
                                    }
                                }
                            }
                        }
                    }
                }
                section.panel {
                    header.panel-head {
                        h2 { "Recently discovered" }
                        span.panel-sub { "filtered companies per day · last 30 days" }
                    }
                    div #chart-discovered .chart.chart-sm {}
                    (json_island("discovered", &discovered_json))
                }
            }

            // Filterable companies table
            (render_table(&companies, total_unfiltered))
        }
    }
}
