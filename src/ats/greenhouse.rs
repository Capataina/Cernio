use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult, get_with_retry};

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
/// Tries both US and EU endpoints with retry on timeout/connection errors.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    for url_base in [BASE_URL, BASE_URL_EU] {
        let url = format!("{url_base}/{slug}/jobs");
        let resp = get_with_retry(client, &url, 2).await.ok()?;
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
/// Uses retry to handle transient timeouts that would silently return zero jobs.
pub async fn fetch_all_with_extra(
    client: &reqwest::Client,
    slug: &str,
    ats_extra: Option<&str>,
) -> Result<Vec<AtsJob>, reqwest::Error> {
    let url_base = base_url(ats_extra);
    let url = format!("{url_base}/{slug}/jobs?content=true");
    let resp = get_with_retry(client, &url, 2).await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
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
    let mut quote_char: Option<char> = None;
    for ch in html.chars() {
        if in_tag {
            match quote_char {
                Some(q) if ch == q => quote_char = None,
                Some(_) => {}
                None if ch == '"' || ch == '\'' => quote_char = Some(ch),
                None if ch == '>' => {
                    in_tag = false;
                }
                None => {}
            }
        } else if ch == '<' {
            in_tag = true;
            quote_char = None;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────
    // base_url
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn base_url_defaults_to_us() {
        assert_eq!(base_url(None), BASE_URL);
    }

    #[test]
    fn base_url_eu_json() {
        assert_eq!(base_url(Some(r#"{"region":"eu"}"#)), BASE_URL_EU);
    }

    #[test]
    fn base_url_eu_bare() {
        assert_eq!(base_url(Some("eu")), BASE_URL_EU);
    }

    #[test]
    fn base_url_us() {
        assert_eq!(base_url(Some("us-east")), BASE_URL);
    }

    // ─────────────────────────────────────────────────────────────
    // normalise + JSON parsing
    // ─────────────────────────────────────────────────────────────

    fn parse_one(raw: &str) -> AtsJob {
        let board: BoardResponse = serde_json::from_str(raw).expect("parse");
        board.jobs.into_iter().map(normalise).next().expect("one job")
    }

    #[test]
    fn normalise_basic_london_job() {
        let raw = r#"{
            "jobs": [{
                "id": 12345,
                "title": "Backend Engineer",
                "absolute_url": "https://boards.greenhouse.io/acme/jobs/12345",
                "location": {"name": "London, UK"},
                "first_published": "2026-04-01T00:00:00Z",
                "updated_at": "2026-04-02T00:00:00Z",
                "content": null,
                "offices": null
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.external_id, "12345");
        assert_eq!(job.title, "Backend Engineer");
        assert_eq!(job.url, "https://boards.greenhouse.io/acme/jobs/12345");
        assert_eq!(job.location.as_deref(), Some("London, UK"));
        assert!(job.all_locations.contains(&"London, UK".to_string()));
        assert_eq!(job.posted_date.as_deref(), Some("2026-04-01T00:00:00Z"));
        assert_eq!(job.remote_policy, None);
    }

    #[test]
    fn normalise_semicolon_separated_locations() {
        let raw = r#"{
            "jobs": [{
                "id": 1,
                "title": "x",
                "absolute_url": "x",
                "location": {"name": "Berlin; London; Munich"},
                "content": null,
                "offices": null
            }]
        }"#;
        let job = parse_one(raw);
        // The full string stays in all_locations, plus each split part.
        assert!(job.all_locations.contains(&"Berlin".to_string()));
        assert!(job.all_locations.contains(&"London".to_string()));
        assert!(job.all_locations.contains(&"Munich".to_string()));
    }

    #[test]
    fn normalise_remote_policy_inferred_from_location() {
        let raw = r#"{
            "jobs": [{
                "id": 1,
                "title": "x",
                "absolute_url": "x",
                "location": {"name": "Remote - UK"},
                "content": null,
                "offices": null
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy.as_deref(), Some("Remote"));
    }

    #[test]
    fn normalise_hybrid_policy_inferred() {
        let raw = r#"{
            "jobs": [{
                "id": 1,
                "title": "x",
                "absolute_url": "x",
                "location": {"name": "London (Hybrid)"},
                "content": null,
                "offices": null
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.remote_policy.as_deref(), Some("Hybrid"));
    }

    #[test]
    fn normalise_office_locations_merged_into_all_locations() {
        let raw = r#"{
            "jobs": [{
                "id": 1,
                "title": "x",
                "absolute_url": "x",
                "location": {"name": "London"},
                "content": null,
                "offices": [
                    {"name": "HQ London", "location": "UK"},
                    {"name": "Remote Office", "location": null}
                ]
            }]
        }"#;
        let job = parse_one(raw);
        assert!(job.all_locations.contains(&"HQ London".to_string()));
        assert!(job.all_locations.contains(&"UK".to_string()));
        assert!(job.all_locations.contains(&"Remote Office".to_string()));
    }

    #[test]
    fn normalise_posted_date_falls_back_to_updated_at() {
        let raw = r#"{
            "jobs": [{
                "id": 1,
                "title": "x",
                "absolute_url": "x",
                "location": {"name": "London"},
                "first_published": null,
                "updated_at": "2026-04-02T00:00:00Z",
                "content": null,
                "offices": null
            }]
        }"#;
        let job = parse_one(raw);
        assert_eq!(job.posted_date.as_deref(), Some("2026-04-02T00:00:00Z"));
    }

    #[test]
    fn normalise_content_html_stripped() {
        let raw = r#"{
            "jobs": [{
                "id": 1,
                "title": "x",
                "absolute_url": "x",
                "location": {"name": "London"},
                "content": "<p>Build <strong>things</strong>.</p>",
                "offices": null
            }]
        }"#;
        let job = parse_one(raw);
        let desc = job.description.expect("description");
        assert!(!desc.contains('<'));
        assert!(desc.contains("Build"));
        assert!(desc.contains("things"));
    }

    #[test]
    fn normalise_empty_jobs_array() {
        let raw = r#"{"jobs": []}"#;
        let board: BoardResponse = serde_json::from_str(raw).expect("parse");
        assert!(board.jobs.is_empty());
    }

    // ─────────────────────────────────────────────────────────────
    // strip_html
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn greenhouse_strip_html_quoted_gt() {
        let input = r#"<span data-x='{"k":">"}'>ok</span>"#;
        assert_eq!(strip_html(input), "ok");
    }

    #[test]
    fn greenhouse_strip_html_nested() {
        assert_eq!(strip_html("<p><b>x</b></p>"), "x");
    }
}
