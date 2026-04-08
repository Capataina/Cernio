# Pipeline Separation Plan

> Restructure the Cernio pipeline so every step has exactly one purpose. Scripts handle volume, AI handles judgment. No step does both.

**Status:** Implemented — pipeline separation complete as of session 3 (2026-04-08). Script pipeline, AI skills, schema migration, and integrity checks all built. Runtime features (batch reporting, portfolio gap tracking during grading) deferred to first real use.

---

## Problem

The current skills conflate mechanical work with judgment work:

- `populate-db` spends ~20 minutes on 5–10 companies because it does ATS probing, web research, company grading, and DB insertion in a single pass.
- `search-jobs` will hit the same bottleneck at scale — fetching thousands of jobs and grading them in the same session will blow the context window.

At target scale (200 companies, 9,000 raw jobs, ~500 after filtering), the AI should never be touching the mechanical work. The pipeline must split into discrete stages where each step completes fully before the next begins.

---

## Target Pipeline

```
┌───────────────────────────────────────────────────────────────────────┐
│  1. discover-companies          (AI skill)                           │
│     Web search for new companies → land in potential.md              │
│                                                                      │
│  2. resolve-ats                 (Rust script + AI fallback)          │
│     Probe URL patterns → insert as resolved                         │
│     Failures → AI scrapes careers page as fallback                   │
│                                                                      │
│  3. grade-companies             (AI skill)                           │
│     Grade ungraded companies (S/A/B/C) → archive C-tier             │
│                                                                      │
│  4. search-jobs                 (Rust script)                        │
│     Fetch all jobs from resolved S/A/B companies                    │
│     → location filter → exclusion filter → inclusion filter          │
│     → auto-insert survivors into DB (no AI)                         │
│                                                                      │
│  5. grade-jobs                  (AI skill)                           │
│     Pick ungraded jobs from DB, prioritise by signal, grade in batch│
│                                                                      │
│  6. clean-db                    (Rust script)                        │
│     Remove F/C-graded jobs, stale listings >14d                     │
│                                                                      │
│  7. check-integrity             (hybrid)                             │
│     Script: verify ATS slugs, detect stale data                     │
│     AI: re-evaluate flagged entries                                  │
└───────────────────────────────────────────────────────────────────────┘
```

### What changed from the current design

| Before | After | Why |
|--------|-------|-----|
| `populate-db` does ATS + grading + insertion | Split into `resolve-ats` (script) + `grade-companies` (AI) | ATS probing is mechanical, grading is judgment |
| `search-jobs` does fetching + evaluation | Split into `search-jobs` (script) + `grade-jobs` (AI) | Fetching/filtering is mechanical, evaluation is judgment |
| Grading happens inline during discovery/search | Grading is a separate pass over ungraded DB entries | AI time is spent only on judgment, not on plumbing |
| No integrity checks | `check-integrity` detects staleness and drift | DB accumulates stale data without active maintenance |

---

## Step-by-step design

### Step 1: discover-companies (AI skill)

**Already designed.** Web search for companies matching the profile, deduplicate against DB and potential.md, land new entries in potential.md.

No changes needed to the existing skill.

- [ ] Update skill to read company websites from DB for deduplication (currently reads only potential.md)

---

### Step 2: resolve-ats (Rust script + AI fallback)

**Purpose:** Take companies from potential.md (or unresolved DB entries), probe ATS URL patterns, insert into DB as `resolved` or `bespoke`.

**Script phase (mechanical):**

```
For each company:
  1. Normalise name → generate slug candidates
     "XTX Markets" → ["xtxmarkets", "xtx-markets", "xtx", "xtxmarketstechnologies", ...]
  2. Probe each slug against each supported ATS:
     - Greenhouse: boards-api.greenhouse.io/v1/boards/{slug}/jobs
     - Lever: api.lever.co/v0/postings/{slug}
     - Ashby: api.ashbyhq.com/posting-api/job-board/{slug}
     - Workable: apply.workable.com/api/v1/widget/accounts/{slug}
     - SmartRecruiters: api.smartrecruiters.com/v1/companies/{slug}/postings
       (MUST check totalFound > 0 — SmartRecruiters returns 200 for any slug)
  3. Record ALL hits — do not stop at the first match.
     A company may use Lever for engineering and Workday for corporate.
     Insert one portal entry per hit, mark the one with the most jobs as is_primary.
  4. No hit on any provider → mark as "needs-ai-resolution"
```

