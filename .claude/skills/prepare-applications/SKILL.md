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

### 0. Run `cernio format` before reading any job description

Step 1's SELECT pulls `raw_description` and `fit_assessment`, and every answer in step 3 is built directly from those strings. Raw HTML in either field — `<p>`, `<strong>`, `&amp;`, `&nbsp;` — leaks into cover letters and "why this role" answers, quoting markup instead of prose; stale HTML in `fit_assessment` degrades the "why company" and technical-project answers the same way. `cernio format` is idempotent and fast; running it at the top of every invocation guarantees the package is built from clean plaintext, not markup fragments.

```bash
cernio format
```

Paste the row-count summary into the chat response. If zero rows were touched, say so explicitly — required evidence that cleanliness is verified this run, not assumed.

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

- How many packages were created, and the job IDs affected.
- Which jobs are now ready (they show the yellow `●` indicator in the TUI).
- Which jobs were skipped and why — one row per skipped job with the specific reason (already packaged, already applied, description missing, fit assessment missing, company context missing, or an answer failed a generation-standard bar and could not be salvaged). Silent omission of the skipped-jobs list is not permitted; if nothing was skipped, say so explicitly.
- The reminder: press `p` on any job with `●` to open Chrome with the form prefilled.

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

Each subagent prompt embeds every item below. Subagents run in isolated contexts and cannot read the profile, the skill directory, or the database themselves.

- **The full content of every file in `profile/`** — verbatim, not summarised.
- The full `raw_description`, `fit_assessment`, `grade`, and company context for each assigned job.
- The Answer Generation Standard from this skill file, reproduced verbatim.
- The JSON key contract (`why_interested`, `why_company`, `technical_project`, `cover_letter`) reproduced verbatim, so subagent output concatenates without reformatting.
- The four mandatory-reads table reproduced verbatim, so the subagent cannot silently skip the profile / job-description / fit-assessment / company-context preconditions.
- Explicit instruction to output the SQL INSERT statements directly, not a narrative summary.

The failure mode this defends against is paraphrased-profile subagents generating answers that match the profile summary but not the actual profile — these ship with subtle factual drift (wrong project names, claimed skills the profile does not support) that only surfaces at interview.

---

## Inviolable Rules

1. **Read the profile fresh every invocation.** Never embed profile content in this file or any reference file. Profile data is a living document and snapshots go stale silently — the moment this skill caches "user has a 2:2 from York" or "user's flagship project is Nyquestro," the cached fact will eventually diverge from reality. The profile is at `profile/`; read it every time.
2. **Never fabricate experience.** Frame gaps honestly with adjacent skills rather than claim capabilities the profile does not support. Detection risk aside, this is non-negotiable because the user will interview on the basis of what this skill wrote.
3. **The four mandatory reads are preconditions, not suggestions.** If any of profile, job description, fit assessment, or company context is unavailable, skip the job with a stated reason. Do not write filler to cover the gap.
4. **`INSERT OR REPLACE` only for the package itself.** Do not modify any other table from this skill — not `jobs`, not `user_decisions`, not `companies`. Applying to a job is a separate user action triggered from the TUI.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. Items that cannot be evidenced in the agent's output are either skipped and declared in step 6's skipped-jobs list, or the skill has not finished.

- [ ] **Profile read fresh this invocation** — cite the tool call that read each file under `profile/`. Relying on earlier-session memory fails this item.
- [ ] **`cernio format` run at step 0** — the row-count summary appears in chat before step 1. If zero rows were touched, the "already clean" declaration is stated explicitly; silence fails this item.
- [ ] **Per-job quotation evidence** — for every job that received a package, the `why_interested` answer quotes at least one responsibility or stack element verbatim from the job's `raw_description`. The quotation is identifiable in the generated text.
- [ ] **Per-answer named profile element** — every answer names a specific project, skill, or experience from the profile. The project name appears in the answer (e.g. "Nyquestro"), not a generic reference ("one of my projects").
- [ ] **`why_company` cites company-specific content** — the answer quotes or paraphrases the company's `what_they_do` field and names either the team/product area from the job description or a specific product the company ships. Generic "great company" phrasings fail this item.
- [ ] **No fabricated experience** — every skill or technology claimed in the answers maps to an entry in `profile/skills.md` or a demonstrated usage in `profile/projects.md`. Gaps are framed with adjacent evidence, not filled with plausible-sounding claims.
- [ ] **Answer length bounds** — each answer is between 2 and 4 paragraphs. Count before submitting.
- [ ] **Cover letter structure verified** — the cover letter has four identifiable parts: specific-role hook, technical fit with named evidence, why-this-company, short close. Each part is visible as its own paragraph or sentence cluster.
- [ ] **JSON syntactic validity** — the JSON parses (test with a JSON-parsing tool or language runtime). Syntactically-invalid JSON breaks the autofill pipeline silently.
- [ ] **JSON key contract** — every package contains exactly the keys `why_interested`, `why_company`, `technical_project`, `cover_letter`, plus any job-specific extras from standard factual questions. Missing keys fail autofill; extra unexpected keys are ignored but noted.
- [ ] **`INSERT OR REPLACE` used** — cite the actual SQL statement executed. `INSERT` without `OR REPLACE` risks erroring on rerun.
- [ ] **Only `application_packages` modified** — no writes to `jobs`, `user_decisions`, or `companies`. Cite the set of tables written to this invocation.
- [ ] **Skipped-jobs list emitted** — step 6 report contains the explicit skipped-jobs list with per-job reasons, or an explicit "no jobs skipped" line. Absence of the list fails this item.
- [ ] **TUI reminder included** — the final report tells the user to press `p` on any job with the `●` indicator.
