# Testing Strategy

**Status:** Implemented across 6 phases on 2026-04-10. **316 tests passing**, zero failing.

This document is the durable record of how Cernio is tested. It is the contract that future changes need to keep intact: when you add a feature, you should know which test file should grow, and why.

---

## Architectural decisions

These were made when introducing the test suite. They shape every test below.

### 1. Lib + bin split

Cernio is now built as both a library (`src/lib.rs`) and a binary (`src/main.rs`). The binary is a thin shim that calls into the library. This was done **specifically to enable integration tests** — Rust integration tests under `tests/` can only see public items from a library crate, not from a binary-only crate.

`src/main.rs` does `use cernio::{ats, config, db, pipeline, tui}` instead of declaring the modules locally. The modules themselves live in the library. Adding a new top-level module means declaring it in `src/lib.rs`.

### 2. CERNIO_DB_PATH env var

The CLI binary used to hardcode `state/cernio.db`. It now reads `CERNIO_DB_PATH` and falls back to `state/cernio.db` only if unset. This was the smallest possible change to make CLI integration tests possible. Tests set this env var to a per-test temp file.

### 3. test_support::open_in_memory_db

`Database::open_in_memory()` is exposed via `cernio::test_support` (under `#[doc(hidden)]`) so integration tests can build fresh databases without touching the filesystem. Each call returns a new isolated database with all migrations applied. This is the workhorse of the integration test suite.

### 4. Inline tests for private functions, integration tests for public flows

A guiding rule: if a function is private and pure, its tests live inline in a `#[cfg(test)] mod tests` block at the bottom of the same file. If a function is public *or* exercises real I/O (DB, filesystem, subprocess), its tests live under `tests/`.

This means `format_description`, `slug_candidates`, `decode_entities`, `convert_links`, every ATS provider's `normalise()`, and the config filters all have unit tests inline next to the code they test. The pipeline entry points (`format::run`, `clean::run`, `import::run`) and the CLI binary have tests under `tests/`.

### 5. Offline JSON fixtures over HTTP mocking

The ATS parser tests deliberately do not use mockito or any HTTP mocking layer. Instead they construct minimal JSON strings shaped like real provider responses, parse them with the provider's internal types (via `serde_json::from_str`), and call `normalise()` directly. This is faster, deterministic, and the JSON fixtures double as living documentation of what each provider's response looks like.

### 6. The TUI is tested by state, not by rendering

There are zero rendering tests. Drawing tests for ratatui apps are brittle and test ratatui more than they test Cernio. Instead, the TUI tests cover pure helpers — `distribute()` layout calculation, `clean_description`, `relative_date`, `truncate_chars` — and leave anything that needs a terminal alone.

### 7. Autofill Chrome paths are not tested

The Chrome CDP integration is impossible to test without a real browser, and is already known broken with a separate fix planned. The pure parts (`ApplicantProfile::load`, field extraction) could be tested but are low priority compared to everything else; they were skipped to keep this pass focused.

---

## What is tested

### Phase 1 — Foundation (1 commit, 2 tests)

**Files:** `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `src/db/schema.rs`, `tests/common/mod.rs`, `tests/smoke.rs`, `tests/fixtures/ats/.gitkeep`

- Added dev-dependencies: `proptest`, `tempfile`, `assert_cmd`, `predicates`
- Lib+bin split (decision 1)
- `CERNIO_DB_PATH` env var support (decision 2)
- `test_support` module (decision 3)
- Shared `tests/common/mod.rs` with `CompanySeed` / `JobSeed` / `seed_decision` builders, `fixture_path()` and `load_fixture()` helpers
- Smoke test verifying the harness compiles end-to-end

### Phase 2 — `format.rs` HTML pipeline (1 commit, 85 tests)

**File:** inline in `src/pipeline/format.rs`

The most complex pure logic in the repo. Tests are organised bottom-up by sub-function then top-down by end-to-end scenario.

| Function | Tests |
|---|---|
| `decode_entities` | 15 — every named entity in the table, decimal/hex numeric, invalid input, unknown names |
| `strip_tags` | 8 — quoted attributes with `>`, single quotes, nesting, eats-on-unmatched-`<` documented |
| `convert_headings` | 7 — h1–h6, attributes, nesting, case |
| `convert_inline_tags`, `convert_links`, `convert_lists`, `convert_blocks` | 24 — case-insensitive, attributes, Unicode, list counter resets, `<a>` vs `<abbr>` prefix hazard |
| `clean_output` | 8 — whitespace collapse, trim, `#LI-` tracking removal both case variants, leading/trailing blanks |
| `remove_*_case_insensitive` helpers | 5 — including the `<div>` vs `<divider>` prefix-match hazard |
| `format_description` end-to-end | 18 — realistic Greenhouse-style payload, br variants, entity-encoded HTML, idempotency |
| Invariants | 3 — never produces raw tags, never produces triple blank lines, never panics on malformed HTML |

