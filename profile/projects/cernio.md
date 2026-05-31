---
name: Cernio
status: active
source_repo: https://github.com/Capataina/Cernio
lifeos_folder: Projects/Cernio
last_synced: 2026-05-31
sources_read: 26
---

# Cernio

## One-line summary

Local-first, conversational job-discovery and curation engine in Rust — SQLite single source of truth, six ATS provider fetchers, six pipeline scripts, nine native Claude Code skills, a 26-file Ratatui TUI, and an embedded axum web UI — built around the principle that scripts handle volume and AI handles judgment.

## What it is

Cernio is a single-developer, Claude-Code-collaborator system for finding, evaluating, and curating job opportunities against a structured personal profile. It composes a SQLite database, six ATS provider fetchers (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday — plus Eightfold accepted in the schema CHECK constraint with no fetcher), six pipeline CLI commands (`resolve`, `search`, `clean`, `check`, `import`, `format`), nine project-local Claude Code skills under `.claude/skills/`, a five-view Ratatui terminal dashboard with mouse support, and (since 2026-05-30) an embedded axum HTML interface on `localhost:7878` over the same SQLite DB.

It is deliberately not an automated pipeline. The original design called for a daily `cernio refresh` and was revised in the first design session: every action happens in a conversational session where the user and Claude decide together what to do. Scripts handle combinatorial volume (probing ~5,000 ATS slug candidates, scanning hundreds of job boards); skills handle reasoning (rubric-based grading, fit assessment, portfolio-gap maintenance); the user owns every application decision. The TUI and web UI are real-time windows into the SQLite database; they never modify data independently.

The project also functions as a career-coaching feedback loop. Grading reveals market patterns; patterns become entries in `profile/portfolio-gaps.md`; gap closure improves the profile; the improved profile shifts future grades. As of the latest LifeOS snapshot the DB carries ~1,370 graded jobs and 456+ companies across resolved + bespoke statuses; the lane-based-relativity refactor on 2026-05-29 expanded the universe to ~706 companies and ~1,068 lane-tagged jobs, and the web frontend redesign on 2026-05-30 added a second UI surface.

## Architecture

Strict three-layer design, dependencies flowing downward through SQLite as the shared contract.

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
│                          │───►│  companies: potential →          │
│  • resolve  • search     │    │    resolved / bespoke            │
│  • clean    • check      │    │  jobs: pending → evaluating →    │
│  • format   • import     │    │    SS / S / A / B / C / F        │
└──────────────────────────┘    │  user_decisions: watching /      │
                                │    applied / rejected            │
                                │  + company_portals,              │
                                │    application_packages          │
                                └──────────────┬───────────────────┘
                                               │ watched by
                              ┌────────────────┴────────────────┐
                              ▼                                 ▼
                  ┌──────────────────────┐         ┌────────────────────────┐
                  │   Ratatui TUI v5     │         │   Web UI (axum + maud) │
                  │   5 views · 26 files │         │   5 tabs · localhost   │
                  └──────────────────────┘         └────────────────────────┘
```

Layer responsibilities are strictly partitioned. The conversation layer orchestrates skills and scripts and tracks portfolio gaps; it never auto-submits applications. The Rust scripts do combinatorial volume — scan ATS boards, probe slug patterns, fetch job JSON — and never read the profile or make judgments. SQLite is the contract; the TUI and web UI poll/read but write back only through user actions. No layer depends upward.

Module map (Rust, ~14k lines across 56 source files at the last LifeOS snapshot):

```
src/
├── lib.rs                    # Library crate — session-9 split to unblock tests/
├── main.rs                   # Thin shim, CLI dispatch
├── config.rs                 # TOML → typed structs from preferences.toml
├── http.rs                   # Shared HTTP client with retry
├── db/
│   ├── mod.rs                # Public DB interface
│   └── schema.rs             # 6 migrations, 5 tables, 29 inline tests
├── ats/                      # 6 provider fetchers + common types
│   ├── mod.rs, common.rs     # Provider dispatch + retry/slug helpers
│   ├── greenhouse.rs         # No pagination; ?content=true for descriptions
│   ├── lever.rs              # Dual US + EU endpoint probe
│   ├── ashby.rs              # POST API
│   ├── workable.rs           # ?offset= pagination
│   ├── smartrecruiters.rs    # totalFound>0 verification (200 ≠ success)
│   └── workday.rs            # Variable subdomain + site; complex resolution
├── autofill/                 # Chrome CDP automation (scaffolded, broken)
│   ├── mod.rs, common.rs, greenhouse.rs
├── pipeline/                 # 6 CLI commands
│   ├── resolve.rs, search.rs, clean.rs, check.rs, import.rs, format.rs
├── tui/                      # 26 files: app/, handler/, views/, widgets/, queries.rs, theme.rs
└── web/                      # axum router + maud templates + handlers + debug_snap CLI
    ├── mod.rs, templates.rs, debug_snap.rs
    └── handlers/{dashboard, companies, jobs, decisions, activity, detail, ops, api}.rs

tests/                        # Integration tests (346 passing)
├── common/mod.rs, cli.rs, pipeline_clean.rs, pipeline_format.rs,
├── pipeline_import.rs, ats_strip_html_parity.rs,
├── preferences_integrity.rs (21 build-time assertions on preferences.toml),
└── smoke.rs

profile/                      # Synced from LifeOS one-way; Cernio-native files preserved
├── personal.md, education.md, experience.md, visa.md, ...
├── projects/<name>.md × N    # Synthesised per-project files (this file is one)
├── skills.md, preferences.toml (Cernio-native), portfolio-gaps.md (Cernio-native)
└── sync-summary.md

