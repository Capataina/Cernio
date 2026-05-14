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

        // Prefer the legal full name from the `## Full Name` section, since
        // Greenhouse's "First Name" / "Last Name" fields are legal-identity
        // fields used for background checks. Split the full name: last token
        // is the surname, everything else is the first name (handles
        // "Ata Caner Cetinkaya" → first="Ata Caner", last="Cetinkaya").
        // Fallback chain: Full Name section → Preferred Name → hardcoded.
        let (first_name, last_name) = match extract_section(&personal, "Full Name")
            .filter(|s| !s.is_empty())
            .map(|full| split_full_name(&full))
        {
            Some((f, l)) => (f, l),
            None => {
                let f = extract_section(&personal, "Preferred Name")
                    .or_else(|| extract_field(&personal, "First name"))
                    .unwrap_or_else(|| "Caner".to_string());
                let l = extract_section(&personal, "Last Name")
                    .or_else(|| extract_field(&personal, "Surname"))
                    .unwrap_or_else(|| "Cetinkaya".to_string());
                (f, l)
            }
        };
        let email = extract_section(&personal, "Email")
            .or_else(|| extract_field(&personal, "Email"))
            .unwrap_or_else(|| "atacanercetinkaya@gmail.com".to_string());
        let phone = extract_section(&personal, "Phone")
            .or_else(|| extract_field(&personal, "Phone"))
            .unwrap_or_else(|| "+44 7391 904514".to_string());
        let linkedin_url = extract_section(&personal, "LinkedIn")
            .or_else(|| extract_field(&personal, "LinkedIn"))
            .unwrap_or_else(|| "https://www.linkedin.com/in/atacanercetinkaya/".to_string());
        let website_url = extract_section(&personal, "Portfolio")
            .or_else(|| extract_section(&personal, "Website"))
            .or_else(|| extract_field(&personal, "Portfolio"))
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

/// Extract the body of an Obsidian-style `## Heading` section.
///
/// Returns the first non-empty line of prose under the heading. Stops at the
/// next heading of any level. Heading match is case-insensitive and ignores
/// leading `#` / whitespace.
///
/// Example:
/// ```text
/// ## Full Name
/// Ata Caner Cetinkaya
/// ```
/// → `Some("Ata Caner Cetinkaya")`.
fn extract_section(content: &str, heading: &str) -> Option<String> {
    let target = heading.trim().to_lowercase();
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        let stripped = line.trim_start_matches('#').trim();
        if !line.trim_start().starts_with('#') || stripped.is_empty() {
            continue;
        }
        if stripped.to_lowercase() == target {
            // Found the heading. Return the first non-empty line that follows
            // and is not itself a heading.
            for body_line in lines.by_ref() {
                let t = body_line.trim();
                if t.is_empty() {
                    continue;
                }
                if t.starts_with('#') {
                    return None;
                }
                return Some(t.to_string());
            }
            return None;
        }
    }
    None
}

