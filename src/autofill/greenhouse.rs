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
use super::greenhouse_api::{self, Field, FieldKind, JobSchema, Question};
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
    crate::tel!(
        "gh_fill_start",
        "job_url": job_url,
        "slug": slug,
        "job_id": job_id,
        "answer_keys": answers.len(),
        "resume_path": profile.resume_path.as_deref(),
    );

    // ── Step 1: fetch the schema ──
    let fetch_start = std::time::Instant::now();
    let schema = match JobSchema::fetch(slug, job_id).await {
        Ok(Some(s)) => {
            crate::tel!(
                "gh_schema_fetched",
                "duration_ms": fetch_start.elapsed().as_millis() as u64,
                "title": s.title.clone(),
                "main_questions": s.questions.len(),
                "compliance_sections": s.compliance.len(),
                "location_questions": s.location_questions.len(),
                "has_demographic": s.demographic_questions.is_some(),
                "absolute_url": s.absolute_url.as_deref(),
            );
            s
        }
        Ok(None) => {
            crate::tel!(
                "gh_schema_404",
                "slug": slug,
                "job_id": job_id,
                "duration_ms": fetch_start.elapsed().as_millis() as u64,
            );
            return AutofillResult::UnsupportedProvider(format!(
                "Greenhouse API returned 404 for slug={slug}, id={job_id} — \
                 company may be off-platform; reclassify as bespoke"
            ));
        }
        Err(e) => {
            crate::tel!(
                "gh_schema_error",
                "slug": slug,
                "job_id": job_id,
                "error": e,
                "duration_ms": fetch_start.elapsed().as_millis() as u64,
            );
            return AutofillResult::BrowserError(format!("API fetch: {e}"));
        }
    };

    // ── Step 2: launch browser at the canonical Greenhouse URL ──
    let navigate_url = schema.absolute_url.as_deref().unwrap_or(job_url);
    crate::tel!("gh_browser_launch_start", "url": navigate_url);
    let launch_start = std::time::Instant::now();
    let (browser, page) = match common::launch_and_navigate(navigate_url).await {
        Ok(result) => {
            crate::tel!(
                "gh_browser_launched",
                "duration_ms": launch_start.elapsed().as_millis() as u64,
            );
            result
        }
        Err(e) => {
            crate::tel!(
                "gh_browser_launch_error",
                "error": e.clone(),
                "duration_ms": launch_start.elapsed().as_millis() as u64,
            );
            return AutofillResult::BrowserError(e);
        }
    };

    // Wait for the application form to render.
    let form_found = common::wait_for_selector(&page, "#application-form", Duration::from_secs(8)).await;
    crate::tel!("gh_form_wait", "found": form_found);

    // Pre-index the package answers by normalised label for fast lookup.
    let answer_index = build_answer_index(answers);
    let company_name = schema.company_name.clone();

    let mut filled = 0u32;
    let mut skipped = 0u32;

    // Track which semantic keys have already been used on this form so we
    // don't write the same answer twice. Example: "Cover Letter" file input
    // and "Anything else you want to share?" textarea both resolve to the
    // `cover_letter` package key via the matcher; without this set, the
    // cover letter ends up in both, which reads as redundant AI output.
    let mut consumed_semantic_keys: std::collections::HashSet<&'static str> =
        std::collections::HashSet::new();

    // ── Step 3: main `questions[]` ──
    crate::tel!("gh_questions_phase_start", "count": schema.questions.len());
    let mut prev_answer: Option<String> = None;
    for question in &schema.questions {
        let q_label = question.label.clone();
        let field_name = question.fields.first().map(|f| f.name.clone()).unwrap_or_default();
        let field_type = question.fields.first().map(|f| f.field_type.clone()).unwrap_or_default();
        match fill_question(
            &page,
            question,
            profile,
            &answer_index,
            &company_name,
            &prev_answer,
            &mut consumed_semantic_keys,
        )
        .await
        {
            FillOutcome::Filled(answer_snapshot) => {
                crate::tel!(
                    "gh_question_filled",
                    "label": q_label,
                    "field": field_name,
                    "type": field_type,
                );
                filled += 1;
                prev_answer = answer_snapshot;
            }
            FillOutcome::Skipped => {
                crate::tel!(
                    "gh_question_skipped",
                    "label": q_label,
                    "field": field_name,
                    "type": field_type,
                );
                skipped += 1;
                prev_answer = None;
            }
            FillOutcome::SkippedConditional => {
                crate::tel!(
                    "gh_question_skipped_conditional",
                    "label": q_label,
                    "field": field_name,
                );
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

    crate::tel!(
        "gh_fill_done",
        "filled": filled,
        "skipped": skipped,
    );

    // Park the browser open — Chrome stays so the user can review + submit.
    tokio::spawn(async move {
        let _keep_alive = browser;
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

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
    company_name: &str,
    prev_answer: &Option<String>,
    consumed_keys: &mut std::collections::HashSet<&'static str>,
) -> FillOutcome {
    let Some(field) = question.fields.first() else {
        return FillOutcome::Skipped;
    };

    // Soft-conditional: if the label starts with "If yes" / "If no" / etc,
    // only fill when the immediately previous question's answer matches.
    if is_soft_conditional(&question.label) {
        // Same quote-stripping normalisation as is_soft_conditional, so that
        // `If "yes" can you specify ...` matches the "if yes" branch (Proton's
        // permit-type follow-up). Without this, the conditional fires but the
        // inner yes/no decision falls through to false and skips the question.
        let label_lower: String = question
            .label
            .to_lowercase()
            .chars()
            .filter(|&c| c != '"' && c != '\'' && c != '\u{201C}' && c != '\u{201D}')
            .collect();
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

    let normalised = normalise_label(&question.label);

    // ── InputFile fields short-circuit before the answer-required guard ──
    // File inputs don't need a string answer; they need a filesystem path
    // (resume) or a "switch to manual entry" click (cover letter as text).
    // The old code hit the `let Some(answer) = answer else { Skipped }`
    // guard and returned Skipped before reaching the InputFile branch.
    if matches!(field.kind(), FieldKind::InputFile) {
        return fill_file_field(page, field, &question.label, profile, answer_index).await;
    }

    // Resolve answer in priority order:
    //   1. Standard field name (e.g. `first_name` → profile.first_name)
    //   2. Exact match of normalised label against package keys
    //   3. Semantic-key fallback (prepare-applications writes domain-aware
    //      keys like `why_company`, `cover_letter`; we map those to common
    //      Greenhouse label patterns at fill time)
    //
    // Track which semantic key (if any) was consumed for this field, so the
    // caller can mark it used and prevent the same answer from being
    // written into a second field that happens to match the same key.
    let (answer, semantic_key_used) = {
        if let Some(v) = profile_field(profile, &field.name) {
            (Some(v.to_string()), None)
        } else if let Some(v) = answer_index.get(&normalised).cloned() {
            (Some(v), None)
        } else if let Some(key) = match_semantic_key(&normalised, company_name) {
            if consumed_keys.contains(key) {
                // Already used elsewhere on this form. Skip rather than
                // duplicate the same essay across multiple fields.
                (None, None)
            } else if let Some(v) = answer_index.get(key).cloned() {
                (Some(v), Some(key))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    };

    let Some(answer) = answer else {
        return FillOutcome::Skipped;
    };

    let outcome = match field.kind() {
        FieldKind::InputText => {
            // Greenhouse uses `input_text` for some questions whose answers
            // are essay-length (e.g. Proton's "Why is this role a good fit"
            // is input_text, not textarea). The browser silently caps these
            // at the field's maxlength (often 255). Trim to 250 chars at
            // the last sentence boundary so we don't ship a mid-word cut.
            let trimmed = trim_for_input_text(&answer);
            let selector = format!("#{}", css_escape_id(&field.name));
            if common::type_into(page, &selector, &trimmed).await {
                FillOutcome::Filled(Some(trimmed))
            } else {
                FillOutcome::Skipped
            }
        }
        FieldKind::Textarea => {
            let selector = format!("#{}", css_escape_id(&field.name));
            if common::type_into(page, &selector, &answer).await {
                FillOutcome::Filled(Some(answer))
            } else {
                FillOutcome::Skipped
            }
        }
        FieldKind::InputFile => unreachable!("handled in fill_file_field above"),
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
    };

    // Mark the semantic key as consumed only if the field was actually
    // filled — a skipped fill (selector not found, focus broken, etc.)
    // does not consume the key, so a later field can retry it.
    if matches!(outcome, FillOutcome::Filled(_)) {
        if let Some(key) = semantic_key_used {
            consumed_keys.insert(key);
        }
    }
    outcome
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

/// Handle a `<input type="file">` field.
///
/// Two modes:
///   1. Resume / CV: upload `profile.resume_path` via CDP `DOM.setFileInputFiles`.
///   2. Cover Letter: Greenhouse renders BOTH an `Attach` button and an
///      `Enter manually` button next to the file input. We don't have a
///      cover-letter PDF on disk — we have `cover_letter` essay text from
///      the package — so click "Enter manually" and type the text into the
///      revealed textarea.
async fn fill_file_field(
    page: &chromiumoxide::page::Page,
    field: &Field,
    label: &str,
    profile: &ApplicantProfile,
    answer_index: &HashMap<String, String>,
) -> FillOutcome {
    let label_lower = label.to_lowercase();
    let is_resume = label_lower.contains("resume")
        || label_lower.contains("cv")
        || field.name == "resume";
    let is_cover_letter = label_lower.contains("cover letter")
        || field.name == "cover_letter";

    if is_resume {
        if let Some(path) = profile.resume_path.as_deref() {
            let selector = format!("#{}", css_escape_id(&field.name));
            if common::set_file(page, &selector, path).await {
                return FillOutcome::Filled(None);
            }
        }
        return FillOutcome::Skipped;
    }

    if is_cover_letter {
        // We have text, not a PDF. Greenhouse exposes "Enter manually" near
        // the file input — clicking it swaps in a textarea (conventionally
        // named `<field>_text`). Try the manual-entry flow with per-step
        // telemetry so the next run pinpoints exactly where it fails.
        let Some(answer) = answer_index.get("cover_letter").cloned() else {
            crate::tel!("gh_cover_letter_no_answer");
            return FillOutcome::Skipped;
        };
        crate::tel!("gh_cover_letter_attempt", "answer_chars": answer.len());

        let clicked = common::click_button_with_text(page, "", "Enter manually").await;
        crate::tel!("gh_cover_letter_button_clicked", "ok": clicked);
        if !clicked {
            return FillOutcome::Skipped;
        }

        // Wait for the textarea to render after the button click.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Try multiple Greenhouse-conventional selectors in priority order.
        let escaped = css_escape_id(&field.name);
        let candidates = [
            format!("#{}_text", escaped),                // cover_letter_text
            format!("textarea#{}", escaped),             // textarea#cover_letter
            format!("textarea[name='{}_text']", escaped),
            format!("textarea[name='{}']", escaped),
        ];
        for sel in &candidates {
            if common::type_into(page, sel, &answer).await {
                crate::tel!("gh_cover_letter_filled", "selector": sel);
                return FillOutcome::Filled(Some(answer));
            }
        }
        crate::tel!(
            "gh_cover_letter_textarea_not_found",
            "tried": candidates.iter().collect::<Vec<_>>()
        );
        return FillOutcome::Skipped;
    }

    FillOutcome::Skipped
}

/// Trim a long answer to fit a typical `input_text` field. Cuts at the
/// last sentence boundary (or punctuation) ≤ 250 chars so we don't ship a
/// mid-word truncation, then appends an ellipsis if truncated.
fn trim_for_input_text(answer: &str) -> String {
    const TARGET: usize = 250;
    if answer.chars().count() <= TARGET {
        return answer.to_string();
    }
    // Find the last sentence-ending punctuation within the limit.
    let prefix: String = answer.chars().take(TARGET).collect();
    let cut_at = prefix
        .rfind(|c: char| matches!(c, '.' | '!' | '?' | ';'))
        .or_else(|| prefix.rfind(','))
        .or_else(|| prefix.rfind(' '))
        .unwrap_or(prefix.len());
    let head = &prefix[..=cut_at.min(prefix.len() - 1)];
    head.trim().to_string()
}

fn is_soft_conditional(label: &str) -> bool {
    // Strip both non-alphabetic leading chars and quote characters from
    // anywhere in the prefix region. Proton's label
    //   `If "yes" can you specify ...`
    // would otherwise miss because lowercase becomes `if "yes" ...` and
    // `starts_with("if yes")` is false (the quote is in the middle).
    // We normalise by removing ASCII double/single quotes before matching.
    let l: String = label
        .trim_start_matches(|c: char| !c.is_alphabetic())
        .to_lowercase()
        .chars()
        .filter(|&c| c != '"' && c != '\'' && c != '\u{201C}' && c != '\u{201D}')
        .collect();
    SOFT_CONDITIONAL_PREFIXES.iter().any(|p| l.starts_with(p))
}

/// Map a Greenhouse question label to a `prepare-applications` semantic key.
///
/// The skill writes JSON with fixed keys (`why_company`, `why_interested`,
/// `technical_project`, `cover_letter`). The autofill matches the schema's
/// per-question labels — they don't align directly, so we bridge here.
///
/// Returns the matching semantic key if the label fits a known pattern.
/// The semantic key vocabulary is closed and documented in the
/// prepare-applications skill body.
fn match_semantic_key(normalised_label: &str, company_name: &str) -> Option<&'static str> {
    let company_lower = company_name.to_lowercase();
    let has_company = !company_lower.is_empty()
        && normalised_label.contains(&company_lower);

    // ── why_company ──
    // "Why Anthropic?", "Why do you want to work at X?",
    // "What is it about Proton that excites you?"
    if has_company
        && (normalised_label.contains("why")
            || normalised_label.contains("excites")
            || normalised_label.contains("about"))
    {
        return Some("why_company");
    }
    if normalised_label.contains("why")
        && (normalised_label.contains("work at")
            || normalised_label.contains("work here")
            || normalised_label.contains("join us")
            || normalised_label.contains("join the team"))
    {
        return Some("why_company");
    }

    // ── why_interested ──
    // "Why this role?", "Why are you interested?",
    // "Why does this track interest you?"
    if normalised_label.contains("why")
        && (normalised_label.contains("interested")
            || normalised_label.contains("this role")
            || normalised_label.contains("this position")
            || normalised_label.contains("this opportunity")
            || normalised_label.contains("this track")
            || normalised_label.contains("this team")
            || normalised_label.contains("good fit"))
    {
        return Some("why_interested");
    }
    if normalised_label.contains("applied for this role")
        || normalised_label.contains("apply for this role")
    {
        return Some("why_interested");
    }

    // ── technical_project ──
    // "Tell us about a technical project you've worked on",
    // "Describe a project you've built",
    // "Outline a highly scalable system you developed"
    if normalised_label.contains("technical project")
        || normalised_label.contains("describe a project")
        || normalised_label.contains("scalable system")
        || normalised_label.contains("system you")
        || (normalised_label.contains("project") && normalised_label.contains("worked on"))
        || (normalised_label.contains("project") && normalised_label.contains("built"))
        || (normalised_label.contains("project") && normalised_label.contains("developed"))
    {
        return Some("technical_project");
    }

    // ── cover_letter ──
    // "Cover Letter" / "Additional Information" textareas where the
    // recruiter is asking for an essay. The standard field name
    // `cover_letter_text` is already handled by the file-or-text routing.
    if normalised_label == "additional information"
        || normalised_label == "anything else you want to share?"
        || normalised_label == "anything else you want to share"
    {
        return Some("cover_letter");
    }

    // ── years_of_<technology> experience ──
    // "How many years of professional experience do you have with Rust?"
    // We map to a single `years_of_rust` key for now; future tech-specific
    // expansions can add more keys (years_of_python, years_of_cpp, etc.).
    if (normalised_label.contains("years")
        || normalised_label.contains("how many years"))
        && (normalised_label.contains("experience") || normalised_label.contains("rust"))
    {
        return Some("years_of_rust");
    }

    // ── links / socials (LinkedIn / GitHub / Portfolio) ──
    // "Please share your LinkedIn profile / GitHub / Portfolio"
    if normalised_label.contains("linkedin")
        || normalised_label.contains("github")
        || normalised_label.contains("portfolio")
        || normalised_label.contains("personal website")
        || (normalised_label.contains("share") && normalised_label.contains("profile"))
    {
        return Some("links");
    }

    // ── salary_expectation (the number) ──
    // "What are your salary expectations? (number only)"
    if normalised_label.contains("salary")
        && (normalised_label.contains("expectation")
            || normalised_label.contains("expected")
            || normalised_label.contains("compensation"))
        && !normalised_label.contains("currency")
        && !normalised_label.contains("choice between")
    {
        return Some("salary_expectation");
    }

    // ── salary_unit (gross annual / monthly / net …) ──
    if normalised_label.contains("salary")
        && (normalised_label.contains("choice between")
            || normalised_label.contains("annual")
            || normalised_label.contains("monthly"))
    {
        return Some("salary_unit");
    }

    // ── salary_currency ──
    if normalised_label.contains("salary") && normalised_label.contains("currency") {
        return Some("salary_currency");
    }
    if normalised_label.contains("currency")
        && (normalised_label.contains("number above")
            || normalised_label.contains("for the number"))
    {
        return Some("salary_currency");
    }

    // ── start_date ──
    // "When can you start working with us?" / "Earliest start date"
    if (normalised_label.contains("when") && normalised_label.contains("start"))
        || normalised_label.contains("earliest start")
        || normalised_label.contains("start date")
    {
        return Some("start_date");
    }

    // ── preferred_office ──
    // "From what Proton's office location you'd like to work …"
    // "Preferred office location"
    if normalised_label.contains("office location")
        || normalised_label.contains("preferred office")
        || (normalised_label.contains("office") && normalised_label.contains("location"))
        || (normalised_label.contains("which location"))
        || (normalised_label.contains("office")
            && (normalised_label.contains("which")
                || normalised_label.contains("where")
                || normalised_label.contains("from what")
                || normalised_label.contains("like to work")))
    {
        return Some("preferred_office");
    }

    // ── visa_status (the "specify permit" textarea) ──
    // 'If "yes" can you specify the type of working permit you posses ...'
    // Checked BEFORE right_to_work because the labels overlap: this one
    // contains both "specify" and "permit", which is the more specific
    // pattern. The soft-conditional path also covers when it fires (only
    // when the prior right_to_work answer was "Yes").
    if normalised_label.contains("type of working permit")
        || normalised_label.contains("type of permit")
        || (normalised_label.contains("specify") && normalised_label.contains("permit"))
        || (normalised_label.contains("citizenship") && normalised_label.contains("visa"))
    {
        return Some("visa_status");
    }

    // ── right_to_work / eligibility ──
    // "Do you have an eligibility / working permit to work in this particular location?"
    // The answer is Yes/No (single-select), sourced from the package's
    // `right_to_work` key.
    if normalised_label.contains("right to work")
        || normalised_label.contains("eligible to work")
        || normalised_label.contains("working permit")
        || normalised_label.contains("eligibility")
        || (normalised_label.contains("permit") && normalised_label.contains("work"))
    {
        return Some("right_to_work");
    }

    None
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

    #[test]
    fn semantic_match_why_company_with_name() {
        assert_eq!(
            match_semantic_key(&normalise_label("Why Anthropic?"), "Anthropic"),
            Some("why_company")
        );
        assert_eq!(
            match_semantic_key(
                &normalise_label("What is it about Proton that excites you?"),
                "Proton"
            ),
            Some("why_company")
        );
    }

    #[test]
    fn semantic_match_why_company_without_name() {
        assert_eq!(
            match_semantic_key(
                &normalise_label("Why do you want to work at this company?"),
                "Anthropic"
            ),
            Some("why_company")
        );
    }

    #[test]
    fn semantic_match_why_interested() {
        assert_eq!(
            match_semantic_key(
                &normalise_label("Why do you think this role is a good fit for you?"),
                "Proton"
            ),
            Some("why_interested")
        );
        assert_eq!(
            match_semantic_key(
                &normalise_label("Why does this track interest you?"),
                "Squarepoint Capital"
            ),
            Some("why_interested")
        );
        assert_eq!(
            match_semantic_key(
                &normalise_label("Please briefly highlight why you have applied for this role specifically at XTX."),
                "XTX Markets"
            ),
            Some("why_interested")
        );
    }

    #[test]
    fn semantic_match_technical_project() {
        assert_eq!(
            match_semantic_key(
                &normalise_label("Tell us about a technical project you've worked on."),
                "Generic"
            ),
            Some("technical_project")
        );
        assert_eq!(
            match_semantic_key(
                &normalise_label("Please briefly outline a highly scalable system that you have played a significant role in developing"),
                "XTX Markets"
            ),
            Some("technical_project")
        );
    }

    #[test]
    fn semantic_match_cover_letter_overflow() {
        assert_eq!(
            match_semantic_key(&normalise_label("Additional Information"), "Anthropic"),
            Some("cover_letter")
        );
    }

    #[test]
    fn semantic_match_returns_none_on_unrelated() {
        assert_eq!(match_semantic_key(&normalise_label("First Name"), "Anthropic"), None);
        assert_eq!(match_semantic_key(&normalise_label("Phone"), "Anthropic"), None);
        assert_eq!(
            match_semantic_key(&normalise_label("Date of Birth"), "Anthropic"),
            None
        );
    }

    #[test]
    fn semantic_match_proton_field_set() {
        // Every custom Proton field maps to a known semantic key after the
        // matcher expansion. Drives the autofill from 4/18 filled to 13+/18.
        let cases = [
            ("How many years of professional experience do you have with Rust?", Some("years_of_rust")),
            ("Why do you think this role is a good fit for you?", Some("why_interested")),
            ("What it is about Proton that excites you?", Some("why_company")),
            ("Please share your LinkedIn profile / GitHub / Portfolio", Some("links")),
            ("What are your salary expectations? Please include your salary expectations (number only).", Some("salary_expectation")),
            ("Salary expectations - please select the right choice between:", Some("salary_unit")),
            ("Salary expectations - please select the currency for the number above:", Some("salary_currency")),
            ("When can you start working with us?", Some("start_date")),
            ("From what Proton's office location you'd like to work (please state country and a city)?", Some("preferred_office")),
            ("Do you have an eligibility / working permit to work in this particular location?", Some("right_to_work")),
            ("If \"yes\" can you specify the type of working permit you posses (citizenship, permanent residency, type of visa etc.)?", Some("visa_status")),
            ("Anything else you want to share?", Some("cover_letter")),
        ];
        for (label, expected) in cases {
            let got = match_semantic_key(&normalise_label(label), "Proton");
            assert_eq!(got, expected, "label: {label}");
        }
    }

    #[test]
    fn is_soft_conditional_handles_inline_quoted_yes() {
        // Proton's label is `If "yes" can you specify ...` — quote inside,
        // not at the start. The original implementation only stripped
        // leading non-alpha chars, so the quote inside broke the prefix
        // match.
        assert!(is_soft_conditional(
            "If \"yes\" can you specify the type of working permit"
        ));
        assert!(is_soft_conditional("\"If yes\" please describe"));
        assert!(is_soft_conditional("If yes, please describe"));
        assert!(!is_soft_conditional("Anything else you want to share?"));
    }

    #[test]
    fn trim_for_input_text_cuts_at_sentence_boundary() {
        // Under the limit: no change.
        let short = "Short answer.";
        assert_eq!(trim_for_input_text(short), short);

        // Over the limit: cut at the last sentence-ending punctuation.
        let long = "First sentence ends here. Second sentence continues with much more text that exceeds the 250 character limit because it includes a lot of padding to ensure we are well past the cap and then keeps going on and on to demonstrate the trimming behaviour in detail forever.";
        let trimmed = trim_for_input_text(long);
        assert!(trimmed.chars().count() <= 250, "trimmed: {trimmed:?}");
        assert!(
            trimmed.ends_with(['.', '!', '?', ';', ',']),
            "expected sentence-boundary end, got: {trimmed:?}"
        );
    }
}
