//! Greenhouse Job Board public API — schema fetch + typed types.
//!
//! Replaces the previous "scrape `window.__remixContext` from HTML" approach.
//! The API at `boards-api.greenhouse.io/v1/boards/{slug}/jobs/{id}?content=true&questions=true`
//! returns the same form schema as the Remix-rendered page, in cleaner JSON,
//! plus richer metadata (pay ranges, GDPR consent, location-type, etc.).
//!
//! Verified against 13 distinct Greenhouse-resolved companies during the
//! form-anatomy survey (see `context/references/greenhouse-form-anatomy.md`).

use serde::{Deserialize, Deserializer};

const API_HOST: &str = "https://boards-api.greenhouse.io";

/// Turn an explicit JSON `null` into `T::default()`.
///
/// `#[serde(default)]` alone handles only missing keys; the Greenhouse API
/// surfaces *present-and-null* fields for empty collections (e.g.
/// `"compliance": null` on jobs with no EEOC block).
/// Without this helper, those nulls fail deserialization into a `Vec<_>`
/// with `parse: error decoding response body`.
fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::<T>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Accept either a JSON string or a JSON number and return it as a `String`.
///
/// Proton's form (job 4585460101) returns `FieldOption.value` as a number
/// for several options (e.g. `{"label": "gross annual", "value": 37551344101}`)
/// while most other forms return strings. The autofill always stringifies
/// the value before sending it to the page anyway, so coercing to `String`
/// at deserialize time is lossless.
fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Visitor;
    use std::fmt;

    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a number")
        }
        fn visit_str<E>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_string<E>(self, v: String) -> Result<String, E> {
            Ok(v)
        }
        fn visit_i64<E>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_f64<E>(self, v: f64) -> Result<String, E> {
            // Avoid scientific notation for IDs that fit in i64/u64.
            if v.fract() == 0.0 && v.abs() < (i64::MAX as f64) {
                Ok((v as i64).to_string())
            } else {
                Ok(v.to_string())
            }
        }
        fn visit_bool<E>(self, v: bool) -> Result<String, E> {
            Ok(v.to_string())
        }
    }
    deserializer.deserialize_any(V)
}

/// Full job + form schema as returned by the Greenhouse Job Board API.
#[derive(Debug, Deserialize)]
pub struct JobSchema {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub language: Option<String>,

    /// Canonical Greenhouse-hosted URL for this job. The autofill navigates
    /// here rather than to whatever wrapper URL the DB carried — fixes the
    /// case where a custom-domain wrapper page (HRT, GSA, Squarepoint)
    /// doesn't itself render the form.
    #[serde(default)]
    pub absolute_url: Option<String>,

    /// Main application form questions.
    #[serde(default, deserialize_with = "null_to_default")]
    pub questions: Vec<Question>,

    /// Geolocation block (lat/long hidden + location text).
    #[serde(default, deserialize_with = "null_to_default")]
    pub location_questions: Vec<Question>,

    /// Company-defined diversity survey (different schema — see DemographicQuestion).
    #[serde(default)]
    pub demographic_questions: Option<DemographicBlock>,

    /// Legally-mandated compliance sections (EEOC, etc.). Each has a type tag.
    /// Present-and-null on jobs with no compliance block (e.g. Proton).
    #[serde(default, deserialize_with = "null_to_default")]
    pub compliance: Vec<ComplianceSection>,
}

/// A single form question. Universal shape — covers main form, location, and
/// nested EEOC questions (all share this schema).
#[derive(Debug, Deserialize)]
pub struct Question {
    #[serde(default)]
    pub required: bool,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default, deserialize_with = "null_to_default")]
    pub values: Vec<FieldOption>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FieldOption {
    pub label: String,
    /// String or number. Proton returns numeric IDs for some options
    /// (e.g. compensation-currency multi_value_single_select); other
    /// forms return strings. `string_or_number` coerces both to String.
    #[serde(deserialize_with = "string_or_number")]
    pub value: String,
}

/// Discriminated field-type enum derived from `Field::field_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Single-line `<input type="text">`.
    InputText,
    /// `<textarea>`.
    Textarea,
    /// `<input type="file">`.
    InputFile,
    /// Hidden input — used for lat/long in `location_questions`.
    /// Autofill should skip these (browser populates via geolocation API).
    InputHidden,
    /// Combobox / single-choice select. Pick one of `values[]`.
    MultiValueSingleSelect,
    /// Checkbox list / multi-choice select. Pick any subset of `values[]`.
    MultiValueMultiSelect,
    /// Anything else surfaced by the API (forward-compat).
    Unknown,
}

