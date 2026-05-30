use axum::extract::{Query, State};
use axum::response::Html;
use maud::{html, Markup, PreEscaped};
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::data::lane::{all_lanes, lane_hex, lane_label, LANE_KEYS};
use crate::data::models::CompanyRow;
use crate::data::queries;
use crate::web::templates::{
    grade_pill, json_island, lane_chip, lane_legend, page_with, PageAssets,
};
use crate::web::AppState;

const ATS_ORDER: [&str; 9] = [
    "greenhouse",
    "ashby",
    "lever",
    "workable",
    "smartrecruiters",
    "workday",
    "eightfold",
    "bespoke",
    "potential",
];

const GRADE_CHIPS: [&str; 5] = ["S", "A", "B", "C", "?"];
const STATUS_CHIPS: [&str; 3] = ["resolved", "bespoke", "potential"];
const ATS_FILTER_CHIPS: [&str; 8] = [
    "greenhouse",
    "ashby",
    "lever",
    "workable",
    "smartrecruiters",
    "workday",
    "eightfold",
    "bespoke",
];
const SPONSOR_CHIPS: [&str; 3] = ["yes", "unknown", "no"];
const LOCATION_CHIPS: [&str; 5] = ["london", "uk_ex_london", "remote", "intl", "unknown"];
const HAS_JOBS_CHIPS: [&str; 2] = ["yes", "no"];

fn ats_color(key: &str) -> &'static str {
    match key {
        "greenhouse" => "#4ade80",
        "ashby" => "#7ea8ff",
        "lever" => "#7adf9a",
        "workable" => "#c39df0",
        "smartrecruiters" => "#ffc94a",
        "workday" => "#ff8a5c",
        "eightfold" => "#aab3bf",
        "bespoke" => "#ffc94a",
        "potential" => "#4f5762",
        _ => "#4f7cff",
    }
}

fn grade_color(g: &str) -> &'static str {
    match g {
        "S" => "var(--grade-s)",
        "A" => "var(--grade-a)",
        "B" => "var(--grade-b)",
        "C" => "var(--grade-c)",
        _ => "var(--text-5)",
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct CompaniesQuery {
    /// Comma-separated lane keys (e.g. "hft,ai-ml")
    pub lane: Option<String>,
    /// Comma-separated grades (e.g. "S,A"); "?" for ungraded
    pub grade: Option<String>,
    /// Comma-separated statuses (e.g. "resolved,bespoke")
    pub status: Option<String>,
    /// Comma-separated ATS providers (+ pseudo "bespoke")
    pub ats: Option<String>,
    /// Comma-separated sponsor states ("yes","unknown","no")
    pub sponsor: Option<String>,
    /// Comma-separated location buckets ("london","uk_ex_london","remote","intl","unknown")
    pub location: Option<String>,
    /// "yes" or "no"
    pub has_jobs: Option<String>,
    /// Back-compat; ignored.
    #[serde(default)]
    #[allow(dead_code)]
    pub layout: Option<String>,
}

fn parse_csv(s: &Option<String>) -> HashSet<String> {
    match s {
        None => HashSet::new(),
        Some(v) => v
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect(),
    }
}

/// Map a company's raw `location` field into the same bucket-set used by the
/// Location filter axis. Returns the set of bucket keys the company belongs to.
fn location_buckets(loc: Option<&str>) -> Vec<&'static str> {
    let raw = match loc {
        None => return vec!["unknown"],
        Some(s) if s.is_empty() => return vec!["unknown"],
        Some(s) => s,
    };
    let mut out = Vec::new();
    let lower = raw.to_lowercase();
    let has_london = lower.contains("london");
    let has_remote = lower.contains("remote");
    let uk_cities = [
        "cambridge",
        "oxford",
        "bristol",
        "manchester",
        "edinburgh",
        "southampton",
        "birmingham",
        "leeds",
    ];
    let has_uk_ex_london = uk_cities.iter().any(|c| lower.contains(c))
        || (lower.contains("uk") && !has_london);
    if has_london {
        out.push("london");
    }
    if has_uk_ex_london {
        out.push("uk_ex_london");
    }
    if has_remote {
        out.push("remote");
    }
    // International heuristic: anything outside the above
    if !has_london && !has_uk_ex_london && !has_remote {
        out.push("intl");
    }
    if out.is_empty() {
        out.push("unknown");
    }
    out
}

