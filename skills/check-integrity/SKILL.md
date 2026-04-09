# check-integrity

Audits the health and consistency of the Cernio database by combining mechanical checks with judgment-based evaluation. Use when asked to "check integrity", "audit the database", "check for stale grades", "verify data quality", "are my grades up to date", "integrity check", "health check on the database", or after significant profile changes that may have invalidated existing grades. Not for grading new companies (that's grade-companies), grading jobs (that's grade-jobs), or discovering new companies (that's discover-companies).

---

## Mandatory reads — do not proceed without completing these

**STOP. Before executing any integrity check, you MUST read these files in full:**

1. **Every file in `profile/`** — all 15 files, no exceptions. The profile is the reference point against which all grades and evaluations are assessed.
2. **`references/remediation-guide.md`** — how to fix every type of issue this skill can detect. Without this, you can identify problems but not solve them.
3. **`references/quality-standards.md`** — what good vs bad grade reasoning and fit assessments look like. This is how you judge whether existing assessments meet the bar.
4. **`references/profile-context.md`** — how to read and synthesise the profile for integrity assessment.
5. **`references/cross-checking-guide.md`** — systematic procedures for cross-checking grade consistency across the entire company and job universe. This is not a spot-check — it defines the comprehensive relative grading pass that compares every graded entity against the full population.

**Do not begin any checks until all mandatory reads are complete.**

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

### 2. Format job descriptions

Run `cernio format` to clean all raw HTML descriptions and fit assessments in the database. This must happen before any judgment-based evaluation — grading agents working on HTML-laden descriptions waste tokens on `data-*` attributes and nested tag soup, and may produce confused assessments. The formatter is idempotent and fast (no-op when already formatted), so it is always safe to run.

```bash
cargo run -- format
```

If any descriptions were formatted, note the count in the integrity report.

### 3. Load the current profile

Read **all files in `profile/`**. Every file, no exceptions. The profile is the reference point against which all grades and evaluations are assessed. Without the full profile loaded, you cannot judge whether existing evaluations are still valid.

Pay particular attention to:
- Recently modified profile files — these signal areas where grades may need updating
- New projects or skills — these expand what the user can match against
- Changed preferences — these shift which companies and roles are relevant
- Visa status — this affects sponsorship weighting

### 4. Detect profile-driven staleness

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

### 5. Audit grade quality

Spot-check a sample of graded entries to verify reasoning quality. Pull 3-5 graded jobs and 3-5 graded companies and verify:

- Does the `grade_reasoning` make sense given the current profile?
- Is the reasoning specific enough to be useful, or is it generic filler?
- Does the `why_relevant` field connect the company to the profile meaningfully?
- For jobs: does the `fit_assessment` accurately reflect the match between the job requirements and the profile?

Flag any entries where the reasoning is thin, generic, or contradicted by the current profile state.

### 6. Cross-check grades across the full universe

This is the most important step and the one that catches errors no individual grading session can detect. Follow the full procedure in `references/cross-checking-guide.md`.

**For companies:** Load all graded companies. Compare within each tier (do they all genuinely belong together?), across tier boundaries (is every A genuinely less valuable than every S?), and check for specific red flags (famous employer at C, unknown startup at S, tech stack as primary driver).

**For jobs:** Verify company grade / job grade consistency (graduate role at S-tier company should rarely be below A). Compare all SS/S jobs against each other. Verify seniority requirements are cited in fit assessments. Spot-check description-assessment consistency by reading both the raw description and the fit assessment for a sample.

**The cardinal rule:** Before changing ANY grade, you MUST have read the entity's complete database record (what_they_do/raw_description, grade_reasoning/fit_assessment, why_relevant) AND the full candidate profile. An integrity agent that changes a grade without reading the record has produced damage, not value. See the cross-checking guide for the full procedure.

Present all findings as recommendations, not executed changes. The user decides what to act on.

### 7. Check for missing data

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

### 8. Portfolio gap analysis — active maintenance

This step does not just check whether portfolio-gaps.md is being maintained — it actively maintains it. Read 10 jobs from EACH grade tier (SS, S, A, B, C, F — fewer if the tier has fewer than 10 jobs), prioritising jobs at S-tier companies, then A, then B.

```sql
-- Sample 10 jobs per grade, prioritised by company grade
SELECT j.id, j.title, j.grade, j.raw_description, j.fit_assessment,
       c.name as company, c.grade as company_grade
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE j.grade = ?1 AND j.evaluation_status <> 'archived'
ORDER BY
    CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END,
    RANDOM()
LIMIT 10;
```

Run this for each grade tier: SS, S, A, B, C, F. Read the `raw_description` and `fit_assessment` for each sampled job. Across all ~60 sampled jobs, track:

1. **Technologies that appear repeatedly but are absent from `profile/skills.md`**: Count occurrences, note which companies and role types ask for them. Example: "Kubernetes appeared in 12 of 60 sampled roles across SS to B tiers."

2. **Domain knowledge the market expects**: Example: "4 trading roles asked for FIX protocol experience."

3. **Experience patterns that recur**: Example: "8 roles mentioned production incident management."

4. **Confirmed strengths**: Skills from the profile that the market clearly values. Example: "Rust appeared in 6 SS/S-tier roles — genuine differentiator."

After analysis, **write the findings to `profile/portfolio-gaps.md`**:
- Update the "Patterns from Job Evaluations" section with concrete findings (technology, count, roles, companies, impact)
- Update "Known Gaps" if new gaps were identified or existing gaps have been closed
- Update "Current Strengths" if the market confirms profile strengths

**This is not optional.** If the integrity check runs and portfolio-gaps.md is not updated, the career coaching loop is broken. Even "no new patterns found" should be noted with the date checked.

Also check staleness:
- If `portfolio-gaps.md` has no entries in "Patterns from Job Evaluations" despite graded jobs existing, flag this prominently
- Check whether "Known Gaps" entries have been closed by recent profile changes

### 9. Relevance refresh

For companies where `why_relevant` is generic (e.g., "interesting tech company", "found on fintech list") or stale (references profile details that have changed), draft updated relevance statements that connect the company to the current profile specifically.

### 10. Present findings

Present a structured integrity report to the user, organised by severity:

```
## Integrity Report

### Mechanical Issues (from cernio check)
- [list any issues from the mechanical check]

### Profile-Driven Staleness
- **[Company Name]** (current grade: B, graded 2026-03-15)
  Grade reasoning says "lacks cloud experience" but profile now includes
  AWS deployment in a recent project. Recommend re-evaluation — likely A-tier.

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
  work aligns with distributed systems experience from the profile's projects."

### Recommendations
1. Re-grade [Company A] and [Company B] — profile changes directly affect their evaluations
2. Grade the 3 ungraded companies
3. Update why_relevant for 2 companies with generic text
```

Do not automatically execute any changes. Present findings and recommendations, then wait for the user to decide what to act on. The user may choose to re-grade specific companies, update relevance text, or defer certain items.

---

## Remediation mode

When the user reviews the integrity report and says "fix these", "update these", "regrade these", or similar, switch from reporting mode to remediation mode. The `references/remediation-guide.md` file contains detailed procedures for every issue type.

### How remediation works

1. **Confirm scope with the user.** Which issues should be fixed? All of them, or a specific subset?
2. **Read `references/remediation-guide.md`** for the procedure for each issue type.
3. **Execute fixes in priority order:**
   - Profile-driven staleness (regrade companies/jobs whose grades are invalidated by profile changes)
   - Quality issues (rewrite shallow grade reasoning and fit assessments with profile-specific evidence)
   - Missing data (fetch job descriptions, grade ungraded entries, write relevance statements)
   - Mechanical issues (fix orphaned records, update dead URLs, re-verify ATS slugs)
4. **Present each fix for user approval** before writing to the database. Never auto-fix without confirmation.
5. **Use the quality standards** from `references/quality-standards.md` as the bar for rewritten assessments.

### What the agent must know for remediation

- **Regrading companies:** Follow the full procedure in `skills/grade-companies/SKILL.md` and its references. Read the profile fresh, apply the rubric, write evidence-based reasoning.
- **Regrading jobs:** Follow the full procedure in `skills/grade-jobs/SKILL.md` and its references. Fetch the job description first (WebFetch the URL), read the profile, apply the rubric.
- **Resolving ATS:** Run `cernio resolve --company "CompanyName"` for mechanical resolution. If that fails, follow the AI fallback procedure in `skills/resolve-portals/SKILL.md` and its `references/ats-providers.md`.
- **Rewriting relevance:** Connect the company to specific projects, technologies, and domains from the profile. See `references/quality-standards.md` for examples.
- **Fetching missing descriptions:** Use WebFetch on the job URL. If behind a login wall, try WebSearch for the job title + company on LinkedIn, Indeed, or Glassdoor.

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

- [ ] All reference files in `references/` were read before starting the integrity check
- [ ] The remediation guide was consulted for every issue type identified
- [ ] `cernio check` was run and its output is included in the report
- [ ] All profile files were read — not a subset, all of them
- [ ] Staleness assessment focuses on entries where the profile change is relevant, not a blanket re-evaluation of everything
- [ ] Grade quality spot-checks include specific examples of good and bad reasoning
- [ ] Missing data queries were run and results are quantified
- [ ] Recommendations are actionable and prioritised — the most impactful items first
- [ ] No changes were made to the database — this skill reports, it does not write
