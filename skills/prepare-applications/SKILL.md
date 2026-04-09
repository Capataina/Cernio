# prepare-applications

Generates personalised application packages for selected jobs so the user can apply with a single keypress. Use when asked to "prepare applications", "get my applications ready", "prep these jobs for applying", "batch apply", or when the user identifies a set of jobs they want to apply to. Not for grading jobs (that's grade-jobs), discovering companies (that's discover-companies), or checking database health (that's check-integrity).

---

## Mandatory reads — do not proceed without completing these

**STOP. Before generating any application package, you MUST read these files in full:**

1. **Every file in `profile/`** — all 15 files, no exceptions. The profile is the source material for every answer you generate. Without it, answers are generic and useless.
2. **The job's `raw_description`** — read the full description from the database. You cannot write "why this company" without knowing what the role actually involves.
3. **The job's `fit_assessment`** — this contains the grading agent's analysis of why the job fits or doesn't. Use it as a starting point, not a copy-paste source — the fit assessment is internal analysis, the application answer is external-facing.
4. **The company's `what_they_do` and `grade_reasoning`** — understand what the company does and why it was graded the way it was.

**Do not begin generating answers until all mandatory reads are complete.**

---

## What this skill produces

For each selected job, this skill generates an **application package** — a JSON object stored in the `application_packages` table that maps common application question labels to personalised answers. When the user presses `p` in the TUI, the autofill system reads this package and fills the answers into the application form automatically.

The package is stored as:

```sql
INSERT INTO application_packages (job_id, answers, created_at)
VALUES (?1, ?2, datetime('now'));
```

Where `answers` is a JSON object like:

```json
{
  "Why are you interested in this role?": "Your Prediction Markets desk is building trading systems from scratch...",
  "Tell us about a technical project": "Nyquestro is a lock-free order matching engine I built in Rust...",
  "Why do you want to work at DRW?": "DRW's approach to building proprietary technology...",
  "cover_letter": "Dear Hiring Team,\n\nI am writing to express my interest in..."
}
```

---

## The answer generation standard

Every answer must meet this standard:

### Specific, not generic
- **Bad:** "I'm passionate about technology and excited about this opportunity."
- **Good:** "Your Workers Runtime team's work on V8 isolates for edge compute maps directly to the isolation and low-latency constraints I've worked with in Nyquestro — my lock-free order matching engine processes orders in under 2 microseconds using similar principles of minimal-allocation, zero-copy design."

### Profile-grounded
Every answer must reference specific elements from the profile by name — projects, technologies, skills, experiences. The reader should understand that this answer could only have been written by this specific candidate.

### Company-aware
The "why this company" answer must reference what the company actually does, their tech stack, their engineering culture, recent work — not just "you're a great company." Use the company's `what_they_do`, `grade_reasoning`, and the job description to ground the answer.

### Honest
Do not fabricate experience. If the role asks for Kubernetes and the profile doesn't have it, don't claim it. Instead, frame adjacent experience: "While I haven't deployed to Kubernetes in production, my work with Docker containers in local development and my understanding of orchestration concepts from building distributed systems in Nyquestro give me a strong foundation to learn quickly."

### Concise
Application answers should be 2-4 paragraphs. Recruiters read hundreds of these. Dense, specific, and short beats long and generic every time.

---

## Workflow

### 1. Identify target jobs

The user specifies which jobs to prepare. Common patterns:
- "Prepare applications for all SS and S jobs"
- "Prep these 5 jobs: [list]"
- "Get the top 10 ready"

Query the jobs from the database:

```sql
SELECT j.id, j.title, j.url, j.raw_description, j.fit_assessment, j.grade,
       c.name as company, c.what_they_do, c.grade_reasoning,
       cp.ats_provider
FROM jobs j
JOIN companies c ON j.company_id = c.id
LEFT JOIN company_portals cp ON cp.company_id = c.id AND cp.is_primary = 1
WHERE j.grade IN ('SS', 'S')  -- or whatever the user specified
  AND j.evaluation_status <> 'archived'
  AND j.id NOT IN (SELECT job_id FROM application_packages)
  AND j.id NOT IN (SELECT job_id FROM user_decisions WHERE decision = 'applied')
ORDER BY
    CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 ELSE 4 END;
```

### 2. Read the full profile

Read **all files in `profile/`**. Every file, every time. Pay special attention to:
- `projects.md` — the flagship projects are your strongest material
- `skills.md` — what you can honestly claim
- `experience.md` — work history context
- `portfolio-gaps.md` — know what gaps exist so you don't overclaim

### 3. Generate answers for each job

For each job, generate answers to the common Greenhouse application questions:

**Standard questions to prepare answers for:**

| Key | Question | Notes |
|-----|----------|-------|
| `why_interested` | "Why are you interested in this role?" | Connect the job's specific responsibilities to profile projects |
| `why_company` | "Why do you want to work at [Company]?" | Use company's what_they_do, engineering culture, specific products |
| `technical_project` | "Tell us about a technical project you've worked on" | Pick the most relevant flagship project for this specific role |
| `cover_letter` | Cover letter text | 3-4 paragraphs, opening + technical fit + why this company + closing |

**Additional questions to prepare if they appear frequently at this ATS provider:**
- "What are your salary expectations?"
- "Are you authorised to work in the UK?"
- "When can you start?"
- "How did you hear about this role?"

For standard factual questions, use profile data directly:
- Salary: leave empty (user decides)
- Work authorisation: "Yes — I hold a UK Graduate visa valid through August 2027, with unrestricted right to work."
- Start date: "Immediately" or "Available from [date]" per user preference
- How heard: "Found through company careers page"

### 4. Write packages to the database

For each job, insert the answers:

```sql
INSERT OR REPLACE INTO application_packages (job_id, answers, created_at)
VALUES (?1, ?2, datetime('now'));
```

The `answers` field is a JSON object. Keys should be the question text (for matching against form labels) and values should be the generated answers.

### 5. Report what was prepared

After generating all packages, report:
- How many packages were created
- Which jobs are now ready (with the `●` indicator in TUI)
- Any jobs that were skipped (already applied, already has package, no description available)
- Remind the user: press `p` on any job with the `●` indicator to open Chrome with the form pre-filled

---

## Parallelisation

For batches of 5+ jobs, consider parallelising across subagents. Each agent handles 2-3 jobs and needs:
- The full profile (all 15 files, embedded in the prompt)
- The job descriptions, fit assessments, and company context for its assigned jobs
- The answer generation standard from this skill file
- Explicit instructions to output SQL INSERT statements

The orchestrator collects the SQL from each agent and executes it.

---

## Quality checklist

Before inserting any package:
- [ ] Every answer references at least one specific project from the profile by name
- [ ] The "why company" answer mentions something specific about this company, not generic praise
- [ ] No fabricated experience — gaps are framed honestly with adjacent skills
- [ ] Answers are 2-4 paragraphs, not walls of text
- [ ] The cover letter has a clear structure: hook, technical fit, company fit, closing
- [ ] The JSON is valid and keys match common Greenhouse form labels
- [ ] The profile was read in full before generating any answer
