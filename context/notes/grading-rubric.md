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

---

## Full database reset (session 5, 2026-04-09)

All jobs (712), user decisions (120), and company metadata wiped. The 273 company names, websites, statuses, and 142 ATS portal records were preserved — this is the expensive resolution work from earlier sessions. 47 wrongly archived companies (from the TUI cleanup bug) were unarchived.

**Why:** Company metadata (`what_they_do`, `why_relevant`, grades) suffered the same title-only problem as job grades. Descriptions were one-liners ("trading firm specialising in derivatives"), relevance fields were empty, and grades were assigned without proper research. Google at A-tier while Akuna Capital was S-tier. Apple at A. The metadata was too broken to patch — faster to rebuild from scratch with proper enrichment.

**What changed in grade-companies skill:** Company grading now also writes enriched `what_they_do` (3-5 sentence paragraph, no stale info), `location`, `sector_tags`, and profile-grounded `why_relevant` alongside the grade. This was previously handled superficially during population. The grading agents are already researching each company — routing that knowledge into the description field is zero extra work.

**Rebuild sequence:** Enrich+grade all 273 companies → search jobs with updated filters → grade jobs with new rubric.

---

## Exclusion keyword purge (session 6, 2026-04-09)

Deleted 1,064 jobs matching exclusion keywords from the database (both active and archived). The DB went from ~2,001 to 937 jobs. This was a bulk retroactive application of the 34 exclusion keywords added in session 5 — the keywords had only been applied to new search results, not to the existing corpus.

Also in session 6: all 167 potential companies were resolved via 8 parallel agents. 64 resolved to supported ATS, 98 marked bespoke with careers URLs, 5 dead/duplicate archived then deleted (Eisler Capital shut down, Nivaura acquired, OpenSSF podcast not employer, Qatalog acquired by ClickUp, Oxbotica duplicate of Oxa). Final company count: 408 (287 resolved, 121 bespoke, 0 potential).

---

## New exclusion keywords: Sr./Lead (session 7, 2026-04-09)

Added "Sr.", "Sr ", and "Lead" to the exclusion keyword list. These were missing from the original 34 keywords, causing 51 senior/lead-level jobs to leak through the search filter and appear as pending jobs.

**Evidence for exclusion:** After grading all 206 pending jobs, zero senior/lead roles received B+ or higher grades. Every single one was graded B or below because the seniority requirements (typically 3-8 years experience) are incompatible with the profile's 1 year of professional experience. These roles waste grading tokens and clutter the pipeline.

The 51 leaked jobs were archived after grading. The exclusion keywords are now applied during `cernio search` so future search runs will filter them out before insertion.

---

## Lifestyle and environment fit as part of the grading rubric (session 8, 2026-04-10)

Added `profile/lifestyle-preferences.md` alongside distributed additions to `personal.md` (worldview, height, presentation), `visa.md` (mobility mindset and hard exclusions), `languages.md` (language-learning flexibility), `interests.md` (café work, evening walks, gym aspiration, frontier-tech-as-user), `preferences.toml` (positive/negative signals), and `portfolio-gaps.md` (Geographic Patterns section). When grading, read all of these alongside the existing profile files — the lifestyle signals are distributed on purpose so that no single file carries the whole picture, and so that an agent that misses one file still picks up most of the context from the others.

### The coherence insight

The lifestyle preferences are **correlated, not independent**. They all derive from a single underlying profile: *cosmopolitan modernist at the frontier* — someone who wants to live where humanity is visibly building the future, with the personal operating infrastructure (cafés, gyms, safe streets, integrated greenery) that supports high performance, in a culturally progressive environment that doesn't impose friction on how he lives day to day.

**Consequence for grading:** do not score the lifestyle axes independently and sum them up. Look for **overall pattern match**. A city matching the overall profile well across several axes is a good fit even if one axis is weak. A city matching the profile on only one or two axes is a weak fit even if one of those axes scores very highly. This is a reasoning exercise, not an arithmetic one.

**The underlying pattern in practice:**

- "Modern" means *proximity to frontier technology*, not architectural taste. A beautiful historic city fails because the future doesn't arrive there first. A modern-looking city with weak tech adoption also fails partially.
- Café culture, walks, and integrated greenery are *personal operating infrastructure*, not lifestyle luxuries — see `interests.md`. Cafés soothe ADHD focus, walks are decompression, green spaces are where Caner takes his laptop between work sessions. Unsafe streets invalidate all of these simultaneously, which is why safety is load-bearing across the whole pattern rather than being one axis among equals.
- Nightlife and active social scenes are recreational access, not identity. Cities get credit for offering an active, vibrant after-hours scene, but they are not penalised for not forcing Caner into it. He's an engineer who wants the option on weekends, not a party-first lifestyle.
- Secular, progressive, and integration-minded are one bundle — see the worldview field in `personal.md` and the integration mindset in `visa.md`.
- Rigidity is physical, not ideological. Where Caner is inflexible (Amsterdam height, Jordaan low-rise scale, Croydon crime, Gulf religious backdrop), the friction is with the environment itself, not the idea of being there. Everywhere else he's flexible.

### How to weave lifestyle fit into grading

