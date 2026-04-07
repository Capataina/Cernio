use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.lever.co/v0/postings";

/// A job posting from Lever's list endpoint.
/// Fields map to Lever's public API response shape.
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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeverCategories {
    pub commitment: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub team: Option<String>,
}

/// Full posting detail from Lever's single-posting endpoint.
/// Includes description text and structured requirement lists.
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

#[derive(Debug, Deserialize, Serialize)]
pub struct LeverList {
    pub text: String,
    pub content: String,
}

/// Fetch all open postings at a company.
pub async fn fetch_all(client: &reqwest::Client, slug: &str) -> Result<Vec<LeverPosting>, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}");
    let postings: Vec<LeverPosting> = client.get(&url).send().await?.json().await?;
    Ok(postings)
}

/// Fetch the full detail for a single posting.
pub async fn fetch_detail(client: &reqwest::Client, slug: &str, id: &str) -> Result<LeverPostingDetail, reqwest::Error> {
    let url = format!("{BASE_URL}/{slug}/{id}");
    let detail: LeverPostingDetail = client.get(&url).send().await?.json().await?;
    Ok(detail)
}

/// Strip HTML tags from a string. Simple implementation for cleaning
/// Lever's HTML content fields into readable plain text.
pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Add newline after block-level closing tags for readability.
                if result.ends_with("/li") || result.ends_with("/p") || result.ends_with("/div") {
                    result.push('\n');
                }
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
