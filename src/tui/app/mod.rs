pub mod state;
pub mod navigation;
pub mod actions;
pub mod pipeline;
pub mod cleanup;

pub use state::*;

use std::collections::HashSet;
use std::path::Path;

use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use rusqlite::Connection;

use super::queries;
use super::theme::Theme;

impl App {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        // Auto-format any unformatted job descriptions on TUI launch.
        // This is fast (no-op when already formatted) and ensures grading
        // agents and TUI display always see clean text.
        crate::pipeline::format::run_silent(&conn);

        let companies = queries::fetch_companies(&conn, false);
        let jobs = queries::fetch_jobs(&conn, None, false, false, false, SortMode::ByGrade);
        let stats = queries::fetch_stats(&conn);
        let total_jobs_unfiltered = queries::fetch_total_job_count(&conn);

        let mut company_state = TableState::default();
        if !companies.is_empty() {
            company_state.select(Some(0));
        }

        let mut job_state = TableState::default();
        if !jobs.is_empty() {
            job_state.select(Some(0));
        }

        let (pipeline_watching, pipeline_applied, pipeline_interview) =
            queries::fetch_pipeline_cards(&conn);

        let activity_data = queries::fetch_activity_data(&conn);
        let activity_timeline = queries::fetch_activity_timeline(&conn);
        let last_search_at = queries::fetch_last_search_at(&conn);
        let last_graded_at = queries::fetch_last_graded_at(&conn);
        let top_companies_by_hits = queries::fetch_top_companies_by_hits(&conn);
        let (new_jobs_since_last, new_companies_since_last, new_decisions_since_last) =
            queries::fetch_new_since_session(&conn);

        Ok(Self {
            running: true,
            view: View::Dashboard,
            focus: Focus::List,
            theme: Theme::default(),
            show_help: false,
            detail_scroll: 0,
            focused_mode: false,
            show_archived: false,
            hide_applied: false,
            frame_count: 0,
            toasts: Vec::new(),
            companies,
            company_state,
            jobs,
            job_state,
            job_filter_company: None,
            job_filter_company_name: None,
            multi_select_jobs: HashSet::new(),
            multi_select_companies: HashSet::new(),
            anchor_job: None,
            anchor_company: None,
            stats,
            activity_data,
            last_search_at,
            last_graded_at,
            session_start: std::time::Instant::now(),
            top_companies_by_hits,
            db_path: db_path.to_path_buf(),
            dashboard_scroll: 0,
            search_mode: false,
            search_query: String::new(),
            sort_mode: SortMode::ByGrade,
            show_grade_picker: false,
            show_bulk_picker: false,
            bulk_action: String::new(),
            total_jobs_unfiltered,
            list_area: Rect::default(),
            detail_area: Rect::default(),
            terminal_width: 0,
            terminal_height: 0,
            pipeline_column: PipelineColumn::Watching,
            pipeline_watching,
            pipeline_applied,
            pipeline_interview,
            pipeline_selections: [0; 3],
            activity_timeline,
            activity_scroll: 0,
            show_quick_peek: false,
            group_by_company: false,
            new_jobs_since_last,
            new_companies_since_last,
            new_decisions_since_last,
        })
    }

    pub fn refresh(&mut self) {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return;
        };

        self.companies = queries::fetch_companies(&conn, self.show_archived);
        self.jobs = queries::fetch_jobs(
            &conn,
            self.job_filter_company,
            self.focused_mode,
            self.show_archived,
            self.hide_applied,
            self.sort_mode,
        );
        self.stats = queries::fetch_stats(&conn);
        self.total_jobs_unfiltered = queries::fetch_total_job_count(&conn);
        self.activity_data = queries::fetch_activity_data(&conn);
        self.activity_timeline = queries::fetch_activity_timeline(&conn);
        self.last_search_at = queries::fetch_last_search_at(&conn);
        self.last_graded_at = queries::fetch_last_graded_at(&conn);
        self.top_companies_by_hits = queries::fetch_top_companies_by_hits(&conn);

        let (pw, pa, pi) = queries::fetch_pipeline_cards(&conn);
        self.pipeline_watching = pw;
        self.pipeline_applied = pa;
        self.pipeline_interview = pi;

        // Re-apply search filter if active.
        if self.search_mode || !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.jobs.retain(|j| {
                j.title.to_lowercase().contains(&query)
                    || j.company_name.to_lowercase().contains(&query)
                    || j.location
                        .as_deref()
                        .map_or(false, |l| l.to_lowercase().contains(&query))
            });
        }

        // Clamp selections so they don't point past the end.
        clamp_selection(&mut self.company_state, self.companies.len());
        clamp_selection(&mut self.job_state, self.jobs.len());
    }

    // ── Toast notifications ────────────────────────────────────────

    pub fn add_toast(&mut self, message: impl Into<String>) {
        self.toasts.push(Toast {
            message: message.into(),
            created_at: std::time::Instant::now(),
        });
    }

    pub fn tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        // Remove toasts older than 3 seconds.
        self.toasts
            .retain(|t| t.created_at.elapsed() < std::time::Duration::from_secs(3));
    }

    /// Spinner character for animated status indicators.
    #[allow(dead_code)]
    pub fn spinner_char(&self) -> char {
        const CHARS: [char; 4] = ['◐', '◑', '◒', '◓'];
        CHARS[(self.frame_count / 5) as usize % 4]
    }
}

fn clamp_selection(state: &mut TableState, len: usize) {
    if let Some(i) = state.selected() {
        if len == 0 {
            state.select(None);
        } else if i >= len {
            state.select(Some(len - 1));
        }
    }
}
