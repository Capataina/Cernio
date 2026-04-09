use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use rusqlite::Connection;

use super::queries;
use super::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Companies,
    Jobs,
    Pipeline,
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
        }
    }
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
    pub show_archived: bool,
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
}

/// A card in the Pipeline/Kanban view.
pub struct PipelineCard {
    pub job_id: i64,
    pub title: String,
    pub company: String,
    pub grade: Option<String>,
}

impl App {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;

        let companies = queries::fetch_companies(&conn, false);
        let jobs = queries::fetch_jobs(&conn, None, false, false, SortMode::ByGrade);
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

        Ok(Self {
            running: true,
            view: View::Dashboard,
            focus: Focus::List,
            theme: Theme::default(),
            show_help: false,
            detail_scroll: 0,
            focused_mode: false,
            show_archived: false,
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
            self.sort_mode,
        );
        self.stats = queries::fetch_stats(&conn);
        self.total_jobs_unfiltered = queries::fetch_total_job_count(&conn);

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
            View::Dashboard | View::Pipeline => (&mut self.company_state, 0),
        }
    }

    // ── Multi-select ──────────────────────────────────────────────

    /// Toggle an item in the multi-select set (Ctrl+click).
    pub fn toggle_multi_select(&mut self, index: usize) {
        let set = match self.view {
            View::Jobs => &mut self.multi_select_jobs,
            View::Companies => &mut self.multi_select_companies,
            _ => return,
        };
        if set.contains(&index) {
            set.remove(&index);
        } else {
            set.insert(index);
        }
        // Update anchor for future shift-clicks.
        match self.view {
            View::Jobs => self.anchor_job = Some(index),
            View::Companies => self.anchor_company = Some(index),
            _ => {}
        }
    }

    /// Select a range from the anchor to the given index (Shift+click).
    pub fn range_select(&mut self, to: usize) {
        let (anchor, set) = match self.view {
            View::Jobs => (self.anchor_job, &mut self.multi_select_jobs),
            View::Companies => (self.anchor_company, &mut self.multi_select_companies),
            _ => return,
        };
        let from = anchor.unwrap_or(0);
        let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
        for i in lo..=hi {
            set.insert(i);
        }
    }

    /// Clear multi-selection.
    pub fn clear_multi_select(&mut self) {
        self.multi_select_jobs.clear();
        self.multi_select_companies.clear();
    }

    /// Get the IDs of multi-selected jobs (or just the current selection if none multi-selected).
    pub fn selected_job_ids(&self) -> Vec<i64> {
        if self.multi_select_jobs.is_empty() {
            self.selected_job().map(|j| vec![j.id]).unwrap_or_default()
        } else {
            self.multi_select_jobs.iter()
                .filter_map(|&i| self.jobs.get(i).map(|j| j.id))
                .collect()
        }
    }

    /// Apply a decision to all selected jobs (multi or single).
    pub fn record_decision_multi(&mut self, decision: &str) {
        let ids = self.selected_job_ids();
        if ids.is_empty() { return; }

        let count = ids.len();
        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            for id in &ids {
                let _ = conn.execute(
                    "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, decision, now],
                );
            }
        }
        if count == 1 {
            let icon = match decision {
                "watching" => "👁",
                "applied" => "✓",
                "rejected" => "✗",
                "interview" => "→",
                _ => "·",
            };
            self.add_toast(format!("{icon} {decision}"));
        } else {
            self.add_toast(format!("{decision} {count} jobs"));
        }
        self.multi_select_jobs.clear();
        self.refresh();
    }

    // ── Jump-to-grade ──────────────────────────────────────────────

    /// Jump selection to the first job matching the given grade.
    #[allow(dead_code)]
    pub fn jump_to_grade(&mut self, grade: &str) {
        if self.view != View::Jobs {
            return;
        }
        if let Some(idx) = self.jobs.iter().position(|j| j.grade.as_deref() == Some(grade)) {
            self.job_state.select(Some(idx));
            self.detail_scroll = 0;
            self.add_toast(format!("Jumped to {grade}"));
        } else {
            self.add_toast(format!("No {grade} jobs"));
        }
    }

    /// Jump to the next grade section (from current selection).
    pub fn jump_next_grade_section(&mut self) {
        if self.view != View::Jobs || self.jobs.is_empty() {
            return;
        }
        let current_idx = self.job_state.selected().unwrap_or(0);
        let current_grade = self.jobs.get(current_idx)
            .and_then(|j| j.grade.as_deref())
            .unwrap_or("");

        // Find first job with a different grade after current position.
        if let Some(idx) = self.jobs.iter().enumerate().skip(current_idx + 1)
            .find(|(_, j)| j.grade.as_deref().unwrap_or("") != current_grade)
            .map(|(i, _)| i)
        {
            let new_grade = self.jobs[idx].grade.as_deref().unwrap_or("—").to_string();
            self.job_state.select(Some(idx));
            self.detail_scroll = 0;
            self.add_toast(format!("→ {new_grade}"));
        }
    }

    /// Jump to the previous grade section (from current selection).
    pub fn jump_prev_grade_section(&mut self) {
        if self.view != View::Jobs || self.jobs.is_empty() {
            return;
        }
        let current_idx = self.job_state.selected().unwrap_or(0);
        let current_grade = self.jobs.get(current_idx)
            .and_then(|j| j.grade.as_deref())
            .unwrap_or("");

        // Find the start of the current grade section.
        let section_start = self.jobs.iter().enumerate().take(current_idx)
            .rev()
            .find(|(_, j)| j.grade.as_deref().unwrap_or("") != current_grade)
            .map(|(i, _)| i + 1)
            .unwrap_or(0);

        if section_start == current_idx || section_start == 0 && current_idx == 0 {
            // Already at start of section or at top — jump to previous section.
            if current_idx > 0 {
                // Go to the start of the previous grade section.
                let prev_grade = self.jobs.get(current_idx.saturating_sub(1))
                    .and_then(|j| j.grade.as_deref())
                    .unwrap_or("");
                let prev_start = self.jobs.iter().enumerate().take(current_idx)
                    .rev()
                    .find(|(_, j)| j.grade.as_deref().unwrap_or("") != prev_grade)
                    .map(|(i, _)| i + 1)
                    .unwrap_or(0);
                let new_grade = self.jobs[prev_start].grade.as_deref().unwrap_or("—").to_string();
                self.job_state.select(Some(prev_start));
                self.detail_scroll = 0;
                self.add_toast(format!("→ {new_grade}"));
            }
        } else {
            // Jump to start of current section.
            let new_grade = self.jobs[section_start].grade.as_deref().unwrap_or("—").to_string();
            self.job_state.select(Some(section_start));
            self.detail_scroll = 0;
            self.add_toast(format!("→ {new_grade}"));
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
            self.jobs = queries::fetch_jobs(&conn, self.job_filter_company, self.focused_mode, self.show_archived, self.sort_mode);
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
            self.jobs = queries::fetch_jobs(&conn, None, self.focused_mode, self.show_archived, self.sort_mode);
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
            View::Dashboard | View::Pipeline => None,
        };
        if let Some(url) = url {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }
    }

    // ── User decisions ───────────────────────────────────────────

    #[allow(dead_code)]
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

    pub fn copy_url_to_clipboard(&self) {
        let url = match self.view {
            View::Jobs => self.selected_job().map(|j| j.url.as_str()),
            View::Companies => self.selected_company().and_then(|c| {
                c.careers_url.as_deref().or(Some(c.website.as_str()))
            }),
            View::Dashboard | View::Pipeline => None,
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

    // ── Bulk actions ──────────────────────────────────────────────

    /// Mark all visible jobs of a given grade with a decision.
    pub fn bulk_decision_by_grade(&mut self, grade: &str, decision: &str) {
        let job_ids: Vec<i64> = self.jobs.iter()
            .filter(|j| j.grade.as_deref() == Some(grade))
            .map(|j| j.id)
            .collect();

        if job_ids.is_empty() {
            self.add_toast(format!("No {grade} jobs to mark"));
            return;
        }

        let count = job_ids.len();
        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            for id in &job_ids {
                let _ = conn.execute(
                    "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, decision, now],
                );
            }
        }
        self.add_toast(format!("{decision} all {count} {grade} jobs"));
        self.refresh();
    }

    // ── Sort cycling ────────────────────────────────────────────

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

    pub fn scroll_viewport_down(&mut self, amount: u16) {
        // Clamp to a reasonable max based on top roles count.
        let max_scroll = (self.stats.top_matches.len() as u16).saturating_sub(5);
        self.dashboard_scroll = self.dashboard_scroll.saturating_add(amount).min(max_scroll);
    }

    pub fn scroll_viewport_up(&mut self, amount: u16) {
        self.dashboard_scroll = self.dashboard_scroll.saturating_sub(amount);
    }

    /// Scroll the table viewport, moving the selection along so it stays visible.
    ///
    /// Ratatui auto-clamps the offset to keep the selected row on screen.
    /// If we only change the offset, the selection can fight it. So we move
    /// both: offset shifts by `lines`, and selection is clamped into the
    /// visible window.
    pub fn scroll_table_down(&mut self, lines: usize) {
        let (state, len) = self.active_list_state();
        if len == 0 {
            return;
        }
        let offset = state.offset();
        let new_offset = (offset + lines).min(len.saturating_sub(1));
        *state.offset_mut() = new_offset;

        // If selection is now above the viewport, push it down.
        if let Some(selected) = state.selected() {
            if selected < new_offset {
                state.select(Some(new_offset));
            }
        }
        self.detail_scroll = 0;
    }

    /// Scroll the table viewport up, keeping selection within the visible window.
    pub fn scroll_table_up(&mut self, lines: usize) {
        // Estimate visible rows from list_area. Fallback to 20.
        let visible_rows = if self.list_area.height > 3 {
            (self.list_area.height - 3) as usize // border + header
        } else {
            20
        };

        let (state, len) = self.active_list_state();
        if len == 0 {
            return;
        }
        let offset = state.offset();
        let new_offset = offset.saturating_sub(lines);
        *state.offset_mut() = new_offset;

        // If selection is now below the viewport, pull it up.
        if let Some(selected) = state.selected() {
            let max_visible = new_offset + visible_rows.saturating_sub(1);
            if selected > max_visible {
                state.select(Some(max_visible.min(len.saturating_sub(1))));
            }
        }
        self.detail_scroll = 0;
    }

    // ── Pipeline / Kanban navigation ─────────────────────────────

    pub fn pipeline_col_len(&self) -> usize {
        match self.pipeline_column {
            PipelineColumn::Watching => self.pipeline_watching.len(),
            PipelineColumn::Applied => self.pipeline_applied.len(),
            PipelineColumn::Interview => self.pipeline_interview.len(),
        }
    }

    pub fn pipeline_col_index(&self) -> usize {
        match self.pipeline_column {
            PipelineColumn::Watching => 0,
            PipelineColumn::Applied => 1,
            PipelineColumn::Interview => 2,
        }
    }

    pub fn pipeline_next(&mut self) {
        let len = self.pipeline_col_len();
        if len == 0 { return; }
        let idx = self.pipeline_col_index();
        self.pipeline_selections[idx] = (self.pipeline_selections[idx] + 1).min(len - 1);
    }

    pub fn pipeline_prev(&mut self) {
        let idx = self.pipeline_col_index();
        self.pipeline_selections[idx] = self.pipeline_selections[idx].saturating_sub(1);
    }

    pub fn pipeline_col_right(&mut self) {
        self.pipeline_column = match self.pipeline_column {
            PipelineColumn::Watching => PipelineColumn::Applied,
            PipelineColumn::Applied => PipelineColumn::Interview,
            PipelineColumn::Interview => PipelineColumn::Interview,
        };
    }

    pub fn pipeline_col_left(&mut self) {
        self.pipeline_column = match self.pipeline_column {
            PipelineColumn::Watching => PipelineColumn::Watching,
            PipelineColumn::Applied => PipelineColumn::Watching,
            PipelineColumn::Interview => PipelineColumn::Applied,
        };
    }

    /// Move the selected card to the target decision column.
    pub fn pipeline_move_card(&mut self, target: &str) {
        let col_idx = self.pipeline_col_index();
        let sel = self.pipeline_selections[col_idx];

        // Check if the current column matches the target — no-op.
        let current_decision = match self.pipeline_column {
            PipelineColumn::Watching => "watching",
            PipelineColumn::Applied => "applied",
            PipelineColumn::Interview => "interview",
        };
        if current_decision == target {
            self.add_toast(format!("Already in {target}"));
            return;
        }

        let card_job_id = match self.pipeline_column {
            PipelineColumn::Watching => self.pipeline_watching.get(sel).map(|c| c.job_id),
            PipelineColumn::Applied => self.pipeline_applied.get(sel).map(|c| c.job_id),
            PipelineColumn::Interview => self.pipeline_interview.get(sel).map(|c| c.job_id),
        };
        let Some(job_id) = card_job_id else {
            self.add_toast("No card selected".to_string());
            return;
        };

        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            let _ = conn.execute(
                "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![job_id, target, now],
            );
        }
        self.add_toast(format!("→ {target}"));
        self.refresh();
        // Clamp selection on source column.
        let new_len = self.pipeline_col_len();
        if new_len == 0 {
            self.pipeline_selections[col_idx] = 0;
        } else if self.pipeline_selections[col_idx] >= new_len {
            self.pipeline_selections[col_idx] = new_len - 1;
        }
    }

    // ── Export ────────────────────────────────────────────────────

    pub fn export_current_view(&mut self) {
        let content = match self.view {
            View::Jobs => self.export_jobs_markdown(),
            View::Companies => self.export_companies_markdown(),
            View::Pipeline => self.export_pipeline_markdown(),
            View::Dashboard => self.export_jobs_markdown(), // default to jobs
        };

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let suffix = match self.view {
            View::Jobs => "jobs",
            View::Companies => "companies",
            View::Pipeline => "pipeline",
            View::Dashboard => "jobs",
        };
        let dir = Path::new("exports");
        let _ = std::fs::create_dir_all(dir);
        let filename = format!("{date}-{suffix}.md");
        let path = dir.join(&filename);

        match std::fs::write(&path, content) {
            Ok(_) => self.add_toast(format!("Exported to exports/{filename}")),
            Err(e) => self.add_toast(format!("Export failed: {e}")),
        }
    }

    fn export_jobs_markdown(&self) -> String {
        let mut out = String::from("# Job Export\n\n");
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        out.push_str(&format!("Generated: {date}\n\n"));

        let grades = ["SS", "S", "A", "B", "C", "F"];
        for grade in &grades {
            let jobs_in_grade: Vec<&JobRow> = self.jobs.iter()
                .filter(|j| j.grade.as_deref() == Some(grade))
                .collect();
            if jobs_in_grade.is_empty() { continue; }

            out.push_str(&format!("## {} ({} jobs)\n\n", grade, jobs_in_grade.len()));
            for j in &jobs_in_grade {
                let loc = j.location.as_deref().unwrap_or("—");
                out.push_str(&format!("### {}\n", j.title));
                out.push_str(&format!("- **Company:** {}\n", j.company_name));
                out.push_str(&format!("- **Location:** {loc}\n"));
                out.push_str(&format!("- **URL:** {}\n", j.url));
                if let Some(assessment) = &j.fit_assessment {
                    out.push_str(&format!("\n{assessment}\n"));
                }
                out.push('\n');
            }
        }
        out
    }

    fn export_companies_markdown(&self) -> String {
        let mut out = String::from("# Company Export\n\n");
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        out.push_str(&format!("Generated: {date}\n\n"));

        out.push_str("| Grade | Company | Status | Jobs | ATS |\n");
        out.push_str("|-------|---------|--------|------|-----|\n");
        for c in &self.companies {
            let grade = c.grade.as_deref().unwrap_or("—");
            let ats = c.ats_provider.as_deref().unwrap_or("—");
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                grade, c.name, c.status, c.job_count, ats
            ));
        }
        out
    }

    fn export_pipeline_markdown(&self) -> String {
        let mut out = String::from("# Pipeline Export\n\n");
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        out.push_str(&format!("Generated: {date}\n\n"));

        let sections = [
            ("Watching", &self.pipeline_watching),
            ("Applied", &self.pipeline_applied),
            ("Interview", &self.pipeline_interview),
        ];
        for (label, cards) in &sections {
            out.push_str(&format!("## {} ({})\n\n", label, cards.len()));
            for card in *cards {
                let g = card.grade.as_deref().unwrap_or("—");
                out.push_str(&format!("- **{g}** {} — {}\n", card.title, card.company));
            }
            out.push('\n');
        }
        out
    }

    // ── Database cleanup ─────────────────────────────────────────

    pub fn run_cleanup(&mut self) {
        if let Ok(conn) = Connection::open(&self.db_path) {
            // Tiered archival: SS=28d, S=21d, A=14d, B=7d, C/F=3d.
            for (grade, days) in &[("SS", 28), ("S", 21), ("A", 14), ("B", 7), ("C", 3), ("F", 3)] {
                let _ = conn.execute(
                    "UPDATE jobs SET evaluation_status = 'archived', archived_at = datetime('now')
                     WHERE grade = ?1
                     AND evaluation_status != 'archived'
                     AND discovered_at < datetime('now', ?2)
                     AND id NOT IN (SELECT job_id FROM user_decisions)",
                    rusqlite::params![grade, format!("-{days} days")],
                );
            }

            // Delete archived jobs after 14 days in archive.
            let _ = conn.execute(
                "DELETE FROM jobs
                 WHERE evaluation_status = 'archived'
                 AND (
                     (archived_at IS NOT NULL AND archived_at < datetime('now', '-14 days'))
                     OR (archived_at IS NULL AND discovered_at < datetime('now', '-42 days'))
                 )
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
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
