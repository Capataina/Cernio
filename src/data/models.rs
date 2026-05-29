//! Data models shared between the TUI and the web frontend.

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    ByGrade,
    ByCompany,
    ByDate,
    ByLocation,
}

#[derive(Debug, Clone)]
pub struct CompanyRow {
    pub id: i64,
    pub name: String,
    pub website: String,
    pub what_they_do: String,
    pub status: String,
    pub location: Option<String>,
    pub sector_tags: Option<String>,
    pub grade: Option<String>,
    pub grade_reasoning: Option<String>,
    pub why_relevant: String,
    pub careers_url: Option<String>,
    pub ats_provider: Option<String>,
    pub ats_slug: Option<String>,
    pub job_count: i64,
    pub fit_count: i64,
    /// JSON array of lane keys per the lane-based-relativity refactor.
    pub lanes: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JobRow {
    pub id: i64,
    pub company_id: i64,
    pub company_name: String,
    pub title: String,
    pub url: String,
    pub location: Option<String>,
    pub remote_policy: Option<String>,
    pub posted_date: Option<String>,
    pub evaluation_status: String,
    pub fit_assessment: Option<String>,
    pub grade: Option<String>,
    pub evidence_basis: Option<String>,
    pub raw_description: Option<String>,
    pub decision: Option<String>,
    pub has_package: bool,
    /// JSON array of lane keys; primary lane is first.
    pub lanes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub occurred_at: String,
    pub date: String,
    pub event_type: String,
    pub subject_label: String,
    pub lane: Option<String>,
    pub grade: Option<String>,
    pub source: String,
    pub detail_json: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DashboardStats {
    pub total_companies: i64,
    pub companies_by_grade: Vec<(String, i64)>,
    pub companies_by_status: Vec<(String, i64)>,
    pub total_jobs: i64,
    pub jobs_by_eval: Vec<(String, i64)>,
    pub jobs_by_grade: Vec<(String, i64)>,
    pub ats_coverage: Vec<(String, i64)>,
    pub top_matches: Vec<TopMatch>,
    pub pending_companies: i64,
    pub bespoke_count: i64,
    pub archived_count: i64,
    pub applied_count: i64,
    pub watching_count: i64,
    pub rejected_count: i64,
    pub bespoke_searchable: i64,
    pub needs_description: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TopMatch {
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobFilters {
    pub grades: HashSet<String>,
    pub ats: HashSet<String>,
    pub decisions: HashSet<String>,
    pub package: HashSet<String>,
    pub archive: HashSet<String>,
    pub evidence: HashSet<String>,
    pub lanes: HashSet<String>,
}

impl Default for JobFilters {
    fn default() -> Self {
        let mut archive = HashSet::new();
        archive.insert("active".to_string());
        let mut evidence = HashSet::new();
        evidence.insert("jd".to_string());
        evidence.insert("semantic".to_string());
        Self {
            grades: HashSet::new(),
            ats: HashSet::new(),
            decisions: HashSet::new(),
            package: HashSet::new(),
            archive,
            evidence,
            lanes: HashSet::new(),
        }
    }
}

impl JobFilters {
    pub fn active_chip_count(&self) -> usize {
        self.grades.len()
            + self.ats.len()
            + self.decisions.len()
            + self.package.len()
            + self.archive.len()
            + self.evidence.len()
            + self.lanes.len()
    }

    pub fn is_default(&self) -> bool {
        self.grades.is_empty()
            && self.ats.is_empty()
            && self.decisions.is_empty()
            && self.package.is_empty()
            && self.archive.len() == 1
            && self.archive.contains("active")
            && self.evidence.len() == 2
            && self.evidence.contains("jd")
            && self.evidence.contains("semantic")
            && self.lanes.is_empty()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn clear_all(&mut self) {
        self.grades.clear();
        self.ats.clear();
        self.decisions.clear();
        self.package.clear();
        self.archive.clear();
        self.evidence.clear();
        self.lanes.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAxis {
    Grade,
    Lane,
    Ats,
    Decision,
    Package,
    Archive,
    Evidence,
}

impl FilterAxis {
    pub const ALL: [FilterAxis; 7] = [
        FilterAxis::Grade,
        FilterAxis::Lane,
        FilterAxis::Ats,
        FilterAxis::Decision,
        FilterAxis::Package,
        FilterAxis::Archive,
        FilterAxis::Evidence,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FilterAxis::Grade => "Grade",
            FilterAxis::Lane => "Lane",
            FilterAxis::Ats => "ATS",
            FilterAxis::Decision => "Decision",
            FilterAxis::Package => "Package",
            FilterAxis::Archive => "Archive",
            FilterAxis::Evidence => "Evidence",
        }
    }

    pub fn chips(self) -> &'static [&'static str] {
        match self {
            FilterAxis::Grade => &["SS", "S", "A", "B", "C", "F", "?"],
            FilterAxis::Lane => &[
                "big-tech", "ai-ml", "hft", "crypto-mm",
                "bank-strats", "systems-infra", "devtools", "fintech",
            ],
            FilterAxis::Ats => &[
                "greenhouse", "ashby", "lever", "workable",
                "smartrecruiters", "workday", "eightfold", "bespoke",
            ],
            FilterAxis::Decision => &["applied", "watching", "interview", "rejected", "none"],
            FilterAxis::Package => &["prepared", "not-prepared"],
            FilterAxis::Archive => &["archived", "active"],
            FilterAxis::Evidence => &["jd", "semantic", "insufficient"],
        }
    }
}
