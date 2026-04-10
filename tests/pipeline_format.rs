//! Integration tests for `cernio::pipeline::format`.
//!
//! These run the real `format::run` / `format::run_silent` entry point
//! against an in-memory database seeded with jobs whose `raw_description`
//! contains representative ATS HTML. The assertions verify the column
//! contents *after* formatting — exercising the DB read/write path, not
//! just the pure formatter (which already has extensive unit tests).

mod common;

use common::{seed_company, seed_job, CompanySeed, JobSeed};
use rusqlite::params;

use cernio::pipeline::format;
use cernio::test_support::open_in_memory_db;

// ─────────────────────────────────────────────────────────────────
// format::run_silent end-to-end
// ─────────────────────────────────────────────────────────────────

#[test]
fn formats_html_description_and_fit_assessment() {
    let db = open_in_memory_db();
    let cid = seed_company(
        db.conn(),
        CompanySeed {
            name: "Acme",
            website: "https://acme.example",
            ..Default::default()
        },
    );
    let jid = seed_job(
        db.conn(),
        JobSeed {
            raw_description: Some("<p>Build <strong>things</strong> with Rust.</p>"),
            // clean_whitespace trims each line and collapses blank lines,
            // but does NOT collapse repeated internal spaces. Use leading
            // and trailing whitespace + blank lines to assert the trimming.
            fit_assessment: Some("   \n  strong fit — Rust stack  \n\n\n   "),
            ..JobSeed::new(cid, "Junior Rust Engineer", "https://j.example/1")
        },
    );

    format::run_silent(db.conn());

    let desc: String = db
        .conn()
        .query_row(
            "SELECT raw_description FROM jobs WHERE id = ?1",
            params![jid],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!desc.contains('<'));
    assert!(!desc.contains('>'));
    assert!(desc.contains("Build things with Rust."));

    let assess: String = db
        .conn()
        .query_row(
            "SELECT fit_assessment FROM jobs WHERE id = ?1",
            params![jid],
            |r| r.get(0),
        )
        .unwrap();
    // Leading/trailing whitespace and blank lines gone. Internal repeated
    // spaces are preserved — this is clean_output's contract.
    assert_eq!(assess, "strong fit — Rust stack");
}

#[test]
fn format_is_idempotent_across_rows() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());

    let html = "<h2>About</h2><p>Build things.</p><ul><li>Rust</li><li>Linux</li></ul>";
    for i in 0..5 {
        seed_job(
            db.conn(),
            JobSeed {
                raw_description: Some(html),
                ..JobSeed::new(cid, &format!("Role {i}"), &format!("https://j.example/{i}"))
            },
        );
    }

    // First pass converts HTML → plaintext.
    format::run_silent(db.conn());
    let first: Vec<String> = collect_descriptions(&db);

    // Second pass must not alter anything.
    format::run_silent(db.conn());
    let second: Vec<String> = collect_descriptions(&db);

    assert_eq!(first, second, "format should be idempotent");
    // And the content must actually be formatted.
    for d in &first {
        assert!(d.contains("ABOUT"));
        assert!(d.contains("• Rust"));
        assert!(d.contains("• Linux"));
        assert!(!d.contains('<'));
    }
}

#[test]
fn format_skips_null_descriptions() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    seed_job(
        db.conn(),
        JobSeed::new(cid, "No desc", "https://j.example/1"),
    );

    // Must not panic on null raw_description / fit_assessment.
    format::run_silent(db.conn());

    let desc: Option<String> = db
        .conn()
        .query_row(
            "SELECT raw_description FROM jobs WHERE url = ?1",
            params!["https://j.example/1"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(desc, None);
}

#[test]
fn format_handles_entity_encoded_html() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    seed_job(
        db.conn(),
        JobSeed {
            raw_description: Some("&lt;p&gt;Build &amp; ship&lt;/p&gt;"),
            ..JobSeed::new(cid, "x", "https://j.example/1")
        },
    );

    format::run_silent(db.conn());

    let desc: String = db
        .conn()
        .query_row(
            "SELECT raw_description FROM jobs WHERE url = ?1",
            params!["https://j.example/1"],
            |r| r.get(0),
        )
        .unwrap();
    assert!(desc.contains("Build & ship"));
    assert!(!desc.contains("&lt;"));
    assert!(!desc.contains("&amp;"));
}

#[test]
fn format_also_processes_archived_jobs() {
    // The docstring on format.rs says: "Operates on ALL jobs (active and
    // archived) since the raw data should be clean everywhere."
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    seed_job(
        db.conn(),
        JobSeed {
            evaluation_status: "archived",
            archived_at: Some("2026-03-01"),
            raw_description: Some("<p>archived role</p>"),
            ..JobSeed::new(cid, "x", "https://j.example/1")
        },
    );

    format::run_silent(db.conn());

    let desc: String = db
        .conn()
        .query_row(
            "SELECT raw_description FROM jobs WHERE url = ?1",
            params!["https://j.example/1"],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!desc.contains('<'));
    assert!(desc.contains("archived role"));
}

// ─────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────

fn collect_descriptions(db: &cernio::db::Database) -> Vec<String> {
    let mut stmt = db
        .conn()
        .prepare("SELECT raw_description FROM jobs WHERE raw_description IS NOT NULL ORDER BY id")
        .unwrap();
    stmt.query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
}
