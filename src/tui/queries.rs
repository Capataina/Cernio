//! Thin re-export shim — the query implementations now live in
//! `src/data/queries.rs` so the web frontend can share them.
//!
//! Existing TUI call sites (`use crate::tui::queries;`) continue to work
//! via this re-export.

pub use crate::data::queries::*;
