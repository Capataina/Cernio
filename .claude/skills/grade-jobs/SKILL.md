---
name: grade-jobs
description: "Grades ungraded jobs in the Cernio database against the user's profile using a calibration-anchored six-tier rubric (SS / S / A / B / C / F). Writes `grade`, `fit_assessment` (cites the description verbatim for seniority + technology claims), `fit_score`, `evaluation_status` ('strong_fit' / 'weak_fit' / 'no_fit'). Fetches missing descriptions via WebFetch / WebSearch before grading. Parallelised across subagents by company cluster. Updates `profile/portfolio-gaps.md` after every batch with market-pattern findings. Invoke on 'grade jobs', 'evaluate pending jobs', 'rate the next batch', 'grade ungraded', 'process pending jobs', 'evaluate the queue', 'clear the grading backlog', or when pending rows exist. Not for searching jobs (search-jobs), discovering companies (discover-companies), grading companies (grade-companies), or preparing applications (prepare-applications). Use whenever pending jobs exist, even if not named."
---

# Grade Jobs

Grades individual jobs in the Cernio database. Company grading determines what gets searched; job grading determines what gets applied to. Every grade emerges from reasoning about this specific role for this specific candidate — the role's actual requirements (from the description, not the title), the candidate's flagship projects and technologies, the sponsorship timeline, and the calibration anchors already graded in the database.

Grades are not permanent. They reflect the current profile state. When the profile changes — a new project, a closed gap, a shifted preference — prior grades become potentially stale and the `check-integrity` skill surfaces them for re-grading.

---

## Mandatory Reads Before Grading Any Job

| # | What | Evidence |
|---|---|---|
| 1 | **Every file in `profile/`** — 15 files | The SS / S / A fit assessments cite at least one project from `projects.md` by name, one technology from `skills.md` by name, one element from `visa.md` (sponsorship situation), and one target from `preferences.toml` (career trajectory) |
| 2 | **`references/grading-rubric.md`** | You can cite the six tiers, the two critical-dimension F-forcers (career ceiling, seniority match), and the mandatory-description-citation rule from the Thought Machine failure |
| 3 | **`references/profile-context.md`** | You can name the profile file and element you pulled each piece of alignment evidence from |
| 4 | **`references/prioritisation-guide.md`** | The first batch you grade is prioritised by `company_grade × title_promise × role_type_alignment`, not database insertion order |

The profile is not cached in this skill or in any reference file. Every invocation reads `profile/` fresh. Subagent prompts embed the full profile + full reference content verbatim — agents cannot read project files.

---

## Workflow

### 1. Query the pending queue and report the count

```sql
SELECT j.id, j.title, j.url, j.location, j.raw_description, j.posted_date,
       c.name AS company_name, c.grade AS company_grade, c.what_they_do
FROM jobs j
JOIN companies c ON c.id = j.company_id
WHERE j.evaluation_status = 'pending'
   OR j.evaluation_status = 'evaluating'
ORDER BY
    CASE c.grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 ELSE 4 END,
    j.id ASC;
```

Report the total immediately: *"142 jobs pending. Starting with the highest-signal batch."*

### 2. Pull calibration anchors

Before grading any new job, pull 2–3 real examples per grade tier from the graded-jobs universe:

```sql
SELECT j.grade, j.title, c.name AS company_name, c.grade AS company_grade,
       j.fit_assessment
FROM jobs j
JOIN companies c ON c.id = j.company_id
WHERE j.grade IS NOT NULL AND j.evaluation_status <> 'archived'
ORDER BY
    CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                  WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6 END,
    RANDOM();
```

These anchors define what each tier looks like in this DB. Every grading decision asks: "does this belong alongside the SS anchors or the A anchors?" rather than "how does this compare to the other jobs in my current batch?" Within-batch grading produces deflation on strong batches.

Embed the anchors in every parallel subagent's prompt.

If existing fit assessments are shallow (pre-rewrite one-liners), use the anchors for grade-level calibration — what kind of role deserves SS vs A — rather than as examples of assessment quality.

### 3. Prioritise the batch

Batch selection follows the prioritisation-guide's compound signal: `priority = company_grade × title_promise × role_type_alignment`. S-company jobs with promising titles are graded before B-company jobs with generic titles. The user sees actionable results early rather than waiting for the full queue to clear.

