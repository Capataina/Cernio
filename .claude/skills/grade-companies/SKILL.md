---
name: grade-companies
description: "Grades ungraded companies in the Cernio database (S / A / B / C) against the user's profile using a calibration-anchored rubric. Writes `what_they_do` (3–5 specific sentences, no stale content), `location`, `sector_tags`, `grade`, `grade_reasoning` (must cite specific profile projects / technologies / preferences by name), `why_relevant`, `relevance_updated_at`, `graded_at`. Weighs engineering reputation, technical alignment, sponsorship capability, career ceiling, and growth — not numeric floors or within-batch distribution. Invoke on 'grade companies', 'grade the ungraded', 'evaluate companies', 'rate the universe', 'score the database', 'assess companies', 'update company grades', 'grade the pending ones', or after populate-db adds companies with `grade IS NULL`. Not for grading jobs (grade-jobs), discovery (discover-companies), populating (populate-db), resolving ATS (resolve-portals), or staleness audits (check-integrity). Use whenever ungraded companies exist, even if not named."
---

# Grade Companies

> [!warning] LANE-BASED RELATIVITY REFACTOR IN-PROGRESS (2026-05-28)
> Per `context/plans/cernio-full-refactor.md §5.5`. DB schema is already migrated. Until the full phased rewrite lands:
>
> 1. **Sponsor-only universe**: verify `sponsors_uk = yes` before grading. Companies that don't sponsor get rejected, not graded.
> 2. **Assign `lanes` array**: which of the 8 lanes (`big-tech`, `ai-ml`, `hft`, `crypto-mm`, `bank-strats`, `systems-infra`, `devtools`, `fintech`) does this company have teams in. Multi-tag.
> 3. **Set `pinnacle_status_per_lane`**: per lane the company tags, classify as `pinnacle` / `strong` / `adjacent` / `borderline`. This positioning is what jobs grade against.
> 4. **Generalised company `grade` stays single-scale**: employer quality (SS/S/A/B/C/F) — separate from per-lane pinnacle positioning. Both fields are needed.
> 5. **Phased structure (target)**: Phase 1 parallel initial pass (lanes + pinnacle status + sponsors_uk + employer grade) → Phase 2 within-lane relativity cross-check on pinnacle status → Phase 3 no-lane deletion sweep (cascade-deletes orphaned jobs).
> 6. **No hardcoded calibration anchors.** Pinnacle ranking emerges from Phase 2 comparison, not from a hardcoded "Anthropic is SS-in-ai-ml" list.
> 7. **Reads `profile/career-goals.md`** for the 8 active lanes and hard rules.

## Phased Structure (canonical, post-refactor)

### Phase 1 — Initial parallel pass

