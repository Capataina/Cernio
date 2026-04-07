# Populate-DB Lessons

Lessons from the first populate-db test run (2026-04-07), processing 8 companies from discovery.

---

## Slug guessing is unreliable

XTX Markets uses the slug `xtxmarketstechnologies` — their legal entity name, not their brand name. Naive variations of "xtx", "xtxmarkets", "xtx-markets" would all fail. The only way to find it was scraping their careers page and seeing the Greenhouse URL.

**Implication:** Deterministic slug probing is a first pass, not a solution. When probing fails, the company's own careers page almost always reveals the correct ATS URL. Scrape the careers page, look for links to Greenhouse/Lever/Ashby/etc.

---

## SmartRecruiters returns 200 for any slug

The SmartRecruiters API (`api.smartrecruiters.com/v1/companies/{slug}/postings`) returns HTTP 200 with `{"totalFound": 0, "content": []}` for completely made-up company names. This means a 200 response is NOT evidence of the company using SmartRecruiters.

**Implication:** For SmartRecruiters, only count a hit if `totalFound > 0`. A 200 with zero jobs is a false positive.

---

## Companies migrate ATS providers

ClearBank has an active Ashby board (25 jobs) AND a residual Workable board (0 jobs). They clearly migrated from Workable to Ashby at some point.

**Implication:** The multi-portal schema was the right call. When checking multiple providers, finding a company on two platforms isn't an error — it's expected. Mark the active one as primary.

---

## Unsupported ATS providers exist

SurrealDB uses Pinpoint HQ (`surrealdb.pinpointhq.com`), a UK-based ATS not in our supported list. These companies go to bespoke with their careers URL preserved.

**Implication:** We may want to add Pinpoint HQ support later if enough companies use it. For now, bespoke status preserves the information.

---

## Validation catches dead companies

Vypercore was confirmed dead via Companies House (in liquidation, no website, overdue accounts). Without the validation step, it would have entered the database as a company with no ATS — wasting a row and potentially misleading future job searches.

**Implication:** The validation step in populate-db (check website, check for activity, verify independence) is essential. Companies House is a useful verification source for UK companies.
