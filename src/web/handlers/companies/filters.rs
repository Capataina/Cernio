use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::collections::HashSet;

use crate::data::lane::all_lanes;
use crate::data::models::CompanyRow;

pub(super) const ATS_ORDER: [&str; 9] = [
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

pub(super) const GRADE_CHIPS: [&str; 5] = ["S", "A", "B", "C", "?"];
pub(super) const STATUS_CHIPS: [&str; 3] = ["resolved", "bespoke", "potential"];
pub(super) const ATS_FILTER_CHIPS: [&str; 8] = [
    "greenhouse",
    "ashby",
    "lever",
    "workable",
    "smartrecruiters",
    "workday",
    "eightfold",
    "bespoke",
];
pub(super) const SPONSOR_CHIPS: [&str; 3] = ["yes", "unknown", "no"];
pub(super) const LOCATION_CHIPS: [&str; 5] =
    ["london", "uk_ex_london", "remote", "intl", "unknown"];
pub(super) const HAS_JOBS_CHIPS: [&str; 2] = ["yes", "no"];

/// Map the human-readable geography bucket the donut renders into the URL
/// `location=` filter key consumed by `CompaniesQuery`. The geography donut
/// has finer buckets than the filter axis, so a few names collapse together
/// (e.g. "UK other" → uk_ex_london).
pub(super) fn geo_bucket_to_filter_key(name: &str) -> &'static str {
    match name {
        "Remote" => "remote",
        "London" | "London + intl" => "london",
        "UK ex-London" | "UK other" => "uk_ex_london",
        "International" => "intl",
        _ => "unknown",
    }
}

pub(super) fn ats_color(key: &str) -> &'static str {
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

pub(super) fn grade_color(g: &str) -> &'static str {
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

pub(super) fn parse_csv(s: &Option<String>) -> HashSet<String> {
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
pub(super) fn location_buckets(loc: Option<&str>) -> Vec<&'static str> {
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
pub(super) fn toggle_qs(q: &CompaniesQuery, axis: &str, value: &str) -> String {
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

pub(super) fn is_active(q: &CompaniesQuery, axis: &str, value: &str) -> bool {
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

pub(super) fn filter_companies(all: &[CompanyRow], q: &CompaniesQuery) -> Vec<CompanyRow> {
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

#[derive(Clone, Copy)]
pub(super) enum ChipKind {
    Lane,
    Grade,
    Plain,
    Segmented,
    SegmentedGood,
}

pub(super) fn chip_classes_style(kind: ChipKind, val: &str, active: bool) -> (String, String) {
    let base = if active { "chip chip-active" } else { "chip" };
    match kind {
        ChipKind::Lane => {
            let hex = crate::data::lane::lane_hex(val);
            (format!("{base} chip-lane"), format!("--lane-color: {hex}"))
        }
        ChipKind::Grade => {
            let g = match val {
                "S" => "chip-grade-s",
                "A" => "chip-grade-a",
                "B" => "chip-grade-b",
                "C" => "chip-grade-c",
                _   => "chip-grade-ungraded",
            };
            (format!("{base} chip-grade {g}"), String::new())
        }
        ChipKind::Plain => (format!("{base} chip-plain"), String::new()),
        ChipKind::Segmented | ChipKind::SegmentedGood => (base.to_string(), String::new()),
    }
}

pub(super) fn render_axis(
    q: &CompaniesQuery,
    label: &str,
    axis: &str,
    chips: &[&str],
    kind: ChipKind,
    chip_label: impl Fn(&str) -> String,
) -> Markup {
    let is_seg = matches!(kind, ChipKind::Segmented | ChipKind::SegmentedGood);
    if is_seg {
        let group_class = match kind {
            ChipKind::SegmentedGood => "seg-group seg-good",
            _ => "seg-group",
        };
        return html! {
            div.filter-axis {
                span.filter-axis-label { (label) }
                div class=(group_class) {
                    @for v in chips {
                        @let active = is_active(q, axis, v);
                        @let href = toggle_qs(q, axis, v);
                        a class="seg" href=(href) data-active=(active) { (PreEscaped(chip_label(v))) }
                    }
                }
            }
        };
    }
    html! {
        div.filter-axis {
            span.filter-axis-label { (label) }
            div.chips {
                @for v in chips {
                    @let active = is_active(q, axis, v);
                    @let href = toggle_qs(q, axis, v);
                    @let (class, style) = chip_classes_style(kind, v, active);
                    a class=(class) href=(href) style=(style) data-active=(active) {
                        (PreEscaped(chip_label(v)))
                    }
                }
            }
        }
    }
}

pub(super) fn render_filter_strip(q: &CompaniesQuery, shown: i64, total: i64) -> Markup {
    use crate::data::lane::{lane_label, LANE_KEYS};

    let any_active = !parse_csv(&q.lane).is_empty()
        || !parse_csv(&q.grade).is_empty()
        || !parse_csv(&q.status).is_empty()
        || !parse_csv(&q.ats).is_empty()
        || !parse_csv(&q.sponsor).is_empty()
        || !parse_csv(&q.location).is_empty()
        || !parse_csv(&q.has_jobs).is_empty();

    html! {
        div.filter-strip {
            (render_axis(q, "Lane", "lane", &LANE_KEYS, ChipKind::Lane, |v| lane_label(v).to_string()))
            (render_axis(q, "Grade", "grade", &GRADE_CHIPS, ChipKind::Grade, |v| v.to_string()))
            (render_axis(q, "Status", "status", &STATUS_CHIPS, ChipKind::Plain, |v| v.to_string()))
            (render_axis(q, "ATS", "ats", &ATS_FILTER_CHIPS, ChipKind::Plain, |v| v.to_string()))
            (render_axis(q, "Sponsor", "sponsor", &SPONSOR_CHIPS, ChipKind::SegmentedGood, |v| v.to_string()))
            (render_axis(q, "Location", "location", &LOCATION_CHIPS, ChipKind::Plain, |v| {
                match v {
                    "london" => "London".into(),
                    "uk_ex_london" => "UK ex-London".into(),
                    "remote" => "Remote".into(),
                    "intl" => "Intl".into(),
                    _ => "Unknown".into(),
                }
            }))
            (render_axis(q, "Has jobs", "has_jobs", &HAS_JOBS_CHIPS, ChipKind::Segmented, |v| v.to_string()))
            div.filter-summary-row {
                div.filter-summary {
                    span.filter-count { (shown) }
                    span.filter-count-total { " of " (total) }
                    span.filter-count-label { @if any_active { " filtered companies" } @else { " companies" } }
                }
                @if any_active {
                    a.filter-reset href="/companies" { "reset all" }
                }
            }
        }
    }
}
