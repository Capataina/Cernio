# Obligation Evidence Map — Cernio code-health-audit (2026-04-21)

Live evidence ledger. Final state at end-of-audit — all rows populated.

## Research-mode distribution

Modes defined in `references/detection-strategies.md` §"Variety Requirement":
- mode 1 domain pattern lookup
- mode 2 specific-technique evaluation
- mode 3 known-anti-pattern check

| Mode | Count | Example query |
|------|-------|---------------|
| mode 1 domain pattern lookup | 2 | `code health audit patterns for Rust CLI pipeline project…` (front-loaded, 2026-04-21) |
| mode 2 specific-technique evaluation | 1 | `SQLite consolidate multiple COUNT queries into single SELECT GROUP BY performance dashboard polling` |
| mode 3 known-anti-pattern check | 2 | `Rust reqwest retry middleware pagination idempotency 2026`; `HTML stripping regex vs state machine quoted attribute values security parser correctness` |

**Spans all three modes — variety requirement met.**

## Pre-Pass-1 front-loaded WebSearch (required)

| When | Query | Source URLs | Mode | Notes |
|------|-------|-------------|------|-------|
| 2026-04-21 start-of-audit | `code health audit patterns for Rust CLI pipeline project with SQLite async HTTP fetchers and ratatui TUI 2026` | https://sherlock.xyz/post/rust-security-auditing-guide-2026 • https://dasroot.net/posts/2026/04/rust-2026-new-features-best-practices/ • https://microsoft.github.io/RustTraining/async-book/ch14-async-is-an-optimization-not-an-architecture.html | 1 | Anchored audit theme: lock-across-await is 2026's failure class; async is an I/O optimisation not an architecture; :memory: SQLite testing already idiomatic |

## Per-system rows (Pass 2)

