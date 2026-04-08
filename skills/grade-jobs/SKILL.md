# grade-jobs

Grades ungraded jobs from the database in prioritised batches, evaluating each against the user's profile across multiple dimensions and writing structured assessments back to the DB. Use when the user says "grade jobs", "evaluate pending jobs", "rate the next batch", "grade ungraded", "what jobs need grading", "evaluate the queue", "process pending jobs", or when the session's purpose is clearing the grading backlog. Not for searching or fetching jobs (that's search-jobs), not for discovering companies (that's discover-companies), and not for grading companies themselves (that's grade-companies).

---

## Why this skill exists

The search pipeline deposits hundreds of jobs into the database with `evaluation_status = 'pending'` and no grade. Those jobs are useless until a human-quality judgment evaluates them against the profile. Grading is the step that turns raw listings into actionable career intelligence. The user should see the best opportunities first, not wade through noise to find signal.

---

## Before you start

Read all reference files in `references/` before grading begins:

| File | What it gives you |
|------|-------------------|
| `references/grading-rubric.md` | The full grading rubric: dimensions, weights, grade scale, worked examples, boundary cases |
| `references/profile-context.md` | Distilled profile: strengths, targets, constraints, dealbreakers, what makes a job exciting |
| `references/prioritisation-guide.md` | How to order the pending queue so the most promising jobs get graded first |

Also read:
- `profile/preferences.toml` for hard constraints and soft signals
- `profile/portfolio-gaps.md` to understand known gaps (and to update it as new patterns emerge)

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

### 3. Get the job description

For each job, check `raw_description` in the database. If it exists and is non-empty, use it directly.

If `raw_description` is NULL or empty, fetch the description from the job's URL. Use the appropriate ATS provider endpoint or fetch the page directly. Write the fetched description back to the database:

```sql
UPDATE jobs SET raw_description = ? WHERE id = ?
```

If you cannot retrieve the description after reasonable effort, grade conservatively based on title and company context alone, and note in the assessment that the description was unavailable.

### 4. Evaluate the job

Read the full description. Assess it across every dimension in the grading rubric, with particular attention to the critical dimensions (career ceiling and seniority match) that can force an F grade regardless of other strengths.

The evaluation should answer:
- Is the seniority achievable given the profile? Look past the title to the actual requirements.
- Does this role lead somewhere valuable over a 10-15 year trajectory?
- What profile elements align strongly? What gaps exist?
- Is there a compelling narrative for why this candidate fits?
- What is the sponsorship situation?

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

### 6. Batch discipline

Grade approximately 30 jobs per session. After each batch:

1. Report progress: "Graded 30/142 pending. Breakdown: 2 SS, 4 S, 8 A, 6 B, 3 C, 7 F."
2. Summarise highlights: name the SS and S roles with one-line reasons.
3. Note any portfolio gap patterns observed (see below).
4. Ask the user whether to continue with the next batch or stop.

If the pending count is small (under 15), grade them all without asking.

---

## Portfolio gap tracking

As you evaluate jobs, watch for patterns in what strong matches ask for that the profile lacks. This is one of the most valuable outputs of the grading process.

Examples of patterns to watch for:
- "Every S-tier infrastructure role mentions Kubernetes. The profile has no containerisation experience."
- "Three trading systems roles asked for FIX protocol experience, which Nyquestro's FIX TCP acceptor partially addresses but isn't highlighted."
- "Go appears in 60% of platform engineering roles. The profile has no Go."

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

- [ ] Every job was graded against the full rubric, not just title-keyword matching
- [ ] Seniority assessment is based on the description's actual requirements, not the title
- [ ] Career ceiling reasoning considers the domain's 10-15 year trajectory, not just the immediate role
- [ ] Company grade was factored into the job grade (a mediocre role at an S-tier company has more signal than the same role at a C-tier company)
- [ ] Sponsorship was assessed based on company size, sponsor licence status, and description signals, not assumed
- [ ] SS and S grades have multi-paragraph assessments that would help the user decide whether to apply
- [ ] C and F grades have a clear, specific reason (not just "not a fit")
- [ ] Grades were written to the database with correct evaluation_status mapping
- [ ] Portfolio gap patterns from this batch have been noted (even if no new patterns emerged, confirm you checked)
- [ ] The batch summary includes a count breakdown and highlights the strongest opportunities
