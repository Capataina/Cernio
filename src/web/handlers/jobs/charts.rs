//! In-handler analytics: every aggregate is filter-scoped.

use maud::{html, Markup};

use crate::data::lane::{primary_lane, LANE_KEYS};
use crate::data::models::JobRow;

use super::filters::set_href;

pub(super) fn decision_funnel_from(jobs: &[JobRow]) -> Vec<(String, i64)> {
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

pub(super) fn lookup_funnel(funnel: &[(String, i64)], key: &str) -> i64 {
    funnel
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, n)| *n)
        .unwrap_or(0)
}

pub(super) fn freshness_from(jobs: &[JobRow]) -> Vec<(String, i64)> {
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

pub(super) fn heatmap_from(jobs: &[JobRow]) -> Vec<(String, Vec<i64>)> {
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

pub(super) fn top_companies_from(jobs: &[JobRow], n: usize) -> Vec<(String, i64)> {
    let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for j in jobs {
        *map.entry(j.company_name.clone()).or_insert(0) += 1;
    }
    let mut v: Vec<(String, i64)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

pub(super) fn top_titles_from(jobs: &[JobRow], n: usize) -> Vec<(String, i64)> {
    let mut map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for j in jobs {
        *map.entry(j.title.clone()).or_insert(0) += 1;
    }
    let mut v: Vec<(String, i64)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

pub(super) fn freshness_median_bucket(buckets: &[(String, i64)]) -> String {
    buckets
        .iter()
        .filter(|(k, _)| k != "unknown")
        .max_by_key(|(_, n)| *n)
        .filter(|(_, n)| *n > 0)
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| "—".to_string())
}

/// Click-navigable heatmap cell linking to /jobs?lane=L&grade=G.
pub(super) fn heatmap_cell_link(count: i64, max: i64, grade: &str, lane: &str) -> Markup {
    let shade = if count == 0 || max == 0 {
        "zero"
    } else {
        let ratio = count as f64 / max as f64;
        if ratio < 0.15 { "low" } else if ratio < 0.5 { "mid" } else { "hot" }
    };
    let class = format!("heatmap-cell heatmap-cell-link {shade}");
    let href = set_href(&[("lane", lane), ("grade", grade)]);
    html! {
        a class=(class) href=(href) title=(format!("{} · {}: {} jobs", grade, lane, count)) {
            (count)
        }
    }
}
