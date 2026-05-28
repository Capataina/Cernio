use std::collections::HashSet;
use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::widgets::TableState;

use super::super::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Companies,
    Jobs,
    Pipeline,
    Activity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    ByGrade,
    ByCompany,
    ByDate,
    ByLocation,
}

impl View {
    pub fn index(self) -> usize {
        match self {
            View::Dashboard => 0,
            View::Companies => 1,
            View::Jobs => 2,
            View::Pipeline => 3,
            View::Activity => 4,
        }
    }
}

pub struct ActivityEntry {
    pub date: String,       // "2026-04-09"
    pub action: String,     // "applied", "searched", "graded", "discovered", "watching", "rejected"
    pub detail: String,     // "Software Engineer — Citadel" or "270 companies searched"
}

/// Which column is focused in the Pipeline/Kanban view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineColumn {
    Watching,
    Applied,
    Interview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

// ── Data models ──────────────────────────────────────────────────

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
    /// Lane tags for the company per the lane-based-relativity refactor
    /// (cernio-full-refactor.md §3 + migration_008). JSON array stored
    /// in the DB as TEXT; displayed compactly in the TUI companies view.
    /// None means the company has no lane tag yet (should be rare post-
    /// migration; cernio clean deletes no-lane companies).
    pub lanes: Option<String>,
}

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
}

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

#[allow(dead_code)]
pub struct TopMatch {
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub grade: Option<String>,
}

// ── Application state ────────────────────────────────────────────

#[allow(dead_code)]
pub struct Toast {
    pub message: String,
    pub created_at: std::time::Instant,
}

/// A card in the Pipeline/Kanban view.
pub struct PipelineCard {
    pub job_id: i64,
    pub title: String,
    pub company: String,
    pub grade: Option<String>,
}

// ── Filter model ─────────────────────────────────────────────────

/// Multi-axis job filter. Each axis is a "show only this" set: an empty
/// set means no filter on that axis (all values pass); a non-empty set
/// shows only jobs matching one of the listed values (OR within axis).
/// Across axes the filters AND together.
#[derive(Debug, Clone)]
pub struct JobFilters {
    /// Grade values to include. Sentinel "?" matches NULL grade.
    pub grades: HashSet<String>,
    /// ATS provider values to include. Sentinel "bespoke" matches companies
    /// without a portal entry (status = 'bespoke' or no row in company_portals).
    pub ats: HashSet<String>,
    /// Decision values to include. Sentinel "none" matches jobs with no
    /// user_decisions row.
    pub decisions: HashSet<String>,
    /// Package filter — "prepared" / "not-prepared". Empty = both.
    pub package: HashSet<String>,
    /// Archive filter — "archived" / "active". Default {"active"} preserves
    /// the historical hide-archived-by-default behaviour.
    pub archive: HashSet<String>,
    /// Evidence-basis filter — "jd" / "semantic" / "insufficient". Empty = all.
    /// 'jd' = grade derived from JD content; 'semantic' = grade from company+
    /// title reasoning when JD missing; 'insufficient' = could not grade at all.
    /// Default {"jd","semantic"} hides only the truly-ungradable rows but keeps
    /// semantically-graded brand-strong roles visible.
    pub evidence: HashSet<String>,
}

impl Default for JobFilters {
    fn default() -> Self {
        let mut archive = HashSet::new();
        archive.insert("active".to_string());
        let mut evidence = HashSet::new();
        evidence.insert("jd".to_string());
        evidence.insert("semantic".to_string());
        // 'insufficient' deliberately omitted from the default set so genuinely-
        // ungradable rows are hidden by default. 'semantic' is included so
        // brand-strong grad-role grades (e.g. Google SS via context reasoning)
        // remain visible.
        Self {
            grades: HashSet::new(),
            ats: HashSet::new(),
            decisions: HashSet::new(),
            package: HashSet::new(),
            archive,
            evidence,
        }
    }
}

impl JobFilters {
    /// Total number of chips currently active across all axes.
    pub fn active_chip_count(&self) -> usize {
        self.grades.len()
            + self.ats.len()
            + self.decisions.len()
            + self.package.len()
            + self.archive.len()
            + self.evidence.len()
    }

