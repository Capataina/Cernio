//! Greenhouse autofill — schema-driven.
//!
//! Workflow:
//!   1. Fetch the form schema from the public Greenhouse Job Board API.
//!   2. Launch headed Chrome and navigate to the application URL.
//!   3. For each question in the schema, look up an answer from the
//!      `ApplicantProfile` (standard fields) or the prepared application
//!      package (custom questions). Match custom questions by label.
//!   4. Drive the DOM through CDP — real keystrokes via `Element::type_str`
//!      for text, real clicks for combobox options, `DOM.setFileInputFiles`
//!      for uploads. This is what fires React's synthetic events natively.
//!   5. Decline compliance / demographic / location-coordinate questions
//!      by default (all optional per the survey).
//!   6. Leave Chrome open for the user to review and submit.
//!
//! See `context/references/greenhouse-form-anatomy.md` for the field-type
//! catalogue and per-company variations this code accommodates.

use std::collections::HashMap;
use std::time::Duration;

use super::common;
use super::greenhouse_api::{self, FieldKind, JobSchema, Question};
use super::{ApplicantProfile, AutofillResult};

/// Decline-to-answer option labels seen across surveyed forms. Used to
/// opt out of EEOC / demographic / similar legally-optional blocks.
const OPT_OUT_LABELS: &[&str] = &[
    "decline to self identify",
    "decline to self-identify",
    "i don't wish to answer",
    "i do not wish to answer",
    "i do not want to answer",
    "prefer not to say",
    "prefer not to answer",
    "not stated",
];

/// Soft-conditional label prefixes. When a textarea's label starts with
/// one of these, treat it as conditional on the preceding question's
/// answer and skip unless that trigger matches.
const SOFT_CONDITIONAL_PREFIXES: &[&str] = &[
    "if yes",
    "if no",
    "if applicable",
    "if any",
    "if you answered",
    "if so",
];

