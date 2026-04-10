use serde::{Deserialize, Serialize};

use super::common::{AtsJob, SlugProbeResult, get_with_retry};

const BASE_URL: &str = "https://api.lever.co/v0/postings";
const BASE_URL_EU: &str = "https://api.eu.lever.co/v0/postings";

/// A job posting from Lever's list endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct LeverPosting {
    pub id: String,
    pub text: String,
    pub categories: LeverCategories,
    #[serde(rename = "hostedUrl")]
    pub hosted_url: Option<String>,
    #[serde(rename = "applyUrl")]
    pub apply_url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<u64>,
    #[serde(rename = "workplaceType")]
    pub workplace_type: Option<String>,
    #[serde(rename = "descriptionPlain")]
    pub description_plain: Option<String>,
    pub additional: Option<String>,
    #[serde(rename = "additionalPlain")]
    pub additional_plain: Option<String>,
    pub lists: Option<Vec<LeverList>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeverCategories {
    pub commitment: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub team: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeverList {
    pub text: String,
    pub content: String,
}

/// Full posting detail from Lever's single-posting endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct LeverPostingDetail {
    pub id: String,
    pub text: String,
    pub categories: LeverCategories,
    #[serde(rename = "descriptionPlain")]
    pub description_plain: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "additionalPlain")]
    pub additional_plain: Option<String>,
    pub lists: Option<Vec<LeverList>>,
    #[serde(rename = "hostedUrl")]
    pub hosted_url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<u64>,
    #[serde(rename = "workplaceType")]
    pub workplace_type: Option<String>,
}

/// Determine base URL based on ats_extra region.
fn base_url(ats_extra: Option<&str>) -> &'static str {
    if let Some(extra) = ats_extra {
        if extra.contains("eu") {
            return BASE_URL_EU;
        }
    }
    BASE_URL
}

/// Probe whether a Lever board exists for this slug.
/// Tries both US and EU endpoints with retry on timeout/connection errors.
pub async fn probe(client: &reqwest::Client, slug: &str) -> Option<SlugProbeResult> {
    for url_base in [BASE_URL, BASE_URL_EU] {
        let url = format!("{url_base}/{slug}");
        let resp = get_with_retry(client, &url, 2).await.ok()?;
        if !resp.status().is_success() {
            continue;
        }
        let postings: Vec<LeverPosting> = resp.json().await.ok()?;
        if !postings.is_empty() {
            return Some(SlugProbeResult {
                provider: "lever",
                slug: slug.to_string(),
                job_count: postings.len(),
            });
        }
    }
    None
}

/// Fetch all open postings at a company (US endpoint).
pub async fn fetch_all(
    client: &reqwest::Client,
    slug: &str,
) -> Result<Vec<LeverPosting>, reqwest::Error> {
    fetch_all_with_extra(client, slug, None).await
}

/// Fetch all open postings with optional EU support.
/// Uses retry to handle transient timeouts that would silently return zero jobs.
pub async fn fetch_all_with_extra(
    client: &reqwest::Client,
    slug: &str,
    ats_extra: Option<&str>,
) -> Result<Vec<LeverPosting>, reqwest::Error> {
    let url_base = base_url(ats_extra);
    let url = format!("{url_base}/{slug}");
    let resp = get_with_retry(client, &url, 2).await?;
    let postings: Vec<LeverPosting> = resp.json().await?;
    Ok(postings)
}

/// Convert Lever postings to normalised AtsJob format with descriptions.
pub fn normalise_postings(postings: Vec<LeverPosting>) -> Vec<AtsJob> {
    postings.into_iter().map(normalise_posting).collect()
}

fn normalise_posting(p: LeverPosting) -> AtsJob {
    let mut all_locations = Vec::new();
    if let Some(loc) = &p.categories.location {
        all_locations.push(loc.clone());
    }

    // Build description from available fields.
    let description = build_description(&p);

    AtsJob {
        external_id: p.id,
        title: p.text,
        url: p.hosted_url.unwrap_or_default(),
        location: p.categories.location,
        all_locations,
        remote_policy: p.workplace_type,
        posted_date: p.created_at.map(|ts| {
            chrono::DateTime::from_timestamp_millis(ts as i64)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        }),
        description,
    }
}

fn build_description(p: &LeverPosting) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(desc) = &p.description_plain {
        if !desc.is_empty() {
            parts.push(desc.clone());
        }
    }

    if let Some(lists) = &p.lists {
        for list in lists {
            parts.push(format!("{}:\n{}", list.text, strip_html(&list.content)));
        }
    }

    if let Some(additional) = &p.additional_plain {
        if !additional.is_empty() {
            parts.push(additional.clone());
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n"))
    }
}

/// Fetch the full detail for a single posting.
pub async fn fetch_detail(
    client: &reqwest::Client,
    slug: &str,
    id: &str,
) -> Result<LeverPostingDetail, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}/{id}");
    let detail: LeverPostingDetail = client.get(&url).send().await?.json().await?;
    Ok(detail)
}

