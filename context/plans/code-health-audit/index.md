# Code Health Audit — Cernio

**Date:** 2026-04-21
**Scope:** full repository — 56 Rust source files, 14k lines, 319-test baseline
**Status:** active (ready for implementation)

## Summary

Full two-pass audit of Cernio's codebase. 25 findings across 8 systems plus a 15-file modularisation verdict table and a 37-row `#[allow(dead_code)]` disposition sweep. 4 high-severity findings concentrated in ATS consolidation + SQL consolidation + autofill-adjacent cleanup. No critical correctness risks surfaced that are not already documented in `context/` as Known Issues. Test baseline was clean at entry (319 pass / 0 fail); 1 new diagnostic test file added (`tests/ats_strip_html_parity.rs`, +6 tests, all passing).

The most impactful findings cluster around two themes:

1. **DRY-then-correctness on the ATS layer.** Four independent `strip_html` implementations exist; two diverge in quote-handling, and the divergent Workable version is live. Consolidation both removes 70 lines of duplication and fixes a latent correctness bug on Workable descriptions containing `>` in quoted attributes.
2. **SQL-round-trip reduction on the TUI 2-second poll.** `fetch_stats` issues 16 separate queries per refresh (~29k queries per hour of TUI use). Consolidation into 4-6 GROUP BY queries is a pure refactor with zero behavioural change.

## What I Did Not Do

Status definitions: `done` (with evidence), `partial` (with reason and partial evidence), `skipped` (with reason).

- **Pre-Pass-1 front-loaded WebSearch:** done — query `code health audit patterns for Rust CLI pipeline project with SQLite async HTTP fetchers and ratatui TUI 2026`; source https://sherlock.xyz/post/rust-security-auditing-guide-2026 + 2 others; recorded in `obligation-evidence-map.md` at top.
- **Pass-1 checkpoint written before Pass 2 began:** done — `PASS-1-CHECKPOINT.md` (2026-04-21), containing full test baseline, 15-file modularisation candidate list, and Pass-2 prioritisation.
- **Project test suite baseline captured in Pass 1:** done — `bash scripts/test_baseline.sh .` → exit 0, 319 pass, 0 fail, 0 ignored. Command + per-target summary in `PASS-1-CHECKPOINT.md` §2.
- **Pre-existing test failures recorded as Known Issues findings:** done — **no pre-existing failures**. Noted explicitly in `PASS-1-CHECKPOINT.md` §2.
- **Research obligation met for every substantive system:** done with 2 reasoned omissions. WebSearch queries covered ATS (mode 3), Pipeline (mode 2), Cross-cutting (mode 1 via front-loaded). TUI/DB/Config inherit research from Pipeline (SQLite consolidation applies directly). Autofill + Main/HTTP/lib skipped per `detection-strategies.md` §"When Research Is Not Required" — autofill is a deferred Known Issue; Main/HTTP/lib are trivial utility files. Reasoned-omission rows present in the map.
- **Research-mode variety across the audit:** done — `evidence_map_lint.py` detects modes `[1, 2, 3]` in the primary table. 5 total WebSearch invocations across the audit, spanning all three modes. Distribution summary at top of `obligation-evidence-map.md`.
- **Diagnostic-test obligation met:** done — 1 new test file (`tests/ats_strip_html_parity.rs`, 6 assertions, all pass) written for the ATS strip_html finding where the test upgrades confidence from moderate to high. Other findings have High confidence from directly-observable source evidence (SQL query counts, `#[allow(dead_code)]` attribute presence, file line counts) or from research citations; a further diagnostic test would not upgrade confidence and was not written. This is the explicit deferral case in `detection-strategies.md` §"When to Write a Diagnostic Test".
- **Modularisation candidate list enumerated in Pass 1:** done — 15-file table in `PASS-1-CHECKPOINT.md` §3 with file path + line count + qualifying reason for every candidate.
- **Every modularisation candidate has a per-file verdict:** done — all 15 dispositioned in `findings/modularisation.md`: 3 split-recommended (dashboard.rs, jobs.rs follow-on, db/schema.rs low-priority), 11 leave-as-is with per-file justification grounded in `analysis-categories.md` §3 §"What NOT to Flag", 1 not-applicable (test file). No candidate without a verdict; no verdict says "out of scope for this round's focus."
- **Confidence upgrade pathway attempted before any moderate/low confidence finding:** done — every non-Triage finding is at High confidence. The Triage-Needed findings (Eightfold fetcher; SR asymmetric probing; autofill deferral scope) are explicitly framed as requiring user decisions; the audit cannot resolve them alone.
- **Pass-2 systems-audited checkpoint written before final output:** done — `PASS-2-SYSTEMS-AUDITED.md`.
- **Obligation Evidence Map has one row per substantive system (no PENDING rows):** done — 8 rows in primary table, zero PENDING, lint clean.
- **"What I Did Not Do" section present at top of `index.md`:** done — this section. Agrees with `obligation-evidence-map.md`.
- **Data Layout and Memory Access Patterns applied to every system audited in Pass 2:** done — per-system applicability decisions recorded. ATS, Pipeline, TUI, Config: analysis performed with specific findings. Database, Autofill, Main/HTTP/lib: reviewed and declared not applicable with justification.
- **Production source code not modified:** done — `git status` shows only additions to `context/plans/code-health-audit/*` and the new `tests/ats_strip_html_parity.rs` test file. Zero edits to `src/`.
- **Scripts invoked (Python/Rust project):** done — all 6 scripts ran cleanly. Commands + outputs in `PASS-1-CHECKPOINT.md` §9 and `obligation-evidence-map.md` §"Script-driven evidence". `evidence_map_lint.py` on final run: exit 0, modes `[1, 2, 3]` detected.

