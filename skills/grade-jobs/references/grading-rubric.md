# Job Grading Rubric

The evaluation framework for grading individual jobs. Job grading is where the real decision happens — company grading determines what we search, but job grading determines what we apply to. Every grade should emerge from careful reasoning about this specific role for this specific candidate, not from mechanically scoring dimensions.

**Important:** All profile facts must come from reading `profile/` files — never from hardcoded values. When this rubric says "the candidate's portfolio" or "the visa timeline," it means: read `projects.md`, read `visa.md`, etc.

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

Grading happens in four steps. Each step informs the next.

**Step 1: Answer the core questions.** These force pragmatic thinking about what this role actually means for the candidate's career. Write 2-3 sentences per question. The answers ARE the evaluation — everything else supports them.

**Step 2: Evaluate against the analytical dimensions.** The dimensions add precision and structure. They catch things the questions might miss — a role that feels right but has a hidden seniority wall, or one that feels wrong but actually has exceptional career ceiling.

**Step 3: Cross-reference questions and dimensions.** Do they agree? When they conflict, reason through why. If the questions say "this would be amazing" but a critical dimension (seniority match) fails — the dimension wins; an unachievable role is F regardless. If the dimensions say "weak alignment" but the questions reveal this would be a transformative career move — the questions might be seeing something the dimensions missed.

**Step 4: Calibration check.** Compare this job against the calibration anchors — real examples from the database at each grade tier. Does it belong alongside the examples at the grade you've assigned? If it's clearly stronger than the A-tier anchors but weaker than the SS anchors, it's S. Do NOT compare against other jobs in the current batch to enforce a distribution.

The fit assessment written to the database should be the output of this process — the actual reasoning, not a summary.

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

Read `experience.md` for formal work history and `projects.md` for demonstrated capability. The portfolio can substitute for 1-2 years of professional experience if the projects demonstrate genuine depth — but it cannot substitute for 5 years of production systems ownership.

**If the answer is clearly no — hard 5+ years requirement, staff/principal scope, leadership expectations — the grade is F. No other question matters.** This is the only question that can unilaterally determine the grade.

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

Read `projects.md` carefully. Every project has a **Tier** field (Flagship, Notable, Minor) and a **Status** field. Both matter:

**Project tiers determine evidence weight:**

| Tier | Meaning | How to use in fit assessment |
|------|---------|------------------------------|
| **Flagship** | Deep, substantial, actively maintained — the candidate's strongest work | Primary evidence. Cite these by name when assessing fit. A role that aligns with 2-3 flagship projects is one where the candidate has a genuine edge. |
| **Notable** | Solid projects showing real skill, but less depth or no longer active | Supporting evidence. Cite when directly relevant to the role, but don't build the entire fit case on these. |
| **Minor** | Small, abandoned early, or completed but not valued by the candidate | Mention only if directly relevant. Never use as primary evidence of capability. An abandoned particle sim website does not demonstrate the same thing as a lock-free matching engine. |

**Status matters too.** An "In Progress" flagship that the candidate is actively investing in demonstrates current capability and genuine interest. An "Abandoned" project — regardless of how interesting the concept was — shows the candidate started but didn't follow through, which is weaker evidence. A "Completed" project shows the candidate can finish what they start. Weight accordingly.

**What the flagship projects demonstrate:**
- Nyquestro → lock-free concurrency, low-latency systems, financial domain knowledge
- NeuroDrive → ML from scratch, RL, performance engineering under real-time constraints
- Aurix → DeFi analytics, quantitative risk modelling, financial mathematics
- Cernio → async networking, database integration, TUI development, systems architecture
- Image Browser → local ML inference, desktop application architecture, ONNX/CLIP integration

A role where 2-3 flagship projects map directly to the requirements is one where the candidate has a genuine edge over typical applicants. A role where only minor/abandoned projects are relevant means the candidate is competing without their strongest evidence — that's a weaker fit even if the technology nominally matches.

Also check `portfolio-gaps.md` — does this role require something the profile explicitly lacks? A gap in a "nice to have" is different from a gap in a core requirement.

### 4. Would the candidate enjoy the day-to-day work?

Not "is it systems engineering" in the abstract, but "would the candidate find this specific work interesting for 2 years?"

**Read the ENTIRE profile — not just projects.** `projects.md` shows what the candidate CAN build. `interests.md` shows what the candidate WANTS to build. These are not always the same. The candidate might have zero health projects but a deep interest in the intersection of AI and health — a health platform engineering role could be highly engaging despite no portfolio evidence. Similarly, `preferences.toml` captures sector preferences and `cover-letter.md` reveals how the candidate frames their motivations.

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

### 5. Does this solve the candidate's practical constraints?

