//! Integration tests for `cernio::pipeline::clean`.
//!
//! Exercises the tiered archival ladder (SS=28d, S=21d, A=14d, B=7d,
//! C/F=3d), the 14-day archive expiry, and the "jobs with user decisions
//! are never cleaned" invariant. Dates are computed relative to `now` so
//! the tests are time-independent.

mod common;

use chrono::{Duration, Utc};
use rusqlite::params;

use cernio::config::CleanupConfig;
use cernio::pipeline::clean;
use cernio::test_support::open_in_memory_db;

use common::{seed_company, seed_decision, seed_job, CompanySeed, JobSeed};

fn days_ago(n: i64) -> String {
    (Utc::now() - Duration::days(n))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn archived_jobs_count(db: &cernio::db::Database) -> i64 {
    db.conn()
        .query_row(
            "SELECT COUNT(*) FROM jobs WHERE evaluation_status = 'archived'",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

fn jobs_count(db: &cernio::db::Database) -> i64 {
    db.conn()
        .query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))
        .unwrap()
}

fn default_config() -> CleanupConfig {
    CleanupConfig::default()
}

// ─────────────────────────────────────────────────────────────────
// Tiered archival ladder
// ─────────────────────────────────────────────────────────────────

#[test]
fn archives_f_grade_after_3_days() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    let discovered = days_ago(5); // > 3 days for F
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("F"),
            discovered_at: &discovered,
            ..JobSeed::new(cid, "Useless", "https://j.example/1")
        },
    );

    clean::run(db.conn(), &default_config(), /*dry_run*/ false, /*jobs_only*/ true);

    assert_eq!(archived_jobs_count(&db), 1);
}

#[test]
fn archives_b_grade_after_7_days() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());

    // 5 days old: NOT archived (< 7d threshold for B)
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("B"),
            discovered_at: &days_ago(5),
            ..JobSeed::new(cid, "fresh", "https://j.example/1")
        },
    );
    // 10 days old: archived (> 7d threshold for B)
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("B"),
            discovered_at: &days_ago(10),
            ..JobSeed::new(cid, "stale", "https://j.example/2")
        },
    );

    clean::run(db.conn(), &default_config(), false, true);

    // Exactly one should be archived.
    assert_eq!(archived_jobs_count(&db), 1);
    let stale_status: String = db
        .conn()
        .query_row(
            "SELECT evaluation_status FROM jobs WHERE url = ?1",
            params!["https://j.example/2"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stale_status, "archived");
}

#[test]
fn archives_a_grade_only_after_14_days() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("A"),
            discovered_at: &days_ago(10), // not yet stale for A
            ..JobSeed::new(cid, "x", "https://j.example/1")
        },
    );

    clean::run(db.conn(), &default_config(), false, true);
    assert_eq!(archived_jobs_count(&db), 0);
}

#[test]
fn tiered_ladder_respects_each_grade() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());

    // Fresh + stale for every grade. Always insert 1d younger and
    // (threshold+2)d older than the threshold.
    let tiers = [("SS", 28), ("S", 21), ("A", 14), ("B", 7), ("C", 3), ("F", 3)];
    for (grade, threshold) in &tiers {
        seed_job(
            db.conn(),
            JobSeed {
                grade: Some(grade),
                discovered_at: &days_ago(threshold - 1),
                ..JobSeed::new(
                    cid,
                    grade,
                    &format!("https://j.example/fresh/{grade}"),
                )
            },
        );
        seed_job(
            db.conn(),
            JobSeed {
                grade: Some(grade),
                discovered_at: &days_ago(threshold + 2),
                ..JobSeed::new(
                    cid,
                    grade,
                    &format!("https://j.example/stale/{grade}"),
                )
            },
        );
    }

    clean::run(db.conn(), &default_config(), false, true);

    // Exactly 6 should be archived — one per grade.
    assert_eq!(archived_jobs_count(&db), 6);

    // And each fresh one should still be active.
    for (grade, _) in &tiers {
        let status: String = db
            .conn()
            .query_row(
                "SELECT evaluation_status FROM jobs WHERE url = ?1",
                params![format!("https://j.example/fresh/{grade}")],
                |r| r.get(0),
            )
            .unwrap();
        assert_ne!(status, "archived", "grade {grade} fresh row was archived");
    }
}

// ─────────────────────────────────────────────────────────────────
// User decisions pin jobs to the active set
// ─────────────────────────────────────────────────────────────────

