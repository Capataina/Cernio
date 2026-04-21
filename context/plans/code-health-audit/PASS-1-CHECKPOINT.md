# Pass 1 Checkpoint — Cernio code-health-audit

**Date:** 2026-04-21
**Stack:** Rust edition 2024; 56 Rust source files; Tokio + reqwest + rusqlite + ratatui 0.29 + chromiumoxide

---

## 1. Context ingested

Fresh reads completed during the session-9 upkeep 20 minutes before this audit began, so the inventory is current:

- `context/architecture.md` — rewritten 2026-04-21 with Inter-System Relationships, Critical Paths, Coverage. 388 lines.
- `context/notes.md` + all 16 `context/notes/*.md` — design rationale, preferences, durable lessons.
- `context/systems/{ats,pipeline,database,tui}.md` — canonical subsystem docs. `ats.md` and `pipeline.md` are new this session.
- `context/references/{greenhouse,smartrecruiters,workable}-api.md` — API contract references.

Key project preferences relevant to audit scope:

- Scripts-for-volume / Claude-for-judgment split (`notes/scaling-architecture.md`) — scripts must not make decisions; pipeline modules are deliberately procedural.
- Reasoning-based grading, no scoring formulas (`notes/grading-rubric.md`) — audit must not propose score-collapsing into numeric rules.
- `cernio format` must be idempotent (`notes/db-maintenance.md`) — any format change must preserve `format(format(x)) == format(x)`.
- Per-request retry at scale (`notes/populate-db-lessons.md`) — do not propose batch-level retry.
- Multi-portal schema is load-bearing (`notes/job-search-strategy.md`) — ClearBank Ashby + Workable case; do not propose collapsing to single portal.

## 2. Test baseline (`scripts/test_baseline.sh .`)

**Exit code:** 0. **Baseline: CLEAN — 319 tests passing, 0 failing, 0 ignored.**

| Test target | Count | Result |
|-------------|-------|--------|
| `src/*` inline unit tests | 273 | pass |
| `tests/cli.rs` | 16 | pass |
| `tests/pipeline_clean.rs` | 11 | pass |
| `tests/pipeline_format.rs` | 5 | pass |
| `tests/pipeline_import.rs` | 12 | pass |
| `tests/smoke.rs` | 2 | pass |
| **Total** | **319** | **all pass** |

No pre-existing failures → no Known Issues findings from baseline. Full suite under 2 seconds once compiled; diagnostic-test writing in Pass 2 has effectively zero runtime cost.

`cargo clippy --lib --bins --tests -- -W clippy::all` produced 44 lib + 1 bin + 2 test = **47 warnings**, distribution:

| Warning class | Count |
|---------------|-------|
| collapsible_if | 27 |
| map_or simplification | 5 |
| field_reassign_with_default (test-only) | 3 |
| redundant_closure | 2 |
| io::Error::other suggestion | 1 |
| useless_format | 1 |
| impl can be derived | 1 |
| if has identical blocks ← **investigate in Pass 2** | 1 |
| manual Option::map | 1 |
| redundant type casts (usize → usize, u16 → u16) | 2 |
| push immediately after Vec::new | 1 |
| "all variants have same prefix: By" (SortMode) | 1 |
| boolean expression can be simplified | 1 |

Most are style (collapsible_if dominates). The `if has identical blocks` hit is worth a Pass-2 read — it is the classic signature of either dead branches or missed refactors. Captured as Known-Issues finding candidate.

## 3. Modularisation candidates (`scripts/modularisation_candidates.py .`)

**15 files qualify** (Rust threshold: ≥350 lines OR top-decile of 56 files):

