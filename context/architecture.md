# Architecture

> **Last updated:** 2026-05-31 (upkeep-context pass). The last large session-class entry below is dated 2026-04-26 (session 10); since then the project has accreted three intertwined waves of change documented in §"Structural Notes / Current Reality" §"Lane-aware universe rebuild" and §"Web frontend redesign": (1) **preferences refactor** — `preferences.toml` slimmed from 335 → ~200 lines, `[hard]`/`[soft]` sections retired, location subtables collapsed to a shared list, `archive_job_grades` rename, 19 ethical-exclusion companies deleted, substance moved to `profile/career-goals.md` (now a Cernio-native file); (2) **web frontend redesign** — `handlers/{jobs,companies}.rs` split into 6-file submodules (`mod` / `filters` / `charts` / `table` / `page` / `lanes_view`), `components.css` (24 KB) split into 10 component-shaped files, lane-view (`?view=lanes`), decisions page rebuilt, `lane_pie` SVG filter, ambient lane-gradient rows, `no_cache_static` middleware, snap CLI `PAGES` expanded; (3) **lane-aware universe rebuild** — 9-agent parallel discovery (687 → 892 active companies), sparse lanes lifted, all 892 company grades wiped (`grade=NULL`) preserving lanes + sponsors_uk + ATS resolution for lane-aware regrade. Autofill remains scaffolded-but-broken.

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
| `ats/` (provider modules) | `config::SearchFilters::passes_location` | A **single shared** `[search_filters.locations].patterns` list (per-provider subtables retired May 2026, commit `f592ca2`). Provider name is no longer a parameter to `passes_location`. | Adding a new provider no longer requires a per-provider TOML entry — the shared list applies to all. Provider names are still a shared identifier via the `ats_provider` CHECK in `company_portals` and the `SUPPORTED_ATS_PROVIDERS` invariant in `tests/preferences_integrity.rs`. |
| `src/data/lane.rs::lane_hex` | TUI theme + Web CSS chips + ECharts series + `ops.js` LANE_HEX const | Single source of truth for lane keys, labels, and hex colours. Consumed by 4 rendering surfaces. | A lane-hex change in `lane.rs` propagates everywhere automatically except `ops.js` which has a duplicate `LANE_HEX` const by necessity (loads on every page). If the two diverge, the ops menu chips will show different colours than the rest of the UI. |
| `cernio import` markdown parser (`pipeline/import.rs::parse_potential_md`) | Discovery markdown files (`companies/discovery-*.md`) | The parser recognises `Website` / `What they do` / `Why relevant` / `Source` fields only. `Lane` and `Sponsor` fields are silently dropped. | Every import of discovery files leaves `lanes` + `sponsors_uk` NULL on the newly-imported rows. The 2026-05-31 discovery run had to backfill 226 companies via `/tmp/cernio-backfill-lanes.py`. Follow-up: extend the parser. |
| `cernio web` static asset bundles | Browser cache | `no_cache_static` middleware (`src/web/mod.rs::no_cache_static`) sets `Cache-Control: no-cache, no-store, must-revalidate` on every `/static/*` response. | Without it, a stale cached CSS file would silently break the layout after a redesign and the user would see a broken page until a hard refresh. Acceptable cost in dev (localhost single-user); production would want different headers. |
| `cernio snap` PAGES const | Web routes | `src/web/debug_snap.rs::PAGES` lists every URL the snap CLI captures. Current entries cover dashboard / companies / jobs / decisions / activity plus filtered variants (`?lane=hft`) and lane-columned view (`?view=lanes`). | A page or visual state not in PAGES cannot be snapped by `snap-all`. Forgetting to add a new page means visual regressions surface only when a human looks, defeating the self-driven verification loop. |
| `pipeline/search` | `db` (`jobs` table) | `INSERT OR IGNORE INTO jobs` keyed on `url UNIQUE` | The unique constraint is the dedup mechanism. Dropping it would cause search to emit duplicates across runs. The `INSERT OR IGNORE` vs `INSERT` distinction is load-bearing — plain `INSERT` would error on every re-run |
| `pipeline/format` | `tui/mod::run_silent` | Called on TUI startup via subprocess; must be idempotent | If `format` were not idempotent, every TUI launch would further mangle already-cleaned descriptions. The property is guarded by an explicit test (`idempotency_on_realistic_payload`) |
| `db` (`application_packages`) | `autofill/` | JSON answers written by `prepare-applications` skill, read by the autofill binary at launch | Schema contract: `job_id` → `answers` (JSON) → consumed by provider-specific field mapper. If the JSON key set drifts, autofill produces partial forms silently |
| Skills in `.claude/skills/` | `profile/` (read fresh every invocation) | Skill SKILL.md bodies enforce a mandatory-read block; CLAUDE.md re-enforces it globally | Skills that silently embed profile snapshots (instead of reading fresh) go stale the moment the profile updates. Visa dates, project tiers, degree classification all drift. This was the discovery that led to the Living System Philosophy in CLAUDE.md |
| `tests/preferences_integrity.rs` | `profile/preferences.toml` + `src/config.rs` + `src/ats/<provider>.rs` modules | Build-time assertions over file shape — required sections, valid grade letters, per-provider location subtables, UK pattern presence. The `every_supported_ats_provider_has_a_location_subtable` test drives off a `SUPPORTED_ATS_PROVIDERS` constant kept in sync with modules in `src/ats/` | The `config.rs` loader is intentionally lenient — typos silently fall back to defaults with only a stderr warning. Without these tests, a typo in `preferences.toml` would surface as the search pipeline running with default filters (thousands of off-target jobs reaching the AI grader). The Workday subtable was added in commit `86097a6` after the test would have flagged its absence — it had been silently bypassing the UK location filter |
| `populate-from-lifeos` skill | `profile/` (writes synthesised + direct-copy files; never touches `preferences.toml` or `portfolio-gaps.md`) + `LifeOS` repo via `gh api` (read-only) + `Capataina/Capataina` README via `gh api` (read-only) | Skill orchestrator; one-way data flow LifeOS → Cernio. Phase 7 verifies Cernio-native preservation by pre/post-timestamp comparison; deviation aborts with explicit error. Phase 1 parses the README's Active + Other + OSS sections as the gatekeeper allow-list (Private section excluded by design) | If the GitHub README is unparseable, the skill aborts — the gatekeeper is unreachable and silent fallback would import private projects. If a Cernio-native file's timestamp changed during the run, the skill aborts with a bug indication. Without the README gatekeeper, every LifeOS project would be imported including private/in-flight ones that the user has chosen not to surface |

