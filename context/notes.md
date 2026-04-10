# Notes Index

Design decisions, preferences, and lessons from sessions. Each file captures a distinct topic — read `architecture.md` first for structural orientation, then dive into these for the reasoning behind specific choices.

| File | Summary |
|------|---------|
| `notes/collaborative-model.md` | Session model (not pipeline), scripts for volume / Claude for judgment, TUI as live dashboard, export on confirmation |
| `notes/discovery-design.md` | Broad not filtered, creative search strategies, parallelisation, separate from resolution, TrueUp reference model |
| `notes/data-layer.md` | SQLite as source of truth, field categories (facts/checkpoints/judgments), what lives where, safety |
| `notes/profile-system.md` | Auto-update from repos, startup reading, career coaching, portfolio gaps, describe built not planned, preferences flexibility |
| `notes/skill-architecture.md` | Project-specific skills in repo, conversational invocation, research-guided design, mandatory-read protocol, question-first rubric rewrite with mandatory description citation |
| `notes/job-search-strategy.md` | Job titles are unreliable for filtering, Claude must read full descriptions, discovery must read DB before searching, companies may use multiple ATS |
| `notes/grading-rubric.md` | Grade system evolution: dimension-weighted → question-first → project tiers + calibration-anchored. Session 5 full DB reset and rebuild. Session 6 exclusion purge (2001→937 jobs, 408 companies). Session 7: added Sr./Lead exclusion keywords (51 jobs leaked), full 206-job grading run. Token economics of proper vs title-only grading. |
| `notes/populate-db-lessons.md` | Slug guessing unreliable, SmartRecruiters false positives, ATS migrations, unsupported providers, validation catches dead companies, Lever EU domain probing, per-request retry at scale |
| `notes/scaling-architecture.md` | Scripts for volume, AI for judgment, every step has one purpose. Batch grading prioritised by signal. False negatives are the enemy. Full plan in `plans/pipeline-separation.md` |
| `notes/tui-design.md` | Design principles (dynamic/density/mouse-first/grade-primary), bar charts rationale, responsive layout, session summary approach, scroll behaviour. Session 7: modularisation rationale, component architecture, one-line kanban cards, activity heatmap, quick-peek popup pattern |
| `notes/db-maintenance.md` | Tiered archival lifecycle, archive expiry, unarchive, `cernio format` (HTML→plaintext, idempotent, runs on TUI startup), application_packages cleanup |
| `notes/autofill-status.md` | Autofill architecture: Chrome CDP via chromiumoxide, per-provider modules, application_packages DB table, prepare-applications skill. Status: scaffolded but broken (React form filling). Fix approach documented. |
| `notes/interview-prep-design.md` | Future interview-prep skill: personalised curriculum from SS/S/A jobs + portfolio gaps, LeetCode-style TDD problems, systems practice with integration tests, company-specific prep |
| `notes/bespoke-search-workflow.md` | Bespoke search results MUST be inserted into the jobs table — not just reported conversationally. Agents search career pages + aggregators, found jobs get INSERT OR IGNORE by URL. |
| `notes/testing-strategy.md` | No tests exist yet. Plan: unit tests (`cargo test`) for DB/CLI/formatting/TUI state, integration tests for end-to-end CLI + pipeline flows against test DB. Planned for 2026-04-11. |