pub async fn fill(
    job_url: &str,
    slug: &str,
    job_id: u64,
    profile: &ApplicantProfile,
    answers: &HashMap<String, String>,
) -> AutofillResult {
    // ── Step 1: fetch the schema ──
    let schema = match JobSchema::fetch(slug, job_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return AutofillResult::UnsupportedProvider(format!(
                "Greenhouse API returned 404 for slug={slug}, id={job_id} — \
                 company may be off-platform; reclassify as bespoke"
            ));
        }
        Err(e) => {
            return AutofillResult::BrowserError(format!("API fetch: {e}"));
        }
    };

    // ── Step 2: launch browser ──
    let (browser, page) = match common::launch_and_navigate(job_url).await {
        Ok(result) => result,
        Err(e) => return AutofillResult::BrowserError(e),
    };

    // Wait for the application form to render.
    if !common::wait_for_selector(&page, "#application-form", Duration::from_secs(8)).await {
        // Not fatal — some embed pages render the form lazily after a click.
        // We continue and let individual selectors fail per-field if the form
        // never appeared.
    }

    // Pre-index the package answers by normalised label for fast lookup.
    let answer_index = build_answer_index(answers);

    let mut filled = 0u32;
    let mut skipped = 0u32;

    // ── Step 3: main `questions[]` ──
    let mut prev_answer: Option<String> = None;
    for question in &schema.questions {
        match fill_question(&page, question, profile, &answer_index, &prev_answer).await {
            FillOutcome::Filled(answer_snapshot) => {
                filled += 1;
                prev_answer = answer_snapshot;
            }
            FillOutcome::Skipped => {
                skipped += 1;
                prev_answer = None;
            }
            FillOutcome::SkippedConditional => {
                skipped += 1;
                // Keep prev_answer — chain of conditionals could continue.
            }
        }
    }

    // ── Step 4: location_questions[] (decline lat/long, fill location text) ──
    for question in &schema.location_questions {
        if let Some(field) = question.fields.first() {
            match field.kind() {
                FieldKind::InputHidden => { /* skip — browser geolocation */ }
                FieldKind::InputText => {
                    if let Some(location) = profile_field(profile, "candidate_location") {
                        let selector = format!("#{}", field.name);
                        if common::type_into(&page, &selector, location).await {
                            filled += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // ── Step 5: compliance sections (EEOC etc.) — decline by default ──
    for section in &schema.compliance {
        for question in &section.questions {
            if decline_question(&page, question).await {
                filled += 1;
            }
        }
    }

    // ── Step 6: demographic_questions — decline by default ──
    if let Some(block) = &schema.demographic_questions {
        for question in &block.questions {
            if decline_demographic(&page, question).await {
                filled += 1;
            }
        }
    }

    // Park the browser open — Chrome stays so the user can review + submit.
    tokio::spawn(async move {
        let _keep_alive = browser;
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    let _ = skipped; // tracked for future telemetry; not surfaced yet
    AutofillResult::Success {
        fields_filled: filled as usize,
    }
}

enum FillOutcome {
    /// The question was filled. Carries the answer text used (for soft-conditional chains).
    Filled(Option<String>),
    /// The question was skipped — no answer available or unsupported type.
    Skipped,
    /// The question was skipped because of a soft-conditional rule
    /// (label starts with "If yes…" etc and previous answer didn't match).
    SkippedConditional,
}

async fn fill_question(
    page: &chromiumoxide::page::Page,
    question: &Question,
    profile: &ApplicantProfile,
    answer_index: &HashMap<String, String>,
    prev_answer: &Option<String>,
) -> FillOutcome {
    let Some(field) = question.fields.first() else {
        return FillOutcome::Skipped;
    };

    // Soft-conditional: if the label starts with "If yes" / "If no" / etc,
    // only fill when the immediately previous question's answer matches.
    if is_soft_conditional(&question.label) {
        let label_lower = question.label.to_lowercase();
        let prev_lower = prev_answer
            .as_deref()
            .unwrap_or("")
            .to_lowercase();
        let conditional_match = match () {
            _ if label_lower.starts_with("if yes") => prev_lower == "yes",
            _ if label_lower.starts_with("if no") => prev_lower == "no",
            _ => false,
        };
        if !conditional_match {
            return FillOutcome::SkippedConditional;
        }
    }

    // Resolve answer: first try the standard-field map (by field.name),
    // then fall back to label match against the answer index.
    let answer = profile_field(profile, &field.name)
        .map(str::to_string)
        .or_else(|| answer_index.get(&normalise_label(&question.label)).cloned());

    let Some(answer) = answer else {
        return FillOutcome::Skipped;
    };

    match field.kind() {
        FieldKind::InputText | FieldKind::Textarea => {
            let selector = format!("#{}", css_escape_id(&field.name));
            if common::type_into(page, &selector, &answer).await {
                FillOutcome::Filled(Some(answer))
            } else {
                FillOutcome::Skipped
            }
        }
        FieldKind::InputFile => {
            // Files are filled only if the answer is a filesystem path.
            // For now, only resume_path on the profile is supported.
            if let Some(path) = profile.resume_path.as_deref() {
                let selector = format!("#{}", css_escape_id(&field.name));
                if common::set_file(page, &selector, path).await {
                    return FillOutcome::Filled(None);
                }
            }
            FillOutcome::Skipped
        }
        FieldKind::MultiValueSingleSelect => {
            // Resolve which option label to click.
            let target_label = pick_option_label(&field.values, &answer);
            let Some(option_label) = target_label else {
                return FillOutcome::Skipped;
            };
            let selector = format!("#{}", css_escape_id(&field.name));
            if common::click_combobox_option(page, &selector, &option_label).await {
                FillOutcome::Filled(Some(option_label))
            } else {
                FillOutcome::Skipped
            }
        }
        FieldKind::MultiValueMultiSelect => {
            // Answer may be a single string ("X, Y, Z") — split, then click
            // each matching option as a checkbox in the fieldset.
            let fieldset_selector =
                format!("fieldset#{}", css_escape_id(&field.name));
            let mut any_ticked = false;
            for chosen in answer.split(|c: char| c == ',' || c == ';') {
                let chosen = chosen.trim();
                if chosen.is_empty() {
                    continue;
                }
                let opt = pick_option_label(&field.values, chosen);
                if let Some(label) = opt {
                    if common::set_checkbox_in_fieldset(
                        page,
                        &fieldset_selector,
                        &label,
                        true,
                    )
                    .await
                    {
                        any_ticked = true;
                    }
                }
            }
            if any_ticked {
                FillOutcome::Filled(Some(answer))
            } else {
                FillOutcome::Skipped
            }
        }
        FieldKind::InputHidden | FieldKind::Unknown => FillOutcome::Skipped,
    }
}

/// Pick the closest matching option label in `values` for the given answer.
/// First tries exact case-insensitive match, then contains, then None.
fn pick_option_label(
    values: &[greenhouse_api::FieldOption],
    answer: &str,
) -> Option<String> {
    let answer_lower = answer.to_lowercase();
    let answer_trim = answer_lower.trim();
    // Exact match.
    for v in values {
        if v.label.to_lowercase().trim() == answer_trim {
            return Some(v.label.clone());
        }
    }
    // Contains match.
    for v in values {
        let lab = v.label.to_lowercase();
        if lab.contains(answer_trim) || answer_trim.contains(&lab) {
            return Some(v.label.clone());
        }
    }
    None
}

/// Try to decline a Schema-A question (used for EEOC / location_questions).
async fn decline_question(
    page: &chromiumoxide::page::Page,
    question: &Question,
) -> bool {
    let Some(field) = question.fields.first() else {
        return false;
    };
    if field.kind() != FieldKind::MultiValueSingleSelect
        && field.kind() != FieldKind::MultiValueMultiSelect
    {
        return false;
    }
    let Some(opt) = find_optout(&field.values) else {
        return false;
    };
    let selector = format!("#{}", css_escape_id(&field.name));
    match field.kind() {
        FieldKind::MultiValueSingleSelect => {
            common::click_combobox_option(page, &selector, &opt).await
        }
        FieldKind::MultiValueMultiSelect => {
            let fs = format!("fieldset#{}", css_escape_id(&field.name));
            common::set_checkbox_in_fieldset(page, &fs, &opt, true).await
        }
        _ => false,
    }
}

/// Try to decline a Schema-C (demographic) question.
async fn decline_demographic(
    page: &chromiumoxide::page::Page,
    question: &greenhouse_api::DemographicQuestion,
) -> bool {
    // Find an opt-out option by label-matching against the OPT_OUT_LABELS vocab.
    let target = question
        .answer_options
        .iter()
        .find(|opt| is_optout_label(&opt.name))
        .map(|o| o.name.clone());
    let Some(opt_label) = target else {
        return false;
    };
    // Demographic questions render with id=`question.id` and `name` as the label.
    // The DOM uses the same combobox pattern as main schema in most surveyed forms.
    let selector = format!("#demographic_question_{}", question.id);
    if common::click_combobox_option(page, &selector, &opt_label).await {
        return true;
    }
    // Fallback: try fieldset-style multi-select.
    let fs = format!("fieldset#demographic_question_{}", question.id);
    common::set_checkbox_in_fieldset(page, &fs, &opt_label, true).await
}

fn find_optout(values: &[greenhouse_api::FieldOption]) -> Option<String> {
    values
        .iter()
        .find(|v| is_optout_label(&v.label))
        .map(|v| v.label.clone())
}

fn is_optout_label(label: &str) -> bool {
    let l = label.to_lowercase();
    OPT_OUT_LABELS.iter().any(|opt| l.contains(opt))
}

fn is_soft_conditional(label: &str) -> bool {
    let l = label.trim_start_matches(|c: char| !c.is_alphabetic())
        .to_lowercase();
    SOFT_CONDITIONAL_PREFIXES.iter().any(|p| l.starts_with(p))
}

/// Map a Greenhouse-standard field name to the corresponding profile field.
/// Returns the value, or None if the field name is custom (matched by label
/// against the answer index instead).
fn profile_field<'a>(profile: &'a ApplicantProfile, name: &str) -> Option<&'a str> {
    match name {
        "first_name" | "preferred_name" => Some(profile.first_name.as_str()),
        "last_name" => Some(profile.last_name.as_str()),
        "email" => Some(profile.email.as_str()),
        "phone" => Some(profile.phone.as_str()),
        // candidate_location maps from profile via the linkedin_url's slot is
        // wrong — there's no dedicated location field yet on ApplicantProfile.
        // location_questions handles this separately.
        "candidate_location" | "location" => None,
        _ => None,
    }
}

/// Normalise a label for fuzzy lookup: lowercase, strip non-alphanumeric
/// leading characters (handles leading emoji), collapse whitespace.
fn normalise_label(label: &str) -> String {
    let trimmed = label.trim_start_matches(|c: char| !c.is_alphanumeric());
    let lower = trimmed.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_space = false;
    for ch in lower.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn build_answer_index(answers: &HashMap<String, String>) -> HashMap<String, String> {
    answers
        .iter()
        .map(|(k, v)| (normalise_label(k), v.clone()))
        .collect()
}

/// Escape an id for use in a CSS selector — Greenhouse uses `[]` in
/// multi-select field names (e.g. `question_NNN[]`), which is invalid
/// unescaped inside `#...`.
fn css_escape_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len() + 2);
    for ch in id.chars() {
        if "!\"#$%&'()*+,./:;<=>?@[\\]^`{|}~".contains(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_strips_emoji_and_punctuation() {
        assert_eq!(normalise_label("🛂 Please confirm UK Right to Work"), "please confirm uk right to work");
        assert_eq!(normalise_label("Why Anthropic?"), "why anthropic?");
        assert_eq!(normalise_label("  First   Name  "), "first name");
    }

    #[test]
    fn optout_detection() {
        assert!(is_optout_label("Prefer not to say"));
        assert!(is_optout_label("Decline To Self Identify"));
        assert!(is_optout_label("I don't wish to answer"));
        assert!(!is_optout_label("Yes"));
        assert!(!is_optout_label("Male"));
    }

    #[test]
    fn soft_conditional_detection() {
        assert!(is_soft_conditional("If yes, please describe..."));
        assert!(is_soft_conditional("If no, please clarify..."));
        assert!(is_soft_conditional("\"If yes\" can you specify"));
        assert!(!is_soft_conditional("Why Anthropic?"));
        assert!(!is_soft_conditional("First Name"));
    }

    #[test]
    fn css_escape_handles_brackets() {
        assert_eq!(css_escape_id("question_123[]"), "question_123\\[\\]");
        assert_eq!(css_escape_id("first_name"), "first_name");
    }
}
