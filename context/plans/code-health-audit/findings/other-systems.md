# Other Systems (Config, Database, Autofill, Main) — Code Health Findings

**Systems covered:** `src/config.rs`, `src/db/{mod,schema}.rs`, `src/autofill/*`, `src/main.rs`, `src/http.rs`, `src/lib.rs`
**Finding count:** 4 findings (0 high, 3 medium, 1 low)

---

## Database (`src/db/schema.rs`)

### `migrate_is_idempotent` test proves the migration batch is re-runnable, but there is no test for schema forward-compatibility across migrations 001..006
- [ ] Add an inline test to `src/db/schema.rs` that exercises the full migration chain on an empty DB, then on a DB already at migration N-1, for every N in 1..=6.

**Category:** Test Coverage Gaps
**Severity:** Medium
**Effort:** Small
**Behavioural Impact:** None

**Location:**
- `src/db/schema.rs` — 1011 lines, 29 tests, 6 migrations

**Current State:**
Existing tests prove:
- `schema_creates_successfully` — clean migration on empty DB
- `migrate_is_idempotent` — running the full migration twice doesn't error
- `archived_status_accepted` — migration 002 + 003 together accept 'archived'

Missing: a test that runs migration 001 alone (simulating an old DB), then runs the full current migration set, and verifies every later migration's expected state holds. Migration-ordering bugs are the canonical production-regression source in SQL systems — a partial chain can succeed in the test env but fail in production where the DB was at an older schema version.

**Proposed Change:**
Add a test that:
1. Creates a bare `companies` + `jobs` + `user_decisions` schema matching migration 001's output.
2. Runs the migration function (which iterates migrations 001..006).
3. Asserts that every column added by migrations 002-006 now exists with the right constraint.

**Justification:**
Session-5 `archived_at` column and session-6 `application_packages` table were added via migration. Every future session that adds a migration carries the same forward-compatibility risk. A single well-designed test catches the whole class.

**Expected Benefit:**
Forward-compat guarantee becomes verifiable. New migrations inherit the test pattern naturally.

**Impact Assessment:**
Zero — additive test. No production-code change.

---

## Config (`src/config.rs`)

### `passes_location` does case-insensitive substring match in a nested O(L × P) loop on every job
- [ ] Pre-lowercase location patterns once at `Preferences::load_from` time; add an early-exit when the search filter is empty.

**Category:** Performance Improvement
**Severity:** Low
**Effort:** Small
**Behavioural Impact:** None (same matching semantics, fewer allocations)

**Location:**
- `src/config.rs:151-177` — `passes_location`

**Current State:**
For every job, for every location in `all_locations`, for every pattern in the provider's `LocationConfig.patterns`, the function:
1. Allocates a new lowercased String for `loc`.
2. Allocates a new lowercased String for `pattern`.
3. Substring-matches.

For 287 companies × ~50 jobs × ~3 locations × ~5 patterns = ~215,000 lowercasing allocations per `cernio search` run. Each is a small heap alloc + copy. Not a bottleneck at current scale but a free improvement.

**Proposed Change:**
Pre-lowercase patterns at TOML-load time:
```rust
pub struct LocationConfig {
    #[serde(deserialize_with = "lowercase_patterns")]
    pub patterns: Vec<String>,  // stored lowercased
}
```
Or derive a `patterns_lower: Vec<String>` in `Preferences::load_from`.

Still lowercase `loc` per iteration, but once per location (not once per loc×pattern pair). Or use `str::eq_ignore_ascii_case` / `str::to_ascii_lowercase` for allocation-free case handling when ASCII-only locations dominate (UK city names do).

**Justification:**
- Classic "hoist invariant computation out of the loop" — patterns don't change per-job; lowercasing them once is a free move.
- The allocation count scales multiplicatively in the worst dimension (3 × 5 = 15 allocations per job); at ~14,000 jobs per search run, that is ~210K small allocations.

**Expected Benefit:**
Reduces allocation count by ~80% on the location filter's hot path. Not measurable end-to-end at current scale but compounds with the `fetch_stats` consolidation into a materially lighter poll cycle.

**Impact Assessment:**
Zero functional change. Same match semantics; only the allocation pattern differs.

---

## Autofill (`src/autofill/*`)

### Audit deliberately defers autofill deep-dive; pure-parts test coverage noted in cross-cutting
- [ ] (Cross-reference: `findings/cross-cutting.md` §Test Coverage Gaps §"Autofill module has no tests despite being implemented and wired up")

**Category:** Triage Needed
**Severity:** (not ranked)
**Effort:** Small (pure-parts tests); Medium (React form filling fix)
**Behavioural Impact:** Varies by scope

**Location:**
- `src/autofill/mod.rs`, `src/autofill/common.rs`, `src/autofill/greenhouse.rs`

**Current State:**
Autofill is scaffolded but broken on React-controlled forms. Chrome launches, DOM selectors may be wrong, `el.value = "..."` doesn't trigger React state. This is a Known Issue documented in `notes/autofill-status.md` and `architecture.md` "Next priorities" #1. Per `references/analysis-categories.md` §15 §"What NOT to Flag": issues already documented in `context/systems/` files' Known Issues are not re-flagged.

The audit confirms the state matches the documentation and surfaces only one net-new item (pure-parts tests, cross-referenced above). No further autofill-specific findings.

**Proposed Resolution:**
The React-fix work is tracked; the audit has nothing to add beyond the test-coverage cross-reference. When that work begins, consider the dead-code attributes in `autofill/*` as part of that session's cleanup (documented in `findings/cross-cutting.md` §Dead Code Removal).

---

## Main (`src/main.rs`)

### `cmd_*` dispatch and helpers total 409 lines with 2 clippy hits that are trivial to fix
- [ ] Apply the two clippy fixes: collapse the nested `if` at `src/main.rs:376-380` into a single `if let ... && ...` pattern (clippy::collapsible_if); leave the rest of the file untouched.

**Category:** Inconsistent Patterns
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None

**Location:**
- `src/main.rs:376-380` — nested `if let Some(parent) = ...` + `if !parent.as_os_str().is_empty()`

**Current State:**
```rust
if let Some(parent) = db_path.parent() {
    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent).expect("failed to create state/ directory");
    }
}
```
Clippy suggests:
```rust
if let Some(parent) = db_path.parent()
    && !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent).expect(...);
    }
```
This is the let-chains feature stabilised in Rust 1.88. Edition 2024 (the project's edition) allows it.

**Proposed Change:**
Apply the clippy `--fix` suggestion manually (or verify `cargo clippy --fix --bin cernio` produces the expected edit).

**Justification:**
Clippy baseline reduction. The let-chains form reads more naturally.

**Expected Benefit:**
Clippy baseline drops by 1. Minor readability improvement.

**Impact Assessment:**
Zero — same logic, tighter syntax.

---

## HTTP & Lib (`src/http.rs`, `src/lib.rs`)

Audited as trivial utility / library-surface files per `references/detection-strategies.md` §"When Research Is Not Required":

- `src/http.rs` — 40 lines; one function `build_client` returning `reqwest::Client` with timeout + UA configuration. One `#[allow(dead_code)]` on a legacy `build_client_with_cookies` helper; verify if live via grep for `build_client_with_cookies` — likely another dead-code removal candidate. Recorded as such in `findings/cross-cutting.md` §Dead Code Removal but not individually itemised.
- `src/lib.rs` — 33 lines; `pub mod ats; …` declarations + the `test_support` module. No findings. This is the file that enables the lib+bin split documented in `notes/testing-strategy.md`.

No further findings for these two files.
