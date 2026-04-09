use std::collections::HashMap;

use super::common;
use super::{ApplicantProfile, AutofillResult};

/// Greenhouse application form field selectors.
///
/// Greenhouse forms follow a consistent structure:
///   - Standard fields have predictable id/name attributes
///   - Custom questions use data-field attributes or are textarea elements
///     inside .custom-question containers
///   - The form is typically at boards.greenhouse.io/{slug}/jobs/{id}
///
/// The application page renders the form directly (no iframe on the
/// main boards.greenhouse.io domain). Fields are standard HTML inputs.
mod selectors {
    // ── Standard fields ──
    // Greenhouse uses id attributes like "first_name", "last_name", etc.
    // Some companies customise these, so we try multiple strategies.

    pub const FIRST_NAME: &[&str] = &[
        "input#first_name",
        "input[name='first_name']",
        "input[autocomplete='given-name']",
    ];

    pub const LAST_NAME: &[&str] = &[
        "input#last_name",
        "input[name='last_name']",
        "input[autocomplete='family-name']",
    ];

    pub const EMAIL: &[&str] = &[
        "input#email",
        "input[name='email']",
        "input[type='email']",
        "input[autocomplete='email']",
    ];

    pub const PHONE: &[&str] = &[
        "input#phone",
        "input[name='phone']",
        "input[type='tel']",
        "input[autocomplete='tel']",
    ];

    pub const LINKEDIN: &[&str] = &[
        // Greenhouse often renders LinkedIn as a custom question or
        // a field with a label containing "linkedin".
        "input[name*='linkedin' i]",
        "input[id*='linkedin' i]",
        "input[placeholder*='linkedin' i]",
        "input[aria-label*='linkedin' i]",
    ];

    pub const WEBSITE: &[&str] = &[
        "input[name*='website' i]",
        "input[name*='portfolio' i]",
        "input[id*='website' i]",
        "input[placeholder*='website' i]",
        "input[placeholder*='portfolio' i]",
    ];

    #[allow(dead_code)]
    pub const RESUME_UPLOAD: &str = "input[type='file'][name*='resume'], input[type='file'][name*='cv']";
}

/// Fill a Greenhouse application form with profile data.
///
/// Opens a headed Chrome window, navigates to the job URL, and fills
/// in standard fields. The browser stays open for the user to review,
/// upload their CV, and submit.
pub async fn fill(
    job_url: &str,
    profile: &ApplicantProfile,
    answers: &HashMap<String, String>,
) -> AutofillResult {
    let (_browser, page) = match common::launch_and_navigate(job_url).await {
        Ok(result) => result,
        Err(e) => return AutofillResult::BrowserError(e),
    };

    let mut filled = 0u32;

    // ── Fill standard fields ──

    if common::fill_by_strategies(&page, selectors::FIRST_NAME, &profile.first_name).await {
        filled += 1;
    }

    if common::fill_by_strategies(&page, selectors::LAST_NAME, &profile.last_name).await {
        filled += 1;
    }

    if common::fill_by_strategies(&page, selectors::EMAIL, &profile.email).await {
        filled += 1;
    }

    if common::fill_by_strategies(&page, selectors::PHONE, &profile.phone).await {
        filled += 1;
    }

    if common::fill_by_strategies(&page, selectors::LINKEDIN, &profile.linkedin_url).await {
        filled += 1;
    }

    if common::fill_by_strategies(&page, selectors::WEBSITE, &profile.website_url).await {
        filled += 1;
    }

    // ── Resume upload ──
    // TODO: Implement file upload via CDP when we have a resume PDF path.
    // For now the user drags in their CV manually.

    // ── Custom question answers from application package ──
    if !answers.is_empty() {
        filled += common::fill_custom_questions(&page, answers).await;
    }

    // Park the browser so Chrome stays open — user reviews and submits.
    tokio::spawn(async move {
        let _keep_alive = _browser;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    AutofillResult::Success {
        fields_filled: filled as usize,
    }
}
