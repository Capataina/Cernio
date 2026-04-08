# ATS Providers Reference

Complete reference for all ATS providers relevant to Cernio — supported providers with API details, unsupported providers to recognise, and identification techniques for unknown career pages.

---

## Table of Contents

1. [Supported Providers](#supported-providers)
   - [Greenhouse](#greenhouse)
   - [Lever](#lever)
   - [Ashby](#ashby)
   - [Workable](#workable)
   - [SmartRecruiters](#smartrecruiters)
   - [Workday](#workday)
   - [Eightfold](#eightfold)
2. [Unsupported Providers (mark as bespoke)](#unsupported-providers)
3. [Identifying the ATS from a Careers Page](#identifying-the-ats-from-a-careers-page)
4. [Common Slug Patterns](#common-slug-patterns)

---

## Supported Providers

### Greenhouse

The most common ATS among tech companies. Two hosting variants exist.

| Detail | Value |
|--------|-------|
| **Job board URL** | `https://boards.greenhouse.io/{slug}` |
| **Alt board URL** | `https://job-boards.greenhouse.io/{slug}` |
| **JSON API** | `https://boards-api.greenhouse.io/v1/boards/{slug}/jobs` |
| **Verification** | GET the API endpoint. 200 with valid JSON = confirmed. 404 = slug does not exist. |
| **Response shape** | `{"jobs": [...], "meta": {"total": N}}` |
| **Zero jobs** | Returns `{"jobs": [], "meta": {"total": 0}}` — this is a valid slug with no current openings. |

**How to spot on a careers page:**
- Links containing `boards.greenhouse.io` or `job-boards.greenhouse.io`
- Iframes loading from `boards.greenhouse.io`
- JavaScript embed scripts referencing `greenhouse`
- URL redirects from the company's careers page to `boards.greenhouse.io/{slug}`
- Query parameters like `gh_jid` in job application URLs

**Gotchas:**
- Some companies use custom domains that proxy Greenhouse (e.g. `careers.company.com` that loads Greenhouse content). Inspect the page source for Greenhouse references.
- The `boards.greenhouse.io` and `job-boards.greenhouse.io` variants use the same API.

---

### Lever

Common among mid-size tech companies and startups.

| Detail | Value |
|--------|-------|
| **Job board URL** | `https://jobs.lever.co/{slug}` |
| **JSON API** | `https://api.lever.co/v0/postings/{slug}` |
| **Verification** | GET the API endpoint. 200 with a JSON array = confirmed. 404 = slug does not exist. |
| **Response shape** | `[{"id": "...", "text": "Job Title", "categories": {...}, ...}, ...]` — a flat array of posting objects. |
| **Zero jobs** | Returns `[]` — empty array. This is a valid slug with no current openings. |

**How to spot on a careers page:**
- Links containing `jobs.lever.co`
- Iframes loading from `jobs.lever.co`
- Apply buttons linking to `jobs.lever.co/{slug}/{job-id}`

**Gotchas:**
- Lever slugs are typically simple lowercase company names, but can include hyphens.
- Some companies embed Lever listings via JavaScript widget rather than linking directly — look for `lever` in page source.

---

### Ashby

Growing in popularity, especially among well-funded startups.

| Detail | Value |
|--------|-------|
| **Job board URL** | `https://jobs.ashbyhq.com/{slug}` |
| **JSON API** | `https://api.ashbyhq.com/posting-api/job-board/{slug}` |
| **Verification** | GET the API endpoint. 200 with valid JSON = confirmed. 404 = slug does not exist. |
| **Response shape** | `{"jobs": [...]}` |
| **Zero jobs** | Returns `{"jobs": []}` — valid slug, no current openings. |

**How to spot on a careers page:**
- Links containing `jobs.ashbyhq.com`
- Embedded job boards loading from `ashbyhq.com`
- Apply URLs containing `ashbyhq.com`

**Gotchas:**
- Ashby slugs sometimes differ from the company name in non-obvious ways.
- Ashby is newer, so companies migrating to Ashby from another ATS may have residual portals elsewhere.

---

### Workable

Common across a wide range of company sizes.

| Detail | Value |
|--------|-------|
| **Job board URL** | `https://apply.workable.com/{slug}` |
| **JSON API** | `https://apply.workable.com/api/v1/widget/accounts/{slug}` |
| **Verification** | GET the API endpoint. 200 with valid JSON = confirmed. 404 = slug does not exist. |
| **Response shape** | `{"results": [...], "total": N}` |
| **Zero jobs** | Returns `{"results": [], "total": 0}` — valid slug, no current openings. |

**How to spot on a careers page:**
- Links containing `apply.workable.com`
- Embedded widgets from `workable.com`
- Apply buttons pointing to `apply.workable.com/{slug}/j/{job-id}`

**Gotchas:**
- Workable slugs are usually straightforward lowercase company names.

---

### SmartRecruiters

Used by larger companies. Has a critical verification caveat.

| Detail | Value |
|--------|-------|
| **Job board URL** | `https://jobs.smartrecruiters.com/{slug}` |
| **JSON API** | `https://api.smartrecruiters.com/v1/companies/{slug}/postings` |
| **Verification** | GET the API endpoint. **200 is NOT sufficient** — check `totalFound` in the response. Only confirmed if `totalFound > 0`. |
| **Response shape** | `{"totalFound": N, "offset": 0, "limit": 100, "content": [...]}` |
| **Zero jobs** | Returns `{"totalFound": 0, "content": []}` — **but this also happens for completely fabricated slugs.** |

**How to spot on a careers page:**
- Links containing `jobs.smartrecruiters.com`
- Embedded job widgets from `smartrecruiters.com`
- Career page iframes sourcing SmartRecruiters

**Gotchas — this is the most dangerous provider for false positives:**
- The API returns HTTP 200 with `{"totalFound": 0, "content": []}` for ANY slug, including completely made-up strings like `asdfghjkl123`. A 200 response from SmartRecruiters proves nothing.
- The only way to confirm a company uses SmartRecruiters is `totalFound > 0` OR finding a SmartRecruiters link on the company's actual careers page.
- If the careers page links to SmartRecruiters but the API returns zero jobs, the company may have recently migrated away. Note the ambiguity and mark as bespoke if uncertain.

---

### Workday

Used by large enterprises. Complex discovery — the mechanical resolver rarely catches these.

| Detail | Value |
|--------|-------|
| **Job board URL** | `https://{company}.wd{N}.myworkdayjobs.com/en-US/{site}` |
| **JSON API** | `https://{company}.wd{N}.myworkdayjobs.com/wday/cxs/{company}/{site}/jobs` (POST) |
| **Verification** | POST to the API endpoint with `{"limit": 1, "offset": 0}`. 200 with job data = confirmed. |
| **Response shape** | `{"total": N, "jobPostings": [...]}` |

**Discovery challenge:**
- The subdomain number (`wd1` through `wd12`) varies per company and cannot be predicted.
- The `{site}` path segment varies per company (e.g. `External`, `en-US`, a custom name).
- Both values must be discovered from the company's careers page redirect chain or via web search.

**Storage:** Use `ats_slug` for the company identifier and `ats_extra` for the full endpoint path (including subdomain number and site name).

**How to spot on a careers page:**
- Redirects to `*.myworkdayjobs.com`
- Iframes loading from `*.myworkday.com`
- Links containing `myworkday` or `myworkdayjobs`

---

### Eightfold

Used by large companies, especially in finance and enterprise tech.

| Detail | Value |
|--------|-------|
| **Job board URL** | Varies — typically `https://{subdomain}/careers` |
| **JSON API** | `https://{subdomain}/api/apply/v2/jobs?domain={domain}` |
| **Verification** | GET the API endpoint. 200 with job data = confirmed. |

**Discovery challenge:**
- The subdomain is company-specific (e.g. `explore.jobs.netflix.net`).
- Must be discovered from the company's careers page.

**Storage:** Store the subdomain and domain in `ats_extra`.

**How to spot on a careers page:**
- URLs containing `eightfold.ai` in page source
- JavaScript loading from Eightfold CDN
- The careers page itself may be hosted on an Eightfold subdomain

---

## Unsupported Providers

When a company uses one of these providers, mark them as `bespoke` with the careers URL preserved. These providers either lack public APIs, require authentication, or have structures too complex for automated scraping.

| Provider | How to recognise | Notes |
|----------|-----------------|-------|
| **iCIMS** | URLs containing `icims.com`, `careers-{company}.icims.com`, or `jobs.icims.com` | Very common among large enterprises. Complex URL structure. |
| **Taleo** | URLs containing `taleo.net`, `{company}.taleo.net` | Oracle product, common in legacy enterprises. Being phased out but still widespread. |
| **BambooHR** | URLs containing `{company}.bamboohr.com/careers` | Common among small-to-mid companies. |
| **Jobvite** | URLs containing `jobs.jobvite.com`, `{company}.jobvite.com` | Mid-market ATS. |
| **Personio** | URLs containing `{company}.jobs.personio.de` or `{company}.jobs.personio.com` | European-focused ATS, common among German and UK companies. |
| **Pinpoint HQ** | URLs containing `{company}.pinpointhq.com` | UK-based ATS, growing in popularity. SurrealDB uses this. |
| **Recruitee** | URLs containing `{company}.recruitee.com` | European mid-market ATS. |
| **Breezy HR** | URLs containing `{company}.breezy.hr` | Small company ATS. |
| **Wellfound (AngelList)** | URLs containing `wellfound.com/company/{slug}/jobs` | Startup-focused. Has a public listing but no stable API for our use. |

When you encounter a provider not on either list, note it in the resolution summary. If multiple companies use the same unknown provider, it may be worth investigating support later.

---

## Identifying the ATS from a Careers Page

When visiting a company's careers page to determine their ATS, check these sources in order of reliability:

### 1. Outbound links (most reliable)

Look at where "Apply" or "View all jobs" buttons link to. The domain in the link URL directly identifies the ATS:

```
boards.greenhouse.io/company     → Greenhouse
jobs.lever.co/company            → Lever
jobs.ashbyhq.com/company         → Ashby
apply.workable.com/company       → Workable
jobs.smartrecruiters.com/company → SmartRecruiters
*.myworkdayjobs.com/*            → Workday
```

### 2. Page source and network requests

If links are not visible (JavaScript-rendered pages), inspect the page source for:
- Script tags loading from ATS CDNs (`greenhouse.io`, `lever.co`, `ashbyhq.com`, etc.)
- API calls in JavaScript to ATS endpoints
- Iframe `src` attributes pointing to ATS domains
- Meta tags or data attributes referencing ATS providers

### 3. Redirect chains

Some careers pages redirect through one or more URLs before landing on the ATS board. The final URL in the redirect chain reveals the ATS.

### 4. Embedded job listings

Some companies embed ATS job listings directly into their own domain. The embed code typically references the ATS provider in script sources or API calls, even when the visible URL is the company's own domain.

---

## Common Slug Patterns

When the careers page reveals an ATS URL, the slug is the path segment identifying the company. These patterns explain the non-obvious slugs that defeat mechanical resolution:

| Pattern | Example | Why it happens |
|---------|---------|---------------|
| **Legal entity name** | XTX Markets → `xtxmarketstechnologies` | The ATS account was created with the legal entity name, not the brand |
| **Former name** | Wise → `transferwise`, Meta → `facebook` | The ATS account predates the rebrand |
| **With suffix** | Cleo → `cleoai`, Thought Machine → `thoughtmachine` | Disambiguation or the brand name was taken |
| **Parent company** | Subsidiary listed under parent's slug | The parent manages all hiring centrally |
| **Domain name** | company.io → `companyio` | Registered with the domain rather than the brand name |
| **Hyphenated vs joined** | Jane Street → `janestreet` or `jane-street` | Inconsistent conventions across providers |
| **Abbreviated** | International Business Machines → `ibm` | Common abbreviation used instead of full name |

The mechanical resolver tries the obvious variations. If you are in this skill, those already failed — extract the slug directly from the careers page URL rather than guessing further.
