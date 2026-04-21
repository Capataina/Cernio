# Pipeline — Code Health Findings

**Systems covered:** `src/pipeline/{search,resolve,clean,check,format,import}.rs`, `src/main.rs` dispatch
**Finding count:** 10 findings (2 high, 5 medium, 3 low)

---

## Algorithm Optimisation

### N+1 query in `pipeline::search::run_by_grade`
- [ ] Replace the per-company `SELECT grade FROM companies WHERE id = ?` inside the filter loop with a `WHERE c.grade = ?` clause in the initial `get_all_search_targets` SQL.

**Category:** Algorithm Optimisation
**Severity:** High
**Effort:** Small
**Behavioural Impact:** None (verified — same companies selected, same order, fewer round-trips)

**Location:**
- `src/pipeline/search.rs:187-201` — `run_by_grade`

**Current State:**
```
let all_targets = get_all_search_targets(conn);   // 1 query: every resolved portal
let targets: Vec<_> = all_targets
    .into_iter()
    .filter(|t| {
        let company_grade: Option<String> = conn.query_row(
            "SELECT grade FROM companies WHERE id = ?1",      // 1 query per company
            params![t.company_id],
            |row| row.get(0),
        ).ok().flatten();
        company_grade.as_deref() == Some(grade)
    })
    .collect();
```

At 287 resolved companies (Cernio current state per `architecture.md`), this is **288 separate SQL round-trips** just to build the filter list — 1 initial fetch + 287 per-company grade lookups. SQLite round-trips are cheap but measurable, and the cost scales linearly with the universe.

`get_search_targets` (line 293, used by `run`) already does the grade filter in SQL via `WHERE c.grade IN (...)`. `run_by_grade` should use the same pattern.

**Proposed Change:**
Introduce a parameterised helper (or extend `get_search_targets` with an exact-match mode) that issues:
```sql
SELECT c.id, c.name, p.id, p.ats_provider, p.ats_slug, p.ats_extra
  FROM companies c
  JOIN company_portals p ON p.company_id = c.id
 WHERE c.status = 'resolved' AND c.grade = ?1
 ORDER BY c.name, p.is_primary DESC
```
`run_by_grade` calls it and skips the filter loop entirely.

**Justification:**
Direct evidence — `get_search_targets` already uses this pattern for IN-list filtering; `run_by_grade` is the outlier. Complexity drops from O(n) SQL round-trips to O(1). SQLite with WAL mode is designed for this kind of filter-in-SQL access per the project's existing pattern.

**Expected Benefit:**
Eliminates 287 round-trips (current scale) on every `cernio search --grade <G>` invocation. Matches the idiom used by the other two `run*` functions in the same file.

**Impact Assessment:**
Zero functional change — the filter is semantically identical (`company.grade == ?`), performed by SQLite instead of in Rust. Iteration order preserved by the `ORDER BY` clause. No interface changes.

---

### TUI `fetch_stats` runs 16 separate SQL queries per 2-second refresh
- [ ] Consolidate the 13+ count queries in `src/tui/queries.rs:159-303` into 3-4 multi-aggregate queries using `SUM(CASE WHEN ...)` or a single `GROUP BY` with all statuses/grades.

**Category:** Algorithm Optimisation
**Severity:** High
**Effort:** Medium
**Behavioural Impact:** None (same aggregate values, fewer round-trips)

**Location:**
- `src/tui/queries.rs:159-303` — `fetch_stats`

**Current State:**
`fetch_stats` runs the following queries in sequence on every TUI refresh (which happens every 2 seconds):

1. `SELECT COUNT(*) FROM companies` — total_companies
2. `query_groups` over companies grade — companies_by_grade
3. `query_groups` over companies status — companies_by_status
4. `SELECT COUNT(*) FROM jobs` — total_jobs
5. `query_groups` over jobs eval_status — jobs_by_eval
6. `query_groups` over jobs grade — jobs_by_grade
7. `query_groups` over company_portals — ats_coverage
8. `SELECT COUNT(*) FROM companies WHERE status = 'potential'` — pending
9. `SELECT COUNT(*) FROM companies WHERE status = 'bespoke'` — bespoke_count
10. `SELECT COUNT(*) FROM companies WHERE status = 'archived'` — archived_count
11. `SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = 'applied'`
12. `SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = 'watching'`
13. `SELECT COUNT(DISTINCT job_id) FROM user_decisions WHERE decision = 'rejected'`
14. `SELECT COUNT(*) FROM companies WHERE status = 'bespoke' AND grade IN ('S','A') AND (last_searched_at...)` — bespoke_searchable
15. `SELECT COUNT(*) FROM jobs WHERE grade IN ('SS','S','A') AND evaluation_status != 'archived' AND (raw_description IS NULL OR LENGTH < 50)` — needs_description
16. `fetch_top_matches` — adds one more

