---
name: Cernio
status: active
source_repo: https://github.com/Capataina/Cernio
lifeos_folder: Projects/Cernio
last_synced: 2026-04-29
sources_read: 21
---

# Cernio

## One-line summary

Local-first, conversationally-driven job discovery and curation engine in Rust — six ATS provider fetchers, a six-command pipeline, a SQLite contract layer, a 26-file Ratatui TUI dashboard, and nine native Claude Code skills, deliberately split so scripts handle volume and the AI layer handles judgment.

## What it is

Cernio is a single-developer Rust system that finds, evaluates, and curates job opportunities by combining a SQLite database, a Ratatui terminal UI, six ATS provider fetchers (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday — Eightfold accepted by the schema CHECK constraint but with no fetcher module), six pipeline scripts (resolve / search / clean / check / import / format), and nine Claude Code skills installed as native `.claude/skills/`. The architectural commitment that runs through every part of the system is that **nothing is automated end-to-end** — every action happens inside a conversational session where the user and Claude decide together what to do, scripts handle volume (probing thousands of ATS slug combinations, scanning hundreds of company boards), and the AI layer handles judgment (reading job descriptions, grading fit against a structured profile, maintaining a portfolio-gap feedback loop). The project is local-first by design: a single SQLite file, no Docker, no server, no API keys for the core pipeline, with the AI layer running through Claude Code sessions rather than hosted inference. Sessions 1–7 (April 7–10 2026) built the core product end-to-end; sessions 8–9 (April 10–21 2026) matured it with a 22-factor location-evaluation rubric, a 306-test retroactive testing pass that surfaced two silent data-loss bugs, a full code-health audit with 27 open findings, and a migration of all nine skills onto the native Claude Code Skill tool framework with obligation-anchored quality checklists.

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

**Layer responsibilities:**

| Layer | Does | Does not |
|-------|------|----------|
| Conversation | Orchestrates skills and scripts, evaluates jobs against profile, recommends actions, tracks portfolio gaps | Submit applications, make decisions without user input |
| Rust scripts | Combinatorial volume: scan ATS boards, probe slug patterns, fetch job JSON, generate exports | Make judgments, know about the profile, decide what to search for |
| TUI | Real-time display of company universe, evaluation progress, and user decisions; markdown export on keypress | Run scripts, evaluate jobs, or modify data independently |
| SQLite | Contract between all layers — single source of truth for structured data | Contain business logic |

**Crate shape (session-9 lib+bin split).** `src/main.rs` is a thin shim over `src/lib.rs`. The split is load-bearing: Rust integration tests under `tests/` can only see public items from a library crate, not a binary-only crate. Every new top-level module must be declared in `lib.rs`. The CLI binary now reads `CERNIO_DB_PATH` with fallback to `state/cernio.db` so integration tests can target tempdir DBs.

## Subsystems and components

| Subsystem | Responsibility | Key files |
|---|---|---|
| **Database** | 5 tables, 6 migrations, WAL mode, idempotent migrations | `src/db/schema.rs` |
| **ATS providers** | Per-provider HTTP fetchers + slug normalisation; Eightfold bespoke | `src/ats/*.rs` |
| **Pipeline** | 6 CLI commands; idempotent; filter chain (location → exclusion → inclusion) | `src/pipeline/*.rs` |
| **TUI** | 5-view dashboard with mouse, responsive layout, 2s SQLite poll | `src/tui/` |
| **Skills** | 9 project-local skills with mandatory-read protocols + reference docs | `.claude/skills/` |
| **Grading** | Question-first calibration-anchored rubric for company + job grading | rubric in `.claude/skills/grade-*/references/` |
| **Location evaluation** | Session-8 22-factor three-tier rubric + lifestyle modulator | `context/references/location-master.md` |
| **Autofill** | Chrome CDP automation, scaffolded but broken on React forms | `src/autofill/` |
| **Profile system** | Auto-sync from LifeOS via `populate-from-lifeos`; portfolio-gaps feedback loop | `profile/` + `populate-from-lifeos` skill |

## Technologies and concepts demonstrated

### Languages
- **Rust 2024** — primary language across pipeline, TUI, ATS fetchers, autofill, tests; ~14k lines, 56 files; lib+bin split for integration testing.

### Frameworks and libraries
- **Tokio** — async runtime for pipeline scripts (resolve, search, clean, check); Semaphore for parallel-fetch concurrency control.
- **Reqwest + rustls-tls** — HTTP client with retry helpers (per-request exponential backoff in `common.rs::get_with_retry`).
- **Ratatui 0.29 + Crossterm** — TUI rendering, mouse support, responsive layout.
- **rusqlite** (bundled, WAL) — SQLite driver with migration framework.
- **Serde / serde_json / toml 0.8** — JSON for ATS responses, TOML for `preferences.toml` config.
- **chromiumoxide** — Chrome DevTools Protocol automation (autofill, scaffolded).
- **assert_cmd / proptest / tempfile / predicates** — integration test infrastructure.

### Tools
- **Git / GitHub** — version control, native Claude Code Skill tool integration via `.claude/skills/`.
- **Cargo** — workspace, dependency management, cross-crate testing.