**AI fallback phase:**

```
For each "needs-ai-resolution" company:
  1. Web search for "{company name} careers"
  2. Scrape careers page for ATS links
  3. If found → insert as resolved with correct slug
  4. If no ATS found → insert as bespoke with careers_url
  5. If company appears dead → skip, do not insert
```

**CLI:**
```bash
cernio resolve                    # resolve all pending companies
cernio resolve --company "XTX"    # resolve one company
cernio resolve --dry-run          # show what would happen
```

- [x] Implement slug candidate generation (name normalisation)
- [x] Implement parallel ATS endpoint probing across all providers
- [x] Handle SmartRecruiters false positive (totalFound > 0 check)
- [x] Insert resolved companies into DB with portal entries
- [x] Mark failures for AI fallback
- [x] Redesign `resolve-portals` AI skill for the fallback cases only

---

### Step 3: grade-companies (AI skill)

**Purpose:** Grade all ungraded companies in the DB. Assign S/A/B/C based on the profile. Soft-archive C-tier.

**Input:** `SELECT * FROM companies WHERE grade IS NULL`

**Per company, the AI evaluates:**
- Relevance to profile (tech stack, domain, role types)
- Career ceiling potential
- Sponsorship track record (critical for post-August 2027)
- Engineering culture signals
- Size and stability

**Output:** Update `grade`, `grade_reasoning`, `graded_at` on each company.

