---
name: Cernio
status: active
source_repo: https://github.com/Capataina/Cernio
lifeos_folder: Projects/Cernio
last_synced: 2026-04-26
sources_read: 22
---

# Cernio

## One-line summary

Local-first, conversationally-driven job discovery and curation engine in Rust — six ATS provider fetchers, a six-command pipeline, a SQLite contract layer, a 26-file Ratatui TUI dashboard, and nine Claude Code skills, deliberately split so scripts handle volume and the AI layer handles judgment.

## What it is

Cernio is a single-developer Rust system that finds, evaluates, and curates job opportunities by combining a SQLite database, a Ratatui terminal UI, six ATS provider fetchers (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday — plus Eightfold accepted by the schema CHECK constraint but with no fetcher module), six pipeline scripts (resolve / search / clean / check / import / format), and nine Claude Code skills installed as native `.claude/skills/`. The architectural commitment that runs through every part of the system is that **nothing is automated end-to-end** — every action happens inside a conversational session where the user and Claude decide together what to do, scripts handle volume (probing thousands of ATS slug combinations, scanning hundreds of company boards), and the AI layer handles judgment (reading job descriptions, grading fit against a structured profile, maintaining a portfolio-gap feedback loop). The project is local-first by design: a single SQLite file, no Docker, no server, no API keys for the core pipeline, and the AI layer runs through Claude Code sessions rather than hosted inference. Sessions 1–7 (April 7–10 2026) built the core product end-to-end; sessions 8–9 (April 10–21 2026) matured it with a 22-factor location-evaluation rubric, a 306-test retroactive testing pass that surfaced two silent data-loss bugs, a full code-health audit with 27 open findings, and a migration of all nine skills onto the native Claude Code Skill tool framework with obligation-anchored quality checklists.

## Architecture

Cernio uses a strict three-layer architecture with SQLite as the shared contract between every layer and dependencies flowing strictly downward.

```
┌─────────────────────────────────────────────────────────────────┐
│                   Conversational Session                         │
│                   (User + Claude Code)                           │
│  • Decide what to do: discover, populate, search, evaluate      │
│  • Claude orchestrates skills/scripts, evaluates results        │
│  • User makes all application decisions                         │
└──────────┬──────────────────────────────────────────────────────┘
           │ invokes                          │ writes evaluations
           ▼                                  ▼
┌─────────────────────────┐    ┌──────────────────────────────────┐
│    Rust Scripts          │    │      SQLite (state/cernio.db)    │
│  • resolve: probe slugs  │───►│  companies → potential/resolved  │
│  • search: scan ATS      │    │            /bespoke/archived     │
│  • clean: archive stale  │    │  jobs → pending/strong/weak/no   │
│  • check: integrity      │    │  user_decisions, application_pkgs│
│  • format: HTML cleanup  │    │  company_portals (ATS mapping)   │
│  • import: bulk load     │    └──────────────┬───────────────────┘
└──────────────────────────┘                   │ watches
                                               ▼
                                ┌──────────────────────────────────┐
                                │         Ratatui TUI               │
                                │  5 views: Dashboard, Companies,  │
                                │  Jobs, Pipeline, Activity        │
                                │  26 source files, modular        │
                                └──────────────────────────────────┘
```

**Layer responsibilities** (LifeOS Architecture.md):

| Layer | Does | Does not |
|-------|------|----------|
| Conversation | Orchestrates skills and scripts, evaluates jobs against profile, recommends actions, tracks portfolio gaps | Submit applications, make decisions without user input |
| Rust scripts | Combinatorial volume: scan ATS boards, probe slug patterns, fetch job JSON, generate exports | Make judgments, know about the profile, decide what to search for |
| TUI | Real-time display of company universe, evaluation progress, and user decisions; markdown export on keypress | Run scripts, evaluate jobs, or modify data independently |
| SQLite | Contract between all layers — single source of truth for structured data | Contain business logic |

**Crate shape (session-9 lib+bin split).** `src/main.rs` is a thin shim over `src/lib.rs`. The split is load-bearing: Rust integration tests under `tests/` can only see public items from a library crate, not a binary-only crate. Every new top-level module must be declared in `lib.rs`. The CLI binary now reads `CERNIO_DB_PATH` with fallback to `state/cernio.db` so integration tests can target tempdir DBs.

**Module map** (LifeOS Architecture.md, ~14k lines Rust across 56 files):

```text
cernio/
├── src/
│   ├── lib.rs                # Library surface — unblocks tests/
│   ├── main.rs               # Thin shim — CLI dispatch only
│   ├── config.rs             # TOML parser → typed structs
│   ├── http.rs               # Shared HTTP client with retry
│   ├── db/
│   │   ├── mod.rs
│   │   └── schema.rs         # 6 migrations, 5 tables, 29 inline tests
│   ├── ats/                  # 6 provider fetchers + common
│   │   ├── greenhouse.rs / lever.rs / ashby.rs
│   │   ├── workable.rs / smartrecruiters.rs / workday.rs
│   │   └── common.rs         # Shared types, retry, slug normalisation
│   ├── autofill/             # Chrome CDP automation (scaffolded, broken)
│   ├── pipeline/             # 6 CLI commands
│   │   ├── resolve.rs / search.rs / clean.rs
│   │   └── check.rs / import.rs / format.rs
│   └── tui/                  # 26 files, modular architecture
│       ├── app/ handler/ views/ widgets/
│       ├── queries.rs        # ~20 DB read queries
│       └── theme.rs          # Semantic colour palette
├── tests/                    # Integration tests (session 9 phases 5-6)
│   ├── common/mod.rs         # CompanySeed, JobSeed fixtures
│   ├── cli.rs                # 16 CLI tests via assert_cmd + CERNIO_DB_PATH
│   ├── pipeline_clean.rs / pipeline_format.rs / pipeline_import.rs
│   ├── ats_strip_html_parity.rs   # 6 audit follow-up tests
│   └── smoke.rs
├── profile/                  # 16 personal profile files (incl. preferences.toml)
├── companies/                # Discovery landing zone (potential.md)
├── .claude/skills/           # 9 native Claude Code skills
├── context/                  # Project memory
└── state/                    # SQLite database (gitignored)
```