At 2-second poll interval and a typical hour-long session, this is 16 × 1800 = **~29,000 queries per hour**. Each is cheap (sub-millisecond) but cumulatively measurable, and each opens/prepares/executes/finalises the `rusqlite::Statement`. Prepared-statement caching is not used (a fresh `Connection` is opened per `refresh()` — see `tui/app/mod.rs:111`).

Queries 8-10 can be one query:
```sql
SELECT status, COUNT(*) FROM companies
  WHERE status IN ('potential','bespoke','archived')
  GROUP BY status
```
Queries 11-13 similarly:
```sql
SELECT decision, COUNT(DISTINCT job_id) FROM user_decisions
  WHERE decision IN ('applied','watching','rejected')
  GROUP BY decision
```
Queries 2+3+8+9+10 collapse to one `GROUP BY` over companies; queries 5+6 collapse to one over jobs.

**Proposed Change:**
Rewrite `fetch_stats` around 4 consolidated queries:
1. One `GROUP BY` over companies covering status AND grade (via `SUM(CASE WHEN ...)` idioms).
2. One `GROUP BY` over jobs covering evaluation_status AND grade.
3. One `GROUP BY` over user_decisions for the three decision counts.
4. One query for ats_coverage (already a single query; unchanged).
Plus `top_matches` and the bespoke-searchable/needs-description specialised queries that depend on joins.

Target: ≤6 queries total, down from 16.

**Justification:**
Research anchor: SQLite's own FTS and analytics guidance recommends `GROUP BY` over multi-query dispatch for dashboards. The prepared-statement caching win from `rusqlite`'s `Connection::prepare_cached` also compounds.

