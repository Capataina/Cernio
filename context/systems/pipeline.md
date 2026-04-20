# Pipeline

> The six-plus Rust CLI subcommands that turn companies into graded jobs. Scripts handle volume; Claude handles judgment. Every command has one purpose.

---

## Scope / Purpose

The pipeline is Cernio's mechanical workhorse. Each subcommand is a thin, parameterised Rust binary that does one thing: probe slugs, fetch jobs, filter, archive, verify, format, or import. No judgment, no profile awareness, no LLM calls. When a command produces output that requires judgment (new companies, graded jobs, grading verdicts), that output is handed back to Claude via the database.

This split is the central architectural decision of the project — see `notes/scaling-architecture.md` and `notes/collaborative-model.md` for the rationale. This system file documents the current pipeline reality; those notes document why it is shaped that way.

---

## Boundaries / Ownership

### Module layout

```text
src/pipeline/
├── mod.rs           # Module declarations
├── resolve.rs       # cernio resolve — slug probing across all providers
├── search.rs        # cernio search — fetch → filter → insert
├── clean.rs         # cernio clean — tiered archival + expiry
├── check.rs         # cernio check — ATS re-verification + integrity report
├── format.rs        # cernio format — HTML → plaintext normalisation
└── import.rs        # cernio import — CSV/JSON bulk load

src/main.rs dispatches argv[1] to the matching cmd_* wrapper which loads
config + opens the DB and calls pipeline::<mod>::run(...).
```

### What the pipeline owns

- Orchestration: which companies to probe, which portals to fetch, which jobs to clean.
- Concurrency: `pipeline::search` runs fetches in parallel under a Tokio `Semaphore`; `resolve` does the same for probes.
- Persistence side effects: the pipeline reads and writes `companies`, `company_portals`, and `jobs` directly via `rusqlite`.
- Dry-run discipline: every command accepts `--dry-run` that must log what would happen and touch zero rows.

### What the pipeline does **not** own

- API shapes — owned by `src/ats/`.
- Filter semantics — owned by `src/config.rs` (`SearchFilters::passes_*`).
- Schema and migrations — owned by `src/db/schema.rs`.
- Visualisation of results — owned by the TUI, which polls the DB.
- Any judgment about whether a company or job is "good" — owned by Claude skills that read from the DB.

---

## Current Implemented Reality

### Command inventory

| Command | Entry | Purpose | Writes | Key properties |
|---------|-------|---------|--------|----------------|
| `cernio resolve [--company NAME] [--dry-run]` | `pipeline::resolve::run` / `run_single` | Probe slug candidates × provider grid, record portals | `company_portals`, `companies.status` | Expanded slug generator (30 tests); no early termination; per-request retry |
| `cernio search [--company NAME] [--grade G] [--dry-run]` | `pipeline::search::run` / `run_single` / `run_by_grade` | Fetch jobs from resolved portals, apply filter stack, insert | `jobs`, `companies.last_searched_at` | Parallel fetch under semaphore; `INSERT OR IGNORE` by URL; pipeline counts reported per stage |
| `cernio clean [--dry-run] [--jobs-only]` | `pipeline::clean::run` | Archive jobs by tiered age × grade; expire archives | `jobs.evaluation_status = 'archived'`, hard-delete expired | Tiered lifecycle (SS 28d, S 21d, A 14d, B 7d, C/F 3d); user decisions pin jobs; 14-day archive expiry |
| `cernio check [--ats-only]` | `pipeline::check::run` | ATS re-verification + integrity report | — (read-only) | Structured report: health, completeness, staleness, dead URLs, duplicates, profile drift |
| `cernio format [--dry-run]` | `pipeline::format::run` | HTML/entity → clean plaintext, in place | `jobs.raw_description`, `jobs.fit_assessment` | Idempotent (`format(format(x)) == format(x)`); runs silently on TUI startup |
| `cernio import --file PATH [--dry-run]` | `pipeline::import::run` | Bulk-load companies from CSV/JSON/potential.md | `companies` | Dedup via website unique constraint with trailing-slash normalisation; file-clearing semantics on success |
| `cernio unarchive --jobs [--grade G]` | `cmd_unarchive` in `main.rs` (not in `pipeline/` — lives at CLI layer) | Restore archived jobs with timer reset | `jobs.evaluation_status`, `jobs.discovered_at` | Preserves grade + fit_assessment; resets `discovered_at = now` so tiered timer restarts |
| `cernio stats` / `cernio db-status` | `cmd_stats` | Quick DB overview (counts by grade, status, pending) | — (read-only) | Smoke command; used by CLI integration tests |
| `cernio pending [--count]` | `cmd_pending` | List jobs awaiting evaluation | — (read-only) | |
| `cernio lever-list` / `lever-detail` | one-off debug | Ad-hoc Lever API inspection | — | Not part of normal workflow |