- Input: companies where `grade IS NULL` OR `lanes IS NULL` OR `sponsors_uk IS NULL`
- Process: 15–20 parallel Opus agents, each handling ~30 companies
- Per company per agent:
  - Verify `sponsors_uk` via Gov.uk Skilled Worker register + careers-page check; if cannot verify, mark `unknown` for resolve fallback
  - Assign `lanes` array from research (which of 8 lanes the company has teams in; multi-tag where genuine)
  - Initial `pinnacle_status_per_lane` per tagged lane (pinnacle / strong / adjacent / borderline)
  - Generalised employer `grade` (SS/S/A/B/C/F) — separate from per-lane positioning
  - Prose `grade_reasoning` citing profile/career-goals.md + profile/skills/* evidence
- Output: company rows with lanes, pinnacle_status_per_lane, sponsors_uk, grade, grade_reasoning

### Phase 2 — Within-lane relativity cross-check

- Input: all companies, grouped by tagged lanes
- Process: ~10 parallel agents, each pulls random sample of 50 companies spanning multiple lanes
- Per agent:
  - Compare within-lane pinnacle assignments for consistency
  - Within-lane percentile bands: top ~10% pinnacle, next ~25% strong, next ~40% adjacent, remainder borderline
  - Adjust `pinnacle_status_per_lane` where similar firms have inconsistent classifications
  - The calibration is emergent — no hardcoded "Anthropic is SS-in-ai-ml" anchor
- Output: refined `pinnacle_status_per_lane` per company

### Phase 3 — No-lane deletion sweep

- Run `cernio clean` (or equivalent SQL) to delete companies with empty `lanes` array
- Cascade-delete their jobs
- Surface deletion summary: company names + grade + reason (could not place in 8 lanes; declassified per plan)
- If the deletion volume is large (>5% of universe), surface for review before sweep

### Phase 4 — Commit + handoff to grade-jobs

- Commit with per-lane company count delta versus prior batch
- Note in commit body which companies migrated pinnacle_status_per_lane bands
- Trigger downstream grade-jobs Phase 1 for any jobs at newly-tagged companies

Grades companies in the Cernio database against the user's profile. Each grade is a reasoned position on one question: **is this company worth monitoring for jobs?** The answer weighs engineering reputation, technical alignment with the specific project portfolio, sponsorship capability against the visa timeline, career ceiling against the long-term trajectory, and growth / stability — calibrated against real anchor companies already graded in the database.

Grades are not permanent. They are snapshots tied to the current profile state. When the profile changes — a new flagship project, a visa-status shift, revised preferences — previously-assigned grades become potentially stale and the `check-integrity` skill surfaces them for re-grading. This skill writes the grade that is correct *right now*, not a grade that is assumed to survive profile evolution.

---

## Mandatory Reads Before Grading Any Company

| # | What | Evidence that the read happened |
|---|---|---|
| 1 | **Every file in `profile/`** — no exceptions, including every per-project file in `profile/projects/` | The grade reasoning written to the DB cites at least two specific profile elements by name (project from `profile/projects/`, technology from `skills.md`, preference from `preferences.toml`, visa fact from `visa.md`, etc.) |
| 2 | **`references/grading-rubric.md`** (full file) | You can cite the four core questions, the calibration-anchored method, the specific failure mode from "Common Grading Errors" that applies to any marginal call you make, and the worked example you used as an anchor |
| 3 | **`references/profile-context.md`** (full file) | You can name the profile file and field you pulled each piece of evidence from when writing `why_relevant` |

The profile is not snapshotted anywhere in this skill, in either reference file, or in any subagent prompt template. Every invocation reads `profile/` fresh — hardcoded profile data (visa expiry, project names, degree classification) would diverge silently from reality and produce incorrect grades. If the profile is not read fresh this invocation, the grading judgment is not grounded.

When delegating grading to parallel subagents, every subagent prompt embeds:

- The **full content of both reference files**, verbatim (agents cannot read the skill's references)
- The **full content of every file in `profile/`** (agents cannot read the profile)
- The **calibration anchors** pulled from the database at the start of this grading session — 2–3 real examples per tier (S / A / B / C) with their `grade_reasoning` (agents cannot query the database)
- **Explicit instruction** to WRITE UPDATE statements to an assigned scratch-path SQL file AND a parallel summary file (see §Subagent Output Contract below), not emit SQL inline in the chat reply

Under-contextualising a subagent produces grades that pattern-match against the agent's pretraining rather than the Cernio rubric. Subagent prompts that summarise the profile or paraphrase the rubric produce tier-accurate grades with profile-unspecific reasoning, failing Inviolable Rule 1's citation requirement — verified in prior production runs.

---

## Subagent Output Contract — Scratch-Path Pattern

Subagents do NOT emit SQL in their chat replies. The orchestrator does not transcribe SQL out of agent messages. Instead, every grading subagent writes two files to a shared scratch directory:

```
/tmp/grade-companies-<run-id>/
├── <batch-id>.sql         ← single-line UPDATE statements, one per company in the batch
└── <batch-id>.summary.md  ← grouped-by-tier human-readable summary for orchestrator review
```

Where:
- `<run-id>` is a session-stable identifier (e.g. epoch timestamp at orchestrator start). The orchestrator picks it once and embeds the full scratch directory path in every subagent prompt.
- `<batch-id>` is the agent's assigned slug (`aa`, `ab`, ...). One agent owns one batch-id and writes exactly two files for it.

**Per-agent file contract:**

`<batch-id>.sql` — pure SQL. Single-line UPDATE per company; semicolon-terminated. Exact column names per Rule 7. Single quotes escaped by doubling. Timestamps via `datetime('now')`. No markdown fences, no commentary lines, no SQL comments. The file is concatenable with every other agent's .sql via `cat`.

`<batch-id>.summary.md` — orchestrator-facing verification artefact. Frontmatter line `companies: N` declares how many companies the agent graded; followed by one section per tier (S / A / B / C) with `**Name**` heading, the grade letter, and a one-line verdict. This is what the orchestrator reads to verify the agent did the work (row counts match the assigned batch size, tier distribution is plausible, no missing companies). NOT for human review — skills are autonomous; the orchestrator validates and applies.

**Orchestrator flow:**

1. Pick `run-id`, `mkdir -p /tmp/grade-companies-<run-id>/`, embed the path in every subagent prompt.
2. Dispatch all subagents in parallel.
3. As agents finish, verify per batch:
   - Both `<batch>.sql` and `<batch>.summary.md` exist; missing = redispatch.
   - Summary frontmatter `companies: N` equals the assigned batch size; mismatch = redispatch.
   - SQL line count equals N; mismatch = redispatch.
   - SQL parses: `sqlite3 :memory: ".read /tmp/grade-companies-<run-id>/<batch>.sql"` returns no errors against an empty schema, OR a dry-run on a temp copy succeeds.
4. Apply SQL: `cat /tmp/grade-companies-<run-id>/*.sql | sqlite3 state/cernio.db`. No approval gate — the verification in step 3 is the safety net.
5. Post-apply report: one line per tier with counts, plus any anomalies the orchestrator noticed (skewed distribution, repeated reasoning patterns) for the session-end summary.
6. Scratch dir is ephemeral.

This pattern keeps the orchestrator's context burn flat regardless of batch count, localises SQL escaping bugs to one file, and keeps the skill autonomous start-to-finish per the no-mid-run-pause-points contract.

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
| `grade_reasoning` | Paragraph explaining the grade | Must name at least two specific profile elements (project from `profile/projects/`, technology from `skills.md`, preference from `preferences.toml`, visa constraint, career target). Must explain why this grade and not the adjacent tier. Must acknowledge any ambiguity where signal is weak. |
| `why_relevant` | Paragraph connecting the company to the profile | Must name at least one specific flagship project or technology by name. Generic phrases like "good alignment" fail this bar. Example that passes: "Nyquestro's lock-free matching engine maps directly to their exchange infrastructure; Aurix's risk modelling connects to their derivatives pricing." |
| `relevance_updated_at` | `datetime('now')` | Auto-set on grade write |
| `graded_at` | `datetime('now')` | Auto-set on grade write |

Grade reasoning that names no specific profile element fails this skill's output bar. The reason is written in CLAUDE.md's Grade and Fit Assessment Quality Standard and the research in `references/profile-context.md`: a grade not grounded in specific evidence is a grade that cannot be checked by the user and cannot be re-evaluated by a future integrity check. The verifier is "can a third party read this reasoning and name the profile elements cited?"

### 5. C-tier companies stay active — do not archive

C-tier companies remain in the active search pool. The cost of searching a few low-signal companies is small; the cost of missing a single high-quality role at a "marginal" company is unrecoverable. Job grading downstream filters the noise — one genuinely good role at a C-tier company is still graded A or B at the job level.

Archival is a separate decision driven by hard exclusions (company is in an excluded sector from `preferences.toml`, company has dissolved, company has no engineering team at all) — not by a C grade. The SQL for C-tier writes is identical to S / A / B except for the `grade` value.

### 6. Verify scratch artefacts

After every dispatched subagent has written its `<batch>.summary.md` and `<batch>.sql` to the scratch directory, the orchestrator runs the verification checks listed in the §Subagent Output Contract Orchestrator flow step 3: file presence, summary-frontmatter row count equals batch size, SQL line count matches, SQL parses cleanly. Any batch failing a check is redispatched with the same prompt — not silently dropped. This is mechanical verification, not user review; the skill remains autonomous.

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

### 7. Write to the database

After verification passes, the orchestrator applies the agent-written SQL files in one shot:

```bash
cat /tmp/grade-companies-<run-id>/*.sql | sqlite3 state/cernio.db
```

The orchestrator does NOT hand-write or re-emit SQL. The .sql files on disk are the source of truth; the orchestrator's only job here is concatenation + execution.

**Column-name and format contract (enforced inside every agent's .sql file):** columns are `what_they_do`, `location`, `sector_tags`, `grade`, `grade_reasoning`, `why_relevant`, `relevance_updated_at`, `graded_at`. Not `reasoning`, not `description`, not `relevance`. Single quotes doubled (`it''s`). One UPDATE per line, semicolon-terminated. Timestamps via `datetime('now')`. Subagent prompts repeat this contract verbatim so the written .sql files conform.

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
- `references/profile-context.md` — how to read the profile for grading (not the profile itself): what to extract from each profile file (including every per-project file in `profile/projects/`), what synthesis to build, what profile elements to cite in grade reasoning. Explicitly forbids embedding profile snapshots. Includes TOC.

Both files are read at invocation — not one or the other. The rubric without the profile-context produces rubric-correct grades with unspecific reasoning; the profile-context without the rubric produces specific reasoning against an ungrounded scale.

---

## Inviolable Rules

1. **Every grade reasoning cites at least two specific profile elements by name.** Project from `profile/projects/`, technology from `skills.md`, preference from `preferences.toml`, visa constraint from `visa.md`, career target — at least two named, no generic phrasing like "good alignment" or "strong match." The third-party verifier check is "can a reader point at the profile elements cited?"
2. **The profile is read fresh every invocation.** No hardcoded profile values in this skill or its references. The profile is a living document and snapshots go stale silently.
3. **Grades are calibrated against DB anchors, not against the current batch.** A batch of ten excellent companies produces ten high grades. Within-batch distribution enforcement is a grading error.
4. **C-tier companies stay active.** Setting `status = 'archived'` on a C-grade write is a rule violation. Archival has separate triggers.
5. **`what_they_do` excludes stale content.** No headcounts, no funding amounts, no "recently launched" news, no employee counts. Only what the company fundamentally is and does.
6. **The skill is autonomous start-to-finish.** No mid-run user-approval gate. Mechanical verification of scratch artefacts (file presence, row-count match, SQL parse) is the safety net before `sqlite3` applies the batch. The orchestrator surfaces anomalies in the post-run report; it does not pause mid-skill for approval.
7. **Exact SQL column names.** `what_they_do`, `location`, `sector_tags`, `grade`, `grade_reasoning`, `why_relevant`, `relevance_updated_at`, `graded_at`. No variants.
8. **Subagents receive full profile + full reference content verbatim in their prompt.** Subagents cannot read project files. Under-contextualising them produces ungrounded grades.
9. **Subagents write SQL + summary to the assigned scratch path; orchestrator never transcribes SQL from chat.** Per the Subagent Output Contract section, every agent writes `<batch>.sql` and `<batch>.summary.md` to `/tmp/grade-companies-<run-id>/`. Agents that emit SQL inline in their chat reply have violated the contract; the orchestrator redispatches with the prompt re-emphasising the file-write instruction. The orchestrator applies SQL via `cat .../*.sql | sqlite3`, never by retyping or hand-aggregating.

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
- [ ] **SQL format:** exact column names, single-line UPDATE statements, single quotes escaped by doubling, `datetime('now')` for both timestamps — enforced inside every agent-written `<batch>.sql` file
- [ ] **Scratch-path file contract honoured:** every dispatched batch has BOTH `/tmp/grade-companies-<run-id>/<batch>.sql` AND `<batch>.summary.md` present on disk; missing files redispatched, not silently dropped
- [ ] **Orchestrator did not hand-write or re-emit SQL:** the SQL applied to the DB is `cat /tmp/grade-companies-<run-id>/*.sql | sqlite3 state/cernio.db`; no Write or Edit calls produced .sql files during this run
- [ ] **Mechanical verification ran before DB apply** — per-batch file-presence, summary `companies: N` vs assigned batch size, SQL line count, and SQL parse-check all passed; failing batches were redispatched, not silently applied
- [ ] **Step 8 "What I did not do" declaration emitted** — names ungraded-with-reason companies, marginal-call tie-break citations, hard-exclusion recommendations, or explicitly states "every queued company graded cleanly"
