//! Smoke test: verify the integration test harness and shared helpers
//! compile and work together. If this file breaks, none of the real
//! integration tests below it will run — so keep it trivially simple.

mod common;

use common::{seed_company, CompanySeed};

#[test]
fn in_memory_db_opens_and_seeds() {
    let db = cernio::test_support::open_in_memory_db();
    let id = seed_company(
        db.conn(),
        CompanySeed {
            name: "Smoke Co",
            website: "https://smoke.example",
            ..Default::default()
        },
    );
    assert!(id > 0, "seed_company should return a positive rowid");
}

#[test]
fn fixture_path_resolves() {
    let p = common::fixture_path("ats/.gitkeep");
    // We don't require the file to exist here — just that the path is
    // correctly rooted at the crate's tests/fixtures directory.
    assert!(p.ends_with("tests/fixtures/ats/.gitkeep"));
}
