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
    /// Public careers-page URL — `https://jobs.smartrecruiters.com/{slug}/{id}-slug`.
    /// The sibling `ref` field exists in the response but points at the API
    /// endpoint (`https://api.smartrecruiters.com/v1/...`), which renders as
    /// raw JSON in a browser — never use it.
    #[serde(rename = "postingUrl")]
    posting_url: Option<String>,
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
/// Handles pagination (max 100 per page). After collecting the list, fetches
/// each job's full description via the detail endpoint and populates the
/// `description` field. Detail fetches are best-effort: a failure on one job
/// leaves its description as None but does not fail the whole batch — same
/// contract as workable / lever / ashby which embed descriptions inline.
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

    // Populate descriptions via detail-endpoint fetches. SmartRecruiters'
    // posting-list endpoint omits the description field entirely; the only
    // way to get it is a per-job GET to /postings/{id}. Without this loop the
    // raw_description column ends up NULL for every smartrecruiters job, and
    // grade-jobs falls back to brand-only reasoning.
    for job in all_jobs.iter_mut() {
        if let Ok(Some(desc)) = fetch_detail(client, slug, &job.external_id).await {
            job.description = Some(desc);
        }
    }

    Ok(all_jobs)
}

/// Fetch the full description for a single SmartRecruiters posting.
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

    // Prefer the API's postingUrl (the real public careers-page URL).
    // Fall back to constructing from slug+id if the field is missing.
    // Defence-in-depth: reject anything pointing at the API host, since the
    // sibling `ref` field used to be deserialised here and stored raw JSON
    // endpoints in the DB (32 rows had to be backfilled — see commit history).
    let url = job
        .posting_url
        .filter(|u| !u.starts_with("https://api.smartrecruiters.com/"))
        .unwrap_or_else(|| format!("https://jobs.smartrecruiters.com/{slug}/{}", job.id));

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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_one(raw: &str, slug: &str) -> AtsJob {
        let list: ListResponse = serde_json::from_str(raw).expect("parse");
        list.content
            .into_iter()
            .map(|j| normalise(j, slug))
            .next()
            .expect("one job")
    }

    // ─────────────────────────────────────────────────────────────
    // The critical SmartRecruiters quirk: HTTP 200 with totalFound=0
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn total_found_zero_is_parseable() {
        // The whole point of the probe function: even garbage slugs return
        // HTTP 200 + totalFound=0. The parser must accept this shape cleanly
        // so probe() can reject based on totalFound, not on response status.
        let raw = r#"{"totalFound": 0, "content": []}"#;
        let list: ListResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(list.total_found, 0);
        assert!(list.content.is_empty());
    }

    #[test]
    fn total_found_positive_with_content() {
        let raw = r#"{
            "totalFound": 2,
            "content": [
                {"id": "a", "name": "Job A"},
                {"id": "b", "name": "Job B"}
            ]
        }"#;
        let list: ListResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(list.total_found, 2);
        assert_eq!(list.content.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────
    // normalise
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn normalise_basic_with_posting_url() {
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "sr-1",
                "name": "Backend Engineer",
                "releasedDate": "2026-04-01",
                "location": {
                    "city": "London",
                    "country": "United Kingdom"
                },
                "postingUrl": "https://jobs.smartrecruiters.com/acme/sr-1-backend-engineer",
                "ref": "https://api.smartrecruiters.com/v1/companies/acme/postings/sr-1"
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.external_id, "sr-1");
        assert_eq!(job.title, "Backend Engineer");
        // Must take postingUrl, never `ref` (which points at the API host).
        assert_eq!(job.url, "https://jobs.smartrecruiters.com/acme/sr-1-backend-engineer");
        assert_eq!(job.location.as_deref(), Some("London, United Kingdom"));
        assert!(job.all_locations.contains(&"London".to_string()));
        assert!(job.all_locations.contains(&"United Kingdom".to_string()));
        assert_eq!(job.posted_date.as_deref(), Some("2026-04-01"));
    }

    #[test]
    fn normalise_url_fallback_constructs_from_slug_and_id() {
        // When `postingUrl` is absent, the parser builds a URL from slug + id.
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "sr-2",
                "name": "ML Engineer"
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.url, "https://jobs.smartrecruiters.com/acme/sr-2");
    }

    #[test]
    fn normalise_rejects_api_host_url() {
        // Regression guard: the SmartRecruiters API returns a `postingUrl`
        // alongside a `ref` field that points at the API endpoint
        // (`api.smartrecruiters.com/...`). The fetcher used to deserialise
        // `ref` into the public-URL slot, which stored raw JSON endpoints
        // in the DB and produced broken applies (Wise, L&G AM, etc.).
        // Even if `postingUrl` is somehow set to the API host, we must
        // reject it and fall back to the constructed form.
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "sr-3",
                "name": "Platform Engineer",
                "postingUrl": "https://api.smartrecruiters.com/v1/companies/acme/postings/sr-3"
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert!(
            !job.url.starts_with("https://api.smartrecruiters.com/"),
            "url must not point at the API host; got {}",
            job.url
        );
        assert_eq!(job.url, "https://jobs.smartrecruiters.com/acme/sr-3");
    }

    #[test]
    fn normalise_remote_flag() {
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "x",
                "name": "x",
                "location": {"city": "Anywhere", "remote": true}
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.remote_policy.as_deref(), Some("Remote"));
    }

    #[test]
    fn normalise_remote_false_yields_no_policy() {
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "x",
                "name": "x",
                "location": {"city": "London", "remote": false}
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.remote_policy, None);
    }

    #[test]
    fn normalise_location_with_region() {
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "x",
                "name": "x",
                "location": {"city": "Cambridge", "region": "Cambridgeshire", "country": "UK"}
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert!(job.all_locations.contains(&"Cambridge".to_string()));
        assert!(job.all_locations.contains(&"Cambridgeshire".to_string()));
        assert!(job.all_locations.contains(&"UK".to_string()));
    }

    #[test]
    fn normalise_missing_country_only_city_primary() {
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "x",
                "name": "x",
                "location": {"city": "London"}
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.location.as_deref(), Some("London"));
    }

    #[test]
    fn normalise_empty_location_object() {
        let raw = r#"{
            "totalFound": 1,
            "content": [{
                "id": "x",
                "name": "x",
                "location": {}
            }]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.location, None);
        assert!(job.all_locations.is_empty());
    }

    #[test]
    fn normalise_description_always_none_from_list_endpoint() {
        // SmartRecruiters list endpoint doesn't carry descriptions. They
        // have to be fetched separately via fetch_detail. This is a real
        // constraint we've hit; keep it asserted.
        let raw = r#"{
            "totalFound": 1,
            "content": [{"id": "x", "name": "x"}]
        }"#;
        let job = parse_one(raw, "acme");
        assert_eq!(job.description, None);
    }

    #[test]
    fn smartrecruiters_strip_html_simple() {
        assert_eq!(strip_html("<p>hi</p>"), "hi");
    }
}