| Path | Lines | Qualifier | Verdict (Pass 2) |
|------|-------|-----------|------------------|
| `src/pipeline/format.rs` | 1274 | ≥350, top-decile | Pass 2 |
| `src/db/schema.rs` | 1011 | ≥350, top-decile | Pass 2 |
| `src/tui/views/dashboard.rs` | 946 | ≥350, top-decile | Pass 2 |
| `src/pipeline/resolve.rs` | 668 | ≥350, top-decile | Pass 2 |
| `src/tui/views/jobs.rs` | 595 | ≥350, top-decile | Pass 2 |
| `src/tui/queries.rs` | 540 | ≥350, top-decile | Pass 2 |
| `src/config.rs` | 535 | ≥350 | Pass 2 |
| `src/tui/views/companies.rs` | 438 | ≥350 | Pass 2 |
| `src/pipeline/search.rs` | 426 | ≥350 | Pass 2 |
| `src/ats/smartrecruiters.rs` | 411 | ≥350 | Pass 2 |
| `src/main.rs` | 409 | ≥350 | Pass 2 |
| `src/ats/lever.rs` | 409 | ≥350 | Pass 2 |
| `src/pipeline/check.rs` | 400 | ≥350 | Pass 2 |
| `tests/pipeline_clean.rs` | 375 | ≥350 | Pass 2 (test file — candidate for split) |
| `src/ats/greenhouse.rs` | 360 | ≥350 | Pass 2 |

Every candidate receives a per-file verdict (`split-recommended` / `leave-as-is` / `not-applicable`) in Pass 2 or the `findings/modularisation.md` file.

## 4. Import graph summary (`scripts/import_graph.py . --top 30`)

Top load-bearing files by fan-in:

| Rank | Path | Fan-in | Role |
|------|------|--------|------|
| 1 | `src/tui/app/mod.rs` | 19 | Central App struct — every TUI handler/view imports from it |
| 2 | `src/ats/common.rs` | 9 | AtsJob + retry helpers — every fetcher depends on it |
| 3 | `src/ats/mod.rs` | 7 | Module dispatch |
| 4–8 | Each ATS provider module | 4 | Called from pipeline + mod.rs + tests |

Top integration hubs by fan-out:

- `src/pipeline/search.rs` — 8 (imports every ATS provider + config + db)
- `src/tui/views/mod.rs` — 8 (draws all 5 views)
- `src/tui/app/mod.rs` — 7 (paired high fan-in + high fan-out: God object candidate)
- `src/ats/mod.rs` — 7 (re-exports all providers)
- `src/lib.rs` — 7 (library surface)
- `src/pipeline/resolve.rs` — 7 (imports every provider for probing)

## 5. Hotspot intersect (`scripts/hotspot_intersect.py . --top 25 --days 90`)

Files with composite ≥ 0.80 — **near-certain Pass-2 deep-dive targets**:

| Rank | Path | Lines | Fan-in | Churn (90d) | Composite |
|------|------|-------|--------|-------------|-----------|
| 1 | `src/tui/queries.rs` | 540 | 3 | 13 | **0.91** |
| 2 | `src/ats/lever.rs` | 409 | 4 | 6 | **0.86** |
| 3 | `src/ats/greenhouse.rs` | 360 | 4 | 5 | **0.84** |

Files 0.70 ≤ composite < 0.80 (strong Pass-2 signal): `src/tui/views/dashboard.rs` 0.79, `src/tui/views/jobs.rs` 0.78, `src/ats/smartrecruiters.rs` 0.77, `src/db/schema.rs` 0.75, `src/ats/workable.rs` 0.75, `src/tui/app/state.rs` 0.74, `src/ats/ashby.rs` 0.72, `src/pipeline/format.rs` 0.72.

## 6. Dead-code baseline

- `scripts/orphans.py .` — **0 orphan candidates**. Every file in `src/` is imported from somewhere.
- `grep -r "#[allow(dead_code)]" src/` — **37 occurrences across 20 files**. This is a *suppressed* dead-code signal: code the compiler would flag as dead that developers have explicitly silenced. Every hit is a Pass-2 candidate for either (a) the attribute is load-bearing (e.g. future-use struct field) and worth a comment, or (b) the code is genuinely dead and the attribute is hiding it. Particularly concentrated in `src/ats/smartrecruiters.rs` (8), `src/ats/mod.rs` (5), `src/tui/app/state.rs` (4).

## 7. Known issues from context files

From `context/notes/autofill-status.md` and `context/architecture.md`:

- **Autofill React form filling broken.** Chrome launches, field injection fails. Fix approach documented: `nativeInputValueSetter` or CDP `Input.insertText`. This is a correctness/completeness risk, not new — already a Known Issue in context, so the audit **does not re-flag** per `references/scope-boundaries.md` §"What NOT to Flag". Pass 2 will check whether there is dead/partial code in `src/autofill/` that could be removed while the feature is deferred.
- **Eightfold has no fetcher** — companies recorded as bespoke. Known, not an audit finding.
- **Workday `ats_extra` null-silently-skips.** Known, in `systems/ats.md`.

## 8. Pass 2 system prioritisation

Based on the intersect of composite score + architectural centrality + behavioural-risk concentration:

| # | System | Primary files | Why prioritised | Data-Layout applicability |
|---|--------|---------------|-----------------|---------------------------|
| 1 | **ATS fetchers** | `src/ats/{common,lever,greenhouse,ashby,workable,smartrecruiters,workday,mod}.rs` | 6 providers, high fan-in, three in top-10 hotspot, HTML parsing + JSON normalisation patterns ripe for per-iteration allocation analysis | **Yes** — fetch loop allocates per-job in parser paths |
| 2 | **Pipeline — format** | `src/pipeline/format.rs` (1274 lines) | Largest file in repo; HTML→plaintext pipeline runs on every TUI boot; idempotency is a tight invariant | **Yes** — string builder allocation patterns in the hot loop |
| 3 | **Pipeline — search + resolve** | `src/pipeline/{search,resolve}.rs` | Critical path (per `architecture.md`). Parallel fetch + filter stack + DB insert. Per-request retry. Lock-across-await risk per initial WebSearch | **Yes** — intermediate `Vec` allocation in filter stack |
| 4 | **Pipeline — clean + check + import** | `src/pipeline/{clean,check,import}.rs` | Tiered archival SQL, ATS re-verification, bulk import. Smaller than the rest of the pipeline but each has distinct risks | Review |
| 5 | **TUI queries + hot path** | `src/tui/queries.rs` (0.91 composite — **highest in repo**), `src/tui/app/mod.rs` (fan-in 19) | Runs every 2 seconds. N+1 SQL candidates. Fresh connection per poll. Sub-millisecond ratatui expectation | **Yes** — the poll path is the steadiest allocator in the process |
| 6 | **TUI views (dashboard, jobs, companies)** | `src/tui/views/*.rs` | Large files, rendered every frame. String formatting and layout builders are allocator-heavy in ratatui | **Yes** — per-frame Spans/Lines allocation |
| 7 | **DB schema + migrations** | `src/db/schema.rs` (1011 lines) | Contains schema DDL + 6 migrations + 29 tests in one file. Modularisation review. Missing indexes possible | Review |
| 8 | **Config + filter predicates** | `src/config.rs` (535 lines, 31 tests inline) | Drives every search filter. Filter predicates are hot inside search loop | **Yes** — allocation in `passes_*` helpers |
| 9 | **Autofill (minimal)** | `src/autofill/*` | Broken by design; audit checks for dead/partial code to remove while feature is deferred | — |
| 10 | **Main dispatch + lib surface** | `src/main.rs`, `src/lib.rs`, `src/http.rs` | Utility-pattern; review argv handling + library re-exports | — |

**Cross-cutting passes:**

- Dead-code sweep via `#[allow(dead_code)]` enumeration (produces a cross-cutting finding file).
- Modularisation verdicts for every one of the 15 candidates (one `findings/modularisation.md` file with per-file disposition).
- Clippy `if has identical blocks` hit — investigate as potential Known Issue.

## 9. Scripts — command provenance

| Script | Command used | Output | Used for |
|--------|--------------|--------|----------|
| modularisation_candidates | `python3 …/modularisation_candidates.py .` | 15-file table | Section 3 above |
| import_graph | `python3 …/import_graph.py . --top 30` | Fan-in / fan-out top 30 | Section 4 above |
| hotspot_intersect | `python3 …/hotspot_intersect.py . --top 25 --days 90` | 25-file composite table | Section 5 above |
| orphans | `python3 …/orphans.py .` | "No orphan candidates detected" | Section 6 above |
| test_baseline | `bash …/test_baseline.sh .` | 319 pass / 0 fail | Section 2 above |

All five deterministic-input scripts ran cleanly. `evidence_map_lint.py` runs at the end of Pass 2 before final output.
