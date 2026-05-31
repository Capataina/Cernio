//! Alternate "lane-columned" rendering of the filtered jobs list.
//!
//! Groups jobs by primary lane and renders an 8-column grid (one column per
//! `LANE_KEYS` entry). Each column scrolls independently so column heights
//! become a visual indicator of relative volume. Cards are uniform height to
//! preserve that comparison.
//!
//! Jobs whose `lanes` field is empty or whose primary lane is not in
//! `LANE_KEYS` are collected into a separate "unassigned" column appended to
//! the right when non-empty; the eight canonical columns always render even
//! if empty so column positions stay stable across filter states.

use maud::{html, Markup};

use crate::data::lane::{lane_hex, lane_label, primary_lane, LANE_KEYS};
use crate::data::models::JobRow;
use crate::web::templates::grade_pill;

pub(super) fn render_lanes_view(jobs: &[JobRow]) -> Markup {
    // Bucket jobs by their primary lane key.
    let mut buckets: Vec<(&'static str, Vec<&JobRow>)> =
        LANE_KEYS.iter().map(|k| (*k, Vec::new())).collect();
    let mut unassigned: Vec<&JobRow> = Vec::new();

    for j in jobs {
        match primary_lane(j.lanes.as_deref()) {
            Some(key) => {
                if let Some(slot) = buckets.iter_mut().find(|(k, _)| *k == key.as_str()) {
                    slot.1.push(j);
                } else {
                    unassigned.push(j);
                }
            }
            None => unassigned.push(j),
        }
    }

    html! {
        section.panel.no-pad.jobs-lanes-panel {
            header.panel-head {
                h2 { "Jobs by lane" }
                span.panel-sub { (jobs.len()) " matching · grouped by primary lane · scroll each column" }
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

fn render_column(key: &str, items: &[&JobRow]) -> Markup {
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
                @for j in items {
                    (render_card(j, hex))
                }
            }
        }
    }
}

fn render_unassigned_column(items: &[&JobRow]) -> Markup {
    html! {
        div.jobs-lane-column.jobs-lane-column-unassigned style="--lane-color: #888888" {
            div.jobs-lane-column-head {
                span.lane-chip.lane-chip-lg.lane-none title="No primary lane" { "Unassigned" }
                span.jobs-lane-count { (items.len()) }
            }
            div.jobs-lane-column-body {
                @for j in items {
                    (render_card(j, "#888888"))
                }
            }
        }
    }
}

fn render_card(j: &JobRow, lane_hex_value: &str) -> Markup {
    // Title is rendered into a single-line truncated span (CSS handles ellipsis).
    // Company always present; location falls back to em-dash so the meta row
    // keeps its shape and the cards remain visually uniform.
    let location = j.location.as_deref().unwrap_or("—");
    html! {
        div.lane-card.row-clickable
            id=(format!("lane-card-job-{}", j.id))
            data-detail=(format!("job-{}", j.id))
            style=(format!("--lane-color: {lane_hex_value}")) {
            div.lane-card-head {
                (grade_pill(j.grade.as_deref()))
                span.lane-card-title title=(j.title) { (j.title) }
            }
            div.lane-card-meta {
                span.lane-card-company { (j.company_name) }
                span.lane-card-sep { " · " }
                span.lane-card-location { (location) }
            }
        }
    }
}
