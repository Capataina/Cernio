# Architecture

> Last updated: 2026-04-09 (session 7). 408 companies, 1184 jobs (484 graded non-archived, 0 pending). Profile scrape rewrote projects.md (honest tiers: Nyquestro→Notable, Aurix→Notable), resume.md, cover-letter.md. Job search: 255 new + 13 bespoke (Citadel, Bloomberg, Google, Apple, Arm). Full grading run: 13 SS, 27 S, 70 A, 142 B, 20 C, 212 F. Added Sr./Lead exclusion keywords (51 jobs archived). TUI v5: major modularisation (app.rs→6 files, handler.rs→4 files, views/mod.rs→3 files), 5 views (new Activity Timeline tab), dashboard overhaul (heatmap, search pulse, visa countdown, top companies leaderboard), quick-peek popup, breadcrumb nav, smart grouping, contextual status bar. 26 TUI source files, 7 ATS fetchers, 6 pipeline scripts, 9 skills, 6 DB migrations.

---

## Scope / Purpose

A local-first, collaborative job discovery and curation engine. The user and Claude work together in conversational sessions to find, evaluate, and curate job opportunities from a personally built universe of UK and remote-UK technology employers.

Cernio is not an automated pipeline. Every action happens in a collaborative session where the user and Claude decide together what to do.

---

## Repository Overview

### Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   Conversational Session                         │
│                   (User + Claude Code)                           │
│                                                                  │
│  • Decide what to do: discover, populate, search, evaluate      │
│  • Claude orchestrates skills/scripts, evaluates results        │
│  • User makes all application decisions                         │
└──────────┬──────────────────────────────────┬───────────────────┘
           │ invokes                          │ writes evaluations
           ▼                                  ▼
┌─────────────────────────┐    ┌──────────────────────────────────┐
│    Rust Scripts          │    │      SQLite (state/cernio.db)    │
│    (parameterised tools) │    │                                  │
│                          │───►│  companies ── lifecycle:         │
│  • search: scan ATS      │    │    potential → resolved/bespoke  │
│    boards for terms      │    │  jobs ── evaluation lifecycle:   │
│  • resolve: probe slugs  │    │    pending → evaluating → fit    │
│  • export: generate md   │    │  user_decisions ── tracking:     │
│                          │    │    watching / applied / rejected  │
└──────────────────────────┘    └──────────────┬──────────────────┘
                                               │ watches
                                               ▼
                                ┌──────────────────────────────────┐
                                │         Ratatui TUI               │
                                │         (live dashboard)          │
                                │                                   │
                                │  • Company universe browser       │
                                │  • Live evaluation status         │
                                │  • User confirms/rejects/exports  │
                                └──────────────────────────────────┘
