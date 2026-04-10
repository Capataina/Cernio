use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult};

// ── API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WorkdayResponse {
    #[serde(rename = "jobPostings")]
    job_postings: Vec<WorkdayJob>,
    total: u64,
}

#[derive(Debug, Deserialize)]
struct WorkdayJob {
    #[serde(rename = "bulletFields")]
    bullet_fields: Option<Vec<String>>,
    title: String,
    #[serde(rename = "externalPath")]
    external_path: String,
    #[serde(rename = "locationsText")]
    locations_text: Option<String>,
    #[serde(rename = "postedOn")]
    posted_on: Option<String>,
}

/// Parse ats_extra JSON to get Workday connection parameters.
/// Expected format: {"subdomain":"gresearch","wd":"wd103","site":"G-Research"}
fn parse_extra(ats_extra: &str) -> Option<(String, String, String)> {
    let v: serde_json::Value = serde_json::from_str(ats_extra).ok()?;
    let subdomain = v.get("subdomain")?.as_str()?.to_string();
    let wd = v.get("wd")?.as_str()?.to_string();
    let site = v.get("site")?.as_str()?.to_string();
    Some((subdomain, wd, site))
}

fn build_base_url(subdomain: &str, wd: &str, site: &str) -> String {
    format!(
        "https://{subdomain}.{wd}.myworkdayjobs.com/wday/cxs/{subdomain}/{site}/jobs"
    )
}

fn build_posting_url(subdomain: &str, wd: &str, site: &str, path: &str) -> String {
    format!(
        "https://{subdomain}.{wd}.myworkdayjobs.com/en-US/{site}{path}"
    )
}

// ── Public interface ─────────────────────────────────────────────

/// Probe whether a Workday board exists with the given ats_extra parameters.
pub async fn probe_with_extra(
    client: &reqwest::Client,
    ats_extra: &str,
) -> Option<SlugProbeResult> {
    let (subdomain, wd, site) = parse_extra(ats_extra)?;
    let url = build_base_url(&subdomain, &wd, &site);

    let body = serde_json::json!({
        "appliedFacets": {},
        "limit": 1,
        "offset": 0,
        "searchText": ""
    });

    let resp = client.post(&url).json(&body).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let data: WorkdayResponse = resp.json().await.ok()?;
    if data.total == 0 {
        return None;
    }

    Some(SlugProbeResult {
        provider: "workday",
        slug: subdomain,
        job_count: data.total as usize,
    })
}

/// Fetch all jobs from a Workday board. Handles pagination.
pub async fn fetch_all_with_extra(
    client: &reqwest::Client,
    _slug: &str,
    ats_extra: &str,
) -> Result<Vec<AtsJob>, Box<dyn std::error::Error + Send + Sync>> {
    let (subdomain, wd, site) = parse_extra(ats_extra)
        .ok_or("Invalid ats_extra for Workday")?;

    let url = build_base_url(&subdomain, &wd, &site);
    let mut all_jobs = Vec::new();
    let mut offset = 0u64;
    let limit = 20u64; // Workday typically limits to 20 per page.

    loop {
        let body = serde_json::json!({
            "appliedFacets": {},
            "limit": limit,
            "offset": offset,
            "searchText": ""
        });

        let resp = client.post(&url).json(&body).send().await?.error_for_status()?;
        let data: WorkdayResponse = resp.json().await?;

        let page_count = data.job_postings.len();
        for job in data.job_postings {
            all_jobs.push(normalise(job, &subdomain, &wd, &site));
        }

        offset += limit;
        if page_count < limit as usize || offset >= data.total {
            break;
        }
    }

    Ok(all_jobs)
}

// ── Normalisation ────────────────────────────────────────────────