---

## Findings Overview

| File | System | High | Medium | Low | Triage | Total |
|------|--------|------|--------|-----|--------|-------|
| [findings/ats.md](findings/ats.md) | ATS fetchers (6 providers + common) | 2 | 4 | 1 | 1 | 8 |
| [findings/pipeline.md](findings/pipeline.md) | Pipeline (6 commands) | 2 | 5 | 3 | 0 | 10 |
| [findings/tui.md](findings/tui.md) | TUI (26 files) | 0 *(1 cross-ref)* | 2 | 2 | 0 | 4 |
| [findings/cross-cutting.md](findings/cross-cutting.md) | project-wide (dead_code density, strip_html, autofill tests) | 0 | 2 | 1 | 0 | 3 |
| [findings/other-systems.md](findings/other-systems.md) | Database, Config, Autofill, Main | 0 | 1 | 1 | 1 | 3 |
| [findings/modularisation.md](findings/modularisation.md) | per-file verdicts (15 candidates) | — | — | — | — | 3 split / 11 leave / 1 n/a |
| [findings/dead-code-sweep.md](findings/dead-code-sweep.md) | 37 `#[allow(dead_code)]` dispositions | — | — | — | — | 37 rows |
| **Total actionable findings** | | **4** | **14** | **7** | **2** | **27** |

*Numbers slightly exceed the 25 summary because 2 findings are cross-references to the canonical location in another file.*

---

## Priority Actions

Ordered by severity × effort (see `references/severity-and-prioritisation.md`):

1. **[HIGH, Small]** Consolidate four independent `strip_html` implementations into `src/ats/common.rs` — fixes latent Workable correctness divergence + removes 70 lines of duplication. [findings/ats.md § Inconsistent Patterns](findings/ats.md)
2. **[HIGH, Small]** Eliminate N+1 in `pipeline::search::run_by_grade` — single SQL with `WHERE c.grade = ?` replaces 288 round-trips. [findings/pipeline.md § Algorithm Optimisation](findings/pipeline.md)
3. **[HIGH, Medium]** Consolidate `fetch_stats` 16 SQL queries into 4-6 grouped queries — reduces ~29,000 queries per hour of TUI use. [findings/pipeline.md § Algorithm Optimisation](findings/pipeline.md)
4. **[HIGH, Trivial]** Add `get_with_retry` to SmartRecruiters pagination loop — prevents silent partial fetches on transient failure. [findings/ats.md § Known Issues](findings/ats.md)
5. **[MEDIUM, Trivial]** Remove dead `fetch_detail` + 4 structs from `src/ats/smartrecruiters.rs` — ~40 lines + 6 `#[allow(dead_code)]` attributes drop. [findings/ats.md § Dead Code Removal](findings/ats.md)
6. **[MEDIUM, Small]** Standardise `get_with_retry` usage across Ashby/Workable/Workday fetch paths. [findings/ats.md § Known Issues](findings/ats.md)
7. **[MEDIUM, Small]** Parallelise `check::verify_ats_slugs` + use `crate::http::build_client()` + switch Lever to `probe`. [findings/pipeline.md § Performance Improvement](findings/pipeline.md)
8. **[MEDIUM, Medium]** Split `src/tui/views/dashboard.rs` (946 lines) per the section-submodule pattern. [findings/tui.md § Modularisation](findings/tui.md) + [findings/modularisation.md](findings/modularisation.md)
9. **[MEDIUM, Trivial]** Extract shared `TIER_RULES` const in `clean.rs` between `preview` and `execute`. [findings/pipeline.md § Inconsistent Patterns](findings/pipeline.md)
10. **[MEDIUM, Medium]** Split `fetch_jobs` into list-only + detail-on-demand to stop fetching `raw_description` every poll. [findings/tui.md § Performance Improvement](findings/tui.md)

