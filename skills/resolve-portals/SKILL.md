# resolve-portals

AI fallback for ATS resolution — finds the job board portal for companies that the mechanical Rust resolver (`cernio resolve`) could not match. Use when the user says "resolve portals", "find ATS for remaining companies", "handle unresolved companies", "resolve the unmatched ones", or "find their job boards". Not for discovering new companies (that's discovery), not for grading companies (that's grade-companies), not for searching jobs (that's the search scripts). This skill has one purpose: turn an unresolved company into a resolved or bespoke entry by finding its ATS portal.

---

## Mandatory reads — do not proceed without completing these

**STOP. Before resolving any company's ATS portal, you MUST read these files in full:**

1. **`references/ats-providers.md`** — the complete reference for all ATS providers: supported providers with API details, unsupported providers to recognise, slug patterns, and verification methods. This file contains critical information like SmartRecruiters false positives that will cause incorrect results if not read.

**When delegating resolution to subagents:** embed the FULL TEXT of `references/ats-providers.md` in each agent's prompt. Without it, agents will not know the correct API endpoints, verification methods, or provider-specific gotchas.

**Do not begin resolving any company until the mandatory read is complete.**

---

## Why this skill exists

The pipeline has a mechanical first pass: `cernio resolve` probes predictable slug patterns against every supported ATS provider. It resolves the easy cases — companies whose slugs match their name or obvious variations. But many companies have non-obvious slugs (XTX Markets uses `xtxmarketstechnologies` — their legal entity name), use unsupported ATS providers, or have custom-built career portals. These are the companies that reach this skill.

The agent brings what the script cannot: web search, careers page interpretation, and judgment about which ATS link is the real one. The goal is to move every remaining `potential` company to either `resolved` (ATS found) or `bespoke` (no supported ATS, but careers URL preserved).

---

## Context before you start

The user has already run `cernio resolve`. The companies reaching this skill failed mechanical slug probing — simple guesses like `{company-name}`, `{domain-without-tld}`, `{hyphenated-name}` did not return valid results on any supported ATS. Expect these companies to fall into three buckets:

1. **Non-obvious slugs on supported ATS** — The company uses Greenhouse/Lever/Ashby/etc., but their slug is unexpected (a former name, a legal entity name, an abbreviation nobody would guess). Their careers page will reveal the correct URL.
2. **Unsupported ATS providers** — The company uses iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, or a custom portal. These become `bespoke` with the careers URL preserved.
3. **Dead or disappeared companies** — No website, no careers page, Companies House shows dissolved. Skip these entirely.

---

## How resolution works

### 1. Query unresolved companies

Query the database for companies that need resolution:

```sql
SELECT id, name, website, careers_url FROM companies WHERE status = 'potential';
```

Present the list to the user before starting work. If the list is large, offer to process in batches.

### 2. For each company, find the careers page

This is where the agent's value lies. The mechanical resolver already tried slug guessing — repeating that is wasted effort.

**Go straight to the source.** Use WebSearch to find the company's careers page:
- Search for `"{company name}" careers` or `"{company name}" jobs`
- Visit the company's website and look for a Careers/Jobs link in navigation or footer
- Check the careers page source and outbound links for ATS URLs

**What you are looking for on the careers page:**
- Links or iframes pointing to `boards.greenhouse.io`, `jobs.lever.co`, `jobs.ashbyhq.com`, `apply.workable.com`, `jobs.smartrecruiters.com`, or Workday/Eightfold subdomains
- JavaScript embeds that load job listings from an ATS API
- Redirect chains — the "View all jobs" button often redirects to the ATS board

The reference file `references/ats-providers.md` has the full list of URL patterns, verification methods, and gotchas per provider. Read it before your first resolution — it contains critical information like SmartRecruiters returning false positives.

### 3. Verify the portal

Once you find an ATS URL, verify it works:
- Extract the slug from the URL
- Hit the provider's JSON API endpoint (see `references/ats-providers.md` for the exact endpoints)
- Confirm the response is valid JSON with a parseable job structure

A portal with zero current jobs is still valid — the company may not be actively hiring right now, but the slug works and jobs may appear later. Mark it as resolved.

**SmartRecruiters caveat:** Their API returns HTTP 200 with `{"totalFound": 0}` for completely fabricated slugs. A 200 response from SmartRecruiters means nothing — only trust it if `totalFound > 0`. If you find SmartRecruiters links on the careers page but the API returns zero jobs, note this ambiguity and check whether the company may have migrated away.

### 4. Handle the result

**If a supported ATS portal is found:**

```sql
INSERT INTO company_portals (company_id, ats_provider, ats_slug, verified_at, is_primary)
VALUES (?, ?, ?, date('now'), 1);

UPDATE companies SET status = 'resolved' WHERE id = ?;
```

**If multiple portals are found** (e.g. an active Ashby board and a residual Workable board after migration), insert both — mark the active one as primary (`is_primary = 1`) and the residual one as secondary (`is_primary = 0`). Companies migrate ATS providers — finding two portals is expected, not an error.

**If the company uses an unsupported ATS or a custom portal:**

```sql
UPDATE companies SET status = 'bespoke', careers_url = ? WHERE id = ?;
```

The careers URL should point to the actual job listings page, not just the company homepage.

**If the company appears dead** (no website, no careers page, Companies House shows dissolved, no recent activity):

Do not insert anything. Report the finding to the user with evidence (e.g. "Companies House shows liquidation filed 2025-08, website returns 404"). The user decides whether to delete the company from the database or leave it.

### 5. Present results for review before writing

Collect all findings into a summary table before making any database writes:

```
| Company        | Result   | ATS       | Slug                    | Evidence                              |
|----------------|----------|-----------|-------------------------|---------------------------------------|
| XTX Markets    | resolved | greenhouse | xtxmarketstechnologies | Found via careers page link           |
| SurrealDB      | bespoke  | pinpoint  | —                       | surrealdb.pinpointhq.com (unsupported)|
| Vypercore      | dead     | —         | —                       | Companies House: in liquidation       |
```

Wait for the user to approve before executing the SQL. This is a review gate — the user should be able to correct mistakes, skip companies, or ask for re-research before anything touches the database.

---

## Lessons from production

These are hard-won lessons from actual resolution runs. They represent real failure modes, not theoretical concerns.

**Slug guessing is unreliable — the careers page is the answer.** XTX Markets uses `xtxmarketstechnologies`. Wise is `transferwise`. Cleo is `cleoai`. The only reliable way to find the correct slug is to find the company's careers page and extract the ATS URL from it. The mechanical resolver already tried guessing — this skill should go straight to the careers page.

**SmartRecruiters gives false positives.** Their API returns 200 with `{"totalFound": 0}` for any slug, real or fabricated. Only count a SmartRecruiters hit if `totalFound > 0`.

**Companies migrate ATS providers.** ClearBank has an active Ashby board (25 jobs) and a residual Workable board (0 jobs). Both portals should be recorded, with the active one marked as primary. The schema supports this — use it.

**Companies House is useful for UK companies.** When a company's website is dead and there is no online presence, Companies House can confirm whether the company is dissolved, in liquidation, or still active. This saves time that would otherwise be spent searching for a ghost.

**Unsupported ATS providers are common.** Pinpoint HQ, Personio, iCIMS, Taleo, BambooHR, Jobvite — these all exist in the wild. Mark them as bespoke with the careers URL. The information is preserved for manual checking or future ATS support.

---

## Quality checklist

Before presenting results to the user, verify:

- [ ] Every resolved portal has been verified against the provider's JSON API — not just a 200 response, but valid parseable job data (or confirmed zero jobs with a provider that does not give false positives)
- [ ] SmartRecruiters results have `totalFound > 0` — a zero-result 200 is not evidence of usage
- [ ] The slug belongs to the correct company, not a different company with a similar name
- [ ] No duplicate portals — check existing `company_portals` entries before inserting
- [ ] Bespoke companies have a careers page URL, not just a homepage URL
- [ ] Dead companies have evidence cited (Companies House status, 404 website, etc.)
- [ ] Results are presented in a summary table for user review before any database writes
- [ ] When multiple portals are found for one company, `is_primary` is set correctly (active portal = 1, residual = 0)
