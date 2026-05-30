use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Parsed search filters from profile/preferences.toml.
#[derive(Debug, Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub search_filters: SearchFilters,
    #[serde(default)]
    pub cleanup: CleanupConfig,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    /// Minimum company grade to include in job searches (default: "B").
    #[serde(default = "default_min_grade")]
    pub min_company_grade: String,

    /// Title must contain at least one of these (OR logic).
    #[serde(default)]
    pub include_keywords: Vec<String>,

    /// Title containing any of these is excluded (AND NOT logic).
    #[serde(default)]
    pub exclude_keywords: Vec<String>,

    /// Shared location patterns applied across every ATS provider.
    /// The provider distinction was dropped — all six providers produce
    /// freetext location strings that match the same UK+remote vocabulary
    /// (case-insensitive `contains`), so per-provider lists were 100%
    /// duplicated content.
    #[serde(default)]
    pub locations: LocationConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct LocationConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupConfig {
    /// Job grades to remove during cleanup.
    #[serde(default = "default_archive_job_grades")]
    pub archive_job_grades: Vec<String>,

    /// Job age threshold in days.
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,

    /// Company grades to archive during cleanup.
    #[serde(default = "default_archive_grades")]
    pub archive_company_grades: Vec<String>,
}

fn default_min_grade() -> String {
    "B".to_string()
}
fn default_archive_job_grades() -> Vec<String> {
    vec!["F".to_string(), "C".to_string()]
}
fn default_stale_days() -> u32 {
    14
}
fn default_archive_grades() -> Vec<String> {
    vec!["C".to_string()]
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            min_company_grade: default_min_grade(),
            include_keywords: Vec::new(),
            exclude_keywords: Vec::new(),
            locations: LocationConfig::default(),
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            archive_job_grades: default_archive_job_grades(),
            stale_days: default_stale_days(),
            archive_company_grades: default_archive_grades(),
        }
    }
}

impl Preferences {
    /// Load preferences from the standard path (profile/preferences.toml).
    pub fn load() -> Self {
        Self::load_from(Path::new("profile/preferences.toml"))
    }

    /// Load preferences from a specific path.
    pub fn load_from(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Warning: failed to parse {}: {e}", path.display());
                eprintln!("Using default preferences.");
                Self::default()
            }),
            Err(_) => {
                eprintln!(
                    "Warning: preferences file not found at {}",
                    path.display()
                );
                eprintln!("Using default preferences.");
                Self::default()
            }
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            search_filters: SearchFilters::default(),
            cleanup: CleanupConfig::default(),
        }
    }
}

impl SearchFilters {
    /// Check if a title passes the exclusion filter.
    /// Returns false if the title contains any excluded keyword.
    pub fn passes_exclusion(&self, title: &str) -> bool {
        let lower = title.to_lowercase();
        !self
            .exclude_keywords
            .iter()
            .any(|kw| lower.contains(&kw.to_lowercase()))
    }

    /// Check if a title passes the inclusion filter.
    /// Returns true if the title contains at least one included keyword,
    /// or if no include keywords are configured (pass-through).
    pub fn passes_inclusion(&self, title: &str) -> bool {
        if self.include_keywords.is_empty() {
            return true;
        }
        let lower = title.to_lowercase();
        self.include_keywords
            .iter()
            .any(|kw| lower.contains(&kw.to_lowercase()))
    }