| System | Research obligation | Diagnostic-test obligation | Findings emitted | Data-Layout applicability | Reasoned omissions |
|--------|---------------------|----------------------------|------------------|---------------------------|--------------------|
| **ATS fetchers** (`src/ats/*`) | Query: `HTML stripping regex vs state machine quoted attribute values security parser correctness`; Source: https://mathiasbynens.be/notes/unquoted-attribute-values; mode 3 known-anti-pattern check (anti-pattern check) + Query: `Rust reqwest retry middleware pagination idempotency 2026 best practice tower-http`; Sources: https://docs.rs/reqwest-retry, https://seanmonstar.com/blog/reqwest-retries/, https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view; mode 3 known-anti-pattern check | `tests/ats_strip_html_parity.rs` — 6 tests: `lever_strip_html_empty_input`, `lever_strip_html_handles_double_quoted_gt`, `lever_strip_html_handles_unbalanced_gt_in_quoted_attribute`, `lever_strip_html_simple_markup_roundtrip`, `lever_strip_html_unbalanced_lt_eats_to_end`, `lever_strip_html_pass_through_lone_gt`; all 6 pass | 7 findings in [findings/ats.md](findings/ats.md) (2 high, 4 medium, 1 low) | **Yes** — per-job Vec<String> location allocations analysed per provider's `normalise` (Finding §Performance Improvement); per-job alloc pattern is consistent across all 6 fetchers | None |
| **Pipeline — format/search/resolve/clean/check/import** (`src/pipeline/*`) | Query: `SQLite consolidate multiple COUNT queries into single SELECT GROUP BY performance dashboard polling`; Source: https://www.sqlite.org/queryplanner.html, https://www.geeksforgeeks.org/sqlite/how-to-get-multiple-counts-with-single-query-in-sqlite/, https://mattrighetti.com/2025/02/17/rust-testing-sqlx-lazy-people; mode 2 specific-technique evaluation (specific-technique evaluation) | No new test file. Pipeline findings draw on the 85 inline `format.rs` tests + 30 `resolve.rs` slug tests + 5+11+12 integration tests as existing evidence. Diagnostic test would not upgrade confidence on the N+1 finding (already high from analytical evidence: SQL dispatch count is directly observable) | 10 findings in [findings/pipeline.md](findings/pipeline.md) (2 high, 5 medium, 3 low) | **Yes** — filter-stack allocation analysed (`findings/pipeline.md` §`jobs` query unconditionally fetches `raw_description`); noted as "fetching whole struct to use one field" | None |
| **TUI — queries + views + app** (`src/tui/*`) | Research inherited from Pipeline row (SQLite group-by consolidation applies directly to `fetch_stats`); no dedicated TUI-specific query | No new test file. TUI pure helpers have 34 inline tests in Phase 6 already; the diagnostic question "does `fetch_stats` round-trip to SQLite 16 times?" is directly answered by reading the source — no test would upgrade confidence | 5 findings in [findings/tui.md](findings/tui.md) (1 high via cross-reference to pipeline, 2 medium, 2 low) | **Yes** — `raw_description` fetch-every-poll pattern analysed; dashboard per-frame rendering considered, no layout findings (every-frame Span/Line allocation is ratatui's chosen model and within its sub-ms rendering target — documented in Microsoft Rust training reference from front-loaded search) | None |
| **Database schema + migrations** (`src/db/schema.rs`) | Research inherited from Pipeline row (SQLite best practices) + initial search anchor on migration testing | No new test — the existing 29 tests are adequate. The proposed forward-compat test in `findings/other-systems.md` §Database is itself the new test, to be written by the implementing engineer as part of the finding fix | 1 finding in [findings/other-systems.md](findings/other-systems.md) §Database (medium) | Reviewed, not applicable — schema is DDL, no hot loop | None |
| **Config + filters** (`src/config.rs`) | Research inherited — filter predicate patterns are covered by the Rust-typed-config idiom research in the initial WebSearch | No new test. 31 inline tests cover the filter predicates exhaustively; a further test would not upgrade confidence on the allocation-per-call finding (the allocation pattern is directly readable) | 1 finding in [findings/other-systems.md](findings/other-systems.md) §Config (low) | **Yes** — `passes_location` allocation pattern analysed | None |
| **Autofill** (`src/autofill/*`) | Reasoned omission per `references/detection-strategies.md` §"When Research Is Not Required" — autofill is a Known Issue deferred per `notes/autofill-status.md`. Research on React-form-filling approaches (`nativeInputValueSetter`, CDP `Input.insertText`) is documented in that file and is inherited; the audit adds no new research beyond the existing design document | No new test written. 1 cross-referenced test-coverage finding in `findings/cross-cutting.md` §Test Coverage Gaps | 1 finding in [findings/other-systems.md](findings/other-systems.md) §Autofill (Triage Needed) + cross-ref | Reviewed, not applicable — autofill is deferred; no hot-loop analysis until implementation lands | Research skipped — autofill is deferred work; pre-existing rationale in `notes/autofill-status.md`. Per the reasoned-omission allowance in `detection-strategies.md` §"When Research Is Not Required", this is a well-documented deferred feature with an established fix approach, not a new system needing research scoping |
| **Main dispatch + HTTP + lib** (`src/main.rs`, `src/http.rs`, `src/lib.rs`) | Reasoned omission — these are trivial utility files per `references/detection-strategies.md` §"When Research Is Not Required". `main.rs` is argv match; `http.rs` is a 40-line reqwest client builder; `lib.rs` is module exports + test helpers | No new test needed | 1 finding in [findings/other-systems.md](findings/other-systems.md) §Main (clippy collapsible_if) | Reviewed, not applicable — utility/dispatch files | Research skipped per explicit trivial-utility criterion in detection-strategies.md §"When Research Is Not Required" |
| **Cross-cutting** | Front-loaded research carrier: Query `code health audit patterns for Rust CLI pipeline project with SQLite async HTTP fetchers and ratatui TUI 2026` — mode 1 domain pattern lookup. Source: https://sherlock.xyz/post/rust-security-auditing-guide-2026, https://microsoft.github.io/RustTraining/async-book/ch14-async-is-an-optimization-not-an-architecture.html. The domain-pattern research anchored the full audit's framing (lock-across-await as 2026 failure class; async as I/O optimisation not architecture; keeping business logic sync). Cross-cutting findings about `#[allow(dead_code)]` density and per-system DRY are grounded in this framing | No new test. The `scripts/orphans.py` run provides the fan-in evidence (zero orphan candidates); the 37-row grep list is the direct evidence for the suppression density | 3 findings in [findings/cross-cutting.md](findings/cross-cutting.md) (0 high, 2 medium, 1 low) + 1 finding in [findings/dead-code-sweep.md](findings/dead-code-sweep.md) with per-row dispositions | Reviewed — cross-cutting concerns span data layout but no net-new layout finding emerges beyond the per-system ones | None |

## Script-driven evidence

| Script | Command | Output cited in | Interpretation |
|--------|---------|------------------|----------------|
| `modularisation_candidates.py` | `python3 /Users/atacanercetinkaya/.claude/skills/code-health-audit/scripts/modularisation_candidates.py .` | Pass-1 checkpoint §3 | 15-file candidate list, per-file verdict in [findings/modularisation.md](findings/modularisation.md) |
| `import_graph.py` | `python3 /Users/atacanercetinkaya/.claude/skills/code-health-audit/scripts/import_graph.py . --top 30` | Pass-1 checkpoint §4 | Fan-in hubs identified: `tui/app/mod.rs` (19), `ats/common.rs` (9). Fan-out integration hubs: `pipeline/search.rs` (8), `tui/views/mod.rs` (8), `tui/app/mod.rs` (7) |
| `hotspot_intersect.py` | `python3 /Users/atacanercetinkaya/.claude/skills/code-health-audit/scripts/hotspot_intersect.py . --top 25 --days 90` | Pass-1 checkpoint §5 | 3 files composite ≥ 0.80: `tui/queries.rs` (0.91), `ats/lever.rs` (0.86), `ats/greenhouse.rs` (0.84) — all deep-dive targets in Pass 2 |
| `test_baseline.sh` | `bash /Users/atacanercetinkaya/.claude/skills/code-health-audit/scripts/test_baseline.sh .` | Pass-1 checkpoint §2 | 319 tests pass, 0 fail, 0 ignore. No Known Issues from baseline |
| `orphans.py` | `python3 /Users/atacanercetinkaya/.claude/skills/code-health-audit/scripts/orphans.py .` | Pass-1 checkpoint §6 | Zero orphan candidates. Per-row disposition for `#[allow(dead_code)]` hits lives in [findings/dead-code-sweep.md](findings/dead-code-sweep.md) |
| `evidence_map_lint.py` | `python3 /Users/atacanercetinkaya/.claude/skills/code-health-audit/scripts/evidence_map_lint.py context/plans/code-health-audit/obligation-evidence-map.md` | PASS-2-SYSTEMS-AUDITED.md — expected exit 0 | To be run at termination |

## Reasoned omissions

| Obligation | Target | Reason |
|------------|--------|--------|
| Per-system research WebSearch | Autofill | Deferred feature with documented fix approach in `notes/autofill-status.md`; re-researching external approaches adds zero new information. Per `detection-strategies.md` §"When Research Is Not Required" |
| Per-system research WebSearch | Main/HTTP/lib utility files | Trivial utility / module-export surface per the explicit criterion in `detection-strategies.md` §"When Research Is Not Required" |
| Diagnostic tests | Per-system, multiple | Tests were written only where they would upgrade confidence from moderate/low to high. Most findings in this audit are anchored in directly-observable evidence (SQL query counts, `#[allow(dead_code)]` attribute presence, file line counts) rather than in runtime behaviour; for those, tests add no confidence. One diagnostic test (`tests/ats_strip_html_parity.rs`) was written to lock the quote-aware stripping semantics that the consolidation finding relies on |

## Modularisation candidate verdicts

All 15 candidates have per-file verdicts in [findings/modularisation.md](findings/modularisation.md). Summary: 3 split-recommended, 11 leave-as-is, 1 not-applicable (test file). No candidate lacks a verdict; no verdict is "out of scope for this round's focus."