Citations:
- [SQLite query planner](https://www.sqlite.org/queryplanner.html) — single queries let the optimiser share scans.
- [SQLx lazy testing patterns](https://mattrighetti.com/2025/02/17/rust-testing-sqlx-lazy-people) — highlights the same consolidation pattern for dashboard-style stats queries.

**Expected Benefit:**
At 400 companies + 1184 jobs today the savings are on the order of milliseconds per poll — enough to show up as reduced TUI CPU usage over long sessions. At 5× the data size (roadmap includes expanding the universe), the current pattern becomes measurable frame-rate pressure; consolidation is a prerequisite for that growth.

**Impact Assessment:**
Same `DashboardStats` struct, same field values. Only the SQL shape changes. Zero functional change by construction.

---

## Performance Improvement

### `jobs` query unconditionally fetches `raw_description` even for the list view
- [ ] Split `fetch_jobs` into `fetch_jobs_list` (no `raw_description`) and `fetch_job_detail(id)` for the detail pane. Update `App::refresh` to use the list variant; add a detail fetch on selection change.

**Category:** Performance Improvement
**Severity:** Medium
**Effort:** Medium
**Behavioural Impact:** Negligible (flagged) — the detail pane will now require one extra SQL query on selection change instead of inline data. User-visible impact is zero if the query executes within a frame; worst-case is a tiny flicker on fast arrow-key navigation.

**Location:**
- `src/tui/queries.rs:54-145` — `fetch_jobs`
- `src/tui/app/mod.rs:116-123` — `App::refresh` calls `fetch_jobs` on every 2s poll

**Current State:**
`fetch_jobs` always selects `j.raw_description` (line 83). `JobRow::raw_description: Option<String>` carries the full HTML-or-plaintext description — typically 2-4 KB per row after `cernio format` normalisation.

The Jobs view's *list* panel displays only title, company, grade, location, freshness, decision — it does not consult `raw_description`. Only the Jobs *detail* panel shows the description.

Yet the TUI refreshes the ENTIRE job list every 2 seconds, transferring all descriptions from SQLite to Rust memory even when the detail pane is not open.

At 1184 jobs × ~3 KB average, every refresh allocates ~3.5 MB of String data that is then (mostly) unused. This is text-book "fetching the whole struct to use one field" per the Data Layout reference's anti-patterns.

**Proposed Change:**
Introduce two query functions:
- `fetch_jobs_list(...)` — returns a `JobListRow` with everything *except* `raw_description`.
- `fetch_job_detail(conn, job_id)` — returns `Option<JobDetail>` with just the description and anything else needed for the detail pane.

`App` stores `Vec<JobListRow>` for the table and lazily fetches detail on selection change (cached until selection changes again).

**Justification:**
- 3.5 MB per 2 seconds = 1.75 MB/s of unnecessary data transfer through the SQLite cursor. While SQLite is in-process and copies are cheap, the allocator cost is real.
- This is the classic "array-of-structs vs struct-of-arrays" layout issue: the list loop reads only the small fields, but the whole struct gets pulled into cache on each poll.

**Expected Benefit:**
Reduces `refresh()` memory churn by an order of magnitude. Eliminates the String clone for description on every poll regardless of whether detail pane is visible.

**Impact Assessment:**
Detail pane now fetches description on demand. When the user arrows through jobs rapidly, each selection triggers a SQL query — but SQLite in WAL mode handles this in well under a millisecond. Debouncing is not needed at current scale. Flagging as "negligible" because the worst-case visible effect would be a sub-frame delay that is below human perception.

---

### `verify_ats_slugs` in `check.rs` is sequential and uses an ad-hoc HTTP client
- [ ] Parallelise ATS verification probes using the same `Semaphore` pattern used by `pipeline::resolve::run`; use `crate::http::build_client()` instead of `reqwest::Client::new()`; use `lever::probe` (not `lever::fetch_all`) to match other providers.

**Category:** Performance Improvement
**Severity:** Medium
**Effort:** Small
**Behavioural Impact:** None (same hit/miss determination, faster)

**Location:**
- `src/pipeline/check.rs:288-326` — `verify_ats_slugs`

**Current State:**
The function:
1. Line 306: `let client = reqwest::Client::new();` — creates a bare client without the project's retry/timeout/UA configuration used by `crate::http::build_client()`.
2. Line 308-323: iterates over portals sequentially, awaiting each probe one at a time.
3. Line 311: for Lever, calls `lever::fetch_all(&client, slug)` — which fetches the FULL job list just to check if the board is alive. All other providers use `probe()`.

For 287 resolved portals at ~200-500 ms per probe, sequential verification takes 1-2 minutes. Parallelised at concurrency 8 (matching `pipeline::search::fetch_all_parallel`), this drops to 10-20 seconds.

**Proposed Change:**
Spawn probe tasks on a `Semaphore::new(8)` (or `10` to match resolve). Use `lever::probe` like the other providers. Use `crate::http::build_client()` for the shared reqwest client.

**Justification:**
- Lever's `probe` exists precisely for this use case (line 74 of `lever.rs`: "Probe whether a Lever board exists for this slug. Tries both US and EU endpoints with retry").
- Three inconsistencies in one function: ad-hoc client, sequential await, wrong Lever entry point. Research anchor: the 2026 Rust HTTP guidance ([OneUptime — Building Resilient HTTP Clients](https://oneuptime.com/blog/post/2026-01-25-resilient-http-clients-retry-policies-rust/view)) specifically recommends a single client instance shared across the app for connection pooling.

**Expected Benefit:**
`cernio check --ats-only` drops from minutes to seconds. Lever verification stops fetching entire job lists (Lever's fetch returns the full posting array — could be hundreds of jobs — when a single `probe` call returns `Some(_)` vs `None`).

**Impact Assessment:**
- Parallelisation: same hit/miss determination, order of results is already re-sorted (line 287) so ordering is not load-bearing.
- Client change: `build_client()` adds retry + timeout + UA headers; transient failures move from "silent miss" to "retried then miss-or-success."
- Lever probe swap: `probe` and `fetch_all` return different shapes but the boolean question "is the board alive?" is identical; `probe` is a strict subset that avoids transferring all jobs.

---

### `print_report` in clean.rs has identical-branch `if` producing two duplicate bindings
- [ ] Replace `let verb_archive = if dry_run { "would archive" } else { "archived" };` at `src/pipeline/clean.rs:209` with `let verb_archive = verb;` (or delete `verb_archive` and use `verb` directly).

**Category:** Inconsistent Patterns
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None (by construction)

**Location:**
- `src/pipeline/clean.rs:208-209` — matching `verb` + `verb_archive` let bindings

**Current State:**
```rust
let verb = if dry_run { "would archive" } else { "archived" };
let verb_archive = if dry_run { "would archive" } else { "archived" };
```
Two `let` bindings with byte-identical right-hand sides. Clippy's `if_same_then_else` or `branches_sharing_code` pass flags this (see Pass-1 checkpoint — the single "if has identical blocks" hit in the whole codebase).

**Proposed Change:**
Delete `verb_archive` and substitute `verb` at its call sites (lines 239, 208 refer to the Companies block). Or keep `verb_archive` as an alias: `let verb_archive = verb;`.

**Justification:**
Clippy-flagged duplication. Zero readability cost to fix; fixes the one `if has identical blocks` warning in the entire codebase's clippy baseline.

**Expected Benefit:**
Clippy baseline drops from 44 to 43 warnings. Code reads slightly cleaner.

**Impact Assessment:**
Zero. The two bindings produce identical values; collapsing them is a pure textual change.

---

## Inconsistent Patterns

### Tiered archival rules duplicated between `preview` and `execute` in clean.rs
- [ ] Extract `const TIER_RULES: &[(&str, i64)]` at module scope and reference it from both `preview` (line 35-37) and `execute` (line 114-121).

**Category:** Inconsistent Patterns
**Severity:** Medium
**Effort:** Trivial
**Behavioural Impact:** None (same values, single source)

**Location:**
- `src/pipeline/clean.rs:35-37` — `tier_rules` array in `preview`
- `src/pipeline/clean.rs:114-121` — identical `tier_rules` array in `execute`

**Current State:**
Both functions define a local `tier_rules` with SS=28, S=21, A=14, B=7, C=3, F=3. If one is updated without the other, `--dry-run` will report counts against different thresholds than `--real-run` archives. This is the exact shape of a bug that tests might miss (the two functions are tested against their own local tables).

**Proposed Change:**
Lift to `const TIER_RULES: &[(&str, i64)] = &[("SS",28), ("S",21), ("A",14), ("B",7), ("C",3), ("F",3)];` at module top. Both functions iterate it.

**Justification:**
DRY. Dry-run/real-run parity is a core invariant of the command (documented in `systems/pipeline.md` §Key Interfaces — "dry-run discipline: every command accepts `--dry-run` that must log what would happen and touch zero rows"). Duplicating the source table undermines that invariant.

**Expected Benefit:**
Single source of truth for the tier ladder. Impossible to drift.

**Impact Assessment:**
None. Values and iteration order preserved.

---

### `probe_all_providers` in resolve.rs handles SmartRecruiters asymmetrically
- [ ] Document the SmartRecruiters asymmetric probing pattern in a comment block so the "why" survives; or fold SmartRecruiters into the parallel `tokio::join!` with the totalFound check inside the probe's Option return (which it already does).

**Category:** Inconsistent Patterns
**Severity:** Low
**Effort:** Small
**Behavioural Impact:** Possible (requires decision) — folding SR into the parallel join runs more probes per candidate, which under high contention might add small latency. The current asymmetric path exists for a reason that is worth preserving explicitly.

**Location:**
- `src/pipeline/resolve.rs:133-145` — the `tokio::join!` over 4 providers
- `src/pipeline/resolve.rs:147-157` — SmartRecruiters handled separately with early-exit

**Current State:**
The 4-way parallel `tokio::join!` covers greenhouse, lever, ashby, workable. SmartRecruiters is appended after the loop with its own "probe until first hit" early-exit. The asymmetry is documented inline ("Probe SmartRecruiters separately — it returns HTTP 200 for any slug (totalFound:0), so it's slower and noisier") and is intentional.

**Proposed Change:**
This is a **Triage Needed** item rather than a clear win. Two paths:
- (a) Preserve the asymmetry, but add a short `// Why:` comment referencing `context/notes/populate-db-lessons.md` §"SmartRecruiters returns 200 for any slug" so the rationale is discoverable from the code.
- (b) Fold SR into the parallel join. SR's `probe` already handles the `totalFound` check, so correctness is preserved; the trade-off is that SR gets probed for every candidate (potentially dozens per company) instead of exiting at first success, costing extra network calls.

**Justification:**
The code-level rationale is captured in the source but could be stronger. The decision between (a) and (b) depends on the cost/latency balance the team wants — the audit surfaces it but does not prescribe.

**Expected Benefit:**
Either a more discoverable rationale comment (option a) or a simpler symmetric control flow (option b).

**Impact Assessment:**
(a) — none.
(b) — SR probes move from "tried until first hit" to "tried for every candidate," increasing network calls for SR companies. This is possibly slower; possibly faster due to parallelism. Requires a decision the audit is not empowered to make.

---

## Dead Code Removal

### `CleanupReport` is `pub` + `#[allow(dead_code)]` but never used outside the module
- [ ] Change `pub struct CleanupReport` to `struct CleanupReport` in `src/pipeline/clean.rs:7`. Remove the `#[allow(dead_code)]` attribute on line 6. Remove the unused `preserved_by_grade` field (written once as 0, never read).

**Category:** Dead Code Removal
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None (visibility reduction + removing a field that is always 0 and never consulted)

**Location:**
- `src/pipeline/clean.rs:6-15` — `#[allow(dead_code)]` + `pub struct CleanupReport { ... preserved_by_grade: u64 }`
- `src/pipeline/clean.rs:102, 172` — assignments `preserved_by_grade: 0`
- `src/pipeline/clean.rs:207-256` — `print_report` reads every other field but never `preserved_by_grade`

**Current State:**
`CleanupReport` is public but has no callers outside `pipeline::clean`. Grep `CleanupReport` across the project: defined + constructed + printed, all within the same file.

The `preserved_by_grade` field is constructed with literal `0` in both `preview` and `execute` (lines 102 and 172), and `print_report` never references it (grep confirms).

**Proposed Change:**
- Reduce visibility: `pub struct` → `struct`.
- Remove `preserved_by_grade` field entirely from the struct and its two construction sites.
- Remove `#[allow(dead_code)]` (line 6) — private struct with all remaining fields read does not trigger the warning.

**Justification:**
- Visibility reduction: no external callers. `pub` serves no purpose.
- Dead field: always zero, never consulted. Removing it makes the struct's real data dependency explicit.
- Attribute removal: `#[allow(dead_code)]` masks the very condition we just fixed.

**Expected Benefit:**
-1 public type from the pipeline module's export surface. -1 dead field. -1 suppressed-warning attribute.

**Impact Assessment:**
Zero. No external callers to break; `print_report` does not read the removed field; the literal `0` written by `preview`/`execute` has no downstream consumer.

---

## Known Issues and Active Risks

### `pipeline::search::fetch_jobs` wrapper swallows errors into `Vec::new()`, breaking upstream distinction between "truly no jobs" and "transient failure"
- [ ] Change `fetch_jobs` (line 357-397) to return `Result<Vec<AtsJob>, …>` rather than unwrapping errors into `Vec::new()`. Update callers to distinguish retry-worthy empty results (Err) from legitimate empty results (Ok with zero-length vec).

**Category:** Known Issues and Active Risks
**Severity:** Medium
**Effort:** Medium
**Behavioural Impact:** Possible (requires decision) — the retry logic at `fetch_all_parallel` currently retries on empty Vec regardless of cause. Under the new scheme, legitimate empty boards (companies with no open positions) stop triggering the 2-then-3-second retry sleep, which is a behaviour change observable as faster `cernio search` runs.

**Location:**
- `src/pipeline/search.rs:357-397` — `fetch_jobs` wrapper
- `src/pipeline/search.rs:261-272` — `fetch_all_parallel` retry-on-empty loop
- `src/pipeline/search.rs:147-150` — `run_single` retry-on-empty block

**Current State:**
`fetch_jobs` catches errors from every provider via `result.unwrap_or_else(|e| { eprintln!(...); Vec::new() })`. The parallel driver at line 261-272 then uses "empty result" as the trigger to sleep and retry:
```rust
if jobs.is_empty() {
    tokio::time::sleep(Duration::from_secs(2)).await;
    jobs = fetch_jobs(...).await;
}
if jobs.is_empty() {
    tokio::time::sleep(Duration::from_secs(3)).await;
    jobs = fetch_jobs(...).await;
}
```

This conflates two distinct cases:
1. The company has zero open jobs right now (legitimate `Ok(vec![])`) — should return immediately.
2. The fetch transient-failed (network error) — should be retried.

Current behaviour: both cases sleep for 2 + 3 = 5 seconds, costing noticeable latency on every `cernio search` run when companies with empty job boards are present. At 287 resolved portals with perhaps 20 legitimately-empty boards at any given time, that is 100+ seconds of avoidable sleep in the critical path.

Per `notes/populate-db-lessons.md`: "Per-request retry with exponential backoff handles transient errors gracefully without affecting other requests." The retry belongs at the request layer (already implemented via `get_with_retry`); the "retry-on-empty" at the search-pipeline layer is both a belt-and-braces defence AND a bug source.

**Proposed Change:**
1. `fetch_jobs` returns `Result<Vec<AtsJob>, Box<dyn Error>>` (same shape as each provider).
2. `fetch_all_parallel` and `run_single` retry only on `Err(_)`, not on `Ok(vec![])`. The retry-on-error path is redundant with `get_with_retry` (which already retries) — consider dropping it entirely once every provider's `fetch_all` is wrapped in retry (see the ATS findings file).
3. Log empty-but-Ok fetches without retry sleep.

**Justification:**
- Research anchor: [reqwest-retry docs](https://docs.rs/reqwest-retry) — the 2026 consensus is retry at the HTTP layer, not at higher application layers, to avoid double-retry amplification.
- The current design hides both bugs and transient errors behind "empty = retry," making troubleshooting harder.

**Expected Benefit:**
- Fast-path for legitimate empty boards (saves 5s per empty company on every run — at 20 empty companies today, that is 100 seconds per full `cernio search`).
- Clearer distinction in logs between "zero jobs found" and "fetch failed, retrying."

**Impact Assessment:**
Flagged — this is a behavioural tightening, not a neutral refactor. Timing of `cernio search` changes (faster on happy path, similar on transient-failure path). Output to stdout changes (different log lines for empty vs. failed). The change is strictly an improvement, but the implementing engineer should verify it does not interact with any external tooling that parses `cernio search` stdout.

---

### `fetch_all` paths in multiple pipeline locations use bare `reqwest::Client::new()` or `client.get` without `get_with_retry`
- [ ] (Addressed in the ATS findings file — pipeline callers inherit the fix when the provider modules adopt `get_with_retry` uniformly.)

**Category:** Known Issues and Active Risks
**Severity:** (see ATS findings — medium)
**Effort:** (see ATS findings)
**Behavioural Impact:** None (same retry-once-more-on-transient pattern applied consistently)

**Location:**
- `src/pipeline/check.rs:306` — bare `reqwest::Client::new()`
- (Cross-reference: `findings/ats.md` §"Fetch-all paths skip retry in multiple providers")

**Current State:**
See the ATS findings file for the detailed per-provider breakdown. The pipeline callers (`check::verify_ats_slugs`, `search::fetch_jobs`) inherit the retry behaviour (or lack of it) from the providers; the fix is at the provider layer.

**Proposed Change:**
Apply the ATS findings first. This pipeline finding resolves automatically when those fixes land.

**Justification:**
Single root cause — shared HTTP-retry discipline across the fetch path.

**Expected Benefit:**
Retry discipline inherited uniformly.

**Impact Assessment:**
See ATS findings.

---

### `import::flush_company` uses `format!("(no description — needs research)")` with no format args
- [ ] Replace `format!("(no description — needs research)")` at `src/pipeline/import.rs:236` with `"(no description — needs research)".to_string()`.

**Category:** Performance Improvement
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None

**Location:**
- `src/pipeline/import.rs:236` — `format!("(no description — needs research)")`

**Current State:**
Clippy flags this as `useless_format`. `format!` with no args spins up the formatting machinery (growable `String`, formatter state) for what is just a literal-to-owned conversion.

**Proposed Change:**
Use `.to_string()` on the string literal, or `String::from(...)`.

**Justification:**
Clippy `useless_format` — the only hit in the baseline.

**Expected Benefit:**
Microsecond speedup per import per empty-description company; consistency with the project's existing `String::from` / `.to_string()` idiom elsewhere.

**Impact Assessment:**
Zero — same resulting `String` value.
