# Job Grading Rubric Design

The rubric for evaluating jobs was designed around one question: what maximises long-term career trajectory for an entry-level systems engineer aiming for £500K+ income over a career?

---

## Why grades instead of fit/no-fit

A binary "fits / doesn't fit" misses the nuance that matters. A role at Palantir that's a slight reach is worth more than a perfect-fit role at a no-name agency. The grading system (SS through F) captures career value, not just skill match.

---

## The critical dimensions

**Career ceiling** and **seniority match** are non-negotiable. A role with no upward trajectory is an F regardless of how well the tech stack matches. A role you can't get hired for is also an F regardless of how perfect it looks.

**Skill breadth** matters more early in career than later. A broad infrastructure role that touches backend, data, and ops gives more career options at year 3 than a narrow role maintaining one microservice.

**Company signal** is disproportionately important for a first job. "Palantir" or "Stripe" on a CV opens doors that "RandomStartup Ltd" cannot. This premium decreases for second and third jobs.

**Sponsorship viability** has a hard deadline (August 2027). Companies that clearly sponsor are worth more than those that "might consider it."

---

## Grades map to actions

- SS/S → apply, full detailed evaluation
- A/B → consider, evaluation on request
- C/F → skip unless desperate, one-line reason

This means the user's daily view in the TUI is a prioritised list, not a wall of equal-weight results. SS jobs surface first, F jobs are invisible unless asked for.

---

## The rubric evolves

As we evaluate more jobs, patterns will emerge. If every A-grade role has the same gap (e.g. "requires Kubernetes"), that's a portfolio gap worth closing. The rubric feeds the career coaching loop.

Full rubric details: `skills/grade-companies/references/grading-rubric.md` (companies) and `skills/grade-jobs/references/grading-rubric.md` (jobs). The `search-jobs` rubric is legacy and should not be used — the dedicated grading skills have more comprehensive rubrics with worked examples, boundary cases, and evidence standards.

---

## Question-first rewrite (session 4, 2026-04-09)

Both rubrics were completely rewritten from dimension-weighted scoring to a question-first approach:

**Old approach:** Score each dimension independently (career ceiling 4/5, tech stack 3/5, sponsorship 3/5...) then combine. This produced mechanical, generic assessments — agents would assign middling scores to everything and arrive at a B without genuine reasoning.

**New approach:** Start with core questions that force thinking about what the company/role actually means for this candidate. "Would you recommend a friend apply here — and what would you warn them about?" The answers ARE the evaluation. Dimensions are then used as analytical support to add precision and catch blind spots, not as the primary scoring mechanism.

Additionally, both rubrics now require **mandatory description citation** — fit assessments must quote specific phrases from the job description or company information to support the grade. "Good tech stack" is no longer acceptable; "uses Rust for low-latency trading infrastructure (quoted from description)" is the standard.

This change addresses the pattern where session 3 grading produced technically correct but shallow assessments that could have applied to any company or job.