The unglamorous but critical question:
- **Location:** Is it in London, Cambridge, or Remote-UK? (Read `preferences.toml`)
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
| **Seniority match** | Can the candidate realistically get hired? Based on `experience.md` and `projects.md`, not the title. |
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
| **SS** | Apply immediately, prioritise above all | "If I got this offer tomorrow, I'd accept without hesitation." Every question has a strong answer. All dimensions confirm. These are rare — ~2-3% of jobs. |
| **S** | Strong candidate, apply with energy | "I'd be genuinely excited about this." Most questions strong, maybe one moderate. A clear career-positive move. ~10-15%. |
| **A** | Worth applying to | "This is good — I'd apply if I have time." Good on several dimensions, 1-2 notable weaknesses. Still a net positive. ~25-30%. |
| **B** | Backup / worth watching | "Maybe, depends what else is available." Acceptable but uninspiring on several fronts. Apply if the pipeline is thin. ~30-35%. |
| **C** | Only if desperate | "Probably not." Achievable but limited career value. Narrow scope, weak signal, poor trajectory. ~10-15%. |
| **F** | Do not apply | "No." Dealbreaker present. Unachievable seniority, excluded role type/sector, non-engineering role disguised by title. ~10-15%. |

**Grades should be conservative.** If more than 20% of jobs are S or above, the bar is too low.

---

## Cross-referencing and Relative Grading

**Mandatory after every batch.** Do not write grades to the database without completing this.

### Calibration-anchored grading, not batch-relative grading

**Critical design principle:** Grades must be calibrated against the full universe of graded jobs in the database, NOT against the current batch. Batches are never representative — the prioritisation system deliberately puts the best jobs first, and re-assessment batches may contain only top-tier jobs. Within-batch distribution enforcement ("surely these can't all be S") produces grade deflation when the batch is legitimately skewed toward high-quality jobs.

**How calibration works:**

1. **Before grading begins**, pull a calibration sample from the database: 2-3 real examples at each grade tier (SS, S, A, B, C, F) with their fit assessments and the company name/grade. These are the grade anchors — they define what each tier looks like in this specific database.

2. **Grade each job against the calibration anchors**, not against other jobs in the batch. Ask: "Does this job belong alongside the SS examples, or alongside the A examples?" The batch composition is irrelevant — a batch of 20 genuinely excellent jobs should produce 20 high grades.

3. **Within-batch comparison is a consistency check**, not a distribution enforcer. After grading, scan for: did I grade two very similar jobs at different tiers? Did I grade two very different jobs at the same tier? These are errors to fix — but "too many S grades in one batch" is NOT an error if each job individually deserves S against the calibration anchors.

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

**Over-weighting tech stack.** A graduate Go role at Monzo with guaranteed sponsorship is worth more than a Rust role at a 3-person startup with no funding. Languages are learned in months; company signal, career trajectory, and sponsorship compound over years. Tech stack should never be the deciding factor between adjacent grades.

**Under-weighting company signal for a first job.** Read `experience.md` — there is no work history. The first employer's name IS the credential. A generic backend role at Bloomberg is worth more for career trajectory than a perfectly-aligned role at a company nobody has heard of.

**Treating "consumer product" as "consumer-facing role."** Spotify's backend is systems engineering. Uber's pricing engine is distributed systems. Monzo's transaction processing is financial infrastructure. The product being consumer-facing does not make the engineering role consumer-facing.

**Assuming "no sponsorship mention" means "won't sponsor."** Large companies with international teams almost always sponsor. Only penalise sponsorship when there are active negative signals.

**Penalising graduate programmes for breadth.** "You'll rotate across three teams" is a feature, not a bug. Do not downgrade for uncertain team placement in a structured programme.

**Grade inflation from enthusiasm.** An exciting role that's unachievable (hard 5+ years requirement) is still F. Enthusiasm is a signal that the application will be strong — it doesn't change achievability.

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

### Example: Graduate SWE @ Monzo (→ A)

**Q1 — Can they get it?** Yes. Graduate-level, achievable.

**Q2 — Good first CV line?** Strong. "Software Engineer — Monzo" is a well-known UK tech brand.

**Q3 — Background gives an edge?** Moderate. Go stack doesn't directly leverage Rust proficiency, but the distributed systems and financial transaction processing connect to Nyquestro and Aurix at the problem level. The candidate's from-scratch systems thinking transfers even if the language is different.

**Q4 — Engaging work?** Moderately. Backend infrastructure for consumer banking — technically deep (distributed systems, real-time financial processing) but not the candidate's core passion domain (trading systems, compilers, ML infrastructure). "Interesting enough, with excellent other factors."

**Q5 — Practical constraints?** All solved. London, guaranteed sponsor, established hiring.

**Grade: A.** Three strong answers (achievable, good CV line, constraints solved), one moderate (edge), one moderate (engagement). The brand signal + sponsorship + engineering depth make this solidly A despite the tech stack and domain being adjacent rather than core.

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
