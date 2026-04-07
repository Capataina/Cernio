# populate-db

Takes companies from the discovery stage, researches each one to find their job board and ATS system, verifies the connection works, and inserts them into the database as resolved entries. Use when the user says "populate the database", "add these companies", "research these companies", "find their job boards", or after a discovery run has produced companies in `companies/potential.md` that need processing. Not for discovering new companies (that's the discovery skill) or searching for jobs (that's the search scripts).

---

## Why this skill exists

Discovery produces a list of promising companies, but a company name and website alone can't be scraped for jobs. To actually search for open roles, we need to know: does this company use Greenhouse, Ashby, Lever, Workable, or something else? What's the specific slug or URL to query? Does the endpoint actually return live job data? Population bridges the gap between "we know this company exists" and "we can programmatically find their open roles."

Companies that use a supported ATS become `resolved` in the database — ready for job scraping. Companies on custom portals, Workday, or other unsupported systems become `bespoke` — still tracked with a direct careers page link, but not scrapeable through our scripts.

---

## How population works

### 1. Read the input

Population works from `companies/potential.md` — the output of the discovery skill. Each entry has a company name, website, and metadata. The skill processes these entries and moves them into the SQLite database.

Before processing, check the database for existing entries with the same website URL to avoid duplicates.

### 2. For each company, find the job board

This is the core research step. The goal is to find which ATS the company uses and the specific slug needed to query their public API.

**Try deterministic patterns first.** Most ATS providers use predictable URL patterns. From the company name, generate slug candidates (lowercase, hyphenated, common variations) and probe:

- `https://boards.greenhouse.io/{slug}` — Greenhouse
- `https://job-boards.greenhouse.io/{slug}` — Greenhouse (alternate)
- `https://jobs.ashbyhq.com/{slug}` — Ashby
- `https://jobs.lever.co/{slug}` — Lever
- `https://apply.workable.com/{slug}` — Workable

A slug that returns HTTP 200 with valid JSON containing at least one job is a confirmed match. Slugs don't always match the company name exactly — Wise is `transferwise` on Greenhouse, Cleo is `cleoai`, DeepMind is `deepmind` not `googledeepmind`. Try obvious variations: the full name, abbreviations, former names, domain name without TLD.

**If deterministic probing fails, search the web.** Search for `"{company name}" careers` or `"{company name}" greenhouse` or `"{company name}" jobs site:greenhouse.io`. The results usually reveal the correct ATS and slug. This is where Claude's judgment comes in — interpreting search results, recognising the right link, and verifying it.

**If no supported ATS is found, mark as bespoke.** Find the company's careers page URL directly and record it. The company is still worth tracking — their jobs just can't be scraped automatically. Common bespoke systems: Workday, iCIMS, Taleo, SmartRecruiters, custom-built portals.

### 3. Verify the connection

For companies matched to a supported ATS:
- Fetch the JSON endpoint and confirm it returns parseable job data
- Record the ATS provider, slug, and verification date

A slug that returns 200 but has zero jobs might be valid but the company might not be actively hiring. Still mark it as resolved — the slug works, jobs may appear later.

### 4. Insert into the database

Write the company to the SQLite database with all available fields:
- Facts: name, website, what they do, discovery source, discovery date
- Checkpoints: ATS provider, slug, verification date, status (resolved or bespoke), careers URL for bespoke
- Judgments: why relevant (carried from discovery), relevance updated date
- Location and sector tags if known

### 5. Update the TUI

As each company is processed, the TUI should reflect the progress. The flow the user sees:

```
Company in potential.md
  → "researching..."     (Claude is probing ATS patterns)
  → "resolved: greenhouse/wise-transferwise"   (found and verified)
  → or "bespoke: workday portal"               (no supported ATS)
```

### 6. Clean up potential.md

Once a company has been processed and inserted into the database (whether as resolved or bespoke), remove it from `companies/potential.md`. After a full population run, `potential.md` should only contain companies that haven't been processed yet.

---

## Slug resolution tips

Company names don't always map cleanly to ATS slugs. Common patterns:

- **Former names:** Wise → `transferwise`, Meta → `facebook`
- **Abbreviated names:** International Business Machines → `ibm`
- **Suffixes stripped:** "Acme Ltd" → `acme`, "FooBar Inc." → `foobar`
- **Spaces to hyphens:** "Jane Street" → `jane-street` or `janestreet`
- **AI/tech suffixes:** Cleo → `cleoai`, Thought Machine → `thoughtmachine`
- **Parent companies:** A subsidiary might be under the parent's slug

When the obvious slugs fail, the company's own website often links to their careers page, which reveals the ATS URL in the link target. Check the footer, "Careers" or "Jobs" navigation links, and the page source.

---

## Batch processing and parallelisation

When processing many companies at once, the deterministic slug probing can be parallelised — it's just HTTP requests. The web search fallback is slower and requires Claude's judgment, so it runs sequentially for each company that fails deterministic resolution.

A reasonable approach for a batch of 30 companies:
1. Run deterministic slug probing for all 30 in parallel (fast, scriptable)
2. Collect the ones that didn't match
3. Research the unmatched ones individually with web search (Claude judgment)

The Rust `resolve` script handles step 1. Claude handles step 3.

---

## Quality checklist

Before marking a company as resolved, verify:

- [ ] The ATS slug returns valid JSON with a parseable job listing structure (not a 404, not an HTML page, not an empty response)
- [ ] The slug is for the right company (not a different company with a similar name)
- [ ] The company hasn't already been inserted into the database (check website URL)
- [ ] Bespoke companies have a working careers page URL, not just a homepage
- [ ] The entry in the database has all available fields populated — don't leave location or sector tags empty if they were known from discovery
