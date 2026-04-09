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
