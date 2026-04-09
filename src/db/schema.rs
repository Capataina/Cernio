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
        self.migrate_002_add_archived_status()?;
        self.migrate_003_add_job_archival()?;
        self.migrate_004_add_last_searched_at()?;
        self.migrate_005_add_archived_at()?;
        Ok(())
    }

    /// Migration 002: Add 'archived' to companies status CHECK constraint.
    ///
    /// SQLite doesn't support ALTER CHECK, so we test whether 'archived' is
    /// already accepted. If not, we rebuild the table with the new constraint.
    fn migrate_002_add_archived_status(&self) -> Result<()> {
        // Test if the constraint already allows 'archived'.
        let needs_migration = self
            .conn
            .execute(
                "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, status, why_relevant, relevance_updated_at)
                 VALUES ('__migration_test__', '__migration_test__', '__test__', '__test__', '__test__', 'archived', '__test__', '__test__')",
                [],
            )
            .is_err();

        if !needs_migration {
            // Clean up the test row — the constraint already accepts 'archived'.
            self.conn.execute(
                "DELETE FROM companies WHERE website = '__migration_test__'",
                [],
            )?;
            return Ok(());
        }

        // Rebuild the companies table with the updated CHECK constraint.
        // Temporarily disable foreign keys for the table rebuild.
        self.conn.execute_batch("
            PRAGMA foreign_keys = OFF;

            DROP TABLE IF EXISTS companies_new;

            CREATE TABLE companies_new (
                id              INTEGER PRIMARY KEY,
                name            TEXT NOT NULL,
                website         TEXT NOT NULL UNIQUE,
                what_they_do    TEXT NOT NULL,
                discovery_source TEXT NOT NULL,
                discovered_at   TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'potential'
                                CHECK (status IN ('potential', 'resolved', 'bespoke', 'archived')),
                location        TEXT,
                sector_tags     TEXT,
                careers_url     TEXT,
                why_relevant    TEXT NOT NULL,
                relevance_updated_at TEXT NOT NULL,
                grade           TEXT CHECK (grade IS NULL OR grade IN ('S', 'A', 'B', 'C')),
                grade_reasoning TEXT,
                graded_at       TEXT
            );

            INSERT INTO companies_new SELECT * FROM companies;
            DROP TABLE companies;
            ALTER TABLE companies_new RENAME TO companies;

            CREATE INDEX IF NOT EXISTS idx_companies_status ON companies(status);
            CREATE INDEX IF NOT EXISTS idx_companies_grade ON companies(grade);

            PRAGMA foreign_keys = ON;
        ")?;

        Ok(())
    }

    /// Migration 003: Add 'archived' to jobs evaluation_status CHECK constraint.
    ///
    /// Allows jobs to be soft-archived instead of deleted. Archived jobs keep
    /// their URL in the DB so they aren't re-added on subsequent searches.
    fn migrate_003_add_job_archival(&self) -> Result<()> {
        // Test if the constraint already allows 'archived'.
        // We need a valid company to satisfy the FK, so use a subquery.
        let has_companies: bool = self
            .conn
            .query_row("SELECT COUNT(*) FROM companies", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|c| c > 0)
            .unwrap_or(false);

        if !has_companies {
            // Fresh DB — MIGRATION_001 will be rewritten to include 'archived' in future.
            // For now, rebuild the jobs table directly.
            let needs_migration = true;
            if needs_migration {
                self.conn.execute_batch("
                    PRAGMA foreign_keys = OFF;
                    DROP TABLE IF EXISTS jobs_new;
                    CREATE TABLE jobs_new (
                        id                  INTEGER PRIMARY KEY,
                        company_id          INTEGER NOT NULL REFERENCES companies(id),
                        portal_id           INTEGER REFERENCES company_portals(id),
                        title               TEXT NOT NULL,
                        url                 TEXT NOT NULL UNIQUE,
                        location            TEXT,
                        remote_policy       TEXT,
                        posted_date         TEXT,
                        raw_description     TEXT,
                        parsed_tags         TEXT,
                        evaluation_status   TEXT NOT NULL DEFAULT 'pending'
                                            CHECK (evaluation_status IN ('pending', 'evaluating', 'strong_fit', 'weak_fit', 'no_fit', 'archived')),
                        fit_assessment      TEXT,
                        fit_score           REAL,
                        grade               TEXT CHECK (grade IS NULL OR grade IN ('SS', 'S', 'A', 'B', 'C', 'F')),
                        discovered_at       TEXT NOT NULL,
                        archived_at         TEXT
                    );
                    INSERT OR IGNORE INTO jobs_new
                        SELECT id, company_id, portal_id, title, url, location,
                               remote_policy, posted_date, raw_description, parsed_tags,
                               evaluation_status, fit_assessment, fit_score, grade,
                               discovered_at, NULL
                        FROM jobs;
                    DROP TABLE jobs;
                    ALTER TABLE jobs_new RENAME TO jobs;
                    CREATE INDEX IF NOT EXISTS idx_jobs_company_id ON jobs(company_id);
                    CREATE INDEX IF NOT EXISTS idx_jobs_evaluation_status ON jobs(evaluation_status);
                    CREATE INDEX IF NOT EXISTS idx_jobs_grade ON jobs(grade);
                    PRAGMA foreign_keys = ON;
                ")?;
            }
            return Ok(());
        }

        // Test with a real company_id.
        let company_id: i64 = self.conn.query_row(
            "SELECT id FROM companies LIMIT 1",
            [],
            |row| row.get(0),
        )?;

        let needs_migration = self
            .conn
            .execute(
                "INSERT INTO jobs (company_id, title, url, evaluation_status, discovered_at)
                 VALUES (?1, '__migration_test__', '__migration_test__', 'archived', '__test__')",
                rusqlite::params![company_id],
            )
            .is_err();

        if !needs_migration {
            self.conn.execute(
                "DELETE FROM jobs WHERE url = '__migration_test__'",
                [],
            )?;
            return Ok(());
        }

        // Rebuild the jobs table with the updated CHECK constraint.
        self.conn.execute_batch("
            PRAGMA foreign_keys = OFF;
            DROP TABLE IF EXISTS jobs_new;
            CREATE TABLE jobs_new (
                id                  INTEGER PRIMARY KEY,
                company_id          INTEGER NOT NULL REFERENCES companies(id),
                portal_id           INTEGER REFERENCES company_portals(id),
                title               TEXT NOT NULL,
                url                 TEXT NOT NULL UNIQUE,
                location            TEXT,
                remote_policy       TEXT,
                posted_date         TEXT,
                raw_description     TEXT,
                parsed_tags         TEXT,
                evaluation_status   TEXT NOT NULL DEFAULT 'pending'
                                    CHECK (evaluation_status IN ('pending', 'evaluating', 'strong_fit', 'weak_fit', 'no_fit', 'archived')),
                fit_assessment      TEXT,
                fit_score           REAL,
                grade               TEXT CHECK (grade IS NULL OR grade IN ('SS', 'S', 'A', 'B', 'C', 'F')),
                discovered_at       TEXT NOT NULL
            );
            INSERT INTO jobs_new SELECT * FROM jobs;
            DROP TABLE jobs;
            ALTER TABLE jobs_new RENAME TO jobs;
            CREATE INDEX IF NOT EXISTS idx_jobs_company_id ON jobs(company_id);
            CREATE INDEX IF NOT EXISTS idx_jobs_evaluation_status ON jobs(evaluation_status);
            CREATE INDEX IF NOT EXISTS idx_jobs_grade ON jobs(grade);
            PRAGMA foreign_keys = ON;
        ")?;

        Ok(())
    }

    /// Migration 004: Add last_searched_at column to companies.
    ///
    /// Tracks when each company was last searched for jobs — either via
    /// automated ATS search or manual bespoke search. Enables the TUI to
    /// show which bespoke companies need searching.
    fn migrate_004_add_last_searched_at(&self) -> Result<()> {
        // Check if column already exists.
        let has_column: bool = self
            .conn
            .prepare("SELECT last_searched_at FROM companies LIMIT 0")
            .is_ok();

        if !has_column {
            self.conn.execute_batch(
                "ALTER TABLE companies ADD COLUMN last_searched_at TEXT;",
            )?;
        }

        Ok(())
    }

    /// Migration 005: Add archived_at column to jobs.
    ///
    /// Tracks when a job was archived, enabling time-based archive expiry.
    /// Archived jobs are fully deleted after 2 weeks in the archive.
    fn migrate_005_add_archived_at(&self) -> Result<()> {
        let has_column: bool = self
            .conn
            .prepare("SELECT archived_at FROM jobs LIMIT 0")
            .is_ok();

        if !has_column {
            self.conn.execute_batch(
                "ALTER TABLE jobs ADD COLUMN archived_at TEXT;",
            )?;
        }

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
    careers_url     TEXT,

    -- Judgments (tied to profile state)
    why_relevant    TEXT NOT NULL,
    relevance_updated_at TEXT NOT NULL,

    -- Company grade (from populate-db evaluation)
    grade           TEXT CHECK (grade IS NULL OR grade IN ('S', 'A', 'B', 'C')),
    grade_reasoning TEXT,
    graded_at       TEXT
);

CREATE TABLE IF NOT EXISTS company_portals (
    id              INTEGER PRIMARY KEY,
    company_id      INTEGER NOT NULL REFERENCES companies(id),
    ats_provider    TEXT NOT NULL CHECK (ats_provider IN ('greenhouse', 'ashby', 'lever', 'workable', 'smartrecruiters', 'workday', 'eightfold')),
    ats_slug        TEXT NOT NULL,
    ats_extra       TEXT,
    verified_at     TEXT,
    is_primary      INTEGER NOT NULL DEFAULT 1,
    UNIQUE(company_id, ats_provider, ats_slug)
);

CREATE TABLE IF NOT EXISTS jobs (
    id                  INTEGER PRIMARY KEY,
    company_id          INTEGER NOT NULL REFERENCES companies(id),
    portal_id           INTEGER REFERENCES company_portals(id),

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
    grade               TEXT CHECK (grade IS NULL OR grade IN ('SS', 'S', 'A', 'B', 'C', 'F')),

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
CREATE INDEX IF NOT EXISTS idx_companies_grade ON companies(grade);
CREATE INDEX IF NOT EXISTS idx_portals_company_id ON company_portals(company_id);
CREATE INDEX IF NOT EXISTS idx_jobs_company_id ON jobs(company_id);
CREATE INDEX IF NOT EXISTS idx_jobs_evaluation_status ON jobs(evaluation_status);
CREATE INDEX IF NOT EXISTS idx_jobs_grade ON jobs(grade);
CREATE INDEX IF NOT EXISTS idx_user_decisions_job_id ON user_decisions(job_id);
";


#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn schema_creates_successfully() {
        let db = Database::open_in_memory().expect("failed to create in-memory db");

        let tables: Vec<String> = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"companies".to_string()));
        assert!(tables.contains(&"company_portals".to_string()));
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

        let result = db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, discovered_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![9999, "Ghost Job", "https://ghost.com/jobs/1", now],
        );

        assert!(result.is_err(), "should reject job with non-existent company_id");
    }

    #[test]
    fn company_with_multiple_portals() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["Palantir", "https://palantir.com", "Data analytics", "test", now, "systems engineering", now],
        ).unwrap();

        let company_id: i64 = db.conn().query_row(
            "SELECT id FROM companies WHERE website = ?1",
            params!["https://palantir.com"],
            |row| row.get(0),
        ).unwrap();

        // Add two portals for the same company.
        db.conn().execute(
            "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary)
             VALUES (?1, ?2, ?3, ?4)",
            params![company_id, "lever", "palantir", 1],
        ).unwrap();

        db.conn().execute(
            "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary)
             VALUES (?1, ?2, ?3, ?4)",
            params![company_id, "greenhouse", "palantir-technologies", 0],
        ).unwrap();

        let portal_count: i64 = db.conn().query_row(
            "SELECT COUNT(*) FROM company_portals WHERE company_id = ?1",
            params![company_id],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(portal_count, 2);
    }

    #[test]
    fn company_grade_constraint() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        // Valid grade should work.
        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at, grade)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["Good Co", "https://good.com", "Good stuff", "test", now, "relevant", now, "S"],
        ).unwrap();

        // Invalid grade should fail.
        let result = db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at, grade)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params!["Bad Co", "https://bad.com", "Bad stuff", "test", now, "relevant", now, "X"],
        );

        assert!(result.is_err(), "should reject invalid company grade");
    }

    #[test]
    fn portal_uniqueness() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-07";

        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params!["Test Co", "https://test.com", "Testing", "test", now, "relevant", now],
        ).unwrap();

        let company_id: i64 = db.conn().query_row(
            "SELECT id FROM companies WHERE website = ?1",
            params!["https://test.com"],
            |row| row.get(0),
        ).unwrap();

        db.conn().execute(
            "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary)
             VALUES (?1, ?2, ?3, ?4)",
            params![company_id, "lever", "testco", 1],
        ).unwrap();

        // Duplicate portal should fail.
        let result = db.conn().execute(
            "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary)
             VALUES (?1, ?2, ?3, ?4)",
            params![company_id, "lever", "testco", 0],
        );

        assert!(result.is_err(), "should reject duplicate portal entry");
    }

    #[test]
    fn archived_status_accepted() {
        let db = Database::open_in_memory().unwrap();
        let now = "2026-04-08";

        db.conn()
            .execute(
                "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, status, why_relevant, relevance_updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    "Archived Co",
                    "https://archived.com",
                    "Was relevant",
                    "test",
                    now,
                    "archived",
                    "no longer relevant",
                    now
                ],
            )
            .unwrap();

        let status: String = db
            .conn()
            .query_row(
                "SELECT status FROM companies WHERE website = ?1",
                params!["https://archived.com"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(status, "archived");
    }
}
