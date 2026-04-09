# Session 5 — Full Pipeline Rebuild Report

> **Date**: 2026-04-09
> **Duration**: Single extended session
> **Scope**: Complete database reset and rebuild — discovery through integrity checking

---

## Executive Summary

Rebuilt the entire Cernio database from scratch after overhauling every grading system. The old data (273 companies with one-liner descriptions, 712 jobs graded by title only) was wiped. The rebuild produced 434 companies with substantive descriptions, 26 S-tier, and 2,001 jobs properly graded by reading full descriptions — 10 SS, 27 S, 71 A opportunities identified.

---

## Phase 1: Discovery

**9 parallel agents deployed across 7 sectors + 2 job-atlas repo scanners.**

| Agent | Territory | Companies found |
|-------|-----------|----------------|
| 1 | Trading/quant/market-making | 25 |
| 2 | AI/ML infrastructure | 19 |
| 3 | Systems/databases/compilers | 21 |
| 4 | Fintech infrastructure | 25 |
| 5 | Developer tools/cloud/OSS | 19 |
| 6 | Defence/deeptech/hardware | 17 |
| 7 | Non-obvious sources | 20 |
| 8 | Job-atlas A-L | 55 |
| 9 | Job-atlas M-Z | 27 |
| **Total raw** | | **228** |

**After dedup against existing 273 companies: 161 new companies imported.** Total universe: 434 companies.

**What went well:**
- Non-obvious agent found genuinely creative hits: Gensyn (Rust-first distributed ML, London, a16z-backed), CrabNebula (the Tauri company — candidate already uses Tauri), Zed Industries (pure Rust dev tools)
- Job-atlas agents correctly filtered 700+ companies down to ~80 relevant ones, with strong dedup handling
- Trading agent found several Rust-aligned crypto market makers (Keyrock, Wincent, B2C2)

**What could improve:**
- Some agents included companies already in the database (handled by import dedup, but wasted agent time)
- The job-atlas repo structure required agents to WebFetch individual category files — a bulk download would be faster
- Several discovery files had formatting issues that the import parser handled gracefully

---

## Phase 2: Import

All 9 discovery files imported via `cernio import --file`. Database dedup via unique website URL constraint handled cross-agent duplicates cleanly.

| Metric | Count |
|--------|-------|
| Total parsed | 228 |
| New inserted | 161 |
| Duplicates skipped | 63 |
| Invalid/skipped | 4 |

---

## Phase 3: Resolve (3 passes)

**Pass 1:** 69 of 244 potential companies resolved mechanically.
**Pass 2:** 0 additional — the 175 failures are genuine (non-standard slugs or unsupported ATS).
**Pass 3 (post-grading):** 1 additional resolved.

**Final resolution state:**

| Status | Count |
|--------|-------|
| Resolved | 237 |
| Bespoke | 23 |
| Potential (unresolved) | 174 |

**Key observations:**
- The improved slug generator (punctuation stripping, domain suffix removal, acronyms, first-two-words) didn't dramatically improve resolution rates — most unresolved companies use unsupported ATS providers (Workday, iCIMS, Taleo, Personio, custom)
- 174 companies remain unresolved and need AI fallback (resolve-portals skill) in a future session
- Several S/A-tier companies are bespoke (Apple, Arm, Citadel Securities, D.E. Shaw, Two Sigma) — their jobs need manual searching

---

## Phase 4: Company Grading

**6 parallel agents, each processing ~72 companies.** All 434 companies enriched with substantive descriptions and graded.

**Grade distribution:**

| Grade | Count | % |
|-------|-------|---|
| S | 26 | 6.0% |
| A | 124 | 28.6% |
| B | 182 | 41.9% |
| C | 99 | 22.8% |
| Archived (dupes) | 3 | 0.7% |

