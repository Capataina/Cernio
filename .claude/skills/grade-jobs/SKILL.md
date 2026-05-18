---
name: grade-jobs
description: "Grades ungraded jobs in the Cernio DB against the profile using a six-tier rubric (SS/S/A/B/C/F). Writes `grade`, `fit_assessment` as structured prose with Q1/Q2/Q3a/Q3b/Q4/Q5/Verdict slots (every slot prose with JD quotes + named profile anchors, no verdict enums), `evaluation_status`, and `evidence_basis` ('jd' / 'semantic' / 'insufficient'). Fetches missing descriptions via WebFetch / WebSearch first; failing that, takes the semantic-reasoning path for brand-strong roles from company + role-title context. Parallelised by company cluster. End-of-batch Relativity Pass: each agent reviews its rows against 3 random already-graded jobs per tier from the DB and adjusts inconsistencies. Updates `profile/portfolio-gaps.md` after every batch. Invoke on 'grade jobs', 'evaluate pending jobs', 'rate the next batch', 'grade ungraded', 'process pending jobs', 'clear the grading backlog'. Not for searching jobs, discovering companies, grading companies, or preparing applications."
---

# Grade Jobs

Grades individual jobs in the Cernio database. Company grading determines what gets searched; job grading determines what gets applied to. Every grade emerges from reasoning about this specific role for this specific candidate — the role's actual requirements (from the description, not the title), the candidate's flagship projects and technologies, the sponsorship timeline, and the calibration anchors already graded in the database.

Grades are not permanent. They reflect the current profile state. When the profile changes — a new project, a closed gap, a shifted preference — prior grades become potentially stale and the `check-integrity` skill surfaces them for re-grading.

---

## Mandatory Reads Before Grading Any Job

| # | What | Evidence |
|---|---|---|
| 1 | **Every file in `profile/` (excluding `sync-summary.md`, which is a sync audit artefact, not profile data)** | The SS / S / A fit assessments cite at least one project from `profile/projects/` by name, one technology from `skills.md` by name, one element from `visa.md` (sponsorship situation), one target from `preferences.toml` (career trajectory), AND one lifestyle anchor from `lifestyle-preferences.md` for the office area (e.g. "matches the Kings Cross gold standard," "Canary Wharf — modern but mixed-scale partial fit," "Croydon — fails the safety / regeneration anchors"). The aggregation semantic in `references/grading-rubric.md` §How to Grade a Job Step 3 also requires every SS / S / A assessment's narrative to lead with Q1 reasoning — name the firm's hiring-pattern signal explicitly (graduate intake volume, university-acceptance breadth, screening shape, conversion-rate signal) AND the candidate's position relative to that signal, citing the specific profile elements that place them in or outside the realistic primary-target pool. Q2 reputation reasoning follows; the grade letter is consistent with the Q1 verdict the narrative names. |
| 2 | **`references/grading-rubric.md`** | You can cite the six tiers, the two critical-dimension F-forcers (career ceiling, seniority match), and the mandatory-description-citation rule from the Thought Machine failure |
| 3 | **`references/profile-context.md`** | You can name the profile file and element you pulled each piece of alignment evidence from |
| 4 | **`references/prioritisation-guide.md`** | The first batch you grade is prioritised by `company_grade × title_promise × role_type_alignment`, not database insertion order |

The profile is not cached in this skill or in any reference file. Every invocation reads `profile/` fresh. Subagent prompts embed the full profile + full reference content verbatim — agents cannot read project files.

---

## Workflow

### 0. Run `cernio format` before any grading starts

