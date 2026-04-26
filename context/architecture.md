# Architecture

> **Last updated:** 2026-04-26, session 10 (current upkeep). Testing now 346 tests (273 inline + 73 integration), up from 316 — the new 21 are the `tests/preferences_integrity.rs` safety net guarding `profile/preferences.toml` against silent loader-fallback corruption (commit `86097a6`). Profile schema migration completed end-to-end: `profile-scrape` retired, `populate-from-lifeos` added and successfully ran for the first time, producing 12 per-project files in `profile/projects/`, an aggregated `open-source-contributions.md`, a derived `skills.md`, and the per-run audit artefact `sync-summary.md`. Skills count steady at 9. Autofill remains scaffolded-but-broken (Chrome launches; React form filling unresolved).

---

## Scope / Purpose

A local-first, collaborative job discovery and curation engine. The user and Claude work together in conversational sessions to find, evaluate, and curate job opportunities from a personally built universe of UK and remote-UK technology employers.

Cernio is not an automated pipeline. Every action happens in a collaborative session where the user and Claude decide together what to do.

---

## Repository Overview

### Three-layer architecture

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
│    Rust CLI (`cernio`)   │    │      SQLite (state/cernio.db)    │
│    parameterised tools   │    │                                  │
│                          │───►│  companies ── lifecycle:         │
│  resolve / search /      │    │    potential → resolved/bespoke  │
│  clean / check /         │    │  company_portals ── 1:N per co   │
│  format / import         │    │  jobs ── evaluation lifecycle:   │
│                          │    │    pending → evaluating → fit    │
└──────────────────────────┘    │  user_decisions ── tracking:     │
                                │    watching / applied / rejected  │
                                │  application_packages ── autofill │
                                └──────────────┬──────────────────┘
                                               │ watches
                                               ▼
                                ┌──────────────────────────────────┐
                                │         Ratatui TUI               │
                                │         (live dashboard)          │
                                │                                   │
                                │  5 views, 26 source files, v5     │
                                │  Dashboard · Companies · Jobs ·   │
                                │  Pipeline · Activity Timeline     │
                                └──────────────────────────────────┘
