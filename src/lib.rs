//! Cernio library surface.
//!
//! This crate is shipped as both a library (`src/lib.rs`) and a binary
//! (`src/main.rs`). The binary is the `cernio` CLI; the library exposes
//! the internal modules so that integration tests under `tests/` can
//! exercise real code paths rather than only black-box the binary.
//!
//! Everything the CLI needs is re-exported here. If you add a new top-level
//! module, declare it here (not in `main.rs`) and import it from the binary.

pub mod ats;
pub mod autofill;
pub mod config;
pub mod db;
pub mod http;
pub mod pipeline;
pub mod tui;

/// Test-only helpers. Hidden from docs.
///
/// These are exposed publicly so integration tests (compiled as separate
/// crates under `tests/`) can construct fresh databases without touching
/// the real filesystem. Not intended for production use.
#[doc(hidden)]
pub mod test_support {
    use crate::db::Database;

    /// Open an in-memory database with all migrations applied.
    /// Each call returns a new, isolated database.
    pub fn open_in_memory_db() -> Database {
        Database::open_in_memory().expect("failed to open in-memory db")
    }
}
