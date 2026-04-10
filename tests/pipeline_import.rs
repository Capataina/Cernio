//! Integration tests for `cernio::pipeline::import`.
//!
//! Writes a `potential.md` fixture to a tempfile, runs `import::run`
//! against an in-memory DB, and verifies parsing, deduplication,
//! validation, and dry-run semantics.

mod common;

use std::io::Write;

use rusqlite::params;
use tempfile::NamedTempFile;

use cernio::pipeline::import;
use cernio::test_support::open_in_memory_db;

use common::{seed_company, CompanySeed};

fn write_md(content: &str) -> NamedTempFile {
    let mut tmp = NamedTempFile::new().expect("tempfile");
    tmp.write_all(content.as_bytes()).expect("write");
    tmp.flush().expect("flush");
    tmp
}

fn companies_count(db: &cernio::db::Database) -> i64 {
    db.conn()
        .query_row("SELECT COUNT(*) FROM companies", [], |r| r.get(0))
        .unwrap()
}

// ─────────────────────────────────────────────────────────────────
// Happy paths
// ─────────────────────────────────────────────────────────────────

#[test]
fn imports_single_company() {
    let md = r#"# Potential Companies

## Fintech Infrastructure

### Acme Payments
- **Website**: https://acme-payments.example
- **What they do**: Cards infrastructure.
- **Why relevant**: Rust stack.
- **Source**: discovery agent
"#;
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), /*dry_run*/ false);

    assert_eq!(companies_count(&db), 1);
    let name: String = db
        .conn()
        .query_row(
            "SELECT name FROM companies WHERE website = ?1",
            params!["https://acme-payments.example"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "Acme Payments");
}

#[test]
fn imports_multiple_companies_across_sectors() {
    let md = r#"## Fintech

### Alpha
- **Website**: https://alpha.example
- **What they do**: a
- **Why relevant**: a
- **Source**: s

### Beta
- **Website**: https://beta.example
- **What they do**: b
- **Why relevant**: b
- **Source**: s

## AI Infrastructure

### Gamma
- **Website**: https://gamma.example
- **What they do**: c
- **Why relevant**: c
- **Source**: s
"#;
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), false);
    assert_eq!(companies_count(&db), 3);
}

#[test]
fn accepts_both_field_punctuation_variants() {
    // "**Field**: value" and "**Field:** value" should both work.
    let md = r#"## Sector

### Flipped
- **Website:** https://flipped.example
- **What they do:** stuff
- **Why relevant:** alignment
- **Source:** s
"#;
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), false);
    assert_eq!(companies_count(&db), 1);
}

// ─────────────────────────────────────────────────────────────────
// Deduplication
// ─────────────────────────────────────────────────────────────────

#[test]
fn skips_already_imported_companies() {
    let db = open_in_memory_db();
    seed_company(
        db.conn(),
        CompanySeed {
            name: "Acme",
            website: "https://acme.example",
            ..Default::default()
        },
    );

    let md = r#"## Sector

### Acme
- **Website**: https://acme.example
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#;
    let tmp = write_md(md);
    import::run(db.conn(), tmp.path(), false);

    // Still exactly one company — the duplicate was skipped.
    assert_eq!(companies_count(&db), 1);
}

#[test]
fn dedup_ignores_trailing_slash() {
    let db = open_in_memory_db();
    seed_company(
        db.conn(),
        CompanySeed {
            name: "Acme",
            website: "https://acme.example",
            ..Default::default()
        },
    );

    let md = r#"## Sector

### Acme
- **Website**: https://acme.example/
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#;
    let tmp = write_md(md);
    import::run(db.conn(), tmp.path(), false);

    // Still just one — the trailing slash did not create a duplicate.
    assert_eq!(companies_count(&db), 1);
}

// ─────────────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────────────

#[test]
fn rejects_entries_missing_website() {
    let md = r#"## Sector

### No Website
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#;
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), false);
    assert_eq!(companies_count(&db), 0);
}

#[test]
fn fills_in_defaults_for_optional_fields() {
    let md = r#"## Sector

### Minimal
- **Website**: https://min.example
"#;
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), false);

    let (what, why): (String, String) = db
        .conn()
        .query_row(
            "SELECT what_they_do, why_relevant FROM companies WHERE website = ?1",
            params!["https://min.example"],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    // Both should be non-empty defaults generated by flush_company.
    assert!(!what.is_empty());
    assert!(!why.is_empty());
}

// ─────────────────────────────────────────────────────────────────
// Dry-run
// ─────────────────────────────────────────────────────────────────

#[test]
fn dry_run_does_not_insert() {
    let md = r#"## Sector

### Dry
- **Website**: https://dry.example
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#;
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), /*dry_run*/ true);
    assert_eq!(companies_count(&db), 0);
}

#[test]
fn dry_run_does_not_clear_file() {
    let md = r#"## Sector

### x
- **Website**: https://x.example
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#;
    let tmp = write_md(md);
    let path = tmp.path().to_path_buf();
    let db = open_in_memory_db();
    import::run(db.conn(), &path, /*dry_run*/ true);

    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("### x"), "dry-run must not clear the file");
}

#[test]
fn real_run_clears_file_after_successful_import() {
    let md = r#"## Sector

### Clearable
- **Website**: https://clear.example
- **What they do**: x
- **Why relevant**: x
- **Source**: x
"#;
    let tmp = write_md(md);
    let path = tmp.path().to_path_buf();
    let db = open_in_memory_db();
    import::run(db.conn(), &path, false);

    // Either the file is shortened to the header-only placeholder, or at
    // minimum the company section no longer appears.
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(!after.contains("### Clearable"));
    assert!(after.contains("# Potential Companies"));
}

// ─────────────────────────────────────────────────────────────────
// Malformed input
// ─────────────────────────────────────────────────────────────────

#[test]
fn malformed_file_does_not_panic() {
    let md = "this is not\n## valid but\n### still_no_fields";
    let tmp = write_md(md);
    let db = open_in_memory_db();
    import::run(db.conn(), tmp.path(), false);
    // No companies inserted (no website field).
    assert_eq!(companies_count(&db), 0);
}

#[test]
fn missing_file_is_noop() {
    let db = open_in_memory_db();
    import::run(db.conn(), std::path::Path::new("/nonexistent/path.md"), false);
    assert_eq!(companies_count(&db), 0);
}
