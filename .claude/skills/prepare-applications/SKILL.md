---
name: prepare-applications
description: "Generates personalised application packages for selected jobs — cover letter, 'why this role', 'why this company', technical-project answer, and other common application questions — grounded in the user's profile, the job description, and the fit assessment. Writes the answers as JSON into the application_packages SQLite table so the TUI autofill flow (press 'p') can prefill the form. Invoke when the user says 'prepare applications', 'prep these jobs', 'get applications ready', 'draft cover letters', 'batch apply', 'fill in answers for these jobs', or names a specific set of jobs they want ready to submit. Not for grading jobs (use grade-jobs), discovering companies (use discover-companies), resolving ATS slugs (use populate-db / resolve-portals), or auditing database health (use check-integrity). Use this skill whenever a user identifies jobs to apply to and wants personalised answers prepared, even if they do not name the skill explicitly."
---

# Prepare Applications

For each selected job, generate a JSON package of tailored answers to common application questions and write it to the `application_packages` SQLite table. The TUI's `p` keypress reads that package and uses Chrome CDP to prefill the application form. This skill produces the judgment half of autofill — the cover letter, the "why this company" response, the technical-project narrative — and hands the mechanical form-filling to the autofill pipeline.

The package is not a template. Every answer references specific projects, skills, and experiences from the profile by name, ties them to what the company actually does and what this role actually demands, and is honest about gaps. Generic answers waste the slot; a recruiter can spot them at a glance.

---

## Mandatory Reads Before Generating Any Answer

These four reads are non-negotiable preconditions. Answers produced without them are discarded.

| # | What to read | Evidence that it was read |
|---|---|---|
| 1 | **Every file in `profile/`** — all files, no exceptions | You can cite the specific project from `profile/projects.md` that grounds the technical-project answer, and the specific skill from `profile/skills.md` that the role asks for |
| 2 | **The job's `raw_description`** from the DB | You can quote two concrete responsibilities or stack elements from the description in the "why this role" answer |
| 3 | **The job's `fit_assessment`** from the DB | Your answer builds on the fit assessment's analysis rather than restating it; the assessment is internal reasoning, the answer is external-facing |
| 4 | **The company's `what_they_do` and `grade_reasoning`** from the DB | The "why this company" answer references what the company actually builds, not generic praise |

If any of the four is skipped, the answer for that question cannot be written — state the skip explicitly and stop rather than produce a generic filler. Profile data is not cached in this skill or in any reference file; it is read fresh from `profile/` on every invocation because the profile is a living document and yesterday's snapshot may not match today's reality.

---

## Workflow

### 1. Identify the target jobs

The user names the jobs to prepare. Common patterns: "prepare applications for all SS and S jobs," "prep these five," "get the top ten ready." Query the database for the full context needed to write answers:

```sql
SELECT j.id, j.title, j.url, j.raw_description, j.fit_assessment, j.grade,
       c.name AS company, c.what_they_do, c.grade_reasoning,
       cp.ats_provider
FROM jobs j
JOIN companies c ON j.company_id = c.id
LEFT JOIN company_portals cp ON cp.company_id = c.id AND cp.is_primary = 1
WHERE j.grade IN ('SS', 'S')              -- substitute user's filter
  AND j.evaluation_status <> 'archived'
  AND j.id NOT IN (SELECT job_id FROM application_packages)
  AND j.id NOT IN (SELECT job_id FROM user_decisions WHERE decision = 'applied')
ORDER BY CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 ELSE 4 END;
```

The `NOT IN` clauses are load-bearing — re-preparing a job that already has a package overwrites existing work, and re-preparing applied jobs wastes effort.

### 2. Read the profile in full

Read every file in `profile/`. Pay particular attention to:

- `projects.md` — flagship projects are the strongest material for the technical-project answer
- `skills.md` — what the candidate can honestly claim
- `experience.md` — work-history context for seniority framing
- `portfolio-gaps.md` — named gaps; the answers must not overclaim against these

### 3. Generate the standard answer set per job

Write answers to this canonical question set. These keys are the contract with the autofill provider module in `src/autofill/`:

| JSON key | Question | Answer shape |
|----------|----------|--------------|
| `why_interested` | "Why are you interested in this role?" | Tie two or three concrete responsibilities from the job description to specific profile projects / skills. Name the projects. |
| `why_company` | "Why do you want to work at [Company]?" | Reference what the company actually builds (from `what_they_do`), the team / product area if the role specifies one, and the alignment with the candidate's trajectory. No "great company" filler. |
| `technical_project` | "Tell us about a technical project you've worked on." | Pick the single most relevant project for this role. One flagship, described in depth — problem, approach, specific techniques, measurable outcome. Not a list of all the projects. |
| `cover_letter` | Cover letter body | 3–4 paragraphs: hook referencing the specific role → technical fit with named evidence → why this company specifically → short close. No "Dear Hiring Manager" boilerplate that every template has. |

### 4. Handle standard factual questions

These appear repeatedly and can be pre-answered deterministically from the profile:

| Question | Source / answer |
|---|---|
| "Are you authorised to work in the UK?" | Read `profile/visa.md`; produce an answer that states the current visa status, expiry, and any sponsorship need |
| "When can you start?" | Default "Immediately" unless the user has stated otherwise; ask if ambiguous |
| "How did you hear about this role?" | "Found through the company careers page" |
| "What are your salary expectations?" | Leave empty — this is a live decision the user owns; do not autofill |

### 5. Write the package to the database

```sql
INSERT OR REPLACE INTO application_packages (job_id, answers, created_at)
VALUES (?1, ?2, datetime('now'));
```

`answers` is a JSON object mapping the keys above (and any extras from the specific job) to the generated text. `INSERT OR REPLACE` makes the operation idempotent — rerunning the skill on the same job updates the package rather than erroring.

### 6. Report the outcome

Tell the user:

- How many packages were created
- Which jobs are now ready (they show the yellow `●` indicator in the TUI)
- Which jobs were skipped and why (already packaged, already applied, no description available)
- The reminder: press `p` on any job with `●` to open Chrome with the form prefilled

---

## The Answer Generation Standard

Every generated answer meets all five bars below. An answer that fails any one is rewritten or the skill admits it could not meet the bar rather than ship generic filler.

**Specific, not generic.**
Weak: "I'm passionate about technology and excited about this opportunity."
Strong: "Your Workers Runtime team's work on V8 isolates for edge compute maps directly to the isolation and low-latency constraints I've worked with in Nyquestro — my lock-free order matching engine processes orders in under 2 microseconds using similar principles of minimal-allocation, zero-copy design."

**Profile-grounded.**
Every answer references at least one specific element from the profile by name — a project, a technology, a concrete experience. The reader should be able to tell this answer could only have been written by this specific candidate.

**Company-aware.**
The "why this company" answer cites what the company actually builds, their stack where known, the specific team or product area named in the job posting. The company's `what_they_do` field is the anchor.

**Honest.**
Do not fabricate experience. If the role asks for Kubernetes and the profile does not have it, do not claim it. Frame adjacent experience instead: "I have not deployed to Kubernetes in production, but my Docker work in local development plus the orchestration concepts from Nyquestro's distributed components give me a strong foundation to learn it quickly."

**Concise.**
2–4 paragraphs per answer. Recruiters read hundreds of these. Dense and specific beats long and generic every time.

---

## Parallelisation

For batches of 5 or more jobs, dispatch parallel subagents — one per 2–3 jobs. The main agent is the orchestrator; subagents generate the JSON and return SQL INSERT statements, which the orchestrator collects and executes.

Each subagent prompt includes:

- **The full content of every file in `profile/`** (the agent cannot read the profile itself)
- The full `raw_description`, `fit_assessment`, `grade`, and company context for each assigned job
- The answer generation standard from this skill file, embedded verbatim
- The JSON key contract (`why_interested`, `why_company`, `technical_project`, `cover_letter`)
- Explicit instruction to output the SQL INSERT statements directly, not a narrative summary

Under-contextualising a subagent produces generic answers that fail the generation standard. Over-share the profile rather than summarise it.

---

## Inviolable Rules

1. **Read the profile fresh every invocation.** Never embed profile content in this file or any reference file. Profile data is a living document and snapshots go stale silently — the moment this skill caches "user has a 2:2 from York" or "user's flagship project is Nyquestro," the cached fact will eventually diverge from reality. The profile is at `profile/`; read it every time.
2. **Never fabricate experience.** Frame gaps honestly with adjacent skills rather than claim capabilities the profile does not support. Detection risk aside, this is non-negotiable because the user will interview on the basis of what this skill wrote.
3. **The four mandatory reads are preconditions, not suggestions.** If any of profile, job description, fit assessment, or company context is unavailable, skip the job with a stated reason. Do not write filler to cover the gap.
4. **`INSERT OR REPLACE` only for the package itself.** Do not modify any other table from this skill — not `jobs`, not `user_decisions`, not `companies`. Applying to a job is a separate user action triggered from the TUI.

---

## Quality Checklist

Before declaring the batch complete:

- [ ] Every answer references at least one specific project or skill from the profile by name
- [ ] The `why_company` answer cites something the company actually builds or a specific team / product area from the job description — not generic praise
- [ ] No fabricated experience: any gap against the job's requirements is framed honestly with adjacent evidence
- [ ] Every answer is 2–4 paragraphs, dense, and specific
- [ ] The cover letter has a clear structure: specific-role hook → technical fit with named evidence → why-this-company → short close
- [ ] The JSON is syntactically valid and the keys match the autofill provider module's expected contract
- [ ] The full profile was read in this invocation — no reliance on earlier-session memory
- [ ] Jobs skipped due to missing description / fit assessment / company context are reported with the specific reason
- [ ] The package was written via `INSERT OR REPLACE` so reruns update rather than error
- [ ] The user was reminded that `p` in the TUI launches the autofill on any job with the yellow `●` indicator
