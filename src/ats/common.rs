use std::time::Duration;

use serde::{Deserialize, Serialize};

/// A normalised job posting from any ATS provider.
///
/// Each provider-specific fetcher converts its API response into this
/// common format. The search pipeline works entirely with `AtsJob` values,
/// never with provider-specific types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtsJob {
    /// Provider-specific job ID (string to handle all providers).
    pub external_id: String,
    /// Job title.
    pub title: String,
    /// Direct URL to the job posting.
    pub url: String,
    /// Location as a single string, if available. Format varies by provider.
    /// `None` means the provider didn't supply location data.
    pub location: Option<String>,
    /// All location strings associated with this job (primary + secondary).
    /// Used for location filtering — a job passes if ANY location matches.
    pub all_locations: Vec<String>,
    /// Remote/hybrid/onsite policy, if available.
    pub remote_policy: Option<String>,
    /// When the job was posted, if available (ISO format).
    pub posted_date: Option<String>,
    /// Full job description as plain text, if fetched.
    pub description: Option<String>,
}

/// Result of probing an ATS provider for a company slug.
#[derive(Debug)]
pub struct SlugProbeResult {
    pub provider: &'static str,
    pub slug: String,
    pub job_count: usize,
}

/// Send a GET request with retry on timeout/connection errors.
/// Returns Ok(Response) on success, or the last error after all retries.
/// Non-retryable errors (4xx responses) are returned immediately.
pub async fn get_with_retry(
    client: &reqwest::Client,
    url: &str,
    max_retries: u32,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut last_err = None;

    for attempt in 0..=max_retries {
        match client.get(url).send().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                // Only retry on timeout or connection errors.
                if e.is_timeout() || e.is_connect() || e.is_request() {
                    last_err = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err(last_err.unwrap())
}

/// Send a POST request with retry on timeout/connection errors.
#[allow(dead_code)]
pub async fn post_json_with_retry(
    client: &reqwest::Client,
    url: &str,
    body: &serde_json::Value,
    max_retries: u32,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut last_err = None;

    for attempt in 0..=max_retries {
        match client.post(url).json(body).send().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if e.is_timeout() || e.is_connect() || e.is_request() {
                    last_err = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err(last_err.unwrap())
}