### Domains and concepts
- **Conversational AI orchestration** — scripts-handle-volume / AI-handles-judgment split; mandatory-read protocols; obligation-anchored quality gates; Living System Philosophy (skills never embed profile snapshots, every grade tied to current profile state).
- **Local-first architecture** — single SQLite file, no Docker/server, no API keys for the core pipeline, archive-not-delete semantics for reversibility.
- **ATS portal probing** — slug-pattern enumeration across 7 providers; AI fallback for unmatchable companies; bespoke marker for unsupported providers (iCIMS, Taleo, Personio).
- **Question-first grading** — replaced dimension-weighted scoring with calibration-anchored rubrics (4 iterations driven by production failures with real data).
- **Filter design with false-negatives-as-enemy bias** — empty data passes every filter; missed-good-role unrecoverable, mis-included-irrelevant-role costs 30 seconds.
- **Idempotent pipeline operations** — every CLI command safe to re-run; format on every TUI launch is a guarded contract (`idempotency_on_realistic_payload` test).
- **Skill ecosystem engineering** — 9 specialised skills with reference documentation, evidence-anchored mandatory-read tables, what-I-did-not-do declarations, Tier-3 quality checklists.

## Key technical decisions

- **Collaborative, not automated.** Original README described a daily `cernio refresh` cron-style pipeline; revised in session 1 to keep the user in every decision loop.
- **Scripts for volume, AI for judgment.** Most fundamental architectural split. Scripts are generic + parameterised; intelligence lives in conversation, not in scripts.
- **SQLite as single source of truth.** Evaluated against markdown, JSON/JSONL, Postgres/MySQL. SQLite won on: zero ops, single file, full SQL, WAL for concurrent access, trivial backup. Profile data stays in markdown (human-edited); companies/jobs/decisions/packages live in SQLite (machine-managed).
- **Question-first grading over dimension-weighted scoring.** Four iterations: dimension-weighted → hard floors → career-stage calibration → question-first. Each driven by production failures.
- **C-tier companies stay active.** Originally auto-archived; changed in session 4. Cost of extra jobs is grading time (cheap); cost of missing a good role is unrecoverable.
- **False negatives are the enemy.** Bias toward inclusion at every filter stage.
- **Mandatory-read protocol for skills.** Added session 3 after agents skipped reference files and produced shallow output.
- **Lib+bin split for testability (session 9).** Integration tests under `tests/` can only see public items from a library crate. Unblocked 306 new tests + surfaced two silent data-loss bugs.
- **`CERNIO_DB_PATH` env var for CLI testability (session 9).** Smallest change to make `tests/cli.rs` viable.
- **Lifestyle fit as same-tier grading modulator (session 8).** `lifestyle-preferences.md` reads alongside the profile; lifestyle moves grades across boundaries, not just within-grade.
- **Native Claude Code skills at `.claude/skills/` (session 9).** Migrated 9 skills from `skills/` for Skill tool auto-discovery.
- **Obligation-anchored over exhortation-anchored (session 9).** Replaced "be thorough" framing with verifiable obligations — produce artefact X, emit section Y, quote the last line of each reference.
- **No-inline-rationale convention.** Design rationale lives in `context/` and skill reference files, not in inline `// WHY ...` comments.

## What is currently built

- 6 ATS provider fetchers in `src/ats/` plus Eightfold marker; ~14k lines Rust across 56 files.
- 6 pipeline CLI commands (resolve/search/clean/check/import/format) — all idempotent, all parameterised.
- 5 SQLite tables, 6 migrations, WAL mode, 29 inline schema tests.
- 9 project-local skills at `.claude/skills/` — all skill-creator-audited.
- 5-view TUI (Dashboard, Companies, Jobs, Pipeline, Activity) — 26 source files, mouse support, responsive layout, markdown export.
- 325-test suite across inline + integration + CLI (was 18 at session 7 — session-9 added 306, surfaced two silent data-loss bugs).
- 22-factor location-evaluation rubric (session 8) with lifestyle modulator.
- Code-health audit with 27 open findings (4 HIGH, 14 MEDIUM, 7 LOW).
- 456 companies, 1184+ jobs (session-7 distribution: 13 SS, 27 S, 70 A, 142 B, 20 C, 212 F).

## Current state

Active. HEAD as of LifeOS verification commit `bee129a` (2026-04-21). Sessions 8–9 closed sessions of structural maturation rather than feature growth. Pending: code-health-audit Batch 1 (dead code), grading the 48 dad-list companies' jobs, autofill React-form fix.

In flight from `Work/Profile Populate Skill.md`: the `populate-from-lifeos` skill itself — a sync from LifeOS Profile/ to Cernio profile/, replacing the legacy `profile-scrape` skill that scraped GitHub directly.

## Gaps and known limitations

