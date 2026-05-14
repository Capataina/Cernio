pub mod common;
pub mod greenhouse;
pub mod greenhouse_api;

use std::path::Path;

/// Profile data extracted from profile/ files for form filling.
#[allow(dead_code)]
pub struct ApplicantProfile {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub linkedin_url: String,
    pub website_url: String,
    pub resume_path: Option<String>,
}

impl ApplicantProfile {
    /// Load profile data from the profile/ directory.
    pub fn load(profile_dir: &Path) -> Self {
        let personal = std::fs::read_to_string(profile_dir.join("personal.md"))
            .unwrap_or_default();

        let first_name = extract_field(&personal, "Preferred name")
            .or_else(|| extract_field(&personal, "First name"))
            .unwrap_or_else(|| "Caner".to_string());
        let last_name = extract_field(&personal, "Last name")
            .or_else(|| extract_field(&personal, "Surname"))
            .unwrap_or_else(|| "Cetinkaya".to_string());
        let email = extract_field(&personal, "Email")
            .unwrap_or_else(|| "atacanercetinkaya@gmail.com".to_string());
        let phone = extract_field(&personal, "Phone")
            .unwrap_or_else(|| "+44 7391 904514".to_string());
        let linkedin_url = extract_field(&personal, "LinkedIn")
            .unwrap_or_else(|| "https://www.linkedin.com/in/atacanercetinkaya/".to_string());
        let website_url = extract_field(&personal, "Portfolio")
            .or_else(|| extract_field(&personal, "Website"))
            .unwrap_or_else(|| "https://capataina.vercel.app/".to_string());

        // Look for a resume PDF in profile/.
        let resume_path = ["profile/resume.pdf", "profile/CV.pdf", "profile/cv.pdf"]
            .iter()
            .find(|p| Path::new(p).exists())
            .map(|p| p.to_string());

        Self {
            first_name,
            last_name,
            email,
            phone,
            linkedin_url,
            website_url,
            resume_path,
        }
    }
}

/// Extract a field value from a markdown file.
/// Looks for patterns like "**Field:** value" or "| Field | value |".
fn extract_field(content: &str, field_name: &str) -> Option<String> {
    let lower_field = field_name.to_lowercase();

    for line in content.lines() {
        let lower_line = line.to_lowercase();

        // Pattern: **Field:** value or **Field**: value
        if lower_line.contains(&lower_field) {
            // Try "**Field:** value" pattern.
            if let Some(pos) = line.find(":**") {
                let after = &line[pos + 3..];
                let value = after.trim().trim_start_matches("**").trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
            // Try "| Field | value |" table pattern.
            if line.contains('|') {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let value = parts[2].trim();
                    if !value.is_empty() && value != "---" {
                        return Some(value.to_string());
                    }
                }
            }
            // Try "Field: value" plain pattern.
            if let Some(pos) = lower_line.find(&lower_field) {
                let after_field = &line[pos + field_name.len()..];
                let after = after_field
                    .trim_start_matches(|c: char| c == ':' || c == '*' || c.is_whitespace());
                if !after.is_empty() {
                    return Some(after.trim().to_string());
                }
            }
        }
    }
    None
}

/// Autofill result indicating what happened.
#[allow(dead_code)]
pub enum AutofillResult {
    /// Browser launched and form filled successfully.
    Success { fields_filled: usize },
    /// ATS provider not supported for autofill, or company is off-platform.
    UnsupportedProvider(String),
    /// Browser launch or API fetch failed.
    BrowserError(String),
}

/// Launch autofill for a job. Dispatches to the correct ATS provider.
///
/// `ats_slug` is the ATS-side identifier for the company (from
/// `company_portals.ats_slug` in the DB) — required for Greenhouse
/// because the public API is keyed on it, not on the URL.
///
/// `package_json` is the pre-generated answers JSON from
/// `application_packages`, if one exists. Format: JSON object mapping
/// question labels to answer text.
pub async fn fill_application(
    job_url: &str,
    ats_provider: Option<&str>,
    ats_slug: Option<&str>,
    profile: &ApplicantProfile,
    package_json: Option<&str>,
) -> AutofillResult {
    let answers: std::collections::HashMap<String, String> = package_json
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();

    match ats_provider {
        Some("greenhouse") => {
            // Resolve slug + job_id. Prefer parsing the URL since it carries
            // both; fall back to (slug-from-DB, gh_jid-from-URL) for the
            // custom-domain-wrapper case.
            let (slug, job_id) = match greenhouse_api::parse_url(job_url) {
                Some((s, id)) => (s, id),
                None => {
                    let Some(slug) = ats_slug else {
                        return AutofillResult::UnsupportedProvider(
                            "greenhouse: no slug in URL and none in DB — \
                             cannot resolve job"
                                .into(),
                        );
                    };
                    let Some(job_id) = greenhouse_api::extract_gh_jid(job_url) else {
                        return AutofillResult::UnsupportedProvider(
                            "greenhouse: no gh_jid in URL".into(),
                        );
                    };
                    (slug.to_string(), job_id)
                }
            };

            greenhouse::fill(job_url, &slug, job_id, profile, &answers).await
        }
        Some(provider) => AutofillResult::UnsupportedProvider(provider.to_string()),
        None => AutofillResult::UnsupportedProvider("unknown".to_string()),
    }
}
