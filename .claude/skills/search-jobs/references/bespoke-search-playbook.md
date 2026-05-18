# Bespoke Search Playbook

> How to find open roles at Cernio's bespoke companies — the companies whose ATS is unsupported by the Rust script and whose jobs are otherwise invisible to the automated pipeline. Bespoke companies include some of the strongest-signal employers in the universe (Apple, Google, Meta, Amazon, Microsoft, Citadel, Bloomberg, Arm, Mastercard, and others); skipping them means silently missing the best opportunities.

---

## Table of Contents

1. [Core rule — bespoke results are inserted into the DB, not just reported](#core-rule--bespoke-results-are-inserted-into-the-db-not-just-reported)
2. [Which companies count as bespoke](#which-companies-count-as-bespoke)
3. [Where to look for bespoke roles](#where-to-look-for-bespoke-roles)
4. [Per-company search procedure](#per-company-search-procedure)
5. [What subagents should return](#what-subagents-should-return)
6. [What the orchestrator does with the results](#what-the-orchestrator-does-with-the-results)
7. [Known limitations](#known-limitations)

---

## Core rule — bespoke results are inserted into the DB, not just reported

Every bespoke match must end up as a row in the `jobs` table. Conversational reports like *"I found 13 promising roles at bespoke companies"* without accompanying `INSERT OR IGNORE INTO jobs` statements leave those jobs invisible to the TUI, the grading pipeline, and the application tracker.

Session-7 failure mode (documented in `context/notes/bespoke-search-workflow.md`): bespoke search produced ~13 strong leads at Apple, Citadel, Bloomberg, Google, Arm, Mastercard, and Amazon — and the agent only reported them in chat. The jobs were never graded and never appeared in the TUI until a later session manually inserted them. The insert step is the workflow; the report is a consequence.

Minimum fields per insert: `company_id`, `title`, `url`, `location`, `raw_description`, `evaluation_status = 'pending'`, `discovered_at = datetime('now')`. **`raw_description` is mandatory now** — bespoke subagents must fetch the JD via `WebFetch` on the role's URL before reporting it, and the orchestrator inserts the description text alongside the other fields. The old "grade-jobs will fetch it later" pattern produced 100% missing-description coverage on bespoke roles (audit: 31/31 bespoke rows had empty JDs as of 2026-05-18, including every SS-grade FAANG row), which forced grade-jobs into brand-only reasoning.

```sql
INSERT OR IGNORE INTO jobs
    (company_id, title, url, location, raw_description, evaluation_status, discovered_at)
VALUES (?, ?, ?, ?, ?, 'pending', datetime('now'));
```

If the JD fetch fails (login wall, JavaScript-only page, 404), pass NULL for `raw_description` and note the failure in the subagent's per-role section. grade-jobs' semantic-reasoning path handles the NULL case — but the default must be to fetch.

The `OR IGNORE` clause relies on the `url` unique constraint — re-running bespoke search on the same company does not produce duplicates.

---

## Which companies count as bespoke

```sql
SELECT id, name, careers_url, grade
FROM companies
WHERE status = 'bespoke'
  AND grade IN ('S', 'A', 'B')
ORDER BY CASE grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 END;
```

(`status` is a single enum — `'bespoke'` and `'archived'` are mutually exclusive values per the `companies` table CHECK constraint, so a separate `status != 'archived'` clause would be dead code.)

Priority: S-tier first (non-negotiable — these are the highest-signal employers in the universe). A-tier second (strong signal, worth the time). B-tier when time permits (lower yield per minute than the script-driven pipeline). C-tier bespoke companies are generally not worth manual search unless the user specifically requests them.

A company is in this list because its ATS provider is unsupported (iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite, Recruitee, Breezy HR, Wellfound) or because it runs a custom career portal (most FAANG, most HFT, some public companies). The `careers_url` field points to the company's job listings page — sometimes useful directly, sometimes behind JavaScript rendering that agents cannot follow.

---

## Where to look for bespoke roles

| Source | When to use | Limitations |
|---|---|---|
| Company careers page (`careers_url` from DB) | First attempt for every bespoke company | Some pages load jobs via JavaScript that cannot be rendered from a WebFetch; redirects through SSO can block access |
| LinkedIn Jobs (`linkedin.com/jobs`) | Major companies with formal recruiting presence | Requires login for full detail; title + location usually accessible without |
| Indeed (`indeed.co.uk`) | UK-specific searches, mid-to-large companies | Aggregates from many sources; occasionally stale |
| Glassdoor (`glassdoor.co.uk`) | Tech companies with strong employer brand | Login wall for deep detail |
| BuiltIn (`builtin.com`) | Tech-sector aggregator | Limited UK coverage; best for US companies with London offices |
| Wellfound (formerly AngelList) | Startups, small companies | Small company bias; fewer large-employer listings |
| Sector-specific job boards | When the company is in a known vertical (e.g. efinancialcareers for HFT, academic mailing lists for research-adjacent companies) | Coverage varies sharply by sector |
| Google / DuckDuckGo site-specific search | `"site:apple.com/careers" engineer london` as a fallback | Ranking bias toward popular roles — specific relevant roles may not be top results |

For major companies whose careers pages are JavaScript-heavy (Meta, Microsoft, some Reactive Markets pages), the aggregator fallback route is often the only workable path.

---

## Per-company search procedure

For each bespoke company in priority order:

1. **Visit the careers page** via `WebFetch` on the stored `careers_url`.
2. **If the page returns job listings:** identify roles matching the profile's target titles (Engineer, Infrastructure, Systems, Platform, ML, Graduate, New Grad, etc.), extract the title / URL / location for each, skip any role that is clearly senior-only (Staff, Principal, Director, VP, etc. — `grade-jobs` does the final seniority call from the description, but obvious senior-only titles do not need inserting).
3. **For each matching role, fetch the full JD.** Run `WebFetch` on the role's URL and capture the description text (responsibilities, qualifications, stack, location, sponsorship language). This is the JD that grade-jobs will reason against. If the URL is a stable job-detail page, the WebFetch returns the role's full content; that text becomes the `raw_description` field on insert. If the JD is split across multiple paragraphs / sections, concatenate them with newlines. Do not summarise — pass the raw text through.
4. **If the JD fetch fails** (login wall, JavaScript-only, 404, redirect to a generic listings page): record the URL fetch failure for this role, mark `raw_description` as unavailable in the per-role section, and continue. The role still gets inserted with NULL description — grade-jobs' semantic-reasoning path handles brand-strong rows without a JD.
5. **If the careers page is empty, JavaScript-walled, or behind SSO:** fall through to aggregators. `WebSearch` for `"{company name}" careers london engineer`, `"{company name}" graduate`, `"{company name}" infrastructure engineer london`, tuning the query terms to the profile's target areas. Try LinkedIn Jobs URLs first (`site:linkedin.com/jobs {company}`). When you land on a role via aggregator, repeat step 3 — fetch the canonical careers-page URL (not the aggregator listing) to get the real JD.
6. **For FAANG / major public companies** specifically: visit their careers site and use the in-page search. Query for roles matching profile target terms. Apple's careers site, Google Careers, Microsoft Careers, Meta Careers, Amazon Jobs all have functional search. Apply step 3 to every matching role — JD-fetch is non-negotiable even when the listing page renders fine.
7. **If no relevant roles exist or none can be found:** record that this company was searched and returned nothing. Do not insert — bespoke companies with no current openings are a valid zero-result.

Depth over breadth per company. Two minutes spent end-to-end on Citadel's careers page beats skim-reading five companies.

---

## What subagents should return

If you dispatch subagents to parallelise bespoke search (one agent per ~3-5 bespoke companies), each subagent's output is structured per-company findings with the full JD per role embedded under a fenced block:

````
## [Company Name] (bespoke, grade: S)

Found 3 matching roles:

### Role 1
- Title: Software Engineer, New Grad — Systems Engineering
- URL: https://jobs.apple.com/en-gb/details/200503201/software-engineer-new-grad-systems-engineering
- Location: London, UK
- JD fetched: yes (4,820 chars)
```jd
[Full job description text — responsibilities, qualifications, stack, location, sponsorship language — pasted verbatim, no summarisation]
```

### Role 2
- Title: Compiler Engineer
- URL: https://jobs.apple.com/en-gb/details/200498214/compiler-engineer
- Location: London, UK
- JD fetched: yes (3,140 chars)
```jd
[Full JD text]
```

### Role 3
- Title: Infrastructure Software Engineer
- URL: https://www.linkedin.com/jobs/view/3851234567
- Location: London, UK (Hybrid)
- JD fetched: no — LinkedIn login wall

Searched: careers.apple.com + LinkedIn site:linkedin.com/jobs apple
Not found / no relevant openings for: N/A
````

The orchestrator parses out the `jd` blocks and passes their contents through to the `raw_description` INSERT field. Roles with `JD fetched: no` are inserted with NULL description and rely on grade-jobs' semantic-reasoning path.

For a company with no hits:

```
## [Company Name] (bespoke, grade: A)

Found 0 matching roles.
Searched: careers page (empty), LinkedIn (no UK engineer roles), Indeed (only US roles).
```

Agents do not write SQL — the orchestrator handles DB writes after collecting all agent results. Agents do not grade — `grade-jobs` handles grading after insert.

---

## What the orchestrator does with the results

After subagent results arrive:

1. **Map company names to `company_id`** via `SELECT id, name FROM companies WHERE status = 'bespoke';`. If a subagent returned a name that does not resolve, flag it — the company may not actually be in the DB as bespoke.
2. **Produce `INSERT OR IGNORE INTO jobs` statements** for every returned role, with minimum fields (`company_id`, `title`, `url`, `location`, `raw_description`, `evaluation_status = 'pending'`, `discovered_at = datetime('now')`). The `raw_description` comes from the per-role `jd` fenced block in the subagent output — pass NULL when the subagent reported `JD fetched: no`. The URL unique constraint blocks duplicates silently.
3. **Execute the inserts** against `state/cernio.db`. Report the insert count (vs how many were pre-existing duplicates skipped by the OR IGNORE).
4. **Queue the new bespoke jobs for grading** — they will appear in `grade-jobs`'s pending queue on its next run.
5. **Report to the user:**
   - Companies searched (S / A / B counts)
   - Jobs found and inserted
   - Jobs that were already in the DB (URL already present)
   - Companies with zero matches — not a failure, just a zero-result record for this session

---

## Known limitations

- Agents can only `WebSearch` / `WebFetch` public pages — they cannot log into career portals, bypass SSO, or render JavaScript.
- Some major employers (Meta, Microsoft, some trading firms like Reactive Markets) load jobs via client-side JavaScript that `WebFetch` cannot render. For those, the aggregator fallback (LinkedIn / Indeed) is the only workable route.
- Bespoke search results are a lower bound, not exhaustive. Manual portal visits by the user may surface roles the agent missed.
- `raw_description` IS populated at bespoke-insert time when the JD fetch succeeds (the new contract above). When the fetch fails — login wall, JavaScript-only listing, redirect to generic page — `raw_description` is left NULL and grade-jobs falls through to its semantic-reasoning path (which can still grade brand-strong rows like Google grad programmes from company + title context alone).
- LinkedIn rate-limits heavy WebFetch from single IPs — batch-searching 50+ companies in one session risks throttling. Keep per-session load proportionate.
