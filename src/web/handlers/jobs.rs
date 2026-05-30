use axum::extract::{Form, Path, Query, State};
use axum::response::Html;
use maud::{html, Markup};
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;

use crate::data::lane::{lane_hex, lane_label, primary_lane, LANE_KEYS};
use crate::data::models::{JobFilters, JobRow, SortMode};
use crate::data::queries;
use crate::web::templates::{
    grade_pill, json_island, lane_chip_for, lane_legend, page_with, PageAssets,
};
use crate::web::AppState;

/// Multi-value chip query — every axis is a comma-separated list, empty/missing
/// means "no filter on this axis" (except `archive`, which defaults to `active`).
#[derive(Debug, Deserialize, Default)]
pub struct JobsQuery {
    pub lane: Option<String>,
    pub grade: Option<String>,
    pub decision: Option<String>,
    pub archive: Option<String>,
    pub ats: Option<String>,
    pub posted: Option<String>,
    pub sponsor: Option<String>,
}

pub async fn page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<JobsQuery>,
) -> Html<String> {
    let body = render(&state, &q).await;
    Html(
        page_with(
            "Jobs",
            "jobs",
            PageAssets::css_js("/static/css/jobs.css", "/static/js/jobs.js"),
            body,
        )
        .into_string(),
    )
}

// ── Axis definitions ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct AxisDef {
    key: &'static str,        // URL param name
    label: &'static str,      // sidebar label
    chips: &'static [(&'static str, &'static str)], // (value, display)
    kind: ChipKind,
}

#[derive(Clone, Copy)]
enum ChipKind {
    Lane,
    Grade,
    Plain,
}

