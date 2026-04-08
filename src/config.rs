use serde::Deserialize;
use std::collections::HashMap;
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

    /// Location patterns per ATS provider.
    #[serde(default)]
    pub locations: HashMap<String, LocationConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LocationConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupConfig {
    /// Job grades to remove during cleanup.
    #[serde(default = "default_remove_grades")]
    pub remove_job_grades: Vec<String>,

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
fn default_remove_grades() -> Vec<String> {
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
            locations: HashMap::new(),
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            remove_job_grades: default_remove_grades(),
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

    /// Check if a job's locations pass the location filter for a given provider.
    /// Returns true if ANY location string contains ANY pattern for this provider.
    /// Also returns true if locations is empty (false negative protection).
    pub fn passes_location(&self, provider: &str, locations: &[String]) -> bool {
        // No location data → keep the job (false negative protection).
        if locations.is_empty() {
            return true;
        }

        // No patterns configured for this provider → keep everything.
        let Some(config) = self.locations.get(provider) else {
            return true;
        };

        if config.patterns.is_empty() {
            return true;
        }

        // Any location matching any pattern → keep.
        for loc in locations {
            let lower = loc.to_lowercase();
            for pattern in &config.patterns {
                if lower.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }

        false
    }

    /// Returns the grade values that should be included in searches.
    /// E.g., if min_company_grade is "B", returns ["S", "A", "B"].
    pub fn included_grades(&self) -> Vec<&str> {
        let all = ["S", "A", "B", "C"];
        let min_idx = all
            .iter()
            .position(|g| g.eq_ignore_ascii_case(&self.min_company_grade))
            .unwrap_or(2); // Default to B
        all[..=min_idx].to_vec()
    }
}