    /// Returns true iff every axis is at its default state.
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
    }

    /// Reset every axis to default.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Clear every chip (including the default "active" on archive).
    pub fn clear_all(&mut self) {
        self.grades.clear();
        self.ats.clear();
        self.decisions.clear();
        self.package.clear();
        self.archive.clear();
        self.evidence.clear();
    }
}

/// Filter-menu axis enumeration — drives the visual rows in the overlay
/// and the keyboard navigation between rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAxis {
    Grade,
    Ats,
    Decision,
    Package,
    Archive,
    Evidence,
}

impl FilterAxis {
    pub const ALL: [FilterAxis; 6] = [
        FilterAxis::Grade,
        FilterAxis::Ats,
        FilterAxis::Decision,
        FilterAxis::Package,
        FilterAxis::Archive,
        FilterAxis::Evidence,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FilterAxis::Grade => "Grade",
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
            FilterAxis::Ats => &[
                "greenhouse",
                "ashby",
                "lever",
                "workable",
                "smartrecruiters",
                "workday",
                "eightfold",
                "bespoke",
            ],
            FilterAxis::Decision => &["applied", "watching", "interview", "rejected", "none"],
            FilterAxis::Package => &["prepared", "not-prepared"],
            FilterAxis::Archive => &["archived", "active"],
            FilterAxis::Evidence => &["jd", "semantic", "insufficient"],
        }
    }
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub focus: Focus,
    pub theme: Theme,
    pub show_help: bool,
    pub detail_scroll: u16,
    pub filters: JobFilters,
    pub show_filter_menu: bool,
    pub filter_menu_axis: usize,   // 0..5 (matches FilterAxis::ALL)
    pub filter_menu_chip: usize,   // chip index within the focused axis
    pub frame_count: u64,
    pub toasts: Vec<Toast>,

    pub companies: Vec<CompanyRow>,
    pub company_state: TableState,

    pub jobs: Vec<JobRow>,
    pub job_state: TableState,
    pub job_filter_company: Option<i64>,
    pub job_filter_company_name: Option<String>,

    // ── Multi-select ─────────────────────────────────────────────
    pub multi_select_jobs: HashSet<usize>,
    pub multi_select_companies: HashSet<usize>,
    pub anchor_job: Option<usize>,      // for shift-click range select
    pub anchor_company: Option<usize>,

    pub stats: DashboardStats,

    // ── Dashboard enhancement data ──────────────────────────────
    pub activity_data: Vec<(String, String)>,   // (date, action_type) for heatmap
    pub last_search_at: Option<String>,          // MAX(last_searched_at)
    pub last_graded_at: Option<String>,          // MAX(graded_at)
    pub session_start: std::time::Instant,       // for session timer
    pub top_companies_by_hits: Vec<(String, i64)>, // top companies by SS+S+A count

    pub db_path: PathBuf,

    // ── TUI v2/v3 additions ──────────────────────────────────────
    pub dashboard_scroll: u16,
    pub search_mode: bool,
    pub search_query: String,
    pub sort_mode: SortMode,
    pub show_grade_picker: bool,
    pub show_bulk_picker: bool,
    pub bulk_action: String, // "watching", "applied", etc.
    pub total_jobs_unfiltered: i64,

    // ── Rendered area tracking (for mouse hit-testing) ────────────
    pub list_area: Rect,
    pub detail_area: Rect,
    pub terminal_width: u16,
    pub terminal_height: u16,

    // ── Pipeline/Kanban view ─────────────────────────────────────
    pub pipeline_column: PipelineColumn,
    pub pipeline_watching: Vec<PipelineCard>,
    pub pipeline_applied: Vec<PipelineCard>,
    pub pipeline_interview: Vec<PipelineCard>,
    pub pipeline_selections: [usize; 3], // selection index per column

    // ── Activity timeline ───────────────────────────────────────
    pub activity_timeline: Vec<ActivityEntry>,
    pub activity_scroll: u16,

    // ── Quick-peek popup ────────────────────────────────────────
    pub show_quick_peek: bool,

    // ── Smart job grouping ──────────────────────────────────────
    pub group_by_company: bool,

    // ── Session welcome diff ────────────────────────────────────
    pub new_jobs_since_last: i64,
    pub new_companies_since_last: i64,
    pub new_decisions_since_last: i64,
}
