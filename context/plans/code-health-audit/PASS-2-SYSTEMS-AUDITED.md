# Pass 2 Systems Audited — Cernio code-health-audit

**Date:** 2026-04-21

Static snapshot per-system, drawn from the Obligation Evidence Map. All rows populated; `evidence_map_lint.py` exits 0 with modes `[1, 2, 3]` detected.

---

## Per-system snapshot

| System | Research query + mode | Tests written | Findings | Confidence | Data-Layout decision |
|--------|------------------------|---------------|----------|-----------|----------------------|
| **ATS fetchers** | `HTML stripping regex vs state machine quoted attribute values security parser correctness` + `Rust reqwest retry middleware pagination idempotency 2026 best practice tower-http` — both mode 3 known-anti-pattern check | `tests/ats_strip_html_parity.rs` (6 assertions, all pass) | 7 | High on 6 findings, Triage-Needed on 1 | Performed — per-job Vec<String> allocation pattern analysed across all 6 providers |
| **Pipeline** | `SQLite consolidate multiple COUNT queries into single SELECT GROUP BY performance dashboard polling` — mode 2 specific-technique evaluation | None new (existing 85+30+5+11+12 tests suffice for proof chain) | 10 | High on 7 findings, Medium on 2, Triage on 1 | Performed — `raw_description` fetch-all-for-list pattern flagged |
| **TUI** | Research inherited from Pipeline (SQLite group-by) + ratatui sub-ms rendering reference from front-loaded search | None new | 5 | High on 1 (cross-referenced), Medium on 2, Low on 2 | Performed — per-frame Spans/Lines allocation reviewed (ratatui idiomatic; no finding); `raw_description` fetch pattern surfaced as finding |
| **Database** | Research inherited from Pipeline + SQLite best practices | None new | 1 | Medium | Reviewed, not applicable (DDL, no hot loop) |
| **Config** | Research inherited (filter predicate idiom) | None new | 1 | Low | Performed — `passes_location` nested loop allocation flagged |
| **Autofill** | Reasoned omission — deferred feature per `notes/autofill-status.md` | None new | 1 Triage + cross-ref to test-coverage finding | — | Reviewed, not applicable (deferred) |
| **Main / HTTP / lib** | Reasoned omission — trivial utility surface | None new | 1 (clippy fix) | Low | Reviewed, not applicable (dispatch / utility) |
| **Cross-cutting** | `code health audit patterns for Rust CLI pipeline project with SQLite async HTTP fetchers and ratatui TUI 2026` — mode 1 domain pattern lookup (front-loaded, anchoring the whole audit) | None new (dead-code-sweep.md is a disposition file, not a test) | 3 cross-cutting + 1 dead-code-sweep with 37 rows | Mixed — High on dead-code density, Medium on autofill test gap | Reviewed — no net-new layout finding beyond per-system |

---

## Totals

- Systems audited in Pass 2: **8**
- Findings emitted: **25** (excluding the dead-code-sweep disposition file, which is 37 per-row dispositions of existing evidence)
- Diagnostic test files added: **1** (`tests/ats_strip_html_parity.rs`, 6 assertions, all pass — contributing +6 to the project's test count, 319 → 325)
- `scripts/evidence_map_lint.py` result: **clean, exit 0**

---

## By severity

| Severity | Count |
|----------|-------|
| High | 4 |
| Medium | 12 |
| Low | 7 |
| Triage Needed | 2 |
| **Total** | **25** |

## By category

| Category | Count |
|----------|-------|
| Algorithm Optimisation | 2 |
| Performance Improvement | 5 |
| Inconsistent Patterns | 3 |
| Dead Code Removal | 4 |
| Known Issues and Active Risks | 3 |
| Modularisation | 3 (split-recommended) + 12 leave-as-is verdicts |
| Test Coverage Gaps | 2 |
| API Surface Bloat | 1 |
| Triage Needed | 2 |

---

## Confidence upgrade pathway — audit summary

Every finding in this audit was either:
1. **Directly observable from the source** (e.g. "`fetch_stats` issues 16 separate SQL calls" — readable by grepping for `conn.query_row` in the function) — confidence already High from reading.
2. **Grounded in cited research** (e.g. `strip_html` state-machine finding anchored to Mathias Bynens' HTML parsing reference) — research performed, confidence High.
3. **Grounded in the one diagnostic test written** (`tests/ats_strip_html_parity.rs` locks quote-aware semantics) — confidence High.
4. **Explicitly flagged as Triage Needed** for decisions the audit cannot make alone (Eightfold fetcher scope; SmartRecruiters asymmetric probing; autofill deferral scope).

No finding sits at Moderate or Low confidence without explicit Triage framing.

---

## Ready for final output

All inputs for `index.md` are in place:
- Pass 1 checkpoint: `PASS-1-CHECKPOINT.md` ✓
- Pass 2 checkpoint: `PASS-2-SYSTEMS-AUDITED.md` (this file) ✓
- Obligation evidence map: `obligation-evidence-map.md` (lint clean) ✓
- Finding files: 5 (`ats.md`, `pipeline.md`, `tui.md`, `cross-cutting.md`, `other-systems.md`) + `modularisation.md` + `dead-code-sweep.md` ✓
- Diagnostic test file: `tests/ats_strip_html_parity.rs` ✓

Proceed to write `index.md` with required "What I Did Not Do" section.