Do not grade in DB insertion order.

### 4. Get the full job description — never grade on title alone

Grading a job without its description is unreliable. Titles lie: "AI Engineer" at Apple could be ML infrastructure or QA for AI-powered hardware accessories; "Software Engineer, Platform" could be Angular/Redux or Kubernetes/Terraform. The description is the only source of truth on responsibilities, seniority, and technical requirements.

**If `raw_description` is NULL, empty, under 100 words, or vague:**

1. `WebFetch` on the job's `url` to visit the actual posting page and extract the description.
2. Write the fetched description back:
   ```sql
   UPDATE jobs SET raw_description = ? WHERE id = ?;
   ```
3. If the page is behind a login wall or returns no useful content, `WebSearch` for `"{job title}" "{company}"` to find the listing on LinkedIn / Indeed / Glassdoor / other aggregators.
4. If after all attempts no description can be found, leave the job at `pending` and flag it in the batch report. A title-only grade is worse than no grade — a false SS wastes the user's time and a false F hides a perfect opportunity.

### 5. Evaluate against the rubric

Read the full description. Evaluate against every dimension in `grading-rubric.md`, with particular attention to the two critical-dimension F-forcers: career ceiling and seniority match.

**Mandatory: the fit assessment cites the description verbatim for seniority and technology claims.** This proves the description was read. For seniority, the assessment states what the description actually says about experience:

- *"Description states: '3–5 years of hands-on experience' — hard seniority mismatch, F"*
- *"Description states: 'no specific years required, looking for strong problem solvers' — achievable"*
- *"Description states: '2+ years preferred' — stretch but potentially achievable given portfolio depth"*
- *"No experience requirements mentioned in description — likely accessible"*

An assessment that writes "entry-accessible" without citing what the description actually says about seniority is unreliable. The Thought Machine SS-grade failure happened because an agent wrote "entry-accessible" when the description literally said "3–5 years of hands-on experience." Citation prevents that class of error entirely.

### 6. Assign grade and write to the DB

```sql
UPDATE jobs
SET grade = ?, fit_assessment = ?, fit_score = ?, evaluation_status = ?
WHERE id = ?;
```

| Grade | `evaluation_status` | `fit_score` range |
|-------|---|---|
| SS | `strong_fit` | 0.90 – 1.00 |
| S | `strong_fit` | 0.75 – 0.89 |
| A | `weak_fit` | 0.60 – 0.74 |
| B | `weak_fit` | 0.40 – 0.59 |
| C | `no_fit` | 0.20 – 0.39 |
| F | `no_fit` | 0.00 – 0.19 |

**Column contract:** `grade`, `fit_assessment`, `fit_score`, `evaluation_status`. Exact names, no variants. Escape single quotes by doubling (`it''s`). Single-line UPDATE, semicolon-terminated.

### 7. Assessment-content standards by grade tier

**SS / S assessments include all of:**

- **Profile alignment** — name specific projects from `profile/projects.md`. Example: *"Your Nyquestro matching engine demonstrates exactly the lock-free, low-latency systems thinking this role demands."* Not *"you have relevant experience."*
- **Technology match** — name specific technologies from `profile/skills.md` that the job requires. *"Requires Rust — your primary language at proficient level."* Not *"good tech stack match."*
- **Gap analysis** — name specific gaps from `profile/portfolio-gaps.md` or skills the job wants that the profile lacks. *"Heavy Kubernetes usage — currently a gap, but your Docker experience from NeuroDrive provides a foundation."*
- **Sponsorship analysis** — cite `profile/visa.md`: current status, timeline, whether the company can sponsor when needed.
- **Career trajectory** — reference long-term targets from `profile/preferences.toml`.
- **Application narrative** — a 1–2 sentence pitch for why the candidate's application would be compelling for this specific role.

**A / B assessments include:**

- At least one specific profile project or technology that aligns
- The primary reason it falls short of S-tier
- Whether it is worth pursuing if higher-grade options are scarce

**C / F assessments include:**

- The specific dealbreaker or primary weakness. Example: *"Requires 5+ years production experience — hard seniority mismatch"* or *"Solutions engineering role disguised by title — 60% customer-facing, excluded by preferences."*

**Unacceptable (real example of what fails this standard):** *"Good role at a strong company. Decent fit with the profile. Worth considering."*

