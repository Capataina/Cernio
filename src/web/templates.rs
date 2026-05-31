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

/// Render the radial lane filter pie as SVG. The caller supplies:
///   - `active`: the set of currently-active lane keys.
///   - `toggle_href`: closure mapping `lane_key → URL` that toggles that lane.
///   - `base_path`: page base URL the centre master-toggle navigates to when
///     clearing the lane filter without JS.
///
/// The pie is server-rendered as eight `<path>` wedges around a circular
/// outline, with a small angular gap between wedges (~2px visual) achieved by
/// shrinking each wedge's sweep by `GAP_RAD` radians on each side. The wedge
/// outer perimeter is a true arc (SVG `A` command), so the silhouette is a
/// proper circle rather than a polygon. Wedge fill, hover lift, and label
/// rotation are driven by CSS using inline-set custom properties.
///
/// JS enhances: shift-click on a wedge = "only this lane"; centre = master
/// toggle. Without JS the pie still works (each wedge is an anchor, centre
/// clears the filter).
pub fn lane_pie(
    active: &std::collections::HashSet<String>,
    toggle_href: impl Fn(&str) -> String,
    base_path: &str,
) -> Markup {
    use std::f64::consts::PI;

    // SVG viewBox is 0 0 240 240 — gives a logical 240px coord system that
    // CSS can scale freely without losing arc fidelity.
    const VB: f64 = 240.0;
    const CX: f64 = VB / 2.0;
    const CY: f64 = VB / 2.0;
    // Outer radius leaves a hair of room for the hover-lift transform so the
    // expanded wedge doesn't clip the viewBox.
    const R_OUTER: f64 = 110.0;
    // Inner radius cuts a hole for the centre master-toggle button (rendered
    // as a separate HTML element absolutely positioned over the SVG).
    const R_INNER: f64 = 34.0;
    // Angular gap between wedges, in radians. ~0.018 rad ≈ 1° ≈ 2px at r=110.
    const GAP_RAD: f64 = 0.022;
    // Label sits at 70% of the way from inner to outer radius.
    const R_LABEL_FRAC: f64 = 0.62;

    let n = LANE_KEYS.len(); // 8
    let step = 2.0 * PI / n as f64;
    // Start at -PI/2 so the first wedge points "up".
    let start_angle = -PI / 2.0;

    /// Build an SVG path string for a donut-segment wedge between angles
    /// `a0`..`a1`, with inner radius `ri` and outer radius `ro`. Renders
    /// outer edge as a true arc.
    fn wedge_path(a0: f64, a1: f64, ri: f64, ro: f64) -> String {
        let (sin0, cos0) = (a0.sin(), a0.cos());
        let (sin1, cos1) = (a1.sin(), a1.cos());
        let p0_x = CX + ro * cos0;
        let p0_y = CY + ro * sin0;
        let p1_x = CX + ro * cos1;
        let p1_y = CY + ro * sin1;
        let p2_x = CX + ri * cos1;
        let p2_y = CY + ri * sin1;
        let p3_x = CX + ri * cos0;
        let p3_y = CY + ri * sin0;
        let large_arc = if (a1 - a0) > PI { 1 } else { 0 };
        format!(
            "M {p0_x:.3} {p0_y:.3} \
             A {ro:.3} {ro:.3} 0 {large_arc} 1 {p1_x:.3} {p1_y:.3} \
             L {p2_x:.3} {p2_y:.3} \
             A {ri:.3} {ri:.3} 0 {large_arc} 0 {p3_x:.3} {p3_y:.3} \
             Z"
        )
    }

    let vb = format!("0 0 {VB} {VB}");

    html! {
        div.lane-pie-wrap {
            div.lane-pie {
                svg.lane-pie-svg viewBox=(vb) preserveAspectRatio="xMidYMid meet"
                    role="img" aria-label="Lane filter" {
                    @for (i, key) in LANE_KEYS.iter().enumerate() {
                        @let a0_raw = start_angle + step * i as f64;
                        @let a1_raw = start_angle + step * (i + 1) as f64;
                        // Shrink the sweep by GAP_RAD/2 on each side to draw
                        // a clean gap between wedges. The gap is the strip
                        // background showing through, not a stroke.
                        @let a0 = a0_raw + GAP_RAD / 2.0;
                        @let a1 = a1_raw - GAP_RAD / 2.0;
                        @let amid = (a0_raw + a1_raw) / 2.0;
                        @let d = wedge_path(a0, a1, R_INNER, R_OUTER);

                        // Label position — polar centre of the wedge at
                        // R_LABEL_FRAC between inner and outer.
                        @let r_label = R_INNER + (R_OUTER - R_INNER) * R_LABEL_FRAC;
                        @let lx = CX + r_label * amid.cos();
                        @let ly = CY + r_label * amid.sin();
                        // Outward-reading rotation, auto-flipped on the left
                        // half so the badge never reads upside-down.
                        @let deg = amid.to_degrees();
                        @let rot = if deg > 90.0 || deg < -90.0 { deg + 180.0 } else { deg };

                        // Hover-lift unit vector (in viewBox units). Wedges
                        // translate ~4 units outward on hover — animated by CSS.
                        @let dx = amid.cos();
                        @let dy = amid.sin();

                        @let is_on = active.is_empty() || active.contains(*key);
                        @let href = toggle_href(key);
                        @let group_style = format!(
                            "--wedge-color: {hex}; --lift-dx: {dx:.4}; --lift-dy: {dy:.4};",
                            hex = lane_hex(key),
                        );

                        a class="lane-wedge"
                          href=(href)
                          data-lane=(*key)
                          data-active=(is_on)
                          style=(group_style) {
                            // <title> gives a native SVG tooltip with the
                            // full human-readable lane name.
                            title { (lane_label(key)) }
                            path.lane-wedge-fill d=(d) {}
                            text.lane-wedge-label
                                x=(format!("{lx:.2}"))
                                y=(format!("{ly:.2}"))
                                text-anchor="middle"
                                dominant-baseline="central"
                                transform=(format!("rotate({rot:.2} {lx:.2} {ly:.2})")) {
                                (lane_badge(key))
                            }
                        }
                    }
                }

                // Centre master toggle — sits on top of the SVG hole.
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
