use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult};

const BASE_URL: &str = "https://boards-api.greenhouse.io/v1/boards";
const BASE_URL_EU: &str = "https://boards-api.eu.greenhouse.io/v1/boards";

// ── API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct BoardResponse {
    jobs: Vec<GreenhouseJob>,
}

#[derive(Debug, Deserialize)]
struct GreenhouseJob {
    id: u64,
    title: String,
    absolute_url: String,
    location: GreenhouseLocation,
    #[serde(rename = "first_published")]
    first_published: Option<String>,
    #[serde(rename = "updated_at")]
    updated_at: Option<String>,
    /// Full HTML description — only present with ?content=true or detail endpoint.
    content: Option<String>,
    /// Office locations — only present with ?content=true or detail endpoint.
    offices: Option<Vec<GreenhouseOffice>>,
}

#[derive(Debug, Deserialize)]
struct GreenhouseLocation {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GreenhouseOffice {
    name: String,
    location: Option<String>,
}

// ── Public interface ─────────────────────────────────────────────

/// Determine the correct base URL based on ats_extra region.
fn base_url(ats_extra: Option<&str>) -> &'static str {
    if let Some(extra) = ats_extra {
        if extra.contains("\"eu\"") || extra.contains("eu") {
            return BASE_URL_EU;
        }
    }
    BASE_URL
}

/// Probe whether a Greenhouse board exists for this slug.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    // Try US first, then EU.
    for url_base in [BASE_URL, BASE_URL_EU] {
        let url = format!("{url_base}/{slug}/jobs");
        let resp = client.get(&url).send().await.ok()?;
        if !resp.status().is_success() {
            continue;
        }
        let board: BoardResponse = resp.json().await.ok()?;
        if !board.jobs.is_empty() {
            return Some(SlugProbeResult {
                provider: "greenhouse",
                slug: slug.to_string(),
                job_count: board.jobs.len(),
            });
        }
    }
    None
}

/// Fetch all jobs from a Greenhouse board WITH full descriptions.
/// Uses ?content=true to include HTML descriptions in a single request.
pub async fn fetch_all(client: &reqwest::Client, slug: &str) -> Result<Vec<AtsJob>, reqwest::Error> {
    fetch_all_with_extra(client, slug, None).await
}

/// Fetch all jobs with optional ats_extra for EU region support.
pub async fn fetch_all_with_extra(
    client: &reqwest::Client,
    slug: &str,
    ats_extra: Option<&str>,
) -> Result<Vec<AtsJob>, reqwest::Error> {
    let url_base = base_url(ats_extra);
    let url = format!("{url_base}/{slug}/jobs?content=true");
    let resp = client.get(&url).send().await?.error_for_status()?;
    let board: BoardResponse = resp.json().await?;

    Ok(board.jobs.into_iter().map(|j| normalise(j)).collect())
}

// ── Normalisation ────────────────────────────────────────────────

fn normalise(job: GreenhouseJob) -> AtsJob {
    let mut all_locations = vec![job.location.name.clone()];

    // location.name can contain semicolon-separated cities (e.g. "Berlin; London; Munich").
    if job.location.name.contains(';') {
        for part in job.location.name.split(';') {
            let trimmed = part.trim().to_string();
            if !trimmed.is_empty() {
                all_locations.push(trimmed);
            }
        }
    }

    // Add office locations if present (from ?content=true).
    if let Some(offices) = &job.offices {
        for office in offices {
            all_locations.push(office.name.clone());
            if let Some(loc) = &office.location {
                all_locations.push(loc.clone());
            }
        }
    }

    let remote_policy = if job.location.name.to_lowercase().contains("remote") {
        Some("Remote".to_string())
    } else if job.location.name.to_lowercase().contains("hybrid") {
        Some("Hybrid".to_string())
    } else {
        None
    };

    AtsJob {
        external_id: job.id.to_string(),
        title: job.title,
        url: job.absolute_url,
        location: Some(job.location.name),
        all_locations,
        remote_policy,
        posted_date: job.first_published.or(job.updated_at),
        description: job.content.map(|html| strip_html(&html)),
    }
}

fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
