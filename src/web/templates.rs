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
                link rel="stylesheet" href="/static/css/chips.css";
                link rel="stylesheet" href="/static/css/buttons.css";
                link rel="stylesheet" href="/static/css/rows.css";
                link rel="stylesheet" href="/static/css/tables.css";
                link rel="stylesheet" href="/static/css/jobs-lanes.css";
                link rel="stylesheet" href="/static/css/chrome.css";
                link rel="stylesheet" href="/static/css/filters.css";
                link rel="stylesheet" href="/static/css/filters-pie.css";
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
                script src="/static/js/filters-toggle.js" defer {}
                script src="/static/js/filters-pie.js" defer {}
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

/// Render the radial lane filter pie. The caller supplies:
///   - `active`: the set of currently-active lane keys (use `is_active` from
///     the page's filter module so URL semantics match the chip variant).
///   - `toggle_href`: closure mapping `lane_key → URL` that toggles that lane.
///
/// The pie is server-rendered; JS only enhances (shift-click for "only this",
/// centre-button for master toggle). Without JS the pie still works as eight
/// independent toggle links plus a centre button that follows a static href.
pub fn lane_pie(
    active: &std::collections::HashSet<String>,
    toggle_href: impl Fn(&str) -> String,
    base_path: &str,
) -> Markup {
    use std::f64::consts::PI;

    // Build polygon points for each wedge. The pie box is treated as a unit
    // square (we use percentages for clip-path), centre = (50%, 50%), radius
    // along the diagonal so wedges meet the bounding box rather than being
    // clipped at a small inset circle.
    let n = LANE_KEYS.len(); // 8
    let step = 2.0 * PI / n as f64;
    // Start at -PI/2 so the first wedge points "up".
    let start_angle = -PI / 2.0;

    fn polar_to_xy(angle: f64) -> (f64, f64) {
        // Project from centre out far enough that the wedge fills the bounding
        // box (radius √2/2 ≈ 0.707). We extend further (1.0) and rely on the
        // bounding box to clip — produces clean edge meets between wedges.
        let r = 1.0_f64;
        let x = 50.0 + 50.0 * r * angle.cos();
        let y = 50.0 + 50.0 * r * angle.sin();
        // Clamp to [0,100] — outside values still clip cleanly but linting
        // friendlier numbers help when debugging in DevTools.
        (x.clamp(-10.0, 110.0), y.clamp(-10.0, 110.0))
    }

    html! {
        div.lane-pie-wrap {
            div.lane-pie {
                @for (i, key) in LANE_KEYS.iter().enumerate() {
                    @let a0 = start_angle + step * i as f64;
                    @let a1 = start_angle + step * (i + 1) as f64;
                    @let amid = (a0 + a1) / 2.0;

                    // Polygon: centre + a0 endpoint + a few interpolated points + a1 endpoint.
                    // 4 interior samples gives a clean arc-ish edge for 45° wedges.
                    @let (x0, y0) = polar_to_xy(a0);
                    @let (xa, ya) = polar_to_xy(a0 + (a1 - a0) * 0.33);
                    @let (xb, yb) = polar_to_xy(a0 + (a1 - a0) * 0.66);
                    @let (x1, y1) = polar_to_xy(a1);
                    @let clip = format!(
                        "polygon(50% 50%, {x0:.2}% {y0:.2}%, {xa:.2}% {ya:.2}%, {xb:.2}% {yb:.2}%, {x1:.2}% {y1:.2}%)"
                    );

                    // Label position: midpoint along the wedge bisector at ~62% radius.
                    @let lx = 50.0 * 0.62 * amid.cos();
                    @let ly = 50.0 * 0.62 * amid.sin();
                    // Rotation: read outward from centre. amid is in radians where
                    // 0 = right, PI/2 = down. Convert to degrees and add 90° so text
                    // baseline points outward. Flip when on the left half so text
                    // doesn't read upside-down.
                    @let deg = amid.to_degrees();
                    @let rot = if deg > 90.0 || deg < -90.0 { deg + 180.0 } else { deg };

                    @let is_on = active.is_empty() || active.contains(*key);
                    @let href = toggle_href(key);
                    @let style = format!(
                        "--wedge-color: {hex}; --wedge-clip: {clip}; --label-x: {lx:.2}px; --label-y: {ly:.2}px; --label-rot: {rot:.2}deg",
                        hex = lane_hex(key),
                    );

                    a class="lane-wedge"
                      href=(href)
                      title=(lane_label(key))
                      data-lane=(*key)
                      data-active=(is_on)
                      style=(style) {
                        span.lane-wedge-label { (lane_badge(key)) }
                    }
                }

                // Centre master toggle. Default href clears the lane filter
                // (no-JS fallback = "show all lanes"). JS upgrades to toggle.
                a class="lane-pie-centre"
                  href=(base_path)
                  data-state="all"
                  title="Toggle all lanes (click) — server: clears lane filter" {
                    span.lane-pie-centre-glyph { "○" }
                }
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