### Hidden coupling

- **Provider names are a shared string across `ats/`, `config.rs`, `preferences.toml` (`SUPPORTED_ATS_PROVIDERS` constant in the integrity test), and `db` (`ats_provider` CHECK constraint).** Renaming `smartrecruiters` anywhere requires touching all four. No single source of truth. (Note: the location-subtable layer was retired May 2026; provider names are no longer a key in `[search_filters.locations]`.)
- **`ats_extra` JSON structure is provider-specific and unversioned.** Changing the Workday `{subdomain, site}` shape without migrating existing rows produces silent zero-job runs for Workday portals.
- **`profile/preferences.toml` is read directly by `config.rs` at every pipeline invocation.** The TUI does not re-read it. If the user edits preferences while the TUI is running, the user keeps the stale config until restart. Acceptable trade-off — flagged here so nobody is surprised.
- **`profile/career-goals.md` is the canonical home for everything that used to live in `preferences.toml [hard]`/`[soft]`** — the 8 active lanes, role-truth-at-hire hard rule, sponsor-only universe rule, ethical-exclusions hard rule, Tier 1/2/3 location table. Grading skills (`grade-companies`, `grade-jobs`, `check-integrity`) read it fresh; `populate-from-lifeos` is forbidden from touching it. The file is Cernio-native, not LifeOS-synced.
- **Lane classification is a JSON-array string in `companies.lanes`** (and cached into `jobs.lanes` at insert). `src/data/lane.rs` parses it via `primary_lane` / `all_lanes`. The web frontend chip filters, TUI badges, grading skills, and analytics queries all key off the cached column rather than re-deriving from the company row each time.
- **Codeplay (Intel) sponsor revocation (2026-05-31)** is the canonical example of the sponsor-only universe rule in action: a company already in the DB lost its UK Skilled Worker licence between Feb–Apr 2026, `sponsors_uk` flipped to `no`, and it was archived. Re-verifying sponsor status is part of the integrity check loop, not a one-time gate at discovery.

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

