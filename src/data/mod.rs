//! Shared data layer for the TUI and the web frontend.
//!
//! Both interfaces import models + queries + activity grouping from here so
//! there's no duplication. UI-specific state (View, Focus, layout, themes)
//! stays in `src/tui/` and `src/web/` respectively.

pub mod activity;
pub mod analytics;
pub mod lane;
pub mod models;
pub mod queries;