impl Field {
    pub fn kind(&self) -> FieldKind {
        match self.field_type.as_str() {
            "input_text" => FieldKind::InputText,
            "textarea" => FieldKind::Textarea,
            "input_file" => FieldKind::InputFile,
            "input_hidden" => FieldKind::InputHidden,
            "multi_value_single_select" => FieldKind::MultiValueSingleSelect,
            "multi_value_multi_select" => FieldKind::MultiValueMultiSelect,
            _ => FieldKind::Unknown,
        }
    }
}

/// Wrapper around the company-defined diversity survey block.
#[derive(Debug, Deserialize)]
pub struct DemographicBlock {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub questions: Vec<DemographicQuestion>,
}

/// Demographic-block question — different schema from main `Question`.
/// Uses `name`/`id`/`answer_options` vocabulary.
#[derive(Debug, Deserialize)]
pub struct DemographicQuestion {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub answer_type: Option<AnswerType>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub answer_options: Vec<AnswerOption>,
}

#[derive(Debug, Deserialize)]
pub struct AnswerType {
    /// e.g. "MULTI_SELECT", "SINGLE_SELECT" (uppercase, not the kebab-case from main schema).
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnswerOption {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub free_form: bool,
    /// Note: per survey finding, this flag is unreliable — Monzo's "Prefer
    /// not to say" had `decline_to_answer: false`. Label-match instead.
    #[serde(default)]
    pub decline_to_answer: bool,
}

/// One element of the `compliance` array — typically EEOC, possibly other
/// legally-mandated buckets. Each contains its own description + questions
/// in the universal `Question` shape.
#[derive(Debug, Deserialize)]
pub struct ComplianceSection {
    /// e.g. "eeoc". The Paperwork Reduction Act preamble has questions:[] empty.
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "null_to_default")]
    pub questions: Vec<Question>,
}

impl JobSchema {
    /// Fetch the schema from the public Greenhouse Job Board API.
    ///
    /// Returns `Ok(None)` on 404 — that signals the company is no longer on
    /// Greenhouse and should be reclassified. Other HTTP errors return `Err`.
    pub async fn fetch(slug: &str, job_id: u64) -> Result<Option<Self>, String> {
        let url = format!(
            "{API_HOST}/v1/boards/{slug}/jobs/{job_id}?content=true&questions=true"
        );

        let response = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("client build: {e}"))?
            .get(&url)
            .header("User-Agent", "cernio-autofill/1.0")
            .send()
            .await
            .map_err(|e| format!("request: {e}"))?;

        match response.status().as_u16() {
            404 => Ok(None),
            200 => {
                let schema: JobSchema = response
                    .json()
                    .await
                    .map_err(|e| format!("parse: {e}"))?;
                Ok(Some(schema))
            }
            code => Err(format!("HTTP {code}")),
        }
    }
}

/// Try to extract `(slug, job_id)` from a Greenhouse URL.
///
/// Handles every URL shape we've observed:
///   - `job-boards.greenhouse.io/{slug}/jobs/{id}`
///   - `job-boards.eu.greenhouse.io/{slug}/jobs/{id}`
///   - `boards.greenhouse.io/{slug}/jobs/{id}` (legacy — 301s but parseable)
///   - `boards.greenhouse.io/embed/job_app?for={slug}&token={id}`
///   - any URL with `?gh_jid={id}` — caller must supply slug from DB
///
/// Returns `None` if the URL doesn't match any known pattern.
pub fn parse_url(url: &str) -> Option<(String, u64)> {
    // Standalone Remix or legacy boards.
    // /{slug}/jobs/{id}
    if let Some((before, after)) = url.split_once("greenhouse.io/") {
        // Skip the embed variants for this branch.
        if !before.is_empty() && !after.starts_with("embed/") {
            if let Some((slug, rest)) = after.split_once("/jobs/") {
                let id_str = rest.split(|c: char| !c.is_ascii_digit()).next()?;
                if !slug.is_empty() && !slug.contains('/') {
                    return id_str.parse::<u64>().ok().map(|id| (slug.to_string(), id));
                }
            }
        }
    }

    // Embed URL: ?for={slug}&token={id}
    if url.contains("embed/job_app") {
        let slug = url_param(url, "for")?;
        let token = url_param(url, "token")?;
        return token.parse::<u64>().ok().map(|id| (slug, id));
    }

    None
}

/// Extract `?gh_jid=...` (or `&gh_jid=...`) from a URL.
pub fn extract_gh_jid(url: &str) -> Option<u64> {
    url_param(url, "gh_jid").and_then(|s| s.parse().ok())
}

