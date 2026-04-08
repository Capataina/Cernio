use std::io::Write;
use std::path::{Path, PathBuf};

use ratatui::widgets::TableState;
use rusqlite::Connection;

use super::queries;
use super::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Companies,
    Jobs,
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
        }
    }
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
    pub fit_score: Option<f64>,
    pub grade: Option<String>,
    pub raw_description: Option<String>,
    pub decision: Option<String>,
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

pub struct App {
    pub running: bool,
    pub view: View,
    pub focus: Focus,
    pub theme: Theme,
    pub show_help: bool,
    pub detail_scroll: u16,
    pub focused_mode: bool,
    pub frame_count: u64,
    pub toasts: Vec<Toast>,

    pub companies: Vec<CompanyRow>,
    pub company_state: TableState,

    pub jobs: Vec<JobRow>,
    pub job_state: TableState,
    pub job_filter_company: Option<i64>,
    pub job_filter_company_name: Option<String>,

    pub stats: DashboardStats,

    pub db_path: PathBuf,

    // ── TUI v2 additions ─────────────────────────────────────────
    #[allow(dead_code)]
    pub dashboard_scroll: u16,
    #[allow(dead_code)]
    pub search_mode: bool,
    #[allow(dead_code)]
    pub search_query: String,
    #[allow(dead_code)]
    pub sort_mode: SortMode,
    #[allow(dead_code)]
    pub show_grade_picker: bool,
    #[allow(dead_code)]
    pub total_jobs_unfiltered: i64,
}

impl App {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        let companies = queries::fetch_companies(&conn);
        let jobs = queries::fetch_jobs(&conn, None, false, SortMode::ByGrade);
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

