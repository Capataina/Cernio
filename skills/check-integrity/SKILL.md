# check-integrity

Audits the health and consistency of the Cernio database by combining mechanical checks with judgment-based evaluation. Use when asked to "check integrity", "audit the database", "check for stale grades", "verify data quality", "are my grades up to date", "integrity check", "health check on the database", or after significant profile changes that may have invalidated existing grades. Not for grading new companies (that's grade-companies), grading jobs (that's grade-jobs), or discovering new companies (that's discover-companies).

---

## Why this skill exists

The Cernio database is a living system. Profile changes — a new project, a new skill, updated preferences, a visa status change — can silently invalidate grades and evaluations that were correct when they were written. A company graded C because the user lacked Kubernetes experience should be reconsidered once a Kubernetes project lands in the profile. A job evaluated as "weak fit" due to a Go requirement may become viable after the user ships a Go project.

Mechanical checks (`cernio check`) catch structural problems: orphaned records, missing ATS slugs, stale entries by timestamp. But they cannot catch *semantic* staleness — a grade whose reasoning no longer holds given the current profile. That requires judgment: reading the profile, reading the grade reasoning, and determining whether the evaluation still makes sense.

This skill bridges the two. It runs the mechanical checks first, then applies judgment to catch the problems that timestamps and foreign keys cannot detect.

---

## How integrity checking works

### 1. Run mechanical checks

Execute `cernio check` and capture the output. This provides the baseline report covering:

- ATS slug verification (do resolved slugs still point to working endpoints?)
- Stale entry detection (entries not updated within configured thresholds)
- Orphaned records (jobs pointing to non-existent companies, etc.)
- Missing required fields

Review the mechanical report and note any issues that need attention.

### 2. Load the current profile

Read **all files in `profile/`**. Every file, no exceptions. The profile is the reference point against which all grades and evaluations are assessed. Without the full profile loaded, you cannot judge whether existing evaluations are still valid.

Pay particular attention to:
- Recently modified profile files — these signal areas where grades may need updating
- New projects or skills — these expand what the user can match against
- Changed preferences — these shift which companies and roles are relevant
- Visa status — this affects sponsorship weighting

### 3. Detect profile-driven staleness

Compare the current profile state against graded entries to find evaluations that may no longer be accurate.

Query companies with existing grades:

```sql
SELECT id, name, grade, grade_reasoning, graded_at, why_relevant, status
FROM companies
WHERE grade IS NOT NULL
  AND status != 'archived'
ORDER BY graded_at ASC;
```

For each graded company, assess whether the grade reasoning still holds given the current profile:

- **Skill alignment shifts**: If the grade reasoning mentions a skill gap that the user has since filled (e.g., "lacks cloud infrastructure experience" but a new AWS project exists), the grade may need upward revision.
- **Preference changes**: If the grade reasoning relies on location or sector preferences that have changed, the evaluation is stale.
- **New portfolio strength**: If the user has added a strong new project that aligns with a company's tech stack, the company may deserve a higher grade.

Do not re-evaluate every company. Focus on entries where the profile change is *relevant* to the grade reasoning. A new Rust project does not affect the grade of a company graded on sponsorship concerns.

### 4. Audit grade quality

Spot-check a sample of graded entries to verify reasoning quality. Pull 3-5 graded jobs and 3-5 graded companies and verify:

- Does the `grade_reasoning` make sense given the current profile?
- Is the reasoning specific enough to be useful, or is it generic filler?
- Does the `why_relevant` field connect the company to the profile meaningfully?
- For jobs: does the `fit_assessment` accurately reflect the match between the job requirements and the profile?

Flag any entries where the reasoning is thin, generic, or contradicted by the current profile state.

### 5. Check for missing data

Identify gaps in the database that prevent effective operation:

```sql
-- Companies without grades (excluding archived)
SELECT id, name, status FROM companies
WHERE grade IS NULL AND status != 'archived';

-- Jobs without evaluations
SELECT j.id, j.title, c.name as company_name
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE j.fit_grade IS NULL
  AND j.status != 'archived'
  AND c.status != 'archived';

-- Companies with empty or null why_relevant
SELECT id, name, grade FROM companies
WHERE (why_relevant IS NULL OR why_relevant = '')
  AND status != 'archived';

-- Jobs without descriptions
SELECT j.id, j.title, c.name as company_name
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE (j.description IS NULL OR j.description = '')
  AND j.status != 'archived';
```

### 6. Relevance refresh

For companies where `why_relevant` is generic (e.g., "interesting tech company", "found on fintech list") or stale (references profile details that have changed), draft updated relevance statements that connect the company to the current profile specifically.

### 7. Present findings

Present a structured integrity report to the user, organised by severity:

```
## Integrity Report

### Mechanical Issues (from cernio check)
- [list any issues from the mechanical check]

### Profile-Driven Staleness
- **[Company Name]** (current grade: B, graded 2026-03-15)
  Grade reasoning says "lacks cloud experience" but profile now includes
  AWS deployment in Nyquestro. Recommend re-evaluation — likely A-tier.

### Grade Quality Issues
- **[Company Name]** — grade reasoning is generic ("good company, decent tech").
  Needs substantive reasoning covering engineering reputation, sponsorship,
  and technical alignment.

### Missing Data
- 3 companies without grades
- 7 jobs without evaluations
- 2 companies with empty why_relevant

### Relevance Refresh Candidates
- **[Company Name]** — why_relevant says "found on UK tech list".
  Suggested update: "Infrastructure-focused payments company; core platform
  work aligns with distributed systems experience from Nyquestro and Paxos."

### Recommendations
1. Re-grade [Company A] and [Company B] — profile changes directly affect their evaluations
2. Run grade-companies on the 3 ungraded companies
3. Update why_relevant for 2 companies with generic text
```

Do not automatically execute any changes. Present findings and recommendations, then wait for the user to decide what to act on. The user may choose to re-grade specific companies, update relevance text, or defer certain items.

---

## When to recommend this skill

This skill should be recommended (by any agent, in any session) when:

- The user has just updated their profile significantly (new project, new skill, changed preferences)
- A substantial amount of time has passed since the last integrity check
- The user is about to start a job search session and wants confidence that the database is current
- A grading run has completed and the user wants to verify quality
- The user explicitly asks about data quality or staleness

---

## Quality checklist

Before presenting the integrity report:

- [ ] `cernio check` was run and its output is included in the report
- [ ] All profile files were read — not a subset, all of them
- [ ] Staleness assessment focuses on entries where the profile change is relevant, not a blanket re-evaluation of everything
- [ ] Grade quality spot-checks include specific examples of good and bad reasoning
- [ ] Missing data queries were run and results are quantified
- [ ] Recommendations are actionable and prioritised — the most impactful items first
- [ ] No changes were made to the database — this skill reports, it does not write
