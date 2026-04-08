use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult};

const BASE_URL: &str = "https://boards-api.greenhouse.io/v1/boards";

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

/// Probe whether a Greenhouse board exists for this slug.
/// Returns the job count if the board exists, or None if 404.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    let url = format!("{BASE_URL}/{slug}/jobs");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let board: BoardResponse = resp.json().await.ok()?;
    if board.jobs.is_empty() {
        return None;
    }
    Some(SlugProbeResult {
        provider: "greenhouse",
        slug: slug.to_string(),
        job_count: board.jobs.len(),
    })
}

/// Fetch all jobs from a Greenhouse board (without descriptions).
/// Returns normalised `AtsJob` values. Fast — no content fetched.
pub async fn fetch_all(client: &reqwest::Client, slug: &str) -> Result<Vec<AtsJob>, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}/jobs");
    let resp = client.get(&url).send().await?.error_for_status()?;
    let board: BoardResponse = resp.json().await?;

    Ok(board.jobs.into_iter().map(|j| normalise(j, slug)).collect())
}

/// Fetch a single job's full description from Greenhouse.
pub async fn fetch_detail(
    client: &reqwest::Client,
    slug: &str,
    job_id: &str,
) -> Result<Option<String>, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}/jobs/{job_id}");
    let resp = client.get(&url).send().await?.error_for_status()?;
    let job: GreenhouseJob = resp.json().await?;
    Ok(job.content.map(|html| strip_html(&html)))
}

// ── Normalisation ────────────────────────────────────────────────

fn normalise(job: GreenhouseJob, _slug: &str) -> AtsJob {
    // Build all_locations from the primary location.name and any offices.
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

    // Add office locations if present.
    if let Some(offices) = &job.offices {
        for office in offices {
            all_locations.push(office.name.clone());
            if let Some(loc) = &office.location {
                all_locations.push(loc.clone());
            }
        }
    }

    // Infer remote policy from location.name.
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

/// Strip HTML tags from a string.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