### Lane-aware universe rebuild (2026-05-29 → 2026-05-31)

A coherent multi-commit move from a flat-grade universe to a sponsor-only, lane-tagged, lane-relatively-graded universe. The work spans four overlapping changes — captured in `notes/lane-aware-universe-rebuild.md` with the full commit trail and per-step rationale. Summary:

- **Preferences refactor (`72006d3`, `f592ca2`, `aebd701`, `c3c9ca2`).** `preferences.toml` reduced 335 → ~200 lines. `[hard]` and `[soft]` sections deleted entirely (substance moved to `profile/career-goals.md` as prose: 8 active lanes, role-truth-at-hire, sponsor-only universe, ethical exclusions, Tier 1/2/3 locations). Six per-provider location subtables collapsed to a single shared `patterns` list. `LocationConfig` Rust struct flattened; `passes_location` no longer takes a provider arg. `remove_job_grades` → `archive_job_grades` rename for honesty. `no_hard_or_soft_sections_present` regression test added to prevent the anti-pattern returning.
- **Ethical-exclusion deletions (`dc7d718`).** 19 companies (5 gambling, 7 adtech, 7 consumer-crypto) deleted from the DB per the new career-goals ethical-exclusions hard rule. Cascade-deleted 20 jobs + 11 portals. Zero user_decisions affected. **Explicit override of archival doctrine** — for ethical-exclusion companies, deletion was preferred so re-discovery is the default behaviour if policy ever changes. Only known intentional exception to "archive, never delete."
- **9-agent parallel discovery (`d6256f6`, 2026-05-31).** 8 per-lane agents + 1 non-obvious-sources agent dispatched in parallel. Net 212 new companies after dedup. Universe: 687 → 892 active. Sparse lanes lifted (bank-strats 18 → ~46, crypto-mm 42 → ~69, devtools 56 → ~105, big-tech 66 → ~97). Codeplay (Intel) archived after losing UK Skilled Worker licence — sponsor-only universe rule in action. Discovery files preserved as research artefacts at `companies/discovery-{lane}-2026-05-31.md`. **Known follow-up:** `cernio import` markdown parser silently drops `Lane:` and `Sponsor:` fields; 226 companies were backfilled via `/tmp/cernio-backfill-lanes.py`.
- **Grade wipe (`c8dc8e6`).** All 687 company grades cleared (`grade`, `grade_reasoning`, `graded_at`, `pinnacle_status_per_lane` set to NULL). Lanes, sponsors_uk, status, location, sector_tags, ATS resolution data all preserved. **Grade-wipe philosophy:** wipe only grade-derived columns; preserve expensive-to-rederive labour. Lane assignment (discovery-agent labour), sponsor verification, ATS slug probing all survive every regrade. Same principle as archival-not-deletion: preserve history, lose only what is fastest to regenerate from the new semantic.

The next action across the rebuild is to run `grade-companies` against the 892 ungraded set — the first proper lane-relative calibration the system will produce.

### Web frontend redesign (2026-05-29 → 2026-05-31)

Five-phase redesign of the web frontend (commits `7e2e36c`, `a897359`, `44de517`, `60ce8d9`, `6b8f4c8`, `8c73dda`, `51fa15d`, `3baaea0`, `d6256f6` web-side):

- **Modularisation (`7e2e36c`).** `handlers/jobs.rs` (864 lines) and `handlers/companies.rs` (900 lines) split into 6-file submodules each — `{mod, filters, charts, table, page, lanes_view}.rs`. Same template across both handlers (convention).
- **CSS componentisation.** `components.css` (24 KB) split into `chips.css`, `buttons.css`, `rows.css`, `tables.css`, `filters.css`, `filters-pie.css`, `jobs-lanes.css`, `companies-lanes.css`, plus rebuilt `decisions.css`. Component-shaped siblings replace the alphabetised wall.
- **Lane view (`?view=lanes`).** Both `/jobs` and `/companies` gain a lane-columned alternative view (8 lane columns; rows distributed by primary lane).
- **Lane-pie SVG filter (`6b8f4c8`).** Per-axis filter visualisation rendered as inline SVG pies in the filter strip.
- **Decisions rebuild (`60ce8d9`).** Funnel nav + next-actions pane.
- **Ambient lane gradient (`51fa15d`, `3baaea0`).** Each row gets a soft localised halo over the lane-badge area, hex colour from `src/data/lane.rs::lane_hex`. The 2026-05-31 fix in `3baaea0` resolved three structural CSS bugs — `grid-area:1/1/-1/-1` containing-block trap, `.row-title-text { display:block }` hover-bar bug, variable-row-height accent geometry — captured in `notes/css-grid-absolute-positioning.md`.
- **`no_cache_static` middleware.** New middleware in `src/web/mod.rs` forces `Cache-Control: no-cache, no-store, must-revalidate` on every `/static/*` response. Prevents stale cached CSS silently breaking the layout after a redesign.
- **Snap CLI expansion.** `PAGES` const grew to include filtered variants and lane-view URLs. Discipline in `notes/snap-self-driven-debug.md`.

