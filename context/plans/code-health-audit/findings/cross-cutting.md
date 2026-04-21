# Cross-Cutting — Code Health Findings

**Systems covered:** project-wide patterns spanning multiple modules
**Finding count:** 3 findings (0 high, 2 medium, 1 low)

---

## Dead Code Removal

### 37 `#[allow(dead_code)]` attributes across 20 files suppress compiler-detected dead code
- [ ] Audit each `#[allow(dead_code)]` attribute listed in `context/plans/code-health-audit/dead-code-sweep.md`. For each hit, either (a) delete the suppressed item, (b) verify the suppression is load-bearing (e.g. test-only accessor, future-use field documented in notes) and add a one-line `// Kept because: ...` comment, or (c) confirm the compiler no longer flags it and remove the attribute.

**Category:** Dead Code Removal
**Severity:** Medium
**Effort:** Medium
**Behavioural Impact:** None (each removal is compiler-enforced — if the item was truly dead, deletion is zero-impact; if not, the build will fail and the finding is wrong for that item)

**Location:**
`grep -rn "#\[allow(dead_code)\]" src/` produces the following distribution (37 occurrences across 20 files):

| File | Occurrences |
|------|-------------|
| `src/ats/smartrecruiters.rs` | 8 |
| `src/ats/mod.rs` | 5 |
| `src/tui/app/state.rs` | 4 |
| `src/tui/theme.rs` | 2 |
| `src/tui/app/actions.rs` | 2 |
| `src/autofill/mod.rs` | 2 |
| 14 other files | 1 each |

See `dead-code-sweep.md` for the row-by-row disposition.

**Current State:**
`#[allow(dead_code)]` suppresses Rust's warn-by-default "item is never used" lint. Each attribute masks a specific warning — typically:

1. **Test-only accessors** on production structs (needed for test-crate visibility but warned against in production builds).
2. **Future-use fields** deserialised by Serde but not yet consumed by application logic.
3. **Genuinely dead code** that somebody forgot to delete.
4. **Stale suppressions** — the item has since been used but the attribute was never removed.

The audit's diagnostic-test-free probe (the `scripts/orphans.py` tool) found zero *module-level* orphans — every file is imported somewhere. But the 37 attributes show that *within* those files, significant surface area is known-dead or future-use.

Dedicated hot-spots:
- `src/ats/smartrecruiters.rs` (8 occurrences) — almost all of these disappear if the related `fetch_detail` dead-code finding in `findings/ats.md` is applied.
- `src/ats/mod.rs` (5 occurrences) — `#[allow(dead_code)] pub mod ashby; pub mod common; #[allow(dead_code)] pub mod greenhouse; …` masks all provider modules. This is a file-level blanket suppression that should be replaced by per-item analysis or removed entirely (the modules ARE used — they are imported by `pipeline/search.rs` and `pipeline/resolve.rs`).
- `src/tui/queries.rs:147` — `#[allow(dead_code)] pub fn fetch_total_job_count` — but this function IS called from `src/tui/app/mod.rs:31` and `:125`. The attribute appears to be stale.

**Proposed Change:**
For each row in the dead-code sweep file, one of:
1. **Delete** — item is genuinely unused (confirmed by grep showing no external callers, considering plugin/dynamic-dispatch possibilities).
2. **Keep with comment** — item is intentionally retained; add `// Kept because: <specific reason>` on the line above.
3. **Remove suppression** — item is actually used; the attribute is stale; removing it and rebuilding shows no warning.

