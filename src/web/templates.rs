//! Shared maud templates — layout chrome, lane badges, grade pills.

use crate::data::lane::{lane_badge, lane_hex, lane_label, primary_lane, LANE_KEYS};
use maud::{html, Markup, DOCTYPE};

/// Top-level page chrome (head, nav, body wrapper).
pub fn page(title: &str, active: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme="palantir" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "cernio · " (title) }
                link rel="stylesheet" href="/static/style.css";
                script src="https://unpkg.com/htmx.org@1.9.12" {}
                script src="https://cdn.jsdelivr.net/npm/echarts@5.4.3/dist/echarts.min.js" {}
            }
            body {
                header.topbar {
                    div.brand {
                        span.brand-mark { "C" }
                        span.brand-name { "cernio" }
                    }
                    nav.tabs {
                        (tab("/", "Dashboard", active == "dashboard"))
                        (tab("/companies", "Companies", active == "companies"))
                        (tab("/jobs", "Jobs", active == "jobs"))
                        (tab("/activity", "Activity", active == "activity"))
                    }
                    div.status-strip {
                        span.dot {}
                        span.status-label { "localhost" }
                    }
                }
                main { (body) }
            }
        }
    }
}

fn tab(href: &str, label: &str, active: bool) -> Markup {
    let class = if active { "tab tab-active" } else { "tab" };
    html! {
        a.(class) href=(href) { (label) }
    }
}

/// Coloured lane badge.
pub fn lane_chip(key: &str) -> Markup {
    let color = lane_hex(key);
    let badge = lane_badge(key);
    let label = lane_label(key);
    html! {
        span.lane-chip title=(label) style=(format!("--lane-color: {color}")) {
            (badge)
        }
    }
}

pub fn lane_chip_for(lanes_json: Option<&str>) -> Markup {
    match primary_lane(lanes_json) {
        Some(key) => lane_chip(&key),
        None => html! { span.lane-chip.lane-none { "—" } },
    }
}

/// Grade pill (SS / S / A / B / C / F).
pub fn grade_pill(grade: Option<&str>) -> Markup {
    let g = grade.unwrap_or("—");
    let class = match grade {
        Some("SS") => "grade-pill grade-ss",
        Some("S") => "grade-pill grade-s",
        Some("A") => "grade-pill grade-a",
        Some("B") => "grade-pill grade-b",
        Some("C") => "grade-pill grade-c",
        Some("F") => "grade-pill grade-f",
        _ => "grade-pill grade-none",
    };
    html! { span.(class) { (g) } }
}

pub fn lane_legend() -> Markup {
    html! {
        div.lane-legend {
            @for key in LANE_KEYS.iter() {
                span.lane-legend-item style=(format!("--lane-color: {}", lane_hex(key))) {
                    span.lane-legend-dot {}
                    span.lane-legend-label { (lane_label(key)) }
                }
            }
        }
    }
}
