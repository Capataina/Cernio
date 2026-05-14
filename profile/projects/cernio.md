---
name: Cernio
status: active
source_repo: https://github.com/Capataina/cernio
lifeos_folder: Projects/Cernio
last_synced: 2026-05-13
sources_read: 23
---

# Cernio

## One-line summary

Local-first collaborative job discovery and curation engine in Rust — combines a SQLite store, 6 ATS provider fetchers, a 5-view Ratatui TUI, and 9 native Claude Code skills into a system where scripts handle volume and AI handles judgment inside conversational sessions.

## What it is

Cernio is a **local-first, collaborative job discovery and curation engine** built in Rust. It combines a SQLite database, a Ratatui terminal UI, 6 ATS provider fetchers (plus Eightfold as bespoke), 6 pipeline scripts (resolve / search / clean / check / import / format), and 9 native Claude Code skills into a system where a human and an AI work together in conversational sessions to find, evaluate, and curate job opportunities. The architectural split is deliberate: **scripts handle volume (scanning hundreds of ATS boards in seconds), Claude handles judgment (reading descriptions, assessing fit against a nuanced profile), and the user owns every decision**. Nothing is automated end-to-end — every action happens inside a conversational session where the orchestrator (Claude) invokes scripts and skills the user approves. The TUI is the real-time window into everything, not the primary interaction mode. The project is *operational*, not aspirational: a database of 456 companies and ~1,370 graded jobs sits behind it, with grading semantics that have been through five major rubric iterations driven by production failures with real data.

## Architecture

Three-layer architecture with strict downward dependency through SQLite as the shared contract:

```
┌─────────────────────────────────────────────────────────────────┐
│                Conversational Session                            │
│                (User + Claude Code)                              │
│  - Decide what to do: discover / populate / search / evaluate    │
│  - Claude orchestrates skills + scripts, evaluates results       │
│  - User makes all application decisions                          │
└────────┬───────────────────────────────┬────────────────────────┘
         │ invokes                       │ writes evaluations
         ▼                               ▼
┌─────────────────────────┐    ┌──────────────────────────────────┐
│   Rust Scripts          │    │   SQLite (state/cernio.db)        │
│   (parameterised tools) │───►│                                   │
│  resolve / search /     │    │  companies   potential→resolved/  │
│  clean / check /        │    │               bespoke             │
│  import / format        │    │  jobs        pending→evaluating→  │
└─────────────────────────┘    │               strong_fit/weak_fit │
                               │  user_decisions   watching/       │
                               │               applied/rejected    │
                               │  application_packages             │
                               │  company_portals                  │
                               └──────────────┬───────────────────┘
                                              │ watches (2s poll)
                                              ▼
                               ┌──────────────────────────────────┐
                               │       Ratatui TUI                 │
                               │       5 views, 26 source files    │
                               │  Dashboard / Companies / Jobs /   │
                               │  Pipeline (kanban) / Activity     │
                               └──────────────────────────────────┘
```

Layer responsibilities (load-bearing — violations are surfaced as architectural bugs):

| Layer | Does | Does not |
|---|---|---|
| Conversation | Orchestrates skills + scripts, evaluates jobs against profile, recommends actions, tracks portfolio gaps | Submit applications, decide without user input |
| Rust scripts | Combinatorial volume: scan ATS boards, probe slug patterns, fetch job JSON, generate exports | Make judgments, know about the profile, decide what to search for |
| TUI | Real-time display of company universe, evaluation progress, user decisions; markdown export on keypress | Run scripts, evaluate jobs, modify data independently |
| SQLite | Single source of truth — the contract between every other layer | Contain business logic |

Module map (~14k lines Rust across 56 files, ~494KB):

```
cernio/
├── src/
│   ├── lib.rs               # Library surface (session-9 split — unblocks tests/)
│   ├── main.rs              # Thin shim calling into lib; CLI dispatch only
│   ├── config.rs            # TOML config parser (preferences.toml → typed structs)
│   ├── http.rs              # Shared HTTP client with retry logic
│   ├── db/
│   │   ├── mod.rs           # Public DB interface
│   │   └── schema.rs        # 6 migrations, 5 tables, 29 inline tests
│   ├── ats/                 # 6 provider fetchers + common types
│   │   ├── common.rs        # Shared retry helpers, slug normalisation, AtsJob struct
│   │   ├── greenhouse.rs    # Greenhouse boards-api
│   │   ├── lever.rs         # US + EU dual-endpoint probe, offset pagination
│   │   ├── ashby.rs         # POST-based job-board API
│   │   ├── workable.rs      # Offset-paginated, per-job detail fetch
│   │   ├── smartrecruiters.rs # totalFound>0 verification, ?country=gb filter
│   │   └── workday.rs       # Variable {company}.wd{1-12}.myworkdayjobs.com + site
│   ├── autofill/            # Chrome CDP automation (scaffolded, broken)
│   │   ├── mod.rs           # ApplicantProfile + provider dispatch
│   │   ├── common.rs        # Chrome launch via chromiumoxide
│   │   └── greenhouse.rs    # CSS selectors (untested against real DOM)
│   ├── pipeline/            # 6 CLI commands
│   │   ├── resolve.rs       # Slug-candidate probing across all providers
│   │   ├── search.rs        # Fetch → filter chain (location/exclude/include) → INSERT OR IGNORE
│   │   ├── clean.rs         # Tiered archival lifecycle (SS 28d → C/F 3d)
│   │   ├── check.rs         # Integrity report (health/completeness/staleness)
│   │   ├── import.rs        # Bulk import from markdown discovery files
│   │   └── format.rs        # HTML → plaintext (idempotent, 514 lines)
│   └── tui/                 # 26 files, modular v5 architecture
│       ├── mod.rs           # Terminal setup/teardown + event loop
│       ├── app/             # 6 files — App state, navigation, actions, pipeline, cleanup
│       ├── handler/         # 4 files — top-level dispatch, per-view, overlays, mouse
│       ├── views/           # 8 files — dashboard (31.5KB), companies, jobs, pipeline, activity
│       ├── widgets/         # 5 files — grade_bar, text_utils, toast, layout.distribute()
│       ├── queries.rs       # ~20 DB query functions (17.4KB)
│       └── theme.rs         # Semantic colour palette
├── tests/                   # Integration tests
│   ├── cli.rs               # 16 CLI tests via assert_cmd + CERNIO_DB_PATH tempdir
│   ├── pipeline_clean.rs    # 11 tests
│   ├── pipeline_format.rs   # 5 tests
│   ├── pipeline_import.rs   # 12 tests
│   ├── ats_strip_html_parity.rs  # 6 tests (code-health-audit follow-up)
│   ├── preferences_integrity.rs  # 21 tests (session 10 — guards preferences.toml shape)
│   └── common/, smoke.rs
├── profile/                 # Synced from LifeOS (one-way) via populate-from-lifeos
│   ├── personal/education/experience/visa/... .md  # Direct-copies from LifeOS Profile/Professional/
│   ├── projects/<name>.md × N      # Per-project synthesis files
│   ├── projects/index.md            # Generated navigation index
│   ├── projects/open-source-contributions.md  # Aggregated OSS record
│   ├── skills.md                    # Derived from per-project files
│   ├── preferences.toml             # Cernio-native — search filters + cleanup config
│   ├── portfolio-gaps.md            # Cernio-native — check-integrity output (454 lines)
│   └── sync-summary.md              # Per-run audit artefact
├── companies/               # Discovery landing zone (potential.md)
├── .claude/skills/          # 9 native Claude Code skills (.claude location, session 9 migration)
├── context/                 # Project memory (architecture, systems/, references/, plans/, notes/)
├── exports/                 # Generated markdown reports
├── CLAUDE.md                # Principal-engineer personality + Cernio doctrine
└── state/                   # SQLite DB (gitignored)
```

Key architectural properties surfaced by the LifeOS evidence base:

- **Idempotency everywhere.** Every pipeline command is safe to run repeatedly. `cernio format` only processes descriptions still containing HTML; `cernio import` deduplicates via URL UNIQUE; `cernio resolve` skips already-resolved companies. `cernio format` runs silently on every TUI startup via `run_silent()` — the idempotency-on-realistic-payload test is the guard that prevents a silent corruption loop.
- **WAL mode** — SQLite runs in WAL on every connection open, so the TUI can read while pipeline scripts write.
- **No hardcoded configuration.** Every filter, keyword, location pattern, cleanup threshold, and grade boundary lives in `profile/preferences.toml`, read as typed structs via `src/config.rs`.
- **Graceful degradation.** If `preferences.toml` is missing or malformed, `config.rs` falls back to sensible defaults (min grade B, 14-day stale threshold, exclude F/C). This is documented as a known coupling — the loader is intentionally lenient, so typos silently fall back to defaults with only a stderr warning. The session-10 `tests/preferences_integrity.rs` guard exists specifically because this leniency had been masking real bugs.
- **Lib + bin split** (session 9, load-bearing). `main.rs` is a thin shim re-exporting from `lib.rs` so integration tests under `tests/` can see public items. Every new top-level module must be declared in `lib.rs`.
- **`CERNIO_DB_PATH` env var** lets the CLI binary be retargeted at tempdir DBs for `assert_cmd` integration tests; falls back to `state/cernio.db` when unset.

Inter-system relationships (session-9-formalised contracts whose breakage produces silent dropouts):

