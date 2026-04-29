use rusqlite::Connection;

use super::App;
use super::state::PipelineColumn;

impl App {
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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
}
