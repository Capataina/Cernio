use rusqlite::Connection;

use super::App;

impl App {
    // ── Database cleanup ─────────────────────────────────────────

    pub fn run_cleanup(&mut self) {
        if let Ok(conn) = Connection::open(&self.db_path) {
            // Tiered archival: SS=28d, S=21d, A=14d, B=7d, C/F=3d.
            for (grade, days) in &[("SS", 28), ("S", 21), ("A", 14), ("B", 7), ("C", 3), ("F", 0)] {
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
