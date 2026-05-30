//! Integrity tests for `profile/preferences.toml`.
//!
//! Why this file matters: `profile/preferences.toml` is read at every CLI
//! invocation by `src/config.rs`, but the loader is intentionally lenient —
//! a malformed TOML or a missing field silently falls back to defaults with
//! a stderr warning. A typo would not surface as a build failure; it would
//! surface as the search pipeline running with default filters (effectively
//! ignoring all of the user's tuning, returning thousands of off-target
//! jobs).
//!
//! Scope: this file holds ONLY mechanical Rust pipeline configuration —
//! search keyword filters, per-ATS location patterns, min_company_grade,
//! and cleanup thresholds. Everything preference-shaped (taste, sectors,
//! role exclusions, signals, seniority, lifestyle) lives in profile/ as
//! prose and is read by grading skills directly. There is no [hard] or
//! [soft] section here anymore.
//!
//! These tests assert that the actual file in the repo:
//! - Is readable from the standard path
//! - Parses as valid TOML
//! - Survives strict parsing through the `Preferences` struct (no fallback)
//! - Contains every section + key the runtime depends on
//! - Has the right shape (arrays vs strings vs numbers) per value
//! - Has location-pattern coverage for every ATS provider in `src/ats/`
//! - Holds sensible values (positive numbers, valid grade letters, non-empty
//!   keyword lists, seniority terms in the exclusion list)
//!
//! When this file fails: stop. Do not commit changes to preferences.toml that
//! make these tests fail without first updating the tests in lockstep — the
//! tests document the contract that `src/config.rs` depends on.

use cernio::config::Preferences;
use std::fs;
use std::path::PathBuf;

const PREFERENCES_FILE: &str = "profile/preferences.toml";

/// Valid `min_company_grade` values per `src/config.rs::included_grades`.
const VALID_COMPANY_GRADES: &[&str] = &["S", "A", "B", "C"];

/// Valid `archive_job_grades` values per the job grading rubric.
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
// No-[hard]-or-[soft] regression guard
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn no_hard_or_soft_sections_present() {
    // preferences.toml was pruned to mechanical-only in May 2026. The [hard]
    // and [soft] sections were duplicating profile/ content and acting as a
    // snapshot of taste that the grading skills now infer from career-goals.md,
    // lifestyle-preferences.md, skills/, and projects/. A re-introduction
    // would reintroduce the snapshot anti-pattern.
    let root = parse_as_toml_value();
    assert!(
        dotted_get(&root, "hard").is_none(),
        "preferences.toml has re-grown a [hard] section. Preference-shaped data belongs in profile/ as prose, not here. This file is for mechanical Rust pipeline config only."
    );
    assert!(
        dotted_get(&root, "soft").is_none(),
        "preferences.toml has re-grown a [soft] section. Preference-shaped data belongs in profile/ as prose, not here. This file is for mechanical Rust pipeline config only."
    );
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
// [search_filters.locations] — single shared list
// ─────────────────────────────────────────────────────────────────────────────
//
// Was previously per-ATS-provider subtables (greenhouse, lever, ashby,
// workable, smartrecruiters, workday). Collapsed to a single shared list
// because every provider matched the same UK + remote vocabulary
// case-insensitively and the per-provider split was 100% duplicated content.

#[test]
fn locations_section_exists() {
    let root = parse_as_toml_value();
    let _ = dotted_get(&root, "search_filters.locations").unwrap_or_else(|| {
        panic!("missing `[search_filters.locations]`. src/config.rs reads this single shared list at every CLI invocation.")
    });
    assert_array_at(&root, "search_filters.locations.patterns");
}

#[test]
fn locations_patterns_is_non_empty() {
    let root = parse_as_toml_value();
    let arr = assert_array_at(&root, "search_filters.locations.patterns");
    assert!(
        !arr.is_empty(),
        "`search_filters.locations.patterns` is empty. Per src/config.rs::passes_location, an empty patterns array degrades to pass-through (every location accepted). For a UK-focused tool this is almost certainly a mistake."
    );
}

#[test]
fn locations_patterns_includes_uk_markers() {
    // Sanity: this is a UK-focused tool. The shared list must include at
    // least one UK marker (London / UK / GB / United Kingdom / Remote).
    let root = parse_as_toml_value();
    let arr = assert_array_at(&root, "search_filters.locations.patterns");
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
        "`search_filters.locations.patterns` has no UK / Cambridge / Remote marker. Found: {strings:?}"
    );
}

#[test]
fn locations_patterns_no_per_provider_subtables() {
    // Regression guard: the per-provider split was retired because it was
    // 100% duplicated content. A re-introduction would reintroduce the
    // maintenance burden it was deleted to solve.
    let root = parse_as_toml_value();
    for provider in &["greenhouse", "lever", "ashby", "workable", "smartrecruiters", "workday"] {
        let path = format!("search_filters.locations.{provider}");
        assert!(
            dotted_get(&root, &path).is_none(),
            "`[search_filters.locations.{provider}]` has re-grown. The per-provider location split was retired in favour of a single shared `[search_filters.locations] patterns = [...]` list. See src/config.rs — passes_location takes only locations, no provider arg."
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
    assert_array_at(&root, "cleanup.archive_job_grades");
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
fn cleanup_archive_job_grades_uses_valid_letters() {
    let root = parse_as_toml_value();
    let arr = assert_array_at(&root, "cleanup.archive_job_grades");
    for v in arr {
        let s = v.as_str().unwrap_or_else(|| panic!("non-string entry in cleanup.archive_job_grades: {v:?}"));
        assert!(
            VALID_JOB_GRADES.contains(&s),
            "cleanup.archive_job_grades contains `{s}`, not in {VALID_JOB_GRADES:?}. The cleanup script would silently skip jobs with this grade."
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
