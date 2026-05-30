//! Shared maud templates — layout chrome, lane badges, grade pills.

use crate::data::lane::{lane_badge, lane_hex, lane_label, primary_lane, LANE_KEYS};
use maud::{html, Markup, DOCTYPE};

/// Per-page asset bundle. The chrome injects shared assets; the page passes
/// its own page-specific stylesheet + script paths (one each, may be empty).
pub struct PageAssets<'a> {
    pub page_css: Option<&'a str>,
    pub page_js: Option<&'a str>,
}

impl<'a> PageAssets<'a> {
    pub const fn css_js(css: &'a str, js: &'a str) -> Self {
        Self { page_css: Some(css), page_js: Some(js) }
    }
}

/// Top-level page chrome with explicit per-page asset paths.
pub fn page_with(title: &str, active: &str, assets: PageAssets, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme="palantir" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "cernio · " (title) }
                // Shared stylesheets — loaded on every page.
                link rel="stylesheet" href="/static/css/base.css";
                link rel="stylesheet" href="/static/css/motion.css";
                link rel="stylesheet" href="/static/css/components.css";
                link rel="stylesheet" href="/static/css/chrome.css";
                link rel="stylesheet" href="/static/css/filters.css";
                link rel="stylesheet" href="/static/css/debug.css";
                link rel="stylesheet" href="/static/css/ops.css";
                // Page-specific stylesheet (optional).
                @if let Some(css) = assets.page_css {
                    link rel="stylesheet" href=(css);
                }
                // External libs.
                script src="https://unpkg.com/htmx.org@1.9.12" {}
                script src="https://cdn.jsdelivr.net/npm/echarts@5.4.3/dist/echarts.min.js" {}
                // Shared JS — defer so DOM is parsed first.
                script src="/static/js/core.js" defer {}
                script src="/static/js/charts.js" defer {}
                script src="/static/js/debug.js" defer {}
                script src="/static/js/ops.js" defer {}
                // Page-specific JS (optional, deferred).
                @if let Some(js) = assets.page_js {
                    script src=(js) defer {}
                }
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
                // Ops menu — floating top-right; opens a panel of pipeline operations.
                div #ops-menu {
                    button #ops-btn class="ops-btn" type="button" title="Pipeline operations" {
                        span.ops-gear { "⚙" }
                        span.ops-label { "ops" }
                    }
                    div #ops-panel class="ops-panel hidden" {
                        header.ops-panel-head { "Pipeline operations" }
                        div.ops-list {
                            (ops_item("clean", "Clean DB", "Archive stale jobs by tier (SS=28d → F=3d) and low-grade companies; prune expired archived rows."))
                            (ops_item("format", "Format text", "Normalise HTML descriptions and whitespace in fit assessments. Capped at 50 rows per click."))
                        }
                    }
                }
                // Debug screenshot trigger — bottom-right floating button.
                button #snap-all class="snap-btn" title="Capture all 4 tabs as PNGs into /tmp/cernio-debug/" {
                    span.snap-dot {}
                    span.snap-label { "snap all" }
                }
                div #snap-toast class="snap-toast" {}
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

fn ops_item(op: &str, title: &str, desc: &str) -> Markup {
    html! {
        div.ops-item data-op=(op) {
            header.ops-item-head {
                h4 { (title) }
                span.ops-preview data-op-preview=(op) { "loading…" }
            }
            p.ops-desc { (desc) }
            // Structured detail rendered by ops.js (chips / kv grid, not JSON).
            div.ops-detail data-op-detail=(op) {}
            div.ops-actions {
                button.ops-run type="button" data-op-run=(op) { "Run" }
            }
        }
    }
}

/// JSON island helper — emits a `<script type="application/json" id="data-<kind>">…</script>`
/// for per-page JS to read.
pub fn json_island(kind: &str, value: &serde_json::Value) -> Markup {
    let id = format!("data-{kind}");
    html! {
        script type="application/json" id=(id) {
            (maud::PreEscaped(value.to_string()))
        }
    }
}