fn normalise(job: WorkdayJob, subdomain: &str, wd: &str, site: &str) -> AtsJob {
    let mut all_locations = Vec::new();
    if let Some(loc) = &job.locations_text {
        all_locations.push(loc.clone());
        // Split on " | " which Workday uses for multiple locations.
        for part in loc.split(" | ") {
            let trimmed = part.trim().to_string();
            if !trimmed.is_empty() && trimmed != *loc {
                all_locations.push(trimmed);
            }
        }
    }

    let remote_policy = job.locations_text.as_deref().and_then(|loc| {
        let lower = loc.to_lowercase();
        if lower.contains("remote") {
            Some("Remote".to_string())
        } else if lower.contains("hybrid") {
            Some("Hybrid".to_string())
        } else {
            None
        }
    });

    // Bullet fields often contain the description summary.
    let description = job.bullet_fields.as_ref().map(|fields| fields.join("\n"));

    AtsJob {
        external_id: job.external_path.clone(),
        title: job.title,
        url: build_posting_url(subdomain, wd, site, &job.external_path),
        location: job.locations_text,
        all_locations,
        remote_policy,
        posted_date: job.posted_on,
        description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────
    // parse_extra: the ats_extra JSON format for Workday
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_extra_valid() {
        let (sub, wd, site) = parse_extra(
            r#"{"subdomain":"gresearch","wd":"wd103","site":"G-Research"}"#,
        )
        .expect("parse");
        assert_eq!(sub, "gresearch");
        assert_eq!(wd, "wd103");
        assert_eq!(site, "G-Research");
    }

    #[test]
    fn parse_extra_missing_field_returns_none() {
        assert!(parse_extra(r#"{"subdomain":"x"}"#).is_none());
        assert!(parse_extra(r#"{"subdomain":"x","wd":"y"}"#).is_none());
    }

    #[test]
    fn parse_extra_invalid_json_returns_none() {
        assert!(parse_extra("not json").is_none());
        assert!(parse_extra("").is_none());
    }

    #[test]
    fn parse_extra_non_string_field_returns_none() {
        assert!(parse_extra(r#"{"subdomain":123,"wd":"y","site":"z"}"#).is_none());
    }

    // ─────────────────────────────────────────────────────────────
    // URL construction
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn build_base_url_shape() {
        let url = build_base_url("gresearch", "wd103", "G-Research");
        assert_eq!(
            url,
            "https://gresearch.wd103.myworkdayjobs.com/wday/cxs/gresearch/G-Research/jobs"
        );
    }

    #[test]
    fn build_posting_url_joins_external_path() {
        let url = build_posting_url("gresearch", "wd103", "G-Research", "/job/London/SWE_123");
        assert_eq!(
            url,
            "https://gresearch.wd103.myworkdayjobs.com/en-US/G-Research/job/London/SWE_123"
        );
    }

    // ─────────────────────────────────────────────────────────────
    // normalise: locations, remote policy, description from bullets
    // ─────────────────────────────────────────────────────────────

    fn parse_one(raw: &str) -> AtsJob {
        let resp: WorkdayResponse = serde_json::from_str(raw).expect("parse");
        resp.job_postings
            .into_iter()
            .map(|j| normalise(j, "gresearch", "wd103", "G-Research"))
            .next()
            .expect("one job")
    }

    #[test]
    fn normalise_single_location() {
        let raw = r#"{
            "jobPostings": [{
                "title": "Rust Engineer",
                "externalPath": "/job/London/Rust_1",
                "locationsText": "London, United Kingdom",
                "postedOn": "Posted 2 days ago"
            }],
            "total": 1
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.title, "Rust Engineer");
        assert_eq!(job.location.as_deref(), Some("London, United Kingdom"));
        assert!(job.all_locations.contains(&"London, United Kingdom".to_string()));
        assert_eq!(job.posted_date.as_deref(), Some("Posted 2 days ago"));
        assert!(job.url.ends_with("/job/London/Rust_1"));
    }

    #[test]
    fn normalise_multi_location_pipe_separated() {
        let raw = r#"{
            "jobPostings": [{
                "title": "x",
                "externalPath": "/x",
                "locationsText": "London, UK | Cambridge, UK | Manchester, UK"
            }],
            "total": 1
        }"#;
        let job = parse_one(raw);
        // Full string preserved, plus each split part added separately.
        assert!(job.all_locations.contains(&"London, UK | Cambridge, UK | Manchester, UK".to_string()));
        assert!(job.all_locations.contains(&"London, UK".to_string()));
        assert!(job.all_locations.contains(&"Cambridge, UK".to_string()));
        assert!(job.all_locations.contains(&"Manchester, UK".to_string()));
    }

    #[test]
    fn normalise_remote_from_locations_text() {
        let raw = r#"{
            "jobPostings": [{
                "title": "x",
                "externalPath": "/x",
                "locationsText": "Remote - United Kingdom"
            }],
            "total": 1
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy.as_deref(), Some("Remote"));
    }

    #[test]
    fn normalise_hybrid_from_locations_text() {
        let raw = r#"{
            "jobPostings": [{
                "title": "x",
                "externalPath": "/x",
                "locationsText": "London (Hybrid)"
            }],
            "total": 1
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy.as_deref(), Some("Hybrid"));
    }

    #[test]
    fn normalise_bullet_fields_become_description() {
        let raw = r#"{
            "jobPostings": [{
                "title": "x",
                "externalPath": "/x",
                "locationsText": "London",
                "bulletFields": ["Rust", "Linux", "5+ years"]
            }],
            "total": 1
        }"#;
        let job = parse_one(raw);
        let desc = job.description.expect("description");
        assert!(desc.contains("Rust"));
        assert!(desc.contains("Linux"));
        assert!(desc.contains("5+ years"));
    }

    #[test]
    fn normalise_no_bullets_no_description() {
        let raw = r#"{
            "jobPostings": [{
                "title": "x",
                "externalPath": "/x",
                "locationsText": "London"
            }],
            "total": 1
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.description, None);
    }

    #[test]
    fn normalise_zero_total_response() {
        let raw = r#"{"jobPostings": [], "total": 0}"#;
        let resp: WorkdayResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(resp.total, 0);
        assert!(resp.job_postings.is_empty());
    }
}
