# Architecture

> Last updated: 2026-04-09 (session 4). Massive universe expansion — 273 active companies (167 resolved, 83 potential, 23 bespoke), grade distribution 25S/107A/94B/47C. 712 jobs in DB (12 SS, 55 S). Grading rubrics completely rewritten to question-first approach with mandatory description citation. Cleanup no longer auto-archives C companies (archive_company_grades = [], min_company_grade = "C"). Lever EU probing added. TUI v4 unchanged. 6 ATS fetchers, 5 pipeline scripts, 8 skills.

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
| Database | SQLite via `rusqlite` (bundled, WAL mode) | Implemented — schema, 2 migrations, 11 tests |
| Date handling | `chrono` | In use |
| Async runtime | Tokio | In use — pipeline scripts (resolve, search, clean, check) |
| HTTP | Reqwest | In use — ATS API calls across 6 providers (incl. Workday) |
| Serialisation | Serde | In use — JSON (ATS responses), TOML (config) |
| Config parsing | `toml = "0.8"` | In use — `preferences.toml` → typed config structs |
| TUI | Ratatui 0.29 + Crossterm backend | v4 implemented — 4 views (dashboard, companies, jobs, pipeline), multi-select, search/filter, sort, export, mouse support, responsive layout, widget refactor |
| AI layer | Claude Code skills (conversational invocation) | 8 skills with mandatory-read blocks enforced on all skills. check-integrity has 3 reference files (remediation-guide, quality-standards, profile-context) |

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
│   ├── pipeline/
│   │   ├── mod.rs              # Pipeline module exports
│   │   ├── resolve.rs          # cernio resolve — slug probing, multi-provider
│   │   ├── search.rs           # cernio search — fetch → filter → insert
│   │   ├── clean.rs            # cernio clean — job removal, company archival
│   │   ├── check.rs            # cernio check — integrity report
│   │   └── import.rs           # cernio import — bulk import from external sources
│   └── tui/
│       ├── mod.rs              # Terminal setup/teardown, event loop
│       ├── app.rs              # App state, View/Focus enums, data models, navigation, multi-select, search, sort
│       ├── handler.rs          # Key event dispatch — global + view/focus-specific + mouse handling
│       ├── theme.rs            # Semantic ANSI colour palette
│       ├── queries.rs          # DB read queries (companies, jobs, stats, top matches, pipeline)
│       ├── views/
│       │   ├── mod.rs          # Draw dispatcher, tabs, status bar, help overlay, search bar, grade picker, toast
│       │   ├── dashboard.rs    # Stats overview — dynamic sizing, session summary, scrollable top roles
│       │   ├── companies.rs    # Company table + detail with full job list, grade bars, sort
│       │   ├── jobs.rs         # Job table + detail with full descriptions, description indicator
│       │   └── pipeline.rs     # Kanban/pipeline view — 3 columns (Watching/Applied/Interview), card rendering
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
│   ├── search-jobs/SKILL.md      # Legacy — job search moved to script
│   ├── grade-companies/
│   │   ├── SKILL.md
│   │   └── references/          # grading-rubric.md, profile-context.md
│   ├── grade-jobs/
│   │   ├── SKILL.md
│   │   └── references/          # grading-rubric.md, profile-context.md, prioritisation-guide.md
│   └── check-integrity/
│       ├── SKILL.md             # AI-driven re-evaluation and grade quality auditing
│       └── references/
│           ├── remediation-guide.md
│           ├── quality-standards.md
│           └── profile-context.md
├── state/
│   └── cernio.db               # SQLite database (gitignored)
├── AgentCreationResearch/      # Skill authoring research (reference)
├── context/                    # Project memory
├── Cargo.toml                  # rusqlite, chrono, tokio, reqwest, serde, toml
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
| `search-jobs` | Legacy | Original job search skill — search logic moved to `cernio search` script |
| `check-integrity` | Designed | AI-driven re-evaluation and grade quality auditing |

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

**End of session 4 (2026-04-09).** Universe expanded from 79 to 273+ companies across multiple discovery and population rounds. Grading rubrics for both companies and jobs completely rewritten — question-first approach replacing dimension-weighted scoring, with mandatory description citation in fit assessments. Cleanup policy revised: C companies no longer auto-archived (kept active because job grading handles quality filtering), min_company_grade lowered to "C" to search all non-archived companies. Lever EU domain probing added. Per-request retry logic in HTTP client. TUI v4 unchanged.

| Component | Status |
|-----------|--------|
| Profile (12 files) | Fully populated, tested profile-scrape against NeuroDrive |
| SQLite schema | 4 tables, 3 migrations (MIGRATION_001 base, MIGRATION_002 archived status, MIGRATION_003 job archival), 11 tests passing, WAL mode |
| Config (`src/config.rs`) | TOML parser for `preferences.toml` — search filters, cleanup config, location patterns |
| ATS fetchers (`src/ats/`) | 6 providers: Lever, Greenhouse, Ashby, Workable, SmartRecruiters, Workday. Common trait + dispatch |
| Pipeline: resolve (`src/pipeline/resolve.rs`) | Slug candidate generation, parallel multi-provider probing, SmartRecruiters false positive handling |
| Pipeline: search (`src/pipeline/search.rs`) | Fetch → location filter → exclusion filter → inclusion filter → dedup → insert |
| Pipeline: clean (`src/pipeline/clean.rs`) | Job removal (F/C grades, stale). Company archival disabled by default (archive_company_grades = []) — C companies kept active for job search coverage |
| Pipeline: check (`src/pipeline/check.rs`) | ATS re-verification, stale detection, completeness, dead URLs, duplicates, profile-change, structured report |
| profile-scrape skill | Designed, tested on NeuroDrive |
| discover-companies skill | Designed with search-strategies reference, first run produced 73 companies |
| populate-db skill | Designed with company grading rubric and ATS docs for 7 providers |
| search-jobs skill | Legacy — job search moved to `cernio search` script |
| check-integrity skill | New — AI-driven re-evaluation and grade quality auditing, 3 reference files (remediation-guide, quality-standards, profile-context) |
| resolve-portals skill | Redesigned — AI fallback only for companies that fail script resolution |
| grade-companies skill | New — extensive SKILL.md, grading rubric with worked examples, profile context reference |
| grade-jobs skill | New — extensive SKILL.md, grading rubric, profile context, prioritisation guide |
| Company universe | 273 active companies (167 resolved, 83 potential, 23 bespoke), grade distribution 25S/107A/94B/47C |
| Jobs | 712 jobs in DB (12 SS, 55 S, 90 A, 130 B), full pipeline run complete with parallel search and grading |
| TUI (`src/tui/`, 14 files) | v4 implemented — 4 views (dashboard, companies, jobs, pipeline), mouse support, multi-select, search/filter, sort, grade override, export, responsive layout, widget refactor. See `context/systems/tui.md` |
| Export | Implemented — `e` key exports current view to `exports/YYYY-MM-DD-*.md` |

**Next priorities:**
1. Resolve remaining 83 potential companies
2. Search and grade jobs from newly resolved companies
3. Interview prep skill design and implementation
4. Portfolio gap analysis from grading patterns
5. Further discovery rounds to expand universe beyond 273