**Acceptable:** *"SS. Graduate Infrastructure Engineer at Cloudflare's London office. The role builds and operates edge network infrastructure handling millions of requests/second — directly aligned with your systems engineering focus. Your Nyquestro matching engine demonstrates the exact lock-free, performance-critical thinking this team values. NeuroDrive's distributed multi-agent simulation shows you can reason about distributed systems at scale. Rust is mentioned as a 'bonus' language — your primary language and strongest differentiator. Cloudflare is a confirmed Skilled Worker sponsor with an established graduate programme, addressing the sponsorship timeline from your Graduate visa (expires Aug 2027). Career ceiling is exceptional — clear IC track to Principal Engineer, compensation reaches your long-term targets. The only gap: no production operations experience, but the graduate programme explicitly provides mentorship and on-call onboarding. Application narrative: your matching engine project + Rust proficiency + systems thinking make you a standout among graduates, most of whom have only web application experience."*

### 8. Parallel grading

Grading is parallelised. Split the pending queue into batches by company cluster — 5 agents × ~100–150 jobs each runs in ~3 minutes vs ~30 minutes sequential. Grading is independent per job; agents do not need each other's output.

**Every parallel subagent's prompt embeds verbatim:**

- The full content of `references/grading-rubric.md`
- The full content of `references/profile-context.md`
- The full content of `references/prioritisation-guide.md`
- The full content of every file in `profile/`
- The calibration anchors pulled in step 2
- The list of assigned jobs with company name, company grade, title, location, and description excerpt from the DB
- Explicit instruction to output SQL UPDATE statements directly — not narrative summaries

Subagents that do not receive these files produce shallow, generic assessments. Under-contextualising is the single largest parallel-grading quality failure. Over-share.

**Subagent output format — exact SQL:**

```sql
UPDATE jobs SET grade = 'X', evaluation_status = 'strong_fit', fit_assessment = 'reasoning', fit_score = 0.85 WHERE id = NNN;
```

The orchestrator collects SQL from all agents and executes in one batch.

### 9. Batch discipline

**Always include in the current batch:** every job with clear high-signal indicators — entry-level / graduate titles at S-tier companies, roles explicitly naming profile technologies, roles at companies with exceptional domain alignment. If 70 graduate roles exist, grade all 70.

**Include generously:** when uncertain whether a job should be in the current batch or deferred, include it. Grading cost is minutes; a deferred perfect role risks missing an application deadline.

**Defer strategically:** senior roles at B-tier companies, roles with generic titles and no clear signal, roles at companies with weaker alignment. Later batch.

If the queue is manageable, grade everything in one pass.

### 10. Portfolio gap tracking — mandatory after every batch

After every grading batch, `profile/portfolio-gaps.md` is updated. This is the career-coaching output of the grading process and one of the highest-value artefacts the system produces. Silently skipping the update breaks the loop.

**What to track across the batch:**

- **Technologies appearing repeatedly in SS / S / A roles but absent from `profile/skills.md`** — count, roles + companies, example: *"Kubernetes appeared in 12 of 30 S-tier infrastructure roles. Not in skills.md."*
- **Domain knowledge strong roles expect but the profile doesn't demonstrate** — example: *"4 trading roles asked for FIX protocol experience. Nyquestro uses a custom binary protocol."*
- **Recurring experience-pattern requirements** — example: *"8 roles mentioned 'production incident management'. No production operations evidence in the profile."*
- **Strengths the market values** — not only gaps. Example: *"Rust appeared in 6 SS-tier roles. The profile's Rust depth is a confirmed differentiator."*

**Write to `profile/portfolio-gaps.md`** in this format:

```markdown
- **[Skill / Technology / Domain]** — appeared in N of M graded roles at [grade tiers].
  Roles: [2-3 specific role names]. Companies: [company names].
  Profile status: [not present / partially addressed by X / strength].
  Impact: [how this affects grading — are roles being downgraded because of it?]
  Closure opportunity: [specific, actionable suggestion if a gap]
```

Also update "Known Gaps" (new gaps identified, gaps closed by recent additions) and "Current Strengths" (strengths confirmed by market demand).

Even a null result deserves a dated note — *"No new portfolio gap patterns in this batch — checked 2026-04-20."*

### 11. Report batch results

Present results grouped by grade for scannability:

```
## Batch results (30 graded, 112 remaining)

### SS (2)
- Graduate Software Engineer, Infrastructure @ Cloudflare — new grad role, tier-1 infrastructure, systems-heavy, confirmed sponsor
- Software Engineer, New Grad — Trading Systems @ Jane Street — perfect domain alignment, legendary engineering culture, sponsors visas

### S (4)
- Software Engineer, Platform @ Palantir — strong brand, broad scope, slight reach on seniority but compelling narrative

### A (8)
- Backend Engineer @ Monzo — good fintech signal, slightly narrow scope

### C / F (10)
- Senior Staff Engineer @ Unknown Corp — hard 8+ years required
```

For SS and S roles, the full `fit_assessment` is available on request. Inline the full text if the batch is small; offer per-row details if the batch is large.

Flag any jobs graded without descriptions (should be zero — step 4 is a hard gate). Report the portfolio-gaps update — what was added, what was null. Ask the user whether to continue with the remaining queue or stop.

---

## Reference Loading

**Mandatory-core — read at skill invocation every time:**

- `references/grading-rubric.md` — six-tier rubric, dimensions, weights, worked examples, boundary cases, the description-citation rule
- `references/profile-context.md` — how to read the profile for job grading (not the profile itself)
- `references/prioritisation-guide.md` — compound-signal batch ordering: `company_grade × title_promise × role_type_alignment`

All three are read at invocation. The rubric alone without profile-context produces rubric-correct grades with unspecific reasoning; prioritisation alone orders a queue of shallow grades; none of the three is optional.

---

## Inviolable Rules

1. **Never grade on title alone.** If `raw_description` is missing / under 100 words / vague, fetch via WebFetch or WebSearch before grading. If no description can be found, leave at `pending` — do not assign any grade.
2. **The fit assessment cites the description verbatim for seniority and technology claims.** Not paraphrase, not interpretation — quoted text from the description. Guards against the Thought Machine class of misgrade.
3. **SS / S / A fit assessments name specific profile elements by name.** Projects from `projects.md`, technologies from `skills.md`, visa facts from `visa.md`, career targets from `preferences.toml`. Generic phrases fail the standard.
4. **Profile is read fresh every invocation.** No caching, no embedded snapshots.
5. **Grades are calibrated against DB anchors, not the current batch.** A batch of genuinely excellent jobs produces excellent grades — no distribution flattening.
6. **`profile/portfolio-gaps.md` is updated after every batch.** Even a null update ("no new patterns this batch") is written — silent skipping breaks the career-coaching loop.
7. **Subagents receive full profile + full reference content verbatim.** Under-contextualised subagents produce shallow assessments.
8. **Exact SQL column names.** `grade`, `fit_assessment`, `fit_score`, `evaluation_status`. `evaluation_status` maps to the six-tier table above.

---

## Quality Checklist

- [ ] All 15 files in `profile/` were read this invocation
- [ ] All three reference files were read this invocation
- [ ] Calibration anchors were pulled from the DB (2–3 per tier) and embedded in every subagent's prompt
- [ ] Every job graded in this batch had its full description — no title-only grades in the session transcript
- [ ] Every SS / S / A fit assessment names at least one specific project from `projects.md`, one technology from `skills.md`, one visa fact from `visa.md`, and one career target from `preferences.toml`
- [ ] Seniority assessment cites what the description actually says about experience (quoted text), not the agent's interpretation
- [ ] Career ceiling reasoning considers the 10–15 year trajectory, not just the immediate role
- [ ] Company grade was factored into each job grade (mediocre role at S-tier company carries more signal than the same role at C-tier)
- [ ] Sponsorship was assessed with reference to `visa.md` timeline + the company's evidenced sponsorship capability (sponsor register / description signals)
- [ ] SS and S grades have multi-paragraph assessments with all six required components (alignment, tech match, gap analysis, sponsorship, trajectory, application narrative)
- [ ] C and F grades cite a specific dealbreaker or weakness
- [ ] `evaluation_status` mapping is correct per tier (SS/S → strong_fit, A/B → weak_fit, C/F → no_fit)
- [ ] `fit_score` decimal is within the tier's band
- [ ] `profile/portfolio-gaps.md` was updated with this batch's patterns — or a dated null-result note was written
- [ ] Batch report includes count breakdown + highlights + any flagged jobs (no-description cases)
