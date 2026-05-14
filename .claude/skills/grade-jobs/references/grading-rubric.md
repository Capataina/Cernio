# Job Grading Rubric

The evaluation framework for grading individual jobs. Job grading is where the real decision happens — company grading determines what we search, but job grading determines what we apply to. Every grade should emerge from careful reasoning about this specific role for this specific candidate, not from mechanically scoring dimensions.

**Important:** All profile facts must come from reading `profile/` files — never from hardcoded values. When this rubric says "the candidate's portfolio" or "the visa timeline," it means: read the per-project files in `profile/projects/`, read `visa.md`, etc.

---

## Table of Contents

1. [How to Grade a Job](#how-to-grade-a-job)
2. [The Core Questions](#the-core-questions)
3. [Analytical Dimensions](#analytical-dimensions)
4. [Grade Scale](#grade-scale)
5. [Cross-referencing and Relative Grading](#cross-referencing-and-relative-grading)
6. [Career-Stage Context](#career-stage-context)
7. [Common Grading Errors](#common-grading-errors)
8. [Worked Examples](#worked-examples)
9. [Evidence Standards for Fit Assessments](#evidence-standards-for-fit-assessments)

---

## How to Grade a Job

Grading happens in five steps. Each step informs the next. The grade letter is the *aggregated output* of the reasoning across these steps, not a categorical lookup keyed to a single dimension.

**Step 1: Answer the core questions.** These force pragmatic thinking about what this role actually means for the candidate's career. Write 2-3 sentences per question. The answers ARE the evaluation — everything else supports them.

**Step 2: Evaluate against the analytical dimensions.** The dimensions add precision and structure. They catch things the questions might miss — a role that feels right but has a hidden seniority wall, or one that feels wrong but actually has exceptional career ceiling.

**Step 3: Aggregate the reasoning into a letter — Q1 is the primary lens, not one of five equal axes.** Q1 (achievability — can the candidate realistically be hired) is the lens through which every other question is interpreted. A role's CV value (Q2), the candidate's edge (Q3), the day-to-day work (Q4), and practical constraints (Q5) are all refinements *within* the Q1-determined band. When Q1 reads as a real headwind — implicit selectivity, sub-1% conversion, hiring patterns the candidate's profile shape doesn't clear — Q2-Q4 strengths refine *which* below-the-line letter the role lands at, not whether it crosses back above the line. When Q1 reads as cleared (the candidate is in the realistic primary-target pool), Q2-Q4 then determine whether the role lands at SS, S, A, or B. The aggregation is judgement, not arithmetic — but the order is fixed: Q1 first, then the rest. The grade letter must be consistent with the assessment's own Q1 verdict; an assessment that names sub-1% conversion cannot produce an A or above. The role for the prestige-trap pattern is to *down-weight Q2/Q3/Q4 strength* when Q1 is weak, not to introduce a separate "stretch" sub-tier.

**Step 4: Cross-reference questions and dimensions.** Do they agree? When they conflict, reason through why. The critical-dimension F-forcers (seniority match, career ceiling) and Q1 work together — Q1 names *whether* the candidate is in the applicant pool; the critical dimensions name *what* puts them outside it. If both signal trouble, the role is F or C regardless of Q2-Q4 quality.

**Step 5: Calibration check.** Compare this job against the calibration anchors — real examples from the database at each grade tier. Does it belong alongside the examples at the grade you've assigned? If it's clearly stronger than the A-tier anchors but weaker than the SS anchors, it's S. Do NOT compare against other jobs in the current batch to enforce a distribution. Do NOT use the rubric's general distribution shape (the rough proportions of SS / S / A / B / C / F roles) as a target to fill — for narrow-profile candidates with strict location / visa / seniority constraints, the realistic SS/S/A pool is naturally smaller, and the right response is fewer roles at the top, not promotion of stretches to fill perceived gaps.

The fit assessment written to the database should be the output of this process — the actual reasoning, not a summary. Q1 reasoning leads the assessment narrative; Q2 reasoning follows; Q3-Q5 close. This ordering surfaces the dominant axis in the visible text so the grade letter and the narrative cannot drift apart.

---

## The Core Questions

For every job, answer these five questions. These are the evaluation. The dimensions provide analytical support.

### 1. Can the candidate actually get this job?

This is the achievability question. It doesn't matter how perfect a role is if the candidate can't get hired.

**Read the description, not the title.** "Senior" at many UK companies means 2-3 years out of university. "Staff" means genuinely senior. The title lies; the description tells the truth.

Look for:
- Explicit years of experience requirements — hard requirement or preference?
- Scope of responsibility — "own a component" vs "own the architecture of the platform"
- Expectations of managing others — "mentor junior engineers" presupposes experience
- Production expectations — "incident management experience" presupposes operational maturity
- "Or equivalent demonstrated ability" — this is an opening for strong portfolios

Read `experience.md` for formal work history and the per-project files in `profile/projects/` for demonstrated capability. The portfolio can substitute for 1-2 years of professional experience if the projects demonstrate genuine depth — but it cannot substitute for 5 years of production systems ownership.

**Beyond what the description states, weigh realistic landing probability.** A description that does not gate on years can still belong to a role the candidate cannot realistically get hired into. Some firms hire single-digit graduates per year out of thousands of strong applicants, and that selection pressure is functionally a credential filter even when no number appears in the description. Reason about whether the candidate is in the realistic applicant pool given the firm's hiring patterns, the role's competition density, and the candidate's overall profile shape — not just whether the description gates them out on paper. The fit assessment makes this reasoning visible: name the selection-pressure signal you observed (e.g. "graduate intake is the firm's only entry route and is known for selectivity at the comp-programming / top-university tier"), then state how it weighs against the candidate's profile. This is judgement, not arithmetic — there are no firm lists, no thresholds, no caps. The grader applies the same kind of realistic reasoning a thoughtful career advisor would use when looking at the same role-and-candidate pair.

**Reputation and selectivity are independent axes — do not conflate them.** A firm being reputable, well-known, or impressive on a CV (Q2 territory) says nothing on its own about whether the candidate can realistically be hired (Q1 territory). The two axes decouple in both directions:

- **Reputable AND accessible.** Large companies with structured graduate, new-grad, or intern pipelines often hire hundreds of entry-level engineers per cycle, accept a wide range of universities and degree classifications, and have established sponsorship infrastructure. A reputable firm with a high-volume graduate pipeline that genuinely takes portfolios like the candidate's is a realistic primary target — strong Q2 and strong Q1, both. These belong at the top of the SS/S list, not lower because of "they're a big name."
- **Reputable AND brutal.** Firms whose graduate pipeline is narrow and selectivity-heavy (small intake counts, heavy filters on competitive-programming pedigree, top-university recruiting concentration) have strong Q2 but weak Q1 for candidates outside that filter shape. These belong at A/B as stretches, not at SS, regardless of how impressive the name reads.
- **Less-reputable AND accessible.** Mid-tier firms with genuine entry-level pipelines and engineering-led culture are often realistic primary targets too — Q1 strong, Q2 moderate. These can land SS/S when the role is specifically aligned with the profile.
- **Less-reputable AND brutal.** Small firms hiring for one role with a niche credential filter — uncommon, but the realistic-conversion analysis still applies.

The detection rule for the grader: when reasoning about Q1, ignore Q2's signal entirely. Q2 answers "would this look good on a CV?" Q1 answers "does this firm hire people who look like this candidate, in volumes that make conversion plausible?" Conflating them is the exact failure mode the realism semantic exists to prevent.

**If the answer to Q1 is clearly no — whether through an explicit credential floor (hard 5+ years requirement, staff/principal scope, leadership expectations, PhD required) or through implicit selectivity that puts the candidate sub-1% on realistic conversion — the grade is F or C depending on how brutal the gap is. No other question matters when Q1 fails.** This is the only question that can unilaterally determine the grade. A role where Q1 reads as a real headwind (genuine non-zero conversion but the candidate is outside the realistic primary-target pool) aggregates down to C through the Q1-primary lens described in §How to Grade a Job Step 3, regardless of how strong Q2-Q4 read in the absolute frame — see the §Grade Scale §A and §C definitions below for how the aggregation produces the letter without sub-categories.

### 2. Would this be a good first line on the candidate's CV?

This combines company signal and role quality into one practical question. Think about how this looks to the next employer:

- "Graduate Software Engineer, Infrastructure — Cloudflare" → opens every door
- "Software Engineer — Bloomberg" → instantly credible
- "Backend Engineer — Monzo" → strong UK signal
- "Junior Developer — Unknown Agency Ltd" → raises questions
- "Solutions Architect — [Any Company]" → "so... pre-sales?"

The company name, the role title, and what the candidate would actually learn all contribute. A mediocre title at a great company can be better than a great title at a mediocre company, especially for a first job.

### 3. Does the candidate's background give them an edge?

This is where the profile matching actually matters — not as an abstract dimension score, but as a practical question: would the candidate's specific projects, skills, and experience make them a standout applicant for this role?

**Q3 presupposes Q1 has been cleared.** "Standout applicant" only meaningfully applies after the candidate is actually in the applicant pool — and whether they are is Q1's job. A strong Q3 answer cannot rescue a failed Q1: a candidate whose portfolio is perfectly aligned but who is not in the realistic applicant pool for the firm is not a "standout applicant," they are not an applicant the firm will see. When Q1 indicates implicit selectivity that the candidate's profile shape doesn't clear, weigh Q3 inside the realistic-applicant frame (against the firm's actual hire profile) rather than the absolute frame (against typical CS graduates).

Read each per-project file in `profile/projects/` carefully. Every project file has a `status` frontmatter field. The status determines the evidence weight:

**Status weighting determines evidence depth:**

| Status | Meaning | How to use in fit assessment |
|--------|---------|------------------------------|
| **active** | Currently being built / actively invested in | Primary evidence. Cite by name when assessing fit. A role that aligns with 2-3 active projects is one where the candidate has a genuine edge. |
| **paused** | Real work invested, paused but not abandoned | Secondary evidence. Cite when directly relevant; don't build the entire fit case on these. |
| **dormant** | Substantive but not currently active. Includes projects that were substantively built and finished but are no longer the candidate's focus. | Secondary evidence. Cite when directly relevant. The candidate's depth is real, but the project is not the current investment signal an active project would be. |
| **abandoned** | Started but not followed through | Background context only. Never use as primary evidence of capability. An abandoned particle sim does not demonstrate the same thing as a lock-free matching engine. |

A role that aligns with 2-3 active projects (or active plus substantively-built dormant projects) is one where the candidate has a genuine edge over typical applicants. A role where only abandoned projects are relevant means the candidate is competing without their strongest evidence — that's a weaker fit even if the technology nominally matches.

**Each per-project file describes what the project demonstrates** — its technologies, domain, scope, and what kind of engineering it shows. Read those files (and `profile/projects/index.md` for the inventory) rather than relying on hardcoded project summaries; the per-project files are the source of truth and evolve over time.

Also check `portfolio-gaps.md` — does this role require something the profile explicitly lacks? A gap in a "nice to have" is different from a gap in a core requirement.

### 4. Would the candidate enjoy the day-to-day work?

Not "is it systems engineering" in the abstract, but "would the candidate find this specific work interesting for 2 years?"

**Read the ENTIRE profile — not just projects.** The per-project files in `profile/projects/` show what the candidate CAN build. `interests.md` shows what the candidate WANTS to build. These are not always the same. The candidate might have zero health projects but a deep interest in the intersection of AI and health — a health platform engineering role could be highly engaging despite no portfolio evidence. Similarly, `preferences.toml` captures sector preferences and `cover-letter.md` reveals how the candidate frames their motivations.

What genuine engagement looks like (from the full profile):
- Building from scratch rather than configuring existing tools
- Performance-critical systems with measurable latency targets
- Financial/trading domain problems
- ML infrastructure and compiler engineering
- Problems where correctness and efficiency both matter
- Cross-disciplinary work connecting engineering to science, health, or finance
- Any domain the candidate has expressed genuine interest in, even without a matching project

A role writing Kubernetes YAML files is "infrastructure" but it's not the kind of infrastructure the candidate finds engaging. A role building a custom database engine is deeply engaging even if the company is unknown. A role building health monitoring infrastructure might be deeply engaging because of personal interests, even though no health project exists in the portfolio.

"Interesting enough, with excellent other factors" is a valid answer. "I would actively dread this work" is a signal that matters even if everything else looks good.

**Read `lifestyle-preferences.md` for office-environment fit.** The day-to-day work includes the daily environment, not just the technical content. Lifestyle preferences names the candidate's calibration anchors for office neighbourhoods (e.g. Kings Cross / Nine Elms / Paddington Basin as the gold standard, Canary Wharf as the mixed-scale partial fit, Croydon and similar outer-ring areas as active negative signal), urban aesthetic preferences, café-as-workspace culture, safety thresholds, and nightlife / secular-public-culture preferences. The office neighbourhood is a stronger signal than the city — a London role at Kings Cross plays differently from the same role in a Croydon office park. Weave lifestyle observations into the Q4 reasoning the same way you weave technical fit; cite the specific anchor (e.g. "matches the Kings Cross gold standard" or "fails the Canary Wharf mixed-scale test") rather than handwaving with "good area" / "nice neighbourhood." Lifestyle is a low-to-medium-weight modulator within tiers, not a standalone score and not a hard filter — it shifts grades within a tier and across boundary cases, but it does not override Q1 or Q5 hard exclusions.

### 5. Does this solve the candidate's practical constraints?

The unglamorous but critical question:
- **Location:** Is it in London, Cambridge, or Remote-UK? (Read `preferences.toml`)
- **Office area:** Within an acceptable city, where specifically is the office? Read `lifestyle-preferences.md` for the candidate's office-neighbourhood anchors. The same London role grades differently when the office is at Kings Cross / Nine Elms / Paddington Basin (gold-standard anchors) versus Canary Wharf (modern but mixed-scale-failure anchor) versus Croydon or similar outer-ring areas (active negative-fit anchor). This is a within-tier modulator, not a hard filter — but a role at a positively-anchored neighbourhood should be cited as a strength, and a role at a negatively-anchored neighbourhood should be cited as a friction worth flagging in the fit assessment.
- **Sponsorship:** Can and will this company sponsor when the Graduate visa expires? (Read `visa.md`)
- **Excluded types:** Is this actually a consulting role, customer-facing role, or support role disguised by the title? (Read `preferences.toml` for hard exclusions)
- **Excluded sectors:** Gambling, adtech, consumer-crypto? (These are hard exclusions — no amount of technical alignment overrides them)

A role that fails on a hard exclusion is F regardless of everything else. A role with uncertain sponsorship is still valuable within the visa window but the grade should reflect the time limit.

---

## Analytical Dimensions

These add precision to the question-based reasoning. They are not a replacement for thinking — they are a tool for catching blind spots.

### Critical (can force F on their own)

| Dimension | What to assess |
|-----------|---------------|
| **Seniority match** | Can the candidate realistically get hired? Based on `experience.md` and the per-project files in `profile/projects/`, not the title. |
| **Career ceiling** | Does this domain lead to high-income, high-impact positions at 10-15 years? Read `preferences.toml` for targets. |

### High weight

| Dimension | What to assess |
|-----------|---------------|
| **Skill breadth** | Multiple technical layers or locked into one narrow thing? Breadth matters more early in career. |
| **Company signal** | Does this company name open doors? Read `experience.md` — for a first job, this is disproportionately important. |
| **Technical depth** | Genuinely hard problems or routine work? Performance-critical, distributed, algorithmic? |
| **Sponsorship viability** | Can they sponsor? Read `visa.md` for timeline. Large companies with grad programmes almost always can. |

### Medium weight (tiebreakers)

| Dimension | What to assess |
|-----------|---------------|
| **Domain transferability** | Skills useful elsewhere, or company-specific? |
| **Growth environment** | Strong engineers, mentorship, code review culture? |
| **Tech stack relevance** | Technologies from `skills.md`? This is a TIEBREAKER. Languages are learned in months. |

---

## Grade Scale

| Grade | Meaning | How it should feel |
|-------|---------|-------------------|
| **SS** | The best role realistically reachable for this candidate. Apply immediately, prioritise above all. | "This is genuinely my best shot — strong technical fit AND I have a real chance of landing it." Q1 is cleared decisively (the candidate is in the realistic primary-target pool, not just nominally eligible), AND Q2-Q4 are strong. SS is "best for this candidate" — never "most prestigious role in the candidate's interest space." A role with perfect technical alignment but sub-1% conversion is not SS regardless of the company's prestige. |
| **S** | Strong candidate where the candidate has a real chance. Apply with energy. | "I'd be excited AND I have a real chance." Q1 is genuinely cleared — not "perhaps if everything goes well" but "this firm hires people with this profile in volumes that make conversion plausible." Q2-Q4 are mostly strong, maybe one moderate. A clear career-positive move with realistic conversion. |
| **A** | Worth applying to. The aggregation places the role in the third band of realistically-landable opportunities. | "This is good — I'd apply if I have time." Q1 is cleared, but Q2-Q4 collectively have at least one notable weakness — moderate CV value, adjacent rather than core domain alignment, or a couple of medium-weight gaps. A is a single coherent band: roles whose aggregation across Q1-clearance plus Q2-Q4 lands them above B but below S. Roles with strong Q2/Q3/Q4 but weak Q1 are NOT A — they aggregate down through the Q1-primary lens to the band that matches their actual landability, regardless of how impressive the company or technical fit would be in an absolute sense. |
| **B** | Landable backup. Worth applying when the SS/S/A pipeline is thin or when the role has narrow but real value. | "Maybe, depends what else is available." Q1 is cleared, but Q2-Q4 are collectively weak — small or low-signal company, narrow scope, adjacent tech the candidate would learn rather than apply, weak career ceiling. Or Q1 is cleared and the role is a structurally-narrow specialism the candidate's portfolio touches only tangentially. B captures the landable-but-mediocre band. |
| **C** | Lottery or low-signal. The aggregation produces this band when Q1 is a real headwind — implicit selectivity, sub-1% conversion, hiring patterns the candidate doesn't clear — OR when Q1 is cleared but the role's quality is genuinely poor on most dimensions. | "Probably not." Q1-headwind roles land here even when Q2-Q4 are strong, because the prestige-trap pattern down-weights through the aggregation: high CV value + strong technical fit + brutal selectivity = C, not A. The fit assessment names the Q1 signal explicitly so the grade is auditable. C is also where genuinely low-quality but technically-achievable roles land. |
| **F** | Do not apply. | "No." Hard dealbreaker present — unachievable seniority on the description (explicit 5+ years floor for an entry-level candidate, staff/principal scope, leadership expectations), excluded role type or sector per the candidate's hard preferences, non-engineering role disguised by title, hard credential floor the candidate cannot clear, location explicitly outside the candidate's hard-preferred set with no remote option. |

**The grade is the aggregation, not a category lookup.** The rubric does not say "C = the lottery bucket" or "A = the stretch bucket". The grade letter emerges from holding Q1-Q5 plus the analytical dimensions in tension and reasoning through them in the order Step 3 above prescribes (Q1 first as primary lens, Q2-Q5 as refinements within the Q1-determined band). When the aggregation produces a letter that contradicts the assessment narrative — when the narrative names sub-1% conversion but the letter is A, or names a confidently-cleared graduate pipeline but the letter is C — the letter is wrong and the grader re-runs the aggregation rather than reaching for a categorical override.

**Distribution is an output of the reasoning across the candidate's actual pool, not a budget.** The right shape of the SS/S/A/B/C/F distribution depends on the candidate's specific constraints — visa timeline, location preferences, seniority band, sector exclusions, stack-concentration. A narrow-profile candidate produces a smaller SS/S/A pool naturally; the realistic-landable shortlist for that candidate is genuinely shorter than for a generic CS graduate. The rubric does not impose target percentages on any tier. If the realistic SS/S pool for a given candidate's session is 30 jobs out of 1,500, the right grade-jobs output is 30 SS/S grades, not 200 grades manufactured by promoting stretches to fill an imagined budget. If a session produces unusually many SS or unusually few — that is information about the candidate's pool, not a signal to inflate or deflate. The fit assessments do the work; the distribution is whatever the assessments produce.

---

## Cross-referencing and Relative Grading

**Mandatory after every batch.** Do not write grades to the database without completing this.

### Calibration-anchored grading, not batch-relative grading

**Critical design principle:** Grades must be calibrated against the full universe of graded jobs in the database, NOT against the current batch. Batches are never representative — the prioritisation system deliberately puts the best jobs first, and re-assessment batches may contain only top-tier jobs. Within-batch distribution enforcement ("surely these can't all be S") produces grade deflation when the batch is legitimately skewed toward high-quality jobs.

**How calibration works:**

1. **Before grading begins**, pull a calibration sample from the database: 2-3 real examples at each grade tier (SS, S, A, B, C, F) with their fit assessments and the company name/grade. These are the grade anchors — they define what each tier looks like in this specific database.

2. **Grade each job against the calibration anchors**, not against other jobs in the batch. Ask: "Does this job belong alongside the SS examples, or alongside the A examples?" The batch composition is irrelevant — a batch of 20 genuinely excellent jobs should produce 20 high grades.

3. **Within-batch comparison is a consistency check**, not a distribution enforcer. After grading, scan for: did I grade two very similar jobs at different tiers? Did I grade two very different jobs at the same tier? These are errors to fix — but "too many S grades in one batch" is NOT an error if each job individually deserves S against the calibration anchors.

4. **Anchor selection — not every graded job is a good anchor.** The calibration sample is what shapes every subsequent grade, so the anchors themselves must reflect the realism semantic above, not the pre-realism distribution. A good SS anchor is a role where the candidate is in the realistic applicant pool AND technical alignment is strong AND the description is specific enough to be a recognisable pattern (graduate / new-grad / intern at a wide-funnel firm with structured pipeline). A role with strong alignment but sub-1% conversion is not an SS anchor — it is an A-tier or B-tier anchor that demonstrates prestige-trap reasoning, and pulling it as an SS calibration example would re-import the inflation the realism semantic exists to prevent. When the database's existing graded universe contains historical pre-realism inflation (S/SS grades on roles that should re-grade to A under the prestige-trap rules), prefer pulling anchors from the post-realism graded set, or hand-select anchors that match the realism criteria. If no clean anchors exist for a tier yet, name that gap explicitly in the batch's grading rationale rather than silently anchoring against inflated examples.

### Cross-referencing checks

After grading against calibration anchors, verify:

1. **Cross-reference with company grades.** A graduate role at an S-tier company should rarely grade lower than A unless there's a specific problem with the role itself (not the company). If the company is S but the graduate SWE role is C, something is likely wrong — re-examine. It CAN happen (the "role" might be pre-sales disguised by title), but it demands explicit justification.

2. **Consistency within the batch.** Two roles with very similar descriptions, requirements, and company quality should land at the same grade. If they don't, one of them is wrong.

3. **The "which offer would you take" test.** For any two jobs you graded at different tiers — if the candidate had offers from both, would they take the higher-graded one? If not, the grades are wrong.

### Sanity checks

Before writing to the database, scan for:
- Any graduate/entry-level role at a major employer graded C or F — does it have a hard exclusion reason, or was it misgraded?
- Any role mentioning Rust as a primary language graded C or F — Rust roles are rare, are we sure?
- Any role at an S-tier company graded C or F without hard seniority mismatch — why?

These are not automatic corrections. They are red flags that demand re-examination and explicit justification in the fit assessment.

### When grading in parallel across agents

Each agent receives the same calibration sample and grades independently against those anchors. The orchestrator MUST still do a cross-batch consistency check before writing to the database — pull the top 5 and bottom 5 from each agent's output and verify they make sense relative to each other and the calibration anchors. But do NOT redistribute grades to fit a target distribution.

---

## Career-Stage Context

The same career-stage factors that affect company grading affect job grading, but with the added benefit of having the actual job description to work with.

**Key calibration:**
- **CV signal is disproportionately important.** "Graduate SWE at Bloomberg" is worth more as a first CV line than "Rust Infrastructure Engineer at Unknown Startup Ltd," even if the second role is more technically aligned. Read `experience.md` — there is no prior work history to fall back on.
- **Tech stack is the least important factor.** A graduate role at Monzo using Go is a strong career move. The candidate will learn Go in weeks. The company signal, sponsorship, mentorship, and career trajectory compound over years. Never make tech stack the deciding factor.
- **Sponsorship-capable employers with graduate programmes are solving two hard problems at once.** These roles should be graded with awareness that sponsorship + entry-level hiring is an exceptionally valuable combination.
- **"Consumer product" is not "consumer-facing role."** A backend infrastructure engineer at Spotify is doing systems engineering. The exclusion is for roles that are customer-facing in nature (consulting, support, sales), not for engineering roles at companies with consumer products.
- **Graduate rotation programmes are a strength, not a weakness.** "You'll rotate across three teams" is breadth — a high-weight positive signal. Do not downgrade because the specific team is uncertain.

---

## Common Grading Errors

**Every one of these has caused real grading failures in production.**

**Grading based on title, not description.** "Senior" at many UK companies is 2-3 years out of university. "AI Engineer" can be cutting-edge ML infrastructure or QA testing for AI products. Always read the full description. If you're grading on title alone, you're guessing.

**Over-weighting tech stack — with one important asymmetry.** As a generic rule, a graduate Go role at a strong sponsoring company is worth more than a Rust role at a 3-person startup with no funding. Languages are learned in months; company signal, career trajectory, and sponsorship compound over years. So tech stack should not normally be the deciding factor between adjacent grades.

The asymmetry: when a candidate's portfolio is **concentrated in a specific stack** (multiple substantive projects at Proficient band in `profile/skills.md`, several per-project files in `profile/projects/` all using the same primary language, the candidate's strongest differentiator vs other graduates in the same applicant pool), stack alignment carries more weight than the generic rule implies. The mechanism: stack alignment with the role is what flips a marginal applicant into a competitive one inside the applicant pool, because the candidate's evidence base (the projects) directly anchors to the role's technical requirements. For these candidates, a role using their primary stack is *meaningfully more landable* (Q1) and *meaningfully more demonstrable* (Q3) than a role using an adjacent stack — those are the two questions doing the work, not "stack as tiebreaker" in isolation.

This is not a stack-specific carveout; it applies to any candidate whose portfolio is concentrated. Detection from `profile/`: count the per-project files where the language listed first in the project's stack matches the role's primary stack, AND the project's status frontmatter is active or substantively-built-dormant. When that count is ≥3 (a meaningful concentration), Q1 and Q3 both benefit from stack alignment beyond the generic tiebreaker weight. When the count is 0-1 (the candidate has touched the stack but it isn't a concentration), the generic rule holds.

The reverse case is symmetrical: a role using a stack the candidate has never touched and has no concentration in is Q3-weak by default, regardless of how "transferable" the underlying paradigm is. The portfolio is the evidence; the absence of relevant projects in `profile/projects/` is real Q3 friction.

**Under-weighting company signal for a first job.** Read `experience.md` — there is no work history. The first employer's name IS the credential. A generic backend role at Bloomberg is worth more for career trajectory than a perfectly-aligned role at a company nobody has heard of.

**Treating "consumer product" as "consumer-facing role."** Spotify's backend is systems engineering. Uber's pricing engine is distributed systems. Monzo's transaction processing is financial infrastructure. The product being consumer-facing does not make the engineering role consumer-facing.

**Assuming "no sponsorship mention" means "won't sponsor."** Large companies with international teams almost always sponsor. Only penalise sponsorship when there are active negative signals.

**Penalising graduate programmes for breadth.** "You'll rotate across three teams" is a feature, not a bug. Do not downgrade for uncertain team placement in a structured programme.

**Grade inflation from enthusiasm.** An exciting role that's unachievable (hard 5+ years requirement) is still F. Enthusiasm is a signal that the application will be strong — it doesn't change achievability.

**Grade inflation from prestige.** A reputable name on a CV is a Q2 (CV-value) signal. It says nothing on its own about Q1 (achievability). The two axes are independent and must be assessed separately: a brand-name firm can be either a realistic primary target (when it has a high-volume graduate / new-grad / intern pipeline that genuinely accepts the candidate's profile shape) or a real Q1 headwind (when its grad pipeline is narrow and selectivity-heavy). The error is collapsing them — letting "this would look great on a CV" pull a top grade when Q1 is actually a headwind, OR under-grading a role at a less-known firm when its grad pipeline is well-cleared by the candidate's profile.

**Binding obligation: every SS / S / A fit assessment names the firm's hiring-pattern signal before stating the grade.** Specifically, the assessment text identifies (1) the firm's actual hiring pattern at the entry-level — graduate intake volume, university-acceptance breadth, screening shape (algorithmic-interview competence vs competitive-programming pedigree vs PhD filter vs structured rotation), the proportion of the applicant pool that converts — and (2) the candidate's position relative to that pattern, citing specific elements of `profile/education.md`, `profile/experience.md`, and `profile/projects/` that place them in or outside the realistic primary-target pool. The Q1 reasoning is the first substantive paragraph of the assessment narrative; Q2 reputation reasoning follows it; the grade letter is consistent with the Q1 verdict the narrative names. An assessment that produces grade A or above without an explicit Q1 hiring-pattern signal, or whose grade letter contradicts its own Q1 verdict, fails this obligation regardless of how strong Q2-Q4 read.

Reputation and selectivity are not the same axis; do not infer one from the other. The two axes both flow into the aggregation in Step 3 — Q1 as the primary lens, Q2 as the tiebreaker within the Q1-determined band — and the assessment's narrative makes both visible.

---

## Worked Examples

### Example: Graduate SWE, Infrastructure @ Cloudflare (→ SS)

**Q1 — Can they get it?** Yes. Explicitly graduate programme. No years required. Structured onboarding with mentorship.

**Q2 — Good first CV line?** Exceptional. "Graduate Infrastructure Engineer — Cloudflare" opens every door in systems engineering.

**Q3 — Background gives an edge?** Strong. Nyquestro demonstrates lock-free, performance-critical systems thinking. NeuroDrive shows distributed system reasoning at scale. Cloudflare uses Rust in production — the candidate's primary language and strongest differentiator vs other graduates who typically bring web application experience.

**Q4 — Engaging work?** Highly. Edge network infrastructure handling millions of requests/second. Performance-critical, systems-level, distributed. Direct alignment with what the candidate builds for fun.

**Q5 — Practical constraints?** All solved. London office, confirmed Skilled Worker sponsor, established graduate programme addressing visa timeline.

**Dimensions confirm:** All critical and high-weight dimensions strong. No weaknesses.

**Grade: SS.** Every question has a strong answer. Dimensions confirm. The Rust + infrastructure + systems alignment with the strongest projects in the portfolio makes this a standout.

### Example: SDE-I, New Grad @ Amazon (→ SS) — reputable AND realistic

This example exists to make the reputation × selectivity decoupling explicit. A reputable name is not, on its own, evidence that the candidate is outside the realistic applicant pool. Some of the strongest CV-signal firms run wide-funnel graduate pipelines that genuinely accept the candidate's profile shape. Those land at SS, not at stretch.

**Q1 — Can they get it?** Yes, with a real chance. Amazon's SDE-I / university-grad pipelines in London and EU hire hundreds of new graduates per intake cycle. The pipeline accepts a wide range of universities and degree classifications — the screen is standard algorithmic-interview competence rather than a top-university or competitive-programming pedigree filter. Realistic conversion is non-trivial. This is a wide-funnel role; the candidate is in the primary applicant pool, not on the outside.

**Q2 — Good first CV line?** Very strong. "SDE-I — Amazon" opens doors at every subsequent cloud / distributed-systems / consumer-tech employer. FAANG-tier signal.

**Q3 — Background gives an edge?** Strong, treated within the realistic-applicant frame. NeuroDrive's distributed simulation, Cernio's async pipeline, and Image Browser's local-first systems engineering all map to AWS infrastructure-adjacent work. The portfolio's depth makes the candidate a genuinely strong applicant in the SDE-I pool.

**Q4 — Engaging work?** Reasonable. Cloud infrastructure at scale — technically deep, distributed, real engineering. Not the candidate's top passion domain (trading / compilers / from-scratch ML), but a solid match for the systems-engineering thread that runs through the portfolio.

**Q5 — Practical constraints?** All solved. London office, established Skilled Worker sponsorship, structured graduate programme.

**Dimensions confirm:** All critical and high-weight dimensions strong. Q1 cleared genuinely.

**Grade: SS.** Reputation is strong AND realistic conversion is strong. This is the load-bearing distinction the realism semantic exists to make: Amazon's wide-funnel grad pipeline + university acceptance breadth + standard-screen shape make Q1 genuinely cleared. Reputable does not mean hard. Compare with the Jane Street example immediately below — same FAANG-tier-or-above CV signal, opposite Q1 reading.

### Example: Software Engineer @ Jane Street (London) (→ C) — reputable BUT brutal

This example exists to make the prestige-trap pattern visible. A reputable name with strong technical alignment aggregates down through the Q1-primary lens when Q1 fails on implicit selectivity. The pre-realism reading of this role would land at SS — Q2, Q3, and Q4 all confirm in the absolute frame. The aggregation in Step 3 catches that Q1 is the weak link and the role lands at C, not in the top tiers.

**Q1 — Can they get it?** Real headwind. Jane Street's London graduate pipeline hires single-digit graduates per cycle out of thousands of applicants. The firm recruits heavily from a small set of top-CS programmes (Oxbridge / Imperial and equivalents) and the screen weights competitive-programming pedigree (IOI / ICPC / high Codeforces ratings) as a primary signal. None of this appears in the role's description — the description reads as openly accessible. The actual hiring patterns do not. The candidate's profile (BEng from York, no formal work history, no competitive-programming track record) puts them outside the realistic primary-target pool. Submitting the application is fine; realistic conversion is sub-1%.

**Q2 — Good first CV line?** Very strong in the absolute frame. "SWE — Jane Street" opens any door in quant / trading / systems infrastructure. Above-FAANG-tier signal in the relevant domain. But Q2 is a refinement *within* the Q1-determined band, not a Q1 substitute — strong Q2 cannot lift a role whose Q1 reads as a real headwind back into the top tiers.

**Q3 — Background gives an edge?** Technically yes within an absolute frame: a lock-free matching engine and exchange-protocol thinking map directly to Jane Street's domain. But Q3 must be assessed within the realistic-applicant frame — and within Jane Street's actual applicant pool (top-CS-programme graduates with comp-programming pedigree), the portfolio is competitive but not differentiating in the way it would be in Amazon's pool.

**Q4 — Engaging work?** Yes. OCaml compiler engineering / trading systems / from-scratch substantive engineering — close to the candidate's passion domain.

**Q5 — Practical constraints?** Solved. London, established Skilled Worker sponsorship.

**Grade: C.** Q1 reads as a real headwind: the firm's narrow-funnel pipeline filters on credentials the candidate does not have, the sub-1% conversion makes the application a lottery rather than a primary target. Through the §How to Grade a Job Step 3 aggregation, the Q1 headwind dominates: strong Q2 + strong Q3 + strong Q4 refine *which* below-the-line letter the role lands at, but cannot lift it back across the line into A or above. The aggregation lands at C — the lottery / Q1-headwind band where the application is worth firing only when the rest of the pipeline is thin enough that lottery tickets are worth the time cost. The fit assessment explicitly names the Q1 hiring-pattern signal so the grade is auditable.

The contrast with the Amazon example above is the load-bearing point of this rubric's aggregation semantic: same FAANG-or-above CV signal in both, but Amazon's wide-funnel pipeline genuinely accepts the candidate's profile shape (Q1 cleared, aggregates to SS) while Jane Street's narrow-funnel pipeline filters on credentials and pedigree the candidate does not have (Q1 a real headwind, aggregates to C despite identical Q2/Q3/Q4 strength). Reputation and selectivity are independent axes; do not conflate them. The aggregation handles the rest.

### Example: Graduate SWE @ Monzo (→ A)

**Q1 — Can they get it?** Yes. Graduate-level, achievable.

**Q2 — Good first CV line?** Strong. "Software Engineer — Monzo" is a well-known UK tech brand.

**Q3 — Background gives an edge?** Moderate. Go stack doesn't directly leverage Rust proficiency, but the distributed systems and financial transaction processing connect to Nyquestro and Aurix at the problem level. The candidate's from-scratch systems thinking transfers even if the language is different.

**Q4 — Engaging work?** Moderately. Backend infrastructure for consumer banking — technically deep (distributed systems, real-time financial processing) but not the candidate's core passion domain (trading systems, compilers, ML infrastructure). "Interesting enough, with excellent other factors."

**Q5 — Practical constraints?** All solved. London, guaranteed sponsor, established hiring.

**Grade: A.** Three strong answers (achievable, good CV line, constraints solved), one moderate (edge), one moderate (engagement). The brand signal + sponsorship + engineering depth make this solidly A despite the tech stack and domain being adjacent rather than core.

### Example: Graduate Backend Engineer @ a mid-tier UK fintech (→ B) — landable-but-mediocre

This example exists to make the landable-but-mediocre case explicit. The aggregation in Step 3 produces B for roles where Q1 is genuinely cleared but Q2-Q4 collectively land in the lower-quality band. These roles previously drifted to C under the old "achievable but limited career value" framing; the revised aggregation keeps them at B, where they belong as backup applications worth firing when the SS/S/A pipeline is thin.

**Q1 — Can they get it?** Yes, cleared decisively. The role is a graduate programme at a 200-person UK fintech with a structured intake, no specific university filter, screens on standard algorithmic-interview competence. The description states verbatim "open to graduates from any 2:2-and-above degree" — the candidate's degree class clears the gate. Realistic conversion is non-trivial; the firm hires ~10 graduates per cycle from a pool of a few hundred applicants.

**Q2 — Good first CV line?** Moderate. The fintech is recognised within the UK payments industry but not a household name; "Graduate Backend Engineer at [B-tier fintech]" opens doors at adjacent fintech firms but does not carry the broad credential signal of a FAANG / top-tier line.

**Q3 — Background gives an edge?** Moderate. The role uses Go and PostgreSQL; the candidate's portfolio is primarily in a different language (per `profile/projects/` and `profile/skills.md`). The systems-engineering paradigm transfers — distributed processing, async I/O, transaction integrity — but no per-project file directly anchors to the role's stack. The candidate is a competent applicant in the pool, not a standout.

**Q4 — Engaging work?** Moderate. Backend infrastructure for payments processing — technically real, with distributed-systems depth — but not the candidate's core passion domain. "Interesting enough" rather than "deeply engaging".

**Q5 — Practical constraints?** All solved. London office, established Skilled Worker sponsorship, structured graduate programme that addresses the visa timeline per `profile/visa.md`.

**Aggregation.** Q1 is cleared, so the role is in the landable band. Q2-Q4 are collectively in the "decent on most dimensions, no single weakness that downgrades to C, no single strength that elevates to A" zone — moderate CV value, adjacent stack, moderate engagement. The aggregation lands at B: a landable backup application worth firing when the SS/S/A pipeline is thin, not a top priority but not a dismissal either.

**Grade: B.** The role is genuinely landable AND genuinely mediocre on quality; B captures both. Compare with the Monzo example above (also A) — Monzo's "Software Engineer at a strong-brand UK fintech" Q2 signal is enough to lift it to A; this role's lesser-known fintech Q2 signal is not, and the aggregation correctly separates them. The contrast also makes the rubric's landability-first aggregation visible: both roles clear Q1, but Q2-Q4 differentiate the letter within the Q1-cleared band.

### Example: Senior Staff Platform Engineer @ Unknown Corp (→ F)

**Q1 — Can they get it?** No. Description requires "8+ years of production experience, led platform teams of 5+, principal-level architecture ownership." Hard seniority mismatch per `experience.md`.

**Grade: F.** Question 1 fails decisively. No other questions matter.

### Example: "Software Engineer" @ Well-funded Startup — Actually Solutions Engineering (→ F)

**Q1 — Can they get it?** Probably, based on seniority.

**Q2 — Good CV line?** Decent company name.

**Q3 — Background gives an edge?** Not really — the description reveals 60% customer calls, integration support, custom API adapters.

**Q4 — Engaging work?** No. This is customer-facing support engineering disguised by title.

**Q5 — Practical constraints?** Fails. Customer-facing roles are a hard exclusion in `preferences.toml`.

**Grade: F.** Hard exclusion triggered (customer-facing role type). Title said "Software Engineer" but description reveals solutions engineering.

---

## Evidence Standards for Fit Assessments

Every fit assessment must connect the job to the candidate's profile with specific evidence. The grade should be the conclusion of the reasoning, not a label attached to generic commentary.

### What "specific" means

| Element | Generic (unacceptable) | Specific (required) |
|---------|----------------------|---------------------|
| Project alignment | "Has relevant projects" | "Nyquestro's lock-free matching engine demonstrates the concurrent systems design this role demands" |
| Technology match | "Good stack overlap" | "Requires Rust and Python — proficient in both per skills.md" |
| Seniority | "Seems achievable" | "Lists '0-2 years' — 4 substantial projects demonstrate equivalent capability" |
| Gaps | "Some gaps exist" | "Heavy Kubernetes usage — listed as a gap in portfolio-gaps.md" |
| Sponsorship | "They probably sponsor" | "Confirmed on UK sponsor register. Graduate visa expires Aug 2027 — 15+ months buffer" |

### The five-question standard

For SS/S grades, the fit assessment should clearly answer all five core questions with specific evidence. For A/B, at minimum questions 1, 2, and 5. For C/F, the primary reason (usually question 1 or 5) with specific justification.

If the assessment doesn't answer the relevant questions with specific profile references, it's not done.

### Aggregation verification — Q1 reasoning leads the narrative

The aggregation Step 3 names Q1 as the primary lens; the fit assessment's narrative makes that primacy visible. For every SS, S, and A assessment, the narrative structure obligation is:

1. **First substantive paragraph: Q1 reasoning.** The assessment opens (after any one-line role / company label) with the Q1 verdict and its evidence. Name the firm's hiring-pattern signal (intake volume, university-acceptance breadth, screening shape, conversion-rate signal) and the candidate's position relative to that signal, citing specific elements from `profile/education.md`, `profile/experience.md`, `profile/visa.md`, and `profile/projects/` that place them in or outside the realistic primary-target pool. For roles with explicit seniority gates, quote the description's experience requirement here as the realism evidence.

2. **Second paragraph: Q2 reasoning.** Once Q1 is named, the CV-value signal of the role and company follows. This is where reputation and trajectory matter — but only within the Q1-determined band. An SS / S grade requires Q1 to read as cleared; an A grade requires Q1 to read as at least viable; a C grade signals Q1 is a headwind even if Q2 reads strong.

3. **Subsequent paragraphs: Q3, Q4, Q5 and the dimensions.** Profile alignment, day-to-day work, practical constraints, and the analytical dimensions land here in whatever order serves the role's specific shape.

4. **Closing line: the grade letter and the one-sentence aggregation summary.** The summary connects the grade letter back to the Q1 verdict named in paragraph 1. If the grade is A or above, the closing sentence cites the Q1-cleared element; if the grade is B, C, or F, the closing sentence cites the Q1 headwind or hard exclusion.

**Consistency obligation.** The grade letter is consistent with the Q1 verdict the narrative names. An assessment that opens with "Q1 is a real headwind — sub-1% conversion at this firm's narrow grad pipeline" cannot close at grade A or higher. An assessment that opens with "Q1 is cleared decisively — this firm's structured graduate intake hires hundreds per cycle from the candidate's degree band" cannot close at grade C or F unless Q5 names a hard exclusion (location, sector, role-type) that overrides everything else. When the letter contradicts the narrative, the grader re-runs Step 3 aggregation rather than reaching for a categorical override.

**Why this obligation exists.** The earlier audit identified that the grader's narrative could correctly diagnose a Q1 headwind ("prestige-trap, lottery, sub-1% conversion") while the letter still landed at A because A's previous definition explicitly permitted that split. The aggregation revision in §Grade Scale removed the permission. The narrative-structure obligation here makes the new aggregation visible to anyone reading the assessment — the user reviewing the grade, a future grader pulling calibration anchors, the `check-integrity` skill auditing stale assessments — so the grade letter cannot drift from the Q1 reasoning that produced it.

---

## Additive-Freedom Permission for Prescribed Lists in This File

The lists in this file are non-exhaustive and may be extended on a per-run basis when a specific role's shape calls for an addition the rubric's author did not anticipate. Additions are pure-additive — they raise the floor of the rubric's coverage, never weaken it.

- **The six grade tiers (SS / S / A / B / C / F)** are the current letter system. The letters and their aggregation semantics stay; no new letter is added as an escape hatch (no "S+", no "A-stretch", no sub-tiers). If a candidate's pool repeatedly produces roles that cluster oddly between two letters, the right response is to refine the §Grade Scale row definitions, not introduce a new letter.

- **The five core questions (Q1-Q5)** are the current evaluation framework. If a role's shape genuinely calls for a sixth question (a recurring pattern across multiple grading sessions), add it to §The Core Questions and update the §Aggregation reasoning to name where it fits in the Q1-primary ordering. Existing questions remain mandatory.

- **The analytical dimensions** (Critical, High-weight, Medium-weight) are the current set. If a new dimension surfaces across multiple grading sessions (e.g. a sector-specific quality signal not covered by the existing dimensions), add it with the appropriate weight band; do not weaken or remove existing dimensions.

- **The Common Grading Errors catalogue** is observation-driven. When a new grading-failure pattern is observed across multiple sessions, add a new entry naming the pattern, the mechanism, and the fix. Existing entries remain.

- **The worked-examples set** is non-exhaustive. When a role shape recurs that the existing examples do not cover well (e.g. a new sector, a new candidate-pool pattern), add a worked example following the existing format (Q1-Q5 walk + dimensions + aggregation summary). Diverse examples are better than uniform ones; new examples should differ from existing ones along at least one dimension (sector, candidate-pool pattern, Q1 verdict, aggregation outcome).

- **The Evidence Standards table and the five-question standard** are the current minimum requirements per tier. New evidence types (e.g. quoted artefacts from a job's company-glassdoor reviews) may be added as required citations; existing requirements stay.

For all six lists above, additions are **strictly additive** — they may not introduce conditionals that gate existing requirements, weaken any existing item, or create sub-tiers that act as escape hatches. Document the addition in the grader's batch report (step 7 of the skill) so future readers can see the extension trail.