fn url_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    let query = query.split('#').next().unwrap_or(query);
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                // Strip any URL-encoded amp suffix like & leftovers (defensive).
                let v = v.trim_end_matches(|c: char| c == '\\' || c == '"');
                return Some(v.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standalone_remix_url() {
        let (slug, id) =
            parse_url("https://job-boards.greenhouse.io/anthropic/jobs/5198074008").unwrap();
        assert_eq!(slug, "anthropic");
        assert_eq!(id, 5198074008);
    }

    #[test]
    fn parses_eu_subdomain() {
        let (slug, id) =
            parse_url("https://job-boards.eu.greenhouse.io/b2c2/jobs/4745320101").unwrap();
        assert_eq!(slug, "b2c2");
        assert_eq!(id, 4745320101);
    }

    #[test]
    fn parses_legacy_boards_url() {
        let (slug, id) =
            parse_url("https://boards.greenhouse.io/cloudflare/jobs/7582169").unwrap();
        assert_eq!(slug, "cloudflare");
        assert_eq!(id, 7582169);
    }

    #[test]
    fn parses_url_with_query_string() {
        let (slug, id) = parse_url(
            "https://boards.greenhouse.io/cloudflare/jobs/7582169?gh_jid=7582169",
        )
        .unwrap();
        assert_eq!(slug, "cloudflare");
        assert_eq!(id, 7582169);
    }

    #[test]
    fn parses_embed_url() {
        let (slug, id) = parse_url(
            "https://boards.greenhouse.io/embed/job_app?for=gsacapital&token=7555839002",
        )
        .unwrap();
        assert_eq!(slug, "gsacapital");
        assert_eq!(id, 7555839002);
    }

    #[test]
    fn extracts_gh_jid() {
        assert_eq!(
            extract_gh_jid("https://www.gsacapital.com/careers/gh/?gh_jid=7555839002"),
            Some(7555839002)
        );
    }

    #[test]
    fn returns_none_for_unrelated_url() {
        assert!(parse_url("https://stripe.com/jobs/search?gh_jid=7531158").is_none());
    }

    #[test]
    fn deserializes_null_compliance_and_demographic() {
        // Proton job 4585460101 returns `"compliance": null` and
        // `"demographic_questions": null`. Without the null_to_default
        // helper, this fails with "parse: error decoding response body".
        let payload = r#"{
            "id": 4585460101,
            "title": "Rust Software Engineer",
            "questions": [
                {
                    "label": "First Name",
                    "required": true,
                    "fields": [{"name": "first_name", "type": "input_text", "values": []}]
                }
            ],
            "location_questions": [],
            "demographic_questions": null,
            "compliance": null
        }"#;
        let schema: JobSchema = serde_json::from_str(payload).expect("must deserialize");
        assert_eq!(schema.questions.len(), 1);
        assert!(schema.compliance.is_empty());
        assert!(schema.demographic_questions.is_none());
    }

    #[test]
    fn deserializes_field_option_value_as_number() {
        // Proton (job 4585460101) returns numeric values for some
        // FieldOption.value fields. Coercing string-or-number to
        // String is required; otherwise the schema fetch fails with
        // `invalid type: integer X, expected a string`.
        let payload = r#"{
            "id": 1,
            "title": "t",
            "questions": [
                {
                    "label": "Compensation currency",
                    "fields": [{
                        "name": "currency",
                        "type": "multi_value_single_select",
                        "values": [
                            {"label": "gross annual", "value": 37551344101},
                            {"label": "CHF", "value": 37551344201},
                            {"label": "Already a string", "value": "USD"}
                        ]
                    }]
                }
            ]
        }"#;
        let schema: JobSchema = serde_json::from_str(payload).expect("must deserialize");
        let opts = &schema.questions[0].fields[0].values;
        assert_eq!(opts[0].value, "37551344101");
        assert_eq!(opts[1].value, "37551344201");
        assert_eq!(opts[2].value, "USD");
    }

    #[test]
    fn deserializes_null_question_fields() {
        // Defensive: if a question itself has `"fields": null` we still cope.
        let payload = r#"{
            "id": 1,
            "title": "t",
            "questions": [
                {"label": "L", "fields": null}
            ]
        }"#;
        let schema: JobSchema = serde_json::from_str(payload).expect("must deserialize");
        assert_eq!(schema.questions.len(), 1);
        assert!(schema.questions[0].fields.is_empty());
    }

    #[test]
    fn field_kind_dispatches_on_type() {
        let f = Field {
            name: "x".into(),
            field_type: "multi_value_single_select".into(),
            values: vec![],
        };
        assert_eq!(f.kind(), FieldKind::MultiValueSingleSelect);
    }
}
