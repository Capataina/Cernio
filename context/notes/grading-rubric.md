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

---

## Project tier system (session 5, 2026-04-09)

Added a `Tier` field to every project in `projects.md`: Flagship, Notable, or Minor. Previously, grading agents treated all 16 projects as equal evidence — citing an abandoned particle sim website with the same weight as a lock-free matching engine. The tier system tells agents which projects represent the candidate's strongest, deepest work.

| Tier | Count | Examples | Evidence weight |
|------|-------|----------|----------------|
| **Flagship** | 5 | Nyquestro, NeuroDrive, Aurix, Cernio, Image Browser | Primary evidence — cite these by name in fit assessments |
| **Notable** | 5 | Vynapse, Xyntra, Tectra, AsteroidsAI, Game Modding | Supporting evidence — cite when directly relevant |
| **Minor** | 6 | Consilium, Zyphos, Chrona, Neuronika, Credit Card Fraud, Personal Website | Weak evidence — mention only if specifically relevant |

Both grading rubrics updated: the job rubric's Question 3 ("does the candidate's background give them an edge?") now instructs agents to weight projects by tier and status. The company rubric's technical alignment dimension now states that alignment with flagship projects is strong evidence while alignment only with minor/abandoned projects is weak. Both profile-context reference files updated to reflect tiers in technical identity synthesis and evidence standards.

Also fixed stale statuses: Xyntra and Tectra changed from "In Progress (not enough interest)" to "Abandoned". Vynapse status updated to reflect low ROI. Game Modding marked as historical.

---

## Calibration-anchored grading replaces batch-relative grading (session 5, 2026-04-09)

The original relative grading system told agents to compare within their batch and enforce a reasonable distribution. This was fundamentally broken because the prioritisation system (grade S-company jobs first) ensures every batch is skewed toward high-quality roles. An agent seeing 20 excellent jobs would think "these can't all be S" and deflate grades that were correct.

**Old approach:** Grade each job, then compare within the batch. "Do all these B-tier jobs genuinely belong together?" This assumed a representative batch, which the prioritisation system guarantees you never have.

**New approach:** Before grading starts, pull 2-3 real examples at each grade tier (SS, S, A, B, C, F) from the existing database as calibration anchors. Grade each job against those anchors individually — "does this belong alongside the SS examples or the A examples?" Within-batch comparison becomes a consistency check (did I grade two similar jobs differently?) rather than a distribution enforcer (I need some C's in here).

This also fixes re-assessment batches: when re-grading all SS/S/A jobs, agents won't deflate them just because the batch contains only high-quality roles.

**Bootstrapping problem acknowledged:** The initial calibration anchors are drawn from the old shallow assessments. The anchors are imperfect but still useful for grade-level calibration (what kind of role deserves SS vs A). As re-graded jobs replace the old ones, the calibration quality improves automatically.

Applied to both company and job grading rubrics, both SKILL.md files, and the prioritisation guide.

---

## Session 5 regrading results (2026-04-09)

Re-assessed all 146 SS/S/A jobs that had descriptions (11 G-Research jobs with no descriptions were skipped). Results confirmed the original session 3 grading was title-only — agents never read job descriptions.

**Distribution change across the 157 re-assessed jobs:**

| Grade | Before | After | Change |
|-------|--------|-------|--------|
| SS | 12 | 7 | -5 |
| S | 55 | 19 | -36 |
| A | 90 | 24 | -66 |
| B | 0 | 25 | +25 |
| C | 0 | 59 | +59 |
| F | 0 | 23 | +23 |

30 unchanged, 7 promoted, 120 demoted (corrections).

**Why the old grades were wrong:** The original ~700-job grading run on the 5x plan couldn't afford to read descriptions — doing so would have consumed ~230% of the plan's token budget. Agents pattern-matched on company name + job title instead. Evidence: identical one-line assessments across dozens of jobs (`"Engineering role at G-Research, decent alignment"`), S-tier grades on roles requiring 5+ years experience, and S-tier grades on Bengaluru-only roles.

**What the new rubric fixed:**
- Mandatory description citation prevents title-only grading structurally
- Project tier awareness correctly downweighted fits based on abandoned projects (e.g. Xyntra)
- Calibration anchors prevented batch-level grade deflation
- 20 Cloudflare Senior roles with 3-6 year requirements correctly demoted to C
- 10 Cloudflare Bengaluru-only roles correctly demoted to F
- 9 Faculty AI roles at a C-tier consulting company correctly demoted from A to C/B
- 4 genuinely accessible roles promoted (Parity Rust dev A→S, Squarepoint accessible quant roles A→S)

**Token economics lesson:** Proper job grading (reading descriptions, answering 5 questions, citing evidence) costs roughly 0.08% of a 20x plan per job. Budget accordingly — ~700 jobs would need ~56% of a 20x plan session. The old approach of grading without reading was cheap but produced random results.
