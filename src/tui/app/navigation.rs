use ratatui::widgets::TableState;
use rusqlite::Connection;

use super::App;
use super::state::{Focus, SortMode, View};
use super::super::queries;

impl App {
    // ── Selection accessors ──────────────────────────────────────

    pub fn selected_company(&self) -> Option<&super::state::CompanyRow> {
        self.company_state
            .selected()
            .and_then(|i| self.companies.get(i))
    }

    pub fn selected_job(&self) -> Option<&super::state::JobRow> {
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

    pub fn active_list_state(&mut self) -> (&mut TableState, usize) {
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
            self.jobs = queries::fetch_jobs(&conn, self.job_filter_company, self.focused_mode, self.show_archived, self.hide_applied, self.sort_mode);
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
            self.jobs = queries::fetch_jobs(&conn, None, self.focused_mode, self.show_archived, self.hide_applied, self.sort_mode);
            self.job_state = TableState::default();
            if !self.jobs.is_empty() {
                self.job_state.select(Some(0));
            }
        }
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
}
