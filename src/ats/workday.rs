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
