# TUI — Code Health Findings

**Systems covered:** `src/tui/{queries,theme}.rs`, `src/tui/app/{mod,state,navigation,actions,pipeline,cleanup}.rs`, `src/tui/views/{mod,dashboard,jobs,companies,pipeline,activity,chrome,overlays}.rs`, `src/tui/widgets/*`
**Finding count:** 5 findings (1 high, 2 medium, 2 low)

---

## Algorithm Optimisation

### 16-query `fetch_stats` dashboard refresh
- [ ] (Documented in `findings/pipeline.md` §Algorithm Optimisation §"TUI `fetch_stats` runs 16 separate SQL queries per 2-second refresh" — the fix lives in `src/tui/queries.rs` but the audit filed it under the pipeline finding because the hot path is owned by the TUI-poll cycle. See that finding for the full proof chain.)

**Category:** Algorithm Optimisation
**Severity:** High
**Effort:** Medium
**Behavioural Impact:** None

Cross-reference only — see `findings/pipeline.md` for the canonical finding.

---

## Performance Improvement

### Fresh `Connection::open` on every 2-second TUI refresh
- [ ] Hold a single `Connection` as a field on `App` and reuse it across `refresh()` calls. Alternatively, apply `prepare_cached` at query sites to at least amortise statement preparation.

**Category:** Performance Improvement
**Severity:** Medium
**Effort:** Small
**Behavioural Impact:** None (same SQL, same WAL-mode concurrency semantics)

**Location:**
- `src/tui/app/mod.rs:110-152` — `App::refresh` opens a fresh `Connection` via `Connection::open(&self.db_path)?` on every poll
- `src/tui/queries.rs` — every query uses `conn.prepare(...)` (not `prepare_cached`)

**Current State:**
`App::refresh` is called every 2 seconds by the event loop. Each invocation:
1. Opens a fresh `rusqlite::Connection` (syscall-level open of `state/cernio.db`).
2. Runs 16 queries via `fetch_stats` + additional queries for companies/jobs/pipeline/activity/timeline.
3. Drops the connection at the end of `refresh()`.

`notes/tui-design.md` §"Fresh connection per refresh" documents this as an intentional trade-off: "Opens a new `Connection` on each 2-second refresh rather than holding one open. SQLite opens are fast and this avoids connection lifetime complexity in the App struct. Could be optimised later if profiling shows it matters." Profiling would now show it matters — the complaint was written before `fetch_stats` grew to 16 queries and before `fetch_activity_timeline` landed.

Per-open cost on SQLite WAL is ~0.5-1 ms for the open + WAL check (platform-dependent). Over a 1-hour session that is ~1800 opens × 0.75 ms = 1.3 seconds of wall-clock wasted on opens. Not catastrophic but not free either.

Additionally, `conn.prepare(sql)` re-parses and re-plans every statement on every refresh. `rusqlite::Connection::prepare_cached` (available since 0.30) keeps a statement cache keyed on SQL text, eliminating re-parse cost on subsequent calls.

**Proposed Change:**
Two complementary paths:
- (Primary) Store the connection as `Arc<Mutex<Connection>>` or `RefCell<Connection>` on `App`. Reuse across `refresh`. This is the change `notes/tui-design.md` explicitly deferred "until profiling shows it matters."
- (Alternative, smaller) Keep the open-per-refresh pattern but switch every `conn.prepare` call site in `queries.rs` to `conn.prepare_cached`. Amortises statement preparation within a single refresh at least.

**Justification:**
- Documented design choice was to defer; profiling condition is now met.
- `prepare_cached` is the rusqlite-idiomatic way to eliminate re-parse cost — documented at [rusqlite docs](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.prepare_cached).
- WAL mode explicitly supports concurrent readers; holding one reader in the TUI is safe by design.

**Expected Benefit:**
~1.3 s/hour CPU saved on opens. Additional savings from prepared-statement caching (eliminating 16 re-parses every 2 s).

**Impact Assessment:**
Zero functional change. The connection is read-only from the TUI's perspective except for `user_decisions` writes and multi-delete clean-up (both still safe under a reused Mutex). Concurrent writers (CLI pipeline invocations during a TUI session) are supported by WAL; the audit does not propose changing that.

---

### `fetch_top_matches` has no `LIMIT`
- [ ] Add `LIMIT 20` (or similar) to the `ORDER BY` clause in `fetch_top_matches` at `src/tui/queries.rs:517-527`. Adjust to whatever count the dashboard actually displays.

**Category:** Performance Improvement
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** Possible (requires decision) — reduces the visible list on the dashboard if more than 20 rows exist.

**Location:**
- `src/tui/queries.rs:517-540` — `fetch_top_matches`

**Current State:**
The query returns ALL SS/S/A non-archived jobs without a `LIMIT`. Grows unboundedly as the job universe expands. At 110 SS+S+A today (per session-7 counts in `architecture.md`), the list is already long enough that the dashboard can't display all of them — but it still transfers every row every 2 s.

**Proposed Change:**
Add `LIMIT 20` or the actual number shown on the dashboard (confirm with `views/dashboard.rs` top-matches rendering code).

**Justification:**
Unbounded queries in a poll loop are a known anti-pattern. `fetch_top_companies_by_hits` at `src/tui/queries.rs:403-422` already has `LIMIT 5` — the matching pattern for `fetch_top_matches` is inconsistent.

**Expected Benefit:**
Memory transfer per poll drops from O(n_top_jobs) to O(20). Matches the idiom already used by its sibling function.

**Impact Assessment:**
Flagged because it reduces the number of top matches available to the renderer. If the dashboard displays > 20 of these, the change is observable. The renderer's limit should be checked first and the LIMIT matched.

---

## Dead Code Removal