### Current project state

| Artefact | State |
|----------|-------|
| Profile | Synced from LifeOS via `populate-from-lifeos`; per-project files in `profile/projects/` with status frontmatter; `skills.md` derived; `portfolio-gaps.md` actively maintained by `check-integrity`; three Cernio-native files preserved (`preferences.toml`, `career-goals.md`, `portfolio-gaps.md`). `career-goals.md` now owns the strategic frame moved out of `preferences.toml [hard]`/`[soft]` in May 2026. |
| SQLite schema | Core 5 tables + `application_packages` + lane-aware additions (`companies.lanes`, `companies.sponsors_uk`, `companies.pinnacle_status_per_lane`, `jobs.lanes` cache) + append-only `events` table. See `systems/database.md` §"Lane-based-relativity schema additions". |
| Companies | **892 active, sponsor-only, all ungraded post-wipe** (was 687 at end of session 10). 19 ethical-exclusion companies deleted as a one-off override of the archival doctrine. |
| ATS fetchers | 6 providers in use, Eightfold recorded as bespoke (no fetcher yet). `passes_location` is now arg-less + reads a single shared `[search_filters.locations].patterns` list (per-provider split retired May 2026). |
| Pipeline (`cernio` CLI) | 6 mainline commands + unarchive + stats + pending + ad-hoc lever debug. `cernio web` boots the web frontend. `cernio snap` drives self-driven visual debugging. |
| Web frontend | Second user-facing surface (sister to TUI). axum + maud + HTMX + ECharts on `localhost:7878`. 22 Rust files (~5,165 LOC) + ~4,428 LOC of CSS + vanilla JS. Modular handler split for `/jobs` and `/companies`. Lane-view (`?view=lanes`) + decisions rebuild + lane-pie filter visualisation. **Currently documented in `notes/web-frontend-architecture.md`; a `systems/web.md` split is recommended for a future Restructure pass — the surface has outgrown notes-shape.** |
| Testing | Test count drifted upward — commit `f592ca2` body cites "All 382 tests green". Most recent stable count: 346 at session 10. The growth came from preferences-refactor regression tests + the `no_hard_or_soft_sections_present` guard. |
| TUI | v5, 5 views, modular (26 source files). Untouched in the 2026-05 web-redesign window (separate surface). |
| Autofill | Scaffolded, broken on React forms; fix approach documented. Reuses the chromiumoxide dep that `cernio snap` also relies on. |
| Skills | 9 skills at `.claude/skills/`, all skill-creator-audited. |

### Next priorities

1. **Run `grade-companies` against the 892 ungraded post-wipe set** — the first proper lane-relative calibration the system has ever produced. Blocks portfolio-gap re-derivation and reordering the web dashboard's top-companies pane.
2. **Extend `cernio import` to recognise `Lane:` and `Sponsor:` fields** (`src/pipeline/import.rs::parse_potential_md`) — would eliminate the backfill-script step from every future discovery import.
3. **Promote `notes/web-frontend-architecture.md` to `systems/web.md`** — the web surface has 22 Rust files + ~4,428 LOC of CSS + vanilla JS + its own filter system + drawer + cmdk + snap CLI; it has outgrown the notes-shape. Recommend in a future Restructure pass; do not silently rewrite this Upkeep.
4. **Reconcile `context/plans/cernio-full-refactor.md` against current state** — substantial parts of the lane-based-relativity refactor have shipped (DB schema, grade wipe, preferences refactor); the plan file should reflect what is and isn't done so the next session knows what remains.
5. **Fix autofill React form filling** — `nativeInputValueSetter` or CDP `Input.insertText` (blocking applications at scale).
6. **Eightfold fetcher** — currently recorded as bespoke; migration is straightforward once prioritised.
7. **Interview prep skill** — designed in `notes/interview-prep-design.md`, not yet implemented.
8. **Periodic integrity check** — ATS re-verification + grade drift detection after the next search cycle.