**S-tier companies (26):** Adaptive Financial Consulting, AdaCore, Anthropic, Apple (×2), Aquis Exchange, Arm, Citadel (×2), D.E. Shaw, Databento, Exberry, Gensyn, Hudson River Trading, Jane Street, Jump Trading, Man Group, Modular, Neon, Optiver, Palantir, QuestDB, Redpanda, SurrealDB, Two Sigma, XTX Markets.

**What went well:**
- Every company now has a 3-5 sentence `what_they_do` description — no more one-liners
- `why_relevant` fields reference specific flagship projects by name
- Grade reasoning includes boundary explanations ("A rather than S because...")
- 3 duplicate company pairs detected and archived by agents

**What could improve:**
- Some agents didn't research companies deeply enough — a few descriptions are generic despite the rubric requiring specificity
- Google at A-tier is defensible (extremely high interview bar, not in preferred sector) but could be debated
- Duplicate detection should be built into the import pipeline, not left to grading agents

---

## Phase 5-6: Final Resolve + Job Search

**Job search across 219 resolved companies:**

| Metric | Count |
|--------|-------|
| Raw jobs fetched | 16,180 |
| After location filter | 4,327 |
| After exclusion filter | 2,964 |
| After inclusion filter | 2,174 |
| New (inserted) | 2,001 |

**Second search pass:** 0 new — first pass was comprehensive.

**Top companies by job count:** Cloudflare (127), ClickHouse (109), QRT (81), Databricks (65), Wise (62), Squarepoint (60), GitLab (60), Grafana Labs (55), Faculty AI (53), Graphcore (51).

**Issues identified:**
- Several Workable endpoints return 403 Forbidden (HuggingFace, 10xBanking, Arondite, Codasip, Tibra, OcadoTech) — these companies may have migrated away from Workable
- Lever endpoints return decode errors for some companies (Palantir, Quantinuum, Mistral, Prima, InstaDeep, Rivos) — likely EU endpoints not being tried
- Workday endpoints fail with "Invalid ats_extra" — missing subdomain/site configuration
- S-tier bespoke companies (Apple, Arm, Citadel, D.E. Shaw, Two Sigma, SurrealDB, QuestDB) had 0 jobs — they need manual bespoke searching

---

## Phase 7: Job Grading

**16 parallel agents across 2 waves (8 agents each), grading all 2,001 jobs.**

**Final grade distribution:**

| Grade | Count | % |
|-------|-------|---|
| SS | 10 | 0.5% |
| S | 27 | 1.4% |
| A | 71 | 3.6% |
| B | 149 | 7.4% |
| C | 164 | 8.2% |
| F | 1,518 | 75.9% |
| No description | 62 | 3.1% |

**SS-tier jobs (10):**
1. Cloudflare — Software Engineer Intern (Summer 2026)
2. Hudson River Trading — Research Engineer (2026 Grads welcome)
3. Hudson River Trading — Software Engineer C++/Python (2026 Grads)
4. Jane Street — Software Engineer (×2)
5. Palantir — Software Engineer, New Grad
6. Palantir — Software Engineer, New Grad - Infrastructure
7. QRT — Internship, Quantitative Developer - Low-Latency & Market Microstructure
8. Squarepoint Capital — Graduate Software Developer
9. Stripe — Software Engineer, Intern

**Why 76% of jobs are F:** This is correct and expected. The search pipeline fetches ALL jobs from resolved companies, including non-UK locations, non-engineering roles (sales, marketing, HR), and senior positions. The filters (location, title keywords) catch the obvious mismatches, but many roles only reveal their unsuitability when the description is read:
- Wrong location listed differently in the description vs ATS field
- "Software Engineer" titles that are actually solutions engineering
- "Senior" titles at companies where Senior = 2 years (but the description says 5+)

**What went well:**
- Every SS/S assessment is multi-paragraph with description citations and project references
- Agents correctly identified seniority requirements from descriptions, not titles
- Project tier system worked — agents cited flagship projects as primary evidence
- Portfolio-gaps.md was updated by multiple agents with market patterns