.claude/skills/               # 9 native Claude Code skills, all skill-creator-audited
context/                      # Project memory — architecture.md, notes/, systems/, plans/
state/cernio.db               # SQLite (WAL mode); gitignored historically; web added on 2026-05-30
```

Dependency direction is enforced by Rust's lib/bin split: `main.rs` is a thin shim over `lib.rs`, and integration tests in `tests/` import public items from the library crate. Every new top-level module must be declared in `lib.rs`. The CLI reads `CERNIO_DB_PATH` env var (fallback `state/cernio.db`) so integration tests can target tempdir DBs without filesystem contention.

Key architectural properties:

- **Idempotency everywhere.** Every pipeline command is safe to re-run. `cernio format` only processes descriptions still containing HTML; `cernio import` deduplicates via URL UNIQUE; `cernio resolve` skips already-resolved companies.
- **WAL mode** so the TUI/web reads while pipeline scripts write.
- **No hardcoded config.** Every filter, keyword, location pattern, cleanup threshold, and grade boundary lives in `profile/preferences.toml`. The `config.rs` loader is intentionally lenient (typos → defaults + stderr warning), guarded by `tests/preferences_integrity.rs`'s 21 build-time assertions.
- **Graceful degradation.** Missing/malformed `preferences.toml` falls back to sensible defaults: min grade B, 14-day stale threshold, exclude F/C.

Critical path is `cernio search`: argv → `config::load` → `pipeline::search::run` → `get_search_targets` SELECT → `fetch_all_parallel` (Tokio semaphore over per-provider fetchers) → per-portal HTTP via `common::get_with_retry` → serde deserialise → normalise → location/exclusion/inclusion filters → `db::job_exists` → `INSERT OR IGNORE INTO jobs` → `UPDATE companies SET last_searched_at` → TUI 2s poll picks it up. Blast radius is per-portal: HTTP failures fail one portal at a time; the unique URL constraint is the dedup; filter drops are counted and reported.

## Subsystems and components

### Database (SQLite via `rusqlite`, bundled, WAL mode)

5 tables, 6 migrations. `companies` (name, website UNIQUE, what_they_do, status enum `potential|resolved|bespoke|archived`, location, sector_tags, careers_url, why_relevant, grade S/A/B/C, grade_reasoning, last_searched_at). `company_portals` (FK to companies, ats_provider CHECK constraint listing all 7 providers, ats_slug, ats_extra JSON for Workday subdomain+site, UNIQUE on company+provider+slug). `jobs` (FK to companies, url UNIQUE as dedup, raw_description, evaluation_status, fit_assessment, fit_score, grade SS/S/A/B/C/F, archived_at). `user_decisions` (watching/applied/rejected/interview, multiple per job permitted). `application_packages` (PK job_id, JSON answers, auto-deleted on applied). Migrations 002 and 003 rebuild tables because SQLite lacks `ALTER CHECK`; the pattern is `create _new` → `INSERT FROM` old → `DROP` old → `RENAME`. All migrations are idempotent.

Tiered archival lifecycle is grade-aware: SS 28d active → 14d archive expiry, S 21d/14d, A 14d/14d, B 7d/14d, C/F 3d/14d. After archive expiry the row is fully deleted to allow re-discovery under a potentially updated profile. Companies are never auto-archived by grade — a C company may still post one good role, and the grading cost is cheap relative to a missed opportunity.

### Pipeline (6 commands)

`cernio resolve` generates ~10-20 slug candidates per company (lowercase, hyphenated, no-spaces, first-word, first-two-words, stripped corporate/domain suffixes, acronyms, parenthesised content) and probes them across all providers without early termination. SmartRecruiters is probed for all companies (not just unresolved) because it requires `totalFound > 0` verification — HTTP 200 alone is the system's single most dangerous false positive.

`cernio search` runs the three-stage filter chain (location → exclusion → inclusion) plus dedup → `INSERT OR IGNORE`. Observed ratios at session-7 baseline: ~16,180 raw jobs → ~2,001 after filtering (~88% removed), ~484 actionable after archival (~3% of raw), ~110 SS+S+A (~0.7% of raw). The 0.7% number is the foundational economic case for the architecture: mechanical volume to find the 7-in-1000, AI judgment to identify which 7.

`cernio clean` runs tiered archival; `cernio check` runs a three-category integrity report (health, completeness, staleness); `cernio import` bulk-loads companies from markdown (auto-clearing the source file after a successful import); `cernio format` is the 514-line HTML-to-plaintext converter that runs silently on every TUI/web startup. Format's idempotency (`format_description(format_description(x)) == format_description(x)`) is load-bearing — without it, every TUI launch would further mangle already-cleaned descriptions. Three direct invariant tests guard the property plus an explicit `idempotency_on_realistic_payload` test against a Greenhouse-shaped payload.

### ATS Providers (6 fetchers + Eightfold as bespoke)

Per-provider quirks captured in module-local handling. Greenhouse: no pagination, descriptions need `?content=true`, locations vary "Hybrid"/"Berlin; London"/"offices[]" depending on company. Lever: dual US (`api.lever.co`) + EU (`api.eu.lever.co`) endpoints, some UK companies are EU-only; has `workplaceType` field unavailable elsewhere. Ashby: POST API with `{"jobBoardSlug": slug}` body, all jobs in one response. Workable: `?offset=` pagination, per-job detail fetch for descriptions, structured `location.city` + `location.country`. SmartRecruiters: `limit`+`offset` pagination capped at 100, supports server-side `?country=gb` (Wise: 369 → 138), per-job detail fetch for descriptions. Workday: most complex — variable subdomain (`{company}.wd{1-12}.myworkdayjobs.com`) and site name, POST-based search, no public probe endpoint so resolution requires manual identification or AI fallback. Shared infrastructure in `common.rs`: `get_with_retry` and `post_json_with_retry` with exponential backoff (500ms × attempt), unified `AtsJob` struct, slug normalisation.

Company distribution by provider (pre-dad-list snapshot): Greenhouse 114 (40%), Ashby 70 (24%), Workable 31 (11%), Lever 26 (9%), Workday 20 (7%), SmartRecruiters 8 (3%), Eightfold 1, bespoke 121. Greenhouse + Ashby together cover 64% of resolved companies, so a bug in either affects the majority of the searchable universe.

### TUI (Ratatui 0.29 + Crossterm, 26 files, 5 views)

Modular `src/tui/` layout: `app/` (6 files: state, navigation, actions, pipeline, cleanup), `handler/` (4 files: keys, overlays, mouse), `views/` (8 files: dashboard, companies, jobs, pipeline kanban, activity timeline, chrome, overlays), `widgets/` (5 files: grade_bar, text_utils, toast, layout with `distribute()` for responsive sizing), `queries.rs` (~20 DB query functions), `theme.rs` (semantic palette: grades, freshness, activity, badges, countdowns). Three responsive layout modes: Full (≥120 cols, side-by-side master/detail), Stacked (80-119, single column), Compact (<80, abbreviated). Mouse support is first-class: scroll wheel moves viewport (3 lines/tick), click selects rows and auto-focuses panes, Ctrl-click toggles multi-select, Shift-click range-selects, MacBook trackpad two-finger scroll works natively. Dashboard carries GitHub-style 7×12 activity heatmap, search-pulse with freshness colouring, application progress bar, visa countdown with urgency colours, top-companies leaderboard, session welcome diff (12h lookback), grade distributions, pipeline-health bar. Largest single file is `src/tui/views/dashboard.rs` at 31.5KB (946 lines), flagged for split in the code-health audit.

### Web (axum + maud + tower-http + HTMX + ECharts, added 2026-05-30)

Embedded local-only server on `localhost:7878` over the same SQLite DB. Five tabs (Dashboard, Companies, Jobs, Decisions, Activity), each a focused view rather than a list. Filter system uses chip-based multi-select on Companies and Jobs: four chip personalities (`chip-lane` coloured pill with dot; `chip-grade` outline + grade colour; `chip-plain` monochrome enum; `seg-group .seg` segmented control for binary/ternary toggles). Each chip is a hyperlink whose href is built server-side by toggling that value in the URL via CSV (`?lane=hft,ai-ml&grade=SS,S`); every pane on the page recomputes from the filtered set together. Detail drawer slides in from the right with URL persistence (`?detail=job-N | co-N`); two routes (`GET /detail/job/:id`, `GET /detail/company/:id`) return HTML fragments; HTMX is re-processed inside the drawer body so Apply/Watch/Reject continue working post-swap. Clickable charts cross-filter via `chart.on('click') → location.href`. Cmd-K command palette with substring + prefix + grade ranking; g-leader keyboard shortcuts (`g d/c/j/a/x` → Dashboard/Companies/Jobs/Activity/Decisions) with 1.5s timeout + floating hint. LocalStorage-backed saved searches (cap 30). Ops menu exposes only Clean DB + Format (Check/Search/Unarchive dropped from web — they belong on the CLI). `cernio snap` CLI drives headless Chrome (chromiumoxide, already in deps for autofill) over all 5 tabs and captures full-page, per-pane, and viewport-slice PNGs into `/tmp/cernio-debug/<ts>/` for self-driven visual debugging. Modular bundles: `static/css/{base,motion,components,chrome,filters,debug,ops,drawer,cmdk,presets}.css` + per-page bundles, mirrored in `static/js/`.

### Skills layer (9 native Claude Code skills under `.claude/skills/`)

All skills migrated from `skills/` to `.claude/skills/` in session 9 for native Skill-tool auto-discovery via YAML frontmatter with engineered triggers and negative-trigger clauses. Each SKILL.md was then iterated through skill-creator: evidence-anchored mandatory-read tables (replacing exhortation framing), What-I-Did-Not-Do declarations between workflow steps, Tier 3 quality checklists. Inventory:

| Skill | Purpose |
|---|---|
| `populate-from-lifeos` | One-way sync `profile/` from LifeOS canonical source via the `Capataina/Capataina` README allow-list; never writes to LifeOS; never touches `preferences.toml` or `portfolio-gaps.md`. Replaces retired `profile-scrape`. |
| `discover-companies` | Profile-aware company discovery with parallel sector agents and creative non-obvious sources. |
| `populate-db` | Validate discovered companies, run `cernio resolve`, AI fallback for failures, write to DB. |
| `resolve-portals` | AI fallback for companies that fail script-based ATS resolution. |
| `search-jobs` | Orchestrate full search cycle: `cernio search` (script half) + parallel bespoke subagents (AI half). Insert-obligation-anchored. |
| `grade-companies` | Enrich + grade companies S/A/B/C with profile-grounded reasoning. |
| `grade-jobs` | Grade jobs SS-F with question-first rubric, calibration anchors, lifestyle modulator, realism semantic (reputation × selectivity decoupling). |
| `check-integrity` | AI-driven re-evaluation; portfolio-gap maintenance; cross-tier consistency. |
| `prepare-applications` | Generate tailored application answers per job; store JSON in `application_packages`. |

Total skill documentation ~290KB — nearly as large as the Rust source code (~494KB). The reference documentation IS the quality bar; without it, agents default to generic "good company, decent tech" output. Every skill enforces the mandatory-read protocol: SKILL.md → every reference file → all of `profile/` fresh on every invocation.

### Profile system (post-session-10 schema)

Each pipeline invocation and skill reads `profile/` fresh — the profile is the lens through which every grading happens. Schema after session-10's `populate-from-lifeos` migration: direct-copy files from LifeOS `Profile/Professional/` (`personal.md`, `education.md`, `experience.md`, `interests.md`, `visa.md`, `languages.md`, `certifications.md`, `military.md`, `lifestyle-preferences.md`, `resume.md`, `cover-letter.md`); synthesised per-project files in `profile/projects/<name>.md` (one per allow-listed project, each comprehensive evidence-anchored synthesis of its LifeOS source); aggregated `projects/open-source-contributions.md`; derived `skills.md` (six tables, four bands) generated by a skills-derivation subagent reading the synthesised projects; navigation `projects/index.md`; per-run `sync-summary.md` audit artefact. Cernio-native files (`preferences.toml`, `portfolio-gaps.md`) are explicitly off-limits to the sync — they remain Cernio's source of truth because they are runtime config and skill output, not profile data.

### Location-evaluation subsystem (session 8)

Not a scoring formula — a reasoning framework. 22-factor three-tier rubric (Tier 1 dominant, Tier 2 meaningful, Tier 3 fine-tuning) evaluating cities at city/country/hybrid level across current state and trajectory horizons (1-3 / 5-7 / 10-15 years). Tier 1 includes visa accessibility for a Turkish national at entry level, target-firm density in chosen sectors, urban aesthetic match, safety and civic order, political/legal 10-15 year stability. 10-agent parallel research pass produced `context/references/location-master.md` (71KB synthesis) plus per-agent files (~6,500 lines combined). Two headline conclusions: London #1 by unanimous agreement, "Amsterdam rejected" in prior profile was overturned unanimously. The `profile/lifestyle-preferences.md` (17.5KB) is read alongside the main profile as a same-tier grading modulator: Kings Cross / Nine Elms-class areas lift boundary grades, Croydon-class areas push them down. This is grade movement across boundaries, not within-grade tiebreaking.

### Testing infrastructure (346 tests; 18 at session 7)

Six architectural decisions shape every test: lib+bin split, `CERNIO_DB_PATH` env var, `test_support::open_in_memory_db()` workhorse fixture (returns fresh in-memory SQLite with all migrations applied), inline tests for private pure functions / integration tests for public flows + CLI, offline JSON fixtures over HTTP mocking, TUI tested by state not by rendering. 327 net new tests added in seven phases across sessions 9-10. Highest test concentration: `format.rs` 85 inline (idempotency), `config.rs` 31, `resolve.rs` 30 (`slug_candidates` regression-proofed against 13 real Cernio companies), `schema.rs` 29, ATS modules 72, `cli.rs` 16, `preferences_integrity.rs` 21 (build-time invariants on `preferences.toml` shape including `every_supported_ats_provider_has_a_location_subtable` driven off a `SUPPORTED_ATS_PROVIDERS` constant). The test pass surfaced three silent bugs that had been live in production: two data-loss bugs in session 9 (commit `12897aa`), the silent Workday UK-filter bypass in session 10 (`86097a6` — Workday's `[search_filters.locations.workday]` subtable was missing since the fetcher shipped), and the timestamp format mismatch breaking Shift+D archive in session 11 (`50359b13` — inserts used `%Y-%m-%dT%H:%M:%S` while SQLite emits `%Y-%m-%d %H:%M:%S` with a space, not T; 7 files patched).

### Autofill (Chrome CDP, scaffolded but broken — #1 known gap)

Architecture is in place: Chrome launches via `chromiumoxide`, navigates to job URL, `application_packages` DB table works, TUI `p` key spawns autofill, yellow ● indicator shows packages-ready, package auto-cleanup on applied. Broken at form filling: JS `el.value = "..."` does not trigger Greenhouse's React controlled component state. Fix path is documented: replace with CDP `Input.insertText` or `nativeInputValueSetter`, then verify CSS selectors against real Greenhouse DOM (currently written from documentation, not inspection), then add Lever and Ashby modules.

## Technologies and concepts demonstrated

### Languages

- **Rust (edition 2024)** — primary and only language for the entire pipeline, TUI, web layer, ATS fetchers, autofill scaffold. ~14k lines across 56 source files. Lib+bin split (`src/lib.rs` library crate + thin `src/main.rs` shim) is load-bearing for integration testability.

### Frameworks and libraries

- **`rusqlite`** (bundled) — SQLite access; no system SQLite dependency. WAL mode set on every open. `test_support::open_in_memory_db()` is the workhorse integration-test fixture.
- **Tokio** — async runtime for pipeline scripts (resolve, search, clean, check). Parallel ATS fetches via `fetch_all_parallel` with a Tokio semaphore for concurrency control.
- **Reqwest with retry helpers** — shared HTTP client in `src/http.rs`, per-request exponential backoff (500ms × attempt), used by every ATS module via `common::get_with_retry` and `post_json_with_retry`.
- **Serde + serde_json** — JSON deserialisation for ATS responses; per-provider types in each `src/ats/<provider>.rs` module.
- **`toml = "0.8"`** — `preferences.toml` → typed config structs via `src/config.rs`. Loader is intentionally lenient (typos → defaults + stderr warning), guarded by 21 build-time integrity tests.
- **Ratatui 0.29 + Crossterm** — TUI rendering and terminal/event handling. Responsive layout via custom `widgets/layout.rs::distribute()`.
- **`chromiumoxide`** (Chrome DevTools Protocol) — used by both the broken autofill scaffold and the working `cernio snap` debug-screenshot CLI.
- **axum** — embedded web server on localhost:7878 (added 2026-05-30).
- **maud** — type-safe HTML templates; gotcha discovered: `selected=(false)` emits `selected="false"` and browsers treat any presence as truthy. Correct conditional-attribute syntax is `selected[bool_expr]` with brackets.
- **tower-http** — middleware for the web layer.
- **HTMX 1.9** — inline writes from the web UI; re-processed inside detail drawer body so Apply/Watch/Reject continue working post-swap.
- **ECharts 5.4** — charts in the web UI; data delivered via JSON islands `<script type="application/json" id="data-<kind>">` written by handlers via `json_island(kind, &value)`.
- **Testing crates** — `cargo test`, `assert_cmd` (CLI subprocess testing against tempdir DBs), `proptest`, `tempfile`, `predicates`.

### Runtimes / engines / platforms

- **SQLite (WAL mode)** — single-file local-first datastore; supports concurrent TUI reads while pipeline scripts write. Five tables, six idempotent migrations, the `companies.website` and `jobs.url` UNIQUE constraints are the dedup mechanism across all layers.

### Tools

- **`gh` CLI** — cross-vault runtime dependency added in session 10 for `populate-from-lifeos`. Reads `Capataina/LifeOS` and `Capataina/Capataina` README; one-way flow.
- **Native Claude Code skills** at `.claude/skills/` — 9 skills, all skill-creator-audited, YAML triggers with negative-trigger clauses, evidence-anchored mandatory-read tables, What-I-Did-Not-Do declarations between workflow steps.
- **`cernio snap` CLI** — self-contained visual-debug tool driving headless Chrome over the web UI and writing per-pane PNGs.

### Domains and concepts

- **Local-first architecture with zero infrastructure.** SQLite file, no Docker, no server, no API keys for the core system. AI layer runs through Claude Code sessions — no hosted inference.
- **Scripts-for-volume / AI-for-judgment partitioning.** A Rust script can probe 5,000 ATS slug candidates in seconds; Claude can read 50 resulting job descriptions and assess fit against a nuanced profile. Neither could do the other's job economically; the entire architecture is built around this asymmetry.
- **Three-layer architecture with SQLite as shared contract.** Conversation → scripts → DB ← TUI/web. Strictly downward dependencies; no layer reads upward.
- **Idempotency as a structural property.** Every pipeline command safe to re-run; format `format(format(x)) == format(x)` is a tested invariant because the function runs silently on every TUI/web startup.
- **Living-System Philosophy.** Profile, grades, preferences, and ATS slugs all change over time. No grade is permanently settled; the `check-integrity` skill compares profile modification dates against `graded_at` timestamps for staleness. Skills must never embed profile snapshots — every skill reads `profile/` fresh on every invocation. Hardcoded profile data goes stale silently and causes grading errors that are difficult to detect.
- **Archival over deletion.** Companies and jobs are archived rather than deleted to preserve grading history, prevent re-discovery overhead, and allow reversibility via `cernio unarchive`. Refined on 2026-05-14: zero-signal Bucket 3 dead-entity rows (no jobs, no portals, no decisions) get hard-deleted; rows with accumulated signal still archive.
- **Question-first grading rubric, evolved across five phases.** Dimension-weighted scoring → hard floors → career-stage calibration → question-first reasoning → realism semantic (reputation × selectivity decoupling). Each rewrite was driven by a concrete production failure: Amazon at B (Phase 1 inflation), Monzo at C (Phase 1 deflation), Thought Machine at SS with 3-5 year requirements (Phase 3 title-only-reading), 40 SS/S after clean sweep with prestige leakage (Phase 4 → Phase 5).
- **Reputation-vs-selectivity decoupling (realism semantic).** A reputable name on a CV is a Q2 (CV-value) signal; it says nothing on its own about Q1 (realistic achievability). Wide-funnel reputable firms (Amazon SDE-1, Bloomberg, Cloudflare interns, Anthropic Fellows, HRT 2026 Grad SWE, Squarepoint Graduate) anchor SS; narrow-funnel reputable firms (Jane Street, Citadel, Anthropic London non-Fellows) cap at A-stretch despite identical Q2. Production validation: 12% S+ density vs the >20% pre-realism inflation; Jane Street pattern confirmed (18 roles → 0 SS, 0 S, 4 A-stretch).
- **Lane-based-relativity grading (2026-05-29 refactor).** Eight active lanes (`big-tech`, `ai-ml`, `hft`, `crypto-mm`, `bank-strats`, `systems-infra`, `devtools`, `fintech`). Jobs graded SS-F within their primary lane, not on a single global axis. Cross-lane decisions happen at user-decision level, not rubric level. No hardcoded calibration anchors — calibration emerges from within-lane comparison during the grade-companies / grade-jobs Phase 2 relativity pass.
- **Sponsor-only universe.** Every company must sponsor UK Skilled Worker visas; `sponsors_uk` mandatory verified-yes for retention; non-sponsors rejected at discovery.
- **Calibration-anchored grading (vs batch-relative).** Pull 2-3 real examples per tier from the DB before grading begins; grade each job against those anchors. Replaces batch-relative deflation when prioritisation skews batches.
- **Mandatory description citation.** Fit assessments must quote specific phrases from the job description; "entry-accessible" without citing what the description actually says about seniority requirements is banned.
- **F12/F15 script-obligation asymmetry pattern.** A skill whose workflow does work a script could have done first silently burns tokens and produces inferior output. Three skills got mandatory step-0 script calls in commit `bee129a` (`resolve-portals` → `cernio resolve`; `grade-jobs` → `cernio format`; `prepare-applications` → `cernio format`).
- **Inter-system contract surfacing.** Session-9 architecture rewrite formalised the cross-system contracts that "break loudest when violated" (e.g. ATS provider name is a shared string across 4 places; `ats_extra` JSON is provider-specific and unversioned; `preferences.toml` re-read at every pipeline invocation but cached for the TUI session). Hidden coupling is documented explicitly so nobody is surprised.
- **Obligation-anchored over exhortation-anchored skills.** Vague "be thorough" / "carefully check" framing gets sycophantically absorbed; verifiable obligations ("produce artefact X", "quote the last line of each reference") cannot be satisficed without producing visibly-incomplete output.
- **Self-driven visual verification.** On web UI work, the agent runs `cernio snap` itself, reads PNGs, and iterates rather than asking the user to verify visually.

## Key technical decisions

**Collaborative, not automated.** Original README described daily `cernio refresh`; revised in first session. Every feature must support a conversational workflow; scripts are parameterised tools invoked by Claude during a session, not cron jobs.

**Scripts for volume, AI for judgment.** Fundamental architectural split. Rust scripts must be generic, reusable, parameterised. Intelligence lives in the conversation, not in the scripts.

**SQLite as single source of truth.** Evaluated against markdown files, JSON/JSONL, and Postgres/MySQL. SQLite won on zero ops, single file, full SQL, WAL for concurrency, trivial backup. Profile data stays in markdown (human-edited); companies/jobs/decisions/packages live in SQLite (machine-managed); discovery results land in markdown first then migrate via import.

**Lib+bin split for testability (session 9).** Rust integration tests under `tests/` can only see public items from a library crate, not a binary-only crate. The split was the smallest change that unblocked `tests/cli.rs` (via `assert_cmd`), `tests/pipeline_*.rs` (via `test_support::open_in_memory_db`), and the 16-test CLI suite. Alternatives considered: keep binary-only and accept no integration tests; move logic into a separate sibling crate.

**`CERNIO_DB_PATH` env var (session 9).** Smallest possible change to make CLI integration tests viable. Alternatives: `--db-path` CLI flag (every test passes it); hardcoded path with symlinking; sqlite in-memory via special mode. The env var means tests spawn real binaries via `assert_cmd::Command::cargo_bin("cernio")` and get isolated DBs without filesystem contention.

**Question-first grading over dimension-weighted scoring.** Four iterations of the rubric driven by production failures; dimension-weighted scoring produced mechanical generic assessments because agents assigned middling scores to everything and arrived at B without reasoning. Questions force genuine reasoning; dimensions become analytical support.

**False negatives are the enemy.** At every filter stage, bias toward inclusion. Empty data → include. A job with no location passes the location filter; a job that doesn't match exclusion keywords passes even with no inclusion match. False negatives are unrecoverable; false positives cost 30 seconds to grade as F.

**Mandatory-read protocol for all skills.** Added session 3 after agents skipped reference files and produced shallow output. Every skill agent must read SKILL.md, all references/, all of profile/. The reference documentation IS the quality bar.

**TUI grade as primary metric, not evaluation_status.** `evaluation_status` is just a coarser bucketing of `grade`; the TUI displays grade only. Future possibility: split into role-quality and accessibility as separate dimensions.

**Per-provider location patterns, not global.** Location formats differ dramatically across providers (Greenhouse "Hybrid" or "Berlin; London", SmartRecruiters server-side `?country=gb`). Location patterns live per-provider in `preferences.toml`.

**Lifestyle fit as same-tier grading modulator (session 8).** Not Tier 3 tiebreaker. Aesthetic-daily-environment compounds over years in a way pay or tax bracket do not; under-weighting it for neatness was the wrong trade-off. Borderline A/B role in Kings Cross lifts to A; same role in outer Croydon drops to B. Grades move across boundaries, not just within-grade.

**Native Claude Code skills at `.claude/skills/` (session 9).** Migrated from `skills/` for Skill-tool auto-discovery, YAML engineered triggers with negative-trigger clauses, `/skill-name` slash completion. The legacy pattern required remembering to invoke; the native pattern surfaces skills automatically when triggers match.

**Obligation-anchored over exhortation-anchored (session 9).** Replace "be thorough" / "carefully check" with verifiable obligations. Research-backed finding: RLHF absorbs exhortation sycophantically — agents produce the appearance of thoroughness while skipping actual work. Falsifiable checklist items cannot be satisficed without producing visibly-incomplete output.

**LifeOS as canonical, Cernio as consumer (session 10).** Made `Capataina/LifeOS` the canonical source of truth for profile data. Alternatives rejected: (a) keep maintaining the duplication manually — drift was already happening; (b) make Cernio canonical — LifeOS's structural role is broader than career data; (c) symlink Cernio's `profile/` to LifeOS — Cernio is public, LifeOS is private, the symlink would expose the vault.

**README-as-gatekeeper for project sync (session 10).** The `Capataina/Capataina` GitHub README's *Active* / *Other* / *Open Source Contributions* sections are the allow-list for which projects appear in `profile/projects/`. Private Projects section excluded by design. Adding a new project to the public-facing portfolio requires editing the README first, then re-syncing — a deliberate boundary, not a technical limitation.

**Status-based project weighting replaces Tier system (session 10).** Retired Flagship / Notable / Minor labels. Once the profile was split into per-project files, every file became its own canonical evidence — there is no longer a need to assign a tier, because file depth, content, and `status` frontmatter carry the same signal more honestly.

**Realism semantic — reputation × selectivity decoupling (session 11, commit `389b1e8a`).** Phase 5 of the grading rubric. When reasoning about Q1 (achievability), ignore Q2's (CV-value) signal entirely. The detection rule prevents prestige-trap inflation. Alternatives rejected: tighten Phase 4 wording (failure was systematic, not isolated); hard "no SS at firms with intake < X" rules (too rigid — would penalise wide-funnel reputable firms).

**Hard-delete zero-signal Bucket 3 companies (2026-05-14, refines archival doctrine).** Archive-over-deletion still stands for rows with accumulated signal; rows that reach Bucket 3 with no jobs, no portals, no user_decisions get hard-deleted. 16 rows deleted on 2026-05-14 met the zero-signal condition. The DB should reflect the active universe of viable applications, not be a graveyard.

**S-band calibration anchors required in grade-jobs agent prompts (2026-05-17).** Without an S-tier worked example, 7 Opus agents with the full rubric all produced 0 S grades from a corpus where 12 roles clearly qualified. Canonical worked S-tier examples (Graphcore Cambridge SWE, GSR Quant Developer Rust) must be embedded before grading starts.

**Lane-based-relativity grading (2026-05-29).** Companies and jobs graded within-lane rather than on a single global SS/S/A/B/C/F scale. Eight lanes carry equal status during junior phase — no lane priority weighting. Strategy A (prestige exit then independent contracting at £1.5k-£3k/day) drives lane selection.

**Role-truth-at-hire — function locked at day 1 (2026-05-29).** Role function (engineering / quant / research / strats) must already be destination function at hire. Vertical progression within function is normal; cross-function lateral hops are auto-downgraded (Solutions Architect hoping to become SWE, IBD Analyst hoping to lateral, etc.).

**Sponsor-only universe (2026-05-29).** Every company must sponsor UK Skilled Worker visas. Non-sponsors rejected at discovery, never enter the DB. `sponsors_uk` mandatory verified-yes for retention.

**No hardcoded calibration anchors (2026-05-29).** Calibration emerges from within-lane comparison during Phase 2 of grade-companies / grade-jobs, not from a hardcoded "Anthropic is SS in AI/ML" list. Hardcoded anchors decay silently as company positioning shifts.

**Web frontend modularised into shared + per-page bundles (2026-05-30).** `static/style.css` + `static/app.js` were monolithic and grew too large to edit reliably. Split via `PageAssets::css_js` system in `src/web/templates.rs::page_with`.

**Filter chips unified with type-based variants (2026-05-30).** Yes/no shouldn't be the same shape as 8 lanes. Coloured multi-select chips for axes (lane, grade); segmented controls for binary/ternary toggles (archive, sponsor, has-jobs).

## What is currently built

The Rust core, the database, the TUI, and the web UI are all production-shaped and operationally proven. ~14k lines of Rust across 56 source files. 346 tests passing (273 inline + 73 integration including the 21 preferences-integrity guards). 6 ATS provider fetchers (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday) plus Eightfold recorded as bespoke. 6 pipeline scripts (resolve, search, clean, check, import, format) plus stats/unarchive/pending. 9 native Claude Code skills, all skill-creator-audited.

The TUI v5 is fully operational across 5 views (Dashboard, Companies, Jobs, Pipeline kanban, Activity timeline) with mouse support, responsive layout across 3 modes, GitHub-style activity heatmap, semantic colour theme, focus mode, smart grouping, multi-select, export to markdown, grade-override picker, and a session welcome diff.

The web UI added 2026-05-30 is also operational across 5 tabs over the same SQLite DB: chip-filter strips with type-based variants, clickable charts that cross-filter via URL navigation, detail drawer with URL persistence and HTMX re-processing, Cmd-K command palette with substring + prefix + grade ranking, g-leader keyboard shortcuts, LocalStorage saved searches capped at 30, ops menu (Clean + Format only), and a `cernio snap` CLI that drives headless Chrome over all 5 tabs and writes per-pane PNGs for self-driven visual debug.

Database carries ~456 companies (318 resolved / 138 bespoke at the 2026-05-13 snapshot; expanded to ~706 companies + ~1,068 lane-tagged jobs in the 2026-05-29 lane-based-relativity refactor and sponsor-only universe scoping) and ~1,370 graded jobs. Grade distribution (combined post-realism + 2026-05-10 incremental): ~19 SS / ~31 S / ~108 A / ~84 B / ~254 C / ~854 F.

The `populate-from-lifeos` skill has shipped and run end-to-end twice. First run synced 11 Professional/ files, synthesised 12 per-project files in parallel (203 LifeOS source files consumed, 3,413 lines of synthesised content), produced 1 aggregated OSS file, derived `skills.md` (six tables, four bands), wrote navigation `index.md`, and produced a `sync-summary.md` audit artefact. Second-run idempotency confirmed.

The realism-semantic grade-jobs rewrite (Phase 5 of the rubric, commit `389b1e8a`) is in production. The lane-based-relativity refactor (2026-05-29, commit `0c9f296`) and sponsor-only universe scoping are in production.

Code-health audit (session 9, commit `c7973e0`) surfaced 27 open findings across 8 systems — 4 high-severity (the four `strip_html` divergences with a latent Workable correctness bug, N+1 query in `pipeline::search::run_by_grade`, `fetch_stats` 16-query-per-2s-poll, SmartRecruiters pagination missing retry). The audit modified zero production code; it added 6 parity tests in `tests/ats_strip_html_parity.rs` that lock the target semantics for the strip_html consolidation. All 27 findings are still open at the LifeOS snapshot.

## Current state

Status: active. Most recent meaningful work captured in LifeOS is the 2026-05-30 web-frontend redesign (5 commits) and the 2026-05-29 lane-based-relativity refactor with sponsor-only universe scoping (706 companies + 1068 jobs lane-tagged, prepare-applications skill deleted, all 7 deferred follow-ups closed). The Cernio repo is local-first and well-tested; the TUI and web UI both ship working; the AI layer is fully operational across 9 skills. In-flight work captured in LifeOS `Projects/Cernio/Work/`: drawer + Cmd-K + g-leader + presets interactive browser audit; prepare-applications batch run on the 12 SS+S list from 2026-05-10; Cloud / Kubernetes / Docker / Terraform deployment as portfolio-gap closer using Cernio itself; periodic vault refresh.

## Gaps and known limitations

**Autofill form filling broken (Priority 1).** Architecture in place — Chrome launches, navigates, `application_packages` table works, TUI `p` key works — but JS `el.value = ...` does not trigger React's controlled component state on Greenhouse forms. Fix path is documented (CDP `Input.insertText` or `nativeInputValueSetter`), but until it lands every application is manual copy-paste from prepared drafts.

**Cloud / Kubernetes / Docker / Terraform / CI-CD portfolio evidence.** The densest gap evidence in the project. Confirmed across 5+ consecutive grading batches as the #1 employability gap. 2026-05-10 batch flagged it across 14 separate roles. The closure prescription is concrete and explicitly uses Cernio itself: a weekend on containerising the Rust binary, adding a GitHub Actions CI workflow, deploying a preview to AWS Lambda/Fargate, adding a Terraform module.

**C++ proficiency (Familiar → Proficient).** Primary blocker on 7+ roles in 2026-04-29 batch alone (Apple JDK, Apple Kafka, Citadel C++ SWE, Tower Quant Developer, QRT Low Latency Market Data, Wayve Robot Software, Wintermute C++ Quant). Self-rated "Familiar"; Nyquestro and Tectra demonstrate the concepts in Rust but the Rust-to-C++ translation is undemonstrated.

**Cybersecurity / cloud-security portfolio bridge.** Wiz roles cap at C-or-below; portfolio absence (no CTF, no CVE, no security-tooling project) is the blocker, not visa.

**CUDA / GPU-systems / PTX / SASS / CUTLASS / Triton / NCCL.** Distinct from "production-scale ML" — this is GPU-kernel-engineering. Currently absent from skills.md.

**Production-scale ML.** NeuroDrive is M2-MacBook-Air scale; A-vs-S boundary at Apple AiDP, Jane Street ML, DRW ML where 10K-GPU / petabyte scale is the divider.

**Distributed-database tenure** (YugabyteDB / CockroachDB / TiDB / Cloud Spanner / Iceberg / Trino). Single-node SQLite work doesn't transfer.

**OCaml.** Jane Street primary language. 8+ Jane Street roles in 2026-04-29 batch involve OCaml.

**2:2-degree-class structural credential filter.** Non-closable. Luminance Cambridge and similar smaller-firm Top-200-University + First-or-2:1 filters categorically reject. Tracked as risk, no technical closure available.

**27 open code-health findings.** 4 HIGH-severity (strip_html consolidation + Workable latent bug; N+1 in search::run_by_grade; fetch_stats 16-query-per-poll; SmartRecruiters pagination retry). None implemented yet.

**Eightfold ATS fetcher not built.** Only 1 company uses it; ROI low; tracked but deprioritised.

**Teamtailor ATS fetcher not built.** Higher ROI than Eightfold — 4 of the 17 dad-list bespoke companies are on Teamtailor with a clean public API at `{slug}.teamtailor.com/jobs.json`.

**Parent-company slug expansion not in `cernio resolve`.** AI fallback surfaced `LexisNexis → workday/relx` and DigitalOcean → `greenhouse/digitalocean98` (numeric suffix). Mechanical resolver did not try parent-company path; falls through to AI fallback every time.

**Workday integration is the most complex.** Variable subdomain + site name means resolution requires manual identification or AI fallback. 20 companies on Workday. Workday UK-filter bypass closed in session 10 (commit `86097a6`) but the resolver still has no mechanical Workday probing.

**Dashboard is the largest single file** at 31.5KB / 946 lines. Flagged in the code-health audit for split.

**Web frontend drawer + Cmd-K + g-leader + presets PNG-audited but not interactively browser-tested.** Action item: 30-min focused session driving everything via keyboard and clicking through drawer flows.

**Activity event log empty most of the time** in the web UI — backfill/migration/raw.* triggers filtered out; real CLI/TUI/web activity is naturally sparse.

## Direction (in-flight, not wishlist)

**Apply to the wide-funnel SS/S targets (time-bound, Priority 1).** ~15 deep-customisation primary targets across the realism-semantic SS/S anchor cluster: Anthropic Fellows (next intake), HRT 2026 Grad SWE, Microsoft UK Graduate SWE + MAIDAP, Bloomberg 2026 SWE + Internship, Cloudflare grad-track interns × 4, Amazon SDE-1 New Grad 2026 UK, Apple London ASE pipeline (the strongest wide-funnel SS/S generator — 7 of 23 Apple roles in the 2026-05-10 batch landed SS/S), Arm AI/ML Cambridge Graduate SWE, B2C2 Graduate Quant Developer London, Graphcore Cambridge Drivers, Squarepoint Graduate, Stripe SWE Intern London, Palantir SWE New Grad, Tradeweb 2026 Tech Grad, Vocalink (Mastercard) Launch Graduate Program 2026.

**Close the Cloud / Kubernetes / Docker / Terraform / CI-CD gap (Priority 2).** Single weekend using Cernio itself as the closer — containerise the Rust binary, add CI workflow, deploy `cernio search` preview to AWS Lambda or Fargate, add Terraform module. Highest-leverage portfolio investment available right now per the 2026-05-10 batch evidence.

**Fix autofill (Priority 3).** CDP `Input.insertText` swap, verify Greenhouse CSS selectors against real DOM, then add Lever and Ashby modules.

**Interview-prep skill (Priority 5).** Design exists in full at `context/notes/interview-prep-design.md`. Reads SS/S/A jobs + portfolio gaps to identify what to study; generates concept files, LeetCode-style TDD problems, multi-component systems practice, company-specific study briefs.

**Code-health audit implementation batches (Priority 7).** 7-batch sequence: dead-code removal (unblocks ATS); strip_html consolidation (fixes Workable latent bug, removes 70 lines); SQL consolidation in `fetch_stats` + N+1 fix in `search::run_by_grade`; retry standardisation across Ashby/Workable/Workday; `verify_ats_slugs` parallelisation + Lever probe swap; dashboard split; `fetch_jobs` list/detail split. Each batch independently testable against the 346-test baseline.

**Tighten search-time filter on disguised non-engineering titles (Priority 8).** Add "Forward Deployed", "Deployed Engineer", "Solutions Architect", "Solutions Engineer" as title-pattern hard-excludes; tighten "Analyst" include_keyword to require "Quant", "Quantitative", "Research", or "Software" pairing.

**Filter hardware / RTL / ASIC / FPGA roles at search time (Priority 9).** Extend `preferences.toml exclude_keywords` with `RTL`, `ASIC`, `FPGA`, `VLSI`, `Physical Design`, `Mechanical`, `Nanofabrication`, `Hardware Integration`, `Optical`, `Emulation`, `Aviation`, `Maintenance`. ~10-15% grading-load reduction with zero false-negative risk.

**Web frontend interactive audit and follow-ups.** Interactive browser audit of drawer + Cmd-K + g-leader + presets. Future HTMX-partial-swap for heatmap cross-filter to avoid page reload. Optional server-side sync for saved-searches presets.

## Demonstrated skills

This project demonstrates that the candidate can:

- **Architect and ship a multi-layer Rust system end-to-end** — library + binary, CLI pipeline, terminal UI, embedded web server, AI orchestration layer, all over a single SQLite store, with strict downward dependency direction and clean inter-layer contracts.
- **Design and integrate against six independent third-party JSON APIs** (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday) with provider-specific quirks (server-side filters, pagination shapes, dual US/EU endpoints, false-200 detection, variable subdomain+site, POST-based search). Shared retry layer with exponential backoff, unified `AtsJob` type, offline JSON fixtures as living documentation.
- **Build a 26-file modular terminal UI** in Ratatui with mouse support, responsive layout across 3 width modes, semantic colour palette, GitHub-style activity heatmap, kanban view with proportional column sizing, multi-select, smart grouping, and a quick-peek floating popup pattern.
- **Build an embedded server-rendered web interface** in axum + maud + HTMX + ECharts over the same SQLite store, with type-based filter chip variants, URL-persistent detail drawer, clickable charts that cross-filter via URL navigation, Cmd-K command palette with substring + prefix + grade ranking, g-leader keyboard shortcuts, LocalStorage saved searches.
- **Run a Rust integration-test pass from 18 to 346 tests** spanning inline unit tests, integration tests against in-memory SQLite, CLI subprocess testing via `assert_cmd` against tempdir DBs, parity tests, and 21 build-time invariants on `preferences.toml` shape. Surfaced three silent production bugs (two data-loss in session 9, Workday UK-filter bypass in session 10, timestamp-format mismatch in session 11) directly via the test investment.
- **Decide and execute a lib+bin split for testability** as a structural change — `main.rs` becomes a thin shim, `lib.rs` exports the library surface, integration tests can import public items.
- **Author and iterate native Claude Code skills** with YAML engineered triggers, evidence-anchored mandatory-read tables, What-I-Did-Not-Do declarations, Tier 3 quality checklists, and ~290KB of reference documentation that IS the quality bar. Skills are obligation-anchored not exhortation-anchored — verifiable artefact production replaces "be thorough" framing.
- **Design and ship a five-phase grading rubric** with concrete production-driven evolution: dimension-weighted → hard floors → career-stage calibration → question-first → realism semantic (reputation × selectivity decoupling). Each phase rewrite traceable to specific production failure data; the realism semantic produced 12% S+ density vs >20% pre-realism inflation, with the Jane Street prestige-trap pattern confirmed across 18 roles.
- **Design a 22-factor three-tier location-evaluation reasoning framework** with city/country/hybrid level evaluation across current state plus three trajectory horizons (1-3 / 5-7 / 10-15 years). Ran a 10-agent parallel research pass producing 71KB of synthesis; integrated lifestyle preferences as a same-tier grading modulator that moves grades across boundaries.
- **Design and ship a one-way cross-repository profile sync** (`populate-from-lifeos`): 8-phase workflow, autonomous start-to-finish, three subagent dispatches (parallel per-project + skills derivation), README-as-gatekeeper allow-list, pre/post-timestamp verification that aborts on Cernio-native file mutation, audit artefact written per run. First run consumed 203 LifeOS source files and produced 3,413 lines of synthesised content; second-run idempotency confirmed.
- **Author a self-driven visual-debug CLI** (`cernio snap`) that spawns the embedded axum server, drives headless Chrome via chromiumoxide, captures full-page + per-pane + viewport-slice PNGs into a timestamped directory, optionally with `--temporal` 3s-gap diffs for animation/state debugging. Used by the agent itself rather than by the user.
- **Run a code-health audit** with 27 actionable findings across 8 systems (4 HIGH-severity), backed by 6 added parity tests that lock target semantics, and 0 production code modified — every finding is a proposed change with evidence chain and effort estimate.
- **Apply Living-System Philosophy enforcement** — grades, preferences, ATS slugs all change over time; no skill embeds profile snapshots; every skill reads `profile/` fresh on every invocation; profile mutations trigger staleness audits via `check-integrity`.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Cernio/_Overview.md | 127 | "> Session 8 added the 22-factor location-evaluation rubric + lifestyle modulator; session 9 added 316 tests (surfacing two silent data-loss bugs), a full code-health audit with 27 findings, and migrated all 9 skills to native Claude Code integration. Session 10 shipped `populate-from-lifeos` + retired `profile-scrape` + added a 21-test `preferences_integrity.rs` guard that surfaced the silently-bypassed Workday UK-location filter. Sessions 11-12 ran the post-clean-slate full re-grading under the new realism semantic (583 jobs in two 2026-04-29 batches) plus a 164-job 2026-05-10 search-and-grade batch. Velocity slowed because depth was the goal. See [[Cernio/Session History]] for the full breakdown." |
| Projects/Cernio/Architecture.md | 343 | "`selected=(false)` in maud emits `selected=\"false\"` — browsers treat any presence of the `selected` attribute as truthy, so the last `<option>` in the source order wins regardless of which was meant to be selected. **Correct conditional-attribute syntax is `selected[bool_expr]` (brackets, not parens).** Same applies to `checked`, `disabled`, etc. Detected this session when filter dropdowns reverted to the last value (fintech) after switching to \"All\". See `context/notes/maud-attribute-gotchas.md`." |
| Projects/Cernio/Data Composition.md | 205 | "- [[Cernio/Roadmap]] — apply targets and filter-tightening priorities surfaced by the 2026-04-29 + 2026-05-10 batches" |
| Projects/Cernio/Decisions.md | 413 | "**Consequence:** Future Cernio UI work uses snap as the verification loop. The image-reader hits per-pane PNGs at native resolution when full-page PNGs are too tall. Saved as a permanent feedback memory: [[feedback_self_driven_visual_verification]]." |
| Projects/Cernio/Gaps.md | 296 | "How they got that way: likely a write that updated `status` to `resolved`/`bespoke` without the companion INSERT/UPDATE on `company_portals`/`careers_url`. Worth adding a Phase-0 invariant check to `populate-db` (cited as a \"Cov\" proposal in the 2026-05-14 populate-db skill log)." |
| Projects/Cernio/Roadmap.md | 196 | "- [[Cernio/Data Composition]] — current grading state and batch composition" |
| Projects/Cernio/Session History.md | 217 | "> 5th major rubric rewrite, driven by user observation that prestige was leaking into Q2-confirmed SS. The reputation × selectivity decoupling produced calibrated grades on 583 jobs across two same-day batches (12% S+ density vs the 20%+ inflation pre-realism) and the Jane Street prestige-trap pattern was concretely confirmed in production data. Every future grade-jobs run inherits the calibration. `[verified: commit 389b1e8a, portfolio-gaps.md §Batch 2026-04-29 batch 1]`" |
| Projects/Cernio/Systems/_Overview.md | 49 | "- [[Projects/Cernio/Roadmap]] — direction-of-travel" |
| Projects/Cernio/Systems/ATS Providers.md | 150 | "- [[Cernio/Systems/Code Health]] — 7 findings open against this subsystem" |
| Projects/Cernio/Systems/Autofill.md | 104 | "- [[Cernio/Gaps]] — autofill is the #1 gap" |
| Projects/Cernio/Systems/Code Health.md | 158 | "- [[Cernio/Roadmap]] — implementation batches are queued" |
| Projects/Cernio/Systems/Config.md | 80 | "- [[Cernio/Architecture]] — no hardcoded configuration is a key architectural property" |
| Projects/Cernio/Systems/Database.md | 185 | "- [[Cernio/Systems/Code Health]] — dashboard `fetch_stats` issues 16 queries per 2s poll; SQL consolidation is a HIGH-severity audit finding" |
| Projects/Cernio/Systems/Grading.md | 204 | "- [[Cernio/Decisions#Realism semantic 2026-04-29]] — the design decision behind phase 5" |
| Projects/Cernio/Systems/Location Evaluation.md | 151 | "- LifeOS canonical: `Profile/Professional/Lifestyle Preferences.md` — Cernio's `profile/lifestyle-preferences.md` is synced from here one-way via populate-from-lifeos (session 10)" |
| Projects/Cernio/Systems/Pipeline.md | 176 | "- [[Cernio/Systems/Code Health]] — 10 open findings in this subsystem" |
| Projects/Cernio/Systems/Profile.md | 158 | "- [[Cernio/Session History#Session 10]] — the migration session" |
| Projects/Cernio/Systems/Skills.md | 191 | "- [[Cernio/Session History#Session 10]]" |
| Projects/Cernio/Systems/TUI.md | 201 | "- [[Cernio/Systems/Testing]] — Phase 6 added 34 TUI helper tests" |
| Projects/Cernio/Systems/Testing.md | 207 | "- [[Cernio/Session History#Session 11]] — timestamp format mismatch bug fixed across 7 files" |
| Projects/Cernio/Systems/Web.md | 141 | "- `context/notes/maud-attribute-gotchas.md` — `selected[bool]` syntax + browser truthy-attribute lessons" |
| Projects/Cernio/Work/Application Pipeline.md | 61 | "- Cernio _Overview / Gaps / Roadmap drift (last_verified 2026-04-24) — see `[[Projects/Cernio/Work/Vault Refresh.md]]`" |
| Projects/Cernio/Work/Cloud Deployment.md | 66 | "- Related: prepare-applications follow-up on the 12 SS+S list — `[[Projects/Cernio/Work/Application Pipeline.md]]`" |
| Projects/Cernio/Work/Profile Populate Skill.md | 197 | "- LifeOS commit `cf14e1d` — Phase 1 landing commit" |
| Projects/Cernio/Work/Vault Refresh.md | 61 | "Orient on 2026-05-10 flagged this as a drift but the session that ran orient (Cernio session start) went on to search-jobs → grade-jobs → exhaustion-of-day pattern, then session wrap deferred this hygiene cut. Cheap-pass items get deferred routinely; the persistence-pin pattern would surface it in morning-brew anti-rec walk." |
| Projects/Cernio/Work/Web Frontend.md | 39 | "- [project_web_frontend_redesign.md](../../.claude/projects/-Users-atacanercetinkaya-Documents-Programming-Projects-cernio/memory/project_web_frontend_redesign.md) — auto-memory snapshot" |