const AXES: &[AxisDef] = &[
    AxisDef {
        key: "lane",
        label: "Lane",
        chips: &[
            ("big-tech", "Big Tech"),
            ("ai-ml", "AI / ML"),
            ("hft", "HFT"),
            ("crypto-mm", "Crypto MM"),
            ("bank-strats", "Bank Strats"),
            ("systems-infra", "Systems Infra"),
            ("devtools", "Devtools"),
            ("fintech", "Fintech"),
        ],
        kind: ChipKind::Lane,
    },
    AxisDef {
        key: "grade",
        label: "Grade",
        chips: &[
            ("SS", "SS"),
            ("S", "S"),
            ("A", "A"),
            ("B", "B"),
            ("C", "C"),
            ("F", "F"),
            ("?", "ungraded"),
        ],
        kind: ChipKind::Grade,
    },
    AxisDef {
        key: "decision",
        label: "Decision",
        chips: &[
            ("none", "untouched"),
            ("watching", "watching"),
            ("applied", "applied"),
            ("interview", "interview"),
            ("rejected", "rejected"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "archive",
        label: "Archive",
        chips: &[
            ("active", "active"),
            ("archived", "archived"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "ats",
        label: "ATS",
        chips: &[
            ("greenhouse", "greenhouse"),
            ("ashby", "ashby"),
            ("lever", "lever"),
            ("workable", "workable"),
            ("smartrecruiters", "smartrec"),
            ("workday", "workday"),
            ("eightfold", "eightfold"),
            ("bespoke", "bespoke"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "posted",
        label: "Posted",
        chips: &[
            ("7d", "≤ 7d"),
            ("30d", "7–30d"),
            ("90d", "30–90d"),
            ("old", "> 90d"),
            ("unknown", "unknown"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "sponsor",
        label: "Sponsor",
        chips: &[
            ("yes", "yes"),
            ("no", "no"),
            ("unknown", "unknown"),
        ],
        kind: ChipKind::Plain,
    },
];

fn parse_csv(s: &Option<String>) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Some(raw) = s {
        for part in raw.split(',') {
            let t = part.trim();
            if !t.is_empty() {
                out.insert(t.to_string());
            }
        }
    }
    out
}

/// Current set for an axis given the query — applies the archive default.
fn axis_set(q: &JobsQuery, axis: &AxisDef) -> HashSet<String> {
    let raw = match axis.key {
        "lane" => &q.lane,
        "grade" => &q.grade,
        "decision" => &q.decision,
        "archive" => &q.archive,
        "ats" => &q.ats,
        "posted" => &q.posted,
        "sponsor" => &q.sponsor,
        _ => &None,
    };
    let mut set = parse_csv(raw);
    // Archive defaults to {active} when not specified.
    if axis.key == "archive" && raw.is_none() {
        set.insert("active".to_string());
    }
    set
}

/// Build the URL produced by toggling `value` in `axis` against the current query.
fn toggle_href(q: &JobsQuery, axis: &AxisDef, value: &str) -> String {
    // Re-emit every axis; replace the one being toggled with the new CSV.
    let mut parts: Vec<(&str, String)> = Vec::new();
    for a in AXES {
        let current = axis_set(q, a);
        let new_set: HashSet<String> = if a.key == axis.key {
            let mut s = current.clone();
            if s.contains(value) {
                s.remove(value);
            } else {
                s.insert(value.to_string());
            }
            s
        } else {
            current
        };
        // Skip default archive (single "active") to keep URLs clean.
        let is_default = a.key == "archive"
            && new_set.len() == 1
            && new_set.contains("active");
        if !new_set.is_empty() && !is_default {
            let mut vals: Vec<&str> = new_set.iter().map(|s| s.as_str()).collect();
            vals.sort();
            parts.push((a.key, vals.join(",")));
        }
    }
    if parts.is_empty() {
        "/jobs".to_string()
    } else {
        let qs = parts
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("/jobs?{qs}")
    }
}

fn urlencode(s: &str) -> String {
    // Only commas + safe ASCII; encode comma → %2C, space → %20, ? → %3F.
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ',' => "%2C".to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

// ── Render ─────────────────────────────────────────────────────────────────

async fn render(state: &AppState, q: &JobsQuery) -> Markup {
    let Ok(conn) = Connection::open(&state.db_path) else {
        return html! { div.error { "Could not open database." } };
    };

    // Build JobFilters from query — JobFilters defaults set archive={active}
    // and evidence={jd,semantic}; we override per-axis from the query.
    let mut filters = JobFilters::default();
    filters.lanes = axis_set(q, &AXES[0]);
    filters.grades = axis_set(q, &AXES[1]);
    filters.decisions = axis_set(q, &AXES[2]);
    filters.archive = axis_set(q, &AXES[3]);
    let ats_set = axis_set(q, &AXES[4]);
    filters.ats = ats_set;
    filters.posted_within = axis_set(q, &AXES[5]);
    filters.sponsor = axis_set(q, &AXES[6]);

    let jobs = queries::fetch_jobs(&conn, None, &filters, SortMode::ByGrade);

    // ── Filter-scoped analytics (all computed from `jobs`) ────────────────
    let kpi_total = jobs.len() as i64;
    let kpi_strong = jobs
        .iter()
        .filter(|j| matches!(j.grade.as_deref(), Some("SS") | Some("S")))
        .count() as i64;
    let funnel = decision_funnel_from(&jobs);
    let kpi_untouched = lookup_funnel(&funnel, "untouched");
    let kpi_watching = lookup_funnel(&funnel, "watching");
    let kpi_applied = lookup_funnel(&funnel, "applied");
    let freshness = freshness_from(&jobs);
    let kpi_freshness_median = freshness_median_bucket(&freshness);
    let heatmap = heatmap_from(&jobs);
    let top_companies = top_companies_from(&jobs, 10);
    let top_titles = top_titles_from(&jobs, 10);

    let heatmap_max = heatmap
        .iter()
        .flat_map(|(_, row)| row.iter())
        .copied()
        .max()
        .unwrap_or(0);

    let freshness_colors_map = [
        ("0-7d", "#4ade80"),
        ("7-30d", "#7ea8ff"),
        ("30-90d", "#ffc94a"),
        ("90d+", "#ff8a5c"),
        ("unknown", "#4f5762"),
    ];
    let freshness_data = serde_json::json!({
        "labels": freshness.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
        "values": freshness.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
        "colors": freshness.iter().map(|(k, _)| {
            freshness_colors_map.iter().find(|(b, _)| *b == k.as_str())
                .map(|(_, c)| (*c).to_string()).unwrap_or_else(|| "#4f5762".to_string())
        }).collect::<Vec<_>>(),
    });

    let funnel_total: i64 = funnel.iter().map(|(_, n)| *n).sum();
    let companies_max = top_companies.iter().map(|(_, n)| *n).max().unwrap_or(0);
    let titles_max = top_titles.iter().map(|(_, n)| *n).max().unwrap_or(0);

    let any_active = AXES.iter().any(|a| {
        let s = axis_set(q, a);
        // Treat default archive={active} as "not active".
        if a.key == "archive" {
            !(s.len() == 1 && s.contains("active"))
        } else {
            !s.is_empty()
        }
    });

    html! {
        div.jobs-page {
            (lane_legend())

            // ── Chip strip filter ─────────────────────────────────────────
            div.filter-strip {
                @for axis in AXES.iter() {
                    (render_axis(q, axis))
                }
                div.filter-summary-row {
                    span.filter-count { (kpi_total) " jobs" }
                    @if any_active {
                        a.filter-reset href="/jobs" { "reset all" }
                    }
                }
            }

            // ── KPI strip ─────────────────────────────────────────────────
            section.kpi-strip {
                div.kpi {
                    div.kpi-label { "Filtered jobs" }
                    div.kpi-value data-ticker=(kpi_total) { "0" }
                }
                div.kpi {
                    div.kpi-label { "SS + S" }
                    div.kpi-value data-ticker=(kpi_strong) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Freshness median" }
                    div.kpi-value { (kpi_freshness_median) }
                }
                div.kpi {
                    div.kpi-label { "Untouched" }
                    div.kpi-value data-ticker=(kpi_untouched) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Watching" }
                    div.kpi-value data-ticker=(kpi_watching) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Applied" }
                    div.kpi-value data-ticker=(kpi_applied) { "0" }
                }
            }

            // ── Heatmap pane (grade × lane) ───────────────────────────────
            section.panel.heatmap {
                header.panel-head {
                    h2 { "Grade × Lane heatmap" }
                    span.panel-sub { "filtered jobs · cell shade scales to volume" }
                }
                div.heatmap-grid {
                    div.heatmap-cell.heatmap-corner { "" }
                    @for key in LANE_KEYS.iter() {
                        div.heatmap-cell.heatmap-head
                            title=(lane_label(key))
                            style=(format!("--lane-color: {}", lane_hex(key))) {
                            (key)
                        }
                    }
                    @for (grade, row) in &heatmap {
                        div.heatmap-cell.heatmap-row-head {
                            (grade_pill(Some(grade.as_str())))
                        }
                        @for n in row {
                            (heatmap_cell(*n, heatmap_max))
                        }
                    }
                }
            }

            // ── Pair row: Freshness + Decision funnel ─────────────────────
            div.jobs-pair {
                section.panel {
                    header.panel-head {
                        h2 { "Posting freshness" }
                        span.panel-sub { "filtered jobs · by posted date" }
                    }
                    div id="chart-freshness" class="chart chart-sm" {}
                    (json_island("freshness", &freshness_data))
                }
                section.panel {
                    header.panel-head {
                        h2 { "Decision funnel" }
                        span.panel-sub { "filtered jobs · where each has landed" }
                    }
                    div.funnel {
                        @for (label, count) in &funnel {
                            @let pct = if funnel_total > 0 {
                                (*count as f64 * 100.0 / funnel_total as f64) as i32
                            } else { 0 };
                            div.funnel-row data-decision=(label) {
                                span.funnel-label { (label) }
                                div.funnel-bar-track {
                                    div.funnel-bar
                                        style=(format!("--target-scale: {}", pct as f64 / 100.0)) {}
                                }
                                span.funnel-num { (count) }
                                span.funnel-pct { (pct) "%" }
                            }
                        }
                    }
                }
            }

            // ── Pair row: Top companies + Top role titles ─────────────────
            div.jobs-pair {
                section.panel {
                    header.panel-head {
                        h2 { "Company concentration" }
                        span.panel-sub { "top 10 by filtered job count" }
                    }
                    div.top-list {
                        @if top_companies.is_empty() {
                            div.empty { "No jobs in current filter." }
                        }
                        @for (i, (name, total)) in top_companies.iter().enumerate() {
                            @let pct = if companies_max > 0 { *total * 100 / companies_max } else { 0 };
                            div.top-list-row {
                                span.top-list-rank { (i + 1) }
                                span.top-list-name { (name) }
                                div.top-list-bar {
                                    div.top-list-bar-fill style=(format!("--target-width: {pct}%")) {}
                                }
                                span.top-list-num { (total) }
                            }
                        }
                    }
                }
                section.panel {
                    header.panel-head {
                        h2 { "Top role titles" }
                        span.panel-sub { "filtered jobs · most common exact titles" }
                    }
                    div.top-list {
                        @if top_titles.is_empty() {
                            div.empty { "No jobs in current filter." }
                        }
                        @for (i, (title, count)) in top_titles.iter().enumerate() {
                            @let pct = if titles_max > 0 { *count * 100 / titles_max } else { 0 };
                            div.top-list-row {
                                span.top-list-rank { (i + 1) }
                                span.top-list-name { (title) }
                                div.top-list-bar {
                                    div.top-list-bar-fill style=(format!("--target-width: {pct}%")) {}
                                }
                                span.top-list-num { (count) }
                            }
                        }
                    }
                }
            }

            // ── Filtered jobs table ───────────────────────────────────────
            section.panel.no-pad {
                header.panel-head {
                    h2 { "Filtered jobs" }
                    span.panel-sub { (jobs.len()) " matching · sorted by grade" }
                }
                table.job-table {
                    thead {
                        tr {
                            th.col-grade { "Grade" }
                            th.col-lane  { "Lane" }
                            th.col-title { "Title" }
                            th.col-co    { "Company" }
                            th.col-loc   { "Location" }
                            th.col-actions { "" }
                        }
                    }
                    tbody {
                        @for j in &jobs {
                            tr.job-row id=(format!("job-{}", j.id)) {
                                td.col-grade { (grade_pill(j.grade.as_deref())) }
                                td.col-lane  { (lane_chip_for(j.lanes.as_deref())) }
                                td.col-title { div.job-title data-marquee="" { (j.title) } }
                                td.col-co    { (j.company_name) }
                                td.col-loc   { (j.location.as_deref().unwrap_or("—")) }
                                td.col-actions {
                                    (decision_buttons(j.id, j.decision.as_deref(), &j.url))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_axis(q: &JobsQuery, axis: &AxisDef) -> Markup {
    let active = axis_set(q, axis);
    html! {
        div.filter-axis {
            span.filter-axis-label { (axis.label) }
            div.chips {
                @for (val, disp) in axis.chips.iter() {
                    @let is_active = active.contains(*val);
                    @let href = toggle_href(q, axis, val);
                    @let (klass, style) = chip_class_and_style(axis.kind, val, is_active);
                    a class=(klass) href=(href) style=(style) data-active=(is_active) {
                        (disp)
                    }
                }
            }
        }
    }
}

fn chip_class_and_style(kind: ChipKind, val: &str, active: bool) -> (String, String) {
    let base = if active { "chip chip-active" } else { "chip" };
    match kind {
        ChipKind::Lane => {
            let kind_class = format!("{base} chip-lane");
            let style = format!("--chip-color: {}", lane_hex(val));
            (kind_class, style)
        }
        ChipKind::Grade => {
            let var = match val {
                "SS" => "var(--grade-ss)",
                "S" => "var(--grade-s)",
                "A" => "var(--grade-a)",
                "B" => "var(--grade-b)",
                "C" => "var(--grade-c)",
                "F" => "var(--grade-f)",
                _ => "var(--text-4)",
            };
            let kind_class = format!("{base} chip-grade");
            let style = format!("--chip-color: {var}");
            (kind_class, style)
        }
        ChipKind::Plain => (base.to_string(), String::new()),
    }
}

// ── In-handler analytics: every aggregate is filter-scoped ────────────────

fn decision_funnel_from(jobs: &[JobRow]) -> Vec<(String, i64)> {
    let mut untouched = 0i64;
    let mut watching = 0i64;
    let mut applied = 0i64;
    let mut interview = 0i64;
    let mut rejected = 0i64;
    for j in jobs {
        match j.decision.as_deref() {
            None => untouched += 1,
            Some("watching") => watching += 1,
            Some("applied") => applied += 1,
            Some("interview") => interview += 1,
            Some("rejected") => rejected += 1,
            _ => untouched += 1,
        }
    }
    vec![
        ("untouched".into(), untouched),
        ("watching".into(), watching),
        ("applied".into(), applied),
        ("interview".into(), interview),
        ("rejected".into(), rejected),
    ]
}

fn lookup_funnel(funnel: &[(String, i64)], key: &str) -> i64 {
    funnel
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, n)| *n)
        .unwrap_or(0)
}

fn freshness_from(jobs: &[JobRow]) -> Vec<(String, i64)> {
    // Compute current date once per render.
    let now = chrono::Utc::now().naive_utc();
    let mut b07 = 0i64;
    let mut b730 = 0i64;
    let mut b3090 = 0i64;
    let mut bold = 0i64;
    let mut bunk = 0i64;
    for j in jobs {
        match j.posted_date.as_deref() {
            None => bunk += 1,
            Some(s) => {
                if let Some(dt) = parse_posted(s) {
                    let days = (now - dt).num_days();
                    if days < 0 {
                        b07 += 1;
                    } else if days <= 7 {
                        b07 += 1;
                    } else if days <= 30 {
                        b730 += 1;
                    } else if days <= 90 {
                        b3090 += 1;
                    } else {
                        bold += 1;
                    }
                } else {
                    bunk += 1;
                }
            }
        }
    }
    vec![
        ("0-7d".into(), b07),
        ("7-30d".into(), b730),
        ("30-90d".into(), b3090),
        ("90d+".into(), bold),
        ("unknown".into(), bunk),
    ]
}

fn parse_posted(s: &str) -> Option<chrono::NaiveDateTime> {
    // Accept "YYYY-MM-DD" and full ISO-8601.
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d.and_hms_opt(0, 0, 0).unwrap_or_default());
    }
    if let Ok(d) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(d);
    }
    if let Ok(d) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(d.naive_utc());
    }
    None
}

fn heatmap_from(jobs: &[JobRow]) -> Vec<(String, Vec<i64>)> {
    let grades = ["SS", "S", "A", "B", "C", "F"];
    let mut out: Vec<(String, Vec<i64>)> = grades
        .iter()
        .map(|g| ((*g).to_string(), vec![0i64; LANE_KEYS.len()]))
        .collect();
    for j in jobs {
        let Some(g) = j.grade.as_deref() else { continue };
        let Some(gi) = grades.iter().position(|x| *x == g) else { continue };
        let Some(lane) = primary_lane(j.lanes.as_deref()) else { continue };
        let Some(li) = LANE_KEYS.iter().position(|x| *x == lane.as_str()) else { continue };
        out[gi].1[li] += 1;
    }
    out
}

fn top_companies_from(jobs: &[JobRow], n: usize) -> Vec<(String, i64)> {
    let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for j in jobs {
        *map.entry(j.company_name.clone()).or_insert(0) += 1;
    }
    let mut v: Vec<(String, i64)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn top_titles_from(jobs: &[JobRow], n: usize) -> Vec<(String, i64)> {
    let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for j in jobs {
        *map.entry(j.title.clone()).or_insert(0) += 1;
    }
    let mut v: Vec<(String, i64)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn freshness_median_bucket(buckets: &[(String, i64)]) -> String {
    buckets
        .iter()
        .filter(|(k, _)| k != "unknown")
        .max_by_key(|(_, n)| *n)
        .filter(|(_, n)| *n > 0)
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| "—".to_string())
}

fn heatmap_cell(count: i64, max: i64) -> Markup {
    let class = if count == 0 || max == 0 {
        "heatmap-cell zero"
    } else {
        let ratio = count as f64 / max as f64;
        if ratio < 0.15 {
            "heatmap-cell low"
        } else if ratio < 0.5 {
            "heatmap-cell mid"
        } else {
            "heatmap-cell hot"
        }
    };
    html! {
        div class=(class) {
            @if count == 0 { "—" } @else { (count) }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DecisionForm {
    pub decision: String,
}

pub async fn decision(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Form(form): Form<DecisionForm>,
) -> Html<String> {
    let decision = form.decision.as_str();
    let allowed = matches!(decision, "applied" | "watching" | "rejected" | "interview");
    if !allowed {
        return Html(html! { div.error { "Invalid decision" } }.into_string());
    }

    if let Ok(conn) = Connection::open(&state.db_path) {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = conn.execute(
            "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, decision, now],
        );
        crate::db::events::decision_recorded(&conn, id, decision, "web");

        let url: String = conn
            .query_row(
                "SELECT url FROM jobs WHERE id = ?1",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        return Html(decision_buttons(id, Some(decision), &url).into_string());
    }

    Html(html! { div.error { "DB error" } }.into_string())
}

fn decision_buttons(id: i64, current: Option<&str>, url: &str) -> Markup {
    let active = |kind: &str| {
        if current == Some(kind) {
            "btn btn-active"
        } else {
            "btn"
        }
    };
    html! {
        div.decision-buttons {
            a.btn.btn-apply href=(url) target="_blank"
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"applied"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Apply"
            }
            button.(active("watching"))
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"watching"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Watch"
            }
            button.(active("rejected"))
                hx-post=(format!("/jobs/{id}/decision"))
                hx-vals=(r#"{"decision":"rejected"}"#)
                hx-swap="outerHTML"
                hx-target="closest .decision-buttons" {
                "Reject"
            }
            @if let Some(d) = current {
                span.decision-current { (d) }
            }
        }
    }
}