- **Autofill form filling broken** — Chrome CDP launches and navigates, but JS `el.value =` does not trigger React controlled-component state on Greenhouse forms. Fix path known: replace with `Input.insertText` or `nativeInputValueSetter`.
- **Eightfold ATS fetcher missing** — schema accepts `eightfold` but `src/ats/eightfold.rs` does not exist (only 1 company uses it; ROI low).
- **Workday slug resolution non-mechanical** — variable subdomain + site name require manual identification or web search.
- **Bespoke company coverage spotty** — 121 bespoke companies need AI-agent search.
- **Code-health audit findings open** — 27 actionable findings (4 HIGH-severity): four divergent `strip_html` implementations (latent Workable correctness bug), N+1 query in `search::run_by_grade` (288 round-trips per grade-scoped search), `fetch_stats` 16 SQL queries per 2s TUI poll, SmartRecruiters pagination missing `get_with_retry`.
- **Dashboard is largest single file** — `tui/views/dashboard.rs` is 31.5KB.
- **Provider-name strings shared across 4 places** without single source of truth (`ats/`, `config.rs`, `preferences.toml`, `db.ats_provider` CHECK constraint).

## Direction (in-flight, not wishlist)

- **Priority 1 — Fix autofill.** Replace JS value injection with CDP `Input.insertText`. Test on real Greenhouse form. Add Lever and Ashby modules.
- **Priority 2 — Interview prep skill.** Design exists in `context/notes/interview-prep-design.md`.
- **Priority 3 — Grade dad-list jobs.** 48 companies graded as companies (0 S / 7 A / 26 B / 15 C); jobs not yet searched.
- **Priority 4 — Code-health audit Batch 1 (dead code) → Batch 2 (`strip_html` consolidation, fixes Workable bug, removes 70 lines).**
- **Priority 5 — Integrity check post-session.** `check-integrity` skill after next grading run.
- **Priority 6 — Periodic re-search.** Operational rhythm of `cernio search` across resolved companies + bespoke AI search.

## Demonstrated skills

- **Building a non-trivial Rust application end-to-end** — ~14k lines, 56 files, lib+bin crate structure, integration test infrastructure with `assert_cmd` + `CERNIO_DB_PATH`.
- **Designing a multi-provider ATS abstraction** — 6 fetchers behind a common interface, per-provider quirks (Workday's variable subdomain+site, SmartRecruiters' `totalFound>0` check, Lever's US+EU endpoints) handled cleanly.
- **Architecting a conversational + script + TUI + DB system** — strict three-layer architecture with SQLite as the shared contract; dependency direction enforced; idempotency guaranteed at every layer.
- **Engineering a production-grade skill ecosystem** — 9 skills with mandatory-read protocols, evidence-anchored quality gates, obligation-anchored framing, what-I-did-not-do declarations.
- **Writing a 22-factor location-evaluation rubric with same-tier grade modulation** — multi-agent synthesis (10 agents), three-tier framework, lifestyle modulator.
- **Retroactive testing of a 7-session codebase** — added 306 tests in one session across 6 phases; surfaced two silent data-loss bugs immediately.
- **Running a code-health audit on a self-authored codebase** — 27 categorised findings, evidence-anchored, plans queued by batch dependency.
- **Reasons about coupling and shared-string boundaries** — provider-name strings as shared identifier across `ats/`, `config.rs`, `preferences.toml`, and `db.ats_provider` CHECK constraint.
- **Treats false negatives as the enemy** in filter design — empty data passes every filter, with explicit reasoning.

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/Cernio/_Overview.md | 102 | "> Session 8 added the 22-factor location-evaluation rubric + lifestyle modulator; session 9 added 316 tests (surfacing two silent data-loss bugs), a full code-health audit with 27 findings, and migrated all 9 skills to native Claude Code integration. Velocity slowed because depth was the goal. See [[Cernio/Session History#Session 9]] for the full breakdown." |
| Projects/Cernio/Architecture.md | 245 | "Commits `319ed60` → `1c9ab85` (sessions 9) shipped per-skill skill-creator iterations. `CLAUDE.md` migrated to the principal-engineer personality (commit `ce24790`), merging Cernio's Living System Philosophy, skill-execution protocol, grade-quality standard, and portfolio-gap tracking doctrines. See [[Cernio/Systems/Skills]]." |
| Projects/Cernio/Data Composition.md | 151 | "- [[Cernio/Session History]] — how the data grew across sessions" |
| Projects/Cernio/Decisions.md | 180 | "See [[Cernio/Systems/Skills#Skill Architecture Decisions]]." |
| Projects/Cernio/Gaps.md | 132 | "- [[Cernio/Systems/Testing]] — what is NOT tested and why" |
| Projects/Cernio/Roadmap.md | 111 | "- [[Cernio/Session History]] — what's been done so far" |
| Projects/Cernio/Session History.md | 144 | "> 18 → 325 tests in one session surfaced two silent data-loss bugs immediately, produced the confidence baseline the code-health audit needed, and now blocks the kind of regression that would have gone unnoticed during sessions 1-7. Every future session benefits from this pass. `[commits 89b37e1, 978be7d, 12897aa]`" |
| Projects/Cernio/Systems/_Overview.md | 49 | "- [[Projects/Cernio/Roadmap]] — direction-of-travel" |
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
