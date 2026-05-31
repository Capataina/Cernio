# Staleness report — 2026-05-31

> Snapshot from the latest `upkeep-context` pass. Overwritten on every run; not an accumulating log.

## Per-file staleness

| File | Verdict | Evidence |
|---|---|---|
| context/architecture.md | needs-updating → updated | "Last updated: 2026-04-26" predated 14 commits since 2026-05-29; web section referred to monolithic `handlers/{jobs,companies}.rs`; `passes_location` signature stale; no mention of lane backfill, grade wipe, ethical-exclusion deletions, preferences refactor, `no_cache_static`, new CSS bundles. |
| context/notes.md | needs-updating → updated | Index missing `maud-attribute-gotchas.md` and `web-frontend-architecture.md`. |
| context/notes/web-frontend-architecture.md | needs-updating → updated | Module-structure table named handlers as single files; reality is `{mod,filters,charts,table,page,lanes_view}.rs` per page. Asset bundle inventory and snap PAGES list were stale. |
| context/notes/maud-attribute-gotchas.md | up-to-date | Bracket-syntax fix still matches `handlers/{jobs,companies}/filters.rs`. |
| context/notes/autofill-status.md | up-to-date | No autofill changes in window. |
| context/notes/bespoke-search-workflow.md | up-to-date | No bespoke-search changes in window. |
| context/notes/collaborative-model.md | up-to-date | Doctrine unchanged. |
| context/notes/data-layer.md | up-to-date | Schema-philosophy claims still hold. |
| context/notes/db-maintenance.md | needs-updating → updated | `remove_job_grades` → `archive_job_grades` rename (commit aebd701). |
| context/notes/discovery-design.md | up-to-date | Broad-then-filter framing still correct. |
| context/notes/grading-rubric.md | up-to-date | Lane-relative refactor already documented. |
| context/notes/interview-prep-design.md | preserved | Future-skill design pending. |
| context/notes/job-search-strategy.md | up-to-date | Still matches `src/pipeline/search.rs`. |
| context/notes/location-rubric.md | up-to-date | Tier reasoning matches canonical `career-goals.md` table. |
| context/notes/populate-db-lessons.md | up-to-date | Provider quirks still hold. |
| context/notes/profile-system.md | needs-updating → updated | Called out that `[hard]`/`[soft]` retired; `career-goals.md` now owns ethical exclusions + tier table + lane list. |
| context/notes/scaling-architecture.md | up-to-date | Volume/judgment principle unchanged. |
| context/notes/skill-architecture.md | up-to-date | No skill-architecture changes in window. |
| context/notes/testing-strategy.md | needs-updating (deferred) | Test count drift (316 → 382); narrow fix deferred for the next testing pass to avoid lossy rewrite. |
| context/notes/tui-design.md | up-to-date | TUI source not touched in window. |
| context/systems/ats.md | needs-updating → updated | §"Contract with config" mentioned per-provider lookup; collapsed to shared list. |
| context/systems/database.md | needs-updating → updated (lane-schema delta noted) | Lane-aware schema additions (lanes, sponsors_uk, pinnacle_status_per_lane) added as a delta paragraph. Full table re-spec deferred until the lane refactor settles. |
| context/systems/pipeline.md | needs-updating → updated | `LocationConfig` per-provider lookup collapsed; `archive_job_grades` rename noted; grade-wipe semantic captured. |
| context/systems/tui.md | up-to-date | TUI source not touched in window. |
| context/plans/cernio-full-refactor.md | preserved | Implementation has progressed (lane DB, grade wipe) but full plan still in flight; not silently ticked this pass. |
| context/plans/code-health-audit/PASS-1-CHECKPOINT.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/PASS-2-SYSTEMS-AUDITED.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/index.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/obligation-evidence-map.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/ats.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/cross-cutting.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/dead-code-sweep.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/modularisation.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/other-systems.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/pipeline.md | preserved | Older audit artefacts. |
| context/plans/code-health-audit/findings/tui.md | preserved | Older audit artefacts. |
| context/references/greenhouse-api.md | up-to-date | External API ref. |
| context/references/greenhouse-form-anatomy.md | up-to-date | Autofill-related; autofill untouched. |
| context/references/location-master.md | up-to-date | Already touched in commit f592ca2. |
| context/references/smartrecruiters-api.md | up-to-date | External API ref. |
| context/references/workable-api.md | up-to-date | External API ref. |
| context/references/location-search/agent-01.md | preserved | Research artefact (10-agent location synthesis input). |
| context/references/location-search/agent-02.md | preserved | Research artefact. |
| context/references/location-search/agent-03.md | preserved | Research artefact. |
| context/references/location-search/agent-04.md | preserved | Research artefact. |
| context/references/location-search/agent-05.md | preserved | Research artefact. |
| context/references/location-search/agent-06.md | preserved | Research artefact. |
| context/references/location-search/agent-07.md | preserved | Research artefact. |
| context/references/location-search/agent-08.md | preserved | Research artefact. |
| context/references/location-search/agent-09.md | preserved | Research artefact. |
| context/references/location-search/agent-10.md | preserved | Research artefact. |
| context/test-runs/test-grade-jobs-2026-05-15-0138.md | preserved | Test-grading baseline. |
| context/test-runs/test-grade-jobs-2026-05-15-0242.md | preserved | Test-grading baseline. |
| context/test-runs/test-grade-jobs-2026-05-15-1058.md | preserved | Test-grading baseline. |
| context/test-runs/test-grade-jobs-2026-05-18-1504.md | preserved | Test-grading baseline. |
| context/test-runs/test-grade-jobs-2026-05-18-1830-iter2.md | preserved | Test-grading baseline. |
| context/test-runs/test-grade-jobs-2026-05-18-1920-iter2-rep.md | preserved | Test-grading baseline. |
| context/test-runs/test-grade-jobs-2026-05-18-2200-iter3.md | preserved | Test-grading baseline. |
| context/notes/css-grid-absolute-positioning.md | new | Captured the `grid-area:1/1/-1/-1` containing-block trap from commit 3baaea0 + the `.row-title-text { display:block }` hover-bar bug. |
| context/notes/lane-aware-universe-rebuild.md | new | Captures the grade-wipe + sponsor-only universe + discovery + ethical-exclusion deletion as one coherent rebuild. |
| context/notes/snap-self-driven-debug.md | new | Captures the snap-CLI-as-self-driven-visual-verification pattern (intent, output layout, when to use). |

**Total files walked: 47.**

## Coverage gaps

| Repository area | Inferred system name | Proposed filename | Why it deserves a file |
|---|---|---|---|
| `src/web/` (mod + templates + handlers/* + debug_snap, ~5,165 LOC + 4,428 LOC CSS + JS) | web | systems/web.md | Second user-facing surface, sister to TUI; 22 Rust files; routes, asset bundles, JSON islands, chip filters, drawer, cmdk, snap CLI. Currently lives only in `notes/web-frontend-architecture.md` (rationale shape, not implementation-state shape). Recommended for a future Restructure pass; not silently created this Upkeep. |
| `src/data/` (lane.rs, analytics, events) | data | systems/data.md | Pure-data helpers shared by TUI and Web — `lane.rs` is the single source of truth for lane keys/colours/labels referenced from 4 distinct rendering surfaces. No canonical home today. |
