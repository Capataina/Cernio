//! Jobs page handler — split into submodules for clarity.
//!
//! Re-exports preserve the previous flat-module API so callers in `routes` /
//! `handlers/mod.rs` and tests can continue using `handlers::jobs::page` etc.

pub mod charts;
pub mod filters;
pub mod page;
pub mod table;

#[allow(unused_imports)]
pub use page::{decision, page, DecisionForm, JobsQuery};
