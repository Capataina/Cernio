use std::io::Write;
use std::path::Path;

use rusqlite::Connection;

use super::App;
use super::state::{JobRow, View};

impl App {
    /// Apply a decision to all selected jobs (multi or single).
    pub fn record_decision_multi(&mut self, decision: &str) {
        let ids = self.selected_job_ids();
        if ids.is_empty() { return; }

        let count = ids.len();
        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            for id in &ids {
                let _ = conn.execute(
                    "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, decision, now],
                );

                // Clean up application package when a job is marked applied.
                if decision == "applied" {
                    let _ = conn.execute(
                        "DELETE FROM application_packages WHERE job_id = ?1",
                        rusqlite::params![id],
                    );
                }
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

    // ── User decisions ───────────────────────────────────────────

    #[allow(dead_code)]
    pub fn record_decision(&mut self, decision: &str) {
        let Some(job) = self.selected_job() else {
            return;
        };
        let job_id = job.id;

        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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

    // ── URL opening and autofill ───────────────────────────────

    pub fn open_selected_url(&mut self) {
        let (url, is_job) = match self.view {
            View::Jobs => (self.selected_job().map(|j| j.url.clone()), true),
            View::Companies => (
                self.selected_company().and_then(|c| {
                    c.careers_url.clone().or(Some(c.website.clone()))
                }),
                false,
            ),
            View::Dashboard | View::Pipeline | View::Activity => (None, false),
        };
        if let Some(url) = url {
            let _ = std::process::Command::new("open").arg(&url).spawn();

            // Auto-mark as applied when opening a job URL.
            if is_job {
                self.record_decision_multi("applied");
            }
        }
    }

    /// Launch autofill for the selected job — opens Chrome with the
    /// application form pre-filled from the user's profile.
    pub fn autofill_selected_job(&mut self) {
        if self.view != View::Jobs {
            return;
        }

        let Some(job) = self.selected_job() else {
            return;
        };

        let job_url = job.url.clone();
        let job_id = job.id;
        let company_id = job.company_id;

        // Look up the ATS provider and application package from the DB.
        let (ats_provider, package_answers) = if let Ok(conn) = Connection::open(&self.db_path) {
            let provider = conn
                .query_row(
                    "SELECT cp.ats_provider FROM company_portals cp
                     WHERE cp.company_id = ?1 AND cp.is_primary = 1 LIMIT 1",
                    rusqlite::params![company_id],
                    |row| row.get::<_, String>(0),
                )
                .ok();

            let answers = conn
                .query_row(
                    "SELECT answers FROM application_packages WHERE job_id = ?1",
                    rusqlite::params![job_id],
                    |row| row.get::<_, String>(0),
                )
                .ok();

            (provider, answers)
        } else {
            (None, None)
        };

        let has_package = package_answers.is_some();

        // For unsupported providers, fall back to regular open.
        if ats_provider.as_deref() != Some("greenhouse") {
            let _ = std::process::Command::new("open").arg(&job_url).spawn();
            self.add_toast("Autofill not supported — opened in browser");
            self.record_decision_multi("applied");
            return;
        }

        // Spawn the autofill on the Tokio runtime (no stderr output — TUI is in raw mode).
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let profile = crate::autofill::ApplicantProfile::load(
                    std::path::Path::new("profile"),
                );
                let _ = crate::autofill::fill_application(
                    &job_url,
                    Some("greenhouse"),
                    &profile,
                    package_answers.as_deref(),
                )
                .await;
            });
        }

        // Mark as applied and show toast.
        let provider_name = ats_provider.as_deref().unwrap_or("unknown");
        let pkg_status = if has_package { " + answers" } else { "" };
        self.add_toast(format!("Autofilling ({provider_name}{pkg_status})..."));
        self.record_decision_multi("applied");
    }

    // ── Clipboard ───────────────────────────────────────────────

    pub fn copy_url_to_clipboard(&self) {
        let url = match self.view {
            View::Jobs => self.selected_job().map(|j| j.url.as_str()),
            View::Companies => self.selected_company().and_then(|c| {
                c.careers_url.as_deref().or(Some(c.website.as_str()))
            }),
            View::Dashboard | View::Pipeline | View::Activity => None,
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
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
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

    // ── Export ────────────────────────────────────────────────────

    pub fn export_current_view(&mut self) {
        let content = match self.view {
            View::Jobs => self.export_jobs_markdown(),
            View::Companies => self.export_companies_markdown(),
            View::Pipeline => self.export_pipeline_markdown(),
            View::Dashboard | View::Activity => self.export_jobs_markdown(), // default to jobs
        };

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let suffix = match self.view {
            View::Jobs => "jobs",
            View::Companies => "companies",
            View::Pipeline => "pipeline",
            View::Dashboard | View::Activity => "jobs",
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

    pub fn export_jobs_markdown(&self) -> String {
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

    pub fn export_companies_markdown(&self) -> String {
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

    pub fn export_pipeline_markdown(&self) -> String {
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
}
