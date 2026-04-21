# Modularisation Verdicts — Per-File

**Scope:** every file qualifying as a modularisation candidate per the Pass-1 checkpoint (Rust ≥350 lines OR top-decile of 56 files).

**Verdict options:** `split-recommended`, `leave-as-is`, or `not-applicable` (with justification grounded in `references/analysis-categories.md` §3).

---

## Verdict table

| # | File | Lines | Verdict | Justification |
|---|------|-------|---------|---------------|
| 1 | `src/pipeline/format.rs` | 1274 | **leave-as-is** | Parser for a complex format (HTML+entities) that genuinely requires this surface. 85 inline tests tightly coupled to internal helpers. The file IS internally well-structured — each phase of the pipeline (entity decode → structural conversion → tag strip → whitespace clean) is a named helper with tests. Splitting into `format/entities.rs`, `format/tags.rs`, etc. would duplicate all the `#[cfg(test)]` scaffolding and break the single-file invariant guarantee that `format(format(x)) == format(x)`. The file's size reflects the format's complexity, not a structural defect. |
| 2 | `src/db/schema.rs` | 1011 | **split-recommended** (low priority) | Contains DDL constants + MIGRATION_001..006 + 29 tests. Splitting the tests into `tests/db_schema.rs` would halve the file while preserving the migration narrative. Defer until a schema change lands — opportunistic split. |
| 3 | `src/tui/views/dashboard.rs` | 946 | **split-recommended** | Multiple independent visualisations (heatmap, search pulse, visa countdown, top companies, session diff, top matches, grade bars). High churn (11 commits / 90d). See `findings/tui.md` §Modularisation for the split proposal. |
| 4 | `src/pipeline/resolve.rs` | 668 | **leave-as-is** | Well-structured — `slug_candidates` + `probe_all_providers` + `run`/`run_single` + 30 tests. Tests are tightly coupled to `slug_candidates` internals (the "realistic roster" test validates production behaviour). Splitting would separate `slug_candidates` from its tests, which is the highest-value pairing in the file. |
| 5 | `src/tui/views/jobs.rs` | 595 | **split-recommended** (low priority) | Second-biggest views file. Same rationale as dashboard split but smaller, lower churn. See `findings/tui.md` §Modularisation. Defer until dashboard split pattern is proven. |
| 6 | `src/tui/queries.rs` | 540 | **leave-as-is** | Hottest file by composite (0.91) but structurally coherent — every function is a typed SQL read with identical shape (prepare → query_map → collect). Splitting by topic (companies queries, jobs queries, stats queries) would fragment a cohesive module whose abstraction is "SQL reads for the TUI." The consolidation finding in `findings/pipeline.md` reduces its content independently of modularisation. |
| 7 | `src/config.rs` | 535 | **leave-as-is** | 31 tests are inline and tightly coupled to `passes_exclusion` / `passes_inclusion` / `passes_location` / `included_grades`. The file's structure is `struct defs + impls + tests`; each block is ~150 lines of the right concern. Splitting config types into `config/types.rs` and predicates into `config/filters.rs` would separate impl blocks from the struct they belong to. The module is a well-understood boundary; the file reflects that. |
| 8 | `src/tui/views/companies.rs` | 438 | **leave-as-is** | Below the "clearly too large" threshold. One view, one concern. Internal structure is flat and readable. |
| 9 | `src/pipeline/search.rs` | 426 | **leave-as-is** | Three public entry points (`run`, `run_single`, `run_by_grade`) + shared helpers + internal types. Each entry point is ~30-50 lines. The shared helpers (`get_search_targets`, `fetch_all_parallel`, `fetch_jobs`, `job_exists`, `insert_job`) are the right level of abstraction for the search pipeline. Splitting would separate the three `run*` functions from their shared helpers, creating worse coupling. |
| 10 | `src/ats/smartrecruiters.rs` | 411 | **leave-as-is** (after dead-code fix) | Significant share of the line count comes from the dead `fetch_detail` path + its 4 dead structs. Once `findings/ats.md` §"Remove unused `fetch_detail`..." is applied, the file drops to ~330 lines, well below the threshold. No modularisation action needed — the dead-code removal subsumes this verdict. |
| 11 | `src/main.rs` | 409 | **leave-as-is** | CLI dispatcher with one `cmd_*` wrapper per subcommand. The wrappers are 10-20 lines each and each calls straight into its pipeline module. Splitting into `main/dispatch.rs` + `main/commands/*.rs` would introduce indirection for no gain — the file is already flat, linear, and each block is self-contained. |
| 12 | `src/ats/lever.rs` | 409 | **leave-as-is** | Provider-specific types + `probe` + `fetch_all` + `normalise` + 30 inline tests. The per-provider file-per-fetcher convention (documented in `systems/ats.md`) is the right granularity. Splitting this single Lever file would only increase cognitive load for code whose scope is already "everything Lever-specific." |
| 13 | `src/pipeline/check.rs` | 400 | **leave-as-is** | Integrity checks file — health/completeness/staleness sections plus `verify_ats_slugs` helper plus report printer. Each block is a distinct check category. The findings about parallelising `verify_ats_slugs` (`findings/pipeline.md` §Performance Improvement) apply independently of modularisation. |
| 14 | `tests/pipeline_clean.rs` | 375 | **not-applicable** | Test file — modularisation rules apply to production source. The file is one test harness covering the entire clean pipeline; splitting would separate test setup helpers from the tests that use them. |
| 15 | `src/ats/greenhouse.rs` | 360 | **leave-as-is** | Same rationale as Lever — per-provider-file-per-fetcher is the convention. At 360 lines it is barely over the threshold. |

**Total:**
- `split-recommended`: 3 (dashboard, jobs.rs as a lower-priority follow-on, db/schema.rs as low-priority opportunistic)
- `leave-as-is`: 11 (with one — `smartrecruiters.rs` — contingent on the dead-code fix landing)
- `not-applicable`: 1 (test file)

---

## Rationale for high `leave-as-is` count

Cernio's codebase predominantly follows the one-concern-per-file convention consistently. Files large enough to qualify as modularisation candidates generally fall into three structural shapes, and for two of those three shapes modularisation is the wrong answer:

1. **Per-provider fetchers** (Lever, Greenhouse, Smartrecruiters, Ashby, Workable, Workday) — the convention is explicitly "one file per provider" because each provider's API quirks + tests + normalisation cohere. Large files here reflect the provider's complexity, not project structural defect.

2. **Parsers for complex formats** (`format.rs`) — the file reflects the format, not the code.

3. **Tightly-coupled-to-tests business logic files** (`config.rs`, `resolve.rs`) — the tests ARE the documentation for the predicates; splitting separates them from what they prove.

Dashboard's split-recommended verdict is the counter-example: it aggregates independent visualisations that share no state, only read from `App`. That is the canonical shape that modularisation improves.

---

## Non-negotiable obligation discharge

Every file in the Pass-1 candidate list appears above with a verdict grounded in `references/analysis-categories.md` §3 §"What NOT to Flag". No verdict is "out of scope for this round's focus" — each is a substantive per-file analysis.