/// Strip HTML tags from a string.
/// Handles '>' inside quoted attribute values (e.g. data-ccp-props with JSON).
#[allow(dead_code)]
pub fn strip_html(html: &str) -> String {
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
    fn base_url_eu_marker_selects_eu() {
        assert_eq!(base_url(Some("eu")), BASE_URL_EU);
        assert_eq!(base_url(Some("region=eu")), BASE_URL_EU);
        assert_eq!(base_url(Some(r#"{"region":"eu"}"#)), BASE_URL_EU);
    }

    #[test]
    fn base_url_non_eu_marker_uses_us() {
        assert_eq!(base_url(Some("us")), BASE_URL);
        assert_eq!(base_url(Some("anything")), BASE_URL);
    }

    // ─────────────────────────────────────────────────────────────
    // strip_html
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn strip_html_simple() {
        assert_eq!(strip_html("<p>hi</p>"), "hi");
    }

    #[test]
    fn strip_html_quoted_gt() {
        // The data-ccp-props case that motivated quote-aware parsing.
        let input = r#"<span data-ccp-props='{"k":true}'>visible</span>"#;
        assert_eq!(strip_html(input), "visible");
    }

    #[test]
    fn strip_html_empty() {
        assert_eq!(strip_html(""), "");
    }

    // ─────────────────────────────────────────────────────────────
    // build_description — parts are joined with double newlines
    // ─────────────────────────────────────────────────────────────

    fn sample_posting() -> LeverPosting {
        LeverPosting {
            id: "abc-123".to_string(),
            text: "Rust Engineer".to_string(),
            categories: LeverCategories {
                commitment: Some("Full-time".to_string()),
                department: Some("Engineering".to_string()),
                location: Some("London".to_string()),
                team: Some("Platform".to_string()),
            },
            hosted_url: Some("https://jobs.lever.co/acme/abc-123".to_string()),
            apply_url: None,
            created_at: Some(1_717_200_000_000),
            workplace_type: Some("hybrid".to_string()),
            description_plain: Some("We are hiring.".to_string()),
            additional: None,
            additional_plain: Some("More info here.".to_string()),
            lists: Some(vec![LeverList {
                text: "Requirements".to_string(),
                content: "<ul><li>Rust</li><li>Linux</li></ul>".to_string(),
            }]),
        }
    }

    #[test]
    fn build_description_joins_all_parts() {
        let p = sample_posting();
        let desc = build_description(&p).expect("description expected");
        assert!(desc.contains("We are hiring"));
        assert!(desc.contains("Requirements"));
        assert!(desc.contains("More info here"));
    }

    #[test]
    fn build_description_list_content_stripped() {
        let p = sample_posting();
        let desc = build_description(&p).unwrap();
        // HTML tags from list.content must be stripped.
        assert!(!desc.contains("<ul>"));
        assert!(!desc.contains("<li>"));
        assert!(desc.contains("Rust"));
        assert!(desc.contains("Linux"));
    }

    #[test]
    fn build_description_empty_sources_is_none() {
        let mut p = sample_posting();
        p.description_plain = None;
        p.additional_plain = None;
        p.lists = None;
        assert!(build_description(&p).is_none());
    }

    #[test]
    fn build_description_skips_empty_strings() {
        let mut p = sample_posting();
        p.description_plain = Some(String::new());
        p.additional_plain = Some(String::new());
        p.lists = None;
        assert!(build_description(&p).is_none());
    }

    // ─────────────────────────────────────────────────────────────
    // normalise_posting — end-to-end on a single posting
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn normalise_posting_happy_path() {
        let p = sample_posting();
        let job = normalise_posting(p);

        assert_eq!(job.external_id, "abc-123");
        assert_eq!(job.title, "Rust Engineer");
        assert_eq!(job.url, "https://jobs.lever.co/acme/abc-123");
        assert_eq!(job.location.as_deref(), Some("London"));
        assert_eq!(job.all_locations, vec!["London".to_string()]);
        assert_eq!(job.remote_policy.as_deref(), Some("hybrid"));
        assert!(job.posted_date.is_some());
        assert!(job.description.is_some());
    }

    #[test]
    fn normalise_posting_missing_hosted_url() {
        let mut p = sample_posting();
        p.hosted_url = None;
        let job = normalise_posting(p);
        assert_eq!(job.url, ""); // default is empty, not a panic
    }

    #[test]
    fn normalise_posting_timestamp_to_iso_date() {
        let mut p = sample_posting();
        p.created_at = Some(1_717_200_000_000); // 2024-06-01 in ms
        let job = normalise_posting(p);
        let posted = job.posted_date.expect("date expected");
        assert!(posted.starts_with("2024-"), "got: {posted}");
        assert_eq!(posted.len(), 10); // YYYY-MM-DD
    }

    #[test]
    fn normalise_posting_no_location() {
        let mut p = sample_posting();
        p.categories.location = None;
        let job = normalise_posting(p);
        assert_eq!(job.location, None);
        assert!(job.all_locations.is_empty());
    }

    #[test]
    fn normalise_postings_batch() {
        let list = vec![sample_posting(), sample_posting()];
        let jobs = normalise_postings(list);
        assert_eq!(jobs.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────
    // Realistic JSON fixture — serde round-trip
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn normalise_from_realistic_json() {
        // Minimal but faithful to Lever's actual response shape.
        let raw = r#"[
            {
                "id": "f00b4r",
                "text": "Junior Backend Engineer",
                "categories": {
                    "commitment": "Full-time",
                    "department": "Engineering",
                    "location": "London, UK",
                    "team": "Core"
                },
                "hostedUrl": "https://jobs.lever.co/acme/f00b4r",
                "createdAt": 1717200000000,
                "workplaceType": "hybrid",
                "descriptionPlain": "Build things in Rust."
            }
        ]"#;
        let postings: Vec<LeverPosting> = serde_json::from_str(raw).expect("parse");
        let jobs = normalise_postings(postings);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].title, "Junior Backend Engineer");
        assert_eq!(jobs[0].location.as_deref(), Some("London, UK"));
    }
}