`cernio tui` launches the TUI and is documented in `systems/tui.md`.

### Dependency direction

```
┌──────────────────────────────────────────────────────────┐
│                       main.rs dispatch                    │
└──────────────┬───────────────────────────────────────────┘
               ▼
┌──────────────────────────────────────────────────────────┐
│                 pipeline::<subcommand>                    │
│                                                           │
│    resolve / search / clean / check / format / import     │
└───────┬──────────────┬──────────────┬────────────────────┘
        │              │              │
        ▼              ▼              ▼
┌─────────────┐ ┌───────────────┐ ┌───────────────┐
│   ats/      │ │  config.rs    │ │  db/schema.rs │
│             │ │  (filters)    │ │  (rusqlite)   │
└─────────────┘ └───────────────┘ └───────────────┘
```

Only `resolve` and `search` reach into `ats/`. `clean`, `check`, `format`, and `import` are database-only operations; `check` additionally calls into `ats/` to re-probe slugs.

### `cernio search` filter stack

Search is the most complex command. For each resolved portal, the flow is:

```
fetch_jobs(portal)                                    // ats/ layer
    │
    ▼ Vec<AtsJob>
passes_location(provider, job.all_locations)          // config
    │
    ▼
passes_exclusion(job.title)                           // config — "Director", "VP", "Principal"
    │
    ▼
passes_inclusion(job.title)                           // config — "Engineer", "ML", "Systems"
    │
    ▼
job_exists(conn, &job.url)                            // db dedup
    │
    ▼ INSERT OR IGNORE INTO jobs (..., evaluation_status='pending')
```

Counters are tallied at every stage (`total_fetched`, `total_after_location`, `total_after_exclusion`, `total_after_inclusion`, `total_new`) and reported at end of run. Inclusion-keyword list is empty-passthrough (empty list = accept all); exclusion is always applied; location filter looks up patterns per-provider (unknown providers pass through).

Each resolved portal also triggers an `UPDATE companies SET last_searched_at = now`, which the TUI dashboard surfaces as "search pulse" freshness.

### `cernio clean` tier ladder

Jobs are archived when `discovered_at` falls more than N days behind today, where N depends on the job's grade:

| Grade | Active days | Rationale |
|-------|-------------|-----------|
| SS | 28 | Worth aggressive follow-up; keep around longer |
| S | 21 | |
| A | 14 | |
| B | 7 | |
| C / F / ungraded | 3 | Short leash for low-signal noise |

Archived jobs sit in `evaluation_status = 'archived'` with `archived_at = now`. They are hard-deleted 14 days after archival (migration 005 adds `archived_at`). `unarchive` resets `discovered_at = now` so the timer restarts, giving the job a second chance against an updated profile. Jobs with any `user_decisions` entry are pinned and never archived.

Clean does **not** auto-archive companies by grade. Company cleanup is manual — the `archive_company_grades` config exists but defaults to empty (`notes/db-maintenance.md` §Companies). The logic for company archival is implemented but intentionally not wired up.

### `cernio format` idempotency

The format pipeline converts raw HTML and HTML-entity-encoded text into clean plaintext (bullet-list normalisation, link conversion to `[text](url)`, heading demotion, entity decoding, whitespace collapse). Three invariants are enforced and tested in `src/pipeline/format.rs`:

1. Never produces raw tags in output.
2. Never produces triple blank lines.
3. Never panics on malformed HTML (including unmatched `<`, mismatched quotes, nested quotes-with-`>`).

Idempotency (`format_description(format_description(x)) == format_description(x)`) is the property that makes it safe to run on TUI startup via `run_silent()`. If it were not idempotent, the TUI would mangle descriptions on every boot.

---

## Key Interfaces / Data Flow

### End-to-end: `cernio search` from CLI to TUI

This is the critical path — the operation the user invokes most often, and the one with the deepest dependency chain. If any step in this chain breaks, the user notices:

```
 1. argv "cernio search"                              → main.rs dispatch
 2. open_db()                                          → db::Database::open
 3. config::Preferences::load()                        → reads profile/preferences.toml
 4. pipeline::search::run(conn, &filters, dry_run)     → enters pipeline
 5.     get_search_targets(conn, filters)              → SELECT from company_portals
        WHERE company.grade >= min_company_grade
 6.     fetch_all_parallel(&targets)                   → N × tokio task
 7.         per task: provider::fetch_jobs(slug, ...)  → ats/ + common::get_with_retry
 8.             HTTP to {provider}.api.example.com     → reqwest client
 9.             serde deserialise                      → provider-specific types
10.             normalise_*                            → Vec<AtsJob>
11.     for each FetchResult:                          → filter stack
12.         passes_location / exclusion / inclusion    → config.rs
13.         job_exists(conn, &url)                     → db dedup
14.         INSERT OR IGNORE INTO jobs (...)           → db write
15.         UPDATE companies SET last_searched_at ...  → db write
16. pipeline counters printed                          → stdout
17. TUI polls every 2s, queries::load_jobs picks up    → display
    the new rows and promotes them to the Jobs view
    with a "New ●" badge (freshness < 24h).
```

Failure behaviour at each step:

- **step 8** (HTTP): `get_with_retry` handles timeouts/502s up to N attempts with 500ms linear backoff. Permanent 4xx fails silently for that portal; other portals continue.
- **step 9** (deserialise): API drift produces a failed deserialise; that portal returns zero jobs; reported in log but nothing blocks.
- **step 13** (dedup): `INSERT OR IGNORE` swallows duplicate URL errors silently. This is the desired behaviour — re-running search should be a no-op on unchanged data.
- **step 17** (TUI): the TUI re-polls via a fresh connection; WAL mode allows reads during writes, so nothing locks. If the search is still running, the TUI shows partial results as they land.

### Contract with `config`

`SearchFilters` holds `min_company_grade`, `include_keywords`, `exclude_keywords`, and `locations: HashMap<String, LocationConfig>`. The per-provider `LocationConfig` is looked up by provider string; an unknown provider passes all locations (empty-list passthrough, verified in `passes_location_unknown_provider_passthrough` test). `included_grades(min_grade)` returns a `Vec<&'static str>` starting from the threshold — used in `get_search_targets` to decide which companies make the cut.

### Contract with `db`

Every pipeline command receives a `&rusqlite::Connection`. The connection is opened once per invocation, used for reads and writes, and dropped on exit. No long-lived connection pool. The TUI uses its own fresh connection per poll. WAL mode (`PRAGMA journal_mode = WAL`) makes all of this concurrent-read-safe.

Table ownership is narrow:

| Command | Reads | Writes |
|---------|-------|--------|
| resolve | companies, company_portals | companies.status, company_portals |
| search | company_portals, companies, jobs | jobs, companies.last_searched_at |
| clean | jobs, user_decisions | jobs.evaluation_status, jobs.archived_at, delete expired |
| check | companies, company_portals, jobs, user_decisions | — |
| format | jobs | jobs.raw_description, jobs.fit_assessment |
| import | companies | companies |
| unarchive | jobs | jobs.evaluation_status, jobs.discovered_at |

---

## Implemented Outputs / Artifacts