/// Build the query string for a chip's toggle URL: flip membership of
/// `value` in axis `axis` and re-encode every active axis.
fn toggle_qs(q: &CompaniesQuery, axis: &str, value: &str) -> String {
    let mut axes: Vec<(&str, HashSet<String>)> = vec![
        ("lane", parse_csv(&q.lane)),
        ("grade", parse_csv(&q.grade)),
        ("status", parse_csv(&q.status)),
        ("ats", parse_csv(&q.ats)),
        ("sponsor", parse_csv(&q.sponsor)),
        ("location", parse_csv(&q.location)),
        ("has_jobs", parse_csv(&q.has_jobs)),
    ];
    for (name, set) in axes.iter_mut() {
        if *name == axis {
            if set.contains(value) {
                set.remove(value);
            } else {
                set.insert(value.to_string());
            }
        }
    }
    let mut parts: Vec<String> = Vec::new();
    for (name, set) in &axes {
        if set.is_empty() {
            continue;
        }
        let mut v: Vec<&str> = set.iter().map(|s| s.as_str()).collect();
        v.sort();
        parts.push(format!("{}={}", name, v.join(",")));
    }
    if parts.is_empty() {
        "/companies".to_string()
    } else {
        format!("/companies?{}", parts.join("&"))
    }
}

fn is_active(q: &CompaniesQuery, axis: &str, value: &str) -> bool {
    let set = match axis {
        "lane" => parse_csv(&q.lane),
        "grade" => parse_csv(&q.grade),
        "status" => parse_csv(&q.status),
        "ats" => parse_csv(&q.ats),
        "sponsor" => parse_csv(&q.sponsor),
        "location" => parse_csv(&q.location),
        "has_jobs" => parse_csv(&q.has_jobs),
        _ => HashSet::new(),
    };
    set.contains(value)
}