```

The conversation layer invokes scripts and skills. Rust scripts write to SQLite. The TUI watches SQLite and writes user decisions back. No layer depends upward.

### Technology stack

| Component | Choice | Status |
|-----------|--------|--------|
| Core language | Rust (edition 2024) | In use |
| Database | SQLite via `rusqlite` (bundled, WAL mode) | Implemented — 6 migrations, 29 inline tests |
| Date handling | `chrono` | In use |
| Async runtime | Tokio | In use — resolve, search, check pipelines |
| HTTP | Reqwest with retry helpers | In use — 6 ATS providers |
| Serialisation | Serde | In use — JSON (ATS responses), TOML (config) |
| Config parsing | `toml = "0.8"` | In use — `preferences.toml` → typed structs |
| TUI | Ratatui 0.29 + Crossterm backend | v5 implemented — 5 views, modular (26 source files) |
| Browser automation | `chromiumoxide` (Chrome CDP) + `futures` | Scaffolded — Chrome launches; React form filling broken |
| Testing | `cargo test`, `assert_cmd`, `proptest`, `tempfile`, `predicates` | 346 tests across 6 phases + preferences integrity guard (273 inline + 73 integration) |
| AI layer | Claude Code skills at `.claude/skills/` | 9 skills, all obligation-anchored via skill-creator |

---

## Repository Structure

```text
cernio/
├── src/                              # ~14,000 lines of Rust (56 files)
│   ├── main.rs                       # CLI dispatch
│   ├── lib.rs                        # Library surface (lib+bin split enables integration tests)
│   ├── config.rs                     # TOML config parser + filter predicates
│   ├── http.rs                       # Shared HTTP client with retry
│   ├── db/
│   │   ├── mod.rs                    # Public DB interface
│   │   └── schema.rs                 # Migrations 001-006, 29 tests
│   ├── ats/                          # 6 ATS fetchers + common types
│   │   ├── mod.rs
│   │   ├── common.rs                 # AtsJob, SlugProbeResult, get_with_retry
│   │   ├── lever.rs                  # US + EU domains
│   │   ├── greenhouse.rs
│   │   ├── ashby.rs
│   │   ├── workable.rs
│   │   ├── smartrecruiters.rs        # totalFound > 0 check
│   │   └── workday.rs                # variable subdomain + site in ats_extra
│   ├── autofill/                     # Scaffolded — broken on React forms
│   │   ├── mod.rs
│   │   ├── common.rs
│   │   └── greenhouse.rs
│   ├── pipeline/                     # The 6 mainline CLI subcommands
│   │   ├── mod.rs
│   │   ├── resolve.rs                # cernio resolve — slug probing
│   │   ├── search.rs                 # cernio search — fetch → filter → insert
│   │   ├── clean.rs                  # cernio clean — tiered archival
│   │   ├── check.rs                  # cernio check — integrity report
│   │   ├── format.rs                 # cernio format — HTML → plaintext (idempotent)
│   │   └── import.rs                 # cernio import — CSV/JSON bulk load
│   └── tui/                          # 26 source files, v5 (modular)
│       ├── mod.rs                    # Terminal setup/teardown, event loop
│       ├── app/                      # State, navigation, actions, pipeline, cleanup (6 files)
│       ├── handler/                  # Keyboard + mouse event dispatch (4 files)
│       ├── theme.rs                  # Semantic palette + freshness/activity/badge styles
│       ├── queries.rs                # DB read queries
│       ├── views/                    # 5 views + chrome + overlays (8 files)
│       └── widgets/                  # grade_bar, text_utils, toast, layout (5 files)
├── tests/                            # Integration tests (Phase 5 + 6 + preferences guard)
│   ├── common/mod.rs                 # CompanySeed, JobSeed, fixtures
│   ├── cli.rs                        # 16 CLI tests via assert_cmd + CERNIO_DB_PATH
│   ├── pipeline_clean.rs             # 11 tests
│   ├── pipeline_format.rs            # 5 tests
│   ├── pipeline_import.rs            # 12 tests
│   ├── preferences_integrity.rs      # 21 tests — guards profile/preferences.toml against silent loader fallback
│   └── smoke.rs                      # Harness sanity
├── profile/                          # Structured personal profile (synced from LifeOS canonical source)
│   ├── personal.md, education.md, experience.md, interests.md
│   ├── visa.md, military.md, languages.md, certifications.md, lifestyle-preferences.md
│   ├── resume.md, cover-letter.md    # Direct-copies from LifeOS Profile/Professional/
│   ├── projects/                     # Per-project synthesis files (one per README-listed project)
│   │   ├── index.md                  # Navigation index of all per-project files
│   │   ├── <name>.md × N             # One file per active/other project from the GitHub README
│   │   └── open-source-contributions.md  # Aggregated OSS record
│   ├── skills.md                     # Derived from projects/ — six tables, four bands
│   ├── preferences.toml              # Cernio-native: search filters, cleanup config, machine-read by Rust
│   ├── portfolio-gaps.md             # Cernio-native: career-coaching output of check-integrity
│   └── sync-summary.md               # Per-run audit artefact written by populate-from-lifeos
├── companies/potential.md            # Discovery landing zone (pre-DB)
├── .claude/skills/                   # Native Claude Code skills — 9 total
│   ├── populate-from-lifeos/         # NEW — sync profile/ from LifeOS canonical source
│   ├── discover-companies/
│   ├── populate-db/
│   ├── resolve-portals/
│   ├── search-jobs/
│   ├── grade-companies/
│   ├── grade-jobs/
│   ├── check-integrity/
│   └── prepare-applications/
├── state/cernio.db                   # SQLite (gitignored)
├── context/                          # Project memory
│   ├── architecture.md               # This file
│   ├── notes.md + notes/             # Design rationale, lessons, preferences (16 topics)
│   ├── systems/                      # Canonical implementation docs
│   │   ├── ats.md                    # NEW — 6 ATS fetchers
│   │   ├── pipeline.md               # NEW — 6+ CLI subcommands
│   │   ├── database.md               # Schema + migrations + tests
│   │   └── tui.md                    # v5 modular architecture
│   └── references/                   # Durable supporting material
│       ├── greenhouse-api.md, smartrecruiters-api.md, workable-api.md
│       ├── location-master.md        # 10-agent synthesis (session 8)
│       └── location-search/          # 10 agent outputs, ~6,500 lines total
├── Cargo.toml
├── CLAUDE.md                         # Principal-engineer personality + Cernio doctrine
└── README.md                         # Project intent and direction
```

---

## Subsystem Responsibilities

| Layer | Does | Does not |
|-------|------|----------|
| **Conversation** | Orchestrates skills and scripts, evaluates jobs against profile, recommends actions, tracks portfolio gaps | Submit applications, make decisions without user input |
| **Rust pipeline (`cernio` CLI)** | Combinatorial volume: scan ATS boards, probe slug patterns, fetch job JSON, filter, archive, format, import | Make judgments, know about the profile, decide what to search for |
| **TUI** | Real-time display of company universe, evaluation progress, user decisions; markdown export on keypress | Run scripts, evaluate jobs, or modify data independently |
| **SQLite** | Contract between all layers — single source of truth for structured data | Contain business logic |

### Canonical subsystem owners

| Subsystem | Canonical home | Maturity |
|-----------|----------------|----------|
| ATS fetchers (6 providers) | `systems/ats.md` | Comprehensive |
| CLI pipeline (6 commands) | `systems/pipeline.md` | Comprehensive |
| SQLite schema + migrations | `systems/database.md` | Comprehensive |
| Ratatui TUI | `systems/tui.md` | Comprehensive |
| Testing infrastructure | `notes/testing-strategy.md` | Working — 316 tests documented |
| Autofill (broken) | `notes/autofill-status.md` | Working — status + fix approach captured |
| Claude Code skills | `.claude/skills/<name>/SKILL.md` + `notes/skill-architecture.md` | 9 skills, all skill-creator-audited |

### AI layer — Claude Code skills

Skills are native Claude Code skills at `.claude/skills/` (migrated from the project-local `skills/` folder in commit `bebfbc5`; legacy folder removed in `d3e4e58`). Each has YAML frontmatter with engineered triggers + a negative-trigger clause and is auto-discovered via the Skill tool. Invoked conversationally — no CLI syntax required.

| Skill | Purpose |
|-------|---------|
| `populate-from-lifeos` | Sync `profile/` from LifeOS canonical source via the GitHub README allow-list (replaces `profile-scrape`) |
| `discover-companies` | Parallel-agent company discovery with creative search strategies |
| `populate-db` | Research companies from discovery, find ATS slugs, insert into SQLite |
| `resolve-portals` | AI fallback for companies that failed script-based ATS resolution |
| `grade-companies` | Enrich + grade companies (S/A/B/C) with calibration-anchored rubric |
| `grade-jobs` | Grade jobs (SS/S/A/B/C/F) with mandatory description citation |
| `search-jobs` | Orchestrate the full search cycle (script + bespoke pass, insert-obligation-anchored) |
| `check-integrity` | AI-driven re-evaluation, cross-checking, portfolio gap maintenance |
| `prepare-applications` | Generate tailored application answers per job → `application_packages` |

All nine went through a full skill-creator iteration in session 9 (commits `319ed60` through `1c9ab85`): evidence-anchored quality checklists, What-I-Did-Not-Do declarations, obligation-vs-exhortation rewrites, per-reference TOCs where missing. See `notes/skill-architecture.md` for the design rationale.

---

## Dependency Direction

```
                            ┌────────────────────────┐
                            │  Conversation (user +  │
                            │  Claude Code + skills) │
                            └───────┬────────────────┘
                                    │
                       ┌────────────┼─────────────────┐
                       ▼            ▼                 ▼
              ┌──────────────┐ ┌───────────┐ ┌───────────────────┐
              │  cernio CLI  │ │   TUI     │ │ SQLite (cernio.db)│
              │  (pipeline)  │ │           │ │  single source of │
              │              │ │           │ │  structured truth │
              └──────┬───────┘ └─────┬─────┘ └─────────┬─────────┘
                     │               │ reads + writes  │
                     │ reads config  │ user_decisions  │
                     │ + writes data │                 │
                     └─────┬─────────┘                 │
                           │                           │
                           ▼                           │
                     ┌─────────┐  ┌─────────┐  ┌───────┘
                     │ ats/    │  │config.rs│  │
                     │ (6 prov)│  │(filters)│  │
                     └─────────┘  └─────────┘  │
                           │                   │
                           └─── HTTP ──────────┘
                                to 6 external
                                ATS providers
