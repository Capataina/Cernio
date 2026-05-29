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
    ///
    /// Exposed to integration tests so they can build fresh, isolated databases
    /// without touching the real filesystem. Runs migrations immediately.
    #[doc(hidden)]
    #[allow(dead_code)]
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
        self.migrate_006_add_application_packages()?;
        self.migrate_007_drop_fit_score_add_insufficient_evidence()?;
        self.migrate_008_lane_based_relativity()?;
        self.migrate_009_activity_events()?;
        Ok(())
    }

    /// Migration 009: Append-only activity_events log.
    ///
    /// Replaces the previous "derive activity from timestamps on living rows"
    /// model. Events are first-class rows that outlive whatever they describe:
    /// when a job is deleted, the `job.deleted` event stays in the log, and
    /// every prior event for that job stays too (subject_label/lane/grade are
    /// cached at emit-time).
    ///
    /// Sources:
    ///   - `tui` — user action in TUI
    ///   - `skill:<name>` — emitted from a skill run
    ///   - `cli:<command>` — emitted from a `cernio` subcommand (clean, search,
    ///     resolve, etc.)
    ///   - `backfill-migration-009` — synthesised at migration time
    ///   - `trigger` — emitted by the SQL trigger backstop
    ///
    /// Backfill synthesises events from existing timestamps so the log starts
    /// populated. Trigger backstops catch any mutation path that bypasses the
    /// Rust-side emit helpers.
    fn migrate_009_activity_events(&self) -> Result<()> {
        let has_table: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='activity_events'",
                [],
                |_| Ok(()),
            )
            .is_ok();

        if has_table {
            return Ok(());
        }

        self.conn.execute_batch(
            "CREATE TABLE activity_events (
                 id              INTEGER PRIMARY KEY,
                 occurred_at     TEXT NOT NULL,
                 event_type      TEXT NOT NULL,
                 subject_type    TEXT NOT NULL,
                 subject_id      INTEGER,
                 subject_label   TEXT,
                 lane            TEXT,
                 grade           TEXT,
                 detail_json     TEXT,
                 source          TEXT NOT NULL
             );
             CREATE INDEX idx_activity_events_occurred_at ON activity_events(occurred_at DESC);
             CREATE INDEX idx_activity_events_event_type ON activity_events(event_type);
             CREATE INDEX idx_activity_events_lane ON activity_events(lane);
             CREATE INDEX idx_activity_events_subject ON activity_events(subject_type, subject_id);",
        )?;

        // ── Backfill from existing timestamps ────────────────────
        //
        // Cached subject_label/lane/grade reflect the state at backfill time;
        // genuine "what happened then" data is lost (timestamps don't carry
        // pre-state diffs), but the events themselves are preserved.
        let source = "backfill-migration-009";

        // companies.discovered_at → company.added
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT discovered_at, 'company.added', 'company', id, name,
                    CASE WHEN lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(lanes, 1, INSTR(lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    grade,
                    json_object('source', COALESCE(discovery_source, '')),
                    ?1
             FROM companies WHERE discovered_at IS NOT NULL",
            rusqlite::params![source],
        )?;

        // companies.graded_at → company.graded
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT graded_at, 'company.graded', 'company', id, name,
                    CASE WHEN lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(lanes, 1, INSTR(lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    grade, NULL, ?1
             FROM companies WHERE graded_at IS NOT NULL",
            rusqlite::params![source],
        )?;

        // companies.last_searched_at → search.ran (one event per company-search)
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT last_searched_at, 'search.ran', 'company', id, name,
                    CASE WHEN lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(lanes, 1, INSTR(lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    grade, NULL, ?1
             FROM companies WHERE last_searched_at IS NOT NULL",
            rusqlite::params![source],
        )?;

        // jobs.discovered_at → job.added
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT j.discovered_at, 'job.added', 'job', j.id,
                    j.title || ' — ' || c.name,
                    CASE WHEN j.lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(j.lanes, 1, INSTR(j.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    j.grade, NULL, ?1
             FROM jobs j JOIN companies c ON c.id = j.company_id
             WHERE j.discovered_at IS NOT NULL",
            rusqlite::params![source],
        )?;

        // Synthesise job.graded events for jobs that have a grade. We don't
        // have a graded_at timestamp on jobs, so use discovered_at as a stand-in.
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT j.discovered_at, 'job.graded', 'job', j.id,
                    j.title || ' — ' || c.name,
                    CASE WHEN j.lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(j.lanes, 1, INSTR(j.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    j.grade,
                    json_object('evidence_basis', COALESCE(j.evidence_basis, 'jd')),
                    ?1
             FROM jobs j JOIN companies c ON c.id = j.company_id
             WHERE j.grade IS NOT NULL",
            rusqlite::params![source],
        )?;

        // jobs.archived_at → job.archived
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT j.archived_at, 'job.archived', 'job', j.id,
                    j.title || ' — ' || c.name,
                    CASE WHEN j.lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(j.lanes, 1, INSTR(j.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    j.grade, NULL, ?1
             FROM jobs j JOIN companies c ON c.id = j.company_id
             WHERE j.archived_at IS NOT NULL",
            rusqlite::params![source],
        )?;

        // user_decisions → decision.<kind>
        self.conn.execute(
            "INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, detail_json, source)
             SELECT ud.decided_at, 'decision.' || ud.decision, 'job', j.id,
                    j.title || ' — ' || c.name,
                    CASE WHEN j.lanes IS NULL THEN NULL
                         ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(j.lanes, 1, INSTR(j.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                    END,
                    j.grade, NULL, ?1
             FROM user_decisions ud
             JOIN jobs j ON j.id = ud.job_id
             JOIN companies c ON c.id = j.company_id",
            rusqlite::params![source],
        )?;

        // ── Trigger backstop ──
        //
        // Any mutation path that bypasses the Rust-side emit helpers produces a
        // `raw.*` event. Cached fields are populated from OLD/NEW row state.
        // The Rust helpers MUST run before the trigger fires for the same
        // logical event (they cite source = 'tui' / 'skill:*' / 'cli:*'); the
        // trigger only fires when the helper was skipped (source = 'trigger').
        self.conn.execute_batch(
            "CREATE TRIGGER trg_activity_job_insert AFTER INSERT ON jobs
             BEGIN
                INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, source)
                SELECT datetime('now'), 'raw.job.inserted', 'job', NEW.id,
                       NEW.title || ' — ' || c.name,
                       CASE WHEN NEW.lanes IS NULL THEN NULL
                            ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(NEW.lanes, 1, INSTR(NEW.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                       END,
                       NEW.grade, 'trigger'
                FROM companies c WHERE c.id = NEW.company_id
                AND NOT EXISTS (
                    SELECT 1 FROM activity_events ae
                    WHERE ae.subject_type = 'job' AND ae.subject_id = NEW.id
                      AND ae.event_type = 'job.added'
                      AND ae.occurred_at > datetime('now', '-2 seconds')
                );
             END;

             CREATE TRIGGER trg_activity_job_delete BEFORE DELETE ON jobs
             BEGIN
                INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, source)
                SELECT datetime('now'), 'raw.job.deleted', 'job', OLD.id,
                       OLD.title || ' — ' || c.name,
                       CASE WHEN OLD.lanes IS NULL THEN NULL
                            ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(OLD.lanes, 1, INSTR(OLD.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                       END,
                       OLD.grade, 'trigger'
                FROM companies c WHERE c.id = OLD.company_id
                AND NOT EXISTS (
                    SELECT 1 FROM activity_events ae
                    WHERE ae.subject_type = 'job' AND ae.subject_id = OLD.id
                      AND ae.event_type IN ('job.deleted', 'job.pruned')
                      AND ae.occurred_at > datetime('now', '-2 seconds')
                );
             END;

             CREATE TRIGGER trg_activity_company_insert AFTER INSERT ON companies
             BEGIN
                INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, source)
                VALUES (datetime('now'), 'raw.company.inserted', 'company', NEW.id, NEW.name,
                        CASE WHEN NEW.lanes IS NULL THEN NULL
                             ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(NEW.lanes, 1, INSTR(NEW.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                        END,
                        NEW.grade, 'trigger');
             END;

             CREATE TRIGGER trg_activity_company_delete BEFORE DELETE ON companies
             BEGIN
                INSERT INTO activity_events (occurred_at, event_type, subject_type, subject_id, subject_label, lane, grade, source)
                VALUES (datetime('now'), 'raw.company.deleted', 'company', OLD.id, OLD.name,
                        CASE WHEN OLD.lanes IS NULL THEN NULL
                             ELSE TRIM(REPLACE(REPLACE(REPLACE(SUBSTR(OLD.lanes, 1, INSTR(OLD.lanes||',', ',')-1), '[', ''), ']', ''), '\"', ''))
                        END,
                        OLD.grade, 'trigger');
             END;",
        )?;

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
                        grade               TEXT CHECK (grade IS NULL OR grade IN ('SS', 'S', 'A', 'B', 'C', 'F')),
                        evidence_basis      TEXT CHECK (evidence_basis IS NULL OR evidence_basis IN ('jd','semantic','insufficient')),
                        discovered_at       TEXT NOT NULL,
                        archived_at         TEXT
                    );
                    INSERT OR IGNORE INTO jobs_new
                        SELECT id, company_id, portal_id, title, url, location,
                               remote_policy, posted_date, raw_description, parsed_tags,
                               evaluation_status, fit_assessment, grade, evidence_basis,
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

    /// Migration 006: Add application_packages table.
    ///
    /// Stores pre-generated application answers for jobs. Created by the
    /// prepare-applications skill, consumed by the autofill system when
    /// the user presses 'p' in the TUI. Automatically cleaned up when a
    /// job is marked as applied.
    fn migrate_006_add_application_packages(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS application_packages (
                job_id      INTEGER PRIMARY KEY REFERENCES jobs(id),
                answers     TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    /// Migration 007: Drop fit_score; add evidence_basis column.
    ///
    /// Why fit_score goes: numeric scores forced incommensurable role-types
    /// onto a 1D ordering (graduate analyst and systems engineer at the same
    /// score were treated as equivalent — they aren't). Grade letter +
    /// structured fit_assessment carry differentiation now.
    ///
    /// Why evidence_basis arrives: grade-jobs should grade Google/Bloomberg/
    /// FAANG graduate roles even when the JD couldn't be fetched, by
    /// reasoning from company brand + role title (semantic-context grading).
    /// We need to mark which path produced the grade so the user can filter
    /// out unreliable evidence cases without losing brand-strong rows.
    ///
    /// Values written by the grade-jobs skill:
    ///   'jd'           — JD text was used (default, most rows)
    ///   'semantic'     — no JD; graded via company+title context reasoning
    ///   'insufficient' — no JD AND no usable context; grade may be NULL
    ///
    /// Critical invariant: a row can have grade='SS' AND evidence_basis='semantic'
    /// simultaneously. The TUI filter hides 'insufficient' by default but NEVER
    /// filters 'semantic' rows out — that's where the Google grad-role grades live.
    fn migrate_007_drop_fit_score_add_insufficient_evidence(&self) -> Result<()> {
        let has_fit_score: bool = self
            .conn
            .prepare("SELECT fit_score FROM jobs LIMIT 0")
            .is_ok();

        let has_evidence_basis: bool = self
            .conn
            .prepare("SELECT evidence_basis FROM jobs LIMIT 0")
            .is_ok();

        if !has_fit_score && has_evidence_basis {
            return Ok(());
        }

        // Add evidence_basis with a plain ADD COLUMN when possible.
        if !has_evidence_basis {
            self.conn.execute_batch(
                "ALTER TABLE jobs ADD COLUMN evidence_basis TEXT
                 CHECK (evidence_basis IS NULL OR evidence_basis IN ('jd','semantic','insufficient'));",
            )?;
        }

        if has_fit_score {
            // Drop fit_score via table rebuild (SQLite < 3.35 fallback).
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
                                        CHECK (evaluation_status IN (
                                            'pending', 'evaluating',
                                            'strong_fit', 'weak_fit', 'no_fit',
                                            'archived'
                                        )),
                    fit_assessment      TEXT,
                    grade               TEXT CHECK (grade IS NULL OR grade IN ('SS', 'S', 'A', 'B', 'C', 'F')),
                    discovered_at       TEXT NOT NULL,
                    archived_at         TEXT,
                    evidence_basis      TEXT CHECK (evidence_basis IS NULL OR evidence_basis IN ('jd','semantic','insufficient'))
                );
                INSERT INTO jobs_new (
                    id, company_id, portal_id, title, url, location, remote_policy,
                    posted_date, raw_description, parsed_tags, evaluation_status,
                    fit_assessment, grade, discovered_at, archived_at, evidence_basis
                )
                SELECT id, company_id, portal_id, title, url, location, remote_policy,
                       posted_date, raw_description, parsed_tags, evaluation_status,
                       fit_assessment, grade, discovered_at, archived_at, evidence_basis
                FROM jobs;
                DROP TABLE jobs;
                ALTER TABLE jobs_new RENAME TO jobs;
                CREATE INDEX IF NOT EXISTS idx_jobs_company_id ON jobs(company_id);
                CREATE INDEX IF NOT EXISTS idx_jobs_evaluation_status ON jobs(evaluation_status);
                CREATE INDEX IF NOT EXISTS idx_jobs_grade ON jobs(grade);
                CREATE INDEX IF NOT EXISTS idx_jobs_evidence_basis ON jobs(evidence_basis);
                PRAGMA foreign_keys = ON;
            ")?;
        }

        Ok(())
    }

    /// Migration 008: Lane-based relative grading refactor.
    ///
    /// Adds the columns the refactor depends on:
    ///   companies.lanes                    — JSON array of lane keys (multi-tag)
    ///   companies.pinnacle_status_per_lane — JSON map {lane → pinnacle/strong/adjacent/borderline}
    ///   companies.sponsors_uk              — 'yes' / 'no' / 'unknown'
    ///   jobs.lanes                         — JSON array of lane keys
    ///
    /// Drops application_packages entirely (prepare-applications skill is removed
    /// from the system).
    ///
    /// Does NOT delete jobs rows here — the jobs reset is an explicit one-shot
    /// activation step handled outside the migration so it's auditable. The
    /// activation pipeline (cernio-full-refactor.md §11.4 step 1) issues the
    /// DELETE before grade-companies re-runs.
    ///
    /// All ALTER statements are idempotent — re-running the migration on a
    /// post-migrated DB is a no-op.
    fn migrate_008_lane_based_relativity(&self) -> Result<()> {
        let has_companies_lanes: bool = self
            .conn
            .prepare("SELECT lanes FROM companies LIMIT 0")
            .is_ok();

        if !has_companies_lanes {
            self.conn.execute_batch(
                "ALTER TABLE companies ADD COLUMN lanes TEXT;",
            )?;
        }

        let has_pinnacle_status: bool = self
            .conn
            .prepare("SELECT pinnacle_status_per_lane FROM companies LIMIT 0")
            .is_ok();

        if !has_pinnacle_status {
            self.conn.execute_batch(
                "ALTER TABLE companies ADD COLUMN pinnacle_status_per_lane TEXT;",
            )?;
        }

        let has_sponsors_uk: bool = self
            .conn
            .prepare("SELECT sponsors_uk FROM companies LIMIT 0")
            .is_ok();

        if !has_sponsors_uk {
            self.conn.execute_batch(
                "ALTER TABLE companies ADD COLUMN sponsors_uk TEXT
                 CHECK (sponsors_uk IS NULL OR sponsors_uk IN ('yes', 'no', 'unknown'));",
            )?;
        }

        let has_jobs_lanes: bool = self
            .conn
            .prepare("SELECT lanes FROM jobs LIMIT 0")
            .is_ok();

        if !has_jobs_lanes {
            self.conn.execute_batch(
                "ALTER TABLE jobs ADD COLUMN lanes TEXT;",
            )?;
        }

        // Drop application_packages — the prepare-applications skill is being
        // removed from the system entirely.
        self.conn.execute_batch(
            "DROP TABLE IF EXISTS application_packages;",
        )?;

        // Index for lane-aware queries.
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_companies_sponsors_uk ON companies(sponsors_uk);",
        )?;

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
                        CHECK (evaluation_status IN (
                            'pending', 'evaluating',
                            'strong_fit', 'weak_fit', 'no_fit',
                            'archived'
                        )),
    fit_assessment      TEXT,
    grade               TEXT CHECK (grade IS NULL OR grade IN ('SS', 'S', 'A', 'B', 'C', 'F')),
    evidence_basis      TEXT CHECK (evidence_basis IS NULL OR evidence_basis IN ('jd','semantic','insufficient')),

    discovered_at       TEXT NOT NULL,
    archived_at         TEXT
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

    // ══════════════════════════════════════════════════════════════
    // Migration / schema surface extensions
    //
    // These tests exercise the parts of the schema that the original
    // suite did not cover: archival lifecycle, migration 004+ columns,
    // application_packages, user_decisions, job-level constraints.
    // ══════════════════════════════════════════════════════════════

    /// Helper — insert a minimal company and return its id.
    fn insert_company(db: &Database, name: &str, website: &str, status: &str) -> i64 {
        db.conn().execute(
            "INSERT INTO companies (name, website, what_they_do, discovery_source, discovered_at, status, why_relevant, relevance_updated_at)
             VALUES (?1, ?2, 'w', 't', '2026-01-01', ?3, 'r', '2026-01-01')",
            params![name, website, status],
        ).unwrap();
        db.conn().last_insert_rowid()
    }

    /// Helper — insert a minimal job and return its id.
    fn insert_job(
        db: &Database,
        company_id: i64,
        title: &str,
        url: &str,
        grade: Option<&str>,
        status: &str,
    ) -> i64 {
        db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, grade, evaluation_status, discovered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '2026-01-01')",
            params![company_id, title, url, grade, status],
        ).unwrap();
        db.conn().last_insert_rowid()
    }

    // ─────────────────────────────────────────────────────────────
    // Job-level constraints and uniqueness
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn job_url_uniqueness() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "Acme", "https://acme.example", "resolved");
        insert_job(&db, cid, "Eng 1", "https://j.example/1", None, "pending");
        let result = db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, evaluation_status, discovered_at)
             VALUES (?1, 'Eng 2', ?2, 'pending', '2026-01-01')",
            params![cid, "https://j.example/1"],
        );
        assert!(result.is_err(), "duplicate URL should be rejected");
    }

    #[test]
    fn job_grade_constraint() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "Acme", "https://acme.example", "resolved");

        // Valid grades are SS, S, A, B, C, F (note: includes SS which is
        // wider than the company grade set).
        for g in &["SS", "S", "A", "B", "C", "F"] {
            let url = format!("https://j.example/{g}");
            let result = db.conn().execute(
                "INSERT INTO jobs (company_id, title, url, grade, evaluation_status, discovered_at)
                 VALUES (?1, 'x', ?2, ?3, 'pending', '2026-01-01')",
                params![cid, url, g],
            );
            assert!(result.is_ok(), "grade {g} should be accepted");
        }

        // Invalid grade rejected.
        let result = db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, grade, evaluation_status, discovered_at)
             VALUES (?1, 'x', 'https://bad.example', 'X', 'pending', '2026-01-01')",
            params![cid],
        );
        assert!(result.is_err());
    }

    #[test]
    fn job_evaluation_status_constraint() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "Acme", "https://acme.example", "resolved");

        for s in &["pending", "evaluating", "strong_fit", "weak_fit", "no_fit", "archived"] {
            let url = format!("https://j.example/{s}");
            let result = db.conn().execute(
                "INSERT INTO jobs (company_id, title, url, evaluation_status, discovered_at)
                 VALUES (?1, 'x', ?2, ?3, '2026-01-01')",
                params![cid, url, s],
            );
            assert!(result.is_ok(), "status {s} should be accepted");
        }

        // Invalid.
        let result = db.conn().execute(
            "INSERT INTO jobs (company_id, title, url, evaluation_status, discovered_at)
             VALUES (?1, 'x', 'https://bad.example', 'wtf', '2026-01-01')",
            params![cid],
        );
        assert!(result.is_err());
    }

    // ─────────────────────────────────────────────────────────────
    // Migration 004: last_searched_at on companies
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn last_searched_at_column_exists_and_is_nullable() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");

        // Initially NULL.
        let v: Option<String> = db.conn().query_row(
            "SELECT last_searched_at FROM companies WHERE id = ?1",
            params![cid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, None);

        // Updatable.
        db.conn().execute(
            "UPDATE companies SET last_searched_at = ?1 WHERE id = ?2",
            params!["2026-04-01", cid],
        ).unwrap();
        let v: Option<String> = db.conn().query_row(
            "SELECT last_searched_at FROM companies WHERE id = ?1",
            params![cid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v.as_deref(), Some("2026-04-01"));
    }

    // ─────────────────────────────────────────────────────────────
    // Migration 005: archived_at on jobs
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn archived_at_column_exists_and_is_nullable() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        let jid = insert_job(&db, cid, "t", "https://j.example/1", None, "pending");

        let v: Option<String> = db.conn().query_row(
            "SELECT archived_at FROM jobs WHERE id = ?1",
            params![jid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, None);
    }

    // ─────────────────────────────────────────────────────────────
    // Migration 006: application_packages — REMOVED as part of the
    // lane-based-relativity refactor (migration_008 drops the table; the
    // prepare-applications skill is removed from the system entirely).
    // ─────────────────────────────────────────────────────────────

    // Migration 008: lane-based-relativity
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn migration_008_adds_lane_columns_to_companies() {
        let db = Database::open_in_memory().unwrap();
        // Just selecting from the columns proves they exist and are nullable.
        let _: Option<String> = db.conn()
            .query_row("SELECT lanes FROM companies LIMIT 1", [], |r| r.get(0))
            .unwrap_or(None);
        let _: Option<String> = db.conn()
            .query_row("SELECT pinnacle_status_per_lane FROM companies LIMIT 1", [], |r| r.get(0))
            .unwrap_or(None);
        let _: Option<String> = db.conn()
            .query_row("SELECT sponsors_uk FROM companies LIMIT 1", [], |r| r.get(0))
            .unwrap_or(None);
    }

    #[test]
    fn migration_008_adds_lanes_to_jobs() {
        let db = Database::open_in_memory().unwrap();
        let _: Option<String> = db.conn()
            .query_row("SELECT lanes FROM jobs LIMIT 1", [], |r| r.get(0))
            .unwrap_or(None);
    }

    #[test]
    fn migration_008_drops_application_packages() {
        let db = Database::open_in_memory().unwrap();
        let result: Result<i64, _> = db.conn()
            .query_row("SELECT COUNT(*) FROM application_packages", [], |r| r.get(0));
        assert!(result.is_err(), "application_packages should be dropped by migration_008");
    }

    #[test]
    fn migration_008_sponsors_uk_check_constraint() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");

        // Valid values
        for v in &["yes", "no", "unknown"] {
            let result = db.conn().execute(
                "UPDATE companies SET sponsors_uk = ?1 WHERE id = ?2",
                params![v, cid],
            );
            assert!(result.is_ok(), "{v} should be accepted");
        }

        // Invalid value rejected
        let result = db.conn().execute(
            "UPDATE companies SET sponsors_uk = 'maybe' WHERE id = ?1",
            params![cid],
        );
        assert!(result.is_err(), "invalid sponsors_uk value should be rejected");
    }

    // ─────────────────────────────────────────────────────────────
    // user_decisions
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn user_decisions_enum_values() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        let jid = insert_job(&db, cid, "t", "https://j.example/1", None, "pending");

        for d in &["watching", "applied", "rejected"] {
            let result = db.conn().execute(
                "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, '2026-04-01')",
                params![jid, d],
            );
            assert!(result.is_ok(), "{d} should be accepted");
        }

        let result = db.conn().execute(
            "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, 'hesitating', '2026-04-01')",
            params![jid],
        );
        assert!(result.is_err());
    }

    #[test]
    fn user_decision_fk_to_jobs() {
        let db = Database::open_in_memory().unwrap();
        let result = db.conn().execute(
            "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (9999, 'watching', '2026-04-01')",
            [],
        );
        assert!(result.is_err());
    }

    // ─────────────────────────────────────────────────────────────
    // Archival lifecycle — the clean pipeline depends on this.
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn job_can_transition_pending_to_archived() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        let jid = insert_job(&db, cid, "t", "https://j.example/1", Some("B"), "pending");

        db.conn().execute(
            "UPDATE jobs SET evaluation_status = 'archived', archived_at = ?1 WHERE id = ?2",
            params!["2026-04-01", jid],
        ).unwrap();

        let (status, archived_at): (String, Option<String>) = db.conn().query_row(
            "SELECT evaluation_status, archived_at FROM jobs WHERE id = ?1",
            params![jid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(status, "archived");
        assert_eq!(archived_at.as_deref(), Some("2026-04-01"));
    }

    #[test]
    fn archived_job_can_be_unarchived_with_timer_reset() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        let jid = insert_job(&db, cid, "t", "https://j.example/1", Some("A"), "archived");

        // Simulate the unarchive --grade A flow from main.rs.
        db.conn().execute(
            "UPDATE jobs SET evaluation_status = CASE grade
                 WHEN 'SS' THEN 'strong_fit' WHEN 'S' THEN 'strong_fit'
                 WHEN 'A' THEN 'weak_fit' WHEN 'B' THEN 'weak_fit'
                 ELSE 'no_fit' END,
             discovered_at = '2026-04-10', archived_at = NULL
             WHERE id = ?1 AND evaluation_status = 'archived'",
            params![jid],
        ).unwrap();

        let (status, archived_at, discovered_at): (String, Option<String>, String) = db.conn().query_row(
            "SELECT evaluation_status, archived_at, discovered_at FROM jobs WHERE id = ?1",
            params![jid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).unwrap();
        assert_eq!(status, "weak_fit"); // A → weak_fit per the CASE expression
        assert_eq!(archived_at, None);
        assert_eq!(discovered_at, "2026-04-10");
    }

    // ─────────────────────────────────────────────────────────────
    // Portal verification and is_primary
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn portal_verification_timestamp() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        db.conn().execute(
            "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary, verified_at)
             VALUES (?1, 'lever', 'x', 1, ?2)",
            params![cid, "2026-04-01"],
        ).unwrap();

        let verified: String = db.conn().query_row(
            "SELECT verified_at FROM company_portals WHERE company_id = ?1",
            params![cid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(verified, "2026-04-01");
    }

    #[test]
    fn portal_ats_provider_constraint() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        let result = db.conn().execute(
            "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary)
             VALUES (?1, 'bamboo', 'x', 1)",
            params![cid],
        );
        assert!(result.is_err(), "unknown ATS provider should be rejected");
    }

    #[test]
    fn portal_accepts_all_known_providers() {
        let db = Database::open_in_memory().unwrap();
        let cid = insert_company(&db, "X", "https://x.example", "resolved");
        for p in &["greenhouse", "ashby", "lever", "workable", "smartrecruiters", "workday", "eightfold"] {
            let result = db.conn().execute(
                "INSERT INTO company_portals (company_id, ats_provider, ats_slug, is_primary)
                 VALUES (?1, ?2, ?3, 1)",
                params![cid, p, format!("{p}-slug")],
            );
            assert!(result.is_ok(), "provider {p} should be accepted");
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Indexes exist (catch accidental migration breakage)
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn expected_indexes_are_created() {
        let db = Database::open_in_memory().unwrap();
        let indexes: Vec<String> = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        for expected in &[
            "idx_companies_status",
            "idx_companies_grade",
            "idx_portals_company_id",
            "idx_jobs_company_id",
            "idx_jobs_evaluation_status",
            "idx_jobs_grade",
            "idx_user_decisions_job_id",
        ] {
            assert!(
                indexes.contains(&expected.to_string()),
                "missing index {expected}, have: {indexes:?}"
            );
        }
    }
}
