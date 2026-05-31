//! Alternate "lane-columned" rendering of the filtered companies list.
//!
//! Mirrors `jobs/lanes_view.rs`: groups companies by primary lane and renders
//! an 8-column grid (one column per `LANE_KEYS` entry). Each column scrolls
//! independently so column heights become a visual indicator of relative
//! universe density. Cards are uniform height so that signal stays honest.
//!
//! Companies whose `lanes` field is empty or whose primary lane is not in
//! `LANE_KEYS` collect into a separate "unassigned" column appended to the
//! right when non-empty; the eight canonical columns always render even if
//! empty so column positions remain stable across filter states.

use maud::{html, Markup};

use crate::data::lane::{lane_hex, lane_label, primary_lane, LANE_KEYS};
use crate::data::models::CompanyRow;
use crate::web::templates::grade_pill;

pub(super) fn render_lanes_view(companies: &[CompanyRow]) -> Markup {
    // Bucket companies by their primary lane key.
    let mut buckets: Vec<(&'static str, Vec<&CompanyRow>)> =
        LANE_KEYS.iter().map(|k| (*k, Vec::new())).collect();
    let mut unassigned: Vec<&CompanyRow> = Vec::new();

    for c in companies {
        match primary_lane(c.lanes.as_deref()) {
            Some(key) => {
                if let Some(slot) = buckets.iter_mut().find(|(k, _)| *k == key.as_str()) {
                    slot.1.push(c);
                } else {
                    unassigned.push(c);
                }
            }
            None => unassigned.push(c),
        }
    }

    html! {
        section.panel.no-pad.jobs-lanes-panel {
            header.panel-head {
                h2 { "Companies by lane" }
                span.panel-sub { (companies.len()) " matching · grouped by primary lane · scroll each column" }
            }
            div.jobs-lanes-view {
                @for (key, items) in &buckets {
                    (render_column(key, items))
                }
                @if !unassigned.is_empty() {
                    (render_unassigned_column(&unassigned))
                }
            }
        }
    }
}

fn render_column(key: &str, items: &[&CompanyRow]) -> Markup {
    let label = lane_label(key);
    let hex = lane_hex(key);
    html! {
        div.jobs-lane-column style=(format!("--lane-color: {hex}")) {
            div.jobs-lane-column-head {
                span.lane-chip.lane-chip-lg title=(label) style=(format!("--lane-color: {hex}")) {
                    (label)
                }
                span.jobs-lane-count { (items.len()) }
            }
            div.jobs-lane-column-body {
                @if items.is_empty() {
                    div.jobs-lane-empty { "— empty —" }
                }
                @for c in items {
                    (render_card(c, hex))
                }
            }
        }
    }
}

fn render_unassigned_column(items: &[&CompanyRow]) -> Markup {
    html! {
        div.jobs-lane-column.jobs-lane-column-unassigned style="--lane-color: #888888" {
            div.jobs-lane-column-head {
                span.lane-chip.lane-chip-lg.lane-none title="No primary lane" { "Unassigned" }
                span.jobs-lane-count { (items.len()) }
            }
            div.jobs-lane-column-body {
                @for c in items {
                    (render_card(c, "#888888"))
                }
            }
        }
    }
}

fn status_pill(status: &str) -> Markup {
    // Compact status indicator. Three canonical states map to three colour
    // codes via class; anything else (defensive) falls through to "plain".
    let class = match status {
        "resolved" => "co-status-pill co-status-resolved",
        "bespoke" => "co-status-pill co-status-bespoke",
        "potential" => "co-status-pill co-status-potential",
        _ => "co-status-pill",
    };
    html! { span class=(class) title=(format!("Status: {status}")) { (status) } }
}

fn render_card(c: &CompanyRow, lane_hex_value: &str) -> Markup {
    // Meta row shows job activity (volume + fit count) and the ATS provider
    // so a glance answers "is this company live, and where do its jobs live?".
    // Bespoke companies have no provider slug — show em-dash to keep cards
    // visually uniform rather than collapsing the column.
    let provider = c
        .ats_provider
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("—");
    let job_label = if c.job_count == 1 { "job" } else { "jobs" };
    html! {
        div.lane-card.row-clickable
            id=(format!("lane-card-company-{}", c.id))
            data-detail=(format!("company-{}", c.id))
            style=(format!("--lane-color: {lane_hex_value}")) {
            div.lane-card-head {
                (grade_pill(c.grade.as_deref()))
                (status_pill(&c.status))
                span.lane-card-title title=(c.name) { (c.name) }
            }
            div.lane-card-meta {
                span.lane-card-jobs {
                    (c.job_count) " " (job_label)
                    @if c.fit_count > 0 {
                        " · " (c.fit_count) " ✓"
                    }
                }
                span.lane-card-sep { " · " }
                span.lane-card-ats { (provider) }
            }
        }
    }
}
