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
        crate::tel!("app_new_start", "db_path": db_path.display().to_string());
        let init_start = std::time::Instant::now();
        let conn = Connection::open(db_path)?;
        crate::tel!("app_db_opened", "duration_ms": init_start.elapsed().as_millis() as u64);

        // Auto-format any unformatted job descriptions on TUI launch.
        // This is fast (no-op when already formatted) and ensures grading
        // agents and TUI display always see clean text.
        crate::pipeline::format::run_silent(&conn);
        crate::tel!("app_format_done");

        let fetch_start = std::time::Instant::now();
        let companies = queries::fetch_companies(&conn, false);
        crate::tel!("app_fetch_companies", "count": companies.len(), "duration_ms": fetch_start.elapsed().as_millis() as u64);

        let filters = JobFilters::default();
        let fetch_start = std::time::Instant::now();
        let jobs = queries::fetch_jobs(&conn, None, &filters, SortMode::ByGrade);
        crate::tel!("app_fetch_jobs", "count": jobs.len(), "duration_ms": fetch_start.elapsed().as_millis() as u64);

        let fetch_start = std::time::Instant::now();
        let stats = queries::fetch_stats(&conn);
        crate::tel!("app_fetch_stats", "duration_ms": fetch_start.elapsed().as_millis() as u64);

        let total_jobs_unfiltered = queries::fetch_total_job_count(&conn);
        crate::tel!("app_fetch_total_jobs", "count": total_jobs_unfiltered);

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
            filters,
            show_filter_menu: false,
            filter_menu_axis: 0,
            filter_menu_chip: 0,
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
        let refresh_start = std::time::Instant::now();
        let Ok(conn) = Connection::open(&self.db_path) else {
            crate::tel!("refresh_db_open_failed", "db_path": self.db_path.display().to_string());
            return;
        };

        // Mirror the filter menu's archive setting onto company fetch so the
        // Companies view reflects the same "show archived?" semantics.
        let show_archived = self.filters.archive.contains("archived");
        self.companies = queries::fetch_companies(&conn, show_archived);
        self.jobs = queries::fetch_jobs(
            &conn,
            self.job_filter_company,
            &self.filters,
            self.sort_mode,
        );
        self.stats = queries::fetch_stats(&conn);
        self.total_jobs_unfiltered = queries::fetch_total_job_count(&conn);
        crate::tel!(
            "refresh_done",
            "duration_ms": refresh_start.elapsed().as_millis() as u64,
            "companies": self.companies.len(),
            "jobs": self.jobs.len(),
            "total_jobs_unfiltered": self.total_jobs_unfiltered,
            "view": format!("{:?}", self.view),
            "filters_active": self.filters.active_chip_count(),
        );
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

    /// Structured snapshot of every observable App state field — attached
    /// to every key-press / action telemetry event so the trace lets us
    /// reconstruct exactly what the user was looking at when something
    /// happened.
    pub fn snapshot(&self) -> serde_json::Value {
        let selected_job = self.selected_job().map(|j| (j.id, j.title.clone(), j.company_name.clone(), j.grade.clone(), j.evaluation_status.clone(), j.has_package));
        let selected_company = self.selected_company().map(|c| (c.id, c.name.clone(), c.status.clone(), c.grade.clone()));

        serde_json::json!({
            "view": format!("{:?}", self.view),
            "focus": format!("{:?}", self.focus),
            "frame_count": self.frame_count,
            "selected_job": selected_job.as_ref().map(|(id, title, company, grade, status, pkg)| serde_json::json!({
                "id": id,
                "title": title,
                "company": company,
                "grade": grade,
                "evaluation_status": status,
                "has_package": pkg,
            })),
            "selected_company": selected_company.as_ref().map(|(id, name, status, grade)| serde_json::json!({
                "id": id,
                "name": name,
                "status": status,
                "grade": grade,
            })),
            "counts": {
                "companies_visible": self.companies.len(),
                "jobs_visible": self.jobs.len(),
                "total_jobs_unfiltered": self.total_jobs_unfiltered,
                "multi_select_jobs": self.multi_select_jobs.len(),
                "multi_select_companies": self.multi_select_companies.len(),
                "toasts": self.toasts.len(),
            },
            "search": {
                "mode": self.search_mode,
                "query": &self.search_query,
            },
            "sort_mode": format!("{:?}", self.sort_mode),
            "group_by_company": self.group_by_company,
            "show_quick_peek": self.show_quick_peek,
            "show_help": self.show_help,
            "show_grade_picker": self.show_grade_picker,
            "show_bulk_picker": self.show_bulk_picker,
            "show_filter_menu": self.show_filter_menu,
            "bulk_action": &self.bulk_action,
            "job_filter_company": self.job_filter_company,
            "job_filter_company_name": self.job_filter_company_name.as_deref(),
            "filters": {
                "active_chip_count": self.filters.active_chip_count(),
                "is_default": self.filters.is_default(),
                "grades": self.filters.grades.iter().cloned().collect::<Vec<_>>(),
                "ats": self.filters.ats.iter().cloned().collect::<Vec<_>>(),
                "decisions": self.filters.decisions.iter().cloned().collect::<Vec<_>>(),
                "package": self.filters.package.iter().cloned().collect::<Vec<_>>(),
                "archive": self.filters.archive.iter().cloned().collect::<Vec<_>>(),
            },
            "filter_menu": {
                "axis": self.filter_menu_axis,
                "chip": self.filter_menu_chip,
            },
            "detail_scroll": self.detail_scroll,
            "dashboard_scroll": self.dashboard_scroll,
            "activity_scroll": self.activity_scroll,
            "pipeline_column": format!("{:?}", self.pipeline_column),
            "terminal": {
                "width": self.terminal_width,
                "height": self.terminal_height,
            },
            "session_uptime_secs": self.session_start.elapsed().as_secs(),
        })
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
