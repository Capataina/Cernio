# populate-db

Takes companies from the discovery stage, validates them, and uses the Rust pipeline to resolve their ATS portals — then handles AI fallback for companies the script cannot match. Use when the user says "populate the database", "add these companies", "research these companies", or after a discovery run has produced companies in `companies/potential.md` that need processing. Not for discovering new companies (that's discover-companies), not for grading companies (that's grade-companies), not for searching jobs (that's search-jobs).

---

## Mandatory reads — do not proceed without completing these

**STOP. Before populating any companies, you MUST read these files in full:**

1. **Every file in `profile/`** — all files, no exceptions. You need the user's background, skills, and career targets to assess whether a company is worth tracking.
2. **`skills/resolve-portals/references/ats-providers.md`** — the complete ATS provider reference. Even though the mechanical resolver handles most resolution, you need to understand ATS URL patterns for the AI fallback phase and to verify results correctly. This file is in a different skill's directory — read it from `skills/resolve-portals/references/ats-providers.md`.

**When delegating to subagents:** embed the full ATS provider reference in each agent's prompt.

**Do not begin processing any companies until all mandatory reads are complete.**

---

## Why this skill exists

Discovery produces a list of promising companies, but a company name and website alone cannot be scraped for jobs. To actually search for open roles, we need to know: does this company use Greenhouse, Ashby, Lever, Workable, or something else? What is the specific slug or URL to query? Population bridges the gap between "we know this company exists" and "we can programmatically find their open roles."

This skill is the **orchestration layer**. The mechanical volume work — probing slug patterns against every supported ATS — is handled by the `cernio resolve` Rust script. This skill provides the judgment that the script cannot: validating companies, handling failed resolutions via web search, and presenting results for human review.

---

## Before you start

**Read all `profile/` files.** You need the user's background, skills, career targets, and preferences to assess whether a company is worth tracking. This is non-negotiable — do not skip it.

---

## How population works

### 1. Read the input

Population works from `companies/potential.md` — the output of the discovery skill. Each entry has a company name, website, and metadata. The user may also provide companies directly in conversation.

Before processing, check the database for existing entries with the same website URL to avoid duplicates.

### 2. Validate companies are real and worth tracking

Before spending time on ATS resolution, verify each company is a viable target. This is where Claude's judgment matters — the script cannot do this.

**Check the basics:**
- Does the website load? A dead website means a dead company or a pivot — remove from potential.md and move on.
- Is there evidence of current activity? Recent blog posts, press releases, job listings, social media activity, or product updates within the last 6–12 months.
- Is the company still independent? It may have been acquired (check for redirects to a parent company), shut down, or rebranded.

**Assess engineering fit:**
- Do they have an engineering team? A 5-person company with no engineers is not going to hire a systems engineer.
- Do they do technical work that aligns with the profile? A fintech that only does marketing automation is not relevant even if it is in "fintech."
- Is there any signal they are hiring or growing? Active careers page, job listings, headcount growth, recent funding.

**If the company fails validation:** Remove it from `companies/potential.md` with a brief note about why (dead website, acquired, too small, not relevant). Do not insert it into the database — the database is for companies worth tracking.

**If the company passes:** Proceed to step 3 (ATS resolution). Do not grade companies during this step — grading is a separate pass.

### 3. Run `cernio resolve` for mechanical ATS probing

This is the handoff to the Rust pipeline. The script probes predictable slug patterns against every supported ATS provider — it handles the volume work that does not require judgment.

**CLI reference:**

| Command | What it does |
|---------|-------------|
| `cernio resolve --dry-run` | Preview which companies will be probed, without making HTTP requests. Use to verify the input looks right. |
| `cernio resolve` | Execute resolution for all unresolved companies in the database. Probes slug variants against Greenhouse, Lever, Ashby, Workable, SmartRecruiters. |
| `cernio resolve --company "Palantir"` | Resolve a single company by name. Useful for re-trying one company or processing an ad-hoc addition. |

**Workflow:**
1. Run `cernio resolve --dry-run` first to preview the batch
2. Review the preview — confirm the company list looks correct
3. Run `cernio resolve` to execute
4. Review the output — note which companies resolved and which failed

### 4. Review the resolve output

The script reports three outcomes per company:

| Outcome | Meaning | Next step |
|---------|---------|-----------|
| **Resolved** | Slug found and verified on a supported ATS | Done — the company is ready for job searching |
| **Failed** | No slug matched on any supported ATS | AI fallback (step 5) |
| **Error** | HTTP error, timeout, or other failure | Re-try or AI fallback |

Present the resolve results to the user. For resolved companies, show the ATS provider and slug. For failed companies, explain they need AI fallback.

### 5. AI fallback for unresolved companies

Companies that failed mechanical resolution need human-AI judgment. These typically fall into three buckets:

1. **Non-obvious slugs on supported ATS** — The company uses Greenhouse/Lever/Ashby/etc., but their slug is unexpected (a former name, a legal entity, an abbreviation). The careers page reveals the correct URL.
2. **Unsupported ATS providers** — iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, or custom portals. These become `bespoke` with the careers URL preserved.
3. **Dead or disappeared companies** — No website, no careers page, Companies House shows dissolved. Skip these entirely.

**Fallback process:**
- Use WebSearch for `"{company name}" careers` or `"{company name}" jobs`
- Visit the careers page and look for ATS URLs in links, iframes, or redirects
- Extract the slug and verify against the provider's JSON API
- If no supported ATS is found, record as `bespoke` with the careers URL
- If the company appears dead, report findings with evidence

### 6. Present results for user review

Before committing anything, present a summary table of all results:

```
| Company        | Result   | ATS        | Slug                     | Source                                |
|----------------|----------|------------|--------------------------|---------------------------------------|
| Cloudflare     | resolved | greenhouse | cloudflare               | cernio resolve (mechanical)           |
| XTX Markets    | resolved | greenhouse | xtxmarketstechnologies   | AI fallback — careers page link       |
| SurrealDB      | bespoke  | pinpoint   | —                        | surrealdb.pinpointhq.com (unsupported)|
| Vypercore      | removed  | —          | —                        | Companies House: in liquidation       |
```

Wait for user approval before writing to the database. This is a review gate — the user can correct mistakes, skip companies, or ask for re-research.

### 7. Clean up potential.md

Once companies have been processed and inserted into the database (whether as resolved or bespoke), remove them from `companies/potential.md`. After a full population run, `potential.md` should only contain companies that have not been processed yet.

---

## Quality checklist

Before presenting results, verify:

- [ ] Every company was validated for activity, independence, and engineering relevance before resolution
- [ ] The `cernio resolve` script was run before attempting manual resolution (no duplicating the script's work)
- [ ] Companies that failed mechanical resolution were researched via AI fallback, not just abandoned
- [ ] Resolved portals return valid JSON from the provider's API (or confirmed zero jobs on a provider that does not give false positives)
- [ ] SmartRecruiters results have `totalFound > 0` — a zero-result 200 is not evidence of usage
- [ ] The slug belongs to the correct company, not a different company with a similar name
- [ ] No duplicate entries — checked existing database entries before inserting
- [ ] Bespoke companies have a working careers page URL, not just a homepage
- [ ] Results are presented in a summary table for user review before any database writes
- [ ] Dead companies have evidence cited (Companies House status, 404 website, etc.)