```

No layer depends upward. The pipeline depends on `ats/`, `config`, and `db` (reads and writes). The TUI depends only on `db`. Skills depend on `profile/` (fresh reads) and indirectly on `db` (through Claude's SQL invocations).

---

## Core Execution / Data Flow

The canonical session flow:

```
1. Session starts
   └─► Claude reads profile/, context/architecture.md, context/notes.md, README.md
   └─► User and Claude discuss what to do

2. Profile sync (when projects or skills have changed in LifeOS or the GitHub README)
   └─► populate-from-lifeos skill reads LifeOS Profile/Professional/ + Projects/<README-listed>/
   └─► Updates direct-copy files, generates per-project files in projects/, derives skills.md
   └─► Writes sync-summary.md as the per-run audit artefact

3. Discovery (when the universe needs expanding)
   └─► discover-companies skill dispatches parallel sector agents
   └─► Agents write to per-agent files in companies/potential.md
   └─► User reviews, accepted companies migrated into SQLite via populate-db

4. Population (new companies → resolved or bespoke)
   └─► populate-db skill researches each company
   └─► Deterministic slug probing via `cernio resolve`
   └─► AI fallback (resolve-portals skill) for companies that fail
   └─► Companies + portals inserted into SQLite

5. Job search
   └─► Claude runs `cernio search` (script half)
   └─► search-jobs skill dispatches bespoke agents for companies without ATS (bespoke half)
   └─► All results INSERT OR IGNORE INTO jobs with evaluation_status='pending'