Lifestyle fit is a **low-to-medium weight factor** that sits alongside the primary dimensions of career ceiling, sponsorship viability, skill match, and company signal. Career still dominates — a top career opportunity in a lifestyle-imperfect city should grade above a mid opportunity in a lifestyle-perfect one.

The grader's job is to **reason through the fit**, not to mechanically adjust grades. In practice this means:

- **Read the profile files and form an overall impression of lifestyle fit** alongside the technical impression. "Is this role in a place where Caner would actually thrive?" is the question — the answer is not a number.
- **Let that impression inform the grade** in the same way that "strong company signal" or "weak sponsorship track record" inform the grade: as one of several factors woven into the reasoning, not as a separate arithmetic adjustment.
- **Pattern-match across the profile, not axis-by-axis.** Use the calibration anchors table in `lifestyle-preferences.md` as a reference point for how the whole package of axes interacts. A city that looks similar to "London Kings Cross" on the pattern is probably a good lifestyle fit; a city that looks similar to "London Croydon" is probably a poor one.
- **Weave the observation into the assessment, don't bolt it on.** Good example: "Office is at Google Kings Cross — the mixed-scale regenerated district Caner explicitly calls the gold standard, with the rooftop atrium and integrated café culture. This is a strong environmental match that compounds the already-strong career fit." Bad example: "Lifestyle is A+, so adding half a tier."
- **Flag mismatches explicitly.** If a mechanically strong role is in a city that fails several anti-preferences at once, the fit assessment should surface this so Caner can make an informed decision rather than being blindsided. The grader's job is to make the trade-off visible, not to hide it.

### Known pitfalls to avoid

1. **Do not re-recommend Amsterdam** or Jordaan-equivalent low-rise historic cities even when the mechanical firm fit is perfect. Optiver, IMC, and Flow Traders are top-tier matches on paper, but Amsterdam is explicitly excluded on aesthetic and physical-scale grounds. This was the case study that motivated the coherence insight — I made the mistake of recommending it once and should not repeat it.
2. **Do not inflate grades for visa-blocked aesthetic-match cities.** Seattle, Toronto, and SF are aesthetic gold-standard fits, but the H-1B lottery (US) and Express Entry structural bias (Canada) make them low-probability outcomes. Grade on realistic expected value, not aspirational fit. Cross-reference `visa.md` before letting lifestyle fit pull the grade up.
3. **Do not downgrade a top career opportunity on tertiary lifestyle axes alone.** A strong career fit in Singapore should not drop because the evening social scene is less lively than NYC.
4. **Canary Wharf roles need specific nuance.** The distinction between "gold standard modern" (Nine Elms, Kings Cross) and "pure skyscraper canyon" (Canary Wharf) is real and was made explicitly by Caner. Do not conflate them — visually modern does not automatically mean lifestyle-fit.
5. **Berlin's social scene should not be downgraded.** The user's stated answer dismissed German nightlife generally, but Berlin specifically has a significantly larger and more active after-hours scene than the rest of Germany — substantially more so than his stated answer implied.
6. **Safety is load-bearing, not a tertiary axis.** If an office is in an actively unsafe area (current Croydon is the live calibration point — the user cannot walk there at night), this invalidates the entire personal-infrastructure stack simultaneously (evening walks, late-night café work, post-dinner decompression, gym at 10pm). Treat unsafe office areas as a meaningful negative factor in the assessment, not a small footnote.

### Where the lifestyle signals live across the profile

The distribution is deliberate — no single file carries all of it, so an agent that misses one file still picks up most of the context from the others:

- `profile/lifestyle-preferences.md` — urban aesthetic, nightlife, safety, climate, calibration anchors (the content with no other natural home)
- `profile/personal.md` — height, presentation, worldview (identity-level facts that happen to be grading-relevant)
- `profile/interests.md` — café work, evening walks, gym, thunderstorms, frontier-tech-as-user (daily habits)
- `profile/visa.md` — mobility mindset, integration mindset, hard exclusions from nationality
- `profile/languages.md` — language-learning flexibility and the B2-German push as a closure opportunity
- `profile/preferences.toml` — machine-readable positive/negative signals for the search pipeline
- `profile/portfolio-gaps.md` — Geographic Patterns tier table and closure opportunities

When grading, read at minimum: `preferences.toml` (hard filters), `lifestyle-preferences.md` (overall picture and calibration anchors), `personal.md` + `visa.md` (constraints), and cross-reference `interests.md` where daily-habit friction is relevant.

---

## Session 7 grading run (2026-04-09)

Full grading of 206 pending jobs (255 from automated search + 13 bespoke - duplicates and previously graded). Results:

| Grade | Count | % |
|-------|-------|---|
| SS | 13 | 2.7% |
| S | 27 | 5.6% |
| A | 70 | 14.4% |
| B | 142 | 29.2% |
| C | 20 | 4.1% |
| F | 212 | 43.7% |

Notable SS additions: Citadel Graduate Programme, Bloomberg 2026 SWE, Google Graduate 2026. The bespoke search for 39 S/A-tier companies yielded 13 jobs — these tend to be higher quality because the companies were pre-screened.

DB state post-session: 408 companies, 1184 jobs (484 graded non-archived), 0 pending.