---

## Coverage

This upkeep pass (2026-05-31) inspected:

- All files under `context/` end-to-end via the per-file staleness pass; see `context/_staleness-report.md` for the verdict + evidence on each of the 47 markdown files walked.
- `git log --format=fuller --since='2026-05-29'` (14 commits) plus `git show <hash>` body inspection of `72006d3`, `f592ca2`, `c3c9ca2`, `aebd701`, `dc7d718`, `c8dc8e6`, `7e2e36c`, `8c73dda`, `51fa15d`, `3baaea0`, `d6256f6` — the rationale-rich commits whose bodies drove the content of `notes/lane-aware-universe-rebuild.md`, `notes/css-grid-absolute-positioning.md`, and the new architecture sections.
- Source: `src/config.rs` (full read — confirmed `LocationConfig` flattening + `passes_location` signature change), `src/data/lane.rs` (full read — single source of truth for lane keys/colours), `src/web/mod.rs` + `src/web/handlers/{jobs,companies}/mod.rs` + `src/web/debug_snap.rs` (the split structure + `no_cache_static` middleware + `PAGES` const + `CHROME_PATH`), `src/pipeline/import.rs:130-230` (parse_potential_md — confirmed the Lane/Sponsor field omission), `profile/preferences.toml` + `profile/career-goals.md` (canonical homes of the moved content).
- Connection-discovery probes (six categories from `references/cross-system-analysis.md`): see "Inter-system relationships" probe results section below.
- Rationale-capture grep across `src/` for `WHY|HACK|IMPORTANT|SAFETY|FIXME` — no matches outside doc-comments. The `git log --format=fuller --since='2026-05-29'` commit-body inspection was the higher-yield rationale source (the Cernio project maintains rich commit bodies as durable rationale capture).
- `scripts/scan_repo.py` output: confirmed 87 Rust + 14 JS + 6 Python source files; 47 markdown files under `context/`; existing `context/` structure (architecture.md, notes.md + 18 notes, 4 systems, 2 plans-tree, 6 references).

### Connection-discovery probe results (2026-05-31)

| Probe | Outcome |
|---|---|
| Shared data structures | `AtsJob` / `SlugProbeResult` (ats↔pipeline) — documented in `systems/ats.md`. `lanes` JSON-array string is shared across `companies` table, cached `jobs.lanes` column, `events` table — documented in `systems/database.md` §"Lane-based-relativity schema additions". |
| Shared configuration | `profile/preferences.toml` read by `src/config.rs` at every pipeline invocation; `profile/career-goals.md` read by every grading skill. Documented in `systems/ats.md` + `notes/profile-system.md`. |
| Parallel evolution | `src/data/lane.rs::lane_hex` is the single source of truth for lane colours — consumed by TUI theme + Web CSS chips (`style="--lane-color: <hex>"`) + ECharts series + `ops.js` LANE_HEX const. `ops.js` const is a deliberate duplicate (loads on every page); flagged in §"Hidden coupling". No silent parallel implementations found. |
| Hidden coupling via global state | None new since session 10. The `profile/preferences.toml` read-on-every-invocation pattern remains as the documented session-state issue (TUI does not re-read until restart). |
| Event producers/consumers | New `events` table (lane-aware refactor) is producer-consumer: writers from pipeline / decisions / search; consumers TUI Activity view + Web `/activity` route. The `raw.*` prefix convention separates migration events from user-visible activity — captured in `systems/database.md`. |
| Common external deps | All 6 ATS providers depend on the shared `reqwest` client + `common::get_with_retry` (documented in `systems/ats.md`). `chromiumoxide` is shared between `src/autofill/` and `src/web/debug_snap.rs` — both depend on `CHROME_PATH` resolution, currently macOS-hardcoded in `debug_snap.rs`. |

### Convention-capture probe results (2026-05-31)

