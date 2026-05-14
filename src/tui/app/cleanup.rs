use rusqlite::Connection;

use super::App;

impl App {
    // ── Database cleanup ─────────────────────────────────────────

    pub fn run_cleanup(&mut self) {
        crate::tel!("cleanup_start");
        let cleanup_start = std::time::Instant::now();
        if let Ok(conn) = Connection::open(&self.db_path) {
            let mut archived_total: u64 = 0;
            for (grade, days) in &[("SS", 28), ("S", 21), ("A", 14), ("B", 7), ("C", 3), ("F", 0)] {
                let rows = conn.execute(
                    "UPDATE jobs SET evaluation_status = 'archived', archived_at = datetime('now')
                     WHERE grade = ?1
                     AND evaluation_status != 'archived'
                     AND datetime(discovered_at) < datetime('now', ?2)
                     AND id NOT IN (SELECT job_id FROM user_decisions)",
                    rusqlite::params![grade, format!("-{days} days")],
                ).unwrap_or(0) as u64;
                archived_total += rows;
                crate::tel!("cleanup_archive_grade", "grade": grade, "days": days, "rows": rows);
            }

            let deleted = conn.execute(
                "DELETE FROM jobs
                 WHERE evaluation_status = 'archived'
                 AND (
                     (archived_at IS NOT NULL AND datetime(archived_at) < datetime('now', '-14 days'))
                     OR (archived_at IS NULL AND datetime(discovered_at) < datetime('now', '-42 days'))
                 )
                 AND id NOT IN (SELECT job_id FROM user_decisions)",
                [],
            ).unwrap_or(0) as u64;
            crate::tel!(
                "cleanup_done",
                "archived": archived_total,
                "deleted": deleted,
                "duration_ms": cleanup_start.elapsed().as_millis() as u64,
            );
        } else {
            crate::tel!("cleanup_db_open_failed");
        }
        self.refresh();
    }
}