**Key architectural properties** (LifeOS Architecture.md):

- **Idempotency everywhere.** Every pipeline command is safe to run repeatedly. `cernio format` only processes descriptions still containing HTML. `cernio import` deduplicates via URL unique constraint. `cernio resolve` skips already-resolved companies.
- **WAL mode for concurrent access.** SQLite runs in WAL mode so the TUI can read while pipeline scripts write.
- **No hardcoded configuration.** All filters, keywords, location patterns, cleanup thresholds, and grade boundaries live in `profile/preferences.toml`, not in source code. `src/config.rs` reads them as typed structs.
- **Graceful degradation.** If `preferences.toml` is missing or malformed, the system falls back to sensible defaults (min grade B, 14-day stale threshold, exclude F/C).

**Inter-system contracts that break loudest when violated** (LifeOS Architecture.md §Inter-System Relationships):

| A | B | Mechanism | Failure mode |
|---|---|-----------|--------------|
| `ats/` provider modules | `config::SearchFilters::passes_location` | Provider-name string used as both module name and TOML key | New provider without `[search_filters.locations.<provider>]` produces zero jobs post-filter |
| `pipeline/search` | `db` (`jobs`) | `INSERT OR IGNORE INTO jobs` keyed on `url UNIQUE` | Dropping the unique constraint would cause duplicates across runs; plain `INSERT` would error on every re-run |
| `pipeline/format` | `tui/mod::run_silent` | Called on TUI startup as subprocess; **must be idempotent** | Non-idempotent format would mangle cleaned descriptions on every TUI launch — a silent corruption loop |
| `db.application_packages` | `autofill/` | JSON answers written by `prepare-applications`, read by autofill at launch | Schema-contract drift produces partial forms silently |
| `.claude/skills/` | `profile/` (fresh read every invocation) | Mandatory-read block in every SKILL.md | Skills that embed profile snapshots go stale the moment the profile updates (visa dates, project tiers, degree classification) |

**Critical path: `cernio search`** (LifeOS Architecture.md §Critical Paths):

```
argv → main.rs → config::load → pipeline::search::run
  → get_search_targets (SELECT FROM company_portals WHERE companies.grade >= threshold)
  → fetch_all_parallel (Tokio Semaphore, N × {provider}::fetch_jobs)
  → per-portal HTTP via common::get_with_retry
  → serde deserialise → normalise_* → Vec<AtsJob>
  → filter stack (location → exclusion → inclusion)
  → db::job_exists → INSERT OR IGNORE INTO jobs
  → UPDATE companies SET last_searched_at
  → TUI picks up via 2s poll → Jobs view, "New ●" badge
```

Blast radius: HTTP failures fail per-portal (other portals keep going); deserialise failures silently drop that portal; filter drops are counted and reported; DB writes are atomic per-URL via `INSERT OR IGNORE`.

## Subsystems and components

### Database (`src/db/`, 5 tables, 6 migrations, 29 inline tests)

SQLite via `rusqlite` (bundled, WAL mode). Tables: `companies` (408 rows at session 7), `company_portals` (maps companies to ATS providers, supports multi-ATS per company), `jobs` (1184 rows, URL is the dedup key), `user_decisions` (watching/applied/rejected — multiple decisions per job allowed), `application_packages` (pre-generated JSON answers, `job_id` PK, auto-deleted on applied). Migrations 002 and 003 rebuild their entire table because SQLite does not support `ALTER CHECK`. All migrations are idempotent. The fixture for integration tests is `Database::open_in_memory()`. Tiered archival lifecycle: SS jobs stay active 28 days, S 21 days, A 14 days, B 7 days, C/F 3 days; archived jobs are fully deleted after 14 days in archive (allowing re-discovery against an updated profile). **Companies are never auto-archived by grade** — a C company might still have one genuinely good role.

### ATS providers (`src/ats/`, 6 provider modules + common.rs)

| Provider | API base | Pagination | Slug discovery | Companies |
|----------|----------|-----------|----------------|-----------|
| Greenhouse | `boards-api.greenhouse.io/v1/boards/{slug}/jobs` | None (all in one response) | Simple slug probe | 114 (42%) |
| Ashby | `api.ashbyhq.com/posting-api/job-board/{slug}` | None | Simple slug probe | 70 (26%) |
| Workable | `apply.workable.com/api/v1/widget/accounts/{slug}` | `?offset=` | Simple slug probe | 31 (11%) |
| Lever | `api.lever.co/v0/postings/{slug}` + EU endpoint | Offset, 10/page | Dual endpoint probe (US + EU) | 26 (10%) |
| Workday | `{company}.{wd1-12}.myworkdayjobs.com/wday/cxs/{company}/{site}/jobs` | POST with `limit`+`offset` | Variable subdomain + site, no public probe | 20 (7%) |
| SmartRecruiters | `api.smartrecruiters.com/v1/companies/{slug}/postings` | `limit`+`offset`, max 100 | `totalFound > 0` check (200 ≠ success) | 8 (3%) |
| Eightfold | — | — | No fetcher module | 1 (<1%, effectively bespoke) |

Shared infrastructure in `common.rs`: `get_with_retry` (exponential backoff 500ms × attempt; non-retryable 4xx return immediately), `post_json_with_retry`, the unified `AtsJob` struct (title, url, location `Vec<String>`, posted_date, description), and slug normalisation. Per-provider quirks include Lever's dual US/EU endpoints (some UK companies use EU exclusively), Workday's variable subdomain stored in `ats_extra` JSON, and SmartRecruiters returning HTTP 200 for any slug — `totalFound > 0` is the only reliable verification (single most dangerous false positive in the system).

### Pipeline (`src/pipeline/`, 6 commands)

All commands accept `--dry-run` for preview. `cernio search` and `cernio resolve` additionally support `--company NAME`; `cernio search` supports `--grade G` to scope by company grade.