6. Evaluation
   └─► grade-jobs skill reads descriptions, compares against profile, writes grades
   └─► Portfolio gap tracking → profile/portfolio-gaps.md
   └─► TUI updates in real time

7. Review and export
   └─► User reviews in TUI, marks watching/applied/rejected
   └─► prepare-applications skill generates tailored answers (→ application_packages)
   └─► Autofill (when fixed) launches Chrome and fills forms

8. Maintenance
   └─► cernio clean archives stale jobs by tiered lifecycle
   └─► cernio check + check-integrity skill flag staleness and profile drift
   └─► cernio format normalises descriptions (runs silently on TUI startup)
```

---

## Inter-System Relationships

Five relationships matter for understanding cross-system behaviour — they are the contracts that break loudest when violated:

| A | B | Mechanism | What breaks if it fails |
|---|---|-----------|-------------------------|
| `ats/` (provider modules) | `config::SearchFilters::passes_location` | Shared identifier — provider name string (`"lever"`, `"greenhouse"`, ...) used as both a module name and a TOML key in `preferences.toml` | A new provider with no `[search_filters.locations.<provider>]` entry produces zero jobs post-filter. Mitigated by unknown-provider passthrough, but still a silent dropout for non-UK locations |
| `pipeline/search` | `db` (`jobs` table) | `INSERT OR IGNORE INTO jobs` keyed on `url UNIQUE` | The unique constraint is the dedup mechanism. Dropping it would cause search to emit duplicates across runs. The `INSERT OR IGNORE` vs `INSERT` distinction is load-bearing — plain `INSERT` would error on every re-run |
| `pipeline/format` | `tui/mod::run_silent` | Called on TUI startup via subprocess; must be idempotent | If `format` were not idempotent, every TUI launch would further mangle already-cleaned descriptions. The property is guarded by an explicit test (`idempotency_on_realistic_payload`) |
| `db` (`application_packages`) | `autofill/` | JSON answers written by `prepare-applications` skill, read by the autofill binary at launch | Schema contract: `job_id` → `answers` (JSON) → consumed by provider-specific field mapper. If the JSON key set drifts, autofill produces partial forms silently |
| Skills in `.claude/skills/` | `profile/` (read fresh every invocation) | Skill SKILL.md bodies enforce a mandatory-read block; CLAUDE.md re-enforces it globally | Skills that silently embed profile snapshots (instead of reading fresh) go stale the moment the profile updates. Visa dates, project tiers, degree classification all drift. This was the discovery that led to the Living System Philosophy in CLAUDE.md |
| `tests/preferences_integrity.rs` | `profile/preferences.toml` + `src/config.rs` + `src/ats/<provider>.rs` modules | Build-time assertions over file shape — required sections, valid grade letters, per-provider location subtables, UK pattern presence. The `every_supported_ats_provider_has_a_location_subtable` test drives off a `SUPPORTED_ATS_PROVIDERS` constant kept in sync with modules in `src/ats/` | The `config.rs` loader is intentionally lenient — typos silently fall back to defaults with only a stderr warning. Without these tests, a typo in `preferences.toml` would surface as the search pipeline running with default filters (thousands of off-target jobs reaching the AI grader). The Workday subtable was added in commit `86097a6` after the test would have flagged its absence — it had been silently bypassing the UK location filter |
| `populate-from-lifeos` skill | `profile/` (writes synthesised + direct-copy files; never touches `preferences.toml` or `portfolio-gaps.md`) + `LifeOS` repo via `gh api` (read-only) + `Capataina/Capataina` README via `gh api` (read-only) | Skill orchestrator; one-way data flow LifeOS → Cernio. Phase 7 verifies Cernio-native preservation by pre/post-timestamp comparison; deviation aborts with explicit error. Phase 1 parses the README's Active + Other + OSS sections as the gatekeeper allow-list (Private section excluded by design) | If the GitHub README is unparseable, the skill aborts — the gatekeeper is unreachable and silent fallback would import private projects. If a Cernio-native file's timestamp changed during the run, the skill aborts with a bug indication. Without the README gatekeeper, every LifeOS project would be imported including private/in-flight ones that the user has chosen not to surface |

### Hidden coupling

- **Provider names are a shared string across `ats/`, `config.rs`, `preferences.toml`, and `db` (`ats_provider` CHECK constraint).** Renaming `smartrecruiters` anywhere requires touching all four. No single source of truth.
- **`ats_extra` JSON structure is provider-specific and unversioned.** Changing the Workday `{subdomain, site}` shape without migrating existing rows produces silent zero-job runs for Workday portals.
- **`profile/preferences.toml` is read directly by `config.rs` at every pipeline invocation.** The TUI does not re-read it. If the user edits preferences while the TUI is running, the user keeps the stale config until restart. Acceptable trade-off — flagged here so nobody is surprised.

---

## Critical Paths and Blast Radius

### `cernio search` — the critical operation

This is the chain that fails loudest in production. Every step is documented in `systems/pipeline.md` §Key Interfaces with the per-step failure behaviour. Summary:

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

Blast radius of each step is in `systems/pipeline.md`. Short version: HTTP failures fail per-portal (other portals keep going); deserialise failures silently drop that portal; filter drops are counted and reported; DB writes are atomic per-URL via `INSERT OR IGNORE`.

### Secondary critical path: startup

TUI startup silently runs `cernio format` via `run_silent()`. If `format` crashes or hangs, the TUI hangs. The three format invariants (no raw tags, no triple blanks, never panics) + the idempotency test guard this path.

---

## Structural Notes / Current Reality

### Session 8 — location research + lifestyle modulator (2026-04-10)

A 10-agent location research pass (captured in `context/references/location-master.md` + `location-search/`) reached unanimous agreement on London as #1 and unanimous reversal of a prior "Amsterdam rejected" framing. The session also introduced `profile/lifestyle-preferences.md` and integrated it as a same-tier modulator in `notes/grading-rubric.md` — Kings Cross / Nine Elms-class lifestyle fits lift boundary grades; Croydon-class areas push them down. The `notes/location-rubric.md` captures the reasoning framework, not a scoring formula.

### Session 9 — testing foundation + skills migration (2026-04-10 to 2026-04-21)

**Testing push:** 316 tests across 6 phases (up from 18 baseline). Full decisions and phase breakdown in `notes/testing-strategy.md`. Key architectural moves: lib+bin split (`src/lib.rs` + `src/main.rs` shim), `CERNIO_DB_PATH` env var, `test_support::open_in_memory_db()`, offline JSON fixtures for ATS parsers, CLI tests via `assert_cmd`. **Found and fixed two silent data-loss bugs** during the test pass (commit `12897aa`).

**Skills migration:** all 9 project-local skills moved from `skills/` to `.claude/skills/` (commit `bebfbc5`) to gain native Claude Code integration (Skill tool auto-discovery, YAML frontmatter, `/skill-name` slash completion). Every SKILL.md gained engineered trigger descriptions, obligation-anchored language replacing exhortation framing, evidence-anchored mandatory-read tables. Legacy `skills/` folder removed (commit `d3e4e58`).

**Skill-creator session:** nine individual skill-creator iterations (commits `319ed60` through `1c9ab85`) applied the full two-pass protocol. Each produced: evidence-anchored quality checklists, What-I-Did-Not-Do declarations between workflow steps and the section separator, Over-share-exhortation cleanups, hard-rule-4 TOC additions on long reference files. Session 9 also iterated skill-creator **on itself** (commit in `~/.claude/skills/skill-creator/`, +451 lines) adding: anti-compression gate, session-aware Pass 0 for research/references, per-invocation Step 5 Post-Run Findings enforcement, worked Pass 2 example.

**CLAUDE.md:** migrated to the principal-engineer personality (commit `ce24790`). Teaches as it works, challenges weak reasoning, proactive improvement, obligation audit before declaring done. Incorporates the Cernio doctrine (Living System Philosophy, skill execution protocol, grade-quality standard, portfolio-gap tracking).

### Session 10 — populate-from-lifeos shipped + preferences integrity guard (2026-04-26)

Today's session completed the profile-schema migration and added a build-time safety net.

**populate-from-lifeos shipped and ran end-to-end.** First-run output: 11 Professional/ files synced, 12 per-project files synthesised in parallel (203 LifeOS source files consumed across all subagents, 3,413 lines of synthesised content), 1 aggregated OSS file, derived `skills.md` (6 tables, 4 bands), navigation `index.md`, and the audit artefact `sync-summary.md`. Three LifeOS folders deliberately excluded by the README gatekeeper (Flat Browser, LifeOS, Claude Config). Two issues surfaced for skill iteration: Phase 5 evidence-block contract was loose (skills agent quoted internal table rows instead of literal last lines), and the schema's `status` enum is too narrow for LifeOS reality (LifeOS uses `scaffold`, `active-status-undecided`, `#dormant`, `#skeleton`).