### `#[allow(dead_code)]` clusters in `tui/app/state.rs` and `tui/queries.rs`
- [ ] Audit each `#[allow(dead_code)]` attribute in `src/tui/app/state.rs` (lines 78, 97, 117, 127) and `src/tui/queries.rs` (line 147 `fetch_total_job_count`) to determine which are masking legitimately unused fields/functions that can be deleted versus which are intentional (e.g. `pub` items exposed for integration tests or future use).

**Category:** Dead Code Removal
**Severity:** Low
**Effort:** Small
**Behavioural Impact:** None (compiler-enforced — removing only items with zero callers)

**Location:**
- `src/tui/app/state.rs:78` — `#[allow(dead_code)] pub struct JobRow` — verify every field is read by a view
- `src/tui/app/state.rs:97` — `#[allow(dead_code)] pub struct DashboardStats` — several fields (applied_count, watching_count, rejected_count) are displayed; confirm all are
- `src/tui/app/state.rs:117` — `#[allow(dead_code)] pub struct TopMatch`
- `src/tui/app/state.rs:127` — `#[allow(dead_code)] pub struct Toast`
- `src/tui/queries.rs:147` — `#[allow(dead_code)] pub fn fetch_total_job_count` — called once in `app/mod.rs:31`, so **this attribute is incorrectly suppressing a real caller**; remove it and verify the compiler is happy

**Current State:**
Five `#[allow(dead_code)]` attributes in the TUI module cluster. The attributes exist as blanket suppressions on whole structs rather than per-field. In one case (`fetch_total_job_count`), the attribute appears to be stale — the function IS called.

**Proposed Change:**
For each attribute:
1. Remove the attribute.
2. Run `cargo build` — if clean, leave it off.
3. If clippy now warns, investigate each warned item: delete if truly unused, or re-add the attribute with a `// Kept because: X` comment.

**Justification:**
`#[allow(dead_code)]` is a tool-of-last-resort suppression; it is more useful to catch the warnings and make explicit decisions. See cross-cutting finding in `findings/cross-cutting.md` for the project-wide dead-code sweep.

**Expected Benefit:**
Eliminates 4-5 blanket suppressions in the TUI. Enables clippy dead-code warnings to catch regressions in the TUI-specific surface.

**Impact Assessment:**
Zero by construction — removing an attribute cannot change runtime behaviour.

---

## Modularisation

### `src/tui/views/dashboard.rs` (946 lines) — split-recommended
- [ ] Split `src/tui/views/dashboard.rs` into section-specific submodules: `dashboard/heatmap.rs`, `dashboard/search_pulse.rs`, `dashboard/top_companies.rs`, `dashboard/visa_countdown.rs`, `dashboard/session_diff.rs`, each exposing a `draw_*` function.

**Category:** Modularisation
**Severity:** Medium
**Effort:** Medium
**Behavioural Impact:** None

**Location:**
- `src/tui/views/dashboard.rs` — 946 lines; composite score 0.79 (see Pass-1 checkpoint)

**Current State:**
Session 7's dashboard overhaul introduced several independent visualisations: the GitHub-style activity heatmap, the search pulse meter, the top-companies leaderboard, the visa countdown, the session-diff welcome card, the top-matches list, the grade distribution bars. They all render from the same `App` data but have no shared internal state — each could live in its own file with a `pub(super) fn draw_heatmap(frame, area, app)` entry point.

At 946 lines, the file is the TUI's biggest after-modularisation heir and the TUI's highest-churn view (11 commits in 90 days per the hotspot table). Each new dashboard enhancement adds another 50-100 line block to an already-long file.

**Proposed Change:**
Mirror the existing `tui/app/` modularisation pattern that the session-7 overhaul established (documented in `notes/tui-design.md`):
```
src/tui/views/dashboard/
├── mod.rs            # draw() entry + shared helpers
├── heatmap.rs
├── search_pulse.rs
├── top_companies.rs
├── visa_countdown.rs
├── session_diff.rs
├── top_matches.rs
└── grade_bars.rs
```

**Justification:**
- Matches the modularisation pattern `notes/tui-design.md` established: "The threshold is pragmatic, not dogmatic: files under ~300 lines that handle one concern stay as single files. The split preserves the App struct as the shared state."
- 946 lines is comfortably above that threshold.
- Each dashboard section is an independent unit of rendering — they do not share mutable state, only read App fields.

**Expected Benefit:**
The next dashboard enhancement adds a new file, not another 100-line block. Individual sections become independently readable and reviewable. Per-section maturity can be tracked.

**Impact Assessment:**
Zero functional change. The draw dispatcher keeps the same entry point; only the internal organisation changes. All state remains on `App`.

---

### `src/tui/views/jobs.rs` (595 lines) — split-recommended but lower priority
- [ ] After the dashboard split lands and the pattern is proven, apply the same pattern to `jobs.rs`: extract detail-pane rendering, smart-grouping logic, and new-badge rendering into separate submodules.

**Category:** Modularisation
**Severity:** Low
**Effort:** Medium
**Behavioural Impact:** None

**Location:**
- `src/tui/views/jobs.rs` — 595 lines

**Current State:**
Jobs view is the second-biggest views file. It contains the table rendering, the detail-pane rendering, the "smart grouping" logic, the new-badge logic, and the Ctrl+G group-by-company overlay. Same modularisation rationale as dashboard but with lower line count and lower churn (9 commits in 90 days vs 11 for dashboard).

**Proposed Change:**
Mirror the dashboard split pattern once it is in place.

**Justification:**
Same rationale as dashboard split, lower priority due to smaller file.

**Expected Benefit:**
Independent readability for each rendering subsystem.

**Impact Assessment:**
Zero functional change.
