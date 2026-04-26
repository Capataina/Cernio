//! Integrity tests for `profile/preferences.toml`.
//!
//! Why this file matters: `profile/preferences.toml` is read at every CLI
//! invocation by `src/config.rs`, but the loader is intentionally lenient —
//! a malformed TOML or a missing field silently falls back to defaults with
//! a stderr warning. A typo would not surface as a build failure; it would
//! surface as the search pipeline running with default filters (effectively
//! ignoring all of the user's tuning, returning thousands of off-target
//! jobs). The same file is also read by every Cernio Claude Code skill as
//! profile data — `[hard]` and `[soft]` sections drive grading judgments.
//!
//! These tests assert that the actual file in the repo:
//! - Is readable from the standard path
//! - Parses as valid TOML
//! - Survives strict parsing through the `Preferences` struct (no fallback)
//! - Contains every section + key the runtime + skills depend on
//! - Has the right shape (arrays vs strings vs numbers vs booleans) per value
//! - Has location-pattern coverage for every ATS provider in `src/ats/`
//! - Holds sensible values (positive numbers, valid grade letters, non-empty
//!   keyword lists, seniority terms in the exclusion list)
//!
//! When this file fails: stop. Do not commit changes to preferences.toml that
//! make these tests fail without first updating the tests in lockstep — the
//! tests document the contract that `src/config.rs` and the skills depend on.

use cernio::config::Preferences;
use std::fs;
use std::path::PathBuf;

const PREFERENCES_FILE: &str = "profile/preferences.toml";

/// All ATS providers Cernio's pipeline supports. Must stay in sync with the
/// modules in `src/ats/`. A new provider added without a matching location
/// subtable in `preferences.toml` would silently bypass the location filter
/// for that provider's jobs.
const SUPPORTED_ATS_PROVIDERS: &[&str] = &[
    "greenhouse",
    "lever",
    "ashby",
    "workable",
    "smartrecruiters",
    "workday",
];

/// Valid `min_company_grade` values per `src/config.rs::included_grades`.
const VALID_COMPANY_GRADES: &[&str] = &["S", "A", "B", "C"];

/// Valid `remove_job_grades` values per the job grading rubric.
const VALID_JOB_GRADES: &[&str] = &["SS", "S", "A", "B", "C", "F"];

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn preferences_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PREFERENCES_FILE)
}

fn read_preferences_file() -> String {
    let path = preferences_path();
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("preferences.toml unreadable at {}: {e}", path.display()))
}

fn parse_as_toml_value() -> toml::Value {
    let content = read_preferences_file();
    toml::from_str(&content).unwrap_or_else(|e| {
        panic!("preferences.toml is not valid TOML: {e}\n\nA TOML parse failure means cernio's loader will silently fall back to defaults and skip every tuning the file encodes.");
    })
}

/// Resolve a dotted path like "search_filters.locations.greenhouse.patterns"
/// against a parsed TOML value. Returns None at the first missing segment.
fn dotted_get<'a>(root: &'a toml::Value, path: &str) -> Option<&'a toml::Value> {
    path.split('.').try_fold(root, |v, key| v.get(key))
}

fn assert_array_at<'a>(root: &'a toml::Value, path: &str) -> &'a toml::value::Array {
    let v = dotted_get(root, path)
        .unwrap_or_else(|| panic!("missing key `{path}` in preferences.toml"));
    v.as_array()
        .unwrap_or_else(|| panic!("`{path}` must be an array, got {v:?}"))
}

fn assert_string_at<'a>(root: &'a toml::Value, path: &str) -> &'a str {
    let v = dotted_get(root, path)
        .unwrap_or_else(|| panic!("missing key `{path}` in preferences.toml"));
    v.as_str()
        .unwrap_or_else(|| panic!("`{path}` must be a string, got {v:?}"))
}

fn assert_bool_at(root: &toml::Value, path: &str) -> bool {
    let v = dotted_get(root, path)
        .unwrap_or_else(|| panic!("missing key `{path}` in preferences.toml"));
    v.as_bool()
        .unwrap_or_else(|| panic!("`{path}` must be a boolean, got {v:?}"))
}

