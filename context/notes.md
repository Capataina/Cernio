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
| `notes/grading-rubric.md` | Grade system evolution: dimension-weighted → question-first → project tiers + calibration-anchored. Session 5 full DB reset and rebuild (434 companies, 2001 jobs). Exclusion keyword philosophy. Token economics of proper vs title-only grading. |
| `notes/populate-db-lessons.md` | Slug guessing unreliable, SmartRecruiters false positives, ATS migrations, unsupported providers, validation catches dead companies, Lever EU domain probing, per-request retry at scale |
| `notes/scaling-architecture.md` | Scripts for volume, AI for judgment, every step has one purpose. Batch grading prioritised by signal. False negatives are the enemy. Full plan in `plans/pipeline-separation.md` |
| `notes/tui-design.md` | Design principles (dynamic/density/mouse-first/grade-primary), bar charts rationale, responsive layout (implemented), session summary approach, scroll behaviour |
| `notes/db-maintenance.md` | Tiered archival lifecycle (SS=28d, S=21d, A=14d, B=7d, C/F=3d), archive expiry after 14 days, unarchive command with timer reset, no auto-archive of companies |
| `notes/interview-prep-design.md` | Future interview-prep skill: personalised curriculum from SS/S/A jobs + portfolio gaps, LeetCode-style TDD problems, systems practice with integration tests, company-specific prep |