```

### Technology Stack

| Component | Choice | Status |
|-----------|--------|--------|
| Core language | Rust (edition 2024) | In use |
| Database | SQLite via `rusqlite` (bundled, WAL mode) | Implemented — schema, 6 migrations, 11 tests |
| Date handling | `chrono` | In use |
| Async runtime | Tokio | In use — pipeline scripts (resolve, search, clean, check) |
| HTTP | Reqwest | In use — ATS API calls across 6 providers (incl. Workday) |
| Serialisation | Serde | In use — JSON (ATS responses), TOML (config) |
| Config parsing | `toml = "0.8"` | In use — `preferences.toml` → typed config structs |
| TUI | Ratatui 0.29 + Crossterm backend | v5 implemented — 5 views (dashboard, companies, jobs, pipeline, activity), modular architecture (26 source files), heatmap, search pulse, visa countdown, quick-peek, breadcrumbs, smart grouping, contextual status bar, session timer |
| Browser automation | `chromiumoxide` (Chrome CDP) + `futures` | Scaffolded — Chrome launches, navigates; form filling broken on React forms |
| AI layer | Claude Code skills (conversational invocation) | 9 skills with mandatory-read blocks enforced on all skills. check-integrity has 3 reference files (remediation-guide, quality-standards, profile-context) |

---

## Repository Structure

```text
cernio/
├── src/
│   ├── main.rs                 # Entry point, CLI dispatch (resolve/search/clean/check/import/tui)
│   ├── config.rs               # TOML config parser (preferences.toml → typed structs)
│   ├── http.rs                  # Shared HTTP client with retry logic and rate limiting
│   ├── db/
│   │   ├── mod.rs              # Public DB interface
│   │   └── schema.rs           # Migration (001 + 002), schema, 11 tests
│   ├── ats/
│   │   ├── mod.rs              # ATS provider trait, provider dispatch
│   │   ├── common.rs           # Shared types (AtsJob, slug normalisation)
│   │   ├── lever.rs            # Lever API fetcher
│   │   ├── greenhouse.rs       # Greenhouse API fetcher
│   │   ├── ashby.rs            # Ashby API fetcher
│   │   ├── workable.rs         # Workable API fetcher
│   │   ├── smartrecruiters.rs  # SmartRecruiters API fetcher (totalFound>0 check)
│   │   └── workday.rs          # Workday API fetcher (variable subdomain + site)
│   ├── autofill/
│   │   ├── mod.rs              # ApplicantProfile, provider dispatch, package JSON parsing
│   │   ├── common.rs           # Chrome launch via chromiumoxide CDP, field filling helpers
│   │   └── greenhouse.rs       # Greenhouse-specific form selectors (broken — React state issue)
│   ├── pipeline/
│   │   ├── mod.rs              # Pipeline module exports
│   │   ├── resolve.rs          # cernio resolve — slug probing, multi-provider
│   │   ├── search.rs           # cernio search — fetch → filter → insert
│   │   ├── clean.rs            # cernio clean — job removal, company archival
│   │   ├── check.rs            # cernio check — integrity report
│   │   ├── import.rs           # cernio import — bulk import from external sources
│   │   └── format.rs           # cernio format — HTML/entity-encoded descriptions → clean plaintext
│   └── tui/                    # 26 source files, v5
│       ├── mod.rs              # Terminal setup/teardown, event loop
│       ├── app/                # App state (modularised from single app.rs)
│       │   ├── mod.rs          # App struct, new(), refresh(), entry point
│       │   ├── state.rs        # View/Focus/SortMode enums, App fields, ActivityEntry
│       │   ├── navigation.rs   # View switching, selection, drill-down, scroll
│       │   ├── actions.rs      # User actions (decisions, export, URL open, grade override)
│       │   ├── pipeline.rs     # Pipeline-specific state management
│       │   └── cleanup.rs      # Database cleanup confirmation and execution
│       ├── handler/            # Event handling (modularised from single handler.rs)
│       │   ├── mod.rs          # Top-level event dispatch
│       │   ├── navigation.rs   # Key-based navigation (j/k/g/G/Tab/1-5)
│       │   ├── overlays.rs     # Help, grade picker, search bar input
│       │   └── mouse.rs        # Click, scroll, multi-select mouse events
│       ├── theme.rs            # Semantic ANSI colour palette + freshness/activity/badge styles
│       ├── queries.rs          # DB read queries (companies, jobs, stats, pipeline, activity, heatmap)
│       ├── views/
│       │   ├── mod.rs          # Draw dispatcher, view routing
│       │   ├── chrome.rs       # Tab bar, status bar, breadcrumbs, session timer
│       │   ├── overlays.rs     # Help overlay, search bar, grade picker, toast, quick-peek popup
│       │   ├── dashboard.rs    # Stats, heatmap, search pulse, visa countdown, top companies, session diff
│       │   ├── companies.rs    # Company table + detail with full job list, grade bars, sort
│       │   ├── jobs.rs         # Job table + detail, description indicator, new badges, smart grouping
│       │   ├── pipeline.rs     # Kanban view — 3 columns, one-line cards, scrollbar, spinners
│       │   └── activity.rs     # Activity Timeline tab (5th view)
│       └── widgets/
│           ├── mod.rs
│           ├── grade_bar.rs    # Proportional grade bar (reused in dashboard + company detail)
│           ├── text_utils.rs   # HTML cleanup, relative dates, truncation (shared across views)
│           ├── toast.rs        # Toast notification rendering
│           └── layout.rs       # Dynamic layout system with distribute() function
├── profile/                    # Structured personal profile (read every startup)
│   ├── personal.md             # Name, contact, links
│   ├── education.md            # Degrees, modules
│   ├── experience.md           # Work history
│   ├── projects.md             # Full project inventory (16 projects)
│   ├── skills.md               # Languages, frameworks, domains
│   ├── preferences.toml        # Search filters, cleanup config, location patterns
│   ├── visa.md                 # Right to work, sponsorship needs
│   ├── portfolio-gaps.md       # Strengths, gaps, closure opportunities
│   ├── resume.md               # LaTeX resume source
│   └── ...                     # certifications, languages, interests, etc.
├── companies/
│   └── potential.md            # Discovery landing zone (pre-DB)
├── skills/
│   ├── profile-scrape/SKILL.md
│   ├── discover-companies/
│   │   ├── SKILL.md
│   │   └── references/search-strategies.md
│   ├── populate-db/SKILL.md
│   ├── resolve-portals/SKILL.md  # Redesigned — AI fallback only
│   ├── search-jobs/SKILL.md      # Orchestrates cernio search + bespoke-company pass
│   ├── grade-companies/
│   │   ├── SKILL.md
│   │   └── references/          # grading-rubric.md, profile-context.md
│   ├── grade-jobs/
│   │   ├── SKILL.md
│   │   └── references/          # grading-rubric.md, profile-context.md, prioritisation-guide.md
│   ├── check-integrity/
│   │   ├── SKILL.md             # AI-driven re-evaluation and grade quality auditing
│   │   └── references/
│   │       ├── remediation-guide.md
│   │       ├── quality-standards.md
│   │       └── profile-context.md
│   └── prepare-applications/
│       └── SKILL.md             # Generate tailored application answers per job
├── state/
│   └── cernio.db               # SQLite database (gitignored)
├── AgentCreationResearch/      # Skill authoring research (reference)
├── context/                    # Project memory
├── Cargo.toml                  # rusqlite, chrono, tokio, reqwest, serde, toml, chromiumoxide, futures
├── claude.md                   # Session configuration
└── README.md                   # Project vision and milestones
```

---

## Subsystem Responsibilities

### Layer Responsibilities

| Layer | Does | Does not |
|-------|------|----------|
| **Conversation** | Orchestrates skills and scripts, evaluates jobs against profile, recommends actions, tracks portfolio gaps | Submit applications, make decisions without user input |
| **Rust scripts** | Combinatorial volume: scan ATS boards, probe slug patterns, fetch job JSON, generate exports | Make judgments, know about the profile, decide what to search for |
| **TUI** | Real-time display of company universe, evaluation progress, and user decisions; markdown export on keypress | Run scripts, evaluate jobs, or modify data independently |
| **SQLite** | Contract between all layers — single source of truth for structured data | Contain business logic |

### AI Layer — Claude Code Skills

Skills handle slow, fuzzy, infrequent work that requires reasoning. Invoked conversationally ("run a discovery"), not via CLI syntax.

| Skill | Status | Purpose |
|-------|--------|---------|
| `profile-scrape` | Designed, tested | Scrape GitHub repos and update profile with evidence-based entries |
| `discover-companies` | Designed | Profile-aware company discovery with parallel sector agents and creative search strategies |
| `populate-db` | Designed | Research companies from discovery, find ATS slugs, verify endpoints, insert into SQLite |
| `resolve-portals` | Redesigned | AI fallback for companies that fail script-based ATS resolution |
| `grade-companies` | Designed | Grade ungraded companies (S/A/B/C) with extensive rubric and profile context |
| `grade-jobs` | Designed | Grade ungraded jobs (SS/S/A/B/C/F) with smart prioritisation by company grade × title signal |
| `search-jobs` | Designed | Orchestrates the full search cycle — runs `cernio search` for resolved-ATS companies, dispatches parallel subagents to search the 121 bespoke companies (Apple, Google, Meta, Citadel, etc.) via careers pages + aggregators, inserts every found role via `INSERT OR IGNORE INTO jobs`, hands pending queue to `grade-jobs` |
| `check-integrity` | Designed | AI-driven re-evaluation and grade quality auditing (runs `cernio format` as step 2) |
| `prepare-applications` | Designed | Generate tailored application answers per job, stored in `application_packages` table |

Skills live in `skills/` within this repo, not in the upstream `agent-skills` framework. They are project-specific.

### Rust Scripts (Pipeline)

Scripts handle all mechanical volume work. Each has a single purpose and takes no judgment decisions.

| Script | CLI | Purpose | Status |
|--------|-----|---------|--------|
| `resolve` | `cernio resolve [--company] [--dry-run]` | Probe ATS slug candidates across 5 providers, insert portals | Implemented |
| `search` | `cernio search [--company] [--grade] [--dry-run]` | Fetch jobs → location/exclusion/inclusion filter → dedup → insert | Implemented |
| `clean` | `cernio clean [--dry-run] [--jobs-only]` | Remove F/C jobs, stale jobs, archive C companies | Implemented |
| `check` | `cernio check [--ats-only] [--fix]` | Integrity report: health, completeness, staleness, recommendations | Implemented |
| `import` | `cernio import <file>` | Bulk import companies from external sources (CSV/JSON) | Implemented |
| `format` | `cernio format` | Convert raw HTML/entity-encoded descriptions to clean plaintext. Idempotent. Runs on TUI startup via `run_silent()`. | Implemented |
| `export` | — | Markdown export of curated results | Not started |

### Data Layer

SQLite (`state/cernio.db`) is the single source of truth for all structured data.

#### Schema

```
companies
  id, name, website (UNIQUE), what_they_do, discovery_source,
  discovered_at, status (potential/resolved/bespoke),
  location, sector_tags, ats_provider, ats_slug, ats_extra,
  ats_verified_at, careers_url, why_relevant, relevance_updated_at

