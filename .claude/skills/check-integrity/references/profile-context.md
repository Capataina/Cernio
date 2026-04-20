# Profile Context for Integrity Checking

**This file does not contain the candidate's profile. It tells you how to read the profile for integrity assessment.**

The candidate's profile is maintained across files in `profile/`. You must read ALL of them before performing integrity checks. The profile is the reference point for judging whether grades and evaluations are still accurate.

---

## What to extract from the profile for integrity checking

### Recent changes (most important for staleness detection)

When checking for profile-driven staleness, focus on what has CHANGED since grades were last assigned:

| Change type | Where to find it | Impact on existing grades |
|-------------|-----------------|--------------------------|
| New project added | `profile/projects.md` — check for entries not reflected in grade reasoning | Companies/jobs graded before this project may underestimate alignment |
| New skill acquired | `profile/skills.md` — check for new entries or upgraded proficiency levels | Companies dismissed for "lacking X skill" may now deserve higher grades |
| Preference changes | `profile/preferences.toml` — check for changed sectors, locations, or constraints | Companies/jobs filtered by old preferences may need reassessment |
| Visa status change | `profile/visa.md` — check expiry date, sponsorship requirements | Sponsorship weighting may need updating across all grades |
| Experience update | `profile/experience.md` — new work history | Seniority assessments may need updating — roles previously "too senior" may now be achievable |

### Profile elements that grade reasoning should reference

When auditing grade quality, check whether existing assessments reference these elements:

**Must reference (for any grade):**
- At least one project from `projects.md` by name
- At least one technology from `skills.md`
- Seniority assessment grounded in `experience.md`

**Must reference (for S/SS):**
- Multiple relevant projects with what each demonstrates
- Full technology overlap analysis
- Visa timeline and sponsorship viability
- Career trajectory against targets in `preferences.toml`
- Gaps from `portfolio-gaps.md`

If an assessment doesn't reference these elements, it fails the quality check and should be flagged for rewrite.

---

## How profile changes affect different grade dimensions

| Profile change | Affected dimension | How to assess impact |
|---------------|-------------------|---------------------|
| New systems project | Technical alignment | Does the new project make a previously weak alignment stronger? |
| New language learned | Tech stack relevance | Does the company use this language? Were they downgraded for stack mismatch? |
| New cloud/infra project | Portfolio gaps | Does this close a gap that was cited in grade reasoning? |
| Visa expiry approaching | Sponsorship weight | Does the urgency change how heavily sponsorship should be weighted? |
| Location preference change | Location assessment | Do previously excluded locations now qualify? |
| New certification | Career ceiling | Does this open doors at companies that require specific credentials? |
