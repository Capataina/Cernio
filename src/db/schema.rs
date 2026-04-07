use rusqlite::{Connection, Result};
use std::path::Path;

/// The central database handle for Cernio.
///
/// Wraps a SQLite connection and provides schema initialisation.
/// All structured data — companies, jobs, evaluations, user decisions —
/// lives here. Profile data stays in markdown files.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at the given path and run migrations.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // WAL mode for better concurrent read performance (TUI reads while scripts write).
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database for testing.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Run all migrations. Idempotent — safe to call on every startup.
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(MIGRATION_001)?;
        Ok(())
    }

    /// Get a reference to the underlying connection.
    /// Used by other modules that need to run queries directly.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

/// Initial schema. Creates all tables if they don't exist.
///
/// Design principles:
/// - Facts (name, website, what_they_do) are stable and rarely updated.
/// - Checkpoints (ats_slug, ats_verified_at, careers_url) need periodic verification.
/// - Judgments (why_relevant) are tied to profile state and re-evaluated when the profile shifts.
/// - Continuously changing metrics (headcount, funding, ratings) are deliberately excluded —
///   look them up live when evaluating, don't cache stale guesses.
const MIGRATION_001: &str = "
CREATE TABLE IF NOT EXISTS companies (
    id              INTEGER PRIMARY KEY,

    -- Facts (stable)
    name            TEXT NOT NULL,
    website         TEXT NOT NULL UNIQUE,
    what_they_do    TEXT NOT NULL,
    discovery_source TEXT NOT NULL,
    discovered_at   TEXT NOT NULL,

    -- Checkpoints (need periodic verification)
    status          TEXT NOT NULL DEFAULT 'potential'
                    CHECK (status IN ('potential', 'resolved', 'bespoke')),
    location        TEXT,
    sector_tags     TEXT,
    ats_provider    TEXT CHECK (ats_provider IN ('greenhouse', 'ashby', 'lever', 'workable', 'smartrecruiters', 'workday', 'eightfold', NULL)),
    ats_slug        TEXT,
    ats_extra       TEXT,
    ats_verified_at TEXT,
    careers_url     TEXT,

    -- Judgments (tied to profile state)
    why_relevant    TEXT NOT NULL,
    relevance_updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    id                  INTEGER PRIMARY KEY,
    company_id          INTEGER NOT NULL REFERENCES companies(id),

    title               TEXT NOT NULL,
    url                 TEXT NOT NULL UNIQUE,
    location            TEXT,
    remote_policy       TEXT,
    posted_date         TEXT,
    raw_description     TEXT,
    parsed_tags         TEXT,

    evaluation_status   TEXT NOT NULL DEFAULT 'pending'
                        CHECK (evaluation_status IN ('pending', 'evaluating', 'strong_fit', 'weak_fit', 'no_fit')),
    fit_assessment      TEXT,
    fit_score           REAL,

    discovered_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_decisions (
    id          INTEGER PRIMARY KEY,
    job_id      INTEGER NOT NULL REFERENCES jobs(id),
    decision    TEXT NOT NULL CHECK (decision IN ('watching', 'applied', 'rejected')),
    decided_at  TEXT NOT NULL,
    notes       TEXT
);

CREATE INDEX IF NOT EXISTS idx_companies_status ON companies(status);
CREATE INDEX IF NOT EXISTS idx_companies_ats_provider ON companies(ats_provider);
CREATE INDEX IF NOT EXISTS idx_jobs_company_id ON jobs(company_id);
CREATE INDEX IF NOT EXISTS idx_jobs_evaluation_status ON jobs(evaluation_status);
CREATE INDEX IF NOT EXISTS idx_user_decisions_job_id ON user_decisions(job_id);
";

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn schema_creates_successfully() {
        let db = Database::open_in_memory().expect("failed to create in-memory db");

        // Verify tables exist by querying sqlite_master.
        let tables: Vec<String> = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"companies".to_string()));
        assert!(tables.contains(&"jobs".to_string()));
        assert!(tables.contains(&"user_decisions".to_string()));
    }

    #[test]
    fn migrate_is_idempotent() {
        let db = Database::open_in_memory().expect("failed to create in-memory db");
        // Running migrate again should not error.
        db.migrate().expect("second migration failed");
    }

    #[test]
    fn insert_and_query_company() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["Wise", "https://wise.com", "International money transfers", "Beauhurst top 100 fintech", now, "Fintech infrastructure, Rust adoption", now],
        ).unwrap();

        let name: String = db.conn().query_row(
            "SELECT name FROM companies WHERE website = ?1",
            params!["https://wise.com"],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(name, "Wise");
    }

    #[test]
    fn company_status_constraint() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        let result = db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, status, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["Bad Co", "https://bad.com", "Testing", "test", now, "invalid_status", "test", now],
        );

        assert!(result.is_err(), "should reject invalid status");
    }

    #[test]
    fn website_uniqueness() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["Company A", "https://example.com", "Does stuff", "test", now, "relevant", now],
        ).unwrap();

        let result = db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["Company B", "https://example.com", "Also does stuff", "test", now, "relevant", now],
        );

        assert!(result.is_err(), "should reject duplicate website");
    }

    #[test]
    fn job_links_to_company() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["Wise", "https://wise.com", "Transfers", "test", now, "fintech", now],
        ).unwrap();

        let company_id: i64 = db.conn().query_row(
            "SELECT id FROM companies WHERE website = ?1",
            params!["https://wise.com"],
            |row| row.get(0),
        ).unwrap();

        db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, discovered_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![company_id, "Junior Backend Engineer", "https://wise.com/jobs/123", now],
        ).unwrap();

        let title: String = db.conn().query_row(
            "SELECT title FROM jobs WHERE company_id = ?1",
            params![company_id],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(title, "Junior Backend Engineer");
    }

    #[test]
    fn foreign_key_enforced() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        // Insert a job referencing a non-existent company.
        let result = db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, discovered_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![9999, "Ghost Job", "https://ghost.com/jobs/1", now],
        );

        assert!(result.is_err(), "should reject job with non-existent company_id");
    }
}
