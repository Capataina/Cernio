use rusqlite::Connection;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use crate::data::lane::{all_lanes, lane_hex, lane_label, LANE_KEYS};
use crate::data::models::CompanyRow;

use super::filters::{ats_color, geo_bucket_to_filter_key, ATS_ORDER};

/// KPI counts displayed in the top strip (over the filtered set).
pub(super) struct Kpis {
    pub total_companies: i64,
    pub s_tier: i64,
    pub a_tier: i64,
    pub bespoke: i64,
    pub potential: i64,
    pub sponsors_yes: i64,
}

pub(super) fn kpis(companies: &[CompanyRow]) -> Kpis {
    Kpis {
        total_companies: companies.len() as i64,
        s_tier: companies.iter().filter(|c| c.grade.as_deref() == Some("S")).count() as i64,
        a_tier: companies.iter().filter(|c| c.grade.as_deref() == Some("A")).count() as i64,
        bespoke: companies.iter().filter(|c| c.status == "bespoke").count() as i64,
        potential: companies.iter().filter(|c| c.status == "potential").count() as i64,
        sponsors_yes: companies
            .iter()
            .filter(|c| c.sponsors_uk.as_deref() == Some("yes"))
            .count() as i64,
    }
}

/// Companies × lane donut JSON island payload.
pub(super) fn companies_lane_json(companies: &[CompanyRow]) -> JsonValue {
    let mut lane_counts: BTreeMap<&str, i64> = LANE_KEYS.iter().map(|k| (*k, 0)).collect();
    for c in companies {
        for l in all_lanes(c.lanes.as_deref()) {
            if let Some(slot) = lane_counts.get_mut(l.as_str()) {
                *slot += 1;
            }
        }
    }
    serde_json::json!({
        "items": LANE_KEYS.iter().map(|k| serde_json::json!({
            "label": lane_label(k),
            "key": k,
            "value": lane_counts.get(*k).copied().unwrap_or(0),
            "color": lane_hex(k),
        })).collect::<Vec<_>>(),
    })
}

/// Grade distribution: (label, count) rows + the max for bar-width scaling.
pub(super) fn grade_distribution(companies: &[CompanyRow]) -> (Vec<(String, i64)>, i64) {
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
    (grade_dist, grade_max)
}

/// ATS / job-board health bar-chart JSON.
pub(super) fn ats_health_json(companies: &[CompanyRow]) -> JsonValue {
    let mut ats_map: BTreeMap<String, i64> = BTreeMap::new();
    for c in companies {
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
    serde_json::json!({
        "labels": ats_labels,
        "values": ats_values,
        "colors": ats_colors,
    })
}

/// Geography donut JSON. Bucketing mirrors `analytics::geography_buckets`,
/// reimplemented in Rust against the filtered list.
pub(super) fn geography_json(companies: &[CompanyRow]) -> JsonValue {
    let mut geo_map: BTreeMap<String, i64> = BTreeMap::new();
    for c in companies {
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
    serde_json::json!({
        "items": geo.iter()
            .map(|(name, n)| serde_json::json!({
                "name": name,
                "value": n,
                "key": geo_bucket_to_filter_key(name),
            }))
            .collect::<Vec<_>>(),
    })
}

/// Sponsorship bar rows: (label, count, color) + the max for bar-width scaling.
pub(super) fn sponsorship_rows(companies: &[CompanyRow]) -> (Vec<(String, i64, &'static str)>, i64) {
    let sponsorship_order = ["yes", "unknown", "no"];
    let rows: Vec<(String, i64, &str)> = sponsorship_order
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
    let max = rows.iter().map(|(_, n, _)| *n).max().unwrap_or(1).max(1);
    (rows, max)
}

/// Grade freshness bar rows + the max for bar-width scaling.
///
/// Re-queries `graded_at` by id for the filtered set. Avoids per-row SQL by
/// fetching the (id, graded_at) map once.
pub(super) fn freshness_rows(
    conn: &Connection,
    companies: &[CompanyRow],
) -> (Vec<(String, i64)>, i64) {
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
    for c in companies {
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
    let max = freshness.iter().map(|(_, n)| *n).max().unwrap_or(1).max(1);
    (freshness, max)
}

/// Top-10 companies by job count: (id, name, total, fit_count) + the max
/// for bar-width scaling.
pub(super) fn top_companies(companies: &[CompanyRow]) -> (Vec<(i64, String, i64, i64)>, i64) {
    let mut top_sorted: Vec<&CompanyRow> = companies.iter().filter(|c| c.job_count > 0).collect();
    top_sorted.sort_by(|a, b| b.job_count.cmp(&a.job_count));
    let top: Vec<(i64, String, i64, i64)> = top_sorted
        .iter()
        .take(10)
        .map(|c| (c.id, c.name.clone(), c.job_count, c.fit_count))
        .collect();
    let max = top.iter().map(|(_, _, n, _)| *n).max().unwrap_or(1).max(1);
    (top, max)
}

/// 30-day discovered-timeline JSON for the line chart.
///
/// Fetches `discovered_at` per id from DB (filtered set only), then buckets
/// by day across the canonical 30-day axis.
pub(super) fn discovered_json(conn: &Connection, companies: &[CompanyRow]) -> JsonValue {
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
    for c in companies {
        if let Some(Some(ts)) = discovered_at_map.get(&c.id) {
            let day = &ts[..ts.len().min(10)];
            if let Some(&i) = date_idx.get(day) {
                values[i] += 1;
            }
        }
    }
    serde_json::json!({
        "dates": dates,
        "values": values,
    })
}