jobs
  id, company_id → companies, title, url (UNIQUE), location,
  remote_policy, posted_date, raw_description, parsed_tags,
  evaluation_status (pending/evaluating/strong_fit/weak_fit/no_fit),
  fit_assessment, fit_score, discovered_at

user_decisions
  id, job_id → jobs, decision (watching/applied/rejected),
  decided_at, notes

application_packages
  job_id → jobs (PRIMARY KEY), answers (JSON), created_at
```

#### Field Categories

| Category | Fields | Staleness trigger |
|----------|--------|-------------------|
| **Facts** (stable) | name, website, what_they_do, discovery_source, discovered_at | Company pivots or rebrands |
| **Checkpoints** (verify periodically) | ats_provider, ats_slug, ats_extra, ats_verified_at, careers_url, status, location, sector_tags | Company switches ATS, relocates, or pivots |
| **Judgments** (tied to profile) | why_relevant, relevance_updated_at | Profile changes — new projects, shifted interests |

Continuously changing metrics (headcount, funding, ratings) are deliberately excluded. Look them up live during evaluation, don't cache stale guesses.

#### Supported ATS Providers

| Provider | API Pattern | Slug Discovery |
|----------|-------------|----------------|
| Greenhouse | `boards-api.greenhouse.io/v1/boards/{slug}/jobs` | Simple slug probe |
| Lever | `api.lever.co/v0/postings/{slug}` | Simple slug probe |
| Ashby | `api.ashbyhq.com/posting-api/job-board/{slug}` | Simple slug probe |
| Workable | `apply.workable.com/api/v1/widget/accounts/{slug}` | Simple slug probe |
| SmartRecruiters | `api.smartrecruiters.com/v1/companies/{slug}/postings` | Simple slug probe |
| Workday | `{company}.{wd1-12}.myworkdayjobs.com/wday/cxs/{company}/{site}/jobs` | Complex — variable subdomain and site name, stored in `ats_extra` |
| Eightfold.ai | `{subdomain}/api/apply/v2/jobs?domain={domain}` | Complex — company-specific subdomain, stored in `ats_extra` |

Companies on iCIMS, Taleo, BambooHR, Jobvite, Personio, or custom portals → `bespoke` with careers URL preserved.

#### What Lives Where

| Store | Contains |
|-------|----------|
| **SQLite** | Companies (full lifecycle), jobs, evaluations, user decisions |
| **Markdown** | `profile/` (human-edited), `companies/potential.md` (discovery landing zone before DB migration), `exports/` (generated on demand) |

---

## Dependency Direction

```
┌─────────────────────────────────────────────────────────────────┐
│                   Conversational Session                         │
│                   (User + Claude Code)                           │
│                                                                  │
│  • Decide what to do: discover, populate, search, evaluate      │
│  • Claude orchestrates skills/scripts, evaluates results        │
│  • User makes all application decisions                         │
└──────────┬──────────────────────────────────┬───────────────────┘
           │ invokes                          │ writes evaluations
           ▼                                  ▼