    /// Check if a job's locations pass the shared location filter.
    /// Returns true if ANY location string contains ANY configured pattern,
    /// also returns true if locations is empty (false negative protection)
    /// or if no patterns are configured (pass-through default).
    pub fn passes_location(&self, locations: &[String]) -> bool {
        // No location data → keep the job (false negative protection).
        if locations.is_empty() {
            return true;
        }

        // No patterns configured → keep everything.
        if self.locations.patterns.is_empty() {
            return true;
        }

        // Any location matching any pattern → keep.
        for loc in locations {
            let lower = loc.to_lowercase();
            for pattern in &self.locations.patterns {
                if lower.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }

        false
    }

    /// Returns the grade values that should be included in searches.
    /// E.g., if min_company_grade is "B", returns ["S", "A", "B"].
    ///
    /// Values are static string literals so the return lifetime is `'static`
    /// — callers can hold onto the result independently of `self`.
    pub fn included_grades(&self) -> Vec<&'static str> {
        let all: [&'static str; 4] = ["S", "A", "B", "C"];
        let min_idx = all
            .iter()
            .position(|g| g.eq_ignore_ascii_case(&self.min_company_grade))
            .unwrap_or(2); // Default to B
        all[..=min_idx].to_vec()
    }
}

// ══════════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn filters_with(include: Vec<&str>, exclude: Vec<&str>) -> SearchFilters {
        SearchFilters {
            min_company_grade: "B".to_string(),
            include_keywords: include.into_iter().map(String::from).collect(),
            exclude_keywords: exclude.into_iter().map(String::from).collect(),
            locations: LocationConfig::default(),
        }
    }

    // ─────────────────────────────────────────────────────────────
    // passes_exclusion
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn exclusion_empty_list_passes_everything() {
        let f = SearchFilters::default();
        assert!(f.passes_exclusion("Senior Rust Engineer"));
        assert!(f.passes_exclusion(""));
    }

    #[test]
    fn exclusion_blocks_on_exact_substring() {
        let f = filters_with(vec![], vec!["Senior"]);
        assert!(!f.passes_exclusion("Senior Rust Engineer"));
        assert!(f.passes_exclusion("Junior Rust Engineer"));
    }

    #[test]
    fn exclusion_is_case_insensitive() {
        let f = filters_with(vec![], vec!["SENIOR"]);
        assert!(!f.passes_exclusion("senior rust engineer"));
        let f = filters_with(vec![], vec!["senior"]);
        assert!(!f.passes_exclusion("SENIOR RUST ENGINEER"));
    }

    #[test]
    fn exclusion_matches_substring_anywhere() {
        let f = filters_with(vec![], vec!["Manager"]);
        assert!(!f.passes_exclusion("Engineering Manager"));
        assert!(!f.passes_exclusion("Manager of Platform"));
    }

    #[test]
    fn exclusion_matches_any_of_multiple_keywords() {
        let f = filters_with(vec![], vec!["Sales", "Support", "Manager"]);
        assert!(!f.passes_exclusion("Sales Engineer"));
        assert!(!f.passes_exclusion("Support Technician"));
        assert!(!f.passes_exclusion("Product Manager"));
        assert!(f.passes_exclusion("Backend Developer"));
    }

    #[test]
    fn exclusion_does_not_leak_partial_word_false_positives() {
        // "Lead" substring appears in "Leader" — this is intentional because
        // "Lead Engineer" is the target, and substring matching is by design.
        let f = filters_with(vec![], vec!["Lead"]);
        assert!(!f.passes_exclusion("Team Lead"));
        assert!(!f.passes_exclusion("Leadership Coach"));
    }

    // ─────────────────────────────────────────────────────────────
    // passes_inclusion
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn inclusion_empty_list_is_passthrough() {
        let f = SearchFilters::default();
        assert!(f.passes_inclusion("Anything"));
        assert!(f.passes_inclusion(""));
    }

    #[test]
    fn inclusion_requires_at_least_one_match() {
        let f = filters_with(vec!["Engineer", "Developer"], vec![]);
        assert!(f.passes_inclusion("Software Engineer"));
        assert!(f.passes_inclusion("Backend Developer"));
        assert!(!f.passes_inclusion("Product Manager"));
    }

    #[test]
    fn inclusion_is_case_insensitive() {
        let f = filters_with(vec!["RUST"], vec![]);
        assert!(f.passes_inclusion("junior rust engineer"));
        let f = filters_with(vec!["rust"], vec![]);
        assert!(f.passes_inclusion("JUNIOR RUST ENGINEER"));
    }

    #[test]
    fn inclusion_matches_any_keyword() {
        let f = filters_with(vec!["ML", "AI", "Data"], vec![]);
        assert!(f.passes_inclusion("ML Engineer"));
        assert!(f.passes_inclusion("AI Researcher"));
        assert!(f.passes_inclusion("Data Scientist"));
        assert!(!f.passes_inclusion("Frontend Developer"));
    }

    // ─────────────────────────────────────────────────────────────
    // passes_location
    // ─────────────────────────────────────────────────────────────

    fn with_patterns(patterns: Vec<&str>) -> SearchFilters {
        SearchFilters {
            locations: LocationConfig {
                patterns: patterns.into_iter().map(String::from).collect(),
            },
            ..SearchFilters::default()
        }
    }

    #[test]
    fn location_empty_locations_is_kept() {
        // False-negative protection: no location data → keep the job.
        let f = with_patterns(vec!["London"]);
        assert!(f.passes_location(&[]));
    }

    #[test]
    fn location_no_patterns_configured_is_pass_through() {
        let f = SearchFilters::default();
        assert!(f.passes_location(&["Mars".to_string()]));
    }

    #[test]
    fn location_empty_patterns_is_pass_through() {
        let f = with_patterns(vec![]);
        assert!(f.passes_location(&["Mars".to_string()]));
    }

    #[test]
    fn location_matches_any_pattern() {
        let f = with_patterns(vec!["London", "Cambridge"]);
        assert!(f.passes_location(&["London, UK".to_string()]));
        assert!(f.passes_location(&["Cambridge, UK".to_string()]));
        assert!(!f.passes_location(&["Paris, FR".to_string()]));
    }

    #[test]
    fn location_case_insensitive() {
        let f = with_patterns(vec!["LONDON"]);
        assert!(f.passes_location(&["london, uk".to_string()]));
        let f = with_patterns(vec!["london"]);
        assert!(f.passes_location(&["LONDON, UK".to_string()]));
    }

    #[test]
    fn location_multiple_locations_any_match() {
        let f = with_patterns(vec!["London"]);
        let locs = vec![
            "Berlin".to_string(),
            "Paris".to_string(),
            "London".to_string(),
        ];
        assert!(f.passes_location(&locs));
    }

    #[test]
    fn location_country_code_match() {
        let f = with_patterns(vec!["GB"]);
        assert!(f.passes_location(&["GB".to_string()]));
    }

    // ─────────────────────────────────────────────────────────────
    // included_grades
    // ─────────────────────────────────────────────────────────────

    fn grades_for(min: &str) -> Vec<&'static str> {
        let mut f = SearchFilters::default();
        f.min_company_grade = min.to_string();
        f.included_grades()
    }

    #[test]
    fn grades_s_includes_only_s() {
        assert_eq!(grades_for("S"), vec!["S"]);
    }

    #[test]
    fn grades_a_includes_s_and_a() {
        assert_eq!(grades_for("A"), vec!["S", "A"]);
    }

    #[test]
    fn grades_b_includes_s_a_b() {
        assert_eq!(grades_for("B"), vec!["S", "A", "B"]);
    }

    #[test]
    fn grades_c_includes_all_four() {
        assert_eq!(grades_for("C"), vec!["S", "A", "B", "C"]);
    }

    #[test]
    fn grades_lowercase_accepted() {
        assert_eq!(grades_for("s"), vec!["S"]);
        assert_eq!(grades_for("a"), vec!["S", "A"]);
    }

    #[test]
    fn grades_unknown_value_defaults_to_b() {
        // Invalid input should be tolerated — returns default set (B).
        assert_eq!(grades_for("X"), vec!["S", "A", "B"]);
        assert_eq!(grades_for(""), vec!["S", "A", "B"]);
        assert_eq!(grades_for("SS"), vec!["S", "A", "B"]);
    }

    // ─────────────────────────────────────────────────────────────
    // Preferences::load_from (TOML parsing)
    // ─────────────────────────────────────────────────────────────

    fn write_tmp(content: &str) -> tempfile::NamedTempFile {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
        tmp.write_all(content.as_bytes()).expect("write tempfile");
        tmp.flush().expect("flush tempfile");
        tmp
    }

    #[test]
    fn load_from_missing_file_returns_defaults() {
        let prefs = Preferences::load_from(Path::new("/nonexistent/path/prefs.toml"));
        assert_eq!(prefs.search_filters.min_company_grade, "B");
        assert!(prefs.search_filters.include_keywords.is_empty());
        assert_eq!(prefs.cleanup.stale_days, 14);
        assert_eq!(prefs.cleanup.archive_job_grades, vec!["F", "C"]);
    }

    #[test]
    fn load_from_empty_file_returns_defaults() {
        let tmp = write_tmp("");
        let prefs = Preferences::load_from(tmp.path());
        assert_eq!(prefs.search_filters.min_company_grade, "B");
    }

    #[test]
    fn load_from_partial_toml_fills_in_defaults() {
        let toml = r#"
            [search_filters]
            min_company_grade = "A"
            include_keywords = ["Rust"]
        "#;
        let tmp = write_tmp(toml);
        let prefs = Preferences::load_from(tmp.path());
        assert_eq!(prefs.search_filters.min_company_grade, "A");
        assert_eq!(prefs.search_filters.include_keywords, vec!["Rust"]);
        // Defaults still applied where unspecified.
        assert_eq!(prefs.cleanup.stale_days, 14);
    }

    #[test]
    fn load_from_invalid_toml_returns_defaults() {
        let tmp = write_tmp("not { valid ][toml");
        let prefs = Preferences::load_from(tmp.path());
        assert_eq!(prefs.search_filters.min_company_grade, "B");
    }

    #[test]
    fn load_from_full_config_preserves_locations() {
        let toml = r#"
            [search_filters]
            min_company_grade = "C"
            include_keywords = ["Engineer"]
            exclude_keywords = ["Senior"]

            [search_filters.locations]
            patterns = ["London", "Cambridge", "UK"]

            [cleanup]
            archive_job_grades = ["F"]
            stale_days = 7
            archive_company_grades = []
        "#;
        let tmp = write_tmp(toml);
        let prefs = Preferences::load_from(tmp.path());
        assert_eq!(prefs.search_filters.min_company_grade, "C");
        assert_eq!(prefs.cleanup.stale_days, 7);
        assert_eq!(prefs.cleanup.archive_job_grades, vec!["F"]);
        assert!(prefs.cleanup.archive_company_grades.is_empty());

        assert_eq!(
            prefs.search_filters.locations.patterns,
            vec!["London", "Cambridge", "UK"]
        );
    }

    // ─────────────────────────────────────────────────────────────
    // Real-world filter composition — verify the pipeline we actually run
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn pipeline_rejects_senior_sales_roles() {
        // Based on the actual keywords in profile/preferences.toml.
        let f = filters_with(
            vec!["Engineer", "Developer"],
            vec!["Senior", "Sales", "Manager"],
        );
        // Each is blocked by at least one rule.
        assert!(!f.passes_exclusion("Senior Rust Engineer"));
        assert!(!f.passes_exclusion("Sales Engineer"));
        assert!(!f.passes_exclusion("Engineering Manager"));
        // Accepted:
        assert!(f.passes_exclusion("Backend Engineer"));
        assert!(f.passes_inclusion("Backend Engineer"));
    }

    #[test]
    fn pipeline_accepts_graduate_engineer_roles() {
        let f = filters_with(
            vec!["Engineer", "Graduate", "Junior"],
            vec!["Senior", "Staff", "Principal"],
        );
        assert!(f.passes_exclusion("Graduate Software Engineer"));
        assert!(f.passes_inclusion("Graduate Software Engineer"));
        assert!(f.passes_exclusion("Junior ML Engineer"));
        assert!(f.passes_inclusion("Junior ML Engineer"));
    }
}