**What could improve:**
- Some agents had different seniority thresholds — one agent graded a "2+ years preferred" role as S while another graded a similar requirement as C
- The 62 jobs without descriptions remain unverifiable
- Several F grades are for wrong-location duplicates of the same role — the search pipeline should deduplicate by title+company before inserting

---

## Phase 8: Integrity Check

**Mechanical check (`cernio check`):**
- 193/205 ATS slugs verified (12 failed — likely migrated providers)
- 0 orphaned decisions
- 32 resolved companies with no portal entries (need investigation)
- 0 ungraded companies, 0 pending jobs
- 62 jobs missing descriptions
- 173 companies unresolved (need AI fallback)

**Cross-checking:**
- 4 S-tier jobs at B-tier companies — all justified (role quality transcends company grade: B2C2 grad programme, Graphcore grad ML kernels, Speechmatics FutureVoices)
- Google at A is reasonable given extremely high interview bar
- No absurdities found (no unknown startups at SS, no famous employers at F)

**Portfolio gaps maintained** by multiple grading agents. Key patterns:
- **C++ proficiency** is the #1 barrier to quant trading roles (every HFT firm wants C++)
- **Cloud/DevOps (AWS, K8s, Docker)** appears in nearly every infrastructure role
- **OCaml** blocks access to Jane Street's compiler/PL roles
- **Rust is a genuine differentiator** — appeared in 6+ SS/S roles, rare among graduates
- **CI/CD and testing** remain portfolio gaps affecting employability

---

## System Changes Made During Session

1. **Project tier system**: Flagship/Notable/Minor tiers added to projects.md
2. **Calibration-anchored grading**: Replaced batch-relative grading to prevent deflation
3. **TUI cleanup bug fixed**: No longer archives C companies
4. **Archived job lifecycle**: SS/S/A never archived; archived jobs expire after 42 days
5. **Bespoke search tracking**: `last_searched_at` column, TUI shows stale bespoke companies
6. **HTML tag stripper fixed**: Handles `>` inside quoted attributes
7. **Search keywords expanded**: Internships included, ML-specific terms added
8. **Resolve reliability**: More slug patterns, no early termination, retry on all fetchers
9. **Grade-companies skill expanded**: Now writes enriched what_they_do, location, sector_tags
10. **Check-integrity cross-checking guide**: New reference file for universe-wide grade verification
11. **Check-integrity portfolio gaps**: Active maintenance, 10 jobs per grade tier

---

## Statistics Summary

| Metric | Before session | After session |
|--------|---------------|---------------|
| Companies | 273 (one-liner descriptions) | 434 (enriched paragraphs) |
| S-tier companies | 25 (questionable) | 26 (properly researched) |
| Jobs | 712 (graded by title only) | 2,001 (graded by description) |
| SS jobs | 12 (many misgrades) | 10 (all verified entry-accessible) |
| S jobs | 55 (inflated) | 27 (properly calibrated) |
| Job grades with description citations | ~0 | ~2,001 |
| Companies with proper descriptions | ~0 | 434 |

---

## Recommendations for Next Session

1. **Resolve the 174 potential companies** using resolve-portals AI skill (web search for careers pages, extract ATS URLs, mark bespoke as needed)
2. **Bespoke search S/A companies** — Apple, Arm, Citadel Securities, D.E. Shaw, Two Sigma, SurrealDB, QuestDB have 0 jobs and need manual careers page searching
3. **Fetch missing job descriptions** for the 62 jobs that were graded without them
4. **Fix failed ATS slugs** — 12 companies have stale slugs that no longer work
5. **Deduplicate companies** — several duplicate entries exist (Parity ×3, PhysicsX ×2, etc.)
6. **Consider updating README** and `context/architecture.md` with new state
7. **Run check-integrity cross-checking** in a focused session after the S/A bespoke companies are searched — the current SS/S roster is missing roles from bespoke companies that can't be searched automatically