- **`resolve`** — generates 10–20 candidate slugs per company name (lowercase, hyphenated, no-spaces, first-word, first-two-words, stripped corporate suffixes, acronyms), probes ALL slugs against ALL providers (no early termination — finds multi-ATS companies like ClearBank's Ashby + residual Workable). 287 of 408 companies resolved mechanically; 121 remain bespoke.
- **`search`** — fetch → filter chain → dedup → insert. Three-stage filter: per-provider location patterns (empty → KEEP), exclusion keywords (34+ validated against 2001 graded jobs, 0 hits at B+), inclusion keywords (OR logic, empty list → pass everything). Observed ratios: 16,180 raw → 2,001 after filtering. Tracks `last_searched_at` per company.
- **`clean`** — tiered archival driven by grade (above) plus archive expiry (full deletion after 14 days). Preserves any job with a user decision; SS/S/A protected from staleness archival.
- **`check`** — three-category integrity report (health: ATS slug verification, orphaned decisions, missing portals, duplicates; completeness: ungraded entities, missing descriptions, unresolved companies; staleness: company grades >30d, jobs >14d without decisions excluding SS/S).
- **`import`** — bulk import from `companies/potential.md`, `INSERT OR IGNORE` via website unique constraint, **auto-clears the source file** after successful import.
- **`format`** — HTML/entity-encoded descriptions → clean plaintext, in-place on `raw_description`. **Idempotent** (only processes descriptions still containing HTML), **runs silently on TUI startup** via `run_silent()`. Attribute-aware HTML stripping handles `>` inside quoted attribute values. Largest pipeline module at 514 lines.

### TUI (`src/tui/`, 26 files, Ratatui 0.29 + Crossterm)

Five views (Dashboard / Companies / Jobs / Pipeline kanban / Activity timeline) reached through tab keys `1`–`5`. Modularised in session 7: `app/` (state, navigation, actions, pipeline, cleanup — 6 files), `handler/` (key dispatch, navigation, overlays, mouse — 4 files), `views/` (8 files including the 31.5KB `dashboard.rs`), `widgets/` (`grade_bar`, `text_utils`, `toast`, `layout` with `distribute()` — 5 files), `queries.rs` (~20 DB query functions), `theme.rs` (semantic palette: SS=magenta, S=green, A=cyan, B=yellow, C=dark_gray, F=red; freshness; activity types; badges; countdown urgency).

Dashboard components include a 7×12 GitHub-style activity heatmap, search pulse with freshness colouring, application progress bar, visa countdown with urgency colours, top-companies leaderboard by SS+S+A count, session welcome diff (12h lookback), grade distribution bars, and pipeline health by ATS provider. Interaction patterns: vim navigation (`j`/`k`), `Space` for quick-peek popup, `Enter` to drill, `w/a/x/i` for decisions, `o` opens URL and auto-marks applied, `y` copies URL via pbcopy with toast, `g` grade override, `Ctrl+G` smart grouping, `s` sort cycle, `f` focus mode (hides F/C + applied), `D` archive F immediately, `/` instant search, `e` export to markdown, `?` help overlay, `A` toggle archived. Mouse support (scroll wheel, click selection, ctrl/shift multi-select, tab-bar click) is first-class. Responsive layout has Full (120+ cols), Stacked (80–119), Compact (<80) modes.

### Skills ecosystem (`.claude/skills/`, 9 native Claude Code skills)

Migrated from `skills/` to `.claude/skills/` in commit `bebfbc5` (legacy folder removed in `d3e4e58`) to gain Skill tool auto-discovery, YAML engineered triggers with negative-trigger clauses, and `/skill-name` slash completion. Every SKILL.md was then re-audited via skill-creator across commits `319ed60`→`1c9ab85` to add Tier 3 evidence-anchored checklists, What-I-Did-Not-Do declarations between workflow steps, and obligation-anchored mandatory-read tables. Total skill documentation is ~288KB — nearly as large as the Rust source code (the documentation IS the product for skills).

| Skill | Purpose |
|-------|---------|
| `discover-companies` | Profile-aware company discovery via parallel sector subagents (creative web search across VC portfolios, Rust crate contributors, conference sponsors, HN "Who's hiring", engineering blogs) |
| `populate-db` | Validates discovered companies, runs `cernio resolve`, handles AI fallback |
| `resolve-portals` | AI fallback for ATS portal resolution when mechanical probing fails |
| `search-jobs` | Orchestrates full search cycle — script half (`cernio search`) + bespoke half (AI agents for companies without supported ATS), insert-obligation-anchored |
| `grade-companies` | Enrich + grade S/A/B/C with profile-grounded reasoning (`what_they_do`, `location`, `sector_tags`, `grade_reasoning`, `why_relevant`) |
| `grade-jobs` | Grade SS-F with question-first rubric, calibration anchors, lifestyle modulator, mandatory description citation |
| `check-integrity` | AI re-evaluation, cross-tier consistency, portfolio-gap maintenance (reads 10 jobs per grade, extracts technology patterns, writes findings) |
| `profile-scrape` | Scrape GitHub repos and update profile with evidence-based entries (describe what's built, not what's planned) |
| `prepare-applications` | Generate tailored application answers per job, store JSON in `application_packages` for autofill |

The mandatory-read protocol (added session 3 after agents skipped reference files and produced shallow output) requires every skill agent to read its `SKILL.md`, every file in its `references/`, and every file in `profile/`. The session-9 audit replaced exhortation framing ("be thorough") with verifiable obligations ("produce artefact X", "quote the last line of each reference"). Three skills (`resolve-portals`, `grade-jobs`, `prepare-applications`) received "step 0 script call" patches in commit `bee129a` to fix the asymmetry where a skill did work a precursor script call could have done mechanically for free.

### Config (`src/config.rs`, 31 inline tests)

`Preferences::load()` parses `profile/preferences.toml` (13.9KB) into typed structs: `SearchFilters` (`min_company_grade`, `include_keywords`, `exclude_keywords`, per-provider `locations: HashMap<String, LocationConfig>`) and `CleanupConfig` (`remove_job_grades`, `stale_days`, `archive_company_grades`). Three filter methods on `SearchFilters` — `passes_exclusion` (case-insensitive title match), `passes_inclusion` (OR logic, empty list → pass everything), `passes_location` (any location matches any provider pattern, empty locations → KEEP). The "empty data → include, not exclude" rule is deliberate across every filter: false negatives are the enemy.

### Grading (system note + 4 reference files in skills)

Iterated through four phases driven by production failures: dimension-weighted scoring (sessions 1–3, failed because agents assigned middling scores to everything) → hard grade floors (session 4, failed because rigid floors forced Solutions Architect at Amazon to A) → career-stage calibration with relative grading (session 4, improved but still mechanical) → **question-first reasoning** (current rubric). Company questions: would you be proud to work here, could they hire you, would you grow, would you find it engaging. Job questions: can I get it (seniority/requirements), good CV line, do I have an edge, would I enjoy it, practical constraints. Dimensions become analytical support, not the primary scoring. Calibration-anchored against existing DB examples (2–3 per tier). Mandatory description citation prevents the title-only failure mode. Session 8 added `lifestyle-preferences.md` as a **same-tier modulator**, not a Tier 3 tiebreaker — Kings Cross / Nine Elms-class areas lift boundary grades, Croydon-class areas push them down (grade movement across boundaries, not just within-grade).

### Location evaluation (session 8)

A 22-factor three-tier rubric evaluating cities at city / country / hybrid level across both current state and trajectory (1–3 / 5–7 / 10–15 year horizons). Tier 1 dominant factors: visa accessibility for a Turkish national at entry level (country), target firm density in profile sectors (city), urban aesthetic match (city), safety and civic order (city with country floor), political/legal stability (country, 10–15 year outlook). Tier 2 meaningful (10 factors including nightlife trajectory, salary × cost-of-living, tax regime for high earners, secular public culture, café culture, path to permanent residency). Tier 3 fine-tuning (7 tiebreakers including integration quality, housing depth, airport connectivity, food, healthcare). Mechanical constraints the evaluator cannot override: Turkish national no dual citizenship; UK Graduate visa expires August 2027; zero years professional work history; BEng CS 2:2 University of York; Turkish/English/A2-B1 German. Session 8 ran 10 parallel research agents producing `context/references/location-master.md` (71KB synthesis) plus `agent-01..10.md` (~6,500 combined lines). Headline conclusions: London #1 by unanimous agreement; "Amsterdam rejected" overturned unanimously (the rubric's explicit instruction to ignore prior profile verdicts paid off).

### Autofill (`src/autofill/`, 3 files, **scaffolded but non-functional**)

Architecture: `prepare-applications` skill writes JSON answers to `application_packages` table; the autofill binary (Chrome via `chromiumoxide` CDP) reads the package on launch and fills the form. Working: `mod.rs` ApplicantProfile + provider dispatch + package JSON parsing; migration 006 `application_packages` table; TUI `p` key spawns autofill and marks applied; yellow `●` indicator for jobs with prepared packages; package cleanup on applied; Chrome launches and navigates. **Broken:** JS value injection (`el.value = "..."; dispatchEvent(...)`) does not trigger Greenhouse's React controlled-component state. Fix path: replace JS `el.value =` with CDP `Input.insertText` or `nativeInputValueSetter`. CSS selectors in `greenhouse.rs` were written from documentation, not from inspecting real Greenhouse forms — they need verification before autofill can work even after the React state fix.

### Testing (`src/**/tests` inline + `tests/`, 325 tests)

Six-phase retroactive pass in sessions 8–9 (18 tests at session 7 → 325 at session 9). Six architectural decisions enabled the pass: lib+bin split (Rust integration tests can only see public items from a library crate); `CERNIO_DB_PATH` env var (per-test tempdir DBs); `test_support::open_in_memory_db()` workhorse fixture; inline `#[cfg(test)] mod tests` for private pure functions, integration `tests/` for public flows and CLI; offline JSON fixtures over HTTP mocking (faster, deterministic, doubles as living provider-response documentation); TUI tested by state and pure helpers, not by rendering. Test counts include `format.rs` 85, `config.rs` 31, `resolve.rs` 30, `db/schema.rs` 29, six ATS providers totaling 72, `tests/cli.rs` 16, `tests/pipeline_*` 28, `tests/ats_strip_html_parity.rs` 6 (audit follow-up), TUI helpers 34. The single most load-bearing property is `format_description(format_description(x)) == format_description(x)` — without idempotency, every TUI launch would mangle cleaned descriptions in a silent corruption loop. **The pass found and fixed two silent data-loss bugs** (commit `12897aa`) — strongest single argument for retroactive test investment.

### Code health (audit at session 9)

Full two-pass repository audit (commit `c7973e0`) surfaced 27 actionable findings across 8 systems: 4 HIGH, 14 MEDIUM, 7 LOW, 2 triage. Audit modified zero production code. Four HIGH findings: (1) four divergent `strip_html` implementations across `src/ats/`, two diverge on quote-handling, the divergent Workable version is live (latent correctness bug on descriptions with `>` inside quoted HTML attributes); (2) N+1 query in `pipeline::search::run_by_grade` — 288 round-trips per grade-scoped search; (3) `fetch_stats` issues 16 SQL queries per 2s TUI poll (~29k queries/hour); (4) SmartRecruiters pagination missing `get_with_retry` — silent partial fetches on transient 502s. Plus a 7-batch implementation sequence and a 37-row dead-code-sweep disposition table. None implemented yet.

## Technologies and concepts demonstrated

### Languages

- **Rust (edition 2024)** — entire pipeline, TUI, ATS fetchers, autofill, config, DB layer. ~14k lines across 56 files (LifeOS Overview.md). Library crate (`src/lib.rs`) plus thin binary shim (`src/main.rs`). Heavy use of typed structs from TOML, serde-driven JSON deserialisation across 6 provider response shapes, async via Tokio.

### Frameworks and libraries

- **Ratatui 0.29** — full 5-view terminal UI with mouse support, responsive layout (Full/Stacked/Compact modes), modularised across 26 files in `src/tui/`. Custom `widgets::layout::distribute()` for proportional block sizing.
- **Crossterm** — terminal backend for Ratatui; mouse and trackpad event handling.
- **rusqlite (bundled)** — SQLite driver with no system-SQLite dependency. WAL mode for concurrent TUI-read-while-script-writes.
- **Tokio** — async runtime for pipeline scripts; `tokio::sync::Semaphore` bounds parallel ATS fetches in `fetch_all_parallel`.
- **Reqwest** — HTTP client wrapped by `common::get_with_retry` and `post_json_with_retry` with exponential backoff (500ms × attempt) and non-retryable 4xx fast-fail.
- **Serde + serde_json** — ATS response deserialisation; `application_packages.answers` JSON.
- **toml = "0.8"** — `preferences.toml` → typed config structs in `src/config.rs`.
- **chromiumoxide** — Chrome DevTools Protocol client for the autofill subsystem (Chrome launches, navigates; React form filling broken).
- **assert_cmd** — CLI integration tests in `tests/cli.rs`; `Command::cargo_bin("cernio")` spawns the real binary against a per-test tempdir.
- **proptest, tempfile, predicates** — property and fixture testing.

### Runtimes / engines / platforms

- **SQLite (single file, WAL)** — local-first storage; the contract layer between conversation, scripts, and TUI. 5 tables, 6 migrations, 29 inline schema tests.
- **Claude Code (Skill tool / native skills)** — `.claude/skills/` with YAML engineered triggers, negative-trigger clauses, slash-completion. The AI layer of Cernio runs through Claude Code sessions, not hosted inference.

### Tools

- **`cargo test`** — full suite runs sub-second once compiled; `--lib` for unit only, `--test <name>` for one integration file.
- **`assert_cmd::Command::cargo_bin("cernio")`** — CLI subprocess testing.
- **`pbcopy`** (macOS) — clipboard integration on `y` key for URL copy.

### Domains and concepts

- **Three-layer architecture with strict downward dependency direction** — conversation drives scripts and TUI; both scripts and TUI depend on SQLite; no layer depends upward.
- **Idempotency as a design property** — every pipeline command safe to re-run; `format::run` provably idempotent on its column (load-bearing because TUI startup runs it silently); `INSERT OR IGNORE` via URL UNIQUE constraint as dedup contract.
- **Filter chain with bias toward inclusion** — empty data passes every filter; false negatives are explicitly framed as the enemy because a missed good role is unrecoverable while a misgraded irrelevant role costs 30 seconds.
- **Question-first reasoning over dimension-weighted scoring** — four iterations of the grading rubric, each driven by a concrete production failure (Amazon at B, Monzo at C, Thought Machine SS with 3-5 year requirements). The current rubric forces genuine reasoning by framing core questions; dimensions are analytical support.
- **Calibration-anchored grading** — agents are given 2–3 real database examples per tier as anchors before grading begins, preventing batch-relative deflation when prioritisation skews batches toward high-quality roles.
- **Lifestyle as same-tier grading modulator (not Tier 3 tiebreaker)** — aesthetic-daily-environment compounds over years and moves grades across boundaries, not just within-grade.
- **22-factor three-tier location rubric with trajectory awareness** — each factor evaluated at city / country / hybrid level and at both current state and 1–3 / 5–7 / 10–15 year trajectories; explicit instruction to ignore prior profile verdicts.
- **Mandatory-read protocol for skills** — every skill must read its SKILL.md, all `references/`, and all `profile/` on every invocation; profile snapshots embedded in skills go stale and produce silent grading errors.
- **Obligation-anchored vs exhortation-anchored skill design** — verifiable checklists with evidence outputs ("produce artefact X", "quote the last line of each reference") move agent behaviour where vague "be thorough" is sycophantically absorbed.
- **Per-provider location patterns** — location formats differ dramatically across ATS providers (Greenhouse `"Hybrid"` or `"Berlin; London; Munich"`; SmartRecruiters server-side `?country=gb`); patterns are per-provider in `preferences.toml`.
- **Tiered archival lifecycle** — grade-driven active duration (SS 28d → C/F 3d) with subsequent 14-day archive-then-delete for re-discovery against an updated profile.
- **Lib+bin split for testability** — Rust integration tests under `tests/` only see library crates; the split is the smallest change that unblocked the entire `tests/` directory.
- **Offline JSON fixtures over HTTP mocking** — ATS parser tests construct minimal JSON shaped like real provider responses; faster, deterministic, doubles as documentation.
- **Realistic-roster regression test** — `slug_candidates` is tested against 13 actually-resolved Cernio companies (XTX, Two Sigma, Jane Street, Citadel, Palantir, Stripe, Anthropic, DeepMind, Bloomberg, DRW, HRT, Jump, Hudson River); regressions would silently lose specific beloved companies.
- **F12/F15 script-precursor obligation pattern** — a skill that does work a script could have done first will silently burn tokens and produce inferior output; three skills got mandatory step-0 script calls in commit `bee129a`.
- **Conversational orchestration as workflow** — "run a discovery", "grade these jobs" map to skill invocations; CLI syntax not required.
- **Markdown-as-discovery-zone, SQLite-as-source-of-truth** — discovery results land in markdown first (`companies/potential.md`), then migrate to SQLite via `cernio import`; profile data stays in markdown (human-edited) while companies/jobs/decisions/packages live in SQLite (machine-managed).

## Key technical decisions

(LifeOS Decisions.md — every entry below has explicit "what was rejected" / "why this path" reasoning in source.)

- **Collaborative, not automated.** Original README described a daily `cernio refresh` cron. Revised in the first design session because Caner wanted to be involved in every decision. Consequence: every feature must support a conversational workflow; scripts are parameterised tools, not cron jobs.
- **Scripts for volume, AI for judgment.** A Rust script can check 5,000 ATS combinations in seconds; Claude can read 50 resulting descriptions and assess fit. Neither could do the other's job economically. Consequence: scripts are generic and parameterised; intelligence lives in conversation.
- **SQLite as single source of truth.** Evaluated against markdown files, JSON/JSONL, and Postgres/MySQL. SQLite won on zero ops, single file, full SQL, WAL for concurrent access, trivial backup. Profile stays in markdown (human-edited); companies/jobs/decisions/packages live in SQLite (machine-managed).
- **Question-first grading over dimension-weighted scoring.** Four iterations driven by production failures (dimension-weighted → hard floors → career-stage calibration → question-first). Current rubric forces genuine reasoning; dimensions become analytical support.
- **C-tier companies stay active.** Originally auto-archived; changed in session 4. Job grading handles quality filtering; a C company might have one good role; cost of extra jobs (grading time) is cheap, cost of missing a good role is unrecoverable.
- **False negatives are the enemy.** At every filtering stage, bias toward inclusion. Empty data → include, not exclude.
- **Mandatory-read protocol for all skills.** Added session 3 after agents skipped reference files and produced shallow output. The reference documentation IS the quality bar.
- **TUI grade as primary metric, not evaluation_status.** `evaluation_status` is just a coarser bucketing of `grade`; TUI displays grade only.
- **Per-provider location patterns, not global.** Provider formats differ too much for one shared pattern.
- **Profile scrape: built not planned.** Profile entries lead with implemented code, not README aspirations. Enforced after session 6 found Nyquestro described biological plasticity as if built.
- **Skills in this repo, not upstream.** Cernio's skills are tightly coupled to its data model and workflow; project-specific skills don't generalise.
- **Lib+bin split for testability (session 9).** Smallest change that unblocked `tests/cli.rs`, `tests/pipeline_*.rs`, and the 16-test CLI suite. Every new top-level module must be declared in `lib.rs`. Has paid for itself via 306 new tests and 2 silent data-loss bugs fixed.
- **`CERNIO_DB_PATH` env var for CLI testability (session 9).** Smallest change to make CLI tests viable without touching production behaviour. Each test sets the env var to a per-test tempdir.
- **Lifestyle fit as same-tier grading modulator (session 8).** Aesthetic-daily-environment compounds in a way pay or tax bracket do not; under-weighting it for neatness was the wrong trade-off. Grades move across boundaries, not just within-grade.
- **Native Claude Code skills at `.claude/skills/` (session 9).** Skill tool auto-discovery, YAML engineered triggers, slash-completion. Replaces the older "read SKILL.md when I tell you to" pattern. Every SKILL.md was subsequently re-audited via skill-creator.
- **Obligation-anchored over exhortation-anchored (session 9, cross-domain).** Replace vague "be thorough" framing with verifiable obligations ("produce artefact X", "quote the last line of each reference"). RLHF absorbs exhortation sycophantically; falsifiable checklist items cannot be satisficed without producing visibly-incomplete output.

## What is currently built

LifeOS Overview.md (session 9 snapshot, commit `bee129a`):

| Metric | Value |
|--------|-------|
| Companies | 456 total (408 + 48 dad-list triage 2026-04-21) |
| Jobs | 1184+ — last full grading was session 7 |
| Job grade distribution (s7) | 13 SS · 27 S · 70 A · 142 B · 20 C · 212 F |
| Company grade distribution (s7) | 26 S · 124 A · 182 B · 99 C |
| ATS fetchers | 6 in code (Greenhouse / Lever / Ashby / Workable / SmartRecruiters / Workday); Eightfold accepted by CHECK constraint, no fetcher module |
| Pipeline scripts | 6 mainline (resolve / search / clean / check / import / format) + unarchive / stats / pending |
| AI skills | 9, all skill-creator-audited at `.claude/skills/` |
| TUI | v5 — 5 views, 26 source files |
| DB schema | 5 tables, 6 migrations, 29 inline tests |
| Testing | 325 tests passing (was 18 at s7) across 6 phases + audit parity |
| Code-health audit | 27 open findings (4 HIGH / 14 MEDIUM / 7 LOW / 2 triage) |
| Rust source | ~14k lines, 56 Rust files, 494KB |
| Sessions | 9 completed |

Filter-chain reality (session 5 measured): 16,180 raw ATS jobs → ~8,000 after location → ~4,000 after exclusion → ~2,001 after inclusion → ~484 after F/C archival → ~110 actionable SS+S+A. The 0.7% actionable rate is the validating number for the entire pipeline architecture.

Resolution split: 287 of 408 (70%) resolved mechanically; 121 (30%) bespoke. Dad-list batch resolution: 31 resolved (21 mechanical + 10 AI fallback) + 17 bespoke; 35% of adds landed bespoke (ATS fragmentation is a real discovery barrier).

## Current state

Status: **active**. Last meaningful activity 2026-04-21 (commit `bee129a` — dad-list triage adding 48 companies + 3 skill script-connection patches). In flight per LifeOS Work/: a Phase-2 `populate-from-lifeos` skill that makes LifeOS the canonical profile source and Cernio populates from it (tier system to be removed; `profile-scrape` to be retired). Sessions 8–9 were depth-over-breadth (location rubric, retroactive testing pass, code-health audit, skills migration to native Claude Code framework, principal-engineer CLAUDE.md migration in commit `ce24790`); the dad-list batch was graded standalone (0 S / 7 A / 26 B / 15 C, calibration-anchored) but **not yet job-searched** — the full search-jobs session was deferred per user request.

## Gaps and known limitations

(LifeOS Gaps.md.)

- **Autofill form filling broken.** Entire pipeline exists except the actual filling. Chrome launches and navigates; JS value injection does not trigger React controlled-component state on Greenhouse forms. Fix path is known: replace JS `el.value =` with CDP `Input.insertText` or `nativeInputValueSetter`, then verify CSS selectors against real DOM, then add Lever and Ashby modules.
- **Interview prep skill not built.** Fully designed in `Context/notes/interview-prep-design.md` (personalised curriculum from SS/S/A jobs + portfolio gaps; LeetCode-style TDD problems; multi-component systems practice; company-specific prep). Design exists; implementation has not started.
- **`cernio export` CLI not implemented.** The TUI `e` key exports current views, but there is no CLI batch export.
- **Eightfold ATS fetcher not built.** Listed in CHECK constraint, no `src/ats/eightfold.rs`. Only 1 company uses it; ROI low.
- **Workday resolution requires manual identification.** Variable subdomain + site name; no mechanical probe possible.
- **Bespoke company coverage is spotty.** 121 companies depend on AI-agent search of careers pages; coverage depends on session time.
- **Dad-list batch (48 companies) not yet job-searched.** Graded but no `cernio search` run executed against them yet.
- **Teamtailor fetcher missing.** 4 of 17 dad-list bespoke companies are on Teamtailor (clean public API at `{slug}.teamtailor.com/jobs.json`); building it would convert those 4 from bespoke to resolved immediately. Higher ROI than Eightfold.
- **Parent-company slug expansion in `cernio resolve` missing.** AI fallback surfaced `LexisNexis → workday/relx` and DigitalOcean → `greenhouse/digitalocean98`; mechanical resolver did not try the parent-company path.
- **27 open code-health-audit findings.** Including 4 HIGH (strip_html consolidation/Workable correctness; N+1 in `run_by_grade`; `fetch_stats` 16 queries per 2s poll; SmartRecruiters retry).
- **Chrome automation detection unverified.** "Chrome is being controlled by automated test software" banner appeared during autofill testing; `--disable-blink-features=AutomationControlled` flag effectiveness untested.
- **Greenhouse CSS selectors unverified.** Selectors in `src/autofill/greenhouse.rs` written from documentation, not from inspecting real Greenhouse forms.
- **No automated re-resolution when ATS slugs break.** `cernio check` detects broken slugs; manual re-resolution workflow.
- **Dashboard is the largest single file.** `src/tui/views/dashboard.rs` 31.5KB — modularisation pass queued.
- **Migration 003 has a complex fresh-DB path.** Separate code path for fresh databases (no companies exist) that manually rebuilds the table.

## Direction (in-flight, not wishlist)

(LifeOS Roadmap.md priority items 1–6.)

- **Priority 1 — Fix autofill.** Open a real Greenhouse form, inspect DOM, find actual selectors; replace JS `el.value =` with CDP `Input.insertText` or `nativeInputValueSetter`; test on a job with a prepared package; add Lever and Ashby autofill modules; evaluate the automation-detection flag. No technical blockers — fix path is known.
- **Priority 2 — Interview prep skill.** Fully designed (personalised curriculum from SS/S/A jobs + portfolio gaps; concept files; learning paths; LeetCode-style TDD problems; multi-component systems practice; company-specific study briefs). Implementation pending.
- **Priority 3 — Grade the dad-list 48 companies.** Already done as a standalone batch (0 S / 7 A / 26 B / 15 C); awaiting `cernio search` and downstream `grade-jobs` run. (The Roadmap item predates the grading; the open work is the search + downstream cycle.)
- **Priority 4 — Code-health audit batches.** 7-batch implementation sequence: dead-code removal, `strip_html` consolidation (fixes Workable + removes 70 lines), SQL consolidation (`fetch_stats` 16→4–6 queries) + N+1 fix, retry standardisation across Ashby/Workable/Workday, `verify_ats_slugs` parallelisation + Lever probe swap, dashboard split, `fetch_jobs` list/detail split. Each batch independently testable against the 325-test baseline.
- **Priority 5 — Integrity check post-session.** Run `check-integrity` after the next grading run; cross-check grades, update portfolio gaps, verify no stale data.
- **Priority 6 — Periodic re-search.** Operational rhythm of `cernio search` across resolved companies + bespoke AI search for high-value targets.
- **Profile reorg (Work/Profile Populate Skill).** Phase 1 complete in LifeOS (commit `cf14e1d`); Phase 2 builds a `populate-from-lifeos` skill in Cernio with one-way LifeOS→Cernio flow, retires `profile-scrape`, removes the Flagship/Notable/Minor tier system from the database and grading prompts.

## Demonstrated skills

What this specific project proves the user can do:

- **Designs and ships a full three-layer Rust application** with strict downward dependency direction, a SQLite contract layer, an async pipeline of six CLI commands, six ATS provider integrations, a 26-file Ratatui TUI with five views and mouse support, and 9 AI skills — single-developer, ~14k lines, 56 files.
- **Builds reliable HTTP integrations against six heterogeneous ATS APIs** with provider-specific quirks (Greenhouse no-pagination + offices[] from detail; Lever dual US/EU endpoints; Ashby POST-based; Workable per-job detail fetch; SmartRecruiters `totalFound > 0` verification + server-side `?country=gb`; Workday variable subdomain + site stored in `ats_extra` JSON), with shared retry infrastructure (`get_with_retry` / `post_json_with_retry` with exponential backoff and non-retryable 4xx fast-fail) and offline JSON fixtures that double as provider-response documentation.
- **Designs SQLite schemas with migration discipline** including idempotent migrations, CHECK-constraint table rebuilds (because SQLite has no `ALTER CHECK`), foreign keys with FK-disable during rebuilds, WAL mode for concurrent TUI-read-while-script-writes, and tiered archival with grade-driven active durations and 14-day archive-expiry windows.
- **Builds a full Ratatui application** with five views, modular architecture (app/handler/views/widgets across 26 files), responsive layout (Full / Stacked / Compact modes), a custom `distribute()` for proportional block sizing, mouse and trackpad event handling, GitHub-style activity heatmap, kanban pipeline view, semantic colour palette, and contextual status bars.
- **Designs and iterates an evaluation rubric against production failures.** Four iterations of the grading system (dimension-weighted → hard floors → career-stage calibration → question-first), each driven by concrete failures named with company names (Amazon at B, Monzo at C, Thought Machine SS with 3–5 year requirements). The mature rubric is question-first with calibration anchors and mandatory description citation.
- **Builds a 22-factor three-tier reasoning rubric with trajectory horizons** that runs across 10 parallel research agents and integrates a same-tier lifestyle modulator into job-grade outputs.
- **Architects a Claude Code skill ecosystem** with mandatory-read protocols, evidence-anchored checklists, What-I-Did-Not-Do declarations, obligation-anchored quality gates (replacing exhortation-anchored framing), and 9 fully-audited skills totaling ~288KB of reference documentation.
- **Designs idempotency as a load-bearing system property.** `format::run` is provably idempotent on its column; runs silently on every TUI startup; without idempotency the system would silently corrupt cleaned descriptions on every launch. Backed by 85 tests including a dedicated idempotency-on-realistic-payload check.
- **Performs a 306-test retroactive testing pass** on a 14k-line Rust codebase via deliberate architectural changes (lib+bin split for `tests/` visibility; `CERNIO_DB_PATH` env var for tempdir DB isolation; `test_support::open_in_memory_db()` workhorse fixture; offline JSON fixtures over HTTP mocking; TUI tested by state and pure helpers, not by rendering). The pass found and fixed two silent data-loss bugs.
- **Runs a full code-health audit** producing 27 actionable findings across 8 systems, a 7-batch implementation sequence, a 37-row dead-code-sweep disposition table, and 6 new diagnostic parity tests — modifying zero production code in the audit itself.
- **Reasons about coupling and shared-string boundaries** — provider-name strings as a shared identifier across `ats/`, `config.rs`, `preferences.toml`, and the `db.ats_provider` CHECK constraint; `ats_extra` JSON as a provider-specific unversioned schema; load-bearing `INSERT OR IGNORE` semantics keyed on `url UNIQUE`.
- **Treats false negatives as the enemy** in filter design — empty data passes every filter, with explicit reasoning that a missed good role is unrecoverable while a misgraded irrelevant role costs 30 seconds.
- **Generalises operational patterns from concrete failures.** The F12/F15 "skill-doing-work-a-script-could-have-done" asymmetry surfaced during the dad-list run and was patched into three skills as mandatory step-0 script calls; the lesson generalises everywhere.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Cernio/Overview.md | 102 | "> Session 8 added the 22-factor location-evaluation rubric + lifestyle modulator; session 9 added 316 tests (surfacing two silent data-loss bugs), a full code-health audit with 27 findings, and migrated all 9 skills to native Claude Code integration. Velocity slowed because depth was the goal. See [[Cernio/Session History#Session 9]] for the full breakdown." |
| Projects/Cernio/Architecture.md | 245 | "Commits `319ed60` → `1c9ab85` (sessions 9) shipped per-skill skill-creator iterations. `CLAUDE.md` migrated to the principal-engineer personality (commit `ce24790`), merging Cernio's Living System Philosophy, skill-execution protocol, grade-quality standard, and portfolio-gap tracking doctrines. See [[Cernio/Systems/Skills]]." |
| Projects/Cernio/Decisions.md | 180 | "See [[Cernio/Systems/Skills#Skill Architecture Decisions]]." |
| Projects/Cernio/Gaps.md | 132 | "- [[Cernio/Systems/Testing]] — what is NOT tested and why" |
| Projects/Cernio/Roadmap.md | 111 | "- [[Cernio/Session History]] — what's been done so far" |
| Projects/Cernio/Data Composition.md | 151 | "- [[Cernio/Session History]] — how the data grew across sessions" |
| Projects/Cernio/Session History.md | 144 | "> 18 → 325 tests in one session surfaced two silent data-loss bugs immediately, produced the confidence baseline the code-health audit needed, and now blocks the kind of regression that would have gone unnoticed during sessions 1-7. Every future session benefits from this pass. `[commits 89b37e1, 978be7d, 12897aa]`" |
| Projects/Cernio/Systems/ATS Providers.md | 150 | "- [[Cernio/Systems/Code Health]] — 7 findings open against this subsystem" |
| Projects/Cernio/Systems/Autofill.md | 104 | "- [[Cernio/Gaps]] — autofill is the #1 gap" |
| Projects/Cernio/Systems/Code Health.md | 158 | "- [[Cernio/Roadmap]] — implementation batches are queued" |
| Projects/Cernio/Systems/Config.md | 80 | "- [[Cernio/Architecture]] — no hardcoded configuration is a key architectural property" |
| Projects/Cernio/Systems/Database.md | 185 | "- [[Cernio/Systems/Code Health]] — dashboard `fetch_stats` issues 16 queries per 2s poll; SQL consolidation is a HIGH-severity audit finding" |
| Projects/Cernio/Systems/Grading.md | 157 | "- [[Cernio/Systems/Location Evaluation]] — the session-8 location rubric and lifestyle modulator plug directly into this subsystem" |
| Projects/Cernio/Systems/Location Evaluation.md | 151 | "- [[Profile/Lifestyle Preferences]] — mirrored in LifeOS; see Profile note for the duplication concern" |
| Projects/Cernio/Systems/Pipeline.md | 176 | "- [[Cernio/Systems/Code Health]] — 10 open findings in this subsystem" |
| Projects/Cernio/Systems/Profile.md | 133 | "- [[Cernio/Gaps]] — active market intelligence tracked via portfolio gaps" |
| Projects/Cernio/Systems/Skills.md | 143 | "- [[Cernio/Systems/Autofill]] consumes prepare-applications output" |
| Projects/Cernio/Systems/TUI.md | 201 | "- [[Cernio/Systems/Testing]] — Phase 6 added 34 TUI helper tests" |
| Projects/Cernio/Systems/Testing.md | 190 | "- [[Cernio/Session History#Session 9]] — this subsystem was the centrepiece of session 9" |
| Projects/Cernio/Work/Profile Populate Skill.md | 197 | "- LifeOS commit `cf14e1d` — Phase 1 landing commit" |