**`profile-scrape` retired** (commit `d907ee8`). Its responsibility — scraping individual GitHub repos for profile data — moved upstream into LifeOS's `extract-project` skill. Cernio is now strictly the consumer side. Five existing skills had their references to the old flat schema (`projects.md`, `volunteering.md`, Tier system) rewritten to the new schema (`projects/<name>.md` files, status weighting). Hardcoded project-name list removed from `grade-companies/grading-rubric.md` (it violated the Living System rule).

**Preferences integrity guard added** (commit `86097a6`). 21 new integration tests in `tests/preferences_integrity.rs` assert structural properties of `profile/preferences.toml` at build time — required sections, valid grade letters in `[cleanup]`, UK-pattern presence in every `[search_filters.locations.<provider>]` subtable, and most importantly the `every_supported_ats_provider_has_a_location_subtable` invariant which drives off a `SUPPORTED_ATS_PROVIDERS` constant kept in sync with modules in `src/ats/`. The Workday `[search_filters.locations.workday]` subtable was added in the same commit — it had been silently bypassing the UK location filter on every Workday-portal job since the fetcher shipped. Total test suite now 346.

### Current project state

| Artefact | State |
|----------|-------|
| Profile | Synced from LifeOS via `populate-from-lifeos` (first end-to-end run completed 2026-04-26); per-project files in `profile/projects/` with status frontmatter (no tier system); `skills.md` derived from projects with six-table / four-band rubric; `portfolio-gaps.md` actively maintained by `check-integrity`; `sync-summary.md` is the per-run audit artefact |
| SQLite schema | 5 tables, 6 migrations, 29 inline tests (was 11 at session 7) |
| ATS fetchers | 6 providers in use, Eightfold recorded as bespoke (no fetcher yet); preferences-integrity test enforces per-provider location-filter coverage |
| Pipeline (`cernio` CLI) | 6 mainline commands + unarchive + stats + pending + ad-hoc lever debug |
| Testing | 346 tests (273 inline + 73 integration including 21 preferences-integrity), zero failing, runs under a second once compiled |
| TUI | v5, 5 views, modular (26 source files), dashboard overhaul applied |
| Autofill | Scaffolded, broken on React forms; fix approach documented |
| Skills | 9 skills at `.claude/skills/`, all skill-creator-audited; `profile-scrape` retired, `populate-from-lifeos` added |
| Skill-creator | Self-iterated in session 9; anti-compression + session-aware Pass 0 live |