**Justification:**
- `#[allow(dead_code)]` is Rust's "silencer of last resort." Using it as a blanket suppression across 20 files means the compiler's dead-code detection is partly disabled across the project.
- The attribute density in `src/ats/smartrecruiters.rs` and `src/ats/mod.rs` suggests suppression-then-forget rather than considered-retention.
- Research anchor: [Rust by Example — Attributes for features](https://doc.rust-lang.org/rust-by-example/attribute/unused.html) documents `#[allow(dead_code)]` as a targeted suppression intended for individual items — not for blanket file-level use.

**Expected Benefit:**
- Removes 10-20 truly dead items (precise count emerges from the per-row disposition in the sweep file).
- Re-enables dead-code warnings for the ~15-20 files where the attribute is stale or file-level.
- Surfaces the handful of genuine "kept for future use" items with explicit rationale comments.

**Impact Assessment:**
Compiler-enforced per removal. If a deletion would break any caller, `cargo build` fails and the item is kept. No behaviour change is possible.

---

## Inconsistent Patterns

### Four independent `strip_html` implementations across `src/ats/` + `src/pipeline/format.rs`
- [ ] (Documented in `findings/ats.md` §Inconsistent Patterns §"Four independent `strip_html` implementations across `src/ats/`" — the fix consolidates the ATS-side strippers. The `pipeline::format` stripper is a separate HTML pipeline and is out of scope for consolidation.)

**Category:** Inconsistent Patterns
**Severity:** High
**Effort:** Small
**Behavioural Impact:** Possible (flagged) — Workable upgrades from simple to quote-aware stripper.

Cross-reference only — see `findings/ats.md` for the canonical finding.

---

## Known Issues and Active Risks

### Clippy's one `if has identical blocks` hit is the only non-style-class warning
- [ ] (Documented in `findings/pipeline.md` §Inconsistent Patterns §"`print_report` in clean.rs has identical-branch `if` producing two duplicate bindings" — single-file fix.)

**Category:** Known Issues and Active Risks
**Severity:** Low
**Effort:** Trivial
**Behavioural Impact:** None

Cross-reference only — see `findings/pipeline.md` for the canonical finding.

---

## Test Coverage Gaps

### Autofill module has no tests despite being implemented and wired up
- [ ] Add inline tests for the pure parts of `src/autofill/mod.rs` (ApplicantProfile::load, field extraction) before further work on the React-form-filling fix lands.

**Category:** Test Coverage Gaps
**Severity:** Low
**Effort:** Small
**Behavioural Impact:** None

**Location:**
- `src/autofill/mod.rs` — 132 lines, 2 `#[allow(dead_code)]`, zero tests
- `src/autofill/common.rs` — 172 lines, 1 `#[allow(dead_code)]`, zero tests
- `src/autofill/greenhouse.rs` — 131 lines, 1 `#[allow(dead_code)]`, zero tests

**Current State:**
`context/notes/testing-strategy.md` §"What is *not* tested (and why)" documents this: "Autofill Chrome paths are not tested. The Chrome CDP integration is impossible to test without a real browser, and is already known broken with a separate fix planned. The pure parts (`ApplicantProfile::load`, field extraction) could be tested but are low priority compared to everything else; they were skipped to keep this pass focused."

The skip is deliberate and documented. The audit does not dispute the rationale but flags the pure-parts gap as an opportunity for the next autofill-related session: once the React-form-filling fix is applied (the top next-priority item in `architecture.md`), the pure parts need test coverage to catch regressions.

**Proposed Change:**
Add inline `#[cfg(test)] mod tests` blocks to `autofill/mod.rs` covering:
- `ApplicantProfile::load` — parses a tmpdir'd profile/personal.md, profile/skills.md, profile/preferences.toml fixture set.
- Provider dispatch — given `ats_provider = "greenhouse"`, returns the Greenhouse filler; given unsupported, returns the fallback.
- Application-package JSON round-trip — given a known JSON string, produces expected field-value pairs.

**Justification:**
The React-form-filling fix will touch `autofill/common.rs` and `autofill/greenhouse.rs`. Without a test baseline, the fix lands blind. Even trivial field-extraction tests catch regressions in the JSON parsing that underlies the entire feature.

**Expected Benefit:**
~10-15 new tests covering the pure parts. Gives the React-fix session a regression gate before touching the broken code paths.

**Impact Assessment:**
Zero behavioural change (tests are additive). Fits the project's existing inline-test convention.
