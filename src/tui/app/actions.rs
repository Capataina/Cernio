use std::io::Write;
use std::path::Path;

use rusqlite::Connection;

use super::App;
use super::state::{JobRow, View};

impl App {
    /// Apply a decision to all selected jobs (multi or single).
    pub fn record_decision_multi(&mut self, decision: &str) {
        let ids = self.selected_job_ids();
        crate::tel!(
            "decision_record_start",
            "decision": decision,
            "job_ids": &ids,
            "count": ids.len(),
        );
        if ids.is_empty() {
            crate::tel!("decision_record_empty");
            return;
        }

        let count = ids.len();
        if let Ok(conn) = Connection::open(&self.db_path) {
            let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            for id in &ids {
                let insert_result = conn.execute(
                    "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, decision, now],
                );
                crate::tel!(
                    "db_user_decisions_insert",
                    "job_id": id,
                    "decision": decision,
                    "rows": insert_result.unwrap_or(0),
                );

                // NOTE: previously this branch DELETEd from application_packages
                // when a job was marked "applied". That fired on every `p` press
                // (autofill auto-marks applied) and wiped the package mid-run,
                // making any retry impossible. Removed — packages are small and
                // keeping them around is harmless. A `cernio clean --packages`
                // command can sweep stale entries if needed later.
                if decision == "applied" {
                    crate::tel!("decision_applied_no_delete", "job_id": id);
                }
            }
        } else {
            crate::tel!("decision_record_db_open_failed");
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
        crate::tel!("decision_record_done", "decision": decision, "count": count);
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
        crate::tel!(
            "open_url_attempt",
            "view": format!("{:?}", self.view),
            "url": url.as_deref(),
            "is_job": is_job,
        );
        if let Some(url) = url {
            let spawn_result = std::process::Command::new("open").arg(&url).spawn();
            match spawn_result {
                Ok(_) => crate::tel!("open_url_spawn_ok", "url": url.clone()),
                Err(e) => crate::tel!("open_url_spawn_error", "url": url.clone(), "error": e.to_string()),
            }

            // Auto-mark as applied when opening a job URL.
            if is_job {
                self.record_decision_multi("applied");
            }
        } else {
            crate::tel!("open_url_no_url");
        }
    }

    /// Launch autofill for the selected job — opens Chrome with the
    /// application form pre-filled from the user's profile.
    pub fn autofill_selected_job(&mut self) {
        crate::tel!("autofill_invoke_attempt", "view": format!("{:?}", self.view));
        if self.view != View::Jobs {
            crate::tel!("autofill_blocked_not_jobs_view", "view": format!("{:?}", self.view));
            return;
        }

        let Some(job) = self.selected_job() else {
            crate::tel!("autofill_blocked_no_selected_job");
            return;
        };

        let job_url = job.url.clone();
        let job_id = job.id;
        let company_id = job.company_id;
        let job_title = job.title.clone();
        let job_company = job.company_name.clone();
        crate::tel!(
            "autofill_target",
            "job_id": job_id,
            "company_id": company_id,
            "title": job_title.clone(),
            "company": job_company.clone(),
            "url": job_url.clone(),
        );

        // Look up the ATS provider, slug, and application package from the DB.
        let lookup_start = std::time::Instant::now();
        let (ats_provider, ats_slug, package_answers) =
            if let Ok(conn) = Connection::open(&self.db_path) {
                let portal = conn
                    .query_row(
                        "SELECT cp.ats_provider, cp.ats_slug FROM company_portals cp
                         WHERE cp.company_id = ?1 AND cp.is_primary = 1 LIMIT 1",
                        rusqlite::params![company_id],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )
                    .ok();

                // CAST(... AS TEXT) defends against rows that were ever
                // inserted via `sqlite3` shell's readfile() (or any other
                // path that stored the answers as BLOB through SQLite's
                // type-affinity coercion). row.get::<_, String> rejects
                // BLOB-typed values and would silently return None,
                // producing the "no package" gate failure observed in
                // session-1778793032.jsonl event [118] despite the row
                // being physically present.
                let answers = conn
                    .query_row(
                        "SELECT CAST(answers AS TEXT) FROM application_packages WHERE job_id = ?1",
                        rusqlite::params![job_id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok();

                let (provider, slug) = match portal {
                    Some((p, s)) => (Some(p), Some(s)),
                    None => (None, None),
                };
                (provider, slug, answers)
            } else {
                crate::tel!("autofill_db_open_failed", "db_path": self.db_path.display().to_string());
                (None, None, None)
            };
        crate::tel!(
            "autofill_db_lookup",
            "duration_ms": lookup_start.elapsed().as_millis() as u64,
            "provider": ats_provider.as_deref(),
            "slug": ats_slug.as_deref(),
            "has_package": package_answers.is_some(),
            "package_chars": package_answers.as_deref().map(|s| s.len()).unwrap_or(0),
        );

        // ── Gate: provider must support autofill ──
        // Anything not currently autofillable is inert under `p` — no browser
        // launch, no "applied" decision. Use `o` to open the URL manually.
        let provider_str = ats_provider.as_deref();
        if provider_str != Some("greenhouse") {
            let msg = match provider_str {
                None => "Autofill unavailable: no ATS portal recorded — use `o` to open".to_string(),
                Some(p) => format!("Autofill not supported for {p} — use `o` to open"),
            };
            crate::tel!(
                "autofill_gate_unsupported_provider",
                "provider": provider_str,
                "toast": msg.clone(),
            );
            self.add_toast(msg);
            return;
        }

        // ── Gate: an application package must be prepared ──
        let Some(package_answers) = package_answers else {
            crate::tel!("autofill_gate_no_package", "job_id": job_id);
            self.add_toast(
                "No application package for this job — run prepare-applications first".to_string(),
            );
            return;
        };

        // Spawn the autofill on the Tokio runtime (no stderr output — TUI is in raw mode).
        let handle_result = tokio::runtime::Handle::try_current();
        match handle_result {
            Err(e) => {
                // Without a running runtime the spawn would silently no-op. Surface this loudly.
                crate::tel!(
                    "autofill_tokio_handle_acquire_failed",
                    "error": e.to_string(),
                );
                self.add_toast("Autofill cannot start — no Tokio runtime available".to_string());
                return;
            }
            Ok(handle) => {
                crate::tel!("autofill_spawn_start", "job_id": job_id);
                handle.spawn(async move {
                    crate::tel!("autofill_task_running", "job_id": job_id);
                    let profile = crate::autofill::ApplicantProfile::load(
                        std::path::Path::new("profile"),
                    );
                    crate::tel!(
                        "autofill_profile_loaded",
                        "job_id": job_id,
                        "first_name": &profile.first_name,
                        "last_name": &profile.last_name,
                        "email": &profile.email,
                        "phone": &profile.phone,
                        "resume_path": profile.resume_path.as_deref(),
                    );
                    let result = crate::autofill::fill_application(
                        &job_url,
                        Some("greenhouse"),
                        ats_slug.as_deref(),
                        &profile,
                        Some(package_answers.as_str()),
                    )
                    .await;
                    // Telemetry-anchor the AutofillResult rather than dropping it
                    // with `let _`. Previously this was silent on failure, which
                    // is why pressing p with a broken autofill produced no
                    // visible error.
                    match &result {
                        crate::autofill::AutofillResult::Success { fields_filled } => {
                            crate::tel!(
                                "autofill_task_completed",
                                "job_id": job_id,
                                "outcome": "success",
                                "fields_filled": fields_filled,
                            );
                        }
                        crate::autofill::AutofillResult::UnsupportedProvider(msg) => {
                            crate::tel!(
                                "autofill_task_completed",
                                "job_id": job_id,
                                "outcome": "unsupported",
                                "message": msg,
                            );
                        }
                        crate::autofill::AutofillResult::BrowserError(msg) => {
                            crate::tel!(
                                "autofill_task_completed",
                                "job_id": job_id,
                                "outcome": "browser_error",
                                "message": msg,
                            );
                        }
                    }
                });
                crate::tel!("autofill_spawn_ok", "job_id": job_id);
            }
        }

        // Show toast. Note: pressing p NO LONGER auto-marks the job as
        // applied — the user hasn't submitted the form yet. Press `a` after
        // submission to record the actual application.
        self.add_toast("Autofilling Greenhouse (with answers)...".to_string());
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