### Next priorities

1. **Fix autofill React form filling** — `nativeInputValueSetter` or CDP `Input.insertText` (blocking applications at scale).
2. **Eightfold fetcher** — currently recorded as bespoke; migration is straightforward once prioritised.
3. **Interview prep skill** — designed in `notes/interview-prep-design.md`, not yet implemented.
4. **Periodic integrity check** — ATS re-verification + grade drift detection after the next search cycle.
5. **Resume + cover-letter refresh** — `profile/resume.md` and `cover-letter.md` rebuilt session 7 against the honest project tiers; next pass after the next significant project update.

---

## Coverage

This upkeep pass (2026-04-26, session 10) inspected:

- All files under `context/` end-to-end (architecture.md, notes.md, 16 notes files, 4 system files, 3 references touched at folder level).
- `git log --format=fuller -8` plus `git show` body inspection of all four commits made today (`86097a6`, `3cd1910`, `d907ee8`, `9f19f73`) — these contain the design rationale for today's changes and were the primary source for the Inter-System Relationships additions and the new Structural Notes section.
- Full-source grep for `WHY|HACK|IMPORTANT|SAFETY|FIXME` annotations across `src/` (none found; only 2 `// TODO` / `// NOTE` lines exist project-wide).
- Connection-discovery probes against `src/`, `tests/`, `.claude/skills/`, and `profile/preferences.toml` for: provider-name string sharing across modules, `CERNIO_DB_PATH` env var (5 sites), skills referencing `profile/` (8 of 9 skills), `gh api` shared external dependency (only `populate-from-lifeos` uses it — new runtime requirement), TUI subprocess invocation of `cernio format`, and `preferences.toml` readers.
- `scripts/scan_repo.py` output (repo inventory + import graph).

