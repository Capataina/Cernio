# search-jobs

Searches for open positions at companies in the database, triages them intelligently, fetches full descriptions for promising candidates, and evaluates fit against the user's profile. Use when the user says "search for jobs", "check Spotify's openings", "what's available at [company]", "find me roles", or when it's time to scan resolved companies for new opportunities. Not for discovering companies (that's discover-companies) or resolving ATS slugs (that's populate-db).

---

## Why this skill exists

The company universe is only valuable if we can see what's hiring. This skill bridges the gap between "we know these companies" and "here are the specific roles worth applying to." The key challenge is avoiding false negatives — missing a perfect role because of overly aggressive filtering is worse than spending a few extra minutes evaluating irrelevant listings.

---

## How job search works

### 1. Pick companies to search

The user and Claude decide which companies to search. Options:
- A specific company: "check Spotify"
- A sector: "search all fintech companies"
- The full resolved universe: "scan everything"
- Companies not searched recently: "what haven't we checked in a while?"

Query the database for resolved companies matching the criteria. Each company has an `ats_provider` and `ats_slug` (and optionally `ats_extra`) that tells us how to query their job board.

### 2. Fetch all open positions

For each company, call its ATS provider's list endpoint to get every open position. The provider-specific reference file in `references/` documents the exact API call, response shape, and quirks.

This is a bulk fetch — get everything, filter nothing. The raw response comes back as structured JSON with titles, locations, departments, and IDs.

### 3. Claude triages the list

This is the critical step where intelligence matters. Claude reads every job title and its metadata (location, department, team) and decides which ones are worth fetching the full description for.

**The only hard filter is location.** Drop roles that are physically impossible — a role exclusively in São Paulo with no remote-UK option. But be careful: location fields are inconsistent across companies and providers. "EMEA", "Europe", "Remote", "UK", "London", "London, UK", "London, England", "United Kingdom", "Home Mix" (Spotify's term for remote) could all mean eligible. When in doubt, keep it in.

**Everything else is Claude's judgment.** Read the title, department, and team. Decide if there's any reasonable chance this role fits the profile. Don't filter on:
- Seniority keywords in titles — companies use these inconsistently
- Department names you don't recognise — investigate, don't drop
- Vague titles — "Engineer III" or "Product - Apple JDK" could be anything

The goal: over-include rather than over-exclude. If 30% of what survives triage turns out to be irrelevant after reading the full description, that's fine. If 1% of what was dropped was actually a strong fit, that's a failure.

### 4. Fetch full descriptions for triaged candidates

For each surviving candidate, call the provider's detail endpoint to get the full job description, requirements, qualifications, and any other useful fields. The provider reference documents the exact call.

### 5. Evaluate each role against the profile

Read the full description. Assess against the user's profile:

- **Seniority reality check** — what does the description actually require? Ignore the title. Look for years of experience, specific expectations, scope of responsibility. A "Backend Engineer" might require 5+ years. A "Software Engineer III" might be entry-level at that company.
- **Tech stack overlap** — do the technologies mentioned match the profile? Partial overlap is fine (you know Rust but they want Java — transferable). Zero overlap is a flag.
- **Domain alignment** — does the work connect to any projects or interests in the profile? Financial systems, ML infrastructure, distributed systems, low-latency engineering, data pipelines?
- **Dealbreakers** — security clearance requirements the user can't meet, specific credentials, years of experience that are clearly hard requirements (not "nice to haves")?
- **Could you make a compelling case?** — even with gaps, would the profile tell a coherent story for this role?

Write the evaluation:
- `strong_fit` — profile aligns well, gaps are minor or addressable in a cover letter
- `weak_fit` — interesting role but significant gaps; worth watching, maybe not worth applying now
- `no_fit` — wrong seniority, wrong domain, or dealbreakers present

### 6. Write results to the database

Insert each evaluated job into the `jobs` table with company_id, title, URL, location, description, evaluation status, fit assessment (the reasoning), and fit score.

The TUI picks these up and displays them in real time as evaluations complete.

---

## Provider references

Each supported ATS has its own reference file documenting API specifics. Read the relevant reference before querying a provider.

| Provider | Reference file | Status |
|----------|---------------|--------|
| Lever | `references/lever.md` | Ready |
| Greenhouse | `references/greenhouse.md` | Planned |
| Ashby | `references/ashby.md` | Planned |
| Workable | `references/workable.md` | Planned |
| SmartRecruiters | `references/smartrecruiters.md` | Planned |
| Workday | `references/workday.md` | Planned |
| Eightfold | `references/eightfold.md` | Planned |

---

## Searching multiple companies

When scanning multiple companies, process them sequentially by company (fetch all jobs → triage → fetch details → evaluate) rather than mixing jobs across companies. This keeps the evaluation context clean — Claude is thinking about one company's culture and role structure at a time.

For large scans (10+ companies), consider parallelising the bulk fetch step (step 2) across companies while keeping triage and evaluation sequential.

---

## Quality checklist

Before presenting results, verify:

- [ ] Every role with a London/UK/Remote location that could plausibly be technical was triaged in, not filtered out
- [ ] Triage decisions were based on reading titles and metadata, not on keyword matching against a hardcoded list
- [ ] Full descriptions were fetched for every triaged candidate — no evaluations based on title alone
- [ ] Evaluation reasoning is specific: "requires 5+ years backend experience" not "too senior"
- [ ] Strong fits include a clear explanation of which profile elements align and what gaps exist
- [ ] Results are written to the database with full evaluation details, not just a pass/fail