/// Split "Ata Caner Cetinkaya" into ("Ata Caner", "Cetinkaya").
/// One-word names return (word, "").
fn split_full_name(full: &str) -> (String, String) {
    let trimmed = full.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    match parts.as_slice() {
        [] => (String::new(), String::new()),
        [only] => ((*only).to_string(), String::new()),
        [first @ .., last] => (first.join(" "), (*last).to_string()),
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
    crate::tel!(
        "autofill_invoked",
        "job_url": job_url,
        "provider": ats_provider,
        "slug": ats_slug,
        "has_package": package_json.is_some(),
        "package_chars": package_json.map(|s| s.len()).unwrap_or(0),
    );

    let answers: std::collections::HashMap<String, String> = package_json
        .and_then(|json| match serde_json::from_str(json) {
            Ok(v) => Some(v),
            Err(e) => {
                crate::tel!("autofill_package_parse_error", "error": e.to_string());
                None
            }
        })
        .unwrap_or_default();

    crate::tel!("autofill_package_parsed", "answer_keys": answers.len());

    match ats_provider {
        Some("greenhouse") => {
            // Resolve slug + job_id. Prefer parsing the URL since it carries
            // both; fall back to (slug-from-DB, gh_jid-from-URL) for the
            // custom-domain-wrapper case.
            let (slug, job_id) = match greenhouse_api::parse_url(job_url) {
                Some((s, id)) => {
                    crate::tel!("autofill_slug_from_url", "slug": s.clone(), "job_id": id);
                    (s, id)
                }
                None => {
                    let Some(slug) = ats_slug else {
                        crate::tel!("autofill_unsupported", "reason": "no_slug_url_or_db");
                        return AutofillResult::UnsupportedProvider(
                            "greenhouse: no slug in URL and none in DB — \
                             cannot resolve job"
                                .into(),
                        );
                    };
                    let Some(job_id) = greenhouse_api::extract_gh_jid(job_url) else {
                        crate::tel!("autofill_unsupported", "reason": "no_gh_jid_in_url");
                        return AutofillResult::UnsupportedProvider(
                            "greenhouse: no gh_jid in URL".into(),
                        );
                    };
                    crate::tel!("autofill_slug_from_db", "slug": slug, "job_id": job_id);
                    (slug.to_string(), job_id)
                }
            };

            let result = greenhouse::fill(job_url, &slug, job_id, profile, &answers).await;
            match &result {
                AutofillResult::Success { fields_filled } => {
                    crate::tel!("autofill_success", "fields_filled": fields_filled);
                }
                AutofillResult::UnsupportedProvider(msg) => {
                    crate::tel!("autofill_unsupported", "message": msg);
                }
                AutofillResult::BrowserError(msg) => {
                    crate::tel!("autofill_browser_error", "message": msg);
                }
            }
            result
        }
        Some(provider) => {
            crate::tel!("autofill_unsupported", "reason": "non_greenhouse_provider", "provider": provider);
            AutofillResult::UnsupportedProvider(provider.to_string())
        }
        None => {
            crate::tel!("autofill_unsupported", "reason": "no_provider");
            AutofillResult::UnsupportedProvider("unknown".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_section_finds_full_name() {
        let md = "# Personal Information\n\n## Full Name\nAta Caner Cetinkaya\n\n## Preferred Name\nCaner\n";
        assert_eq!(
            extract_section(md, "Full Name"),
            Some("Ata Caner Cetinkaya".to_string())
        );
        assert_eq!(
            extract_section(md, "Preferred Name"),
            Some("Caner".to_string())
        );
    }

    #[test]
    fn extract_section_is_case_insensitive() {
        let md = "## full name\nFoo Bar\n";
        assert_eq!(extract_section(md, "Full Name"), Some("Foo Bar".to_string()));
    }

    #[test]
    fn extract_section_returns_none_when_heading_missing() {
        let md = "## Other\nvalue\n";
        assert_eq!(extract_section(md, "Full Name"), None);
    }

    #[test]
    fn extract_section_returns_none_for_empty_section() {
        let md = "## Full Name\n\n## Email\nfoo@bar\n";
        assert_eq!(extract_section(md, "Full Name"), None);
    }

    #[test]
    fn split_full_name_handles_compound_first_name() {
        let (f, l) = split_full_name("Ata Caner Cetinkaya");
        assert_eq!(f, "Ata Caner");
        assert_eq!(l, "Cetinkaya");
    }

    #[test]
    fn split_full_name_handles_two_words() {
        let (f, l) = split_full_name("Jane Doe");
        assert_eq!(f, "Jane");
        assert_eq!(l, "Doe");
    }

    #[test]
    fn split_full_name_handles_single_word() {
        let (f, l) = split_full_name("Cher");
        assert_eq!(f, "Cher");
        assert_eq!(l, "");
    }

    #[test]
    fn applicant_profile_loads_legal_first_name_from_obsidian_personal_md() {
        // Smoke test: ensure the real profile/personal.md parses to
        // first_name="Ata Caner", last_name="Cetinkaya" (not "Caner" alone).
        // Skipped if profile/personal.md is absent (e.g. in CI without
        // the profile checked in).
        let profile_dir = std::path::Path::new("profile");
        if !profile_dir.join("personal.md").exists() {
            eprintln!("skip: profile/personal.md not present");
            return;
        }
        let p = ApplicantProfile::load(profile_dir);
        assert_eq!(p.first_name, "Ata Caner");
        assert_eq!(p.last_name, "Cetinkaya");
    }
}
