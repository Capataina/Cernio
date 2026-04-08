use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult};

const BASE_URL: &str = "https://apply.workable.com/api/v1/widget/accounts";

// ── API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WidgetResponse {
    jobs: Vec<WorkableJob>,
}

#[derive(Debug, Deserialize)]
struct WorkableJob {
    shortcode: String,
    title: String,
    shortlink: String,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    #[serde(default)]
    telecommuting: bool,
    locations: Option<Vec<WorkableLocation>>,
    /// Full description — only present with ?details=true.
    description: Option<String>,
    #[serde(rename = "published_on")]
    published_on: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WorkableLocation {
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
}

// ── Public interface ─────────────────────────────────────────────

/// Probe whether a Workable board exists for this slug.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    let url = format!("{BASE_URL}/{slug}");
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None; // 404 = slug doesn't exist
    }
    let widget: WidgetResponse = resp.json().await.ok()?;
    if widget.jobs.is_empty() {
        return None;
    }
    Some(SlugProbeResult {
        provider: "workable",
        slug: slug.to_string(),
        job_count: widget.jobs.len(),
    })
}

/// Fetch all jobs from a Workable board with descriptions.
pub async fn fetch_all(client: &reqwest::Client, slug: &str) -> Result<Vec<AtsJob>, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}?details=true");
    let resp = client.get(&url).send().await?.error_for_status()?;
    let widget: WidgetResponse = resp.json().await?;

    Ok(widget.jobs.into_iter().map(normalise).collect())
}

// ── Normalisation ────────────────────────────────────────────────

fn normalise(job: WorkableJob) -> AtsJob {
    let mut all_locations = Vec::new();

    // Add top-level city/state/country.
    if let Some(city) = &job.city {
        if !city.is_empty() {
            all_locations.push(city.clone());
        }
    }
    if let Some(state) = &job.state {
        if !state.is_empty() {
            all_locations.push(state.clone());
        }
    }
    if let Some(country) = &job.country {
        if !country.is_empty() {
            all_locations.push(country.clone());
        }
    }

    // Add structured locations array.
    if let Some(locations) = &job.locations {
        for loc in locations {
            if let Some(city) = &loc.city {
                if !city.is_empty() {
                    all_locations.push(city.clone());
                }
            }
            if let Some(country) = &loc.country {
                if !country.is_empty() {
                    all_locations.push(country.clone());
                }
            }
            // Country code (uppercase ISO: "GB", "US").
            if let Some(code) = &loc.country_code {
                if !code.is_empty() {
                    all_locations.push(code.clone());
                }
            }
        }
    }

    // Primary location: prefer "City, Country" if both exist.
    let primary_location = match (&job.city, &job.country) {
        (Some(c), Some(co)) if !c.is_empty() && !co.is_empty() => {
            Some(format!("{c}, {co}"))
        }
        (Some(c), _) if !c.is_empty() => Some(c.clone()),
        (_, Some(co)) if !co.is_empty() => Some(co.clone()),
        _ => None,
    };

    let remote_policy = if job.telecommuting {
        Some("Remote".to_string())
    } else {
        None
    };

    AtsJob {
        external_id: job.shortcode,
        title: job.title,
        url: job.shortlink,
        location: primary_location,
        all_locations,
        remote_policy,
        posted_date: job.published_on,
        description: job.description.map(|html| strip_html(&html)),
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
