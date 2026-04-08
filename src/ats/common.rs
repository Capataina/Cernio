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