**C-tier handling:** Mark C-tier companies with `status = 'archived'` so they:
- Don't appear in the TUI
- Don't get searched for jobs
- Still exist for deduplication (discovery won't re-discover them)

This requires a schema change — add `'archived'` to the `status` CHECK constraint.

**Batch size:** All ungraded companies in one pass. Company count is manageable (tens, not thousands).

#### Skill quality requirements

This skill must be crafted to a high standard following the principles in `AgentCreationResearch/writing-skills.md`. The existing rubric in `skills/populate-db/references/company-grading-rubric.md` is a solid foundation but needs to be expanded into a standalone skill with:

- **Extensive SKILL.md** explaining the reasoning behind each grade level — not just "S means excellent" but *why* engineering reputation matters more than company size, *why* sponsorship likelihood is weighted higher than culture fit, *why* a B-tier company with interesting Rust work is worth keeping even if it's pre-revenue. The AI needs to understand the decision-making framework deeply enough to handle edge cases it's never seen before.

- **Detailed reference files:**
  - `references/grading-rubric.md` — expanded from the existing rubric with more worked examples across different company types (FAANG, fintech scaleup, infrastructure startup, defence, quant firm). Each example should show the reasoning chain, not just the outcome.
  - `references/profile-context.md` — a distilled view of what matters from the profile for company evaluation. The skill shouldn't need to re-read all 14 profile files — it should have a focused summary of the dimensions that matter for company grading.

- **Grade definitions with boundary guidance:** The agent must know where the lines are. What separates an A from a B? What makes something a C rather than being removed entirely? Concrete boundary examples prevent grade inflation and ensure consistency.

- [x] Create `grade-companies` skill with extensive SKILL.md
- [x] Write expanded grading rubric reference with worked examples and boundary cases
- [x] Write profile context reference for company evaluation
- [x] Add `'archived'` to companies status CHECK constraint (MIGRATION_002)
- [x] Update TUI queries to exclude archived companies
- [ ] Update `why_relevant` during grading

---

### Step 4: search-jobs (Rust script)

**Purpose:** Fetch all jobs from resolved companies above the configured grade threshold, apply mechanical filters, auto-insert survivors into DB. Zero AI involvement.

**Pipeline:**

```
1. Query DB for all resolved companies with grade >= configured threshold
   (default: S, A, B — configurable via preferences.toml)
   + their primary portal (ats_provider, ats_slug)

2. For each company, fetch ALL open jobs from their ATS
   (parallel HTTP — tokio tasks, rate-limited per provider)

3. Location filter (provider-specific patterns from preferences.toml):
   Keep jobs where location matches any pattern for this provider
   If location is NULL or ambiguous → keep it (false negative protection)

4. Exclusion filter (title keywords from preferences.toml):
   Remove jobs whose title contains any exclusion keyword
   Case-insensitive, word-boundary matching

5. Inclusion filter (title keywords from preferences.toml):
   Keep jobs whose title contains at least one inclusion keyword
   If title matches NONE of the inclusion keywords → remove
   Case-insensitive

6. Dedup against existing DB entries (by URL)
   Skip jobs already in the DB

7. Insert survivors into DB with:
   - company_id, portal_id
   - title, url, location, remote_policy, posted_date
   - raw_description (fetch detail page if available)
   - evaluation_status = 'pending'
   - grade = NULL
   - discovered_at = now
```

**Filter configuration in `profile/preferences.toml`:**

Every configurable option includes inline documentation explaining what it does, what valid values are, and how it affects the pipeline. The preferences file is the user's control surface — it should be self-documenting.

```toml
[search_filters]
# Minimum company grade to include in job searches.
# Companies below this threshold are skipped entirely.
#
# Valid grades (from best to worst):
#   S — Excellent company. Strong engineering reputation, clear technical
#       alignment with profile, growing, likely to sponsor, career progression.
#       Examples: Palantir, Cloudflare, XTX Markets.
#   A — Good company. Solid on most dimensions, one or two weaknesses.
#       Good engineering work, decent growth, less brand recognition.
#   B — Decent company. Has relevant engineering work but weaker on some
#       dimensions. Maybe small, uncertain growth, or narrow domain.
#   C — Marginal. Borderline relevance. Archived by default — not searched
#       for jobs, but kept in DB for deduplication.
#
# Default: "B" (search S, A, and B companies; skip C)
min_company_grade = "B"

# Title must contain at least one of these (OR logic).
# Case-insensitive matching. A job is KEPT if its title contains
# ANY of these keywords. If a title matches NONE, the job is removed.
#
# Bias toward inclusion — it's better to have false positives
# (AI grades them F in 30 seconds) than false negatives (perfect
# job silently filtered out and lost forever).
include_keywords = [
    "Engineer", "Engineering", "Developer", "Dev", "SWE", "Software",
    "Systems", "System", "Infrastructure", "Infra", "Platform",
    "Backend", "Back-end", "Back End", "Fullstack", "Full-stack",
    "ML", "Machine Learning", "AI", "Data",
    "Compiler", "Runtime", "Database", "Distributed",
    "Embedded", "Low-latency", "Performance", "HFT",
    "DevOps", "SRE", "Reliability", "Cloud",
    "Security", "Cryptography", "Cyber",
    "Quantitative", "Quant", "Research",
    "Graduate", "Grad", "Junior", "Entry",
    "Analyst",
]

# Title containing any of these is excluded (AND NOT logic).
# Case-insensitive matching. A job is REMOVED if its title contains
# ANY of these keywords, regardless of inclusion keywords.
# Exclusion runs BEFORE inclusion — excluded jobs never reach the
# inclusion filter.
#
# "Senior" is deliberately NOT here. Many companies use "Senior"
# for roles accessible to strong candidates without traditional
# years of experience. The AI will grade these appropriately —
# giving most a B or C, but not missing the ones where "Senior"
# just means "not intern".
exclude_keywords = [
    "Principal", "Staff", "Distinguished",
    "Director", "VP", "Vice President",
    "Head of", "Chief", "CTO", "CIO",
    "Manager", "Managing",
    "Intern", "Internship", "Apprentice", "Placement",
    "Lead Architect",
]

# Location patterns per ATS provider (OR within provider).
# Each ATS uses different location field formats. A job is KEPT if
# its location field contains ANY of the patterns for its provider.
# If a job's location is NULL or empty, it is KEPT (false negative
# protection — better to grade an irrelevant job than miss a good one).
#
# These patterns need research per provider. Different ATS systems
# use different location schemas:
#   Greenhouse: typically short ("London", "London, UK")
#   Lever: typically full ("London, United Kingdom")
#   Ashby: varies by company configuration
#   Workable: typically city-level
#   SmartRecruiters: typically "City, Country"
[search_filters.locations.greenhouse]
patterns = ["London"]

[search_filters.locations.lever]
patterns = ["London, UK", "London, United Kingdom", "London, England", "London"]

[search_filters.locations.ashby]
patterns = ["London"]

[search_filters.locations.workable]
patterns = ["London"]

[search_filters.locations.smartrecruiters]
patterns = ["London"]
```

**Note on "Senior":** Deliberately NOT in the exclusion list. Many companies use "Senior" for roles accessible to strong candidates without traditional YoE. The AI will grade these appropriately in step 5 — giving most of them a B or C, but not missing the ones where "Senior" just means "not intern".

**CLI:**
```bash
cernio search                         # search all resolved S/A/B companies
cernio search --company palantir      # search one company
cernio search --grade S               # only S-tier companies
cernio search --dry-run               # show filter stats without inserting
```

**Expected flow at scale:**
```
200 resolved companies
  → fetch all jobs         ~9,000 raw
  → location filter        ~1,500 (London/UK/Remote)
  → exclusion filter       ~1,200 (remove Principal/Director/etc.)
  → inclusion filter       ~600   (keep anything with Engineer/Dev/ML/etc.)
  → dedup                  ~500   (skip already-seen URLs)
  → INSERT 500 pending jobs into DB
```

- [x] Implement Greenhouse fetcher (`src/ats/greenhouse.rs`)
- [x] Implement Ashby fetcher (`src/ats/ashby.rs`)
- [x] Implement Workable fetcher (`src/ats/workable.rs`)
- [x] Implement SmartRecruiters fetcher (`src/ats/smartrecruiters.rs`)
- [x] Implement generic search pipeline (fetch → filter chain → insert)
- [x] Add `[search_filters]` section to `profile/preferences.toml`
- [x] Add location pattern config per provider
- [x] Add CLI commands: `cernio search`, `cernio search --company`, `cernio search --grade`
- [x] Research location field formats per ATS provider (separate task — each provider uses different location schemas)

---

### Step 5: grade-jobs (AI skill)

**Purpose:** Grade ungraded jobs from the DB. Work in batches, prioritised by signal.

**Prioritisation logic (AI decides batch order):**

```
Priority 1: Jobs at S-tier companies with promising titles
            (e.g. "Graduate SWE" at Apple > "Senior Analyst" at unknown company)
Priority 2: Jobs at A-tier companies
Priority 3: Jobs at B-tier companies
Priority 4: Remaining ungraded jobs
```

The AI reads the title, company name, company grade, and any available description to decide which jobs to grade first. This means the most likely strong matches get graded early in the session, and the user sees actionable results faster.

**Per job, the AI evaluates (reading the full description):**
- Career ceiling
- Tech stack match
- Breadth of engineering work
- Company signal
- Sponsorship likelihood
- Seniority fit

**Output:** Update `grade` (SS/S/A/B/C/F), `fit_assessment`, `fit_score`, `evaluation_status` on each job.

**Batch size:** ~30 jobs per session. The skill should state how many ungraded jobs remain so the user can decide whether to continue.

**Portfolio gap tracking:** While grading, the AI watches for patterns — skills, tools, or experience areas that appear repeatedly in strong matches but are absent from the profile. Updates `profile/portfolio-gaps.md`.

#### Skill quality requirements

Same high standard as `grade-companies`. The existing rubric in `skills/search-jobs/references/grading-rubric.md` is a strong starting point but needs expansion for standalone use:

- **Extensive SKILL.md** explaining the full evaluation framework — not just the dimensions but the reasoning behind their weights. Why is career ceiling critical? Why does skill breadth matter more early in a career? Why is sponsorship viability weighted as high as it is? The AI must understand the *philosophy* behind the grading so it can make judgment calls on roles that don't fit neatly into any category.

- **Detailed reference files:**
  - `references/grading-rubric.md` — expanded with more worked examples across different role types (new grad infra, mid-level backend, ML engineer, SRE, quant developer). Each example should show the full reasoning chain through every evaluation dimension. Include boundary cases: "This looks like an A but is actually a B because..." and "This looks like a C but is actually an S because..."
  - `references/profile-context.md` — focused summary of what matters from the profile for job evaluation. Career ceiling targets, tech stack strengths, seniority constraints, sponsorship timeline, dealbreakers.
  - `references/prioritisation-guide.md` — how to triage ungraded jobs for batch order. The compound signal of company grade × title keywords × role type. Worked examples of how to order a mixed batch of 50 ungraded jobs.

- **Grade definitions with boundary guidance:** Concrete examples of what separates each grade from the next. What makes an SS vs an S? When does a B become a C? The agent needs to know the lines, not just the labels.

- [x] Create `grade-jobs` skill with extensive SKILL.md (replacing evaluation logic in `search-jobs`)
- [x] Write expanded grading rubric reference with worked boundary examples
- [x] Write profile context reference for job evaluation
- [x] Write prioritisation guide reference
- [x] Implement smart prioritisation (company grade × title signal)
- [ ] Add batch progress reporting ("graded 30/142 pending")
- [ ] Integrate portfolio gap tracking

---

### Step 6: clean-db (Rust script)

**Purpose:** Remove noise from the DB — both bad jobs AND bad companies. Since company grading now happens as a separate step (not during initial population), the DB will contain ungraded and low-grade companies that need cleanup.

**Job cleanup:**

| Target | Condition | Rationale |
|--------|-----------|-----------|
| F-graded jobs | `grade = 'F'` | Categorically irrelevant — wrong department, wrong role type |
| C-graded jobs | `grade = 'C'` | Weak fit, not worth revisiting. Re-search would surface better versions |
| Stale jobs | `discovered_at` > 14 days, no user decision | Almost certainly closed or filled |

**Company cleanup:**

| Target | Condition | Rationale |
|--------|-----------|-----------|
| C-graded companies | `grade = 'C'` | Marginal relevance — soft-archive (set `status = 'archived'`), don't delete (preserves dedup) |
| Unresolvable companies | `status = 'potential'` with failed resolution attempts | Couldn't find ATS, no careers page — not worth keeping in active view |

**Preserves (never touches):**
- Jobs with user decisions (watching/applied/rejected) — user intent overrides everything
- SS/S-graded jobs regardless of age — high-value matches worth keeping as records
- All archived companies — they stay for dedup, just hidden from TUI and searches

**Cleanup thresholds are configurable in `profile/preferences.toml`:**

```toml
[cleanup]
# Job grades to remove during cleanup. Jobs with these grades are deleted
# unless they have a user decision (watching/applied/rejected).
# Valid grades: SS, S, A, B, C, F
remove_job_grades = ["F", "C"]

# Job age threshold in days. Jobs older than this with no user decision
# are removed, regardless of grade (except SS/S which are always kept).
stale_days = 14

# Company grades to archive during cleanup. Companies with these grades
# are set to status='archived' — hidden from TUI and excluded from
# job searches, but preserved for deduplication.
archive_company_grades = ["C"]
```

**CLI:**
```bash
cernio clean              # execute cleanup
cernio clean --dry-run    # show what would be removed/archived
cernio clean --jobs-only  # only clean jobs, skip companies
```

Also available from the TUI via `D` key with confirmation popup.

- [x] Implement `cernio clean` command
- [x] Implement `cernio clean --dry-run`
- [x] Implement company archival in cleanup
- [x] Add `[cleanup]` config section to `profile/preferences.toml`
- [x] Add `D` key to TUI dashboard with confirmation popup

---

### Step 7: check-integrity (hybrid)

**Purpose:** Comprehensive verification that the DB is accurate, complete, and up to date. This is not just a staleness detector — it's a full health report that identifies everything that's missing, broken, stale, or inconsistent.

**Script checks (mechanical):**

| Check | What it does | Action on failure |
|-------|-------------|-------------------|
| **ATS slug verification** | Re-probe each resolved company's ATS endpoint | Flag as `needs-reverification` |
| **Stale job detection** | Jobs with `discovered_at` > 14 days and no user decision | Flag for cleanup |
| **Empty company detection** | Resolved companies with 0 jobs after 3+ search runs | Flag for review — may have changed ATS |
| **Orphaned decisions** | User decisions pointing to deleted jobs | Clean up |
| **Duplicate detection** | Companies with similar names/websites that might be duplicates | Flag for manual review |
| **Missing portal detection** | Resolved companies with no portal entries | Flag — data integrity issue |
| **Dead URL detection** | Probe `careers_url` for bespoke companies — check for 404/dead | Flag for removal or re-resolution |

**Completeness checks (what's missing):**

| Check | What it does | Report |
|-------|-------------|--------|
| **Ungraded companies** | Companies with `grade IS NULL` | "X companies have not been graded yet" |
| **Ungraded jobs** | Jobs with `grade IS NULL` | "X jobs are pending grading" |
| **Missing descriptions** | Jobs with `raw_description IS NULL` | "X jobs have no description fetched — grading quality will be lower" |
| **Missing assessments** | Jobs graded but `fit_assessment IS NULL` | "X graded jobs have no fit assessment text" |
| **Unresolved companies** | Companies still at `status = 'potential'` | "X companies have not been resolved yet" |
| **Missing relevance** | Companies where `why_relevant` is empty or generic | "X companies have weak relevance explanations" |
| **Stale grades** | Companies graded > 30 days ago | "X company grades may be stale" |

**AI checks (judgment):**

| Check | What it does | Trigger |
|-------|-------------|---------|
| **Re-grade companies** | `why_relevant` and grade may be stale if profile changed significantly | Profile files modified since `relevance_updated_at` |
| **Re-evaluate strong fits** | SS/S jobs may no longer be strong if profile shifted (new skills, changed preferences) | Profile files modified since job grading date |
| **Validate bespoke companies** | Bespoke companies may now have a supported ATS — worth re-probing | Periodic (monthly) |
| **Grade quality audit** | Sample graded jobs and verify reasoning is sound, not just pattern-matching titles | On request — spot-check for grade inflation/deflation |
| **Relevance refresh** | Update `why_relevant` for companies where the profile has changed enough to shift relevance | Profile changes that affect sector/tech preferences |

**Output format:**

The script produces a structured report, not just a list of errors:

```
┌─ Integrity Report ─────────────────────────────────────────────┐
│                                                                 │
│  ── Health ──                                                   │
│  ✓  ATS slugs: 7/7 verified                                    │
│  ✓  No orphaned decisions                                       │
│  ✓  No duplicate companies                                      │
│  ✗  2 bespoke companies have dead careers URLs                  │
│  ✗  1 resolved company has no portal entries                    │
│                                                                 │
│  ── Completeness ──                                             │
│  ⚠  12 companies ungraded                                       │
│  ⚠  47 jobs pending grading                                     │
│  ⚠  8 jobs missing descriptions                                 │
│  ✓  All graded jobs have fit assessments                        │
│                                                                 │
│  ── Staleness ──                                                │
│  ⚠  3 company grades older than 30 days                         │
│  ⚠  Profile changed since last company grading pass             │
│  ✓  No jobs older than 14 days without decisions                │
│                                                                 │
│  ── Recommended Actions ──                                      │
│  1. Run grade-companies (12 ungraded)                           │
│  2. Run grade-jobs (47 pending)                                  │
│  3. Review dead bespoke URLs: [SurrealDB, Coadjute]             │
│  4. Fix missing portal for [Helsing]                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**CLI:**
```bash
cernio check              # run all integrity checks, print report
cernio check --ats-only   # just verify ATS slugs
cernio check --dry-run    # report without fixing anything
cernio check --fix        # auto-fix what can be fixed (orphans, etc.)
```

- [x] Implement ATS slug re-verification
- [x] Implement stale job detection
- [x] Implement completeness checks (ungraded counts, missing data)
- [x] Implement dead URL detection for bespoke companies
- [x] Implement duplicate/missing portal detection
- [x] Implement profile-change detection (compare file mtimes against `relevance_updated_at`)
- [x] Implement structured report output
- [ ] Create `check-integrity` AI skill for re-evaluation and grade quality auditing

---

## Schema changes required

### MIGRATION_002

```sql
-- Add 'archived' to company status options (for C-tier soft archive)
-- SQLite doesn't support ALTER CHECK, so this requires:
-- 1. Create new table with updated CHECK
-- 2. Copy data
-- 3. Drop old table
-- 4. Rename new table

-- Or: just remove the CHECK constraint and enforce in application code.
-- Simpler and more flexible. The CHECK was a nice-to-have, not a safety net.
```

Decision needed: remove the CHECK constraint on `companies.status` and enforce valid values in application code, or do the table-rebuild dance. Recommend removing the CHECK — it's simpler and the app already validates on insert.

- [x] Decide on CHECK constraint approach
- [x] Implement MIGRATION_002

---

## Implementation order

The steps below are ordered by dependency and value. Each is independently shippable.

```
Phase 1: Foundation                                          ✅ DONE
  ├── 1a. Add [search_filters] to preferences.toml
  ├── 1b. Implement Greenhouse fetcher (src/ats/greenhouse.rs)
  ├── 1c. Implement Ashby fetcher (src/ats/ashby.rs)
  └── 1d. MIGRATION_002 (add 'archived' status)

Phase 2: Script pipeline                                      ✅ DONE
  ├── 2a. Implement resolve-ats script (slug probing + DB insertion)
  ├── 2b. Implement search-jobs script (fetch → filter → insert)
  └── 2c. Implement clean-db script

Phase 3: AI skills                                            ✅ DONE
  ├── 3a. Create grade-companies skill
  ├── 3b. Create grade-jobs skill (with smart prioritisation)
  └── 3c. Redesign resolve-portals as AI-fallback-only skill

Phase 4: Integrity (script only)                              ✅ DONE
  ├── 4a. Implement check-integrity script (mechanical checks)
  └── 4b. Create check-integrity AI skill (re-evaluation)   ← not created as skill file, script only

Phase 5: Integration                                          ✅ DONE
  ├── 5a. Update TUI to exclude archived companies
  ├── 5b. Add clean-db trigger to TUI (D key)
  └── 5c. Update existing skills to remove grading logic
```

---

## Completion criteria

**Pipeline separation:**
- [x] Every step in the pipeline has exactly one purpose
- [x] No AI skill does mechanical work (HTTP calls, filtering, DB insertion)
- [x] No Rust script does judgment work (grading, evaluation, relevance assessment)

**Script pipeline:**
- [x] `cernio resolve` probes ALL providers per company (not just first hit) and records all portals
- [x] `cernio search` fetches and filters thousands of jobs without any AI involvement
- [x] `cernio clean` removes bad jobs AND archives bad companies
- [x] `cernio check` produces a comprehensive structured report covering health, completeness, and staleness

**AI skills:**
- [x] `grade-companies` and `grade-jobs` skills are crafted to a high standard with extensive SKILL.md files, detailed reference files with worked examples, boundary cases, and reasoning chains
- [x] `grade-jobs` works in batches from the DB, prioritised by signal (company grade × title promise)
- [x] Both grading skills follow the principles in `AgentCreationResearch/writing-skills.md` — explain why, not just what

**Configurability:**
- [x] Everything configurable lives in `preferences.toml` with inline documentation
- [x] Each config option explains what it does, what valid values are, and how it affects the pipeline
- [x] Company grade threshold for job searches is configurable
- [x] Cleanup thresholds (which grades to remove, stale age) are configurable
- [x] Filter keywords and location patterns are configurable per ATS provider

**Data integrity:**
- [x] C-tier companies are soft-archived, not deleted (dedup preservation)
- [x] `check-integrity` detects ungraded entries, missing data, stale grades, dead URLs, and orphaned records
- [x] `check-integrity` recommends specific next actions based on findings
- [x] All existing tests still pass
