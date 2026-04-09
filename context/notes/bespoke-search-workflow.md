# Bespoke Search Workflow

## Core Rule

Bespoke search results **must be inserted into the `jobs` table** — not just reported conversationally. If a bespoke search agent finds a matching role at a company's careers page, it must produce an `INSERT OR IGNORE INTO jobs` statement with at minimum: `company_id`, `title`, `url`, `location`, `evaluation_status = 'pending'`, `discovered_at`.

This was a session 7 gap: agents found ~13 strong leads (Apple compiler roles, Citadel grad programme, Bloomberg 2026 SWE, Google grad, Arm grad, Mastercard grad programme, Amazon intern) but initially only reported them in conversation without DB insertion. The jobs were invisible to the TUI and grading pipeline until manually inserted.

## Why This Matters

- Jobs not in the DB don't appear in the TUI
- Jobs not in the DB can't be graded by the grade-jobs skill
- Jobs not in the DB can't be tracked in the pipeline (watching/applied/rejected)
- Jobs not in the DB get re-discovered and re-reported in future bespoke searches

## What the Search Agents Should Do

1. Search company career pages + external aggregators (LinkedIn, Indeed, Glassdoor)
2. For each matching role found, return a structured result with: title, URL, location
3. The orchestrator inserts all results into the DB via `INSERT OR IGNORE INTO jobs`
4. The URL unique constraint prevents duplicates automatically

## What the Orchestrator Should Do After Bespoke Search

1. Collect all agent results
2. Map company names to `company_id` values
3. Insert all found jobs with `evaluation_status = 'pending'`
4. Report what was inserted vs what was already in the DB
5. Queue the new bespoke jobs for grading alongside the automated search results

## Limitations of Bespoke Search

- Agents can only web-search and fetch public pages — they can't log into career portals
- Some career pages (Meta, Microsoft, Reactive Markets) load jobs via JavaScript that agents can't render
- Results are a lower bound, not exhaustive — manual portal visits may find more
- Bespoke jobs don't have `raw_description` populated — the grading agents need to fetch descriptions from the job URLs
