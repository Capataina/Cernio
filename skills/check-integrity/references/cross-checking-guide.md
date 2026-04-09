# Cross-Checking and Relative Grading Guide

> Systematic procedures for verifying grade consistency across the entire company and job universe. This is not a spot-check — it is a comprehensive pass that compares every graded entity against the full population to catch misgradings that individual grading sessions cannot detect.

---

## Why cross-checking exists

Individual grading sessions grade companies and jobs in batches. Even with calibration anchors, errors accumulate:

- An agent grades Akuna Capital S and Google A in different sessions — no single session sees the absurdity.
- A job at an S-tier company gets graded C because one agent applied stricter seniority standards than another.
- Two nearly identical roles at similar companies end up at different grades because different agents graded them.
- A company's grade reasoning no longer matches its actual grade because the grade was manually overridden but the reasoning wasn't updated.

Cross-checking catches these by comparing across the entire universe at once — something no individual grading session can do because each session only sees its batch.

---

## The cardinal rule: NEVER grade blind

**This is the most critical instruction in this entire reference file.**

Before changing ANY grade — company or job — you MUST have read:

1. **The full candidate profile** (all 15 files in `profile/`). Without this, you cannot assess fit.
2. **The entity's complete database record.** For companies: `what_they_do`, `why_relevant`, `grade_reasoning`, `grade`. For jobs: `raw_description`, `fit_assessment`, `grade`, plus the parent company's record.
3. **Comparison entities at the same and adjacent grade tiers.** You must understand what S looks like in this database before deciding something isn't S.

An integrity agent that reads a company name, sees "Grade: S", thinks "that seems high", and downgrades to A **without reading the grade reasoning or why_relevant** has produced damage, not value. The original grading agent had more information than you do if you skip the record.

**If you cannot read the full record for an entity, do not touch its grade.** Flag it for manual review instead.

---

## Company cross-checking procedure

### Step 1: Load the full universe

```sql
SELECT id, name, website, what_they_do, grade, grade_reasoning,
       why_relevant, status, sector_tags, location
FROM companies
WHERE grade IS NOT NULL AND status != 'archived'
ORDER BY grade, name;
```

Also load the full candidate profile (all `profile/` files). You need both to make any judgment.

### Step 2: Within-tier consistency

For each grade tier (S, A, B, C), review ALL companies at that tier and ask:

**"Would the candidate genuinely be indifferent between all companies at this tier?"**

If the answer is no — if one company is clearly a better career move than another at the same tier — one of them is misgraded. But before changing anything:

1. Read BOTH companies' `what_they_do`, `grade_reasoning`, and `why_relevant` in full.
2. Identify specifically what makes one stronger than the other: engineering reputation? sponsorship? technical alignment with flagship projects? career ceiling?
3. Decide which company is at the wrong tier and which direction it should move.
4. Write a justification that references both companies' records and the profile.

**Common within-tier errors:**
- A trillion-dollar tech company and a 10-person unfunded startup both at B
- Two companies in the same sector at the same tier, but one has confirmed sponsorship and the other doesn't
- A company whose `what_they_do` clearly describes work aligned with flagship projects, graded the same as one with no alignment

### Step 3: Cross-tier boundary verification

For every company near a grade boundary, ask:

**"Is this genuinely less valuable than everything in the tier above?"**

Pull the weakest companies from the tier above and the strongest from the tier below. Compare them directly:

```sql
-- Weakest S-tier companies (compare against strongest A)
SELECT name, grade, grade_reasoning FROM companies
WHERE grade = 'S' AND status != 'archived'
ORDER BY name;

-- Strongest A-tier companies (compare against weakest S)
SELECT name, grade, grade_reasoning FROM companies
WHERE grade = 'A' AND status != 'archived'
ORDER BY name;
```

**The "which offer would you take" test:** For any two companies at different grades, imagine the candidate has a graduate SWE offer from both. Would they take the higher-graded one? If the answer consistently contradicts the grades, the grades are wrong.

### Step 4: Specific red flags to check

These patterns have caused real grading errors in production. Check for each one:

| Red flag | What to look for | How to verify |
|----------|-----------------|---------------|
| **Famous employer at C** | Well-known tech companies graded C | Read grade_reasoning — is there a hard exclusion reason, or was it undergraded? |
| **Unknown startup at S** | Company with no brand recognition at the highest tier | Read grade_reasoning — does the technical alignment justify S despite weak CV signal? |
| **Sponsorship inconsistency** | Two similar companies at different grades, one sponsors and the other doesn't, but the non-sponsor is graded higher | Sponsorship should push the sponsoring company UP, not be ignored |
| **Tech stack as primary driver** | A company graded C because "they use Go not Rust" | Tech stack is a tiebreaker, not a primary driver. Re-examine. |
| **Stale grade reasoning** | `grade_reasoning` mentions facts that are no longer true, or references projects the candidate has since abandoned | Compare reasoning against current profile state |
| **Grade/reasoning mismatch** | The written reasoning describes an A-tier company but the grade says S, or vice versa | The grade may have been manually overridden without updating reasoning |

### Step 5: Output format for company cross-checks

For each issue found, output:

```
COMPANY CROSS-CHECK ISSUE:
  Company: [name] (current grade: [grade])
  Issue type: [within-tier inconsistency / boundary error / red flag]
  Evidence: [what you found — cite the grade_reasoning and what_they_do]
  Comparison: [which other company at which tier makes this grade wrong]
  Recommendation: [new grade] with justification
  Profile reference: [which profile elements inform this judgment]
```

