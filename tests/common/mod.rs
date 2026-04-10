//! Shared helpers for integration tests.
//!
//! Placed under `tests/common/mod.rs` (rather than `tests/common.rs`) so that
//! `cargo test` does not treat it as a standalone test binary. Each integration
//! test file can `mod common;` to pull these helpers in.
//!
//! What lives here:
//! - `seed_companies` / `seed_job` — builders that insert realistic fixture rows
//!   into an in-memory Cernio database, so integration tests can exercise DB
//!   code against a known shape.
//! - `fixture_path` / `load_fixture` — resolve and read fixture files from
//!   `tests/fixtures/`, used by ATS parser tests.
//!
//! These helpers deliberately do not abstract the schema — tests should keep
//! their SQL visible so drift between schema and fixtures surfaces loudly.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

/// Insert a single fully-populated company row. Returns the new company id.
///
/// Defaults are chosen to satisfy every NOT NULL / CHECK constraint without
/// callers having to care about fields they aren't exercising. Override only
/// the fields that matter for the test.
pub struct CompanySeed<'a> {
    pub name: &'a str,
    pub website: &'a str,
    pub status: &'a str,
    pub grade: Option<&'a str>,
    pub discovered_at: &'a str,
}

impl<'a> Default for CompanySeed<'a> {
    fn default() -> Self {
        Self {
            name: "Seed Co",
            website: "https://seed.example",
            status: "resolved",
            grade: Some("B"),
            discovered_at: "2026-01-01",
        }
    }
}

pub fn seed_company(conn: &Connection, seed: CompanySeed<'_>) -> i64 {
    conn.execute(
        "INSERT INTO companies (
            name, website, what_they_do, discovery_source, discovered_at,
            status, why_relevant, relevance_updated_at, grade
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            seed.name,
            seed.website,
            "Fixture description",
            "test-fixture",
            seed.discovered_at,
            seed.status,
            "fixture why relevant",
            seed.discovered_at,
            seed.grade,
        ],
    )
    .expect("seed_company insert failed");

    conn.last_insert_rowid()
}

/// Insert a single job row. Returns the new job id.
pub struct JobSeed<'a> {
    pub company_id: i64,
    pub title: &'a str,
    pub url: &'a str,
    pub grade: Option<&'a str>,
    pub evaluation_status: &'a str,
    pub discovered_at: &'a str,
    pub archived_at: Option<&'a str>,
    pub raw_description: Option<&'a str>,
    pub fit_assessment: Option<&'a str>,
}

impl<'a> JobSeed<'a> {
    pub fn new(company_id: i64, title: &'a str, url: &'a str) -> Self {
        Self {
            company_id,
            title,
            url,
            grade: None,
            evaluation_status: "pending",
            discovered_at: "2026-01-01",
            archived_at: None,
            raw_description: None,
            fit_assessment: None,
        }
    }
}

pub fn seed_job(conn: &Connection, seed: JobSeed<'_>) -> i64 {
    conn.execute(
        "INSERT INTO jobs (
            company_id, title, url, grade, evaluation_status,
            discovered_at, archived_at, raw_description, fit_assessment
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            seed.company_id,
            seed.title,
            seed.url,
            seed.grade,
            seed.evaluation_status,
            seed.discovered_at,
            seed.archived_at,
            seed.raw_description,
            seed.fit_assessment,
        ],
    )
    .expect("seed_job insert failed");

    conn.last_insert_rowid()
}

/// Insert a user decision row. Used to verify "preserved by decision" logic.
pub fn seed_decision(conn: &Connection, job_id: i64, decision: &str, when: &str) {
    conn.execute(
        "INSERT INTO user_decisions (job_id, decision, decided_at) VALUES (?1, ?2, ?3)",
        params![job_id, decision, when],
    )
    .expect("seed_decision insert failed");
}

/// Path to `tests/fixtures/<rel>`. Panics if the fixture directory cannot be
/// located, which means the test is misconfigured and should fail loudly.
pub fn fixture_path(rel: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR points at the crate root during `cargo test`.
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set — are we running under cargo test?");
    Path::new(&manifest).join("tests").join("fixtures").join(rel)
}

/// Read a fixture file as a string.
pub fn load_fixture(rel: &str) -> String {
    let path = fixture_path(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}