Grading agents read `raw_description` verbatim (see step 1's SELECT). Raw HTML — `<p>`, `<strong>`, `&nbsp;`, `&amp;` — degrades grading quality: tokens are spent parsing markup, seniority cues buried in `<h3>` headers get missed, and the description-quote evidence obligation in step 5 produces rows with HTML fragments instead of readable prose. `cernio format` converts HTML to clean plaintext in-place on `raw_description` and `fit_assessment` (the same fields the calibration-anchor query in step 2 reads) and is fully idempotent, so running it at the top of every invocation is free insurance.

```bash
cernio format
```

Paste the row-count summary into the chat response. If zero rows were touched (already clean), say so explicitly — the step is required evidence that cleanliness is verified this run, not assumed. Without this step, every subagent that grades an HTML-laden job is reasoning against broken input.

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

### 4. Get the full job description — try description-first, fall back to semantic reasoning

The default path is JD-grounded grading. The description is the most reliable source of truth on responsibilities, seniority, and technical requirements. Titles can mislead: "AI Engineer" at one firm is ML infrastructure, at another it is LLM-glue prompting; "Software Engineer, Platform" can be React/Redux or Kubernetes/Terraform.

**If `raw_description` is NULL, empty, under 100 words, or vague:**

1. `WebFetch` on the job's `url` to visit the actual posting page and extract the description.
2. Write the fetched description back:
   ```sql
   UPDATE jobs SET raw_description = ? WHERE id = ?;
   ```
3. If the page is behind a login wall or returns no useful content, `WebSearch` for `"{job title}" "{company}"` to find the listing on LinkedIn / Indeed / Glassdoor / other aggregators.
4. If after both attempts no description can be found, **do not silently default to brand-stamp grading and do not blanket-flag as un-gradable.** Instead, take one of two paths:

   **(a) Semantic-reasoning path (`evidence_basis = 'semantic'`).** Use this when the company + role title together carry enough signal to ground a defensible grade without the JD. Specifically: wide-funnel established-firm graduate / intern / new-grad pipelines at companies whose hiring shape is well-known publicly (the agent can reason from training data about how the company hires at this band) AND a role title that is non-ambiguous about the work shape ("Software Engineer, University Graduate 2026" is non-ambiguous; "Engineer" alone is not). The Q3a slot in the fit assessment must explicitly state *"JD unavailable — reasoning from company hiring-pipeline pattern + role-title shape"* and the Q3a/Q3b prose must reference what the agent knows publicly about the company's grad pipeline AND the candidate's portfolio that maps to it. The grade can be any letter the structured reasoning supports including SS.

   **(b) Insufficient-evidence path (`evidence_basis = 'insufficient'`, `grade = NULL`, `evaluation_status = 'pending'`).** Use this when neither the description nor company+title context support reasoning. Specifically: unknown / low-signal company AND opaque role title ("Engineer" / "Developer" at an unknown firm). Leave the grade NULL — do not invent one. The row stays in the pending queue for a future pass after a description fetch succeeds.

   When semantic reasoning is used (path a), the fit_assessment is identical in shape to a JD-grounded assessment — Q1 through Verdict slots, all prose — but Q3a names the missing-JD substitution and the reasoning is anchored on the company-pipeline pattern rather than a JD quote. A semantically-graded Google graduate role is not lesser-evidence than a JD-graded Google graduate role; both produce defensible reasoning, evidence_basis distinguishes them so the user can filter if they want to. The default TUI filter keeps semantic-graded rows visible.

### 5. Evaluate against the rubric — structured prose output, no verdict enums

Read the full description (or, on the semantic-reasoning path, the company + role-title pattern). Evaluate against every dimension in `grading-rubric.md`. The fit_assessment is a **structured prose document** with named slots; every slot is prose, not a label or category-pick. The format below is the contract — every slot must be present, and every slot must be reasoned, not labelled.

**The structure** (each slot's target length is a floor, not a ceiling — write more when the case warrants):

```
## Q1 — Seniority and realistic landability
[2-4 sentences of prose. Must include a verbatim JD quote about experience /
qualifications / seniority band when JD is present (e.g. "the description
states: '2+ years of commercial Go experience'"). Must connect to Caner's
specific evidence — years from experience.md, project names from
projects/*.md, contributions from skills.md — by name. Concludes with the
agent's reasoned read on whether the gate is a hard floor, an artefact of
HR boilerplate, or a wide-funnel signal. On the semantic-reasoning path,
the JD quote slot is replaced with "JD unavailable — reasoning from
{company}'s known {grad / new-grad / intern / lateral} hiring pipeline shape".]

## Q2 — Company motivation
[2-4 sentences of prose. Must name at least one specific fact about the
company beyond brand recognition — funding stage, engineering reputation,
recent technical work, hiring practices, public engineering blog posts,
known infrastructure patterns. Connect that fact to why Caner specifically
would want to be there given preferences.toml + interests.md.]

## Q3a — Stack overlap
[2-4 sentences of prose. MUST include all three of:
 (a) a verbatim JD substring of ≥6 chars in double-quotes naming a
     specific technology, e.g. *the JD names "Rust 1.7+ and tokio"*;
 (b) a profile project FILE PATH, e.g. `projects/nyquestro.md`, not
     just the project name in prose;
 (c) a one-line evidence of what the cited project demonstrates
     relative to that JD technology — what specific feature of the
     project maps to the JD's requirement.
Stack alignment alone is NOT enough — Q3a only answers "does the
technology overlap?". The career-trajectory question is Q3b.
Worked example shape:
*The JD names "Go primary, Rust adjacent". `projects/nyquestro.md`
(active, 6.5k LOC Rust) demonstrates the lock-free / low-latency
systems work the role describes — the HDR-histogram tail-latency
tracking + deterministic LOB matching engine is exactly the
performance-critical paradigm. Go is adjacent: Caner has zero Go
projects but the conceptual surface is identical.*]

## Q3b — Career-axis match
[2-4 sentences of prose. Answers separately from Q3a: does this role's
day-1 work build toward Caner's target career trajectory? Reference what
kind of engineer Caner is becoming per profile/preferences.toml and the
active-status projects in profile/projects/. Reason in prose — "on-axis"
/ "adjacent" / "off-axis" is the agent's conclusion-in-prose, NOT a label
to pick from a list. The same technology stack can be on-axis at one
company and off-axis at another; the prose explains which it is here.]

## Q4 — Domain
[2-3 sentences of prose. What is the company building, who pays for it,
why does it matter to Caner specifically. If the domain is incidental
("a payments company that happens to have a frontend role"), name that.]

## Q5 — Logistics
[2-3 sentences of prose. Office location (cite a specific neighbourhood
when known and weigh against lifestyle-preferences.md anchors), hybrid
policy, sponsorship stance (cite visa.md timeline), deadline if any,
salary band if disclosed. Cite the JD or careers page directly.]

## Verdict
[3-5 sentences of plain-language summary. MUST contain all four of:
 (1) The role's strongest pull, named explicitly.
 (2) The role's strongest pushback, named explicitly.
 (3) One role-type framing from this exact list: "career launch",
     "axis bet", "credibility builder", "stretch", "deadweight".
     The phrase appears verbatim in the Verdict prose. Choose the
     framing that best describes the role for THIS candidate, given
     the Q1-Q5 reasoning above.
 (4) An answer to the budget question — "in a budget of ~30
     applications, does this make the cut?" — stated in prose
     (e.g. "yes — top of the cut" / "yes if there's room" /
     "no — does not make the cut").
The Verdict prose must be consistent with the Q1-Q5 reasoning above;
the grade letter follows from the Verdict.]

Grade: <SS | S | A | B | C | F>
```

**Banned satisficing patterns** (each is a failure mode the format exists to prevent):

- Verdict enums in Q1-Q5 slots: *"Q1 cleared-decisively"*, *"Q3 moderate"*, *"Q5 ✓"*. Every slot is prose; pick-from-list output fails.
- Arrow-shorthand derivations: *"Q2 strong, Q4 ✓ → A"*. The Verdict slot does the aggregation in prose; arrows skip the aggregation.
- Skill-band labels as evidence: *"Caner's React Comfortable level"* with no project named. The Comfortable-band label is profile metadata, not evidence; name the project.
- Boilerplate Q4/Q5 fills: *"Q5 clean"* / *"Q4 London ✓"* identical across rows. If Q4 and Q5 are written identically across the batch, the agent is template-filling, not reasoning.
- Invented categories: *"Standard B for accessible junior SE at mid-tier fintech"*. There is no "standard B" — the grade follows from the Verdict prose, which follows from Q1-Q5.
- Risk-decoration: naming a risk in Q3b but the Verdict + Grade unchanged. Acknowledged risks bite the grade (the worked examples in `grading-rubric.md` §Worked Examples include risks-bite templates).

The skill's job is to push the agent into reasoning, not classification. Anything that reduces to a multiple-choice pick is a satisficing slot the format intentionally removes.

### 6. Assign grade and write to the DB

```sql
UPDATE jobs
SET grade = ?,
    fit_assessment = ?,
    evaluation_status = ?,
    evidence_basis = ?
WHERE id = ?;
```

| Grade | `evaluation_status` |
|-------|---|
| SS | `strong_fit` |
| S | `strong_fit` |
| A | `weak_fit` |
| B | `weak_fit` |
| C | `no_fit` |
| F | `no_fit` |

| `evidence_basis` | When to write it |
|---|---|
| `jd` | JD content present (≥100 words) and used in the Q-slot reasoning |
| `semantic` | No usable JD; graded from company + role-title context. Grade can be any letter the structured reasoning supports including SS. |
| `insufficient` | No JD AND no usable semantic context. **Grade is NULL; evaluation_status stays at `pending`.** |

**Column contract:** `grade`, `fit_assessment`, `evaluation_status`, `evidence_basis`. Exact names, no variants. The legacy `fit_score` column was dropped from the schema — do not write it; including `fit_score` in any UPDATE will fail. Escape single quotes by doubling (`it''s`). Single-line UPDATE, semicolon-terminated.

### 7. Per-tier assessment substance (within the structured format)

The structured format from step 5 is the same shape for every grade. What changes per tier is the *substance* the Q-slots carry. Use this as a calibration check after writing the assessment — does the substance match the grade letter?

**SS / S — the assessment should let the reader feel why the role makes a fixed budget of 30 applications:**

- Q3a names ≥2 specific projects from `profile/projects/` with active-status anchors, each tied to a JD-quoted technology or paradigm.
- Q3b reasons concretely about career trajectory — "this role builds the [systems / ML / autonomy / quant-dev] axis Caner is on" with reference to the active projects that establish that axis.
- Q2 names a concrete fact about the company beyond brand recognition — a recent technical blog, an open-source contribution pattern, a known infrastructure choice, an engineering-led culture signal.
- Q5 cites visa.md sponsorship status against the company's known sponsor stance.
- Verdict answers the budget-of-30 question with "yes" and names the strongest pull.

**A — the assessment should explain why the role is worth applying but is not S/SS:**

- Q3a or Q3b (one of them) is moderate-strong, not strong. Q3b naming an "adjacent" career-axis match with prose reasoning is the typical pattern.
- At least one slot names a friction (e.g. "Q1 cleared-with-friction because the description mentions '2+ years preferred', clearable by portfolio depth but not decisively") — the friction is named in prose.
- Verdict answers the budget-of-30 question with "yes if there's room" — credibility builder, not career launch.

**B — the assessment should explain why the role is landable but mediocre:**

- Q3b is "off-axis" or only adjacent, reasoned in prose.
- Q3a stack overlap exists but doesn't pull the role up by itself.
- Q2 may be strong (good company) but the role's substance keeps it at B — the Verdict prose makes this trade-off explicit.

**C — the assessment should name what makes the role low-signal even though it's not F:**

- Q1 has named selectivity friction OR the role is structurally narrow / off-axis / weak ceiling.
- Verdict: budget-of-30 answer is "no" but the row is preserved for visibility.

**F — the assessment should name the specific dealbreaker by JD quote (or by company-context reasoning on the semantic path):**

- A hard floor cited by JD quote: *"the description states: 'minimum 5 years of production experience' — hard floor, no portfolio substitute available at this seniority"*.
- A hard preference exclusion: *"role is 60% customer-facing per the JD's responsibilities list — excluded by preferences.toml.hard.exclude_role_types"*.
- A hard location/sector exclusion.

**Unacceptable (any tier):** *"Good role at a strong company. Decent fit with the profile. Worth considering."* — no JD quote, no project name, no career-axis reasoning, no Verdict prose. Every Q-slot must carry its own load.

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

Subagents that do not receive these files produce shallow, generic assessments. Under-contextualising is the single largest parallel-grading quality failure — verified by prior production runs where summarised-profile subagents produced grade-level-correct but profile-unspecific assessments that failed the citation rule.

**Subagent output format — exact SQL:**

```sql
UPDATE jobs
SET grade = 'X',
    evaluation_status = 'strong_fit',
    fit_assessment = 'structured prose with Q1/Q2/Q3a/Q3b/Q4/Q5/Verdict slots',
    evidence_basis = 'jd'
WHERE id = NNN;
```

Do NOT include `fit_score` — the column has been dropped from the schema; any UPDATE containing `fit_score = ...` will fail. `evidence_basis` is always set: 'jd' when the JD was used, 'semantic' when company+title reasoning was used, 'insufficient' when neither (in which case `grade` is NULL and `evaluation_status` stays 'pending'). The orchestrator collects SQL from all agents and executes in one batch.

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

### 11. Relativity Pass — review just-graded rows against DB-sampled anchors

After the batch's UPDATE statements have been executed and before the batch report is written, each agent (or the orchestrator, if grading was serial) runs the relativity pass against the DB. The purpose is to catch within-batch inconsistencies — an agent grading 30 jobs in sequence can drift, calibrate against the wrong neighbours, or apply a fix from job 5 to job 25 without realising the cases differ.

**Step 11.1 — Sample 3 random already-graded jobs per grade tier from the DB.** Run this query verbatim (the agent receives it pre-filled in its prompt):

```sql
WITH ranked AS (
    SELECT j.id, j.title, c.name AS company_name, c.grade AS company_grade,
           j.grade, j.fit_assessment,
           ROW_NUMBER() OVER (
               PARTITION BY j.grade
               ORDER BY RANDOM()
           ) AS rn
    FROM jobs j
    JOIN companies c ON c.id = j.company_id
    WHERE j.grade IS NOT NULL
      AND j.evaluation_status <> 'archived'
      AND j.id NOT IN (<the IDs just graded in this batch>)
)
SELECT * FROM ranked WHERE rn <= 3
ORDER BY CASE grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                    WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6 END, rn;
```

The result is up to 18 reference rows (3 × 6 tiers). Tiers with fewer than 3 graded rows return what they have; do not pad with cross-tier substitutes.

**Step 11.2 — Compare each just-graded row against the reference set.** For each row the agent just wrote, ask:

- Are there reference rows at the same grade whose Q1-Q5 reasoning is structurally weaker than this row's? (If yes, this row may belong one tier higher.)
- Are there reference rows one tier higher whose reasoning is structurally weaker than this row's? (If yes, this row likely belongs at that higher tier.)
- Are there reference rows at the same grade whose reasoning is structurally much stronger than this row's? (If yes, this row may belong one tier lower.)

"Structurally stronger/weaker" is reasoned in prose: the Verdict slot of the reference is more decisive, the Q3b career-axis match is more direct, the Q1 friction is named more sharply. The relativity pass is not a numeric comparison.

**Step 11.3 — Adjust grades and rewrite affected slots.** When the relativity pass reveals an inconsistency, the agent:

1. Re-reads the just-graded row's structured assessment.
2. Identifies which Q-slot's reasoning is out of step with the reference cohort.
3. Rewrites that slot to either justify the original grade (if the original is correct and the prose was just imprecise) OR adjusts the grade and updates the Verdict to reflect the new aggregation.
4. Issues a follow-up UPDATE for that row.

**Step 11.4 — Emit the relativity delta summary.** Before the batch report, the agent writes:

```
## Relativity Pass

Reviewed N just-graded rows against 18 DB-sampled reference rows
(3 per tier).

Adjustments: M grades changed.
- job_id=NNN: B → A, reason: "Q3b career-axis match is direct
  (Nyquestro lock-free + Cernio async pipelines on the same axis
  as the role's stated platform work) — reference B rows are
  off-axis adjacent fits, this is on-axis."
- job_id=NNN: SS → S, reason: "Reference SS rows cite ≥2 active
  projects per Q3a; this row only cites 1. Adjustments here move
  the row to S; Q3a rewritten to name the second project."

Confirmations: K grades reviewed and held.
- The remaining (N - M - K) rows were not flagged by the
  comparison and were held without explicit review.
```

If M = 0 (no adjustments made), the section still emits with `Adjustments: 0 grades changed` plus a one-line confirmation that the comparison was run. Silent omission of the section fails Inviolable Rule 9.

### 12. Report batch results

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

Flag the evidence_basis breakdown: how many rows graded via `jd`, how many via `semantic`, how many `insufficient`. Report the portfolio-gaps update — what was added, what was null. Report the Relativity Pass delta (adjustments made vs grades held). Ask the user whether to continue with the remaining queue or stop.

### 13. Declare what was skipped

Close the batch report with a "What I did not do" section covering: jobs left at `pending` with `evidence_basis = 'insufficient'` (no JD AND no usable semantic context, grade NULL); jobs the orchestrator deferred to a later batch (with reason — low company grade + generic title + weak role-type alignment); portfolio-gap patterns the orchestrator noticed but did not write to `portfolio-gaps.md` because they appeared only once (single occurrences are not patterns); rows the Relativity Pass flagged for review that were held without adjustment (with reason); any subagent that returned incomplete output (missing SQL for assigned jobs, assessments with missing Q-slots). If every assigned job was graded, every Q-slot is populated, and every portfolio-gap pattern was written, say so explicitly.

---

## Reference Loading

**Mandatory-core — read at skill invocation every time:**

- `references/grading-rubric.md` — six-tier rubric, dimensions, weights, worked examples, boundary cases, the description-citation rule
- `references/profile-context.md` — how to read the profile for job grading (not the profile itself)
- `references/prioritisation-guide.md` — compound-signal batch ordering: `company_grade × title_promise × role_type_alignment`

All three are read at invocation. The rubric alone without profile-context produces rubric-correct grades with unspecific reasoning; prioritisation alone orders a queue of shallow grades; none of the three is optional.

---

## Inviolable Rules

1. **Try a JD-grounded grade first; fall back to the semantic-reasoning path explicitly.** If `raw_description` is missing / under 100 words / vague, fetch via WebFetch or WebSearch. If still no description, the row takes either the semantic-reasoning path (`evidence_basis = 'semantic'`, brand-strong roles graded from company + role-title context, Q3a names the JD substitution explicitly) or the insufficient-evidence path (`evidence_basis = 'insufficient'`, grade NULL, evaluation_status stays 'pending'). The skill does NOT silently brand-stamp grades from company name alone — every grade is anchored on either a JD quote or an explicit semantic-context substitution.
2. **The fit assessment is structured prose, not a paragraph.** Every row has Q1, Q2, Q3a, Q3b, Q4, Q5, and Verdict slots, in that order. Each slot is prose (2-4 sentences; Q3a+Q3b are the load-bearing slots and may be longer; Verdict is 3-5 sentences). On the JD path, Q1 includes a verbatim JD quote about seniority, and Q3a includes a verbatim JD quote about technology. On the semantic path, those quote slots are replaced by an explicit "JD unavailable — reasoning from {company}'s known {pipeline-shape}". Single-paragraph fit_assessments fail this rule.
3. **No verdict enums anywhere in the assessment.** "Q1 cleared-decisively", "Q3 moderate", "Q5 ✓", arrow-shorthand derivations like "Q2 strong → A", invented categories like "Standard B for mid-tier fintech junior" — all fail. Every Q-slot is prose reasoning; the grade letter follows from the Verdict prose, not from a label-string aggregation. The skill's purpose is to push the agent into reasoning, not classification.
4. **SS / S / A fit assessments name specific profile elements by name in their Q-slots.** Projects from `profile/projects/` with file names and active-status anchors, technologies from `skills.md`, visa facts from `visa.md`, career targets from `preferences.toml`. Profile skill-band labels ("React Comfortable level") are profile metadata, not evidence — name the project that demonstrates the skill.
5. **Q3 is split into two slots: Q3a stack overlap and Q3b career-axis match.** Q3a answers "does the technology overlap?". Q3b answers separately "does this role build toward the engineer Caner is becoming?". Conflating them is the failure mode that produced the pure-frontend / data-analyst / generic-data-engineer-in-B problem. Q3b reasons in prose about career trajectory; "on-axis" / "adjacent" / "off-axis" is the agent's conclusion-in-prose, never a label to pick.
6. **Profile is read fresh every invocation.** No caching, no embedded snapshots.
7. **Grades are calibrated against DB anchors, not the current batch.** A batch of genuinely excellent jobs produces excellent grades — no distribution flattening. The Relativity Pass (step 11) is the structural defence: after grading, each agent compares its rows against 3 random DB-sampled jobs per tier and adjusts inconsistencies.
8. **`profile/portfolio-gaps.md` is updated after every batch.** Even a null update ("no new patterns this batch") is written — silent skipping breaks the career-coaching loop.
9. **Subagents receive full profile + full reference content verbatim, AND the Relativity Pass query verbatim.** Under-contextualised subagents produce shallow assessments; subagents without the Relativity Pass query cannot run step 11.
10. **Exact SQL column names.** `grade`, `fit_assessment`, `evaluation_status`, `evidence_basis`. The `fit_score` column was dropped from the schema — UPDATE statements that reference it will fail at execution. `evaluation_status` maps to the six-tier table in step 6; `evidence_basis` is always set ('jd' / 'semantic' / 'insufficient').
11. **No mechanical company-to-grade or role-type-to-grade rules.** Grading is AI reasoning, not classification. The rubric does NOT contain rules like "company X = grade Y", "frontend role = max C", "junior + top brand = SS", or any threshold mapping from a single attribute to a grade. Every grade emerges from prose Q-slot reasoning plus the Verdict. If a rubric edit ever introduces such a rule, it is an inviolable-rule violation and must be reverted.
12. **Banned strings inside Q-slot prose** (this is the enumerated list of verdict-enum labels and arrow-shorthand patterns that must NOT appear anywhere inside Q1, Q2, Q3a, Q3b, Q4, Q5, or Verdict prose): `cleared-decisively`, `cleared-with-friction`, `real-headwind`, `hard-fail`, `Q1 cleared`, `Q2 strong`, `Q2 moderate`, `Q2 weak`, `Q3 moderate`, `Q3 strong`, `Q3 weak`, `Q5 ✓`, `Q4 ✓`, `→ A`, `→ B`, `→ C`, `→ S`, `→ SS`, `→ F`, `-> A`, `-> B`, `-> C`, `-> S`, `-> SS`, `-> F`. Each is a label or shortcut that bypasses prose reasoning. The agent expresses the same content in prose without using these strings. The list is extended whenever a new shortcut pattern is observed.
13. **Q3a includes all three citation elements.** Per the structured-prose format spec in step 5, Q3a's prose contains (a) a verbatim JD quote of ≥6 chars in double-quotes naming a technology, (b) a profile project file path (e.g. `projects/nyquestro.md`), and (c) a one-line evidence of what the project demonstrates relative to the JD technology. Q3a without all three of these is incomplete.
14. **Verdict slot contains all four framing elements.** Per the structured-prose format spec in step 5, the Verdict prose contains (a) the strongest pull, (b) the strongest pushback, (c) one role-type framing from the enumerated list (`career launch` / `axis bet` / `credibility builder` / `stretch` / `deadweight`) stated verbatim, and (d) a prose answer to the budget-of-30 question. A Verdict missing any of these is incomplete.
15. **Batch size does NOT change grading rigor.** A 30-job or 60-job batch produces individually-reasoned assessments at the same depth as a 10-job batch. The grader never satisfices on a distribution shape (no implicit budgeting like "I've already given 5 S grades, the rest should be lower") and never reaches for a grade because the batch's surrounding jobs make a particular grade feel structurally appropriate. Each job is graded on its own merits against the rubric — period. Larger batches mean more individual prose, not lower per-row reasoning density. If a 30-job batch produces noticeably higher SS/S concentrations than a 10-job batch with similar composition, the larger batch is the suspect; the grader re-reads the Verdict prose for each high-grade row and checks whether it would still hold the grade if the row were graded in isolation.
16. **Risks named in any Q-slot are engaged in the Verdict prose.** If Q3b names an off-axis or career-axis-friction risk, the Verdict prose names the risk explicitly and weighs it. If Q5 names a sponsorship gap or location concern, the Verdict prose engages with it. A risk named in a Q-slot but absent from the Verdict is risk-decoration — the failure mode the §Risks-That-Bite anchor section warns against. The Verdict's engagement with the risk is what makes the risk bite the grade.
17. **Every per-job output ends with two literal lines: `evidence_basis: <value>` and `Grade: <letter>`.** The `<value>` is one of `jd` / `semantic` / `insufficient`. The `<letter>` is one of `SS` / `S` / `A` / `B` / `C` / `F`, or `NULL` when evidence_basis is `insufficient`. These two lines are the machine-readable footer per assessment — they appear regardless of batch size, regardless of whether the grade is high or low, regardless of how terse the invocation prompt is. Verdict prose alone does NOT satisfy this rule; the literal `evidence_basis:` and `Grade:` lines are mandatory format anchors.
18. **The rubric's worked-example narration vocabulary is rubric-internal teaching language, not slot-prose template.** The Q-slot prose the agent writes for a real assessment never contains the rubric's narration tokens (`cleared decisively`, `cleared with friction`, `real headwind`, `hard fail`, `Q1 cleared`, `Q2 strong / moderate / weak`, `Q3 strong / moderate / weak`, `Q5 ✓`, arrow-shorthand `→ A` / `→ S` / `→ SS` / `→ B` / `→ C` / `→ F`, or any other label-pick or formula-shorthand). The rubric uses these tokens to teach a reader how reasoning aggregates; the agent expresses the same content as prose reasoning in its Q-slots. Inviolable Rule 12 bans these tokens; this rule names *why* the rubric appears to use them (teaching language for the reader) and reaffirms the agent's prose must not copy them.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. An item that cannot be evidenced in the agent's output is either unmet and surfaced under step 12 "What I did not do," or the skill has not finished.

- [ ] **All files in `profile/` read fresh this invocation** — cite the tool call per file (this includes every per-project file in `profile/projects/` and `profile/projects/index.md`).
- [ ] **All three reference files read fresh this invocation** — cite the tool call per file.
- [ ] **`cernio format` run at step 0** — the row-count summary appears in chat before step 1. If zero rows were touched, the "already clean" declaration is stated explicitly; silence fails this item.
- [ ] **Calibration anchors pulled from the DB** — cite the SQL query run and reproduce the rows returned (2–3 per tier); the same block appears in every subagent's prompt.
- [ ] **Every subagent prompt embeds the full profile + three reference files verbatim** — verifiable by inspecting the prompt contents in the transcript.
- [ ] **Every graded row has either a JD-grounded grade or an explicit semantic-reasoning grade** — for `evidence_basis = 'jd'` rows, the transcript shows the description was read (fetched or already present, ≥100 words). For `evidence_basis = 'semantic'` rows, the Q3a slot of the fit_assessment explicitly states "JD unavailable — reasoning from {company}'s known {pipeline-shape}" and the Q-slot reasoning anchors on that pipeline-shape plus the candidate's matching portfolio. Rows that cannot satisfy either path are written with `evidence_basis = 'insufficient'`, `grade = NULL`, `evaluation_status = 'pending'` — and are listed in step 13.
- [ ] **Every fit_assessment is structured prose with all seven slots populated** — Q1, Q2, Q3a, Q3b, Q4, Q5, Verdict. Single-paragraph assessments, missing slots, or label-only slots ("Q3 moderate", "Q5 ✓") fail this item. Verify by grepping a sample of assessments for the slot headers and confirming each carries prose.
- [ ] **No verdict-enum strings inside Q-slots** — grep the batch's fit_assessments for the banned strings: "cleared-decisively", "cleared-with-friction", "real-headwind", "hard-fail", "Q1 cleared", "Q2 strong", "Q3 moderate", "Q5 ✓", and arrow-shorthand patterns like "→ A". Any hit indicates a slot has degenerated from prose to label; rewrite the affected slot.
- [ ] **SS / S / A fit assessments name specific profile elements in their Q-slots** — each SS / S / A assessment names at least one project from `profile/projects/` (with file name), one technology from `skills.md`, one fact from `visa.md`, one career target from `preferences.toml`, AND one lifestyle anchor from `lifestyle-preferences.md` (named neighbourhood like Kings Cross / Nine Elms / Canary Wharf / Croydon — "good area" does not satisfy). The specific element name appears in the prose, not a profile-band label.
- [ ] **Seniority JD quote present (or semantic substitution)** — every JD-path fit_assessment's Q1 slot contains a quoted JD fragment about experience / years / seniority OR the literal phrase "No experience requirements mentioned in the description". Semantic-path assessments substitute with "JD unavailable — reasoning from {company}'s known {pipeline-shape}".
- [ ] **Technology JD quote present in Q3a (or semantic substitution)** — every JD-path SS / S / A assessment's Q3a slot contains a quoted JD fragment naming a required technology or stack element. Semantic-path assessments substitute with the company-pipeline reasoning.
- [ ] **Q3b career-axis reasoning is separate from Q3a** — every SS / S / A assessment has a Q3b slot that reasons in prose about the role's career-trajectory match (on-axis / adjacent / off-axis is the conclusion-in-prose, NOT a label pick). Q3b cannot be empty or reduce to "see Q3a".
- [ ] **Sponsorship citation present in Q5** — every Q5 slot either cites the company's sponsorship evidence (sponsor register, description language, prior DB evidence) or names the sponsorship question as open and flags the job as needing verification.
- [ ] **Verdict slot reads as a budget-of-30 answer** — the Verdict prose names the strongest pull AND strongest pushback, classifies the role-type (career-launch / axis-bet / credibility-builder / stretch / deadweight in prose, not by label), and answers whether the role makes the budget cut. The grade letter follows the Verdict.
- [ ] **C / F assessments cite the specific dealbreaker** — named quoted JD text (seniority gap, technology mismatch, role-type exclusion) or the company-context dealbreaker on the semantic path. "Bad fit" without citation fails.
- [ ] **Column mapping correct** — UPDATE statements set `grade`, `fit_assessment`, `evaluation_status`, `evidence_basis` exactly. `evaluation_status` maps SS/S → `strong_fit`, A/B → `weak_fit`, C/F → `no_fit`. `evidence_basis` is one of 'jd' / 'semantic' / 'insufficient'. No UPDATE includes `fit_score = ...` — the column has been dropped and including it fails the statement.
- [ ] **Relativity Pass ran and the delta summary was emitted** — step 11's section is present in the batch output. The DB-sampled reference rows query was run; adjustments are listed by job_id with reason, or `Adjustments: 0 grades changed` is stated explicitly with a one-line confirmation that the comparison was run.
- [ ] **`profile/portfolio-gaps.md` updated** — the diff to `portfolio-gaps.md` is cited (new patterns with counts + role names + companies, or a dated null-result note). Silent omission fails Inviolable Rule 8.
- [ ] **Batch report includes evidence_basis breakdown** — counts of `jd` / `semantic` / `insufficient` rows in this batch, plus the standard tier breakdown and SS / S highlights.
- [ ] **Step 13 "What I did not do" declaration emitted** — names rows left at `insufficient_evidence`, deferred jobs, pattern-threshold misses, Relativity-Pass-flagged-but-held rows, or subagent-output issues, or explicitly states none.

---

## Additive-Freedom Permission for Prescribed Lists

The lists in this SKILL.md are non-exhaustive and may be extended on a per-run basis when a specific batch's shape calls for an addition the skill's author did not anticipate. Additions are pure-additive — they raise the floor of the skill's rigour, never weaken it.

- **The Mandatory Reads table (4 items)** is the minimum precondition set. If a future grading session needs an additional precondition (e.g. a per-job calendar-deadline read from a tracking system, recruiter context from a CRM), add a row. Existing reads stay mandatory.
- **The 13-step workflow** is the current sequential structure. Add new steps when a new mandatory phase surfaces across multiple sessions (recurrence threshold ~3 sessions). Existing steps remain mandatory.
- **The Inviolable Rules (16 rules)** are the current structural constraints. If a new constraint surfaces (e.g. a new data-quality invariant the grader must honour), add Rule 17 or higher. Existing rules stay inviolable.
- **The banned-strings list (Rule 12)** is the observed-in-practice set of verdict-enum labels and arrow-shorthand patterns. New shortcut patterns observed in agent output are appended — never removed.
- **The role-type framings in Verdict (Rule 14)** are the current 5-element set (`career launch` / `axis bet` / `credibility builder` / `stretch` / `deadweight`). New framings may be added when a genuinely new role-type emerges; existing framings remain the verbatim-cite vocabulary.
- **The Quality Checklist items** are the current verifiable obligations. New items may be added; existing items remain mandatory.
- **The six grade tiers (SS / S / A / B / C / F) in `references/grading-rubric.md`** are the current letter system; no new letters are added as escape hatches (no "S+", no "A-stretch", no sub-tiers). See the rubric's Additive-Freedom Permission section for the full per-list permission breakdown that applies to its prescribed lists.
- **The structured fit_assessment slots (Q1 / Q2 / Q3a / Q3b / Q4 / Q5 / Verdict)** are the current minimum set. New slots may be added when a new analytical question becomes load-bearing across multiple sessions. Existing slots remain mandatory and prose-only.
- **The three `evidence_basis` values ('jd' / 'semantic' / 'insufficient')** are the current set. New values may be added when a new evidence-quality category genuinely emerges (e.g. 'partial_jd' for descriptions that loaded but were truncated). Existing values keep their semantics.

For all five lists above, additions are **strictly additive** — they may not introduce conditionals that gate existing requirements, weaken any existing item, or create escape hatches that let the grader skip prescribed work. Document the addition in the next commit's message so future readers can see the extension trail.