#[test]
fn user_decision_prevents_archival() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    let jid = seed_job(
        db.conn(),
        JobSeed {
            grade: Some("F"),
            discovered_at: &days_ago(30),
            ..JobSeed::new(cid, "pinned", "https://j.example/1")
        },
    );
    seed_decision(db.conn(), jid, "watching", &days_ago(1));

    clean::run(db.conn(), &default_config(), false, true);

    // The decision pins it — it must NOT be archived.
    assert_eq!(archived_jobs_count(&db), 0);
}

#[test]
fn user_decision_prevents_expiry_delete() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    let jid = seed_job(
        db.conn(),
        JobSeed {
            grade: Some("S"),
            evaluation_status: "archived",
            archived_at: Some(&days_ago(30)),
            discovered_at: &days_ago(40),
            ..JobSeed::new(cid, "applied", "https://j.example/1")
        },
    );
    seed_decision(db.conn(), jid, "applied", &days_ago(20));

    let before = jobs_count(&db);
    clean::run(db.conn(), &default_config(), false, true);
    let after = jobs_count(&db);
    assert_eq!(before, after, "applied job must not be deleted");
}

// ─────────────────────────────────────────────────────────────────
// Expired archived jobs
// ─────────────────────────────────────────────────────────────────

#[test]
fn archive_older_than_14_days_is_deleted() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("C"),
            evaluation_status: "archived",
            archived_at: Some(&days_ago(20)),
            discovered_at: &days_ago(30),
            ..JobSeed::new(cid, "expired", "https://j.example/1")
        },
    );
    // Fresh archive (2 days) — must be kept.
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("C"),
            evaluation_status: "archived",
            archived_at: Some(&days_ago(2)),
            discovered_at: &days_ago(5),
            ..JobSeed::new(cid, "fresh_archive", "https://j.example/2")
        },
    );

    clean::run(db.conn(), &default_config(), false, true);

    let remaining: Vec<String> = db
        .conn()
        .prepare("SELECT url FROM jobs ORDER BY url")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    assert_eq!(remaining, vec!["https://j.example/2".to_string()]);
}

// ─────────────────────────────────────────────────────────────────
// Dry-run mode
// ─────────────────────────────────────────────────────────────────

#[test]
fn dry_run_does_not_modify_state() {
    let db = open_in_memory_db();
    let cid = seed_company(db.conn(), CompanySeed::default());
    seed_job(
        db.conn(),
        JobSeed {
            grade: Some("F"),
            discovered_at: &days_ago(30),
            ..JobSeed::new(cid, "x", "https://j.example/1")
        },
    );

    clean::run(db.conn(), &default_config(), /*dry_run*/ true, true);
    assert_eq!(archived_jobs_count(&db), 0, "dry-run must not archive");
}

// ─────────────────────────────────────────────────────────────────
// Company archival path
// ─────────────────────────────────────────────────────────────────

#[test]
fn company_archival_honours_config() {
    let db = open_in_memory_db();
    let _cid_c = seed_company(
        db.conn(),
        CompanySeed {
            name: "c_co",
            website: "https://c.example",
            grade: Some("C"),
            ..Default::default()
        },
    );
    let _cid_a = seed_company(
        db.conn(),
        CompanySeed {
            name: "a_co",
            website: "https://a.example",
            grade: Some("A"),
            ..Default::default()
        },
    );

    let mut cfg = CleanupConfig::default();
    cfg.archive_company_grades = vec!["C".to_string()];

    clean::run(db.conn(), &cfg, false, /*jobs_only*/ false);

    let c_status: String = db
        .conn()
        .query_row(
            "SELECT status FROM companies WHERE website = 'https://c.example'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let a_status: String = db
        .conn()
        .query_row(
            "SELECT status FROM companies WHERE website = 'https://a.example'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(c_status, "archived");
    assert_ne!(a_status, "archived");
}

#[test]
fn jobs_only_does_not_touch_companies() {
    let db = open_in_memory_db();
    let _cid = seed_company(
        db.conn(),
        CompanySeed {
            grade: Some("C"),
            ..Default::default()
        },
    );
    let mut cfg = CleanupConfig::default();
    cfg.archive_company_grades = vec!["C".to_string()];

    clean::run(db.conn(), &cfg, false, /*jobs_only*/ true);

    let status: String = db
        .conn()
        .query_row(
            "SELECT status FROM companies WHERE website = 'https://seed.example'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_ne!(status, "archived");
}

// ─────────────────────────────────────────────────────────────────
// Empty database
// ─────────────────────────────────────────────────────────────────

#[test]
fn clean_empty_database_is_noop() {
    let db = open_in_memory_db();
    // Must not panic on an empty DB.
    clean::run(db.conn(), &default_config(), false, true);
    assert_eq!(jobs_count(&db), 0);
}
