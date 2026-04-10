# Testing Strategy

**Status:** Not started — no tests exist yet. Planned for next session (2026-04-11).

## What Needs Testing

### Unit Tests (`cargo test`)

The codebase has grown significantly across multiple sessions without any test coverage. Priority areas:

- **Database layer** — CRUD operations, archival/unarchive logic, slug generation, deduplication, field updates
- **Grading/scoring logic** — if any scoring calculations exist in Rust, they need deterministic tests
- **CLI argument parsing** — `cernio search`, `cernio resolve`, `cernio clean`, `cernio check`, `cernio format` all have distinct argument shapes
- **Data formatting** — HTML-to-plaintext conversion (`cernio format`), output serialisation
- **TUI state management** — view transitions, filtering, sorting, scroll behaviour (where testable without a terminal)

### Integration Tests (`tests/` directory)

- **End-to-end CLI commands** against a test SQLite database (not the real one)
- **Search + resolve pipeline** — mock or fixture-based tests that verify the full flow from search to DB insertion
- **Database migrations** — ensure schema changes apply cleanly to a fresh DB
- **Format idempotency** — running `cernio format` twice should produce identical output

## Approach

- Use a separate test database (`:memory:` or a temp file) — never touch the real DB in tests
- Fixture data for companies/jobs so tests are deterministic
- `#[cfg(test)]` modules in each source file for unit tests
- `tests/` directory at crate root for integration tests
- Run with `cargo test` (unit + integration) as the standard verification command