┌─────────────────────────┐    ┌──────────────────────────────────┐
│    Rust Scripts          │    │      SQLite (state/cernio.db)    │
│    (parameterised tools) │    │                                  │
│                          │───►│  companies ── lifecycle:         │
│  • search: scan ATS      │    │    potential → resolved/bespoke  │
│    boards for terms      │    │  jobs ── evaluation lifecycle:   │
│  • resolve: probe slugs  │    │    pending → evaluating → fit    │
│  • export: generate md   │    │  user_decisions ── tracking:     │
│                          │    │    watching / applied / rejected  │
└──────────────────────────┘    └──────────────┬──────────────────┘
                                               │ watches
                                               ▼
                                ┌──────────────────────────────────┐
                                │         Ratatui TUI               │
                                │         (live dashboard)          │
                                │                                   │
                                │  • Company universe browser       │
                                │  • Live evaluation status         │
                                │  • User confirms/rejects/exports  │
                                └──────────────────────────────────┘
```

The conversation layer sits at the top and drives everything. It invokes Rust scripts downward and writes evaluations into SQLite. Rust scripts write results into SQLite but never read the profile or make judgments. The TUI watches SQLite for changes but never modifies data independently — user actions in the TUI (confirm, reject, export) write back through SQLite. All layers depend on SQLite as the shared contract; no layer depends upward.

---

## Core Execution / Data Flow

```
1. Session starts
   └─► Claude reads profile/, context/, README.md
   └─► User and Claude discuss what to do

