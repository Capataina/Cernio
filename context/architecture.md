# Architecture

> Last updated: 2026-04-07. Database schema implemented, three skills designed, profile populated. Pre-production — no job search or TUI yet.

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
| Database | SQLite via `rusqlite` (bundled, WAL mode) | Implemented — schema, migrations, 7 tests |
| Date handling | `chrono` | Added |
| Async runtime | Tokio | Planned (for search scripts) |
| HTTP | Reqwest | Planned (for ATS API calls) |
| Serialisation | Serde | Planned (JSON, TOML, JSONL) |
| TUI | Ratatui | Planned |
| AI layer | Claude Code skills (conversational invocation) | 3 skills designed |

---

## Repository Structure

```text
cernio/
├── src/
│   ├── main.rs                 # Entry point, opens DB
│   └── db/
│       ├── mod.rs              # Public DB interface
│       └── schema.rs           # Migration, schema, tests
├── profile/                    # Structured personal profile (read every startup)
│   ├── personal.md             # Name, contact, links
│   ├── education.md            # Degrees, modules
│   ├── experience.md           # Work history
│   ├── projects.md             # Full project inventory (16 projects)
│   ├── skills.md               # Languages, frameworks, domains
│   ├── preferences.toml        # Hard and soft job filters
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
│   └── populate-db/SKILL.md
├── state/
│   └── cernio.db               # SQLite database (gitignored)
├── AgentCreationResearch/      # Skill authoring research (reference)
├── context/                    # Project memory
├── Cargo.toml                  # rusqlite + chrono
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

Skills live in `skills/` within this repo, not in the upstream `agent-skills` framework. They are project-specific.

### Rust Scripts (Planned)

Scripts are generic, stateless, and parameterised. They take inputs, produce outputs, and exit.

| Script | Inputs | Output | Status |
|--------|--------|--------|--------|
| `search` | ATS slugs, search terms, provider | Matching jobs as structured results | Not started |
| `resolve` | Company names, ATS URL patterns | Slug matches with verification | Not started |
| `export` | Result set, format options | Markdown file with tables and links | Not started |

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

**Early implementation.** Database schema is implemented and tested. Three skills are designed. Profile is fully populated. No production pipeline code yet.

| Component | Status |
|-----------|--------|
| Profile (12 files) | Populated with real data, tested profile-scrape against NeuroDrive |
| SQLite schema | Implemented — companies, jobs, user_decisions tables with migrations and 7 tests |
| profile-scrape skill | Designed and tested |
| discover-companies skill | Designed with search-strategies reference |
| populate-db skill | Designed with full ATS endpoint documentation |
| Company universe | Empty — discovery not yet run |
| Search scripts | Not started |
| TUI | Not started |
| Job evaluation | Not started |
