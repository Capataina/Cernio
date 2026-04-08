use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult};

const BASE_URL: &str = "https://api.ashbyhq.com/posting-api/job-board";

// ── API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct BoardResponse {
    jobs: Vec<AshbyJob>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AshbyJob {
    id: String,
    title: String,
    location: String,
    #[serde(rename = "secondaryLocations")]
    secondary_locations: Option<Vec<AshbySecondaryLocation>>,
    #[serde(rename = "employmentType")]
    employment_type: Option<String>,
    #[serde(rename = "isRemote")]
    is_remote: Option<bool>,
    #[serde(rename = "workplaceType")]
    workplace_type: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    #[serde(rename = "jobUrl")]
    job_url: String,
    #[serde(rename = "descriptionPlain")]
    description_plain: Option<String>,
    address: Option<AshbyAddress>,
}

#[derive(Debug, Deserialize)]
struct AshbySecondaryLocation {
    location: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AshbyAddress {
    #[serde(rename = "postalAddress")]
    postal_address: Option<AshbyPostalAddress>,
}

#[derive(Debug, Deserialize)]
struct AshbyPostalAddress {
    #[serde(rename = "addressLocality")]
    address_locality: Option<String>,
    #[serde(rename = "addressRegion")]
    address_region: Option<String>,
    #[serde(rename = "addressCountry")]
    address_country: Option<String>,
}

// ── Public interface ─────────────────────────────────────────────

/// Probe whether an Ashby board exists for this slug.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    let url = format!("{BASE_URL}/{slug}");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let board: BoardResponse = resp.json().await.ok()?;
    if board.jobs.is_empty() {
        return None;
    }
    Some(SlugProbeResult {
        provider: "ashby",
        slug: slug.to_string(),
        job_count: board.jobs.len(),
    })
}

/// Fetch all jobs from an Ashby board.
/// Ashby returns full descriptions in the list endpoint — no second request needed.
pub async fn fetch_all(client: &reqwest::Client, slug: &str) -> Result<Vec<AtsJob>, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}");
    let resp = client.get(&url).send().await?.error_for_status()?;
    let board: BoardResponse = resp.json().await?;

    Ok(board.jobs.into_iter().map(normalise).collect())
}

// ── Normalisation ────────────────────────────────────────────────

fn normalise(job: AshbyJob) -> AtsJob {
    let mut all_locations = vec![job.location.clone()];

    // Add structured address components.
    if let Some(addr) = &job.address {
        if let Some(postal) = &addr.postal_address {
            if let Some(locality) = &postal.address_locality {
                all_locations.push(locality.clone());
            }
            if let Some(region) = &postal.address_region {
                all_locations.push(region.clone());
            }
            if let Some(country) = &postal.address_country {
                all_locations.push(country.clone());
            }
        }
    }

    // Add secondary locations.
    if let Some(secondary) = &job.secondary_locations {
        for loc in secondary {
            if let Some(name) = &loc.location {
                all_locations.push(name.clone());
            }
        }
    }

    // Remote/workplace policy.
    let remote_policy = job
        .workplace_type
        .clone()
        .or_else(|| {
            job.is_remote
                .and_then(|r| if r { Some("Remote".to_string()) } else { None })
        });

    AtsJob {
        external_id: job.id,
        title: job.title,
        url: job.job_url,
        location: Some(job.location),
        all_locations,
        remote_policy,
        posted_date: job.published_at,
        description: job.description_plain,
    }
}
