# Notes Index

Design decisions, preferences, and lessons from sessions. Each file captures a distinct topic — read `architecture.md` first for structural orientation, then dive into these for the reasoning behind specific choices.

| File | Summary |
|------|---------|
| `notes/collaborative-model.md` | Session model (not pipeline), scripts for volume / Claude for judgment, TUI as live dashboard, export on confirmation |
| `notes/discovery-design.md` | Broad not filtered, creative search strategies, parallelisation, separate from resolution, TrueUp reference model |
| `notes/data-layer.md` | SQLite as source of truth, field categories (facts/checkpoints/judgments), what lives where, safety |
| `notes/profile-system.md` | LifeOS-canonical / Cernio-downstream one-way flow; README as gatekeeper; status (not tier) weighting; Cernio-native files (preferences.toml, career-goals.md, portfolio-gaps.md) off-limits to sync; skills.md derived not hand-maintained; anti-puffing; Living System Philosophy |
| `notes/skill-architecture.md` | Project-specific skills in repo, conversational invocation, research-guided design, mandatory-read protocol, question-first rubric rewrite with mandatory description citation |
| `notes/job-search-strategy.md` | Job titles are unreliable for filtering, Claude must read full descriptions, discovery must read DB before searching, companies may use multiple ATS |
| `notes/grading-rubric.md` | Grade system evolution: dimension-weighted → question-first → calibration-anchored → status-based → lane-relative. No hardcoded calibration anchors; calibration emerges within the dataset. |
| `notes/populate-db-lessons.md` | Slug guessing unreliable, SmartRecruiters false positives, ATS migrations, unsupported providers, validation catches dead companies, Lever EU domain probing, per-request retry at scale |
| `notes/scaling-architecture.md` | Scripts for volume, AI for judgment, every step has one purpose. Batch grading prioritised by signal. False negatives are the enemy. |
| `notes/tui-design.md` | Design principles (dynamic/density/mouse-first/grade-primary), bar charts rationale, responsive layout, session summary approach, scroll behaviour. Modularisation rationale (session 7), one-line kanban cards, activity heatmap, quick-peek popup. |
| `notes/db-maintenance.md` | Tiered archival lifecycle, archive expiry, unarchive, `cernio format` (HTML→plaintext, idempotent, runs on TUI startup), application_packages cleanup, `archive_job_grades` rename |
| `notes/autofill-status.md` | Autofill architecture: Chrome CDP via chromiumoxide, per-provider modules, application_packages DB table, prepare-applications skill. Status: scaffolded but broken (React form filling). Fix approach documented. |
| `notes/interview-prep-design.md` | Future interview-prep skill: personalised curriculum from SS/S/A jobs + portfolio gaps, LeetCode-style TDD problems, systems practice with integration tests, company-specific prep |
| `notes/bespoke-search-workflow.md` | Bespoke search results MUST be inserted into the jobs table — not just reported conversationally. Agents search career pages + aggregators, found jobs get INSERT OR IGNORE by URL. |
| `notes/testing-strategy.md` | Test suite across 6+ phases (lib+bin split, format/config/slug/ATS/DB/pipeline/CLI/preferences-integrity). Inline unit tests for private logic, `tests/` integration tests for public flows + CLI binary via `assert_cmd` with `CERNIO_DB_PATH`. |
| `notes/location-rubric.md` | Location reasoning framework: tier system (London + commute belt → stretch hybrid → relocation candidates), lifestyle modulator, false-positive tolerance |
| `notes/maud-attribute-gotchas.md` | `selected[bool]` brackets for HTML boolean attributes (Maud's `attr=(false)` emits `selected="false"` which is browser-truthy); audit pattern `grep -rn 'selected=('` |
| `notes/web-frontend-architecture.md` | Web frontend (`cernio web`): axum + maud + HTMX + ECharts; shared CSS/JS bundles + per-page bundles via `PageAssets`; chip filter system; URL-persistent detail drawer; JSON islands; Cmd-K palette; snap CLI; lane-view (`?view=lanes`); modular handler split (jobs/companies into {mod,filters,charts,table,page,lanes_view}); `no_cache_static` middleware |
| `notes/css-grid-absolute-positioning.md` | `grid-area:1/1/-1/-1` on absolute children pins them to the implicit grid track, not the padding-box. Two companion bugs (display:block link border + variable-row-height accent geometry) captured from commit `3baaea0`. |
| `notes/lane-aware-universe-rebuild.md` | The 2026-05-29 → 2026-05-31 rebuild: preferences refactor + shared location list + ethical-exclusion deletion + 9-agent discovery (687→892) + grade wipe. Grade-wipe philosophy: wipe only grade-derived columns; preserve expensive-to-rederive labour (lanes, sponsor verification, ATS slugs). |
| `notes/snap-self-driven-debug.md` | `cernio snap` as Claude's self-driven visual verification loop — spawns ephemeral server, drives headless Chrome over `PAGES` const, captures PNGs into `/tmp/cernio-debug/<ts>/`. Discipline: run it yourself; don't ask the user to verify visually. |
