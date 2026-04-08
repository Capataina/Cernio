# grade-jobs

**Read ALL files in `profile/` before grading. Every file. Do not skip any — missing context leads to grading errors.** The profile is a living system that changes as the candidate's portfolio grows. Never rely on embedded snapshots or cached assumptions about the profile.

Grades ungraded jobs from the database in prioritised batches, evaluating each against the user's profile across multiple dimensions and writing structured assessments back to the DB. Use when the user says "grade jobs", "evaluate pending jobs", "rate the next batch", "grade ungraded", "what jobs need grading", "evaluate the queue", "process pending jobs", or when the session's purpose is clearing the grading backlog. Not for searching or fetching jobs (that's search-jobs), not for discovering companies (that's discover-companies), and not for grading companies themselves (that's grade-companies).

---

## Why this skill exists

The search pipeline deposits hundreds of jobs into the database with `evaluation_status = 'pending'` and no grade. Those jobs are useless until a human-quality judgment evaluates them against the profile. Grading is the step that turns raw listings into actionable career intelligence. The user should see the best opportunities first, not wade through noise to find signal.

---

## Before you start

1. **Read ALL files in `profile/`.** Every single one — `personal.md`, `visa.md`, `education.md`, `experience.md`, `projects.md`, `skills.md`, `preferences.toml`, `portfolio-gaps.md`, `resume.md`, `cover-letter.md`, `interests.md`, `certifications.md`, `languages.md`, `military.md`, `volunteering.md`. The profile-context reference file explains what to extract from each.

2. Read all reference files in `references/`:

| File | What it gives you |
|------|-------------------|
| `references/grading-rubric.md` | The full grading rubric: dimensions, weights, grade scale, worked examples, boundary cases |
| `references/profile-context.md` | Guide to reading and synthesising the profile files for job evaluation |
| `references/prioritisation-guide.md` | How to order the pending queue so the most promising jobs get graded first |

---

## The grading workflow

### 1. Query the pending queue

```sql
SELECT j.id, j.title, j.url, j.location, j.raw_description, j.posted_date,
       c.name AS company_name, c.grade AS company_grade, c.what_they_do
FROM jobs j
JOIN companies c ON c.id = j.company_id
WHERE j.evaluation_status = 'pending'
   OR j.evaluation_status = 'evaluating'
ORDER BY
    CASE c.grade
        WHEN 'S' THEN 1
        WHEN 'A' THEN 2
        WHEN 'B' THEN 3
        ELSE 4
    END,
    j.id ASC
```

Report the total count immediately: "142 jobs pending. Starting with the highest-signal batch."

### 2. Prioritise the batch

Do not grade in database insertion order. Use the prioritisation logic in `references/prioritisation-guide.md` to select the most promising jobs first. The compound signal is:

```
priority = company_grade x title_promise x role_type_alignment
```

Grade S-company jobs with promising titles before B-company jobs with generic titles. The user sees actionable results early instead of waiting for the full queue to clear.

### 3. Get the job description — never grade blind

For each job, check `raw_description` in the database. If it exists, is non-empty, and gives you enough detail to evaluate the role's actual responsibilities, seniority expectations, and technical requirements, use it directly.

**If `raw_description` is NULL, empty, too short (under ~100 words), or vague:**

1. Use WebFetch on the job's `url` to visit the actual posting page and extract the full description.
2. Write the fetched description back to the database: `UPDATE jobs SET raw_description = ? WHERE id = ?`
3. If the job page is behind a login wall or returns no useful content, try WebSearch for the job title + company name to find the listing on LinkedIn, Indeed, Glassdoor, or other job aggregators.

**Why this matters:** A title like "AI Engineer" at Apple could be anything — cutting-edge ML infrastructure, or QA testing for AI-powered hardware accessories. The description is the only way to know. A "Software Engineer, Platform" could require Angular and Redux (frontend disguised as platform) or Kubernetes and Terraform (actual infrastructure). Titles lie. Descriptions tell the truth.

**Never grade on title alone.** If after all attempts you still cannot find a description, skip the job and flag it for later rather than guessing. A wrong grade is worse than no grade — it either wastes the user's time (false SS) or hides a perfect opportunity (false F).

### 4. Evaluate the job

Read the full description. Assess it across every dimension in the grading rubric, with particular attention to the critical dimensions (career ceiling and seniority match) that can force an F grade regardless of other strengths.

The evaluation should answer:
- Is the seniority achievable given the candidate's experience level (from `experience.md`) and project depth (from `projects.md`)? Look past the title to the actual requirements.
- Does this role lead somewhere valuable over a 10-15 year trajectory, relative to the candidate's long-term targets (from `preferences.toml`)?
- What profile elements align strongly? What gaps exist (cross-reference `portfolio-gaps.md`)?
- Is there a compelling narrative for why this candidate fits?
- What is the sponsorship situation (based on the timeline from `visa.md`)?

### 5. Assign a grade and write to DB

```sql
UPDATE jobs
SET grade = ?,
    fit_assessment = ?,
    fit_score = ?,
    evaluation_status = ?
WHERE id = ?
```

| Grade | evaluation_status | fit_score range |
|-------|-------------------|-----------------|
| SS | `strong_fit` | 0.90 - 1.00 |
| S | `strong_fit` | 0.75 - 0.89 |
| A | `weak_fit` | 0.60 - 0.74 |
| B | `weak_fit` | 0.40 - 0.59 |
| C | `no_fit` | 0.20 - 0.39 |
| F | `no_fit` | 0.00 - 0.19 |

The `fit_assessment` field carries the reasoning. Scale depth by grade:

- **SS/S**: Full multi-paragraph assessment. Profile alignment, specific strengths that map to the role, gaps and how to address them in the application, sponsorship analysis, career trajectory analysis, and a recommended application approach.
- **A/B**: Concise assessment. Key strengths, notable gaps, why it falls short of S-tier, whether it's worth pursuing if higher-grade options are scarce.
- **C/F**: One to two sentences. The primary reason for the low grade.

### 6. Parallel grading

Job grading should always be parallelised using multiple agents. Split the pending queue into batches by company cluster (e.g. 5 agents each handling ~100-150 jobs) and run them simultaneously. Each agent reads the same profile, gets a different set of jobs with their descriptions, and outputs SQL UPDATE statements.

**Why parallel:** 700 jobs graded in ~3 minutes with 5 parallel agents vs 30+ minutes sequentially. The grading is independent per job — no agent needs another agent's output. Each agent reads the full profile and makes independent judgments.

**How to split:** Group by company so each agent has context about the companies it's grading. S-tier companies with many jobs should be in the same batch for consistency. Give each agent the job ID, company name, company grade, title, location, and description excerpt from the database.

**Each agent outputs SQL only:**
```sql
UPDATE jobs SET grade = 'X', evaluation_status = 'strong_fit'/'weak_fit'/'no_fit', fit_assessment = 'reason' WHERE id = NNN;
```

The orchestrator collects all SQL outputs and executes them in a single batch against the database.

### 7. Batch discipline

The orchestrator decides how many jobs to grade based on signal strength. The guiding principle: **grade everything important, not a random slice.**

- **Always include:** Every job with clear high-signal indicators (entry-level/graduate titles at S-tier companies, roles explicitly mentioning technologies from the profile, roles at companies with exceptional domain alignment). If there are 70 graduate roles, grade all 70 — they're all high priority.
- **Include generously:** When uncertain whether a job should be in this batch or deferred, include it. The cost of grading an extra job is a few minutes. The cost of deferring a perfect opportunity is potentially missing an application deadline.
- **Defer strategically:** Senior roles at B-tier companies, roles with generic titles and no clear signal, roles at companies with weaker alignment. These can wait for a later batch.

After grading completes:

1. Report progress: "Graded X/Y pending. Breakdown: N SS, N S, N A, N B, N C, N F."
2. Summarise highlights: name the SS and S roles with one-line reasons.
3. Note any portfolio gap patterns observed (see below).
4. Flag any jobs graded without descriptions — these need verification.
5. Ask the user whether to continue with remaining jobs or stop.

If the pending count is manageable, grade everything in one pass. Grading is largely a one-time cost per job — once done, the user has a stable, browsable, actionable list.

---

## Portfolio gap tracking

As you evaluate jobs, watch for patterns in what strong matches ask for that the profile lacks. This is one of the most valuable outputs of the grading process.

Examples of patterns to watch for:
- "Every S-tier infrastructure role mentions Kubernetes. The profile has no containerisation experience."
- "Three trading systems roles asked for FIX protocol experience — check `projects.md` to see if any project partially addresses this."
- "Go appears in 60% of platform engineering roles. Check `skills.md` for current Go proficiency."

After each batch, update `profile/portfolio-gaps.md` with any new patterns observed under the "Patterns from Job Evaluations" section. Be specific: name the roles that surfaced the pattern, the skill/tool that was missing, and how frequently it appeared.

---

## Presenting results to the user

After grading a batch, present a summary grouped by grade for scannability:

```
## Batch results (30 graded, 112 remaining)

### SS (2)
- Graduate Software Engineer, Infrastructure @ Cloudflare
  New grad role, tier-1 infrastructure company, systems-heavy, confirmed sponsor
- Software Engineer, New Grad — Trading Systems @ Jane Street
  Perfect domain alignment, legendary engineering culture, sponsors visas

### S (4)
- Software Engineer, Platform @ Palantir
  Strong brand, broad scope, slight reach on seniority but compelling profile narrative
  ...

### A (8)
- Backend Engineer @ Monzo — Good fintech signal, slightly narrow scope
  ...

### C/F (10)
- Senior Staff Engineer @ Unknown Corp — Hard 8+ years required
  ...
```

For SS and S roles, the full `fit_assessment` from the database should be available on request. Present it inline if the batch is small, or offer to show details for specific roles if the batch is large.

---

## Quality checklist

Before presenting a batch to the user, verify:

- [ ] All files in `profile/` were read before grading began
- [ ] Every job was graded against the full rubric, not just title-keyword matching
- [ ] Seniority assessment is based on the description's actual requirements, not the title
- [ ] Career ceiling reasoning considers the domain's 10-15 year trajectory, not just the immediate role
- [ ] Company grade was factored into the job grade (a mediocre role at an S-tier company has more signal than the same role at a C-tier company)
- [ ] Sponsorship was assessed based on company size, sponsor licence status, and description signals — using the timeline from `visa.md`, not assumed dates
- [ ] SS and S grades have multi-paragraph assessments that would help the user decide whether to apply
- [ ] C and F grades have a clear, specific reason (not just "not a fit")
- [ ] Grades were written to the database with correct evaluation_status mapping
- [ ] Portfolio gap patterns from this batch have been noted (even if no new patterns emerged, confirm you checked)
- [ ] The batch summary includes a count breakdown and highlights the strongest opportunities