| A | B | Mechanism | Failure mode |
|---|---|---|---|
| `ats/` provider modules | `config::SearchFilters::passes_location` | Shared provider-name string used as module name AND TOML key | A new provider without a `[search_filters.locations.<provider>]` entry produces zero jobs post-filter (mitigated by unknown-provider passthrough but still a silent dropout for non-UK locations) |
| `pipeline/search` | `db` `jobs` table | `INSERT OR IGNORE INTO jobs` keyed on `url UNIQUE` | The UNIQUE constraint is the dedup mechanism; `INSERT OR IGNORE` vs plain `INSERT` is load-bearing |
| `pipeline/format` | `tui/mod::run_silent` | Called on TUI startup as subprocess; MUST be idempotent | A non-idempotent format would mangle cleaned descriptions on every launch — silent corruption loop |
| `db` `application_packages` | `autofill/` | JSON answers written by `prepare-applications`, read by autofill binary | JSON key-set drift → partial forms silently |
| Skills in `.claude/skills/` | `profile/` | Mandatory-read protocol — every skill reads fresh on every invocation | Skills that embed profile snapshots go stale the moment the profile updates (Living System rule) |
| `tests/preferences_integrity.rs` | `profile/preferences.toml` + `src/config.rs` + `src/ats/<provider>.rs` | Build-time assertions over file shape; `every_supported_ats_provider_has_a_location_subtable` drives off `SUPPORTED_ATS_PROVIDERS` const | Without these tests, a typo in preferences.toml silently falls back to defaults (the actual failure mode that masked the Workday UK-filter bypass for the fetcher's entire lifetime until session 10) |
| `populate-from-lifeos` skill | `profile/` + LifeOS repo via `gh api` + `Capataina/Capataina` README via `gh api` | One-way orchestrator; Phase 7 verifies Cernio-native preservation by pre/post-timestamp comparison | If the README is unparseable the skill aborts — the gatekeeper would be unreachable and silent fallback would import private projects |

Hidden coupling worth surfacing (from LifeOS Architecture):
- **Provider names are a shared string across `ats/`, `config.rs`, `preferences.toml`, AND the `db` `ats_provider` CHECK constraint.** Renaming `smartrecruiters` anywhere requires touching all four — no single source of truth.
- **`ats_extra` JSON structure is provider-specific and unversioned.** Changing Workday's `{subdomain, site}` shape without migrating existing rows silently zeros out Workday-portal job runs.
- **TUI does not re-read `preferences.toml` while running.** Edits during a TUI session take effect on restart only.

Critical path — `cernio search` blast radius:

```
argv → main.rs → config::load → pipeline::search::run
  → get_search_targets (SELECT FROM company_portals WHERE companies.grade >= threshold)
  → fetch_all_parallel (Tokio Semaphore, N × {provider}::fetch_jobs)
  → per-portal HTTP via common::get_with_retry (exponential backoff)
  → serde deserialise → normalise_* → Vec<AtsJob>
  → filter stack (location → exclusion → inclusion)
  → db::job_exists → INSERT OR IGNORE INTO jobs
  → UPDATE companies SET last_searched_at
  → TUI picks up via 2s poll → Jobs view, "New ●" badge
```

HTTP failures fail per-portal (other portals keep going); deserialise failures silently drop that portal; filter drops are counted and reported; DB writes are atomic per-URL via `INSERT OR IGNORE`.

## Subsystems and components

### ATS Providers

Six provider fetchers in code plus Eightfold accepted by the CHECK constraint as bespoke (no fetcher module — only 1 company uses it, ROI stayed low). Each provider lives in `src/ats/<name>.rs` with probe / fetch / parse functions and shares `common.rs` for retry, slug normalisation, and the unified `AtsJob` struct.

| Provider | API base | Pagination | Companies (session-7 snapshot) |
|---|---|---|---|
| Greenhouse | `boards-api.greenhouse.io/v1/boards/{slug}/jobs` | None (all in one response) | 114 |
| Ashby | `api.ashbyhq.com/posting-api/job-board/{slug}` (POST) | None | 70 |
| Workable | `apply.workable.com/api/v1/widget/accounts/{slug}` | `?offset=` | 31 |
| Lever | `api.lever.co/v0/postings/{slug}` + EU endpoint | Offset, 10/page | 26 |
| Workday | `{company}.wd{1-12}.myworkdayjobs.com/wday/cxs/{company}/{site}/jobs` (POST) | `limit`+`offset` in body | 20 |
| SmartRecruiters | `api.smartrecruiters.com/v1/companies/{slug}/postings` | `limit`+`offset` max 100 | 8 |
| Eightfold | `{subdomain}/api/apply/v2/jobs?domain={domain}` | — | 1 (bespoke, no fetcher) |

Per-provider quirks documented in LifeOS Systems/ATS Providers.md:

- **SmartRecruiters returns 200 for ANY slug** — the single most dangerous false positive in the system. The API returns `{"totalFound": 0, "content": []}` for completely fake company names; HTTP 200 is NOT evidence of a company using SmartRecruiters. Only `totalFound > 0` is reliable verification.
- **Greenhouse** description requires `?content=true` parameter or detail-endpoint fetch; `metadata` field can be null or array; `offices[]` is the most structured location data.
- **Lever** has dual US (`api.lever.co`) and EU (`api.eu.lever.co`) endpoints; the probe function tries both. Description included by default in both HTML and `descriptionPlain`.
- **Workday** is the most complex: variable subdomain (`wd1` through `wd12`) and site name, both stored in `ats_extra` JSON. No public probe endpoint — resolution requires manual identification or web search via the `resolve-portals` AI fallback skill.
- **Workable** has the most structured location data (`location.city` and `location.country` separately).
- **Slug guessing is unreliable for some companies** — XTX Markets uses `xtxmarketstechnologies` (legal entity name). Parent-company slug expansion (LexisNexis → `workday/relx`) and numeric suffixes (DigitalOcean → `greenhouse/digitalocean98`) currently fall through to AI fallback.

Shared infrastructure in `common.rs`: `get_with_retry` retries on timeout/connection/request errors with exponential backoff (500ms × attempt); `post_json_with_retry` mirrors for POST endpoints; `AtsJob` is the unified type (title, url, location Vec, posted_date, description); slug normalisation is centralised.

121 + 17 = 138 bespoke companies use unsupported ATS providers (iCIMS, Taleo, BambooHR, Pinpoint HQ, Personio, custom portals) and are handled via the `search-jobs` skill's bespoke-subagent dispatch — careers-page + aggregator (LinkedIn, Indeed, Glassdoor, BuiltIn) searches.

### Pipeline (6 CLI commands)

| Command | Purpose | Async |
|---|---|---|
| `cernio resolve` | Probe ATS slug candidates across 7 providers | Yes |
| `cernio search` | Fetch jobs → filter → dedup → insert | Yes |
| `cernio clean` | Tiered archival and stale job removal | No |
| `cernio check` | Integrity report (health / completeness / staleness) | Yes |
| `cernio import` | Bulk import companies from markdown files | No |
| `cernio format` | HTML / entity-encoded descriptions → clean plaintext (idempotent) | No |

All accept `--dry-run`. `resolve` and `search` accept `--company NAME`. `search` accepts `--grade G` to scope by company grade.

**Resolve** generates ~10–20 candidate slugs from each company name (lowercase, hyphenated, no-spaces, first-word, first-two-words, stripped domain/corporate suffixes, acronyms, parenthetical content) and probes ALL slugs against ALL providers (no early termination — this finds multi-ATS companies like ClearBank with both Ashby and residual Workable). Companies remain `potential` if nothing hits.

**Search** applies a three-stage filter chain after fetching:

```
Raw ATS jobs (~16,180)
  ▼ Location filter (per-provider patterns; empty locations → KEEP)
Filtered ~8,000
  ▼ Exclusion filter (34+ title keywords: Principal, Director, VP, Staff, Sr., Sr , Lead, Manager, Head of, Chief, Distinguished, Fellow, Architect …)
Filtered ~4,000
  ▼ Inclusion filter (OR logic; empty list → pass-through)
Filtered ~2,001
  ▼ Dedup via URL UNIQUE
  ▼ INSERT OR IGNORE INTO jobs
```

The observed actionable-rate is ~0.7% (raw → SS+S+A); under the post-realism semantic it tightens to ~3-4% combined SS+S+legit-A on the post-filter pool. Filter design bias is toward inclusion — empty location data passes, empty include list passes. *False negatives are the enemy.*

**Clean** runs tiered archival (SS 28d → S 21d → A 14d → B 7d → C/F 3d active windows; archived items expire 14d after `archived_at`). Jobs with user decisions and SS/S/A are protected from staleness archival; companies are never auto-archived by grade.

**Check** produces a three-category integrity report (health, completeness, staleness) including ATS slug re-verification, orphaned decisions, duplicate companies, ungraded entities, missing descriptions, and stale grades >30d.

**Format** is the largest pipeline module at 514 lines — handles entity-encoded strings, nested tags, quoted attributes containing `>`, inconsistent whitespace. Three invariants guarded by tests: never produces raw HTML tags, never produces triple blank lines, never panics on malformed HTML. The `idempotency_on_realistic_payload` test guards against re-mangling.

**Import** parses markdown table format from `companies/potential.md`, INSERT OR IGNOREs via website UNIQUE, then **auto-clears the source file** to prevent stale entries.

### TUI (5 views, 26 source files, v5)

Five views: Dashboard (`1`), Companies (`2`), Jobs (`3`), Pipeline kanban (`4`), Activity timeline (`5`).

Modular architecture (post-session-7 split when 3 monolithic files exceeded 500 lines each):

```
src/tui/
├── mod.rs           # Terminal setup/teardown + event loop
├── app/    (6 files) # App state, navigation, actions, pipeline kanban, cleanup
├── handler/ (4 files)# Key dispatch, per-view nav, overlay input, mouse
├── views/  (8 files) # Dispatcher, chrome, overlays, dashboard, companies, jobs, pipeline, activity
├── widgets/(5 files) # grade_bar, text_utils, toast, layout.distribute()
├── queries.rs        # ~20 DB query functions (17.4KB)
└── theme.rs          # Semantic colour palette + freshness/activity/badge styles
```

Design principles: dynamic over hardcoded (`Percentage(80)` not `Length(25)`), density over whitespace, mouse-first keyboard-enhanced, grade is the primary metric (`evaluation_status` adds nothing beyond `grade` and is not displayed).

Dashboard components: activity heatmap (GitHub-style 7×12, action-type coloured), search pulse with freshness colours, application progress bar, visa countdown with urgency colours, top companies leaderboard, session welcome diff (12h lookback), grade distribution, pipeline health by ATS provider.

Interaction: `j/k` navigate, `Space` quick-peek popup, `Enter` drill in, `w/a/x/i` decisions (watching/applied/rejected/interview), `o` open URL + auto-mark applied, `y` copy URL, `g` grade picker, `Ctrl+G` smart grouping, `s` cycle sort, `f` focus mode (hide F/C+applied), `D` archive F immediately, `/` instant search, `e` export markdown, `?` help, `A` toggle archived. Mouse: scroll moves viewport not selection, Ctrl+click toggles multi-select, Shift+click range, click tab bar switches.

Responsive layout: Full (120+ cols, side-by-side master/detail), Stacked (80–119, single column list-above-detail), Compact (<80, list only). Pipeline columns size proportionally — empty collapses to 20-char minimum.

### Database (SQLite, 5 tables, 6 migrations, 29 inline tests)

Five tables:

- **companies** — id, name, website (UNIQUE — dedup key across all layers), what_they_do (3–5 sentence enrichment from `grade-companies`), discovery_source, discovered_at, status (`potential`/`resolved`/`bespoke`/`archived`), location, sector_tags, careers_url, why_relevant + relevance_updated_at, grade (S/A/B/C nullable), grade_reasoning, graded_at, last_searched_at (migration 004).
- **company_portals** — maps a company to its ATS providers (a company can have multiple portals); UNIQUE(company_id, ats_provider, ats_slug); `ats_provider` CHECK accepts greenhouse/ashby/lever/workable/smartrecruiters/workday/eightfold; `ats_extra` for provider-specific JSON (Workday subdomain + site).
- **jobs** — id, company_id, portal_id, title, url (UNIQUE — dedup key), location, remote_policy, posted_date, raw_description, parsed_tags, evaluation_status (pending/evaluating/strong_fit/weak_fit/no_fit/archived), fit_assessment, fit_score, grade (SS/S/A/B/C/F), discovered_at, archived_at (migration 005).
- **user_decisions** — multiple decisions per job allowed (watching → applied).
- **application_packages** — migration 006; `job_id` PK; `answers` JSON; auto-deleted when the job is marked applied.

Migration ladder: 001 base, 002 add `archived` to companies status CHECK (table rebuild), 003 add `archived` to jobs evaluation_status CHECK + `archived_at` column, 004 add `last_searched_at` to companies, 005 add `archived_at` to jobs for tiered expiry, 006 add `application_packages`. SQLite CHECK constraint changes require table rebuilds (`create _new → INSERT → DROP → RENAME`); foreign keys are temporarily disabled during rebuilds. All migrations are idempotent.

Indices on companies.status, companies.grade, company_portals.company_id, jobs.company_id, jobs.evaluation_status, jobs.grade, user_decisions.job_id.

Tiered archival lifecycle: SS 28d active / S 21d / A 14d / B 7d / C+F 3d, then 14d archive expiry before full deletion (allows re-discovery with a potentially-updated profile).

### Grading System (5 phases, the most-iterated subsystem)

Companies grade S/A/B/C. Jobs grade SS/S/A/B/C/F. Grades map directly to TUI behaviour — SS/S surface first, F invisible unless toggle, focus mode hides F/C+applied, Pipeline kanban shows only decisions.

Five rubric phases driven by production failures:

1. **Dimension-weighted scoring** (sessions 1–3) — agents assigned middling scores arriving at B without genuine reasoning. Amazon at B, Monzo at C, Netflix at B.
2. **Hard grade floors** (session 4) — FAANG min A, large UK sponsors min B, Rust min B. Failed because floors are rigid (Solutions Architect at Amazon forced to A).
3. **Career-stage calibration + relative grading** (session 4) — dimension reweighting for a candidate with no work experience; CV signal and sponsorship at very high weight, tech stack at low. Improved but still mechanical.
4. **Question-first reasoning** (sessions 4–5) — complete rewrite. Five job questions: Can I get it? / Good CV line? / Do I have an edge? / Would I enjoy it? / Practical constraints? Four company questions: Would you be proud to work here? / Could they hire you? / Would you grow? / Would you find it engaging? Dimensions become analytical support, not primary scoring.
5. **Realism semantic** (session 11, commit `389b1e8a`, 2026-04-29) — explicit decoupling of reputation (Q2 — CV value) from selectivity (Q1 — realistic achievability). A reputable name on a CV says nothing on its own about whether the candidate can realistically be hired; the two axes are independent and must be assessed separately. When reasoning about Q1, ignore Q2's signal entirely. Wide-funnel reputable firms (Amazon SDE-1, Bloomberg, Microsoft Graduate, Cloudflare interns, Anthropic Fellows, HRT 2026 Grad SWE, Squarepoint Graduate) anchor SS correctly; narrow-funnel reputable firms (Jane Street SWE UK, Anthropic London non-Fellows, regular Citadel/Two Sigma/D.E. Shaw/HRT roles outside their grad pipelines) cap at A-stretch.

Calibration anchors (session 5) — before grading, pull 2–3 real examples at each tier from the existing DB. Grade each job against those anchors rather than enforcing a within-batch distribution. Mandatory description citation prevents title-only grading (the Phase 3 failure mode that put "entry-accessible" labels on jobs requiring 3-5 years).

Status-based project weighting (session 10) replaces the retired Flagship/Notable/Minor Tier system. Frontmatter status (`active`, `complete` substantive depth → primary evidence; `paused`, `scaffold`, `active-status-undecided` → secondary; `dormant`, `#dormant`, `#skeleton` → avoid citing).

Lifestyle modulator (session 8) — `profile/lifestyle-preferences.md` is read alongside the main profile by every grading invocation as a same-tier modulator, not a Tier 3 tiebreaker. Kings Cross / Nine Elms-class areas lift boundary grades; Croydon-class areas push them down. A borderline A/B role in Kings Cross lifts to A; the same role in outer Croydon drops to B.

Production impact of the realism semantic (2026-04-29 morning batch, 300 jobs): 15 SS / 21 S / 62 A / 35 B / 51 C / 116 F. 12% S+ density confirms calibration (rubric warns if >20%). Jane Street prestige-trap pattern confirmed across 18 roles → 0 SS / 0 S / 4 A-stretch / 5 B / rest C-F.

### Skills Ecosystem (9 native Claude Code skills, ~290KB documentation)

| Skill | Purpose |
|---|---|
| `populate-from-lifeos` (s10 NEW) | Sync `profile/` from LifeOS canonical source via the GitHub README allow-list — one-way flow, never writes to LifeOS, never touches Cernio-native files (`preferences.toml`, `portfolio-gaps.md`). Replaces retired `profile-scrape`. 8-phase autonomous workflow. |
| `discover-companies` | Profile-aware company discovery via parallel sector agents (AI/ML, fintech, trading, systems, devtools, non-obvious sources) with creative web search. |
| `populate-db` | Research companies from discovery, find ATS slugs, insert into SQLite. |
| `resolve-portals` | AI fallback for companies that fail script-based resolution. |
| `search-jobs` (s9 upgrade + s11 mini-iteration) | Orchestrates full search cycle — script half (`cernio search`) + bespoke half (AI agents for companies without ATS), insert-obligation-anchored. |
| `grade-companies` | Enrich + grade companies (S/A/B/C) with profile-grounded reasoning, question-first rubric, calibration-anchored. |
| `grade-jobs` (s9 + s11 realism rewrite) | Grade jobs SS-F with question-first rubric, mandatory description citation, calibration anchors, lifestyle modulator, and the realism semantic with reputation × selectivity decoupling. 371-line reference rubric. |
| `check-integrity` | AI-driven re-evaluation, cross-checking, portfolio gap maintenance. The most complex skill with 4 reference files. |
| `prepare-applications` | Generate tailored application answers per job (cover letter / why-this-role / project-answer / common Qs), JSON-stored in `application_packages` table, consumed by autofill via TUI `p` key. |

All skills enforce a **mandatory-read protocol** (added session 3): every skill agent must read its SKILL.md, every file in its `references/` directory, and all relevant files in `profile/` — every file, every time, on every invocation. Skills are obligation-anchored (session 9) — verifiable checklists with evidence outputs replace vague exhortation language ("be thorough"). Three skills (`resolve-portals`, `grade-jobs`, `prepare-applications`) had step-0 script-call patches added in commit `bee129a` to enforce `cernio resolve` / `cernio format` precursors and prevent the F12/F15 tool-action / script-obligation asymmetry.

### Testing Infrastructure (346 tests, 6+ phases)

Architectural decisions that made the test surface viable:
1. **Lib + bin split** — `main.rs` is a thin shim over `lib.rs` so integration tests under `tests/` can see public items.
2. **`CERNIO_DB_PATH` env var** — CLI binary reads it with fallback to `state/cernio.db`, so each `tests/cli.rs` case targets a per-test tempdir.
3. **`test_support::open_in_memory_db()`** — exposed via `cernio::test_support` under `#[doc(hidden)]`; fresh in-memory SQLite with all migrations on every call. The workhorse fixture.
4. **Inline for private pure helpers, integration for public flows** — private pure functions live in `#[cfg(test)] mod tests` at the bottom of their source files; public flows and the CLI binary live under `tests/`.
5. **Offline JSON fixtures, never HTTP mocking** — ATS parser tests construct minimal JSON shaped like real responses and call `normalise()` directly. Deterministic, fast, doubles as response-shape documentation.
6. **TUI tested by state, not by rendering** — zero rendering tests; pure helpers (`distribute()`, `clean_description`, `relative_date`, `truncate_chars`) only.

Test count by area: 85 in `format.rs`, 31 in `config.rs`, 30 in `resolve.rs`, 29 in `schema.rs`, 16 Lever, 14 jobs view, 13 Greenhouse, 13 Workday, 12 SmartRecruiters, 11 layout, 10 Workable, 8 Ashby, 16 CLI integration, 11 pipeline_clean, 12 pipeline_import, 5 pipeline_format, 6 strip_html parity, 21 preferences_integrity, 2 smoke — **346 total**.

Idempotency guarantee — the single most load-bearing property:
```rust
format_description(format_description(x)) == format_description(x)
```
Three direct invariant tests plus an explicit Greenhouse-shaped-payload idempotency test guard this. `cernio format` runs silently on every TUI startup, so non-idempotency would mangle cleaned descriptions on every launch.

Bugs found by the test/integrity investment:
- **Two silent data-loss bugs** in session 9 (commit `12897aa`) — surfaced by the retroactive pass.
- **Workday UK-filter silent bypass** in session 10 (commit `86097a6`) — the `[search_filters.locations.workday]` subtable had been absent since the fetcher shipped, silently bypassing the UK location filter. The new `every_supported_ats_provider_has_a_location_subtable` test in `preferences_integrity.rs` was added in the same commit alongside the fix.
- **Timestamp format mismatch** in session 11 (commit `50359b13`) — cleanup queries compared `discovered_at` as raw strings against SQLite's `datetime('now')`; inserts used chrono format `%Y-%m-%dT%H:%M:%S` while SQLite emits `%Y-%m-%d %H:%M:%S` (space, not T). Shift+D archive was silently broken. Patched 7 files (`pipeline/check.rs`, `clean.rs`, `search.rs`, `tui/app/actions.rs`, `cleanup.rs`, `pipeline.rs`, `tui/queries.rs`).

### Autofill (scaffolded, broken — Priority 1 fix)

Architecture is in place — Chrome launches via `chromiumoxide`, navigates to the job URL, the DB table works, the TUI integration works. But form filling does not work on real Greenhouse forms: JavaScript value injection (`el.value = "..."; el.dispatchEvent(...)`) does not trigger Greenhouse's React state management. React-controlled inputs ignore direct `.value` assignment — they need synthetic React events or `nativeInputValueSetter` tricks. Documented fix: replace JS value-injection with CDP `Input.insertText` or `nativeInputValueSetter`, test against a real Greenhouse form, update CSS selectors from real DOM inspection, add Lever + Ashby modules. The "Chrome is being controlled by automated test software" banner also needs the `--disable-blink-features=AutomationControlled` flag evaluated.

### Location Evaluation (session-8 subsystem)

A reasoning framework, not a scoring formula. Three-tier rubric across 22 factors evaluating cities at city / country / hybrid level across current state and three trajectory horizons (1-3 / 5-7 / 10-15 years). Tier 1 (deal-makers / breakers): visa accessibility for a Turkish national at entry level (country), target firm density in HFT / fintech infra / AI infra / systems / Rust / modern devtools (city), urban aesthetic match (city — mixed-scale, integrated greenery, walkability), safety and civic order (city), political/legal stability (country). Tier 2 (shifts the verdict without overriding Tier 1) covers nightlife trajectory, salary × cost-of-living, tax regime for high earners, secular public culture, café culture, path to permanent residency, frontier-tech access (Waymo-class), gym infrastructure (Third Space-class), English accessibility, climate tolerance. Tier 3 (tiebreakers) covers integration quality, housing depth, airport connectivity, food, healthcare, time-zone overlap, currency stability.

Mechanical constraints the evaluator cannot override: Turkish national (no dual citizenship → excludes UK SC/DV and US clearance), UK Graduate visa expires August 2027 (dominant forcing function), zero years professional work history (3+ year roles mechanically out of reach), BEng CS 2:2 University of York (some firms enforce 2:1), languages Turkish native / English fluent / German A2-B1 / nothing else.

Session-8 research pass: 10 parallel agents over candidate cities, ~6,500 combined lines per-agent + 71KB `location-master.md` synthesis. London #1 by unanimous agreement. "Amsterdam rejected" overturned unanimously from prior profile verdict.

### Code Health Audit (27 open findings, none implemented yet)

Full two-pass repository audit landed at session 9 (commit `c7973e0`). **The audit modified no production code** — it added `context/plans/code-health-audit/*` and one new test file (`tests/ats_strip_html_parity.rs`, 6 tests). 27 actionable findings across 8 systems: 4 high-severity, 14 medium, 7 low, 2 triage. Plus a modularisation verdict table (3 split / 11 leave / 1 n/a) and a 37-row dead-code-sweep disposition.

Four high-severity findings (worth naming):

1. **Four divergent `strip_html` implementations across `src/ats/`** — two diverge on quote-handling; the divergent Workable version is live (latent correctness bug on descriptions with `>` inside quoted HTML attributes). Consolidation removes 70 lines.
2. **N+1 query in `pipeline::search::run_by_grade`** — 288 round-trips per grade-scoped search at 287 resolved companies; a single `SELECT ... WHERE c.grade = ?` replaces the loop.
3. **`fetch_stats` issues 16 SQL queries per dashboard poll** — at 2s polling that is ~29,000 round-trips/hour of TUI use. Consolidation into 4-6 `GROUP BY` queries reduces by 3-4×. *The audit's largest observable performance win.*
4. **SmartRecruiters pagination missing `get_with_retry`** — transient 502 mid-pagination produces silent partial fetch with no error surfacing.

## Technologies and concepts demonstrated

### Languages

- **Rust (edition 2024)** — entire codebase. ~14k lines across 56 files. Used across pipeline, TUI, ATS code, DB layer, config, autofill scaffolding, and tests. The library/binary crate split is itself a Rust-mechanic choice (binary-only crates cannot be integration-tested via `tests/`).
- **TOML** — `profile/preferences.toml` is the runtime configuration surface, parsed by `toml = "0.8"` into typed Rust structs in `src/config.rs` with serde derives.
- **SQL** — SQLite dialect; 6 migrations, 5 tables, 7 indices. Migrations include manual table-rebuild patterns for CHECK constraint changes.
- **Markdown** — discovery landing zone (`companies/potential.md`), exports, README, profile files, the entire LifeOS-side documentation surface.
- **JavaScript (CDP-injected)** — autofill's broken value-injection path; the documented fix uses CDP `Input.insertText` to bypass JS for React forms.

### Frameworks and libraries

- **`rusqlite` (bundled)** — SQLite with WAL mode enabled on every connection open. Bundled SQLite means no system dependency.
- **`tokio`** — async runtime for pipeline scripts. `pipeline::search::fetch_all_parallel` uses a `Semaphore` to bound concurrent provider calls.
- **`reqwest`** — HTTP client. Shared `http::build_client()` plus the `ats::common::get_with_retry` / `post_json_with_retry` helpers wrap it with exponential backoff retries (500ms × attempt) on timeout/connection/request errors; non-retryable 4xx errors return immediately.
- **`ratatui` 0.29** — TUI framework. Five views, modular widget architecture, responsive layout, `Percentage`-based sizing.
- **`crossterm`** — terminal backend for `ratatui`. Mouse support including Ctrl+click multi-select and Shift+click range.
- **`serde` + `serde_json`** — JSON for ATS responses + provider-specific `ats_extra` payloads. TOML deserialisation for preferences. `application_packages.answers` JSON.
- **`toml = "0.8"`** — preferences parser.
- **`chromiumoxide`** — Chrome DevTools Protocol bindings for autofill. Currently scaffolded (Chrome launches headed) but the form-filling path is broken on React controlled components.
- **`assert_cmd`** — CLI integration tests; `Command::cargo_bin("cernio")` spawns the real binary against a tempdir DB via `CERNIO_DB_PATH`.
- **`proptest`** — property-based testing (one of the test-suite tools listed in the Overview's stack table).
- **`tempfile`** — per-test tempdirs for CLI integration tests.
- **`predicates`** — `assert_cmd` assertion library.
- **`gh` CLI (runtime cross-vault dependency)** — the `populate-from-lifeos` skill calls `gh api` to fetch LifeOS folder contents and the `Capataina/Capataina` README. Cross-repo dependency added in session 10.

### Runtimes / engines / platforms

- **SQLite WAL** — concurrent reads (TUI) while writes (pipeline scripts) proceed without contention. Set on every connection open.
- **Tokio** — async I/O multiplexing for HTTP-heavy pipeline work (resolve, search, check). Business logic stays sync; async is the I/O layer only (matches the LifeOS-cited Microsoft Rust training principle "Async Is an Optimization, Not an Architecture").
- **Chrome (CDP)** — autofill subsystem; headed mode, real browser.
- **Claude Code skills runtime** — 9 native skills auto-discovered via the Skill tool with YAML-frontmatter engineered triggers + negative-trigger clauses, plus `/skill-name` slash completion.

### Tools

- **`cargo test`** — full suite runs sub-second once compiled. The slowest cases are CLI tests (each spawns a subprocess).
- **`cargo` toolchain** — Rust edition 2024.
- **`skill-creator` meta-skill** (upstream from `Capataina/.claude`) — used to audit and iterate every Cernio skill in session 9 (commits `319ed60` → `1c9ab85`).
- **GitHub Actions** — referenced in the Cloud Deployment Work file as the proposed CI surface for the cloud-gap closure (not yet implemented).
- **`gh` CLI** — runtime dependency of `populate-from-lifeos`.

### Domains and concepts

- **ATS (applicant tracking system) integration** — 6 providers across REST/JSON APIs with provider-specific quirks (POST vs GET, pagination shapes, location format variability, `totalFound`-based false-positive guards, dual regional endpoints). Mature pattern of normalising heterogeneous provider responses into a unified `AtsJob` struct.
- **Slug-candidate generation and probing** — ~10-20 candidates per company name across naming conventions (lowercase, hyphenated, no-spaces, first-word, first-two-words, stripped domain/corporate suffixes, acronyms, parenthetical content). Probes against all providers in parallel with no early termination (catches multi-ATS companies).
- **Filter chain design with false-negative-aversion bias** — every filter stage treats empty data as pass (empty location list → KEEP; empty include list → pass-through). Validated against 2001 graded jobs that every exclusion keyword has 0 hits at B+.
- **Idempotency as a load-bearing invariant** — the format pipeline runs silently on every TUI startup; non-idempotency would silently corrupt cleaned descriptions on every launch. Three invariant tests + an explicit realistic-payload idempotency test guard the property.
- **Tiered archival lifecycle** — grade-tied active windows (SS 28d → C/F 3d), archive expiry, protection for SS/S/A and any job with a user decision. Lets re-discovery happen on future searches with potentially-updated profile.
- **SQLite migration discipline** — manual table-rebuild for CHECK constraint changes (SQLite limitation); every migration idempotent (test before applying); FK temporarily disabled during rebuilds.
- **Async I/O multiplexing with bounded concurrency** — `tokio::Semaphore` caps parallel provider calls in `fetch_all_parallel`; per-portal failures don't fail the batch.
- **HTTP retry with exponential backoff** — `get_with_retry` / `post_json_with_retry` on timeout/connection/request errors; non-retryable 4xx returns immediately. Pattern enforced at the `common.rs` boundary; the audit's MEDIUM finding is standardising this across Ashby/Workable/Workday paths that don't yet route through it.
- **Quote-aware HTML stripping** — handles `>` inside quoted attribute values (Graphcore `data-ccp-props` artefacts); divergence across four provider-side implementations is a HIGH-severity audit finding with a parity-test guard in place.
- **Lib + bin testability split** — the smallest possible architectural change that unblocks Rust integration testing (binary-only crates cannot be tested via `tests/`).
- **TUI architecture (modular)** — App struct as shared state, per-concern modules (state / navigation / actions / pipeline / cleanup) as methods on App; rendering separate from event handling; widgets reusable. The session-7 split at ~500 lines per file is the explicit threshold.
- **Reactive dashboard polling** — 2s poll cycle, GitHub-style activity heatmap, freshness colours by age, multi-tier visa countdown urgency, smart grouping by company.
- **Calibration-anchored grading** — instead of within-batch distribution, anchor to 2-3 real DB examples per tier. Realism-aware anchor selection (post-2026-04-29) excludes prestige-trap roles from SS calibration.
- **Question-first reasoning rubric** — five questions for jobs, four for companies; dimensions are analytical support not primary scoring. Mandatory description citation prevents title-only grading. Five major iterations driven by production-data failure modes (dimension scoring inflated middle ranks → hard floors were too rigid → career-stage calibration was still mechanical → question-first stopped title-only grading → realism semantic stopped prestige leak).
- **Reputation × selectivity decoupling (realism semantic)** — explicit independence of Q2 (CV value / brand reputation) from Q1 (realistic achievability / firm hiring pattern). The detection rule: when reasoning about Q1, ignore Q2's signal entirely. Operationalised through wide-funnel vs narrow-funnel firm calibration and worked-example pairs (Amazon SDE-1 New Grad as SS anchor; Jane Street SWE UK as A-stretch anchor with same FAANG-or-above CV signal but opposite Q1 reading).
- **Lifestyle as same-tier grading modulator** — `lifestyle-preferences.md` moves grades across boundaries (Kings Cross / Nine Elms lift, Croydon push down), not just within-grade tiebreaking. The rationale: aesthetic-daily-environment compounds over years in a way pay or tax bracket do not.
- **Status-based project weighting (Living System rule)** — replaces the retired Flagship/Notable/Minor Tier system. Per-project file `status` frontmatter (`active`, `complete`, `paused`, `scaffold`, `dormant`) drives evidence weight at grading time; no hardcoded project lists in rubrics.
- **Mandatory-read protocol for skills** — every skill agent reads its SKILL.md, every reference file, and all relevant profile files on every invocation. Profile snapshots embedded in skills go stale silently and produce incorrect evaluations.
- **Obligation-anchored skill design** — verifiable checklists with evidence outputs ("produce artefact X", "cite the last line of each reference") replace exhortation language ("be thorough"). Session-9 audit shifted every skill onto this footing across commits `319ed60`–`1c9ab85`.
- **F12/F15 tool-action / script-obligation asymmetry** — a skill whose workflow does work a script could have done first will silently burn tokens. Step-0 patches added to `resolve-portals` (precursor `cernio resolve`), `grade-jobs` (precursor `cernio format`), and `prepare-applications` (precursor `cernio format`).
- **One-way cross-repo sync architecture (LifeOS → Cernio)** — README-as-gatekeeper allow-list, pre/post-timestamp Cernio-native preservation check, autonomous 8-phase workflow, parallel per-project subagent dispatch with mandatory evidence-block returns.
- **Anti-puffing principle in synthesis** — describe what the project demonstrates, not what its README pitches. LifeOS folder content is the evidence; the skill structures it, never inflates it. Operationalised through Tier-3 evidence anchors (file path, line count, verbatim last line) in subagent contracts.
- **Three-layer reasoning rubric for cities** — 22 factors across Tier 1 / Tier 2 / Tier 3 weights, evaluated at city / country / hybrid level across current state and three trajectory horizons. Trajectory is first-class (a 10/10 declining is worse than a 7 rising). Mechanical constraints (visa, work history, degree, languages) cannot be overridden.

## Key technical decisions

LifeOS `Decisions.md` documents the following major design decisions with reasoning, alternatives considered, and consequences:

- **Collaborative, not automated.** The original README described `cernio refresh` as a daily automated pipeline; this was revised in the first design session. There is no single "run everything" command. Every feature must support a conversational workflow; scripts are parameterised tools invoked by Claude during a session, not cron jobs.
- **Scripts for volume, AI for judgment.** A Rust script can check 5,000 ATS combinations in seconds; Claude can read 50 resulting job descriptions and assess fit; neither could do the other's job economically. Scripts must be generic, reusable, parameterised; intelligence lives in the conversation.
- **SQLite as single source of truth.** Evaluated against markdown files, JSON/JSONL, and Postgres/MySQL. SQLite won on zero ops, single file, full SQL, WAL for concurrent access, trivial backup. Profile data stays in markdown (human-edited); companies/jobs/decisions/packages live in SQLite (machine-managed); discovery results land in markdown first, then migrate via import.
- **Question-first grading over dimension-weighted scoring.** Four iterations to reach this point, each driven by production failures with real data. Questions force genuine reasoning; dimensions are analytical support.
- **C-tier companies stay active.** Originally C companies were auto-archived. Changed in session 4: job grading handles quality filtering. A C company might have one genuinely good role; cost of extra jobs is grading time (cheap), cost of missing a good role is unrecoverable.
- **False negatives are the enemy.** At every filtering stage, bias toward inclusion. Empty data → include, not exclude. A job with no location passes the location filter; a title that doesn't match exclusion keywords passes even without matching inclusion keywords.
- **Mandatory-read protocol for all skills** (session 3). Every skill agent must read SKILL.md, all references/, all profile/. The reference documentation IS the quality bar. Added after agents skipped reference files and produced shallow output.
- **TUI grade as primary metric, not evaluation_status.** `evaluation_status` is just a coarser bucketing of `grade`. The TUI displays grade only.
- **Per-provider location patterns, not global.** Location formats differ dramatically across ATS providers (Greenhouse "Hybrid" or "Berlin; London; Munich"; SmartRecruiters supports server-side `?country=gb`); patterns live per-provider in `preferences.toml`.
- **Profile scrape: built not planned** (session 6). Profile entries lead with implemented code, not README aspirations. Anti-puffing principle.
- **Skills in this repo, not upstream.** Cernio's skills are tightly coupled to its data model and workflow. They live at `.claude/skills/` within the repo. The upstream `Capataina/.claude` has universal skills; project-specific skills don't generalise.
- **Lib + bin split for testability** (session 9). Rust integration tests under `tests/` can only see public items from a library crate, not a binary-only crate. The split was the smallest change that unblocked `tests/cli.rs`, `tests/pipeline_*.rs`, and the 16-test CLI suite. Alternative: separate sibling crate (rejected as more change for the same outcome).
- **`CERNIO_DB_PATH` env var for CLI testability** (session 9). Smallest change to make `tests/cli.rs` viable; each test sets the env var to a per-test tempdir. Alternative `--db-path` flag rejected (requires every test to pass it explicitly).
- **Lifestyle fit as same-tier grading modulator, not Tier 3 tiebreaker** (session 8). Caner spends every day of his life in the environment the role is based in; aesthetic-daily-environment compounds over years in a way pay or tax bracket do not. Grades move across boundaries based on lifestyle fit, not just within-grade.
- **Native Claude Code skills at `.claude/skills/`** (session 9). Skill tool auto-discovery, YAML engineered triggers, slash completion. Replaces the older "read SKILL.md when I tell you to" pattern.
- **Obligation-anchored over exhortation-anchored** (session 9, cross-domain). Verifiable obligations ("produce artefact X", "emit section Y") replace vague "be thorough" / "carefully check" framing. Research-backed: RLHF absorbs exhortation sycophantically — agents produce the *appearance* of thoroughness while skipping actual work.
- **LifeOS as canonical, Cernio as consumer** (session 10). LifeOS is human-curated and the existing source of truth for the candidate. Profile-scrape's GitHub-repo-scraping role moved upstream into LifeOS's `extract-project` skill. Cernio is now strictly the consumer side. Cernio-native files (`preferences.toml`, `portfolio-gaps.md`) explicitly off-limits to the sync; one-way data flow. Alternatives considered: keep manual mirror maintenance with periodic audits (rejected — drift was happening); make Cernio canonical (rejected — LifeOS's structural role is broader than career data); symlink (rejected — would expose private LifeOS contents to the public Cernio repo).
- **README-as-gatekeeper for project sync** (session 10). The `Capataina/Capataina` GitHub README's Active / Other / OSS sections are the allow-list for which projects appear in `profile/projects/`. Private Projects section excluded by design. Alternative considered: include every LifeOS project (rejected — would surface private/in-flight work the user hasn't chosen to expose).
- **Status-based project weighting replaces the Tier system** (session 10). Flagship/Notable/Minor retired. Status frontmatter (`active`, `complete`, `paused`, `scaffold`, `dormant`) now drives grading weight. Once the profile was split into per-project files (each a comprehensive evidence-anchored synthesis), every file became its own canonical evidence — no need to assign a tier label because the file's depth and status frontmatter carry the signal more honestly.
- **Realism semantic — reputation × selectivity decoupling** (session 11, 2026-04-29, commit `389b1e8a`). Phase 5 grading rubric. Reputation (Q2 — CV value) and selectivity (Q1 — realistic achievability) are independent axes; do not infer one from the other. When reasoning about Q1, ignore Q2's signal entirely. Operationalised through worked examples (Amazon SDE-1 New Grad as SS anchor; Jane Street SWE UK as A-stretch anchor) and wide-funnel vs narrow-funnel firm calibration. Alternatives considered: tighten Phase 4 wording (rejected — failure mode was systematic prestige leakage, not wording problems); add hard "no SS at firms with intake < X" rules (rejected — too rigid).

## What is currently built

The system is **operational** — not aspirational. The honest current scope:

- **~14k lines Rust across 56 files, ~494KB.** Edition 2024. Lib + bin split.
- **6 ATS provider fetchers** in code (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday) + Eightfold accepted by the CHECK constraint as bespoke (no fetcher module). All 6 fetchers normalise into a unified `AtsJob` struct.
- **6 mainline pipeline commands** (resolve, search, clean, check, import, format) plus unarchive/stats/pending. All accept `--dry-run`; `resolve` and `search` accept `--company NAME`; `search` accepts `--grade G`.
- **TUI v5 — 5 views, 26 source files, modular architecture.** Dashboard / Companies / Jobs / Pipeline kanban / Activity timeline. Mouse-first with keyboard accelerators. Responsive across three layout modes.
- **DB schema — 5 tables, 6 migrations, 29 inline tests.** Tiered archival lifecycle. `application_packages` table feeds autofill.
- **346 tests passing** (273 inline + 73 integration, including 21 in the session-10 preferences-integrity guard). Was 18 at session 7. The retroactive test pass surfaced two silent data-loss bugs immediately and the Workday UK-filter silent bypass when the integrity guard landed.
- **9 native Claude Code skills** at `.claude/skills/`, all skill-creator-audited with obligation-anchored mandatory-read tables, Tier-3 evidence-anchored quality checklists, and What-I-Did-Not-Do declarations between workflow steps. Total skill documentation is ~290KB.
- **DB state:** 456 total companies (318 resolved with ATS / 138 bespoke / 0 potential / 0 archived) and ~1,370 graded jobs. Post-realism non-archived pipeline distribution: 15 SS / 23 S / 90 A / 71 B / 214 C / 773 F (=1,186 graded as of 2026-04-29); incremental 2026-05-10 batch added 4 SS / 8 S / 18 A / 13 B / 40 C / 81 F.
- **Companies by ATS provider** (pre-dad-list snapshot): Greenhouse 114 (40% of resolved), Ashby 70 (24%), Workable 31 (11%), Lever 26 (9%), Workday 20 (7%), SmartRecruiters 8 (3%), Eightfold 1 (<1%).
- **34+ validated exclusion keywords** (Principal, Director, VP, Staff, Sr., Sr , Lead, Manager, Head of, Chief, Distinguished, Fellow, plus the "Senior" inclusion-to-exclusion flip in session 5 that caught 742 F/C with only 18 B+ in the firing line — 41:1 ratio).
- **27 open code-health-audit findings** (4 high / 14 medium / 7 low / 2 triage) — none implemented yet; the audit explicitly modified no production code, only `context/plans/code-health-audit/*` plus the one new `tests/ats_strip_html_parity.rs` (6 tests).
- **`portfolio-gaps.md` at 454 lines** — career-coaching output from `check-integrity`, accumulated across 1,370+ job evaluations.
- **Autofill scaffolded but broken** — Chrome launches, navigates, the DB table works, the TUI integration works, the package cleanup works; only the actual form filling fails on React controlled components.

The README's narrative may pitch features not yet realised — the per-project file leans on LifeOS's honest scope and grade distributions to surface current state. Aspirational items belong in the Direction section.

## Current state

**Status: active**, slowed-cadence. HEAD is `aff9590` (2026-05-10 — portfolio-gaps batch findings). The most recent feature commit was 2026-04-29 (`50359b13` — timestamp format bug fix across 7 files). Post-2026-04-29 work has been grading-batch and documentation rather than structural feature commits — the system is operational; the bottleneck has shifted from build to throughput on the surfaced SS/S apply targets.

Sessions 1–7 (April 7–10, 2026) built the core system in three days: ~14k lines Rust, 408 companies, 1184 graded jobs. Sessions 8–12 (April 10 → May 10) matured the project rather than growing it — 22-factor location rubric + lifestyle modulator (session 8), 316-test retroactive pass + skills migration + code-health audit (session 9), `populate-from-lifeos` shipped + `profile-scrape` retired + preferences-integrity guard (session 10), grade-jobs realism rewrite + full post-clean-slate re-grade of 583 jobs across two batches + timestamp bug fix + second LifeOS sync (session 11), and the first end-to-end `cernio-search` + 5-parallel-bespoke-subagent batch of 164 jobs (session 12).

In-flight from LifeOS Work files: the `prepare-applications` skill needs to run against the 12 SS+S list from the 2026-05-10 batch (top SS: Amazon SDE-1 New Grad 2026 UK, Arm AI/ML Cambridge Graduate SWE, Apple Swift Compiler Backend Engineer, Apple Debugger Engineer (LLDB); top S: 5× Apple ASE wide-funnel + Squarepoint Trading Infrastructure Graduate Programme + Balyasny Tech Academy New Grad). Cloud / Kubernetes / Docker / Terraform on Cernio is the queued portfolio-gap-closure pass (containerise Rust binary → GitHub Actions CI → Lambda/Fargate deployment of `cernio search` preview → Terraform module). Vault refresh (Cernio LifeOS Overview / Gaps / Roadmap) for the session 11–12 work and the cloud-gap row is also queued.

## Gaps and known limitations

Drawn from LifeOS `Gaps.md`:

**Broken:**
- **Autofill form filling (Priority 1)** — entire pipeline built and integrated except the actual form filling. Chrome launches and navigates, the DB table works, the TUI `p` key works, the package cleanup works — but JS value injection (`el.value = "..."`) does not trigger React controlled component state on Greenhouse forms. Fix: replace JS value-injection with CDP `Input.insertText` or `nativeInputValueSetter`; test against real Greenhouse DOM; update CSS selectors; add Lever + Ashby modules. The "Chrome is being controlled by automated test software" banner also needs `--disable-blink-features=AutomationControlled` evaluated.

**Not built:**
- **Interview prep skill** — design exists in full at `context/notes/interview-prep-design.md`; implementation has not started. Would generate personalised curriculum from SS/S/A job descriptions + portfolio gaps including LeetCode-style TDD problems, multi-component systems practice, and company-specific prep materials.
- **Markdown export CLI command** — TUI `e` key works, but there is no `cernio export` for batch markdown generation.
- **Eightfold ATS fetcher** — listed in the CHECK constraint, no `src/ats/eightfold.rs` module. Only 1 company uses Eightfold; low ROI.
- **Teamtailor fetcher** — higher-ROI than Eightfold. 4 of the 17 dad-list bespoke companies use Teamtailor; the provider has a clean public API at `{slug}.teamtailor.com/jobs.json`. Implementing would convert those 4 bespoke → resolved immediately.
- **Cloud / Kubernetes / Docker / Terraform / CI-CD portfolio evidence** — the densest gap-evidence in the project. Confirmed across 5+ consecutive grading batches as the #1 employability gap. 2026-05-10 batch flagged it across 14 separate roles. Closure prescription: a weekend on Cernio itself — containerise the Rust binary, add GitHub Actions CI, deploy a preview to AWS Lambda or Fargate, write a Terraform module. Captured as a Work item in LifeOS.
- **C++ proficiency Familiar → Proficient** — primary blocker on 7+ roles in the 2026-04-29 batch (Apple JDK × 2, Apple Kafka, Citadel C++ SWE, Tower Quant Developer, QRT Low Latency Market Data, Wayve Robot Software, Wintermute C++ Quant Trading Platform). Caner's C++ is self-rated Familiar; Nyquestro and Tectra demonstrate the concepts in Rust but the Rust-to-C++ translation is undemonstrated. Closure prescription: take Tectra past its Clock-interface scaffold into a working feed-handler + matching loop, OR finish Chrona's commit DAG to a working `chrona init / commit / log` MVP.
- **Cybersecurity / cloud-security portfolio bridge** — newly named in 2026-04-29 batch 2. All 10 Wiz postings capped at C-or-below. Closure options: explicitly accept cybersecurity as a non-target sector and update `preferences.toml`, OR build a defensible security project (small CVE PoC, OSS fuzzer, CTF write-up portfolio).
- **CUDA / GPU-systems / PTX / SASS / CUTLASS / Triton / NCCL** — newly named in 2026-04-29 batch 1 (Jane Street ML Performance Engineer). Distinct from "production-scale ML" — this is GPU-kernel engineering specifically. Closure prescription: a CUDA kernel project (custom GEMM / attention kernel / matmul tiling).
- **Production-scale ML (petabyte / 10K-GPU / cloud-trained)** — confirmed A-vs-S boundary at Apple AiDP, Jane Street ML Engineer + ML Researchers, DRW ML. NeuroDrive is M2-MacBook-Air scale. Closure prescription: a one-time cloud-GPU experiment (Lambda Labs / Vast.ai).
- **Distributed-database tenure (YugabyteDB / CockroachDB / TiDB / Cloud Spanner / Iceberg / Trino)** — newly observed gap from Wise Data Platform + Balyasny DB Platform + Spotify Data Platform. Caner's SQLite-only DB work is single-node.
- **OCaml** — Jane Street uses OCaml as primary language; 8+ Jane Street roles in 2026-04-29 batch involve OCaml. Demonstrating some OCaml familiarity (typed expression tree, small interpreter, OSS contribution) would lift Jane Street from templated lottery to credible stretch.

**Incomplete:**
- **Workday integration** — fetcher exists and works, but Workday's complex URL pattern (variable subdomain `wd1`–`wd12` + site name) means resolution requires manual identification or web search; no mechanical probing. 20 companies use Workday. The Workday UK-filter silent bypass was closed in session 10 (commit `86097a6`) when the `[search_filters.locations.workday]` subtable was added alongside the new `every_supported_ats_provider_has_a_location_subtable` build-time guard.
- **Bespoke company coverage** — 138 companies are bespoke. The 2026-05-10 batch demonstrated the operational pattern (5 parallel bespoke subagents alongside `cernio search`) but coverage is still spotty.
- **Dad-list jobs-search not yet scoped to the 48-company set** — the 48 dad-list companies (commit `bee129a`, 2026-04-21) were graded standalone; the grading half is closed but a scoped `cernio search` run against the 48 plus bespoke subagents for the 17 dad-list bespoke entries is the proper closure. Dassault Systèmes (id 471) needs revisiting when its careers page repopulates.
- **Parent-company slug expansion in `cernio resolve`** — the dad-list AI fallback surfaced `LexisNexis → workday/relx` and DigitalOcean `digitalocean98` (numeric suffix). Parent-slug cases fall through to AI fallback every time, which is more expensive than a mechanical attempt.
- **Code-health audit findings — 27 open items** with 4 high-severity. None implemented yet.
- **Forward-Deployed-Engineer / Solutions-Architect title-disguise leak** — 6 F's per 2026-05-10 batch from customer-facing roles slipping through exclude_keywords.
- **"Analyst" include-keyword too broad** — 12 F's per batch from pure compliance/finance/risk-ops analyst roles.
- **Hardware/RTL/ASIC role filter not in place** — 22 F's in 2026-05-10 batch from hardware/RTL/ASIC/FPGA/MEMS/PCB/optical/mechanical/aviation roles.
- **Empty-description grad postings** — ~10 jobs per pair of 2026-04-29 batches (Microsoft × 5, Cisco × 3, Darktrace, FNZ) where `search-jobs` captures the listing but misses body text. Closure: post-step that reports `LENGTH(raw_description) < 200` and prompts re-fetch.
- **Smarkets mis-graded at company level** — self-described as "the future of betting"; three roles F'd on gambling sector but the company is still held at C. Should be archived under gambling-sector exclusion.
- **Defence-prime visa-friction not flagged at company tier** — 17+ F's in 2026-04-29 batch 2 on UK-defence-prime companies (Helsing UK × 8, Faculty AI Defence × 2, Anduril London × 3, Arondite × 2) — all categorical F per visa.md SC/DV blocker for Turkish nationality.
- **2:2-degree-class credential filter is a non-closable structural gap.** Luminance Cambridge's 5 graduate roles all hard-gate on "Top 200 Global University with First or 2:1." 2:2 from York fails categorically — opaque credential filter independent of technical fit. No technical closure; track as risk.

**Unknown / needs investigation:**
- **Chrome automation detection** — the banner appeared during autofill testing; `--disable-blink-features=AutomationControlled` may or may not be effective.
- **CSS selectors for Greenhouse forms** — selectors in `src/autofill/greenhouse.rs` were written from documentation, not real DOM inspection.
- **Long-term ATS provider stability** — slugs can break when companies migrate providers (ClearBank: Workable → Ashby). `cernio check` detects this; re-resolution is manual.
- **Company-level grades pre-date the realism semantic.** Session 7 company grades were assigned under the pre-realism rubric; some may need to re-grade to A-stretch or B under the realism lens.

**Technical debt:**
- **Dashboard is the largest single file** — `src/tui/views/dashboard.rs` is 31.5KB / 946 lines and handles grade distributions, pipeline health, action items, top roles, activity heatmap, search pulse, visa countdown, top companies leaderboard, session diff. Modularisation candidate.
- **Migration 003 has a complex fresh-DB path.** Separate code path for fresh databases that manually rebuilds the table.

## Direction (in-flight, not wishlist)

Drawn from LifeOS `Roadmap.md` — items that are actively being worked on or have a concrete near-term plan:

- **Apply to the wide-funnel SS/S targets (time-bound).** ~15 deep-customisation primary targets surfaced by the 2026-04-29 + 2026-05-10 batches: Anthropic Fellows (AI Safety / ML Systems & Performance / RL); HRT 2026 Grad SWE; Microsoft UK Graduate SWE Full-Time + MAIDAP; Bloomberg 2026 SWE + Internship London; Cloudflare grad-track interns × 4; Amazon SDE-1 New Grad 2026 UK; Apple London ASE pipeline (7/23 Apple roles SS or S in the 2026-05-10 batch); Arm AI/ML Cambridge Graduate SWE; B2C2 Graduate Quant Developer London; Graphcore Cambridge Drivers; Squarepoint Graduate; Stripe Software Engineer Intern London; Palantir SWE New Grad; Tradeweb 2026 Technology Graduate Programme; Vocalink (Mastercard) Launch Graduate Program 2026. Application work, not build work — but the highest-leverage thing Cernio's output is asking the user to do this week.
- **Close the Cloud / Kubernetes / Docker / Terraform / CI-CD gap.** Densest gap evidence in the project; flagged across 14 separate roles in the 2026-05-10 batch alone. Closure prescription is a weekend on Cernio itself — containerise Rust binary, add GitHub Actions CI, deploy a preview to AWS Lambda or Fargate, write a Terraform module. Captured as a Work item.
- **Fix Autofill (Priority 3).** Open a real Greenhouse form, inspect DOM, find actual CSS selectors. Replace JS `el.value =` with CDP `Input.insertText` or `nativeInputValueSetter`. Test on a job with a prepared package. Once Greenhouse works, add Lever and Ashby modules. Evaluate `--disable-blink-features=AutomationControlled` effectiveness.
- **Search jobs for the 48 dad-list companies** as a scoped subset. Grading half closed; `cernio search` + bespoke subagents against the 48 specifically. Revisit Dassault Systèmes (id 471).
- **Interview Prep Skill** (priority 5). Design exists in full. With the realism semantic in place the SS targets are well-calibrated — the curriculum input is sharper than it would have been pre-2026-04-29.
- **Close the C++ proficiency gap** via Tectra past Clock-interface scaffold into a working feed-handler + matching loop, OR finish Chrona's commit DAG to a working `chrona init / commit / log` MVP.
- **Code-health audit implementation batches** in the audit's recommended order: dead-code removal → `strip_html` consolidation (fixes Workable latent bug, removes 70 lines) → SQL consolidation (`fetch_stats` 16→4-6 queries + N+1 fix in `search::run_by_grade`) → retry standardisation across Ashby/Workable/Workday → `verify_ats_slugs` parallelisation + Lever probe swap → dashboard split → `fetch_jobs` list/detail split. Each batch independently testable against the 346-test baseline.
- **Tighten the search-time filter on disguised non-engineering titles.** Add "Forward Deployed", "Deployed Engineer", "Solutions Architect", "Solutions Engineer" as title-pattern hard-excludes. Tighten "Analyst" include_keyword to require pairing with "Quant", "Quantitative", "Research", or "Software".
- **Filter hardware/RTL/ASIC/FPGA roles at search time.** Extend preferences.toml exclude_keywords with `RTL`, `ASIC`, `FPGA`, `VLSI`, `Physical Design`, `Mechanical`, `Nanofabrication`, `Hardware Integration`, `Optical`, `Emulation`, `Aviation`, `Maintenance`. ~10-15% grading-load reduction with zero false-negative risk.
- **Periodic integrity check + re-search.** The search pipeline is built for periodic re-runs (`last_searched_at` on companies; TUI shows which bespoke companies need searching). `check-integrity` maintains `portfolio-gaps.md` as a side effect.

## Demonstrated skills

What this project specifically proves (anchored to the evidence in LifeOS):

- **Architecting a three-layer system with strict dependency direction and SQLite as the shared contract** — explicit layer responsibilities, no upward dependencies, idempotency on every pipeline step, WAL for read/write concurrency between TUI and pipeline scripts.
- **Implementing 6 production ATS integrations in Rust** (Greenhouse, Lever with dual US+EU endpoints, Ashby POST-based, Workable per-job detail fetch, Workday with variable subdomain + site stored in `ats_extra`, SmartRecruiters with `totalFound>0` false-positive guard) and unifying them behind a normalised `AtsJob` struct with shared retry helpers and slug normalisation.
- **Designing a filter chain with documented false-negative-aversion bias** — empty data passes through; 34+ exclusion keywords validated against 2,001 graded jobs for zero B+ false-negatives.
- **Tiered archival lifecycle design** — grade-tied active windows, archive expiry, protection for decisions and SS/S/A, archival-vs-deletion discrimination so re-discovery can happen with an updated profile.
- **SQLite migration discipline including manual table-rebuild patterns for CHECK constraint changes** — 6 idempotent migrations.
- **Building a 26-file Ratatui TUI with mouse + keyboard support, 5 views, responsive layout, GitHub-style activity heatmap, kanban pipeline, and modular widget architecture** — App-struct shared state, per-concern modules as methods, rendering separate from event handling.
- **Retroactive 316-test pass on a real codebase that surfaced two silent data-loss bugs immediately** — lib+bin split, `CERNIO_DB_PATH`, in-memory DB fixture, offline JSON fixtures over HTTP mocking, CLI integration tests via `assert_cmd`, 346 tests passing sub-second.
- **Build-time integrity guard design** that closed a multi-month silent UK-filter bypass (Workday) the moment it landed — `every_supported_ats_provider_has_a_location_subtable` invariant driving off a `SUPPORTED_ATS_PROVIDERS` constant kept in sync with `src/ats/` modules.
- **Designing and iterating a grading rubric five times against production data** — dimension scoring → hard floors → career-stage calibration → question-first → realism semantic. Each iteration driven by a concrete production failure with named DB examples (Amazon at B / Monzo at C under dimensions; 120 demotions when descriptions were finally read; 40 SS/S after clean sweep under pure question-first).
- **Operationalising the reputation × selectivity decoupling in a rubric — including worked-example pairs and detection rules** that produced 12% S+ density on a 300-job batch (vs >20% inflation pre-realism) and confirmed the Jane Street prestige-trap pattern (0 SS / 0 S / 4 A-stretch on 18 roles).
- **Designing 9 obligation-anchored Claude Code skills** with mandatory-read protocols, evidence-anchored quality checklists, What-I-Did-Not-Do declarations, and engineered triggers with negative-trigger clauses — verifiable obligations replacing exhortation language, ~290KB of structured skill documentation.
- **One-way cross-repo synchronisation architecture** (`populate-from-lifeos`, session 10) — 8-phase autonomous workflow, README-as-gatekeeper allow-list parsing, parallel per-project subagent dispatch, Tier-3 evidence-anchored synthesis contract (file path / line count / verbatim last line), pre/post-timestamp Cernio-native preservation check, never-writes-upstream invariant.
- **Two-pass repository code-health audit** that surfaced 27 actionable findings (4 high / 14 medium / 7 low / 2 triage) across 8 systems, modified zero production code, added 6 parity tests locking target semantics for the highest-severity consolidation, and shipped a 7-batch implementation sequence each independently testable against a baseline.
- **22-factor three-tier location-reasoning rubric** evaluating cities at city / country / hybrid level across current state and three trajectory horizons (1-3 / 5-7 / 10-15 years), integrated as a same-tier grading modulator that moves grades across boundaries — not just within-grade tiebreaking.
- **Asynchronous I/O multiplexing with bounded concurrency** via `tokio::Semaphore` and per-provider retry with exponential backoff — business logic stays sync, async is the I/O layer only.
- **Quote-aware HTML stripping with a 514-line idempotent format pipeline** that runs silently on every TUI startup, guarded by three invariants (no raw tags, no triple blank lines, no panics) plus an explicit realistic-payload idempotency test.
- **Diagnosing a cross-cutting bug from a symptom pattern** — the timestamp-format mismatch (`%Y-%m-%dT%H:%M:%S` vs `%Y-%m-%d %H:%M:%S`) was found via tests, traced through 7 files (`pipeline/check.rs`, `clean.rs`, `search.rs`, `tui/app/actions.rs`, `cleanup.rs`, `pipeline.rs`, `tui/queries.rs`), fixed atomically, and locked behind regression coverage.
- **Treating a profile as a living system with the Living System rule** — every skill reads the profile fresh on every invocation; profile snapshots embedded in skills are an architectural error caught at the rule layer.
- **Career-coaching feedback loop** — `portfolio-gaps.md` (454 lines) accumulates market-pattern intelligence across 1,370+ job evaluations; gaps drive concrete project-level closure recommendations (cloud-on-Cernio weekend, Tectra feed-handler completion, Chrona MVP) that loop back into improving the profile that feeds future grading.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Cernio/_Overview.md | 79 | "> Session 8 added the 22-factor location-evaluation rubric + lifestyle modulator; session 9 added 316 tests (surfacing two silent data-loss bugs), a full code-health audit with 27 findings, and migrated all 9 skills to native Claude Code integration. Session 10 shipped `populate-from-lifeos` + retired `profile-scrape` + added a 21-test `preferences_integrity.rs` guard that surfaced the silently-bypassed Workday UK-location filter. Sessions 11-12 ran the post-clean-slate full re-grading under the new realism semantic (583 jobs in two 2026-04-29 batches) plus a 164-job 2026-05-10 search-and-grade batch. Velocity slowed because depth was the goal. See [[Cernio/Session History]] for the full breakdown." |
| Projects/Cernio/Architecture.md | 254 | "**Session 11 grade-jobs realism rewrite (commit `389b1e8a`, 2026-04-29):** added reputation × selectivity decoupling, prestige-trap worked example, status enum cleanup. Search-jobs got a mini-iteration in commit `08bfa8b2`. See [[Cernio/Systems/Skills]] and [[Cernio/Systems/Grading#Phase 5 — Realism semantic 2026-04-29]]." |
| Projects/Cernio/Data Composition.md | 187 | "- [[Cernio/Roadmap]] — apply targets and filter-tightening priorities surfaced by the 2026-04-29 + 2026-05-10 batches" |
| Projects/Cernio/Decisions.md | 213 | "See [[Cernio/Systems/Grading#Phase 5 — Realism semantic 2026-04-29]] and [[Cernio/Data Composition#Batch 2026-04-29 batch 1]]." |
| Projects/Cernio/Gaps.md | 254 | "- [[Cernio/Data Composition]] — the data backing the gap evidence" |
| Projects/Cernio/Roadmap.md | 173 | "- [[Cernio/Data Composition]] — current grading state and batch composition" |
| Projects/Cernio/Session History.md | 219 | "> 5th major rubric rewrite, driven by user observation that prestige was leaking into Q2-confirmed SS. The reputation × selectivity decoupling produced calibrated grades on 583 jobs across two same-day batches (12% S+ density vs the 20%+ inflation pre-realism) and the Jane Street prestige-trap pattern was concretely confirmed in production data. Every future grade-jobs run inherits the calibration. `[verified: commit 389b1e8a, portfolio-gaps.md §Batch 2026-04-29 batch 1]`" |
| Projects/Cernio/Systems/_Overview.md | 41 | "- [[Projects/Cernio/Roadmap]] — direction-of-travel" |
| Projects/Cernio/Systems/ATS Providers.md | 145 | "- [[Cernio/Systems/Code Health]] — 7 findings open against this subsystem" |
| Projects/Cernio/Systems/Autofill.md | 90 | "- [[Cernio/Gaps]] — autofill is the #1 gap" |
| Projects/Cernio/Systems/Code Health.md | 166 | "- [[Cernio/Roadmap]] — implementation batches are queued" |
| Projects/Cernio/Systems/Config.md | 78 | "- [[Cernio/Architecture]] — no hardcoded configuration is a key architectural property" |
| Projects/Cernio/Systems/Database.md | 157 | "- [[Cernio/Systems/Code Health]] — dashboard `fetch_stats` issues 16 queries per 2s poll; SQL consolidation is a HIGH-severity audit finding" |
| Projects/Cernio/Systems/Grading.md | 195 | "- [[Cernio/Decisions#Realism semantic 2026-04-29]] — the design decision behind phase 5" |
| Projects/Cernio/Systems/Location Evaluation.md | 117 | "- LifeOS canonical: `Profile/Professional/Lifestyle Preferences.md` — Cernio's `profile/lifestyle-preferences.md` is synced from here one-way via populate-from-lifeos (session 10)" |
| Projects/Cernio/Systems/Pipeline.md | 159 | "- [[Cernio/Systems/Code Health]] — 10 open findings in this subsystem" |
| Projects/Cernio/Systems/Profile.md | 121 | "- [[Cernio/Session History#Session 10]] — the migration session" |
| Projects/Cernio/Systems/Skills.md | 167 | "- [[Cernio/Session History#Session 10]]" |
| Projects/Cernio/Systems/TUI.md | 144 | "- [[Cernio/Systems/Testing]] — Phase 6 added 34 TUI helper tests" |
| Projects/Cernio/Systems/Testing.md | 192 | "- [[Cernio/Session History#Session 11]] — timestamp format mismatch bug fixed across 7 files" |
| Projects/Cernio/Work/Application Pipeline.md | 49 | "- Cernio _Overview / Gaps / Roadmap drift (last_verified 2026-04-24) — see `[[Projects/Cernio/Work/Vault Refresh.md]]`" |
| Projects/Cernio/Work/Cloud Deployment.md | 54 | "- Related: prepare-applications follow-up on the 12 SS+S list — `[[Projects/Cernio/Work/Application Pipeline.md]]`" |
| Projects/Cernio/Work/Profile Populate Skill.md | 152 | "- LifeOS commit `cf14e1d` — Phase 1 landing commit" |
| Projects/Cernio/Work/Vault Refresh.md | 35 | "Orient on 2026-05-10 flagged this as a drift but the session that ran orient (Cernio session start) went on to search-jobs → grade-jobs → exhaustion-of-day pattern, then session wrap deferred this hygiene cut. Cheap-pass items get deferred routinely; the persistence-pin pattern would surface it in morning-brew anti-rec walk." |