Inferred from prior context, not freshly re-read this pass:

- Internals of `pipeline/resolve.rs`, `pipeline/clean.rs`, `pipeline/check.rs`, `pipeline/format.rs`, `pipeline/import.rs` — last verified end-to-end in session 9; today's changes did not touch them. Captured at behaviour-contract level in `systems/pipeline.md`.
- Individual per-provider fetcher internals — unchanged since session 9 last re-read; no provider source touched today (Workday gained a `preferences.toml` subtable but no Rust change).
- `src/tui/*` — unchanged today; `systems/tui.md` still current.
- `src/autofill/*` — status unchanged since `notes/autofill-status.md` was last written.

Deliberately not inspected:

- Individual location-research agent files (`context/references/location-search/agent-*.md`) — treated as research artefacts, the synthesis in `location-master.md` is the maintained surface.
- The full content of every per-project file in `profile/projects/` just generated by `populate-from-lifeos` — file-list and frontmatter were verified, but the 3,413 lines of synthesised content were not re-read in this upkeep. The `sync-summary.md` audit artefact is the substitute. If drift is suspected, the agent-evidence-block reproductions in `sync-summary.md` are the spot-check surface.

No subsystem was noted-but-not-read at the boundary level. The specific gap worth surfacing for the next upkeep: if a `notes/` file other than `profile-system.md` and `grading-rubric.md` (both updated today) is ever found to describe Tier-system or `profile-scrape` mechanics, it slipped past this pass — those two were the deliberate scope for the schema-migration cleanup.
