# Scaling Architecture

How the system handles hundreds of companies and thousands of jobs without hitting session limits or context constraints.

---

## Core principle

**Scripts for volume, AI for judgment. Every step has one purpose.**

The pipeline is split into discrete stages. Each stage completes fully before the next begins. No stage mixes mechanical work with judgment work.

---

## What scripts handle

| Task | Why a script |
|------|-------------|
| Probe ATS URL patterns for company slugs | HTTP probing, no judgment |
| Fetch all jobs from ATS APIs | Parallel HTTP, no judgment |
| Location filtering | Pattern matching against provider-specific strings |
| Title exclusion filtering | Keyword matching — "Principal", "Director", etc. |
| Title inclusion filtering | Keyword matching — "Engineer", "ML", "Systems", etc. |
| Dedup against existing DB entries | URL comparison |
| Insert into DB | SQL inserts |
| Clean stale/low-grade entries | Rule-based deletion |
| Verify ATS slugs still work | HTTP probing |

All filters live in `profile/preferences.toml`, not hardcoded. Configurable per ATS provider.

---

## What AI handles

| Task | Why AI |
|------|--------|
| Discovering new companies from web sources | Creative search, dedup judgment |
| Resolving ATS for companies where probing failed | Web search, careers page scraping |
| Grading companies (S/A/B/C) | Profile-aware relevance assessment |
| Grading jobs (SS–F) | Reading descriptions, multi-dimensional fit evaluation |
| Re-evaluating stale entries | Judgment about whether profile changes affect fit |
| Tracking portfolio gaps | Pattern recognition across many evaluations |

AI works from the database in batches across sessions. "Grade the next 30 pending jobs" is a valid session task.

---

## Batch grading prioritisation

The AI doesn't grade jobs randomly. It prioritises by signal:

```
Priority 1: S-tier companies + promising titles (e.g. "Graduate SWE @ Apple")
Priority 2: A-tier companies
Priority 3: B-tier companies
Priority 4: Remaining ungraded
```

This means the most likely strong matches get graded first, and the user sees actionable results early in each session.

---

## False negatives are the enemy

At every stage of filtering, the bias must be toward inclusion. A job that gets through mechanical filtering and turns out irrelevant costs the AI 30 seconds to grade as F. A job that gets mechanically filtered out and was actually a perfect fit is a lost opportunity that can never be recovered.

The mechanical filters should only remove things that are:
- Physically impossible (wrong country, no remote option)
- Categorically irrelevant by title (Director, VP, Intern, Principal)
- Outside the inclusion keywords (Legal Counsel, Marketing Manager)

"Senior" is deliberately NOT excluded — many companies use it for roles accessible to strong candidates.

Everything else goes to the AI for judgment.

---

## Expected flow at scale (updated session 4)

```
273 active companies (actual as of 2026-04-09)
  resolve-ats script    → 167 resolved, 83 still potential, 23 bespoke
  grade-companies AI    → 25S / 107A / 94B / 47C
  C companies kept active (not archived — job grading handles quality)

273 graded companies (all searched, min_company_grade = "C")
  search-jobs script    → raw jobs fetched across all providers
  location filter       → UK/London/Remote/Cambridge
  exclusion filter      → remove Principal/Director/etc.
  inclusion filter      → keep anything with Engineer/Dev/ML/etc.
  dedup                 → skip already-seen URLs
  → 712 jobs in DB (12 SS, 55 S, 90 A, 130 B)

  grade-jobs AI         → 30 per session, batched by company grade × title signal
  clean-db script       → remove F/C grades + stale (does NOT archive companies)

Steady state: ~50–100 active jobs, refreshed weekly
```

---

## Full design

See `context/plans/pipeline-separation.md` for the complete implementation plan with checkboxes, CLI commands, schema changes, and phased implementation order.