2. Profile update (when projects or skills have changed)
   └─► User provides a GitHub repo link
   └─► profile-scrape skill reads repo (context/, README, code)
   └─► Updates projects.md, skills.md, portfolio-gaps.md

3. Discovery (when the universe needs expanding)
   └─► discover-companies skill dispatches parallel sector agents
   └─► Each agent searches its territory creatively
   └─► Orchestrator deduplicates, user reviews
   └─► Accepted companies land in companies/potential.md

4. Population (after discovery produces new companies)
   └─► populate-db skill researches each company
   └─► Deterministic slug probing (parallelisable, scriptable)
   └─► Web search fallback for unmatched (Claude judgment)
   └─► Companies inserted into SQLite as resolved or bespoke
   └─► TUI shows progress in real time

5. Job search (collaborative decision)
   └─► User and Claude decide what to search for
   └─► Claude invokes Rust search script with params
   └─► Results written to SQLite
   └─► TUI shows results appearing

6. Evaluation (Claude)
   └─► Claude reads each job description
   └─► Compares against profile
   └─► Writes evaluation to SQLite
   └─► TUI updates: pending → evaluating → fit/no fit
   └─► Tracks market patterns → portfolio-gaps.md

7. Review and export
   └─► User reviews in TUI, marks watching/applied/rejected
   └─► Presses export key → markdown report generated