fn filter_companies(all: &[CompanyRow], q: &CompaniesQuery) -> Vec<CompanyRow> {
    let lanes = parse_csv(&q.lane);
    let grades = parse_csv(&q.grade);
    let statuses = parse_csv(&q.status);
    let atss = parse_csv(&q.ats);
    let sponsors = parse_csv(&q.sponsor);
    let locations = parse_csv(&q.location);
    let has_jobs = parse_csv(&q.has_jobs);

    all.iter()
        .filter(|c| {
            // Lane: intersect
            if !lanes.is_empty() {
                let cls = all_lanes(c.lanes.as_deref());
                if !cls.iter().any(|l| lanes.contains(l)) {
                    return false;
                }
            }
            // Grade
            if !grades.is_empty() {
                let g = c.grade.as_deref();
                let matched = match g {
                    Some(v) => grades.contains(v),
                    None => grades.contains("?"),
                };
                if !matched {
                    return false;
                }
            }
            // Status
            if !statuses.is_empty() && !statuses.contains(&c.status) {
                return false;
            }
            // ATS: provider in set, OR "bespoke" pseudo matches status=="bespoke"
            if !atss.is_empty() {
                let provider_match = c
                    .ats_provider
                    .as_deref()
                    .map(|p| atss.contains(p))
                    .unwrap_or(false);
                let bespoke_match = atss.contains("bespoke") && c.status == "bespoke";
                if !provider_match && !bespoke_match {
                    return false;
                }
            }
            // Sponsor
            if !sponsors.is_empty() {
                let s = c.sponsors_uk.as_deref().unwrap_or("unknown");
                if !sponsors.contains(s) {
                    return false;
                }
            }
            // Location
            if !locations.is_empty() {
                let buckets = location_buckets(c.location.as_deref());
                if !buckets.iter().any(|b| locations.contains(*b)) {
                    return false;
                }
            }
            // has_jobs
            if !has_jobs.is_empty() {
                let yes = c.job_count > 0;
                let want_yes = has_jobs.contains("yes");
                let want_no = has_jobs.contains("no");
                if want_yes && want_no {
                    // both → no filter
                } else if want_yes && !yes {
                    return false;
                } else if want_no && yes {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

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
    let total_companies = companies.len() as i64;
    let s_tier = companies.iter().filter(|c| c.grade.as_deref() == Some("S")).count() as i64;
    let a_tier = companies.iter().filter(|c| c.grade.as_deref() == Some("A")).count() as i64;
    let bespoke = companies.iter().filter(|c| c.status == "bespoke").count() as i64;
    let potential = companies.iter().filter(|c| c.status == "potential").count() as i64;
    let sponsors_yes = companies
        .iter()
        .filter(|c| c.sponsors_uk.as_deref() == Some("yes"))
        .count() as i64;

    // ── Companies × lane donut (filtered) ──
    let mut lane_counts: BTreeMap<&str, i64> = LANE_KEYS.iter().map(|k| (*k, 0)).collect();
    for c in &companies {
        for l in all_lanes(c.lanes.as_deref()) {
            if let Some(slot) = lane_counts.get_mut(l.as_str()) {
                *slot += 1;
            }
        }
    }
    let companies_lane_json = serde_json::json!({
        "items": LANE_KEYS.iter().map(|k| serde_json::json!({
            "label": lane_label(k),
            "value": lane_counts.get(*k).copied().unwrap_or(0),
            "color": lane_hex(k),
        })).collect::<Vec<_>>(),
    });

    // ── Grade distribution (filtered) ──
    let mut grade_dist: Vec<(String, i64)> = Vec::new();
    for g in &["S", "A", "B", "C"] {
        let n = companies
            .iter()
            .filter(|c| c.grade.as_deref() == Some(*g))
            .count() as i64;
        grade_dist.push((g.to_string(), n));
    }
    let ungraded = companies.iter().filter(|c| c.grade.is_none()).count() as i64;
    grade_dist.push(("—".to_string(), ungraded));
    let grade_max = grade_dist.iter().map(|(_, n)| *n).max().unwrap_or(1).max(1);

    // ── ATS / job-board health (filtered) ──
    let mut ats_map: BTreeMap<String, i64> = BTreeMap::new();
    for c in &companies {
        if let Some(p) = c.ats_provider.as_deref() {
            *ats_map.entry(p.to_string()).or_insert(0) += 1;
        }
        if c.status == "bespoke" {
            *ats_map.entry("bespoke".to_string()).or_insert(0) += 1;
        }
        if c.status == "potential" {
            *ats_map.entry("potential".to_string()).or_insert(0) += 1;
        }
    }
    let mut ats_labels: Vec<String> = Vec::with_capacity(ATS_ORDER.len());
    let mut ats_values: Vec<i64> = Vec::with_capacity(ATS_ORDER.len());
    let mut ats_colors: Vec<&str> = Vec::with_capacity(ATS_ORDER.len());
    for key in ATS_ORDER.iter() {
        ats_labels.push((*key).to_string());
        ats_values.push(ats_map.get(*key).copied().unwrap_or(0));
        ats_colors.push(ats_color(key));
    }
    let ats_json = serde_json::json!({
        "labels": ats_labels,
        "values": ats_values,
        "colors": ats_colors,
    });

    // ── Geography (filtered) ──
    // Bucket using the same SQL logic from analytics::geography_buckets,
    // reimplemented in Rust against the filtered list.
    let mut geo_map: BTreeMap<String, i64> = BTreeMap::new();
    for c in &companies {
        let bucket = match c.location.as_deref() {
            None | Some("") => "Unknown".to_string(),
            Some(loc) => {
                let l = loc;
                if l.contains("Remote") {
                    "Remote".to_string()
                } else if l.contains("London")
                    && (l.contains("New York")
                        || l.contains("Singapore")
                        || l.contains("global")
                        || l.contains("Global"))
                {
                    "London + intl".to_string()
                } else if l.contains("London") {
                    "London".to_string()
                } else if l.contains("Cambridge")
                    || l.contains("Oxford")
                    || l.contains("Bristol")
                    || l.contains("Manchester")
                    || l.contains("Edinburgh")
                    || l.contains("Southampton")
                    || l.contains("Birmingham")
                    || l.contains("Leeds")
                {
                    "UK ex-London".to_string()
                } else if l.contains("UK") {
                    "UK other".to_string()
                } else {
                    "International".to_string()
                }
            }
        };
        *geo_map.entry(bucket).or_insert(0) += 1;
    }
    let mut geo: Vec<(String, i64)> = geo_map.into_iter().collect();
    geo.sort_by(|a, b| b.1.cmp(&a.1));
    let geography_json = serde_json::json!({
        "items": geo.iter()
            .map(|(name, n)| serde_json::json!({"name": name, "value": n}))
            .collect::<Vec<_>>(),
    });

    // ── Sponsorship split (filtered) ──
    let sponsorship_order = ["yes", "unknown", "no"];
    let sponsorship_rows: Vec<(String, i64, &str)> = sponsorship_order
        .iter()
        .map(|k| {
            let n = companies
                .iter()
                .filter(|c| c.sponsors_uk.as_deref().unwrap_or("unknown") == *k)
                .count() as i64;
            let color = match *k {
                "yes" => "#4ade80",
                "unknown" => "#7a838f",
                _ => "#ff5c5c",
            };
            ((*k).to_string(), n, color)
        })
        .collect();
    let sponsorship_max = sponsorship_rows.iter().map(|(_, n, _)| *n).max().unwrap_or(1).max(1);

    // ── Grade freshness (filtered) ──
    // Re-query graded_at by id for the filtered set. Avoids per-row SQL by
    // fetching the (id, graded_at) map once.
    let mut graded_at_map: std::collections::HashMap<i64, Option<String>> =
        std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT id, graded_at FROM companies") {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?))
        }) {
            for row in rows.flatten() {
                graded_at_map.insert(row.0, row.1);
            }
        }
    }
    let now = chrono::Local::now().naive_local();
    let freshness_order = ["0-7d", "7-30d", "30-90d", "90d+", "ungraded"];
    let mut freshness_counts: BTreeMap<&str, i64> =
        freshness_order.iter().map(|k| (*k, 0)).collect();
    for c in &companies {
        let bucket = match graded_at_map.get(&c.id).and_then(|x| x.clone()) {
            None => "ungraded",
            Some(ts) => {
                let parsed = chrono::NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .or_else(|| {
                        chrono::NaiveDateTime::parse_from_str(&ts, "%Y-%m-%dT%H:%M:%S").ok()
                    });
                match parsed {
                    None => "ungraded",
                    Some(dt) => {
                        let days = (now - dt).num_days();
                        if days <= 7 {
                            "0-7d"
                        } else if days <= 30 {
                            "7-30d"
                        } else if days <= 90 {
                            "30-90d"
                        } else {
                            "90d+"
                        }
                    }
                }
            }
        };
        if let Some(slot) = freshness_counts.get_mut(bucket) {
            *slot += 1;
        }
    }
    let freshness: Vec<(String, i64)> = freshness_order
        .iter()
        .map(|k| ((*k).to_string(), freshness_counts.get(*k).copied().unwrap_or(0)))
        .collect();
    let freshness_max = freshness.iter().map(|(_, n)| *n).max().unwrap_or(1).max(1);

    // ── Top companies by job count (filtered) ──
    let mut top_sorted: Vec<&CompanyRow> = companies.iter().filter(|c| c.job_count > 0).collect();
    top_sorted.sort_by(|a, b| b.job_count.cmp(&a.job_count));
    let top_companies: Vec<(String, i64, i64)> = top_sorted
        .iter()
        .take(10)
        .map(|c| (c.name.clone(), c.job_count, c.fit_count))
        .collect();
    let top_max = top_companies.iter().map(|(_, n, _)| *n).max().unwrap_or(1).max(1);

    // ── Discovered timeline (filtered) ──
    // Fetch discovered_at per id from DB (filtered set only), then bucket by
    // day across the canonical 30-day axis.
    let mut discovered_at_map: std::collections::HashMap<i64, Option<String>> =
        std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT id, discovered_at FROM companies") {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?))
        }) {
            for row in rows.flatten() {
                discovered_at_map.insert(row.0, row.1);
            }
        }
    }
    let today = chrono::Local::now().date_naive();
    let start = today - chrono::Duration::days(29);
    let dates: Vec<String> = (0..30)
        .map(|i| (start + chrono::Duration::days(i)).format("%Y-%m-%d").to_string())
        .collect();
    let mut date_idx: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (i, d) in dates.iter().enumerate() {
        date_idx.insert(d.as_str(), i);
    }
    let mut values = vec![0i64; 30];
    for c in &companies {
        if let Some(Some(ts)) = discovered_at_map.get(&c.id) {
            let day = &ts[..ts.len().min(10)];
            if let Some(&i) = date_idx.get(day) {
                values[i] += 1;
            }
        }
    }
    let discovered_json = serde_json::json!({
        "dates": dates,
        "values": values,
    });

    let filter_strip_markup = render_filter_strip(q, total_companies, total_unfiltered);

    html! {
        div.companies-page {
            // Lane legend
            (lane_legend())

            // Filter strip
            (filter_strip_markup)

            // KPI strip
            section.kpi-strip {
                div.kpi {
                    div.kpi-label { "Companies" }
                    div.kpi-value data-ticker=(total_companies) { "0" }
                }
                div.kpi {
                    div.kpi-label { "S-tier" }
                    div.kpi-value data-ticker=(s_tier) { "0" }
                }
                div.kpi {
                    div.kpi-label { "A-tier" }
                    div.kpi-value data-ticker=(a_tier) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Bespoke" }
                    div.kpi-value data-ticker=(bespoke) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Potential" }
                    div.kpi-value data-ticker=(potential) { "0" }
                }
                div.kpi {
                    div.kpi-label { "Sponsors UK" }
                    div.kpi-value data-ticker=(sponsors_yes) { "0" }
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
                        @for (i, (name, total, strong)) in top_companies.iter().enumerate() {
                            @let pct = (total * 100 / top_max) as i32;
                            div.top-list-row {
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
                        @for c in &companies {
                            tr.company-row {
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
}

fn render_axis(
    q: &CompaniesQuery,
    label: &str,
    axis: &str,
    chips: &[&str],
    chip_label: impl Fn(&str) -> String,
) -> Markup {
    html! {
        div.filter-axis {
            span.filter-axis-label { (label) }
            div.chips {
                @for v in chips {
                    @let active = is_active(q, axis, v);
                    @let href = toggle_qs(q, axis, v);
                    @let class = if active { "chip chip-active" } else { "chip" };
                    a class=(class) href=(href) { (PreEscaped(chip_label(v))) }
                }
            }
        }
    }
}

fn render_filter_strip(q: &CompaniesQuery, shown: i64, total: i64) -> Markup {
    let any_active = !parse_csv(&q.lane).is_empty()
        || !parse_csv(&q.grade).is_empty()
        || !parse_csv(&q.status).is_empty()
        || !parse_csv(&q.ats).is_empty()
        || !parse_csv(&q.sponsor).is_empty()
        || !parse_csv(&q.location).is_empty()
        || !parse_csv(&q.has_jobs).is_empty();

    html! {
        div.filter-strip {
            (render_axis(q, "Lane", "lane", &LANE_KEYS, |v| lane_label(v).to_string()))
            (render_axis(q, "Grade", "grade", &GRADE_CHIPS, |v| v.to_string()))
            (render_axis(q, "Status", "status", &STATUS_CHIPS, |v| v.to_string()))
            (render_axis(q, "ATS", "ats", &ATS_FILTER_CHIPS, |v| v.to_string()))
            (render_axis(q, "Sponsor", "sponsor", &SPONSOR_CHIPS, |v| v.to_string()))
            (render_axis(q, "Location", "location", &LOCATION_CHIPS, |v| {
                match v {
                    "london" => "London".into(),
                    "uk_ex_london" => "UK ex-London".into(),
                    "remote" => "Remote".into(),
                    "intl" => "Intl".into(),
                    _ => "Unknown".into(),
                }
            }))
            (render_axis(q, "Has jobs", "has_jobs", &HAS_JOBS_CHIPS, |v| v.to_string()))
            div.filter-summary {
                span.filter-count { (shown) " of " (total) " companies" }
                @if any_active {
                    a.filter-reset href="/companies" { "reset all" }
                }
            }
        }
    }
}