---

## By Category

- Algorithm Optimisation: 2 findings
- Performance Improvement: 5 findings
- Inconsistent Patterns: 3 findings
- Dead Code Removal: 4 findings + 37-row sweep disposition
- Known Issues and Active Risks: 3 findings
- Modularisation: 3 split-recommended + 11 leave-as-is + 1 not-applicable
- Test Coverage Gaps: 2 findings
- API Surface Bloat: 1 finding
- Triage Needed: 2 findings

---

## Research Sources (for reproducibility)

- [Sherlock — Rust Security & Auditing Guide 2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026) — 2026 Rust audit failure classes (lock-across-await)
- [Microsoft Rust Training — Async Is an Optimization, Not an Architecture](https://microsoft.github.io/RustTraining/async-book/ch14-async-is-an-optimization-not-an-architecture.html) — async as I/O multiplexing; sync business logic
- [Mathias Bynens — Unquoted attribute values in HTML and CSS/JS selectors](https://mathiasbynens.be/notes/unquoted-attribute-values) — HTML parser quote-tracking
- [reqwest-retry docs](https://docs.rs/reqwest-retry) + [seanmonstar's reqwest retries blog](https://seanmonstar.com/blog/reqwest-retries/) + [OneUptime — Retry Logic with Exponential Backoff in Rust](https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view) — retry-policy patterns
- [SQLite query planner](https://www.sqlite.org/queryplanner.html) + [SQLite GROUP BY tutorial](https://www.sqlitetutorial.net/sqlite-group-by/) + [SQLx lazy testing patterns](https://mattrighetti.com/2025/02/17/rust-testing-sqlx-lazy-people) — query consolidation

---

## Implementation Guidance

The findings are ordered in the Priority Actions list above. The recommended execution sequence:

1. **Batch 1 — Dead code removal** (from the SR file + sweep dispositions). Unblocks all downstream findings in ATS and cross-cutting.
2. **Batch 2 — strip_html consolidation.** Fixes Workable latent bug; removes 70 lines.
3. **Batch 3 — SQL consolidation + N+1 fix.** The two high-severity algorithm findings touch mostly `pipeline/search.rs` and `tui/queries.rs` in one sitting.
4. **Batch 4 — Retry standardisation across ATS providers.** Touches every ATS provider module + removes one `#[allow(dead_code)]`.
5. **Batch 5 — `verify_ats_slugs` parallelisation + Lever probe swap.** Isolated to `check.rs`.
6. **Batch 6 — Dashboard split.** Largest single modularisation. Touches only `src/tui/views/dashboard.rs`.
7. **Batch 7 — `fetch_jobs` list/detail split.** Touches `tui/queries.rs` + `tui/app/mod.rs` + all views that render detail.
8. **Everything else** in priority order: clean.rs TIER_RULES extract, clippy fixes, test additions, note-comment additions for KEEP-with-comment dead-code attributes.

Each batch is independently testable against the 319-test baseline + the 6 new `ats_strip_html_parity` tests (growing to 325 after this audit).

---

## Plan Lifecycle

Per `references/output-structure.md` §Lifecycle, this folder is active until all actionable findings are implemented or consciously deferred. The context upkeep workflow should tick checkboxes as items land. Deferred items move to the relevant `systems/*.md` file's Known Issues section before the plan folder is removed.