8. Session wrap-up
   └─► User reports what they applied to
   └─► Claude updates state, suggests next focus
```

---

## Structural Notes / Current Reality

**End of session 7 (2026-04-09).** Profile scrape found major discrepancies — Nyquestro only has type system implemented, Aurix only Tab 1. projects.md rewritten with honest entries, resume.md and cover-letter.md rebuilt. Nyquestro downgraded from Flagship to Notable, Aurix from Flagship to Notable. Job search: 255 new jobs from `cernio search` + 13 bespoke from S/A-tier companies (Apple, Citadel, Bloomberg, Google, Arm, Mastercard, Amazon). Full grading run: 13 SS, 27 S, 70 A, 142 B, 20 C, 212 F. Added "Sr.", "Sr ", "Lead" to exclusion keywords — 51 leaked senior jobs archived. 39 bespoke companies marked as searched. DB: 408 companies, 1184 jobs (484 graded non-archived), 0 pending.

**TUI v5 overhaul (session 7):** Major modularisation — `app.rs` (1,112 lines) split into `app/` directory (6 files), `handler.rs` (490 lines) into `handler/` (4 files), `views/mod.rs` (438 lines) into 3 files (mod.rs, chrome.rs, overlays.rs). New 5th view: Activity Timeline. Dashboard enhancements: GitHub-style activity heatmap, search pulse with freshness colouring, application progress bar, visa countdown with urgency colours, top companies leaderboard, session welcome diff. View enhancements: one-line kanban cards, viewport scrolling with scrollbars, animated spinners (◐◑◒◓), fit assessment structured rendering (Q1-Q5), quick-peek popup (Space), breadcrumb navigation, "New" badges (magenta ●), smart job grouping (Ctrl+G). Chrome: contextual status bar, coloured tab badges, session timer, URL preview, decision history indicator, enhanced help overlay (6 sections). Filter fixes: focus mode (f) hides F/C + applied, D key archives F jobs immediately.

| Component | Status |
|-----------|--------|
| Profile (15 files) | Fully populated. Session 7: profile-scrape found major discrepancies, projects.md rewritten with honest entries (Nyquestro Notable, Aurix Notable), resume.md rebuilt (verified with tectonic), cover-letter.md rebuilt. Project tiers (Flagship/Notable/Minor). Portfolio-gaps.md actively maintained. |
| SQLite schema | 5 tables, 6 migrations (001 base, 002 archived status, 003 job archival, 004 last_searched_at, 005 archived_at, 006 application_packages), 18 tests passing, WAL mode |
| Config (`src/config.rs`) | TOML parser for `preferences.toml` — search filters, cleanup config, location patterns |
| ATS fetchers (`src/ats/`) | 6 providers: Lever, Greenhouse, Ashby, Workable, SmartRecruiters, Workday. All fetchers use `get_with_retry`. Attribute-aware HTML stripping. |
| Pipeline: resolve (`src/pipeline/resolve.rs`) | Expanded slug generator (punctuation stripping, domain suffixes, acronyms, first-two-words). No early termination — probes all providers for all slugs. SmartRecruiters probed for all companies. |
| Pipeline: search (`src/pipeline/search.rs`) | Fetch → location filter → exclusion filter → inclusion filter → dedup → insert. Retry on empty results. Sets `last_searched_at` per company. |
| Pipeline: clean (`src/pipeline/clean.rs`) | Tiered archival: SS=28d, S=21d, A=14d, B=7d, C/F=3d. Archive expires after 14 days (tracked via `archived_at`). No company auto-archival. |
| Pipeline: check (`src/pipeline/check.rs`) | ATS re-verification, stale detection, completeness, dead URLs, duplicates, profile-change, structured report |
| Pipeline: format (`src/pipeline/format.rs`) | Converts raw HTML/entity-encoded descriptions to clean plaintext. Idempotent. Runs on TUI startup via `run_silent()`. Also step 2 in check-integrity skill. |
| Pipeline: import (`src/pipeline/import.rs`) | Bulk import from discovery files. Supports `--file` flag. Dedup via website URL unique constraint. |
| profile-scrape skill | Designed, tested on NeuroDrive |
| discover-companies skill | 9-agent discovery run produced 228 raw companies (161 new after dedup). Agents write to individual files. |
| populate-db skill | Designed with company grading rubric and ATS docs for 7 providers |
| search-jobs skill | Orchestrates the full search cycle — runs `cernio search` for resolved-ATS companies (287), dispatches parallel subagents to search the 121 bespoke companies via careers pages + aggregators (LinkedIn / Indeed / Glassdoor / BuiltIn), inserts via `INSERT OR IGNORE INTO jobs`, hands to `grade-jobs` |
| check-integrity skill | AI-driven re-evaluation, cross-checking guide (4 reference files), active portfolio gap maintenance (10 jobs per grade tier). Step 2 runs `cernio format`. |
| resolve-portals skill | AI fallback for companies that fail script resolution |
| prepare-applications skill | Generate tailored application answers per job. Reads profile + job description + fit assessment, stores JSON answers in `application_packages` table. |
| Autofill (`src/autofill/`) | Scaffolded. Chrome CDP via `chromiumoxide`. Provider dispatch (Greenhouse first). TUI `p` key triggers. **Broken:** JS value injection doesn't trigger React state. Needs nativeInputValueSetter or CDP Input.insertText. |
| grade-companies skill | Enriches + grades: writes `what_they_do` (3-5 sentences), `location`, `sector_tags`, `grade`, `grade_reasoning`, `why_relevant`. Calibration-anchored grading. |
| grade-jobs skill | Question-first rubric, project tier awareness, calibration-anchored grading, mandatory description citation, prioritisation guide |
| Company universe | 408 total (287 resolved, 121 bespoke, 0 potential). ATS: 114 Greenhouse, 70 Ashby, 31 Workable, 26 Lever, 20 Workday, 8 SmartRecruiters, 1 Eightfold. Every company has a known path to job discovery. |
| Jobs | 1184 jobs (484 graded non-archived, 0 pending). Full grading: 13 SS, 27 S, 70 A, 142 B, 20 C, 212 F. Every SS/S assessment is multi-paragraph with description citations. |
| TUI (`src/tui/`, 26 files) | v5 — 5 views (dashboard, companies, jobs, pipeline, activity). Modularised: app/ (6 files), handler/ (4 files), views/ (8 files), widgets/ (5 files) + mod.rs, queries.rs, theme.rs. Dashboard: heatmap, search pulse, visa countdown, top companies, session diff. Views: one-line kanban cards, scrollbars, spinners, quick-peek (Space), breadcrumbs, new badges, smart grouping (Ctrl+G). Chrome: contextual status bar, session timer, coloured tab badges, URL preview, decision history, enhanced help (6 sections). Focus mode hides F/C + applied. D archives F immediately. |
| Export | Implemented — `e` key exports current view to `exports/YYYY-MM-DD-*.md` |
| Unarchive | `cernio unarchive --jobs [--grade G]` restores archived jobs with timer reset |

**Next priorities:**
1. Fix autofill React form filling (nativeInputValueSetter or CDP Input.insertText)
2. Interview prep skill design and implementation
3. Integrity check after session 7 grading
4. Add Lever and Ashby autofill modules once Greenhouse works
5. Next job search cycle (periodic re-search of resolved companies)
