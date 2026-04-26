# Remediation Guide

> How to fix every type of issue the integrity check can detect. Each section covers one issue type with the exact procedure, SQL, and quality requirements.

---

## Table of Contents

1. [Stale company grades (profile-driven)](#issue-stale-company-grades-profile-driven)
2. [Stale job grades (profile-driven)](#issue-stale-job-grades-profile-driven)
3. [Shallow / generic grade reasoning](#issue-shallowgeneric-grade-reasoning)
4. [Shallow / generic fit assessments](#issue-shallowgeneric-fit-assessments)
5. [Unresolved ATS (companies still at 'potential' status)](#issue-unresolved-ats-companies-still-at-potential-status)
6. [Dead bespoke URLs](#issue-dead-bespoke-urls)
7. [Missing job descriptions](#issue-missing-job-descriptions)
8. [Missing company grades](#issue-missing-company-grades)
9. [Missing job grades](#issue-missing-job-grades)
10. [Generic why_relevant](#issue-generic-why_relevant)
11. [Orphaned records](#issue-orphaned-records)

---

## Issue: Stale company grades (profile-driven)

**When this happens:** The profile has changed (new project, new skill, updated preferences) in a way that's relevant to a company's grade reasoning. For example, a company was graded B because "lacks cloud infrastructure experience" but the user has since added a cloud project.

**How to fix:**

1. Read ALL files in `profile/` fresh.
2. Read the company's current `grade_reasoning` from the database.
3. Identify which profile change affects this grade (e.g., new Kubernetes project invalidates "lacks container orchestration experience").
4. Re-evaluate the company against the full rubric in `skills/grade-companies/references/grading-rubric.md`.
5. Write new grade reasoning with specific profile evidence (projects by name, technologies, career targets).
6. Present the old grade + reasoning alongside the new grade + reasoning for user review.
7. After user approval:

```sql
UPDATE companies
SET grade = ?,
    grade_reasoning = ?,
    graded_at = datetime('now'),
    why_relevant = ?
WHERE id = ?;
```

If the new grade is C, also set `status = 'archived'`.

---

## Issue: Stale job grades (profile-driven)

**When this happens:** Profile changes affect job fit assessments. A job graded C because "no Go experience" might deserve reassessment after the user ships a Go project.

**How to fix:**

1. Read ALL files in `profile/` fresh.
2. Fetch the job description — if `raw_description` is NULL or thin, use WebFetch on the job URL first.
3. Re-evaluate against the full rubric in `skills/grade-jobs/references/grading-rubric.md`.
4. Write new fit_assessment with specific profile connections (project names, technologies, gaps).
5. Present old vs new for user review.
6. After approval:

```sql
UPDATE jobs
SET grade = ?,
    fit_assessment = ?,
    fit_score = ?,
    evaluation_status = ?,
    graded_at = datetime('now')
WHERE id = ?;
```

---

## Issue: Shallow/generic grade reasoning

**When this happens:** Existing `grade_reasoning` or `why_relevant` is generic — "good company, decent tech" instead of specific profile connections.

**How to fix:**

1. Read ALL files in `profile/`.
2. Research the company (website, engineering blog, careers page, funding).
3. Rewrite `grade_reasoning` with:
   - Specific projects from `profile/projects/` that align (by name)
   - Specific technologies from `profile/skills.md` the company uses
   - Growth/funding signals if findable (but absence of data is not a negative — see evidence standards)
   - Sponsorship evidence (sponsor register check)
   - Boundary clarity — why this grade and not the adjacent one
4. Rewrite `why_relevant` with specific profile connections.
5. See `references/quality-standards.md` for examples of good vs bad reasoning.

---

## Issue: Shallow/generic fit assessments

**When this happens:** Existing `fit_assessment` on jobs is generic — "good role, decent fit" instead of specific analysis.

**How to fix:**

1. Read ALL files in `profile/`.
2. Read the full job description (fetch via WebFetch if needed).
3. Rewrite `fit_assessment` with:
   - Specific projects that demonstrate relevant capability (by name)
   - Technology match details (what the job requires vs what the candidate has)
   - Gap analysis (what's missing, how significant, whether portfolio-gaps.md lists it)
   - Sponsorship analysis with visa timeline
   - Career trajectory fit
4. See `references/quality-standards.md` for the full standard.

---

## Issue: Unresolved ATS (companies still at 'potential' status)

**How to fix:**

1. Run the mechanical resolver first: `cernio resolve --company "CompanyName"`
2. If that succeeds, the company is now resolved. Done.
3. If it fails, follow the AI fallback procedure:
   - WebSearch for `"{company name}" careers`
   - Visit the careers page, look for ATS URLs in links/iframes/redirects
   - Reference `skills/resolve-portals/references/ats-providers.md` for URL patterns per provider
   - Extract slug, verify against provider API
   - If supported ATS found: insert portal, update status to 'resolved'
   - If unsupported ATS or custom portal: update status to 'bespoke', set careers_url
   - If company appears dead: report to user with evidence, recommend removal

---

## Issue: Dead bespoke URLs

**How to fix:**

1. WebFetch the stored `careers_url` to confirm it's dead (404, timeout, domain parked).
2. WebSearch for `"{company name}" careers` to find the current careers page — the company may have moved.
3. If a new careers page is found:
   - Check if they've moved to a supported ATS (extract ATS URL, verify)
   - If supported ATS: insert portal, update status to 'resolved'
   - If still bespoke: update `careers_url` to the new URL
4. If no careers page exists and the company appears inactive:
   - Check Companies House (for UK companies) for dissolution/liquidation status
   - Recommend archival to the user with evidence

---

## Issue: Missing job descriptions

**How to fix:**

1. WebFetch the job's URL to get the full posting page.
2. Extract the job description text.
3. If the page is behind a login wall or returns no content:
   - WebSearch for the job title + company name on LinkedIn, Indeed, Glassdoor
   - Try to find the listing on an aggregator
4. If found, update the database:

```sql
UPDATE jobs SET raw_description = ? WHERE id = ?;
```

5. If the description cannot be found after multiple attempts, flag the job — it should not be graded without a description.

---

## Issue: Missing company grades

**How to fix:**

Invoke the grade-companies workflow: read the profile, read the rubric, research the company, assign a grade with full evidence-based reasoning. Follow `skills/grade-companies/SKILL.md` exactly.

---

## Issue: Missing job grades

**How to fix:**

Invoke the grade-jobs workflow: read the profile, read the rubric, fetch the job description, evaluate across all dimensions, assign a grade with profile-specific fit assessment. Follow `skills/grade-jobs/SKILL.md` exactly.

---

## Issue: Generic why_relevant

**How to fix:**

Rewrite to connect the company to specific profile elements:
- Name at least one project from `profile/projects/` that aligns
- Name at least one technology from `skills.md` the company uses
- Explain what about their work maps to the candidate's technical identity

See `references/quality-standards.md` for examples.

---

## Issue: Orphaned records

**How to fix:**

```sql
-- Find orphaned user decisions
SELECT ud.id FROM user_decisions ud
LEFT JOIN jobs j ON ud.job_id = j.id
WHERE j.id IS NULL;

-- Delete them
DELETE FROM user_decisions WHERE job_id NOT IN (SELECT id FROM jobs);
```

These are safe to auto-fix — orphaned records serve no purpose. But still present the count to the user before deleting.