---

## Job cross-checking procedure

### Step 1: Load jobs with full context

```sql
SELECT j.id, j.title, j.grade, j.fit_assessment, j.raw_description,
       c.name as company, c.grade as company_grade
FROM jobs j
JOIN companies c ON j.company_id = c.id
WHERE j.grade IS NOT NULL AND j.evaluation_status != 'archived'
ORDER BY j.grade, c.name, j.title;
```

**Critical:** For any job you plan to re-evaluate, you MUST also read its `raw_description`. The fit assessment is the agent's interpretation — the description is the ground truth. If you change a grade based only on the fit assessment without reading the description, you are grading on someone else's summary, not the actual job.

### Step 2: Company grade / job grade consistency

A graduate role at an S-tier company should rarely grade below A. If it does, there must be a specific reason in the fit assessment (hard seniority mismatch, customer-facing role disguised by title, wrong location).

```sql
-- Jobs at S-tier companies graded C or below
SELECT j.id, j.title, j.grade, j.fit_assessment, c.name
FROM jobs j JOIN companies c ON j.company_id = c.id
WHERE c.grade = 'S' AND j.grade IN ('C', 'F')
  AND j.evaluation_status != 'archived';
```

For each result:
1. Read the `fit_assessment`. Does it cite a hard exclusion (seniority, location, role type)?
2. If yes → grade is correct, the company being S doesn't save a fundamentally wrong role.
3. If no → read the `raw_description` to verify. The fit assessment may be wrong.

### Step 3: Within-tier job consistency

For SS and S tiers (the most important ones), compare all jobs at each tier:

**"Would the candidate genuinely accept any of these jobs with equal enthusiasm?"**

All SS jobs should feel like "drop everything and apply." If some feel more like "good, worth applying" — they're S, not SS.

All S jobs should feel like "genuinely excited." If some feel like "meh, decent" — they're A or B.

### Step 4: Cross-tier boundary verification for jobs

Pull SS/S boundary jobs and compare:

```sql
SELECT j.id, j.title, j.grade, c.name, c.grade as co_grade,
       substr(j.fit_assessment, 1, 500)
FROM jobs j JOIN companies c ON j.company_id = c.id
WHERE j.grade IN ('SS', 'S') AND j.evaluation_status != 'archived'
ORDER BY j.grade, c.grade, j.title;
```

**The "which offer" test:** For any SS job and any S job — if the candidate had offers from both, would they always take the SS? If an S job is clearly preferable to an SS job, one of them is misgraded.

### Step 5: Seniority verification

The most common job grading error is failing to check seniority requirements in the description. For every SS and S job:

1. Read the `fit_assessment`. Does it cite the description's experience requirements?
2. If not — the job was graded without reading the description. Read `raw_description` and check.
3. If the description says "5+ years" and the fit assessment says "entry-accessible" — the grade is wrong.

```sql
-- SS/S jobs — verify each has seniority citation in fit_assessment
SELECT j.id, j.title, c.name, j.grade, j.fit_assessment
FROM jobs j JOIN companies c ON j.company_id = c.id
WHERE j.grade IN ('SS', 'S') AND j.evaluation_status != 'archived';
```

### Step 6: Description-assessment consistency

For a random sample of 10-15 jobs across all tiers, read BOTH the `raw_description` AND the `fit_assessment`. Verify:

- Does the fit assessment accurately reflect what the description says?
- Are the seniority requirements correctly cited?
- Are the technical requirements accurately described?
- Does the profile alignment claimed in the assessment actually hold?

This catches cases where an agent wrote a plausible-sounding assessment that doesn't match the actual job.

### Step 7: Output format for job cross-checks

For each issue found:

```
JOB CROSS-CHECK ISSUE:
  Job: [title] @ [company] (ID: [id], current grade: [grade])
  Issue type: [company/job inconsistency / within-tier / boundary / seniority / description mismatch]
  Evidence: [what you found — cite fit_assessment and description]
  Comparison: [which other job at which tier makes this grade wrong, if applicable]
  Recommendation: [new grade or "verify — read full description"]
  Profile reference: [which profile elements inform this judgment]
```

---

## What this check does NOT do

- **It does not re-grade everything.** It identifies specific inconsistencies and presents them for review.
- **It does not enforce a target distribution.** If 30% of companies are S-tier and they all genuinely deserve it, that's correct. The check verifies each grade individually, not the overall shape.
- **It does not override hard evidence with vibes.** If a grade reasoning cites specific evidence (sponsor register, description quotes, project alignment) and you can't find counter-evidence, the grade stands even if it "feels" wrong.
- **It does not make changes without user approval.** Every recommended change is presented with evidence and justification. The user decides what to act on.

---

## Execution checklist

Before presenting cross-check results:

- [ ] All files in `profile/` were read before any comparisons
- [ ] Every company comparison was made after reading BOTH companies' full records (what_they_do, grade_reasoning, why_relevant)
- [ ] Every job grade change recommendation was made after reading the FULL raw_description, not just the fit_assessment
- [ ] Within-tier comparisons were done for every grade tier, not just S
- [ ] Cross-tier boundary checks were done at every boundary (S/A, A/B, B/C)
- [ ] The "which offer would you take" test was applied to flag obvious misorderings
- [ ] Seniority verification was done for all SS and S jobs
- [ ] No grade was changed without citing specific evidence from the entity's record AND the profile
- [ ] Results are presented as recommendations, not executed changes