The idempotency test is particularly load-bearing: `format_description(format_description(x)) == format_description(x)`. This is the property `cernio format` depends on when it runs on TUI startup.

### Phase 3 — Config and slug generation (1 commit, 61 tests)

**Files:** inline in `src/config.rs` (31), inline in `src/pipeline/resolve.rs` (30)

config.rs:
- `passes_exclusion`: case-insensitivity, substring semantics, multi-keyword
- `passes_inclusion`: empty-list passthrough, case-insensitivity, OR logic
- `passes_location`: per-provider isolation, empty-list false-negative protection, unknown-provider passthrough, country codes
- `included_grades`: every boundary, lowercase input, invalid fallback to B
- `Preferences::load_from`: missing file, empty file, invalid TOML, partial TOML, full realistic config
- Pipeline composition tests using the actual keywords from `profile/preferences.toml`

resolve.rs `slug_candidates`:
- Every transformation rule individually: space variants, punctuation stripping, all suffix strippers (Ltd/Inc/PLC/GmbH/Technologies), suffix appending, first-word, initials, parenthetical, slash handling, domain suffixes
- Invariants: no duplicates, no empty candidates, all lowercase, insertion order preserved
- Edge cases: single char, empty string, very long, Unicode
- A realistic-roster test covering 13 actually-resolved Cernio companies (XTX, Two Sigma, Jane Street, Citadel, Palantir, Stripe, Anthropic, DeepMind, Bloomberg, DRW, HRT, Jump, Hudson River) — regressions here would silently lose companies in production

Also fixed `included_grades` return lifetime to `'static` (the values are string literals; the inferred lifetime was wrongly tied to `&self`).

### Phase 4 — ATS parsers (1 commit, 72 tests)

**Files:** inline in `src/ats/{lever,greenhouse,ashby,workable,smartrecruiters,workday}.rs`

Per provider:

| Provider | Tests | Key coverage |
|---|---|---|
| Lever | 16 | base_url EU/US, build_description joins, normalise_posting timestamp → ISO date conversion, strip_html quoted-attribute hazard |
| Greenhouse | 13 | Semicolon-separated locations, Remote/Hybrid inference, office locations merged, posted_date fallback chain |
| Ashby | 8 | postalAddress decomposition, secondaryLocations, workplaceType > isRemote precedence |
| Workable | 10 | City/state/country composition, empty-string skip, telecommuting flag, locations[] array with countryCode |
| SmartRecruiters | 12 | **The critical totalFound=0 case** the probe function depends on, ref_url fallback, list-endpoint description always None |
| Workday | 13 | parse_extra valid/invalid/missing/non-string, build_base_url and build_posting_url shapes, pipe-separated locations, bullet fields → description |

### Phase 5 — DB expansion + pipeline integration (1 commit, 46 tests)

**Files:** inline in `src/db/schema.rs` (18), `tests/pipeline_format.rs` (5), `tests/pipeline_clean.rs` (11), `tests/pipeline_import.rs` (12)

DB schema extensions:
- Job URL uniqueness, all grade and evaluation_status enum values
- Migration 004 (`last_searched_at`) nullable + updatable
- Migration 005 (`archived_at`) nullable
- Migration 006 (`application_packages`): CRUD, one-per-job PK, FK to jobs
- `user_decisions` enum values and FK to jobs
- Archival lifecycle: pending → archived → unarchive with timer reset (modelling exact SQL in `cmd_unarchive`)
- Portal `verified_at`, `ats_provider` enum, all 7 known providers accepted
- All expected indexes created

Pipeline integration:
- **format**: HTML + fit_assessment formatting end-to-end, idempotency across 5 rows, null skip, entity-encoded HTML, archived jobs included
- **clean**: every tier of the SS/S/A/B/C/F ladder with fresh-just-under and stale-just-over timestamps, 6-out-of-12 tier integration test, user decisions pin jobs, 14-day expiry, dry-run no-op, company archival honours config, `--jobs-only` flag, empty DB safe
- **import**: single + multi-sector parsing, both punctuation variants, dedup with trailing-slash handling, missing website rejection, default field filling, dry-run, file-clearing semantics, malformed input safe