fn assert_integer_at(root: &toml::Value, path: &str) -> i64 {
    let v = dotted_get(root, path)
        .unwrap_or_else(|| panic!("missing key `{path}` in preferences.toml"));
    v.as_integer()
        .unwrap_or_else(|| panic!("`{path}` must be an integer, got {v:?}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// File-level integrity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn preferences_file_exists_at_standard_path() {
    let path = preferences_path();
    assert!(
        path.exists(),
        "preferences.toml not found at {}. The Rust pipeline reads from this exact path at every CLI invocation.",
        path.display()
    );
}

#[test]
fn preferences_file_parses_as_valid_toml() {
    // Failure mode this guards: a stray comma, unbalanced bracket, or unquoted
    // string in preferences.toml. The loader would log a warning and use
    // defaults — search would run with no filters and return thousands of
    // off-target jobs. This test fails loudly at build time instead.
    let _ = parse_as_toml_value();
}

#[test]
fn preferences_parses_strictly_through_preferences_struct() {
    // Failure mode this guards: the file parses as TOML but does not match
    // the shape `Preferences` expects (e.g. `stale_days = "fourteen"` instead
    // of an integer). The lenient `load_from` would swallow this and return
    // defaults; this test catches it.
    let content = read_preferences_file();
    let _: Preferences = toml::from_str(&content).unwrap_or_else(|e| {
        panic!("preferences.toml does not parse as Preferences struct: {e}\n\nThis means src/config.rs would silently fall back to defaults at runtime.")
    });
}

#[test]
fn loader_does_not_silently_fall_back_to_defaults() {
    // The lenient `load_from` returns defaults on parse failure. If that
    // path were taken, our actual tuned values would not be present. Sanity
    // check: at least one of our tuned fields differs from the struct default.
    // Defaults: include_keywords is empty.
    let prefs = Preferences::load_from(&preferences_path());
    assert!(
        !prefs.search_filters.include_keywords.is_empty(),
        "include_keywords is empty after loading — the lenient loader fell back to defaults silently. This means a malformed preferences.toml is being masked."
    );
    assert!(
        !prefs.search_filters.exclude_keywords.is_empty(),
        "exclude_keywords is empty after loading — fallback to defaults occurred."
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// [hard] section — read by skills, not by Rust code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn hard_section_exists_with_required_fields() {
    let root = parse_as_toml_value();
    let _ = dotted_get(&root, "hard")
        .unwrap_or_else(|| panic!("missing `[hard]` section. Skills load this for grading."));
    assert_array_at(&root, "hard.locations");
    assert_array_at(&root, "hard.exclude_locations");
    assert_bool_at(&root, "hard.requires_sponsorship");
    assert_string_at(&root, "hard.seniority_min");
    assert_string_at(&root, "hard.seniority_max");
    assert_array_at(&root, "hard.exclude_role_types");
    assert_array_at(&root, "hard.exclude_sectors");
    assert_array_at(&root, "hard.tech_must_have");
}

#[test]
fn hard_locations_includes_uk_target() {
    let root = parse_as_toml_value();
    let locs = assert_array_at(&root, "hard.locations");
    let strings: Vec<&str> = locs.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        strings.iter().any(|s| s.contains("London") || s.contains("UK") || s.contains("Remote-UK")),
        "hard.locations does not include any UK target (London/UK/Remote-UK). Cernio is a UK-focused job-discovery tool — removing UK targets would break the entire premise. Found: {strings:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// [soft] section — read by skills
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn soft_section_exists_with_required_arrays() {
    let root = parse_as_toml_value();
    let _ = dotted_get(&root, "soft")
        .unwrap_or_else(|| panic!("missing `[soft]` section. Skills load this for grading."));
    assert_array_at(&root, "soft.tech_preferred");
    assert_array_at(&root, "soft.sectors_preferred");
    assert_array_at(&root, "soft.positive_signals");
    assert_array_at(&root, "soft.negative_signals");
}

// ─────────────────────────────────────────────────────────────────────────────
// [search_filters] section — read by src/config.rs at every CLI invocation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn search_filters_section_exists_with_required_fields() {
    let root = parse_as_toml_value();
    let _ = dotted_get(&root, "search_filters")
        .unwrap_or_else(|| panic!("missing `[search_filters]` section. src/config.rs reads this at every CLI invocation."));
    assert_string_at(&root, "search_filters.min_company_grade");
    assert_array_at(&root, "search_filters.include_keywords");
    assert_array_at(&root, "search_filters.exclude_keywords");
}

#[test]
fn min_company_grade_is_a_valid_letter() {
    let root = parse_as_toml_value();
    let grade = assert_string_at(&root, "search_filters.min_company_grade");
    assert!(
        VALID_COMPANY_GRADES.contains(&grade),
        "search_filters.min_company_grade = `{grade}` is not in {VALID_COMPANY_GRADES:?}. src/config.rs::included_grades would silently default to `B`."
    );
}

#[test]
fn include_keywords_is_non_empty() {
    let root = parse_as_toml_value();
    let kws = assert_array_at(&root, "search_filters.include_keywords");
    assert!(
        !kws.is_empty(),
        "search_filters.include_keywords is empty — every job title would pass the inclusion filter, defeating the whole filter chain."
    );
}

#[test]
fn include_keywords_contains_engineering_terms() {
    // Sanity: this is an engineering job search. If "Engineer" or "Developer"
    // is missing, the filter is broken in a non-obvious way.
    let root = parse_as_toml_value();
    let kws = assert_array_at(&root, "search_filters.include_keywords");
    let strings: Vec<&str> = kws.iter().filter_map(|v| v.as_str()).collect();
    let has_engineering = strings.iter().any(|s| {
        let lower = s.to_lowercase();
        lower.contains("engineer") || lower.contains("developer") || lower.contains("swe")
    });
    assert!(
        has_engineering,
        "search_filters.include_keywords contains no engineering term (Engineer/Developer/SWE). Found: {strings:?}"
    );
}

#[test]
fn exclude_keywords_is_non_empty() {
    let root = parse_as_toml_value();
    let kws = assert_array_at(&root, "search_filters.exclude_keywords");
    assert!(
        !kws.is_empty(),
        "search_filters.exclude_keywords is empty — every Senior/Manager/Sales role would survive the exclusion filter, drowning the AI grading queue."
    );
}

#[test]
fn exclude_keywords_blocks_seniority_terms() {
    // Cernio targets entry-level roles. Senior/Principal/Lead must be excluded
    // for the funnel to make sense. This test guards against an accidental
    // delete that would let those roles flood the grading queue.
    let root = parse_as_toml_value();
    let kws = assert_array_at(&root, "search_filters.exclude_keywords");
    let strings: Vec<&str> = kws.iter().filter_map(|v| v.as_str()).collect();
    let lower: Vec<String> = strings.iter().map(|s| s.to_lowercase()).collect();
    for seniority_term in &["senior", "principal", "staff", "lead", "manager"] {
        assert!(
            lower.iter().any(|s| s == seniority_term),
            "search_filters.exclude_keywords missing `{seniority_term}` (case-insensitive). Entry-level focus depends on these exclusions. Found: {strings:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// [search_filters.locations.<provider>] subtables — per-ATS coverage
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn locations_subtable_exists() {
    let root = parse_as_toml_value();
    let _ = dotted_get(&root, "search_filters.locations").unwrap_or_else(|| {
        panic!("missing `[search_filters.locations]` parent table. Per-provider location filters are configured under this.")
    });
}

#[test]
fn every_supported_ats_provider_has_a_location_subtable() {
    // The most important integrity test: a new ATS provider added to
    // `src/ats/` without a corresponding location subtable here would
    // silently bypass the location filter for all of that provider's jobs
    // (per src/config.rs::passes_location, unknown provider = pass-through).
    let root = parse_as_toml_value();
    for provider in SUPPORTED_ATS_PROVIDERS {
        let path = format!("search_filters.locations.{provider}");
        let _ = dotted_get(&root, &path).unwrap_or_else(|| {
            panic!(
                "missing `[search_filters.locations.{provider}]`. The provider exists in `src/ats/{provider}.rs` but has no location filter — every {provider} job would bypass the UK location filter and reach the AI grader unfiltered."
            )
        });
    }
}

#[test]
fn every_provider_subtable_has_a_patterns_array() {
    let root = parse_as_toml_value();
    for provider in SUPPORTED_ATS_PROVIDERS {
        let path = format!("search_filters.locations.{provider}.patterns");
        let arr = assert_array_at(&root, &path);
        assert!(
            !arr.is_empty(),
            "`{path}` is an empty array. Per src/config.rs::passes_location, an empty patterns array degrades to pass-through (every location accepted). If the intent is no filtering, that should be done by removing the subtable entirely; if filtering is wanted, add patterns."
        );
    }
}

#[test]
fn every_provider_subtable_has_a_uk_pattern() {
    // Sanity: this is a UK-focused tool. Each provider's location subtable
    // must include at least one UK marker (London / UK / GB / United Kingdom).
    let root = parse_as_toml_value();
    for provider in SUPPORTED_ATS_PROVIDERS {
        let path = format!("search_filters.locations.{provider}.patterns");
        let arr = assert_array_at(&root, &path);
        let strings: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
        let has_uk = strings.iter().any(|s| {
            let lower = s.to_lowercase();
            lower == "london"
                || lower == "uk"
                || lower == "gb"
                || lower == "united kingdom"
                || lower == "england"
                || lower == "cambridge"
                || lower.contains("remote")
        });
        assert!(
            has_uk,
            "`{path}` has no UK / Cambridge / Remote pattern. Found: {strings:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// [cleanup] section — read by src/config.rs and the cernio clean command
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cleanup_section_exists_with_required_fields() {
    let root = parse_as_toml_value();
    let _ = dotted_get(&root, "cleanup")
        .unwrap_or_else(|| panic!("missing `[cleanup]` section. src/config.rs reads this for `cernio clean` operations."));
    assert_array_at(&root, "cleanup.remove_job_grades");
    assert_integer_at(&root, "cleanup.stale_days");
    assert_array_at(&root, "cleanup.archive_company_grades");
}

#[test]
fn cleanup_stale_days_is_positive() {
    let root = parse_as_toml_value();
    let days = assert_integer_at(&root, "cleanup.stale_days");
    assert!(
        days > 0,
        "cleanup.stale_days = {days}. Must be a positive integer — a zero or negative value would either delete every job on the next clean run or never delete anything."
    );
}

#[test]
fn cleanup_remove_job_grades_uses_valid_letters() {
    let root = parse_as_toml_value();
    let arr = assert_array_at(&root, "cleanup.remove_job_grades");
    for v in arr {
        let s = v.as_str().unwrap_or_else(|| panic!("non-string entry in cleanup.remove_job_grades: {v:?}"));
        assert!(
            VALID_JOB_GRADES.contains(&s),
            "cleanup.remove_job_grades contains `{s}`, not in {VALID_JOB_GRADES:?}. The cleanup script would silently skip jobs with this grade."
        );
    }
}

#[test]
fn cleanup_archive_company_grades_uses_valid_letters() {
    let root = parse_as_toml_value();
    let arr = assert_array_at(&root, "cleanup.archive_company_grades");
    for v in arr {
        let s = v.as_str().unwrap_or_else(|| panic!("non-string entry in cleanup.archive_company_grades: {v:?}"));
        assert!(
            VALID_COMPANY_GRADES.contains(&s),
            "cleanup.archive_company_grades contains `{s}`, not in {VALID_COMPANY_GRADES:?}."
        );
    }
}