- `src/pipeline/{resolve,search,clean,check,format,import}.rs` — the six mainline commands
- `src/main.rs` `cmd_*` wrappers for argv parsing
- 30 inline tests for `slug_candidates` in `resolve.rs` (Phase 3)
- 85 inline tests for `format.rs` (Phase 2) — largest test file in the codebase
- `tests/pipeline_clean.rs` (11), `tests/pipeline_format.rs` (5), `tests/pipeline_import.rs` (12) integration tests
- `tests/cli.rs` (16) CLI tests via `assert_cmd` + `CERNIO_DB_PATH`
- `state/cernio.db` — the shared persistence target
- Log output and stage counters printed to stdout on each run

---

## Known Issues / Active Risks

- **`cernio resolve` has no test coverage for the fetch/HTTP path.** `slug_candidates` is tested exhaustively (30 tests), but the actual probe loop is network-dependent and excluded from the test suite. A regression in probe ordering or provider dispatch would only surface on real runs. Mitigated by the production fact that resolve is run interactively and anomalies are caught by eye.
- **`cernio check` likewise excluded from tests** — same reason, real HTTP calls. Its outputs are text reports consumed by the `check-integrity` skill, so regressions there would be caught by the conversational post-processing step.
- **Search does not re-fetch existing job descriptions.** Once a URL is in `jobs`, it is never re-fetched. If the company updates the listing (corrects the location, adds requirements), Cernio shows the first-seen version forever. Acceptable trade-off — the alternative is a churning fetch on every run.
- **Workday portals with null `ats_extra` skip silently in search.** If someone manually inserts a Workday row without subdomain+site, that portal produces zero jobs without warning. See `systems/ats.md` §Workday for the shape `ats_extra` must carry.
- **`cernio clean` company-archival is implemented but unwired.** The config path exists; defaulting empty means company archival never triggers. Deliberate — see `notes/db-maintenance.md`.
- **No rate-limit-aware backoff.** If a provider starts 429-ing at scale, the current retry is linear and will keep hammering. Not observed in production.

Downstream blast radius: `search` failures are silent per-portal; the dashboard and job count drift downward; the user notices but nothing corrupts. `clean` failures in dry-run are caught by the counts printed; in real runs, archived jobs can be restored via `unarchive`. `format` failures would be the most visible (garbled descriptions in the TUI), which is why the idempotency and never-panics tests are tight.

---

## Partial / In Progress

None. All six commands are in routine use. No half-built commands behind flags.

---

## Planned / Missing / Likely Changes

- **`cernio export`** — markdown export of curated results. Partially replaced by the TUI's `e` keybinding which exports the current view, so the standalone command is lower priority.
- **Rate-limit-aware backoff** — if provider-side 429s start biting.
- **Incremental search** — fetch-only-new-since-last-seen, keyed on `posted_date` or provider cursor. Useful once the steady-state DB is large and every full scan returns 95% duplicates.
- **Grade-based search frequency** — currently every resolved portal gets searched on every run. Could tier: SS/S companies daily, A/B weekly, C monthly. Low value until search becomes expensive.

---

## Durable Notes / Discarded Approaches

- **No end-to-end pipeline.** An early design described a single `cernio refresh` command that ran discover → populate → search → grade. Rejected — the project is collaborative, and every stage depends on human judgment at the seams (review discovered companies, confirm grades, decide whether to apply). See `notes/collaborative-model.md`.
- **Sequential search was dropped in favour of parallel.** Original `pipeline::search` fetched portals one at a time. A 273-company run took 10+ minutes. The Semaphore-bounded parallel fetch now completes the same run in under a minute.
- **Grade-threshold-in-SQL beats in-Rust filtering.** `get_search_targets` originally filtered in Rust after loading all portals; now the SQL `WHERE` clause does it. Trivial on 400 companies but compounds at larger scale.
- **`cernio format` was almost two passes.** First design had separate "strip HTML" and "decode entities" commands. Merging them into one pipeline is idempotent and runs once on TUI startup — the two-pass design would have doubled the startup cost for no semantic gain.

---

## Obsolete / No Longer Relevant

- The early `skills/` folder reference in `notes/scaling-architecture.md` § Expected Flow is from pre-migration times. Skills now live at `.claude/skills/`, but the reasoning in that note still applies structurally — scripts still handle volume, Claude still handles judgment.