### Phase 6 — CLI + TUI helpers (1 commit, 34 tests)

**Files:** `tests/cli.rs` (16), inline in `src/tui/widgets/layout.rs` (8 new), inline in `src/tui/views/jobs.rs` (10 new)

CLI tests via `assert_cmd` against the real binary, with `CERNIO_DB_PATH` pointing each test at a fresh tempdir DB:
- `stats`, `db-status`, `pending`, `pending --count`
- `clean` / `clean --dry-run` / `clean --jobs-only`
- `format` / `format --dry-run`
- `import --dry-run --file <tmp>` and `import` with missing file
- `unarchive` (no flag → usage), `unarchive --jobs` (empty DB)
- No-args and unknown-command both print usage
- Meta-test asserting per-test DB isolation

TUI helper extensions:
- `distribute()`: empty specs, single spec with zero priority, mixed priority, proportional surplus, min_height floor, exact-fit edge, u16::MAX safety
- `relative_date`: today, 1 day, 3 days, 1 week, 2 weeks, ISO timestamp truncation
- `truncate_chars`: short passthrough, exact length, ellipsis on overflow, Unicode (Japanese), emoji

---

## How to run

```sh
# Everything (recommended)
cargo test

# Just unit tests inside src/
cargo test --lib

# Just one integration test file
cargo test --test pipeline_clean

# A single test by name
cargo test --lib config::tests::passes_inclusion_empty_list_is_passthrough
```

The full suite runs in well under a second once compiled. CLI tests are the slowest because each one spawns a subprocess.

---

## What is *not* tested (and why)

| Area | Why skipped |
|---|---|
| TUI rendering | Tests would couple to ratatui internals and break on every layout change. Pure helpers covered instead. |
| Live HTTP to ATS providers | Network-dependent tests are flaky and provider-banned. Offline JSON fixtures cover the parsing logic. |
| `cernio resolve` and `cernio search` end-to-end | Both make real HTTP calls. The slug generation, filter logic, and ATS parsers they compose are all tested in isolation. |
| `cernio check` | Same — makes real HTTP calls to verify slugs. |
| Autofill Chrome integration | Requires a real browser. The pure parts of `autofill::` are also untested in this pass; they're a follow-up. |
| Profile loading from `profile/*.md` | Could be added; was deprioritised since the runtime contract is "read fresh every session" rather than parse-once. |

---

## When to add a new test

| Change | Where the test goes |
|---|---|
| New private pure helper in `src/foo.rs` | Inline in `src/foo.rs` under `mod tests` |
| New ATS provider | New `src/ats/{provider}.rs` with inline `mod tests`, modelled on the existing 6 |
| New DB column or migration | New test in `src/db/schema.rs::tests` asserting the column exists, has the right type, and is nullable/non-nullable as designed |
| New pipeline subcommand | New `tests/pipeline_{name}.rs` exercising the public entry against `open_in_memory_db()` |
| New CLI command | New `#[test]` in `tests/cli.rs` using `assert_cmd::Command::cargo_bin("cernio")` with `CERNIO_DB_PATH` set |
| New filter or grading rule in `config.rs` | Inline in `src/config.rs::tests` with both unit-level and pipeline-composition tests |

---

## Test count by area

| Area | Count |
|---|---|
| `src/db/schema.rs` (inline) | 29 |
| `src/config.rs` (inline) | 31 |
| `src/pipeline/format.rs` (inline) | 85 |
| `src/pipeline/resolve.rs` (inline) | 30 |
| `src/ats/lever.rs` (inline) | 16 |
| `src/ats/greenhouse.rs` (inline) | 13 |
| `src/ats/ashby.rs` (inline) | 8 |
| `src/ats/workable.rs` (inline) | 10 |
| `src/ats/smartrecruiters.rs` (inline) | 12 |
| `src/ats/workday.rs` (inline) | 13 |
| `src/tui/widgets/layout.rs` (inline) | 11 |
| `src/tui/views/jobs.rs` (inline) | 14 |
| `tests/cli.rs` | 16 |
| `tests/pipeline_clean.rs` | 11 |
| `tests/pipeline_format.rs` | 5 |
| `tests/pipeline_import.rs` | 12 |
| `tests/smoke.rs` | 2 |
| **Total** | **316 (was 18)** |
