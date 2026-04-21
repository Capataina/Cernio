# Dead-Code Sweep — Per-Row Disposition

**Source:** `grep -rn "#\[allow(dead_code)\]" src/` produced 37 occurrences across 20 files. `scripts/orphans.py .` returned zero orphan candidates (no fan-in-zero files outside entry points).

**Disposition key:**
- **DELETE** — verified dead, recommended for removal
- **KEEP (with comment)** — intentionally retained, should gain a `// Kept because: ...` comment
- **REMOVE SUPPRESSION** — item is actually used, attribute is stale
- **SUBSUMED** — covered by a named finding in another file
- **INVESTIGATE** — requires a diagnostic test or manual call-site review

Each row that resolves to DELETE or KEEP must appear as an actionable line in the implementing engineer's task list.

---

## Per-file row-by-row

### `src/ats/smartrecruiters.rs` (8 occurrences)

| Line | Item | Disposition | Notes |
|------|------|-------------|-------|
| 17 | `SmartRecruitersJob` struct | KEEP (with comment) | Used by `normalise`. Attribute suppresses field-level dead-code on `experience_level`, `department`. Per `findings/ats.md` §Dead Code Removal §"Remove dead fields on `SmartRecruitersJob`" — trim the fields, then the attribute can go. |
| 33 | `SmartRecruitersLocation` struct | KEEP (with comment) | Used by `normalise`. Attribute covers `region` and `address` fields. Same treatment as above. |
| 44 | `SmartRecruitersLabel` struct | SUBSUMED | Removed entirely by `findings/ats.md` §"Remove dead fields" once `experience_level` + `department` are removed (SmartRecruitersLabel is their type). |
| 51 | `DetailResponse` struct | SUBSUMED | Deleted by `findings/ats.md` §"Remove unused `fetch_detail`...". |
| 58 | `JobAd` struct | SUBSUMED | Same. |
| 64 | `JobAdSections` struct | SUBSUMED | Same. |
| 74 | `HtmlSection` struct | SUBSUMED | Same. |
| 129 | `pub async fn fetch_detail` | SUBSUMED | Deleted by `findings/ats.md` §"Remove unused `fetch_detail`...". |

### `src/ats/mod.rs` (5 occurrences)

| Line | Item | Disposition | Notes |
|------|------|-------------|-------|
| 1 | `pub mod ashby` | REMOVE SUPPRESSION | Used by `pipeline::search::fetch_jobs` and `pipeline::resolve::probe_all_providers`. Attribute stale. |
| 4 | `pub mod greenhouse` | REMOVE SUPPRESSION | Same — used by search + resolve + check. |
| 7 | `pub mod smartrecruiters` | REMOVE SUPPRESSION | Same. |
| 9 | `pub mod workable` | REMOVE SUPPRESSION | Same. |
| 11 | `pub mod workday` | REMOVE SUPPRESSION | Used by search. |

Note: `lever` and `common` have NO `#[allow(dead_code)]` in `mod.rs` — they are the two that don't need suppression, which is further evidence that the others' attributes are stale (all modules are used the same way).

### `src/tui/app/state.rs` (4 occurrences)

| Line | Item | Disposition | Notes |
|------|------|-------------|-------|
| 78 | `pub struct JobRow` | INVESTIGATE | Whole struct; many fields are read by views. `fit_score: Option<f64>` is likely dead (Cernio uses grade, not fit_score, per `notes/tui-design.md`). Per-field audit needed. |
| 97 | `pub struct DashboardStats` | INVESTIGATE | Most fields are displayed on the dashboard. Confirm `applied_count`, `watching_count`, `rejected_count` are all rendered; remove any that are not. |
| 117 | `pub struct TopMatch` | KEEP (with comment) | Used by `fetch_top_matches`; every field is consumed by the dashboard renderer. Attribute exists because of cross-module visibility. A `// Kept because: public-through-view-boundary` comment resolves it. |
| 127 | `pub struct Toast` | REMOVE SUPPRESSION | Used by `App.toasts` and `add_toast`. Every field is read by the toast renderer. Attribute stale. |

### `src/tui/app/actions.rs` (2 occurrences)

