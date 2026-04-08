use serde::Deserialize;

use super::common::{AtsJob, SlugProbeResult, get_with_retry};

const BASE_URL: &str = "https://api.smartrecruiters.com/v1/companies";

// ── API response types ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListResponse {
    #[serde(rename = "totalFound")]
    total_found: u64,
    content: Vec<SmartRecruitersJob>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmartRecruitersJob {
    id: String,
    name: String,
    #[serde(rename = "releasedDate")]
    released_date: Option<String>,
    location: Option<SmartRecruitersLocation>,
    #[serde(rename = "experienceLevel")]
    experience_level: Option<SmartRecruitersLabel>,
    department: Option<SmartRecruitersLabel>,
    /// URL to the public posting.
    #[serde(rename = "ref")]
    ref_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmartRecruitersLocation {
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    remote: Option<bool>,
    #[serde(rename = "address")]
    address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmartRecruitersLabel {
    label: Option<String>,
}

/// Detail endpoint response — for fetching full description.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DetailResponse {
    #[serde(rename = "jobAd")]
    job_ad: Option<JobAd>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JobAd {
    sections: Option<JobAdSections>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JobAdSections {
    #[serde(rename = "jobDescription")]
    job_description: Option<HtmlSection>,
    qualifications: Option<HtmlSection>,
    #[serde(rename = "additionalInformation")]
    additional_information: Option<HtmlSection>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HtmlSection {
    text: Option<String>,
}

// ── Public interface ─────────────────────────────────────────────

/// Probe whether a SmartRecruiters board exists for this slug.
///
/// CRITICAL: SmartRecruiters returns HTTP 200 with `totalFound: 0` for
/// ANY slug, even completely fake ones. Only count as a hit if totalFound > 0.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    let url = format!("{BASE_URL}/{slug}/postings?limit=1");
    let resp = get_with_retry(client, &url, 2).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let list: ListResponse = resp.json().await.ok()?;
    if list.total_found == 0 {
        return None; // False positive — slug doesn't really exist.
    }
    Some(SlugProbeResult {
        provider: "smartrecruiters",
        slug: slug.to_string(),
        job_count: list.total_found as usize,
    })
}

/// Fetch all jobs from a SmartRecruiters board.
/// Handles pagination (max 100 per page).
pub async fn fetch_all(client: &reqwest::Client, slug: &str) -> Result<Vec<AtsJob>, reqwest::Error> {
    let mut all_jobs = Vec::new();
    let mut offset: u64 = 0;
    let limit: u64 = 100;

    loop {
        let url = format!("{BASE_URL}/{slug}/postings?limit={limit}&offset={offset}");
        let resp = client.get(&url).send().await?.error_for_status()?;
        let page: ListResponse = resp.json().await?;

        let page_count = page.content.len();
        for job in page.content {
            all_jobs.push(normalise(job, slug));
        }

        offset += limit;
        if page_count < limit as usize || offset >= page.total_found {
            break;
        }
    }

    Ok(all_jobs)
}

/// Fetch the full description for a single SmartRecruiters posting.
#[allow(dead_code)]
pub async fn fetch_detail(
    client: &reqwest::Client,
    slug: &str,
    job_id: &str,
) -> Result<Option<String>, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}/postings/{job_id}");
    let resp = client.get(&url).send().await?.error_for_status()?;
    let detail: DetailResponse = resp.json().await?;

    let mut parts = Vec::new();
    if let Some(ad) = detail.job_ad {
        if let Some(sections) = ad.sections {
            if let Some(desc) = sections.job_description {
                if let Some(text) = desc.text {
                    parts.push(strip_html(&text));
                }
            }
            if let Some(quals) = sections.qualifications {
                if let Some(text) = quals.text {
                    parts.push(strip_html(&text));
                }
            }
            if let Some(info) = sections.additional_information {
                if let Some(text) = info.text {
                    parts.push(strip_html(&text));
                }
            }
        }
    }

    if parts.is_empty() {
        Ok(None)
    } else {
        Ok(Some(parts.join("\n\n")))
    }
}

// ── Normalisation ────────────────────────────────────────────────

fn normalise(job: SmartRecruitersJob, slug: &str) -> AtsJob {
    let mut all_locations = Vec::new();
    let mut primary_location = None;
    let mut remote_policy = None;

    if let Some(loc) = &job.location {
        if let Some(city) = &loc.city {
            if !city.is_empty() {
                all_locations.push(city.clone());
            }
        }
        if let Some(region) = &loc.region {
            if !region.is_empty() {
                all_locations.push(region.clone());
            }
        }
        if let Some(country) = &loc.country {
            if !country.is_empty() {
                all_locations.push(country.clone());
            }
        }

        // Build primary location string.
        primary_location = match (&loc.city, &loc.country) {
            (Some(c), Some(co)) if !c.is_empty() && !co.is_empty() => {
                Some(format!("{c}, {co}"))
            }
            (Some(c), _) if !c.is_empty() => Some(c.clone()),
            _ => None,
        };

        if loc.remote == Some(true) {
            remote_policy = Some("Remote".to_string());
        }
    }

    // Construct URL — use ref_url if available, otherwise construct from slug+id.
    let url = job.ref_url.unwrap_or_else(|| {
        format!("https://jobs.smartrecruiters.com/{slug}/{}", job.id)
    });

    AtsJob {
        external_id: job.id,
        title: job.name,
        url,
        location: primary_location,
        all_locations,
        remote_policy,
        posted_date: job.released_date,
        description: None, // Requires separate detail fetch.
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