| Convention | Captured where |
|---|---|
| `handlers/{jobs,companies}/` 6-file template (`mod` / `filters` / `charts` / `table` / `page` / `lanes_view`) | New text in §"Module structure (under `src/web/`)" — explicitly named as a convention. |
| Per-feature CSS file split (rather than monolithic `style.css`) | `notes/web-frontend-architecture.md` + new architecture §"Asset bundles" expansion. |
| JSON-island naming (`chart-<kind>` + `data-<kind>` + `bootEchart(<kind>, …)`) | `notes/web-frontend-architecture.md` §"JSON islands". Existing capture, still current. |
| `?view=lanes` URL convention for lane-columned alternative views | New text in handler table; cross-referenced in `notes/snap-self-driven-debug.md` PAGES list. |
| Maud `attr[bool]` brackets for HTML boolean attributes | `notes/maud-attribute-gotchas.md`. Existing capture, still current. |

This pass (2026-05-31) inspected (pre-existing scope retained):

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

---

## Web frontend layer (added 2026-05-30 session)

A second UI surface alongside the TUI: an embedded axum server on `localhost:7878` that serves a server-rendered HTML interface over the same SQLite DB. Boot via `cernio web`. Binds 127.0.0.1 only.

### Stack

axum (router) + maud (HTML macros) + HTMX 1.9 (inline writes) + ECharts 5.4 (charts) + chromiumoxide (`cernio snap` headless debug) + tower-http ServeDir (`/static`). No build step — hand-written CSS + vanilla JS.

### Top-level shape

```
┌──────────────────────────────────────────────────────────────────┐
│                       cernio web (axum)                          │
│                                                                  │
│  Tabs: / · /companies · /jobs · /decisions · /activity            │
│  Detail drawer: ?detail=job-N | co-N (URL-persistent)            │
│  Cmd-K palette: /api/search-index.json (cached client-side)      │
│  Ops menu: Clean + Format (preview + run)                        │
│  Saved searches: localStorage cernio_presets                     │
│  cernio snap CLI: headless Chrome → /tmp/cernio-debug/<ts>/      │
└──────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                  ┌──────────────────────┐
                  │   SQLite DB           │
                  │   (shared with TUI)   │
                  └──────────────────────┘
```

### Module structure (under `src/web/`)

| File | Role |
|---|---|
| `mod.rs` | axum router + `AppState` + `no_cache_static` middleware (forces fresh `/static/*` fetches on every request — prevents stale cached CSS silently breaking the layout after a redesign) |
| `templates.rs` | chrome (topbar, tabs, lane-legend popover, drawer, ops menu, preset menu, cmdk palette, snap button) + `PageAssets::css_js` loader + `lane_legend(page)` + `json_island(kind, value)` |
| `debug_snap.rs` | `cernio snap` CLI + POST `/debug/snap-all` handler. `PAGES` const lists every URL to snap (dashboard, companies, jobs, decisions, activity, plus filtered + lane-view variants) |
| `handlers/dashboard.rs` | Action queue + companies×lane donut + jobs×lane×grade stacked bar + 7d activity + recent decisions |
| `handlers/companies/` (mod + filters + charts + table + page + lanes_view, 6 files) | 6-axis chip filter strip + analytics recomputed from filtered list + lane-columned `?view=lanes` view. Split from the previous 900-line single file in commit `7e2e36c`. |
| `handlers/jobs/` (mod + filters + charts + table + page + lanes_view, 6 files) | 7-axis chip filter strip + heatmap + freshness + funnel + table + lane-columned `?view=lanes` view. Same 6-file template as `companies/`. Split from the previous 864-line single file in commit `7e2e36c`. |
| `handlers/decisions.rs` | Watched/Applied/Interview/Rejected pipeline. Rebuilt 2026-05-30 with funnel nav + next-actions pane. |
| `handlers/activity.rs` | 7d/30d/90d timeframe toggle + raw-events toggle |
| `handlers/detail.rs` | drawer HTML fragments for `GET /detail/job/:id` + `GET /detail/company/:id` |
| `handlers/ops.rs` | Clean + Format preview + run |
| `handlers/api.rs` | stats.json + search-index.json |

**The 6-file split is a convention.** Both `handlers/jobs/` and `handlers/companies/` follow the same template: `mod.rs` re-exports, `filters.rs` chip-strip + query parsing, `charts.rs` JSON-island builders, `table.rs` row-rendering, `page.rs` top-level assembly, `lanes_view.rs` the `?view=lanes` lane-columned alternative. Adding a third page with this much surface would be expected to follow the same split.

