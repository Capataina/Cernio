---
name: grade-companies
description: "Grades ungraded companies in the Cernio database (S / A / B / C) against the user's profile using a calibration-anchored rubric. Writes `what_they_do` (3–5 specific sentences, no stale content), `location`, `sector_tags`, `grade`, `grade_reasoning` (must cite specific profile projects / technologies / preferences by name), `why_relevant`, `relevance_updated_at`, `graded_at`. Weighs engineering reputation, technical alignment, sponsorship capability, career ceiling, and growth — not numeric floors or within-batch distribution. Invoke on 'grade companies', 'grade the ungraded', 'evaluate companies', 'rate the universe', 'score the database', 'assess companies', 'update company grades', 'grade the pending ones', or after populate-db adds companies with `grade IS NULL`. Not for grading jobs (grade-jobs), discovery (discover-companies), populating (populate-db), resolving ATS (resolve-portals), or staleness audits (check-integrity). Use whenever ungraded companies exist, even if not named."
---

# Grade Companies

Grades companies in the Cernio database against the user's profile. Each grade is a reasoned position on one question: **is this company worth monitoring for jobs?** The answer weighs engineering reputation, technical alignment with the specific project portfolio, sponsorship capability against the visa timeline, career ceiling against the long-term trajectory, and growth / stability — calibrated against real anchor companies already graded in the database.

Grades are not permanent. They are snapshots tied to the current profile state. When the profile changes — a new flagship project, a visa-status shift, revised preferences — previously-assigned grades become potentially stale and the `check-integrity` skill surfaces them for re-grading. This skill writes the grade that is correct *right now*, not a grade that is assumed to survive profile evolution.

---

## Mandatory Reads Before Grading Any Company

| # | What | Evidence that the read happened |
|---|---|---|
| 1 | **Every file in `profile/`** — 15 files, no exceptions | The grade reasoning written to the DB cites at least two specific profile elements by name (project from `projects.md`, technology from `skills.md`, preference from `preferences.toml`, visa fact from `visa.md`, etc.) |
| 2 | **`references/grading-rubric.md`** (full file) | You can cite the four core questions, the calibration-anchored method, the specific failure mode from "Common Grading Errors" that applies to any marginal call you make, and the worked example you used as an anchor |
| 3 | **`references/profile-context.md`** (full file) | You can name the profile file and field you pulled each piece of evidence from when writing `why_relevant` |

The profile is not snapshotted anywhere in this skill, in either reference file, or in any subagent prompt template. Every invocation reads `profile/` fresh — hardcoded profile data (visa expiry, project names, degree classification) would diverge silently from reality and produce incorrect grades. If the profile is not read fresh this invocation, the grading judgment is not grounded.

When delegating grading to parallel subagents, every subagent prompt embeds:

- The **full content of both reference files**, verbatim (agents cannot read the skill's references)
- The **full content of every file in `profile/`** (agents cannot read the profile)
- The **calibration anchors** pulled from the database at the start of this grading session — 2–3 real examples per tier (S / A / B / C) with their `grade_reasoning` (agents cannot query the database)
- **Explicit instruction** to output UPDATE statements in the exact SQL format below, not narrative summaries

Under-contextualising a subagent produces grades that pattern-match against the agent's pretraining rather than the Cernio rubric. Subagent prompts that summarise the profile or paraphrase the rubric produce tier-accurate grades with profile-unspecific reasoning, failing Inviolable Rule 1's citation requirement — verified in prior production runs.

---

## Workflow

### 1. Pull calibration anchors from the database

Before touching any ungraded company, pull 2–3 real examples per grade tier from the graded-company universe. These are the calibration anchors — they define what each grade looks like in this specific database.

```sql
SELECT grade, name, grade_reasoning
FROM companies
WHERE grade IS NOT NULL
  AND status != 'archived'
ORDER BY
    CASE grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 WHEN 'B' THEN 3 WHEN 'C' THEN 4 END,
    RANDOM();
```

Select 2–3 per tier. These anchors go into every grading decision and every parallel subagent's prompt. Grading runs that skip this step produce batch-relative grades — a batch of ten genuinely excellent companies gets deflated to "surely these can't all be A." Anchor-based grading prevents that.

### 2. Query ungraded companies

```sql
SELECT id, name, website, what_they_do, status, location, sector_tags, why_relevant
FROM companies
WHERE grade IS NULL
  AND status != 'archived';
```

Companies in `archived` status were already evaluated and set aside — exclude them unless the user explicitly asks for re-grading.

### 3. Research each company to sufficient depth

"Sufficient depth" is calibrated to the company's visibility, not a uniform procedure:

- **Well-known companies (Cloudflare, Stripe, Palantir):** training-data knowledge plus a current-state verification — still hiring, still independent, no recent pivots or layoffs.
- **Lesser-known companies:** visit the website, read the engineering blog, check OSS activity on GitHub, read recent news, check the UK sponsor register, inspect the careers page.
- **Startups with minimal web presence:** triangulate — Crunchbase for funding, LinkedIn for headcount signals, Companies House for registration, GitHub for engineering activity.

The goal is confident placement against the calibration anchors, not exhaustive research. When signal is ambiguous, the grade reasoning states the uncertainty explicitly — a B with acknowledged uncertainty is more honest than a confident grade built on thin evidence.

### 4. Enrich and grade each company

Each company gets the following fields written together — enrichment and grading are one pass, not two:

| Field | Content | Standard |
|---|---|---|
| `what_they_do` | 3–5 sentence paragraph | Specific enough to distinguish the company from every other in its sector. Excludes stale content: no headcounts, no funding amounts, no employee counts, no "recently launched X" news. Only what the company fundamentally IS and DOES. |
| `location` | Engineering office(s) relevant to the candidate | Examples: "London", "London, Bristol", "Remote-UK" |
| `sector_tags` | Comma-separated tags | Example: "trading-systems, derivatives, market-making" |
| `grade` | `S`, `A`, `B`, or `C` | Emerges from the rubric's four core questions + analytical dimensions + calibration-anchor comparison per `references/grading-rubric.md`. |
| `grade_reasoning` | Paragraph explaining the grade | Must name at least two specific profile elements (project from `projects.md`, technology from `skills.md`, preference from `preferences.toml`, visa constraint, career target). Must explain why this grade and not the adjacent tier. Must acknowledge any ambiguity where signal is weak. |
| `why_relevant` | Paragraph connecting the company to the profile | Must name at least one specific flagship project or technology by name. Generic phrases like "good alignment" fail this bar. Example that passes: "Nyquestro's lock-free matching engine maps directly to their exchange infrastructure; Aurix's risk modelling connects to their derivatives pricing." |
| `relevance_updated_at` | `datetime('now')` | Auto-set on grade write |
| `graded_at` | `datetime('now')` | Auto-set on grade write |

Grade reasoning that names no specific profile element fails this skill's output bar. The reason is written in CLAUDE.md's Grade and Fit Assessment Quality Standard and the research in `references/profile-context.md`: a grade not grounded in specific evidence is a grade that cannot be checked by the user and cannot be re-evaluated by a future integrity check. The verifier is "can a third party read this reasoning and name the profile elements cited?"

### 5. C-tier companies stay active — do not archive

C-tier companies remain in the active search pool. The cost of searching a few low-signal companies is small; the cost of missing a single high-quality role at a "marginal" company is unrecoverable. Job grading downstream filters the noise — one genuinely good role at a C-tier company is still graded A or B at the job level.

Archival is a separate decision driven by hard exclusions (company is in an excluded sector from `preferences.toml`, company has dissolved, company has no engineering team at all) — not by a C grade. The SQL for C-tier writes is identical to S / A / B except for the `grade` value.

### 6. Present grouped results for user review

Group graded companies by tier. Show `what_they_do`, `grade`, and `why_relevant` inline. The user reviews and approves before anything is written to the database. The review gate catches miscalibrated grades, wrong-tier placements, and missing profile-element citations before they reach production.

Example:

```
## Grading Results

### S-tier
- **Cloudflare**
  What they do: [3–5 sentence description]
  Grade: S — [reasoning citing specific profile elements]
  Why relevant: [cites Nyquestro, NeuroDrive, specific technology]

### A-tier
- ...

### C-tier
- ...
```

### 7. Write to the database (exact SQL format)

After approval, execute the updates. Column names and format are exact — do not rename, do not add fields, do not multi-line:

```sql
UPDATE companies SET what_they_do = 'description paragraph', location = 'London', sector_tags = 'tag1, tag2', grade = 'X', grade_reasoning = 'reasoning text', why_relevant = 'relevance text naming specific profile elements', relevance_updated_at = datetime('now'), graded_at = datetime('now') WHERE id = N;
```

**Column name contract:** `what_they_do`, `location`, `sector_tags`, `grade`, `grade_reasoning`, `why_relevant`, `relevance_updated_at`, `graded_at`. Not `reasoning`, not `description`, not `relevance`. Column-name drift produces silent DB-level mismatches that downstream queries miss.

**Escaping:** single quotes in text are doubled — `it''s` not `it's`. One statement per line; semicolon-terminated. Timestamps are `datetime('now')`, not hardcoded date strings.

**Do not set `status = 'archived'` for any grade, including C.** Archival is a separate workflow with its own triggers.

### 8. Declare what was skipped

Close the batch with a "What I did not do" section covering: companies where research could not produce sufficient evidence for a confident grade (left ungraded with the reason — dead website, ambiguous signals, no careers page); companies where a marginal-call decision was made between two tiers (name the company, name the tiers considered, cite the anchor comparison that broke the tie); companies flagged as possible hard-exclusions that belong in archival rather than graded (surface them as recommendations; the archival decision is a separate workflow). If every queued company was graded cleanly with no marginal calls and no ambiguous research, say so explicitly.

---

## Regrading

When the user asks to regrade specific companies (new information surfaced, the rubric evolved, or the profile changed significantly):

- Query by name or id rather than by `grade IS NULL`
- Show the previous grade and `grade_reasoning` alongside the new evaluation
- Explain what changed and why the new grade differs, or confirm it if it does not

Regrading uses the same SQL write format — `grade`, `grade_reasoning`, `relevance_updated_at`, `graded_at` are overwritten.

---

## Reference Loading

**Mandatory-core — read at skill invocation every time:**

- `references/grading-rubric.md` — the complete rubric: core questions, analytical dimensions (high / medium / low weight), grade definitions, calibration-anchored grading method, career-stage context, common grading errors, worked examples, evidence standards. 257 lines, includes TOC.
- `references/profile-context.md` — how to read the profile for grading (not the profile itself): what to extract from each of the 15 profile files, what synthesis to build, what profile elements to cite in grade reasoning. Explicitly forbids embedding profile snapshots. 128 lines, includes TOC.

Both files are read at invocation — not one or the other. The rubric without the profile-context produces rubric-correct grades with unspecific reasoning; the profile-context without the rubric produces specific reasoning against an ungrounded scale.

---

## Inviolable Rules

1. **Every grade reasoning cites at least two specific profile elements by name.** Project from `projects.md`, technology from `skills.md`, preference from `preferences.toml`, visa constraint from `visa.md`, career target — at least two named, no generic phrasing like "good alignment" or "strong match." The third-party verifier check is "can a reader point at the profile elements cited?"
2. **The profile is read fresh every invocation.** No hardcoded profile values in this skill or its references. The profile is a living document and snapshots go stale silently.
3. **Grades are calibrated against DB anchors, not against the current batch.** A batch of ten excellent companies produces ten high grades. Within-batch distribution enforcement is a grading error.
4. **C-tier companies stay active.** Setting `status = 'archived'` on a C-grade write is a rule violation. Archival has separate triggers.
5. **`what_they_do` excludes stale content.** No headcounts, no funding amounts, no "recently launched" news, no employee counts. Only what the company fundamentally is and does.
6. **No DB write without user approval of the grouped summary.** The user-review gate is the last correction opportunity before grades reach production.
7. **Exact SQL column names.** `what_they_do`, `location`, `sector_tags`, `grade`, `grade_reasoning`, `why_relevant`, `relevance_updated_at`, `graded_at`. No variants.
8. **Subagents receive full profile + full reference content verbatim in their prompt.** Subagents cannot read project files. Under-contextualising them produces ungrounded grades.

---

## Quality Checklist

- [ ] **Pre-grading:** calibration anchors pulled from the DB (2–3 per tier), visible in the session transcript, embedded in every subagent prompt
- [ ] **Pre-grading:** every file in `profile/` was read this invocation — not from earlier-session memory
- [ ] **Pre-grading:** both reference files (`grading-rubric.md`, `profile-context.md`) were read end-to-end this invocation
- [ ] **Per company:** `what_they_do` is a 3–5 sentence paragraph specific enough that a reader could not confuse this company with another in the same sector; contains no headcounts, funding amounts, or recent-news content
- [ ] **Per company:** `location` and `sector_tags` are filled with specific values, not placeholders
- [ ] **Per company:** `grade_reasoning` names at least two specific profile elements by name (cite which profile file each came from in the session transcript)
- [ ] **Per company:** `grade_reasoning` explains why this grade and not the adjacent tier, citing the specific dimension or question that distinguishes them
- [ ] **Per company:** `why_relevant` names at least one specific flagship project or technology by name — generic "good alignment" phrases fail
- [ ] **Per company:** uncertainty is acknowledged explicitly where evidence is thin; false confidence against weak evidence is flagged and rewritten
- [ ] **No company has `status` set to `archived` by this skill.** C-tier stays active. Archival is a separate decision with separate triggers.
- [ ] **SQL format:** exact column names, single-line UPDATE statements, single quotes escaped by doubling, `datetime('now')` for both timestamps
- [ ] **Results presented grouped by tier** for user review before any DB write happened
- [ ] **User approved** the grouped results before the UPDATE statements ran
- [ ] **Step 8 "What I did not do" declaration emitted** — names ungraded-with-reason companies, marginal-call tie-break citations, hard-exclusion recommendations, or explicitly states "every queued company graded cleanly"
