# grade-jobs

**Read ALL files in `profile/` before grading. Every file. Do not skip any — missing context leads to grading errors.** The profile is a living system that changes as the candidate's portfolio grows. Never rely on embedded snapshots or cached assumptions about the profile.

Grades ungraded jobs from the database in prioritised batches, evaluating each against the user's profile across multiple dimensions and writing structured assessments back to the DB. Use when the user says "grade jobs", "evaluate pending jobs", "rate the next batch", "grade ungraded", "what jobs need grading", "evaluate the queue", "process pending jobs", or when the session's purpose is clearing the grading backlog. Not for searching or fetching jobs (that's search-jobs), not for discovering companies (that's discover-companies), and not for grading companies themselves (that's grade-companies).

---

## Mandatory reads — do not proceed without completing these

**STOP. Before grading any job, you MUST read these files in full:**

1. **Every file in `profile/`** — all 15 files, no exceptions.
2. **`references/grading-rubric.md`** — the full grading rubric with dimensions, weights, worked examples, and boundary cases.
3. **`references/profile-context.md`** — how to read and synthesise the profile files for job evaluation.
4. **`references/prioritisation-guide.md`** — how to order the pending queue so the best jobs get graded first.

**When delegating grading to parallel agents:** embed the FULL TEXT of all three reference files and all relevant profile data in each agent's prompt. Agents spawned via the Agent tool cannot read files from the repo — if you don't embed the rubric, they will produce shallow, generic assessments that are useless to the user. This is a hard requirement.

**Do not begin evaluating any job until all mandatory reads are complete.**

---

## Why this skill exists

The search pipeline deposits hundreds of jobs into the database with `evaluation_status = 'pending'` and no grade. Those jobs are useless until a human-quality judgment evaluates them against the profile. Grading is the step that turns raw listings into actionable career intelligence. The user should see the best opportunities first, not wade through noise to find signal.

---

## Before you start

1. **Read ALL files in `profile/`.** Every single one — `personal.md`, `visa.md`, `education.md`, `experience.md`, `projects.md`, `skills.md`, `preferences.toml`, `portfolio-gaps.md`, `resume.md`, `cover-letter.md`, `interests.md`, `certifications.md`, `languages.md`, `military.md`, `volunteering.md`. The profile-context reference file explains what to extract from each.

2. Read all reference files in `references/`:

| File | What it gives you |
|------|-------------------|
| `references/grading-rubric.md` | The full grading rubric: dimensions, weights, grade scale, worked examples, boundary cases |
| `references/profile-context.md` | Guide to reading and synthesising the profile files for job evaluation |
| `references/prioritisation-guide.md` | How to order the pending queue so the most promising jobs get graded first |

---

## The grading workflow

### 1. Query the pending queue

```sql
SELECT j.id, j.title, j.url, j.location, j.raw_description, j.posted_date,
       c.name AS company_name, c.grade AS company_grade, c.what_they_do
FROM jobs j
JOIN companies c ON c.id = j.company_id
WHERE j.evaluation_status = 'pending'
   OR j.evaluation_status = 'evaluating'
ORDER BY
    CASE c.grade
        WHEN 'S' THEN 1
        WHEN 'A' THEN 2
        WHEN 'B' THEN 3
        ELSE 4
    END,
    j.id ASC
```

Report the total count immediately: "142 jobs pending. Starting with the highest-signal batch."

### 2. Prioritise the batch

Do not grade in database insertion order. Use the prioritisation logic in `references/prioritisation-guide.md` to select the most promising jobs first. The compound signal is:

```
priority = company_grade x title_promise x role_type_alignment
```

Grade S-company jobs with promising titles before B-company jobs with generic titles. The user sees actionable results early instead of waiting for the full queue to clear.

### 3. Get the job description — never grade blind

You MUST have the full job description before assigning any grade other than 'skip'. For each job, check `raw_description` in the database. If it exists, is non-empty, and gives you enough detail to evaluate the role's actual responsibilities, seniority expectations, and technical requirements, use it directly.

**If `raw_description` is NULL, empty, under 100 words, or vague:**

1. Use WebFetch on the job's `url` to visit the actual posting page and extract the full description.
2. Write the fetched description back to the database: `UPDATE jobs SET raw_description = ? WHERE id = ?`
3. If the job page is behind a login wall or returns no useful content, use WebSearch for the job title + company name to find the listing on LinkedIn, Indeed, Glassdoor, or other job aggregators.
4. If after all attempts you still cannot find a description, do NOT grade the job — leave it as `pending` and flag it in the batch report. Grading on title alone produces unreliable results and defeats the purpose of the entire grading system.

**Why this matters:** A title like "AI Engineer" at Apple could be anything — cutting-edge ML infrastructure, or QA testing for AI-powered hardware accessories. The description is the only way to know. A "Software Engineer, Platform" could require Angular and Redux (frontend disguised as platform) or Kubernetes and Terraform (actual infrastructure). Titles lie. Descriptions tell the truth.

**Never grade on title alone.** A wrong grade is worse than no grade — it either wastes the user's time (false SS) or hides a perfect opportunity (false F).

### 4. Evaluate the job

Read the full description. Assess it across every dimension in the grading rubric, with particular attention to the critical dimensions (career ceiling and seniority match) that can force an F grade regardless of other strengths.

**Mandatory: cite the description in the fit assessment.** The fit assessment MUST quote or paraphrase specific requirements from the job description. This proves you actually read it. For seniority assessment specifically, you must state what the description says about experience requirements — not your interpretation, but the actual text:

- "Description states: '3-5 years of hands-on experience' — hard seniority mismatch, F"
- "Description states: 'no specific years required, looking for strong problem solvers' — achievable"
- "Description states: '2+ years preferred' — stretch but potentially achievable given portfolio depth"
- "No experience requirements mentioned in description — likely accessible"

**If you write "entry-accessible" without citing what the description actually says about seniority, the grade is unreliable.** The Thought Machine SS-grade failure happened because an agent wrote "entry-accessible" when the description literally said "3-5 years of hands-on experience." Citing the description prevents this class of error entirely.

The evaluation should answer:
- Is the seniority achievable given the candidate's experience level (from `experience.md`) and project depth (from `projects.md`)? Look past the title to the actual requirements.
- Does this role lead somewhere valuable over a 10-15 year trajectory, relative to the candidate's long-term targets (from `preferences.toml`)?
- What profile elements align strongly? What gaps exist (cross-reference `portfolio-gaps.md`)?
- Is there a compelling narrative for why this candidate fits?
- What is the sponsorship situation (based on the timeline from `visa.md`)?

### 5. Assign a grade and write to DB

```sql
UPDATE jobs
SET grade = ?,
    fit_assessment = ?,
    fit_score = ?,
    evaluation_status = ?
WHERE id = ?
```

| Grade | evaluation_status | fit_score range |
|-------|-------------------|-----------------|
| SS | `strong_fit` | 0.90 - 1.00 |
| S | `strong_fit` | 0.75 - 0.89 |
| A | `weak_fit` | 0.60 - 0.74 |
| B | `weak_fit` | 0.40 - 0.59 |
| C | `no_fit` | 0.20 - 0.39 |
| F | `no_fit` | 0.00 - 0.19 |

The `fit_assessment` field carries the reasoning. It must connect the job to the candidate's specific profile — not generic evaluation, but "this job wants X, you have Y" analysis.

**SS/S assessments must include ALL of the following:**
- **Profile alignment:** Name specific projects from `profile/projects.md` that demonstrate relevant capability. "Your Nyquestro matching engine demonstrates exactly the lock-free, low-latency systems thinking this role demands" — not "you have relevant experience."
- **Technology match:** Name specific technologies from `profile/skills.md` that the job requires and the candidate has. "Requires Rust — your primary language at proficient level" — not "good tech stack match."
- **Gap analysis:** Name specific gaps from `profile/portfolio-gaps.md` or skills the job requires that the candidate lacks. "Heavy Kubernetes usage — currently a gap in your portfolio, but your Docker experience from NeuroDrive provides a foundation."
- **Sponsorship analysis:** Reference `profile/visa.md` — current visa status, timeline, and whether this company can sponsor when needed.
- **Career trajectory:** How this role fits the long-term targets from `preferences.toml`.
- **Application narrative:** A 1-2 sentence pitch for why the candidate's application would be compelling for this specific role.

**A/B assessments must include:**
- At least one specific profile project or technology that aligns
- The primary reason it falls short of S-tier
- Whether it's worth pursuing if higher-grade options are scarce

**C/F assessments must include:**
- The specific dealbreaker or primary weakness (e.g., "Requires 5+ years production experience — hard seniority mismatch" or "Solutions engineering role disguised by title — 60% customer-facing, excluded by preferences")

**Unacceptable fit assessment (real example of what NOT to write):**
> "Good role at a strong company. Decent fit with the profile. Worth considering."

**Acceptable fit assessment (what we expect):**
> "SS. Graduate Infrastructure Engineer at Cloudflare's London office. The role builds and operates edge network infrastructure handling millions of requests/second — directly aligned with your systems engineering focus. Your Nyquestro matching engine demonstrates the exact lock-free, performance-critical thinking this team values. NeuroDrive's distributed multi-agent simulation shows you can reason about distributed systems at scale. Rust is mentioned as a 'bonus' language — your primary language and strongest differentiator. Cloudflare is a confirmed Skilled Worker sponsor with an established graduate programme, addressing the sponsorship timeline from your Graduate visa (expires Aug 2027). Career ceiling is exceptional — clear IC track to Principal Engineer, compensation reaches your long-term targets. The only gap: no production operations experience, but the graduate programme explicitly provides mentorship and on-call onboarding. Application narrative: your matching engine project + Rust proficiency + systems thinking make you a standout among graduates, most of whom have only web application experience."

### 6. Parallel grading

**Critical: every parallel agent must receive the full content of ALL reference files.** Embed the complete text of `references/grading-rubric.md`, `references/profile-context.md`, and `references/prioritisation-guide.md` in each agent's prompt. Also embed the full content of every file from `profile/`. Agents that don't receive these files will produce shallow, generic assessments — "good role, decent fit" — that are useless. The reference files are what enable specific, profile-grounded evaluation. This is the single most important requirement for parallel grading quality.

Job grading should always be parallelised using multiple agents. Split the pending queue into batches by company cluster (e.g. 5 agents each handling ~100-150 jobs) and run them simultaneously. Each agent reads the same profile, gets a different set of jobs with their descriptions, and outputs SQL UPDATE statements.

**Why parallel:** 700 jobs graded in ~3 minutes with 5 parallel agents vs 30+ minutes sequentially. The grading is independent per job — no agent needs another agent's output. Each agent reads the full profile and makes independent judgments.

**How to split:** Group by company so each agent has context about the companies it's grading. S-tier companies with many jobs should be in the same batch for consistency. Give each agent the job ID, company name, company grade, title, location, and description excerpt from the database.

**Each agent outputs SQL only, using EXACTLY this format:**
```sql
UPDATE jobs SET grade = 'X', evaluation_status = 'strong_fit', fit_assessment = 'reasoning here', fit_score = 0.85 WHERE id = NNN;
```

**Critical SQL rules for agents:**
- Column names are `grade`, `evaluation_status`, `fit_assessment`, `fit_score`. NOT `reasoning`, NOT `assessment`, NOT any variation.
- `evaluation_status` must be exactly one of: `'strong_fit'` (SS/S), `'weak_fit'` (A/B), `'no_fit'` (C/F)
- `fit_score` must be a decimal between 0.0 and 1.0 (SS: 0.90-1.00, S: 0.75-0.89, A: 0.60-0.74, B: 0.40-0.59, C: 0.20-0.39, F: 0.00-0.19)
- Escape single quotes by doubling them: `it''s` not `it's`
- Every statement must end with a semicolon
- One UPDATE per line, no multi-line statements

The orchestrator collects all SQL outputs and executes them in a single batch against the database.

### 7. Batch discipline

The orchestrator decides how many jobs to grade based on signal strength. The guiding principle: **grade everything important, not a random slice.**

- **Always include:** Every job with clear high-signal indicators (entry-level/graduate titles at S-tier companies, roles explicitly mentioning technologies from the profile, roles at companies with exceptional domain alignment). If there are 70 graduate roles, grade all 70 — they're all high priority.
- **Include generously:** When uncertain whether a job should be in this batch or deferred, include it. The cost of grading an extra job is a few minutes. The cost of deferring a perfect opportunity is potentially missing an application deadline.
- **Defer strategically:** Senior roles at B-tier companies, roles with generic titles and no clear signal, roles at companies with weaker alignment. These can wait for a later batch.

After grading completes:

1. Report progress: "Graded X/Y pending. Breakdown: N SS, N S, N A, N B, N C, N F."
2. Summarise highlights: name the SS and S roles with one-line reasons.
3. Note any portfolio gap patterns observed (see below).
4. Flag any jobs graded without descriptions — these need verification.
5. Ask the user whether to continue with remaining jobs or stop.

If the pending count is manageable, grade everything in one pass. Grading is largely a one-time cost per job — once done, the user has a stable, browsable, actionable list.

---

## Portfolio gap tracking — MANDATORY after every batch

**This is not optional.** Updating `profile/portfolio-gaps.md` after every grading batch is one of the most valuable outputs of the entire grading process. The patterns you spot across dozens of job descriptions — skills the market consistently asks for that the profile lacks — directly drive what the user should build or learn next. If the file isn't being updated, the career coaching loop is broken.

### What to track

As you evaluate each job, maintain a running tally of:

1. **Technologies that appear repeatedly in SS/S/A roles but are absent from `profile/skills.md`:**
   - Count how many roles mention each technology
   - Note which companies and role types ask for it
   - Example: "Kubernetes appeared in 12 of 30 S-tier infrastructure roles. Not in skills.md."

2. **Domain knowledge that strong roles expect but the profile doesn't demonstrate:**
   - Example: "4 trading systems roles asked for FIX protocol experience. Nyquestro uses a custom binary protocol but doesn't implement FIX."

3. **Experience patterns that recur as requirements:**
   - Example: "8 roles mentioned 'production incident management' or 'on-call experience'. No production operations evidence in the profile."

4. **Strengths the profile has that the market values:**
   - Don't only track gaps — track what converts well. If Rust keeps appearing in SS/S roles, that's a confirmed strength worth noting.
   - Example: "Rust appeared in 6 SS-tier roles. The profile's Rust depth is a genuine differentiator."

### How to update portfolio-gaps.md

After each batch, write to the "Patterns from Job Evaluations" section of `profile/portfolio-gaps.md`. Each entry should follow this format:

```markdown
- **[Skill/Technology/Domain]** — appeared in N of M graded roles at [grade levels].
  Roles: [2-3 specific role names that asked for it]. Companies: [company names].
  Profile status: [not present / partially addressed by X / strength].
  Impact: [how this gap affects grading — are roles being downgraded because of it?]
  Closure opportunity: [specific, actionable suggestion if it's a gap]
```

Also update the "Known Gaps" and "Current Strengths" sections if the batch reveals:
- A new gap not previously identified
- A gap that has been closed by a recent project
- A new strength confirmed by market demand

### When to update

- **After every grading batch, no exceptions.** Even if no new patterns emerged, confirm in the batch report that you checked. "No new portfolio gap patterns in this batch" is an acceptable update — silently skipping is not.
- **At minimum, update the "Patterns from Job Evaluations" section** with any observations, even if the other sections don't need changing.

---

## Presenting results to the user

After grading a batch, present a summary grouped by grade for scannability:

```
## Batch results (30 graded, 112 remaining)

### SS (2)
- Graduate Software Engineer, Infrastructure @ Cloudflare
  New grad role, tier-1 infrastructure company, systems-heavy, confirmed sponsor
- Software Engineer, New Grad — Trading Systems @ Jane Street
  Perfect domain alignment, legendary engineering culture, sponsors visas

### S (4)
- Software Engineer, Platform @ Palantir
  Strong brand, broad scope, slight reach on seniority but compelling profile narrative
  ...

### A (8)
- Backend Engineer @ Monzo — Good fintech signal, slightly narrow scope
  ...

### C/F (10)
- Senior Staff Engineer @ Unknown Corp — Hard 8+ years required
  ...
```

For SS and S roles, the full `fit_assessment` from the database should be available on request. Present it inline if the batch is small, or offer to show details for specific roles if the batch is large.

---

## Quality checklist

Before presenting a batch to the user, verify:

- [ ] All files in `profile/` were read before grading began
- [ ] Every job was graded against the full rubric, not just title-keyword matching
- [ ] Seniority assessment is based on the description's actual requirements, not the title
- [ ] Career ceiling reasoning considers the domain's 10-15 year trajectory, not just the immediate role
- [ ] Company grade was factored into the job grade (a mediocre role at an S-tier company has more signal than the same role at a C-tier company)
- [ ] Sponsorship was assessed based on company size, sponsor licence status, and description signals — using the timeline from `visa.md`, not assumed dates
- [ ] SS and S grades have multi-paragraph assessments that would help the user decide whether to apply
- [ ] C and F grades have a clear, specific reason (not just "not a fit")
- [ ] Grades were written to the database with correct evaluation_status mapping
- [ ] `profile/portfolio-gaps.md` has been updated with patterns from this batch — technologies, domains, and experience areas that SS/S/A roles asked for. If no new patterns emerged, the batch report explicitly states "no new portfolio gap patterns observed." The file must never be silently skipped.
- [ ] Every SS/S/A grade was assigned after reading the full job description, not just the title
- [ ] The batch summary includes a count breakdown and highlights the strongest opportunities
