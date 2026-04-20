---
name: resolve-portals
description: "AI fallback for ATS portal resolution — handles the companies that the mechanical `cernio resolve` Rust script could not match. Uses WebSearch + careers-page inspection to extract the correct ATS slug (which the script missed because the slug is non-obvious: legal entity name, former name, abbreviation, domain-based) or to identify an unsupported ATS provider (iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, Recruitee) and mark the company as bespoke with the careers URL preserved. Verifies every supposed slug against the provider's JSON API before writing. Invoke when the user says 'resolve portals', 'find ATS for remaining companies', 'handle unresolved companies', 'resolve the unmatched ones', 'find their job boards', 'the resolver failed on these', or names specific companies that stayed in `potential` status after `cernio resolve` ran. Not for running the initial mechanical probe (that is `cernio resolve` via populate-db), discovering new companies (use discover-companies), grading (use grade-companies), or searching jobs on already-resolved companies (use search-jobs). Use this skill whenever companies remain in `potential` status after the script has run, even if the user does not name it explicitly."
---

# Resolve Portals

AI fallback for companies the mechanical resolver could not match. Every company reaching this skill has already had its obvious slug variants probed — `{company-name}`, `{domain-without-tld}`, `{hyphenated-name}` — against every supported ATS. None returned valid results. The remaining cases need judgment: find the real careers page, inspect it for the actual ATS link, and verify the slug against the provider's JSON API.

The skill's single output is a lifecycle transition: every target company moves from `potential` to either `resolved` (ATS found and verified) or `bespoke` (no supported ATS; careers URL preserved). Dead companies are surfaced with evidence and left for the user to decide.

---

## Mandatory Read Before Resolving Any Company

| # | What | Evidence it was read |
|---|---|---|
| 1 | **`references/ats-providers.md`** (the full file) | You can cite the exact JSON API endpoint for each of the 7 supported providers and name the SmartRecruiters false-positive rule (`totalFound > 0` required) |

Without the reference, the agent cannot verify slugs correctly, misses the SmartRecruiters false-positive trap, and misclassifies unsupported providers. A resolution session without this read produces silently broken DB entries. When delegating work to subagents, the full reference text is embedded verbatim in each agent's prompt — agents cannot read the skill's references themselves.

---

## Context

The companies reaching this skill fall into three buckets. Categorisation happens during inspection, not upfront:

1. **Non-obvious slug on a supported ATS.** The company uses Greenhouse, Lever, Ashby, etc., but the script's candidate set missed the real slug. Canonical examples: XTX Markets → `xtxmarketstechnologies` (legal entity), Wise → `transferwise` (former name), Cleo → `cleoai` (suffix). The careers page outbound "Apply" link reveals the correct slug directly — no further guessing.
2. **Unsupported ATS provider.** iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, Recruitee, Breezy HR, Wellfound. These become `bespoke` with the careers URL preserved. Recognition markers for each are in `references/ats-providers.md`.
3. **Dead or dissolved company.** No working website, no careers page, Companies House shows dissolved or in liquidation. Report with evidence and stop; do not insert anything.

---

## Workflow

### 1. Query unresolved companies

```sql
SELECT id, name, website, careers_url
FROM companies
WHERE status = 'potential';
```

Present the list to the user. For large batches (more than ~10), propose processing in batches so the review gate stays tractable.

### 2. For each company, find the careers page

The mechanical resolver already exhausted slug guessing. Guessing further is wasted effort. Go to the source:

- `WebSearch` for `"{company name}" careers` or `"{company name}" jobs`
- Visit the company's website; find the Careers / Jobs link in navigation or footer
- Inspect the careers page for ATS markers: outbound links, iframe sources, embedded JavaScript widgets, redirect chains

ATS identification signals per provider (full playbook in `references/ats-providers.md`):

- `boards.greenhouse.io` or `job-boards.greenhouse.io` → Greenhouse
- `jobs.lever.co` → Lever
- `jobs.ashbyhq.com` → Ashby
- `apply.workable.com` → Workable
- `jobs.smartrecruiters.com` → SmartRecruiters
- `*.myworkdayjobs.com` → Workday (stores subdomain + site name in `ats_extra`)
- `eightfold.ai` in page source → Eightfold (stores subdomain in `ats_extra`)

### 3. Verify the portal against the provider's JSON API

Finding an ATS URL on the careers page is evidence; confirming the slug returns valid job data is verification. Extract the slug, hit the provider's JSON API endpoint (exact endpoints in the reference file), and confirm a parseable response:

- **Greenhouse / Lever / Ashby / Workable:** 200 with valid JSON = confirmed; 404 = slug does not exist; valid response with zero jobs = valid slug with no current openings (still confirmed).
- **SmartRecruiters:** 200 is **not** sufficient. The API returns `{"totalFound": 0, "content": []}` for fabricated slugs. Confirmation requires `totalFound > 0` OR a SmartRecruiters link on the company's actual careers page combined with a response that parses cleanly. When ambiguous, mark as bespoke and flag the uncertainty.
- **Workday:** POST `{"limit": 1, "offset": 0}` to the API endpoint. 200 with `jobPostings` = confirmed.