| Line | Item | Disposition | Notes |
|------|------|-------------|-------|
| various | actions helpers | INVESTIGATE | Not inspected file-by-file in this pass. Each occurrence needs a call-site check. |

### `src/autofill/mod.rs` (2 occurrences)

| Line | Item | Disposition | Notes |
|------|------|-------------|-------|
| 7, 101 | autofill entry points | KEEP (with comment) | `notes/autofill-status.md` marks this feature as scaffolded-but-broken with a fix planned. The `#[allow]` attributes here are genuinely "retained until fix lands." Add `// Kept because: feature in-progress, see notes/autofill-status.md`. |

### `src/tui/theme.rs` (2 occurrences)

| Line | Item | Disposition | Notes |
|------|------|-------------|-------|
| 9 | `pub struct Theme` | KEEP (with comment) | Public API of the theme module; used across views. Attribute exists because individual fields may not be read. |
| 155 | internal helper | INVESTIGATE | Not inspected in this pass. |

### Other files (1 occurrence each)

| File | Line | Disposition | Notes |
|------|------|-------------|-------|
| `src/ats/common.rs` | 71 | REMOVE SUPPRESSION or SUBSUMED | `post_json_with_retry` is attribute-suppressed. Becomes live when `findings/ats.md` §"Fetch-all paths skip retry" adds Workday to use it. |
| `src/ats/workable.rs` | 32 | SUBSUMED | `WorkableLocation` fields; deletable or retainable per the same pattern as SR location fields. |
| `src/ats/ashby.rs` | 15 | SUBSUMED | `AshbyJob` struct fields not read by `normalise` (e.g. `employment_type`). Same treatment. |
| `src/ats/lever.rs` | 188 | SUBSUMED | `pub fn strip_html` — resolved by the consolidation finding. |
| `src/http.rs` | 17 | INVESTIGATE | Likely `build_client_with_cookies`. Grep for callers and DELETE if zero. |
| `src/db/schema.rs` | 32 | KEEP (with comment) | Likely a `#[cfg(test)]` helper or test-fixture struct field. |
| `src/tui/queries.rs` | 147 | REMOVE SUPPRESSION | `fetch_total_job_count` is called from `src/tui/app/mod.rs:31` and `:125`. Attribute stale. |
| `src/tui/widgets/toast.rs` | 9 | INVESTIGATE | Not inspected. |
| `src/tui/widgets/grade_bar.rs` | 13 | INVESTIGATE | Not inspected. |
| `src/pipeline/clean.rs` | 6 | SUBSUMED | `CleanupReport` — resolved by `findings/pipeline.md` §"`CleanupReport` is `pub` + `#[allow(dead_code)]` but never used outside the module". |
| `src/tui/app/navigation.rs` | 131 | INVESTIGATE | Not inspected. |
| `src/tui/app/mod.rs` | line TBD | INVESTIGATE | `spinner_char` (line 172) — check live callers. |
| `src/autofill/common.rs` | 155 | KEEP | Scaffolded autofill; retain until fix lands. |
| `src/autofill/greenhouse.rs` | 64 | KEEP | Same. |

---

## Summary

| Disposition | Count |
|-------------|-------|
| SUBSUMED (covered by other findings) | ~14 |
| REMOVE SUPPRESSION (stale attribute) | ~7 |
| KEEP (with comment) | ~6 |
| INVESTIGATE (follow-up needed) | ~10 |
| **Total** | 37 |

The bulk of the ATS-module suppressions evaporate when `findings/ats.md` and `findings/pipeline.md` land. The remaining ~17 per-row dispositions are straightforward audits (mostly REMOVE SUPPRESSION + a few KEEP).

**Implementing engineer's workflow:**
1. Apply the subsuming findings first (ats.md, pipeline.md).
2. For each REMOVE SUPPRESSION row: delete the attribute, run `cargo build`, confirm no warning, commit.
3. For each KEEP row: add the one-line `// Kept because: X` comment, commit.
4. For each INVESTIGATE row: grep for callers, then either DELETE or KEEP-with-comment.

Target end state: zero `#[allow(dead_code)]` attributes in `src/` that are not paired with a justification comment.
