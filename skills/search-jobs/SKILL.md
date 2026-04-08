# search-jobs

Searches for open positions at companies in the database using the Rust pipeline, then grades the results against the user's profile. Use when the user says "search for jobs", "check Spotify's openings", "what's available at [company]", "find me roles", "scan everything", or when it is time to scan resolved companies for new opportunities. Not for discovering companies (that's discover-companies), not for resolving ATS slugs (that's populate-db / resolve-portals), not for grading companies (that's grade-companies).

---

## Why this skill exists

The company universe is only valuable if we can see what is hiring. This skill bridges the gap between "we know these companies" and "here are the specific roles worth applying to." The key challenge is avoiding false negatives — missing a perfect role because of overly aggressive filtering is worse than spending a few extra minutes evaluating irrelevant listings.

This skill is the **orchestration layer**. The mechanical volume work — fetching job listings from ATS APIs, applying location and keyword filters, deduplicating against the database — is handled by the `cernio search` Rust script. This skill provides the judgment that the script cannot: evaluating roles against the user's profile, grading fit, and presenting results for human review.

---

## Before you start

**Read all `profile/` files.** You need the user's background, skills, career targets, visa status, and preferences to evaluate job fit. This is non-negotiable — without the profile loaded, you cannot grade jobs.

---

## How job search works

### 1. Run `cernio search` to fetch and filter jobs

This is the handoff to the Rust pipeline. The script fetches all open positions from resolved companies, applies mechanical filters (location, department), deduplicates against existing database entries, and inserts new jobs.

**CLI reference:**

| Command | What it does |
|---------|-------------|
| `cernio search --dry-run` | Preview filter statistics without fetching. Shows how many companies will be searched, expected job counts, filter breakdown. |
| `cernio search` | Fetch all jobs from all resolved companies. Applies filters, deduplicates, inserts new jobs into the database. |
| `cernio search --company palantir` | Search a single company by name. Useful for checking one company's latest openings. |
| `cernio search --grade S` | Search only S-tier companies. Useful for prioritising high-value targets. |

**Workflow:**
1. Run `cernio search --dry-run` first to preview the batch and filter stats
2. Review the preview — confirm the scope looks right
3. Run `cernio search` (or with filters) to execute
4. Review the output

### 2. Review what was found

After the search completes, report the results to the user:

- How many companies were searched
- How many total jobs were fetched from ATS APIs
- How many survived filters (location, department)
- How many are new (not already in the database)
- Any errors (companies that failed to fetch, API timeouts)

This gives the user a sense of scale before moving to the grading step.

### 3. Show ungraded jobs

Run `cernio pending` to list jobs that have been fetched but not yet graded. These are the jobs that need Claude's evaluation.

Present the pending count grouped by company so the user can decide whether to grade everything or focus on specific companies.

### 4. Grade jobs against the profile

This is where the skill's core value lies — the script fetches, Claude evaluates.

For each ungraded job, read the full description from the database and evaluate it against the user's profile. Use the grading framework in `references/grading-rubric.md`.

**Evaluation dimensions (from the rubric):**

- **Seniority match** — Is this role achievable given the profile? Ignore the title; read the actual requirements.
- **Career ceiling** — Does this role/domain lead to strong long-term earnings? Systems, infrastructure, ML infra, trading systems = high ceiling.
- **Skill breadth** — Exposure to multiple layers (backend + infra + data) or locked into one narrow thing?
- **Company signal** — Does this company name open doors on a CV?
- **Technical depth** — Genuinely hard problems? Performance-critical, distributed systems, scale?
- **Sponsorship viability** — Can and will they sponsor when needed?
- **Domain transferability** — Are the skills portable to other companies?
- **Growth environment** — Strong engineers, mentorship, code review culture?
- **Tech stack relevance** — Broadly valuable technologies?

**Grading scale:**

| Grade | Meaning | DB status | Action |
|-------|---------|-----------|--------|
| SS | Apply immediately | `strong_fit` | Detailed evaluation with profile alignment and gaps |
| S | Strong candidate | `strong_fit` | Detailed evaluation with profile alignment and gaps |
| A | Worth applying | `weak_fit` | Evaluation with noted gaps |
| B | Backup | `weak_fit` | Brief evaluation |
| C | Only if desperate | `no_fit` | One-line reason |
| F | Do not apply | `no_fit` | One-line reason |

**For S and above:** Always provide detailed evaluation — which profile elements align, what gaps exist, what makes this role special.

**For A and B:** Evaluate if the user wants depth.

**For C and F:** The grade and one-line justification is sufficient.

Write grades and evaluations back to the database as each job is assessed.

### 5. Report and continue

After grading a batch, present results grouped by grade for scannability:

```
## SS
- Software Engineer, New Grad - Infrastructure @ Palantir
  New grad, tier-1 brand, infrastructure breadth across 4 tracks, known sponsor

## S
- Software Engineer, New Grad - Production Infra @ Palantir
  Same brand, genuine new grad, more ops-flavoured but strong growth environment

## A
- Backend Software Engineer - Infrastructure @ Palantir
  Rust in stack, perfect domain, but reads as mid-level — reach application

## F
- Account Executive - Backstage @ Palantir
  Sales role, not engineering
```

Ask the user if they want to continue grading the next batch or stop here.

---

## Provider references

The Rust scripts handle all ATS API interactions, so per-provider API documentation is no longer needed for this skill. The reference files below remain in the repository for architectural reference and are used by the scripts themselves:

| Provider | Reference file | Purpose |
|----------|---------------|---------|
| Lever | `references/lever.md` | API shape reference for script development |
| Ashby | `references/ashby.md` | API shape reference for script development |

The grading rubric remains the primary reference for this skill:

| Reference | File | Purpose |
|-----------|------|---------|
| Job grading rubric | `references/grading-rubric.md` | Evaluation dimensions, grading scale, presentation format |

---

## Searching multiple companies

When scanning multiple companies, the `cernio search` script handles the fetching in bulk. The grading step processes jobs sequentially by company to keep evaluation context clean — Claude thinks about one company's culture and role structure at a time.

For large batches (50+ pending jobs), offer to grade in batches of 10–15 with user review between batches. This prevents context fatigue and lets the user steer.

---

## Quality checklist

Before presenting results, verify:

- [ ] Profile files were read before grading began — evaluations reference the actual profile, not generic criteria
- [ ] `cernio search` was run before grading — no manual ATS fetching
- [ ] Every role with a London/UK/Remote location that could plausibly be technical was included by the script's filters
- [ ] Evaluation reasoning is specific: "requires 5+ years backend experience" not "too senior"
- [ ] Strong fits (S/SS) include detailed explanation of which profile elements align and what gaps exist
- [ ] Grades are written to the database with full evaluation details, not just a pass/fail
- [ ] Results are presented grouped by grade for scannability
- [ ] User was asked whether to continue after each batch
