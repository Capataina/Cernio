use std::collections::HashSet;
use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::widgets::TableState;

use super::super::theme::Theme;

// Data models live in src/data/models.rs so the web frontend shares them.
// Re-exported here for backwards-compat with `crate::tui::app::CompanyRow` etc.
pub use crate::data::models::{
    ActivityEntry, CompanyRow, DashboardStats, FilterAxis, JobFilters, JobRow, SortMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Companies,
    Jobs,
    Activity,
}

impl View {
    pub fn index(self) -> usize {
        match self {
            View::Dashboard => 0,
            View::Companies => 1,
            View::Jobs => 2,
            View::Activity => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompaniesLayout {
    Classic,
    Lanes,
}

#[allow(dead_code)]
pub struct Toast {
    pub message: String,
    pub created_at: std::time::Instant,
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

    // ── Activity timeline ───────────────────────────────────────
    pub activity_timeline: Vec<ActivityEntry>,
    pub activity_scroll: u16,
    /// Group keys (per `ActivityGroup::key()`) that are currently expanded.
    pub activity_expanded: std::collections::HashSet<String>,
    /// Index of the currently-focused row in the grouped activity view
    /// (used for Enter-to-toggle).
    pub activity_cursor: usize,

    // ── Quick-peek popup ────────────────────────────────────────
    pub show_quick_peek: bool,

    // ── Smart job grouping ──────────────────────────────────────
    pub group_by_company: bool,

    // ── Companies view layout ───────────────────────────────────
    /// `Classic` is the table layout (single list, all lanes mixed).
    /// `Lanes` is the kanban-style 8-column layout (one column per lane,
    /// companies ranked top→worst within each column). Toggled with Tab
    /// when focused on the Companies list. Multi-lane companies appear in
    /// every lane column they belong to (with a `*` marker).
    pub companies_layout: CompaniesLayout,
    /// Currently-focused lane column in the Lanes layout (0..7).
    pub companies_lane_col: usize,
    /// Per-lane selection index in the Lanes layout.
    pub companies_lane_selections: [usize; 8],

    // ── Session welcome diff ────────────────────────────────────
    pub new_jobs_since_last: i64,
    pub new_companies_since_last: i64,
    pub new_decisions_since_last: i64,
}