        Ok(Self {
            running: true,
            view: View::Dashboard,
            focus: Focus::List,
            theme: Theme::default(),
            show_help: false,
            detail_scroll: 0,
            focused_mode: false,
            frame_count: 0,
            toasts: Vec::new(),
            companies,
            company_state,
            jobs,
            job_state,
            job_filter_company: None,
            job_filter_company_name: None,
            stats,
            db_path: db_path.to_path_buf(),
            dashboard_scroll: 0,
            search_mode: false,
            search_query: String::new(),
            sort_mode: SortMode::ByGrade,
            show_grade_picker: false,
            total_jobs_unfiltered,
        })
    }

    pub fn refresh(&mut self) {
        let Ok(conn) = Connection::open(&self.db_path) else {
            return;
        };

        self.companies = queries::fetch_companies(&conn);
        self.jobs = queries::fetch_jobs(
            &conn,
            self.job_filter_company,
            self.focused_mode,
            self.sort_mode,
        );
        self.stats = queries::fetch_stats(&conn);
        self.total_jobs_unfiltered = queries::fetch_total_job_count(&conn);

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

    // ── Selection accessors ──────────────────────────────────────

    pub fn selected_company(&self) -> Option<&CompanyRow> {
        self.company_state
            .selected()
            .and_then(|i| self.companies.get(i))
    }

    pub fn selected_job(&self) -> Option<&JobRow> {
        self.job_state.selected().and_then(|i| self.jobs.get(i))
    }

    // ── List navigation ──────────────────────────────────────────

    pub fn next_in_list(&mut self) {
        let (state, len) = self.active_list_state();
        if len == 0 {
            return;
        }
        let i = state.selected().map_or(0, |i| {
            if i >= len - 1 {
                0
            } else {
                i + 1
            }
        });
        state.select(Some(i));
        self.detail_scroll = 0;
    }

    pub fn prev_in_list(&mut self) {
        let (state, len) = self.active_list_state();
        if len == 0 {
            return;
        }
        let i = state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
        state.select(Some(i));
        self.detail_scroll = 0;
    }

    pub fn go_to_top(&mut self) {
        let (state, len) = self.active_list_state();
        if len > 0 {
            state.select(Some(0));
            self.detail_scroll = 0;
        }
    }

    pub fn go_to_bottom(&mut self) {
        let (state, len) = self.active_list_state();
        if len > 0 {
            state.select(Some(len - 1));
            self.detail_scroll = 0;
        }
    }

    fn active_list_state(&mut self) -> (&mut TableState, usize) {
        match self.view {
            View::Companies => (&mut self.company_state, self.companies.len()),
            View::Jobs => (&mut self.job_state, self.jobs.len()),
            View::Dashboard => (&mut self.company_state, 0), // no-op for dashboard
        }
    }

    // ── Detail scrolling ─────────────────────────────────────────

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    // ── Company → Jobs drill-down ────────────────────────────────

    pub fn enter_company_jobs(&mut self) {
        let Some(company) = self.selected_company() else {
            return;
        };
        let id = company.id;
        let name = company.name.clone();

        self.job_filter_company = Some(id);
        self.job_filter_company_name = Some(name);
        self.view = View::Jobs;
        self.focus = Focus::List;
        self.detail_scroll = 0;

        if let Ok(conn) = Connection::open(&self.db_path) {
            self.jobs = queries::fetch_jobs(&conn, self.job_filter_company, self.focused_mode, self.sort_mode);
            self.job_state = TableState::default();
            if !self.jobs.is_empty() {
                self.job_state.select(Some(0));
            }
        }
    }

    pub fn clear_job_filter(&mut self) {
        if self.job_filter_company.is_none() {
            return;
        }
        self.job_filter_company = None;
        self.job_filter_company_name = None;
        if let Ok(conn) = Connection::open(&self.db_path) {
            self.jobs = queries::fetch_jobs(&conn, None, self.focused_mode, self.sort_mode);
            self.job_state = TableState::default();
            if !self.jobs.is_empty() {
                self.job_state.select(Some(0));
            }
        }
    }

    // ── URL opening ──────────────────────────────────────────────

    pub fn open_selected_url(&self) {
        let url = match self.view {
            View::Jobs => self.selected_job().map(|j| j.url.as_str()),
            View::Companies => self.selected_company().and_then(|c| {
                c.careers_url
                    .as_deref()
                    .or(Some(c.website.as_str()))
            }),
            View::Dashboard => None,
        };
        if let Some(url) = url {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }
    }

    // ── User decisions ───────────────────────────────────────────

    pub fn record_decision(&mut self, decision: &str) {
        let Some(job) = self.selected_job() else {
            return;
        };
        let job_id = job.id;

        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            let _ = conn.execute(
                "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![job_id, decision, now],
            );
        }
        let icon = match decision {
            "watching" => "👁",
            "applied" => "✓",
            "rejected" => "✗",
            _ => "·",
        };
        self.add_toast(format!("{icon} Marked as {decision}"));
        self.refresh();
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

    // ── Clipboard ───────────────────────────────────────────────

    #[allow(dead_code)]
    pub fn copy_url_to_clipboard(&self) {
        let url = match self.view {
            View::Jobs => self.selected_job().map(|j| j.url.as_str()),
            View::Companies => self.selected_company().and_then(|c| {
                c.careers_url.as_deref().or(Some(c.website.as_str()))
            }),
            View::Dashboard => None,
        };
        if let Some(url) = url {
            if let Ok(mut child) = std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(ref mut stdin) = child.stdin {
                    let _ = stdin.write_all(url.as_bytes());
                }
                let _ = child.wait();
            }
        }
    }

    // ── Grade override ──────────────────────────────────────────

    #[allow(dead_code)]
    pub fn override_grade(&mut self, grade: &str) {
        let Some(job) = self.selected_job() else {
            return;
        };
        let job_id = job.id;

        if let Ok(conn) = Connection::open(&self.db_path) {
            let _ = conn.execute(
                "UPDATE jobs SET grade = ?1 WHERE id = ?2",
                rusqlite::params![grade, job_id],
            );
        }
        self.add_toast(format!("Grade → {grade}"));
        self.show_grade_picker = false;
        self.refresh();
    }

    // ── Sort cycling ────────────────────────────────────────────

    #[allow(dead_code)]
    pub fn toggle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::ByGrade => SortMode::ByCompany,
            SortMode::ByCompany => SortMode::ByDate,
            SortMode::ByDate => SortMode::ByLocation,
            SortMode::ByLocation => SortMode::ByGrade,
        };
        let label = match self.sort_mode {
            SortMode::ByGrade => "grade",
            SortMode::ByCompany => "company",
            SortMode::ByDate => "date",
            SortMode::ByLocation => "location",
        };
        self.add_toast(format!("Sort: {label}"));
        self.refresh();
    }

    // ── Viewport scrolling (independent of selection) ───────────

    #[allow(dead_code)]
    pub fn scroll_viewport_down(&mut self, amount: u16) {
        self.dashboard_scroll = self.dashboard_scroll.saturating_add(amount);
    }

    #[allow(dead_code)]
    pub fn scroll_viewport_up(&mut self, amount: u16) {
        self.dashboard_scroll = self.dashboard_scroll.saturating_sub(amount);
    }

    // ── Database cleanup ─────────────────────────────────────────

    pub fn run_cleanup(&mut self) {
        if let Ok(conn) = Connection::open(&self.db_path) {
            // Archive F and C graded jobs (preserving those with user decisions and SS/S).
            let _ = conn.execute(
                "UPDATE jobs SET evaluation_status = 'archived'
                 WHERE grade IN ('F', 'C')
                 AND grade NOT IN ('SS', 'S')
                 AND evaluation_status != 'archived'
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
                [],
            );

            // Archive stale jobs (>14 days, no decision, not SS/S).
            let _ = conn.execute(
                "UPDATE jobs SET evaluation_status = 'archived'
                 WHERE discovered_at < datetime('now', '-14 days')
                 AND grade NOT IN ('SS', 'S')
                 AND evaluation_status != 'archived'
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
                [],
            );

            // Archive C-graded companies.
            let _ = conn.execute(
                "UPDATE companies SET status = 'archived'
                 WHERE grade = 'C' AND status != 'archived'",
                [],
            );
        }
        self.refresh();
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