### Asset bundles (under `static/`)

Split into shared bundles (loaded on every page) + per-page bundles (loaded via `PageAssets::css_js`). Each major feature (filters, drawer, cmdk, presets, ops) gets its own CSS+JS file rather than a monolithic `style.css` + `app.js`. Loaded via:

```rust
PageAssets::css_js("/static/css/jobs.css", "/static/js/jobs.js")
```

Shared bundles in chrome always; per-page bundle conditionally appended.

**2026-05-30 CSS modular split.** `components.css` (24 KB) was decomposed into component-shaped siblings to make styles findable by feature rather than alphabetised in one wall:

| File | Role |
|---|---|
| `chips.css` | Filter chip variants (lane / grade / plain / segmented) |
| `buttons.css` | Apply / Watch / Reject + ops controls |
| `rows.css` | `.row-clickable` + lane-accent strip + ambient glow + hover state (the file commit `3baaea0` fixed structurally) |
| `tables.css` | Table grid declarations |
| `components.css` | Panels, KPI strips, charts (everything that did not split out) |
| `filters.css` | Filter strip layout |
| `filters-pie.css` | Lane-pie SVG filter visualisation (new) |
| `jobs-lanes.css` | Lane-columned jobs view |
| `companies-lanes.css` | Lane-columned companies view |
| `decisions.css` | Rebuilt decisions page styling (funnel nav + next-actions pane) |

Full inventory + per-file roles in `context/notes/web-frontend-architecture.md`. The CSS-Grid + absolute-positioning trap fixed in commit `3baaea0` (and its two companion bugs) is captured in `context/notes/css-grid-absolute-positioning.md`.

### Cross-cutting features

- **Chip filter strips** — multi-select via CSV (`?lane=hft,ai-ml`). Every pane on the page recomputes from the filtered set. Chip kinds (`chip-lane`, `chip-grade`, `chip-plain`, `seg-group .seg`) distinguish multi-select coloured axes from binary mutually-exclusive toggles.
- **Detail drawer** — Side drawer slides in from right when row clicked. URL-persistent via `?detail=kind-N`. HTMX is re-processed inside drawer so Apply/Watch/Reject keep working.
- **Clickable charts** — Donut segments, heatmap cells, ATS bars, decision funnel rows, top-list rows, lane legend chips all navigate to filtered URLs. ECharts uses `chart.on('click')`; HTML elements wrap content in hyperlinks server-side.
- **Cmd-K palette** — Fetches `/api/search-index.json` (active companies + jobs). Substring + prefix + grade ranking. `>` prefix surfaces command shortcuts.
- **g-leader keyboard shortcuts** — `g d/c/j/a/x` jumps to Dashboard/Companies/Jobs/Activity/Decisions.
- **Saved searches** — Star-bookmark current filter URL; localStorage; cap 30.

### `cernio snap` CLI for visual debugging

Self-contained: spawns ephemeral server → drives headless Chrome → captures full-page + per-pane + viewport-slice PNGs into `/tmp/cernio-debug/<ts>/` → tears down. Optional `--temporal` re-captures with 3s gap. Used both as a CLI command and triggered from the floating `snap all` button.

This enables Claude to do its own visual observation pass on UI work — fix → snap → read PNGs → iterate without asking the user to verify. The discipline is captured in `context/notes/snap-self-driven-debug.md`.

`PAGES` const expanded in commit `d6256f6` to include filtered variants and the lane-columned view: `/`, `/companies`, `/jobs`, `/jobs?lane=hft`, `/companies?lane=hft`, `/jobs?view=lanes`, `/companies?view=lanes`, `/decisions`, `/activity`. Adding a new page or visual state requires extending PAGES — otherwise `snap-all` cannot cover it.

### Maud gotcha

`selected=(false)` emits `selected="false"` which browsers treat as truthy. Use `selected[bool_expr]` (brackets) for conditional emission. Same applies to `checked`, `disabled`, etc. See `context/notes/maud-attribute-gotchas.md`.

### See also

- `context/notes/web-frontend-architecture.md` — full module + bundle inventory, routes table, filter system, JSON-island convention, drawer mechanics
- `context/notes/maud-attribute-gotchas.md` — Maud boolean-attribute pitfalls
