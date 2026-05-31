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
                link rel="stylesheet" href="/static/css/drawer.css";
                link rel="stylesheet" href="/static/css/cmdk.css";
                link rel="stylesheet" href="/static/css/presets.css";
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
                script src="/static/js/drawer.js" defer {}
                script src="/static/js/cmdk.js" defer {}
                script src="/static/js/presets.js" defer {}
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
                        (tab("/decisions", "Decisions", active == "decisions"))
                        (tab("/activity", "Activity", active == "activity"))
                    }
                    div.topbar-right {
                        div.lane-legend-topbar {
                            span.lane-legend-trigger { "lanes" }
                            div.lane-legend-pop {
                                (lane_legend_inline(active))
                            }
                        }
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
                        div.preset-menu {
                            button #preset-btn class="preset-btn" type="button" title="Saved searches" {
                                "★ presets"
                            }
                            div #preset-panel class="preset-panel hidden" {}
                        }
                        div.status-strip {
                            span.dot {}
                            span.status-label { "localhost" }
                        }
                    }
                }
                main { (body) }
                // Side drawer — populated by /static/js/drawer.js via fetch.
                div #detail-drawer-backdrop class="hidden" {}
                aside #detail-drawer class="detail-drawer hidden" {
                    header.drawer-head {
                        span.drawer-kind {}
                        button.drawer-close type="button" aria-label="close" title="close (esc)" { "×" }
                    }
                    div.drawer-body { "loading…" }
                }
                // Command palette (Cmd-K / Ctrl-K) — driven by /static/js/cmdk.js.
                div #cmdk-backdrop class="cmdk-backdrop hidden" {}
                div #cmdk-palette class="cmdk-palette hidden" role="dialog" aria-modal="true" {
                    header.cmdk-head {
                        input #cmdk-input type="text" class="cmdk-input"
                            placeholder="Search companies, jobs, commands…"
                            autocomplete="off" autocapitalize="off" spellcheck="false";
                        span.cmdk-hint { "⌘K · Esc to close" }
                    }
                    div #cmdk-results class="cmdk-results" {}
                    footer.cmdk-foot {
                        span { "↑↓ navigate · ⏎ open · ⌘⏎ open in new tab" }
                    }
                }
                // Shortcut leader-mode hint (top-right transient).
                div #cmdk-leader class="cmdk-leader hidden" { "g _ — d j c a x" }
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

/// Inline lane legend used inside the topbar popover.
fn lane_legend_inline(page: &str) -> Markup {
    let prefix = match page {
        "companies" => "/companies",
        _ => "/jobs",
    };
    html! {
        @for key in LANE_KEYS.iter() {
            a.lane-legend-item
                href=(format!("{prefix}?lane={key}"))
                style=(format!("--lane-color: {}", lane_hex(key))) {
                span.lane-legend-dot {}
                span.lane-legend-label { (lane_label(key)) }
            }
        }
    }
}

/// Lane legend used to render inline on every page. Now a no-op — the
/// legend lives in the topbar popover (see `lane-legend-topbar` + the
/// hover-triggered `.lane-legend-pop`). Kept as a function so per-page
/// handlers don't need to be touched.
pub fn lane_legend(_page: &str) -> Markup {
    html! {}
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
