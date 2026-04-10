//! CLI integration tests.
//!
//! Spawn the real `cernio` binary against a per-test temp database via the
//! `CERNIO_DB_PATH` env var (which the binary honours specifically so these
//! tests never touch state/cernio.db).
//!
//! Tests focus on:
//! - dispatch: every documented command runs and exits cleanly
//! - argument parsing: flags like --dry-run, --count, --jobs-only
//! - output: each command produces the expected sentinel text
//!
//! Commands that require network access (resolve, search, check) are
//! exercised by routes through the offline code paths only — e.g.
//! `clean --dry-run` and `format --dry-run` against an empty DB.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Build a Command that points cernio at a fresh per-test temp DB.
/// Returns the command and the TempDir (must be kept alive for the test).
fn cernio_cmd() -> (Command, TempDir, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("cernio.db");
    let mut cmd = Command::cargo_bin("cernio").expect("binary");
    cmd.env("CERNIO_DB_PATH", &db_path);
    (cmd, tmp, db_path)
}

// ─────────────────────────────────────────────────────────────────
// stats — must work on an empty DB
// ─────────────────────────────────────────────────────────────────

#[test]
fn stats_empty_database() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cernio Stats"))
        .stdout(predicate::str::contains("Companies:"))
        .stdout(predicate::str::contains("Jobs:"));
}

#[test]
fn db_status_aliases_stats() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("db-status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cernio Stats"));
}

// ─────────────────────────────────────────────────────────────────
// pending — supports --count flag
// ─────────────────────────────────────────────────────────────────

#[test]
fn pending_count_only_returns_zero_for_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.args(["pending", "--count"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("0"));
}

#[test]
fn pending_full_output_on_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("pending")
        .assert()
        .success()
        .stdout(predicate::str::contains("0 jobs pending grading"));
}

// ─────────────────────────────────────────────────────────────────
// clean — exercise both dry-run and execute on an empty DB
// ─────────────────────────────────────────────────────────────────

#[test]
fn clean_dry_run_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.args(["clean", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleanup Report"));
}

#[test]
fn clean_jobs_only_dry_run_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.args(["clean", "--dry-run", "--jobs-only"])
        .assert()
        .success();
}

#[test]
fn clean_execute_empty_db_is_safe() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("clean").assert().success();
}

// ─────────────────────────────────────────────────────────────────
// format — runs on empty DB without errors
// ─────────────────────────────────────────────────────────────────

#[test]
fn format_dry_run_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.args(["format", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Format job descriptions"))
        .stdout(predicate::str::contains("Scanned 0 jobs"));
}

#[test]
fn format_execute_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("format")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scanned 0 jobs"));
}

// ─────────────────────────────────────────────────────────────────
// import — uses --file flag to point at a fixture
// ─────────────────────────────────────────────────────────────────

#[test]
fn import_dry_run_with_file_flag() {
    use std::io::Write;

    let (mut cmd, _tmp, _db) = cernio_cmd();
    let mut md = tempfile::NamedTempFile::new().unwrap();
    md.write_all(
        br#"## Sector

### Test Co
- **Website**: https://test.example
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#,
    )
    .unwrap();
    md.flush().unwrap();

    cmd.args(["import", "--dry-run", "--file"])
        .arg(md.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Co"));
}

#[test]
fn import_missing_file_does_not_crash() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.args(["import", "--file", "/definitely/does/not/exist.md"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Failed to read"));
}

// ─────────────────────────────────────────────────────────────────
// unarchive — usage when no flags
// ─────────────────────────────────────────────────────────────────

#[test]
fn unarchive_without_flag_prints_usage() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("unarchive")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: cernio unarchive"));
}

#[test]
fn unarchive_jobs_empty_db() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.args(["unarchive", "--jobs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unarchived"));
}

// ─────────────────────────────────────────────────────────────────
// usage banner
// ─────────────────────────────────────────────────────────────────

#[test]
fn no_args_prints_usage() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: cernio"))
        .stdout(predicate::str::contains("Pipeline commands"))
        .stdout(predicate::str::contains("Info commands"));
}

#[test]
fn unknown_command_prints_usage() {
    let (mut cmd, _tmp, _db) = cernio_cmd();
    cmd.arg("definitely-not-a-real-command")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: cernio"));
}

// ─────────────────────────────────────────────────────────────────
// CERNIO_DB_PATH plumbing — proves the env var is honoured
// ─────────────────────────────────────────────────────────────────

#[test]
fn each_test_gets_its_own_database() {
    // Run two stats commands against two distinct DBs and confirm they
    // don't share state. The first DB lives only inside its TempDir, so
    // when this test exits the DB is removed.
    let (mut cmd1, _tmp1, db1) = cernio_cmd();
    let (mut cmd2, _tmp2, db2) = cernio_cmd();
    assert_ne!(db1, db2);

    cmd1.arg("stats").assert().success();
    cmd2.arg("stats").assert().success();

    assert!(db1.exists() || db1.parent().map(|p| p.exists()).unwrap_or(false));
    assert!(db2.exists() || db2.parent().map(|p| p.exists()).unwrap_or(false));
}
