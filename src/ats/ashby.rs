use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult, get_with_retry};

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
    let resp = get_with_retry(client, &url, 2).await.ok()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_one(raw: &str) -> AtsJob {
        let board: BoardResponse = serde_json::from_str(raw).expect("parse");
        board.jobs.into_iter().map(normalise).next().expect("one job")
    }

    #[test]
    fn normalise_basic() {
        let raw = r#"{
            "jobs": [{
                "id": "job-123",
                "title": "Junior Rust Engineer",
                "location": "London",
                "employmentType": "FullTime",
                "jobUrl": "https://jobs.ashbyhq.com/acme/job-123",
                "descriptionPlain": "We are hiring."
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.external_id, "job-123");
        assert_eq!(job.title, "Junior Rust Engineer");
        assert_eq!(job.location.as_deref(), Some("London"));
        assert_eq!(job.description.as_deref(), Some("We are hiring."));
        assert!(job.all_locations.contains(&"London".to_string()));
    }

    #[test]
    fn normalise_postal_address_decomposed() {
        let raw = r#"{
            "jobs": [{
                "id": "x",
                "title": "x",
                "location": "London, UK",
                "jobUrl": "x",
                "address": {
                    "postalAddress": {
                        "addressLocality": "London",
                        "addressRegion": "England",
                        "addressCountry": "GB"
                    }
                }
            }]
        }"#;
        let job = parse_one(raw);
        assert!(job.all_locations.contains(&"London, UK".to_string()));
        assert!(job.all_locations.contains(&"London".to_string()));
        assert!(job.all_locations.contains(&"England".to_string()));
        assert!(job.all_locations.contains(&"GB".to_string()));
    }

    #[test]
    fn normalise_secondary_locations_collected() {
        let raw = r#"{
            "jobs": [{
                "id": "x",
                "title": "x",
                "location": "London",
                "jobUrl": "x",
                "secondaryLocations": [
                    {"location": "Paris"},
                    {"location": "Berlin"},
                    {"location": null}
                ]
            }]
        }"#;
        let job = parse_one(raw);
        assert!(job.all_locations.contains(&"Paris".to_string()));
        assert!(job.all_locations.contains(&"Berlin".to_string()));
    }

    #[test]
    fn normalise_workplace_type_takes_precedence() {
        let raw = r#"{
            "jobs": [{
                "id": "x",
                "title": "x",
                "location": "London",
                "jobUrl": "x",
                "workplaceType": "Hybrid",
                "isRemote": true
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy.as_deref(), Some("Hybrid"));
    }

    #[test]
    fn normalise_is_remote_fallback() {
        let raw = r#"{
            "jobs": [{
                "id": "x",
                "title": "x",
                "location": "Anywhere",
                "jobUrl": "x",
                "isRemote": true
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy.as_deref(), Some("Remote"));
    }

    #[test]
    fn normalise_is_remote_false_yields_none() {
        let raw = r#"{
            "jobs": [{
                "id": "x",
                "title": "x",
                "location": "London",
                "jobUrl": "x",
                "isRemote": false
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy, None);
    }

    #[test]
    fn normalise_posted_date_passthrough() {
        let raw = r#"{
            "jobs": [{
                "id": "x",
                "title": "x",
                "location": "London",
                "jobUrl": "x",
                "publishedAt": "2026-04-01T10:00:00Z"
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.posted_date.as_deref(), Some("2026-04-01T10:00:00Z"));
    }

    #[test]
    fn normalise_multiple_jobs_in_board() {
        let raw = r#"{
            "jobs": [
                {"id": "a", "title": "A", "location": "x", "jobUrl": "ua"},
                {"id": "b", "title": "B", "location": "y", "jobUrl": "ub"}
            ]
        }"#;
        let board: BoardResponse = serde_json::from_str(raw).expect("parse");
        let jobs: Vec<_> = board.jobs.into_iter().map(normalise).collect();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].title, "A");
        assert_eq!(jobs[1].title, "B");
    }
}