A valid portal with zero current jobs is still a valid resolution — the company may simply not be hiring right now and the slug will light up later.

### 4. Handle the result

**Supported ATS found:**

```sql
INSERT INTO company_portals (company_id, ats_provider, ats_slug, ats_extra, verified_at, is_primary)
VALUES (?, ?, ?, ?, date('now'), 1);

UPDATE companies SET status = 'resolved' WHERE id = ?;
```

If multiple portals are found for one company (active Ashby + residual Workable after a migration), insert both. Mark the active one `is_primary = 1` and the residual one `is_primary = 0`. Multi-portal entries are expected, not errors.

**Unsupported ATS or custom portal:**

```sql
UPDATE companies
SET status = 'bespoke', careers_url = ?
WHERE id = ?;
```

The careers URL points to the actual job listings page, not just the homepage — downstream bespoke search needs a page that shows jobs.

**Dead or dissolved company:**

Do not insert. Report to the user with specific evidence — Companies House status, HTTP 404 on the website, redirect to a parent company, no activity in the last 12 months. The user decides whether to delete the DB entry or leave it for future retry.

### 5. Present the summary table before any DB write

```
| Company        | Result   | ATS        | Slug                     | Evidence                              |
|----------------|----------|------------|--------------------------|---------------------------------------|
| XTX Markets    | resolved | greenhouse | xtxmarketstechnologies   | Found via careers page outbound link  |
| SurrealDB      | bespoke  | pinpoint   | —                        | surrealdb.pinpointhq.com (unsupported)|
| Vypercore      | dead     | —          | —                        | Companies House: in liquidation 2025-08|
```

Wait for user approval before executing any SQL. This is a review gate — the user catches misclassified providers, wrong slugs, or premature "dead" calls.

---

## Lessons From Production

These are failure modes observed in real resolution runs, not theoretical concerns:

- **Slug guessing keeps failing; the careers page keeps being the answer.** Every non-obvious slug seen in production (XTX, Wise, Cleo, Thought Machine) was trivially extractable from the careers page outbound link. The careers page is the authoritative source.
- **SmartRecruiters returns 200 for fabricated slugs.** `totalFound > 0` is the only trustworthy signal. Without this check, the DB accumulates false-positive SmartRecruiters entries that yield zero jobs forever.
- **Companies migrate ATS providers and leave residues.** ClearBank has an active Ashby board and a zero-job residual Workable board. The schema supports `is_primary` on `company_portals` precisely for this — record both, flag the active one.
- **Companies House is the fastest death verifier for UK companies.** When a website is dead, Companies House resolves "dissolved / in liquidation / active" in seconds and prevents hours of searching for a ghost.
- **Unsupported ATS providers are common, not exotic.** Pinpoint HQ, Personio, iCIMS, Taleo, BambooHR, Jobvite all exist in the current company universe. Bespoke with careers URL preserves the information for later manual handling or future ATS support.

---

## Subagent Context Requirements

When dispatching parallel resolution work (one subagent per 3–5 unresolved companies), each prompt includes:

- The **full text of `references/ats-providers.md`** — verbatim. Subagents cannot read it.
- The **list of assigned companies** with their existing `website` and `careers_url` fields.
- Explicit instruction to produce the summary-table rows (company / result / ATS / slug / evidence) directly, not a narrative.
- The SmartRecruiters `totalFound > 0` rule stated inline, not only linked — tool-use obligations degrade when buried in references.

Under-contextualised subagents misclassify providers. Over-share.

---

## Reference Loading

**Mandatory-core — read at skill invocation:**

- `references/ats-providers.md` — complete reference: 7 supported providers with API endpoints and verification rules, 9+ unsupported providers with recognition markers, careers-page identification playbook, common slug patterns that defeat mechanical resolution.

---

## Inviolable Rules

1. **Every resolved portal is verified against the provider's JSON API before DB write.** Raw careers-page evidence is necessary but not sufficient.
2. **SmartRecruiters requires `totalFound > 0`.** A 200 with zero totalFound is ambiguous and defaults to bespoke when the careers page itself does not link to SmartRecruiters.
3. **No DB write without user approval of the summary table.** The review gate catches mislabelled providers and wrong-slug assignments before they reach production.
4. **Bespoke entries point to a working job listings page.** Not a homepage. Downstream bespoke search depends on this.
5. **Dead-company claims carry concrete evidence.** Companies House status, HTTP 404, redirect-to-parent. No unsubstantiated "seems dead" calls.

---

## Quality Checklist

- [ ] Every resolved portal verified against the provider's JSON API with parseable response data
- [ ] All SmartRecruiters entries have `totalFound > 0` or are backed by a careers-page link plus noted ambiguity — no zero-totalFound 200s treated as confirmation
- [ ] Each slug verifiably belongs to the correct company, not a similarly-named different entity
- [ ] No duplicate `company_portals` rows — existing entries checked before insert
- [ ] Bespoke entries point to an actual job listings page, not the company homepage
- [ ] Dead-company findings cite specific evidence (Companies House status, HTTP response, redirect chain)
- [ ] When multiple portals exist for one company, `is_primary` is set correctly (active = 1, residual = 0)
- [ ] Summary table was presented and user-approved before any SQL executed
