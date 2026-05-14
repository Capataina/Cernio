---
name: prepare-applications
description: "Generates personalised application packages for selected jobs — cover letter, 'why this role', 'why this company', technical-project answer, and other common application questions — grounded in the user's profile, the job description, and the fit assessment. Writes the answers as JSON into the application_packages SQLite table so the TUI autofill flow (press 'p') can prefill the form. Invoke when the user says 'prepare applications', 'prep these jobs', 'get applications ready', 'draft cover letters', 'batch apply', 'fill in answers for these jobs', or names a specific set of jobs they want ready to submit. Not for grading jobs (use grade-jobs), discovering companies (use discover-companies), resolving ATS slugs (use populate-db / resolve-portals), or auditing database health (use check-integrity). Use this skill whenever a user identifies jobs to apply to and wants personalised answers prepared, even if they do not name the skill explicitly."
---

# Prepare Applications

For each selected job, generate a JSON package of tailored answers to common application questions and write it to the `application_packages` SQLite table. The TUI's `p` keypress reads that package and uses Chrome CDP to prefill the application form. This skill produces the judgment half of autofill — the cover letter, the "why this company" response, the technical-project narrative, plus the durable factual answers (visa, sponsorship, current employer, etc.) — and hands the mechanical form-filling to the autofill pipeline.

The package is not a template. Every essay answer references specific projects, skills, and experiences from the profile by name, ties them to what the company actually does and what this role actually demands, and is honest about gaps. Every factual answer cites a specific value from `profile/applications.md` rather than a guess. Generic answers waste the slot; a recruiter can spot them at a glance.

---

## Mandatory Reads Before Generating Any Answer

These five reads are non-negotiable preconditions. Answers produced without them are discarded.

| # | What to read | Evidence that it was read |
|---|---|---|
| 1 | **Every file in `profile/`** — all files, no exceptions | You can cite the specific project from `profile/projects/` that grounds the technical-project answer, and the specific skill from `profile/skills.md` that the role asks for |
| 2 | **The job's `raw_description`** from the DB | You can quote two concrete responsibilities or stack elements from the description in the "why this role" answer |
| 3 | **The job's `fit_assessment`** from the DB | Your answer builds on the fit assessment's analysis rather than restating it; the assessment is internal reasoning, the answer is external-facing |
| 4 | **The company's `what_they_do` and `grade_reasoning`** from the DB | The "why this company" answer references what the company actually builds, not generic praise |
| 5 | **`profile/applications.md`** — durable factual answers (visa, sponsorship, current employer, graduation year, GPA, preferred office, etc.) | The factual-answer keys in step 4 cite values directly from this file. If the file is absent on first run, step 4a bootstraps a skeleton at that path; if specific fields in the file are blank or marked `TODO`, the corresponding keys are omitted from the package and listed in the skipped-fields report |

If any of reads 1–4 cannot be performed (DB unreachable, profile directory missing), the answer for the affected job cannot be written — state the skip explicitly and stop rather than produce a generic filler. Read 5 has its own bootstrap (step 4a); a missing or partial `applications.md` does not block essay generation, only factual-key generation.

Profile data is not cached in this skill or in any reference file; it is read fresh from `profile/` on every invocation because the profile is a living document and yesterday's snapshot may not match today's reality.

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

- `projects/` — substantive active or complete projects are the strongest material for the technical-project answer; see `projects/index.md` for the inventory and read the relevant per-project files
- `skills.md` — what the candidate can honestly claim
- `experience.md` — work-history context for seniority framing
- `applications.md` — durable factual answers consumed by step 4
- `portfolio-gaps.md` — named gaps; the answers must not overclaim against these

### 3. Generate the essay answer set per job

Write essay-style answers to this canonical question set. These four keys are the essay half of the contract with the autofill provider module in `src/autofill/`; the factual half is in step 4.

| JSON key | Question | Answer shape |
|----------|----------|--------------|
| `why_interested` | "Why are you interested in this role?" | Tie two or three concrete responsibilities from the job description to specific profile projects / skills. Name the projects. |
| `why_company` | "Why do you want to work at [Company]?" | Reference what the company actually builds (from `what_they_do`), the team / product area if the role specifies one, and the alignment with the candidate's trajectory. No "great company" filler. |
| `technical_project` | "Tell us about a technical project you've worked on." | Pick the single most relevant project for this role. One flagship, described in depth — problem, approach, specific techniques, measurable outcome. Not a list of all the projects. |
| `cover_letter` | Cover letter body | 3–4 paragraphs: hook referencing the specific role → technical fit with named evidence → why this company specifically → short close. No "Dear Hiring Manager" boilerplate. |

The four essay keys are the minimum required set. If a specific job's form asks a fifth essay question the four keys do not cover (e.g. "Describe a time you handled an ambiguous technical decision"), add a new key with a descriptive name and write the answer at the same standard. The four listed keys remain mandatory; additional keys are pure additions per the Additive-Freedom Permission section at the end of this file.

### 4. Generate the factual answer set per job

Write factual answers to this canonical question set. Each value comes from `profile/applications.md` (the durable, per-user source) unless the per-job context requires an override. These keys are the factual half of the autofill contract — the autofill's `match_semantic_key` in `src/autofill/greenhouse.rs` maps Greenhouse form labels onto them.

| JSON key | Question shape | Source / value |
|---|---|---|
| `visa_status` | "What is your right to work in the UK?" / "Please confirm your Right to Work status" | `applications.md` → `**Visa status:** ...` — the exact phrase as it appears on form options (e.g. "Skilled Worker Visa", "UK / Irish National") |
| `sponsorship_needed` | "Do you require visa sponsorship now or in the future?" — frequently asked TWICE on the same form with slight phrasing variations; both get the same answer | `applications.md` → `**Sponsorship needed:** Yes / No` |
| `us_work_authorization` | "Are you legally authorised to work in the United States?" — appears on US-HQ company forms even for UK-located roles | `applications.md` → `**US work authorisation:** Yes / No` |
| `how_heard` | "How did you hear about this job?" — required on most forms; some have 16+ options | `applications.md` → `**How heard:** LinkedIn` (or a stable user-chosen value the autofill label-matches against the options list) |
| `start_date` | "When can you start?" / "Earliest start date" | `applications.md` → `**Earliest start date:** Immediately` (or a specific date / phrase the user has set) |
| `notice_period` | "What is your current notice period?" | `applications.md` → `**Notice period:** 1 month` (or the user's actual value); when not currently employed, this is `N/A` |
| `current_employer` | "Who is your current employer?" | `applications.md` → `**Current employer:** ...` |
| `graduation_year` | "What year did you graduate?" / "Year you expect to begin full time employment" | `applications.md` → `**Graduation year:** 2025` |
| `gpa` | "What is your current cumulative GPA?" — varies between numeric ("3.8") and class-based ("First-class honours") | `applications.md` → `**Degree class / GPA:** First-class honours` |
| `preferred_office` | "Preferred office location" — frequently multi-select with N options | `applications.md` → `**Preferred office:** London` |
| `interviewed_before` | "Have you interviewed at [Company] before?" | `applications.md` → `**Interviewed at prior firms:** No` (the autofill semantic-key matcher inverts when the form names a specific company) |
| `relocation_open` | "Are you open to relocation for this role?" | `applications.md` → `**Open to relocation:** No` |
| `data_protection_consent` | Privacy-policy / GDPR / data-protection acknowledgement checkbox — present on every form | Always `Yes` (the form's options are typically `Acknowledge / Confirm`; the autofill matches by label) |

Fields explicitly NOT in this skill's output:

- **Salary expectations.** Live decision the user owns per-job; never autofilled. The autofill leaves these blank for the user to fill manually before submission.
- **Free-form text the profile cannot answer.** When a form asks something genuinely novel ("What's the most interesting bug you've debugged?") that is not in `applications.md` and is not one of the four essay keys, the package omits that key. The autofill UI surfaces the unfilled fields.

The 13 factual keys listed above are the minimum durable-fact surface. If a specific job's form asks a 14th factual question that all 13 keys collectively do not cover (e.g. a security-cleared role asks "Do you hold an active SC clearance?"), add a new key with a descriptive name and source its value from a new `applications.md` field. The 13 listed keys remain the minimum; additional keys are pure additions per the Additive-Freedom Permission section.

**When a value is missing from `applications.md` (the field exists in the schema but is blank or `TODO`):** omit that key from the package and list the field name in the per-job skipped-fields entry in step 7. The autofill will leave the corresponding form field empty, and the user fills it manually. Do not invent a value to fill the gap.

**Per-job overrides.** Most factual values are durable across applications. A handful genuinely vary per-job — `preferred_office` for a multi-region role, `start_date` if the user has flagged a constraint for a specific firm. When the user supplies an override during the invocation ("for the Tower job, set preferred_office=Paris"), record it in the JSON package with the override value and note the override in the skipped-fields/overrides log of step 7. Without an explicit override, the durable value from `applications.md` stands.

### 4a. Bootstrap `profile/applications.md` if missing

`profile/applications.md` is the source of truth for the factual-answer keys in step 4. If the file does not exist when step 4 needs to read it, the skill creates a skeleton at `profile/applications.md` with the schema below, every value initially marked `TODO`, and continues the run. The factual keys whose values are still `TODO` after bootstrap are omitted from the packages (per step 4's missing-value rule) and surfaced in the skipped-fields report.

Skeleton content the skill writes on bootstrap:

```markdown
# Application Factual Answers

Durable per-user factual answers consumed by the `prepare-applications` skill (step 4) and the Greenhouse autofill (`src/autofill/greenhouse.rs`). Edit every `TODO` to a real value before the next prepare-applications run; values marked `TODO` are omitted from generated packages.

**Visa status:** TODO  (exact phrase used on UK Greenhouse forms — e.g. "Skilled Worker Visa", "UK / Irish National", "EU / EEA or Swiss National with Settled or Pre-Settled Status", "Spousal Visa", "I do not have the Right to Work in the UK and require sponsorship")
**Sponsorship needed:** TODO  (Yes / No — applies to both "do you require sponsorship now" and "will you require sponsorship in the future" — answer once, autofill replies twice when the form asks twice)
**US work authorisation:** TODO  (Yes / No — answered on US-HQ company forms even for UK-located roles)
**How heard:** TODO  (e.g. "LinkedIn", "Company careers page", "Referral" — pick the value that matches options on most forms)
**Earliest start date:** TODO  (e.g. "Immediately", or a specific date in YYYY-MM-DD)
**Notice period:** TODO  (e.g. "1 month", "2 weeks", "N/A")
**Current employer:** TODO
**Graduation year:** TODO  (e.g. "2025"; for postgraduates, the most recent degree year)
**Degree class / GPA:** TODO  (e.g. "First-class honours", "2:1", "3.8/4.0")
**Preferred office:** TODO  (e.g. "London"; if a job asks multi-region, supply per-job override during invocation)
**Interviewed at prior firms:** TODO  (Yes / No — used to answer "Have you interviewed at [Company] before?"; default is No unless the user has interviewed at this exact company)
**Open to relocation:** TODO  (Yes / No)
```

Bootstrap is a single write; the run continues without pausing for user input. Subsequent runs read whatever values the user has filled in.

The skeleton's keys are the minimum durable-fact surface. The user may add fields beyond the listed set when their situation requires it (e.g. a Security Clearance field for cleared roles); the skill reads the file as a free-form key-value source and any `**Key:** value` line is available to per-job answer generation.

### 5. Apply per-job overrides supplied at invocation

When the user names a job-specific override for a factual key during invocation ("for Tower Research, preferred_office=Paris"), record the override in the JSON for that one job. Do not propagate the override to other jobs in the batch. Per-job overrides do not modify `profile/applications.md` — that file remains the durable source; overrides are per-package adjustments only.

### 6. Write the package to the database

```sql
INSERT OR REPLACE INTO application_packages (job_id, answers, created_at)
VALUES (?1, ?2, datetime('now'));
```

`answers` is a JSON object containing the four essay keys (step 3), the factual keys with non-TODO values (step 4), any per-job overrides (step 5), and any job-specific extras (the new keys allowed under the additive-freedom permission). `INSERT OR REPLACE` makes the operation idempotent — rerunning the skill on the same job updates the package rather than erroring.

### 7. Report the outcome

Tell the user:

- How many packages were created, and the job IDs affected.
- Which jobs are now ready (they show the yellow `●` indicator in the TUI).
- Which jobs were skipped and why — one row per skipped job with the specific reason (already packaged, already applied, description missing, fit assessment missing, company context missing, or an answer failed a generation-standard bar and could not be salvaged). Silent omission of the skipped-jobs list is not permitted; if nothing was skipped, say so explicitly.
- **Skipped factual fields per job** — for every job whose package was written, the list of factual keys omitted because their `applications.md` value was `TODO` or absent. One row per omitted key per job; if every factual key was filled, say so explicitly.
- **Per-job overrides applied** — if step 5 applied any per-job overrides, list each (job ID + key + override value). If none, say so explicitly.
- **Applications.md bootstrap** — if step 4a created the skeleton this run, say so explicitly so the user knows the file is now ready to edit.
- The reminder: press `p` on any job with `●` to open Chrome with the form prefilled. Note that any TODO fields remain blank in the autofill UI — the user fills them manually before submission.

---

## The Answer Generation Standard

Every generated essay answer meets all five bars below. An answer that fails any one is rewritten or the skill admits it could not meet the bar rather than ship generic filler.

**Specific, not generic.**
Weak: "I'm passionate about technology and excited about this opportunity."
Strong: "Your Workers Runtime team's work on V8 isolates for edge compute maps directly to the isolation and low-latency constraints I've worked with in Nyquestro — my lock-free order matching engine processes orders in under 2 microseconds using similar principles of minimal-allocation, zero-copy design."

**Profile-grounded.**
Every essay answer references at least one specific element from the profile by name — a project, a technology, a concrete experience. The reader should be able to tell this answer could only have been written by this specific candidate.

**Company-aware.**
The `why_company` answer cites what the company actually builds, their stack where known, the specific team or product area named in the job posting. The company's `what_they_do` field is the anchor.

**Honest.**
Do not fabricate experience. If the role asks for Kubernetes and the profile does not have it, do not claim it. Frame adjacent experience instead: "I have not deployed to Kubernetes in production, but my Docker work in local development plus the orchestration concepts from Nyquestro's distributed components give me a strong foundation to learn it quickly."

**Concise.**
2–4 paragraphs per essay answer. Recruiters read hundreds of these. Dense and specific beats long and generic every time.

Factual answers are not subject to the five bars — they are exact values from `profile/applications.md`. The standard for factual answers is "matches the source file verbatim" (modulo the per-job override mechanism in step 5).

---

## Parallelisation

For batches of 5 or more jobs, dispatch parallel subagents — one per 2–3 jobs. The main agent is the orchestrator; subagents generate the JSON and return SQL INSERT statements, which the orchestrator collects and executes.

Each subagent prompt embeds every item below. Subagents run in isolated contexts and cannot read the profile, the skill directory, or the database themselves.

- **The full content of every file in `profile/`** — verbatim, not summarised. Includes `applications.md` so subagents have direct access to factual-key source values.
- The full `raw_description`, `fit_assessment`, `grade`, and company context for each assigned job.
- The Answer Generation Standard from this skill file, reproduced verbatim.
- The four essay JSON keys (`why_interested`, `why_company`, `technical_project`, `cover_letter`) reproduced verbatim with their answer-shape descriptions.
- The 13 factual JSON keys with their question-shape and source-from-applications.md mapping, reproduced verbatim from step 4.
- The missing-value rule (omit the key, list in skipped-fields) reproduced verbatim, so subagents cannot silently invent values.
- The mandatory-reads table reproduced verbatim, so the subagent cannot silently skip the profile / job-description / fit-assessment / company-context preconditions.
- Any per-job overrides supplied by the user for jobs in the subagent's batch.
- Explicit instruction to output the SQL INSERT statements directly (with the full JSON inline in the values), not a narrative summary.

The failure mode this defends against is paraphrased-context subagents generating answers that match a summary but not the actual profile or applications source — these ship with subtle factual drift (wrong project names, claimed skills the profile does not support, factual answers that diverge from the durable source) that only surfaces at interview.

---

## Declare What Was Skipped

Close every run of this skill with a "What I Did Not Do" section in the step 7 report covering:

- **Jobs left without a package** — each with the specific blocker (missing description, missing fit_assessment, missing company context, generation-standard failure that could not be salvaged, already-applied / already-packaged).
- **Factual keys omitted because `applications.md` had `TODO` or missing values** — per job, the list of keys not in the package and the specific field name the user needs to fill in.
- **Essay-set extensions added beyond the four canonical keys** — per job that received a new essay key, the key name and the form question that drove it.
- **Per-job overrides applied** — per job, the (key, value) pairs the user explicitly overrode for this job.
- **Bootstrap actions** — whether `profile/applications.md` was created this run (and where), or whether it was already present.

If a category genuinely has nothing to declare, state so explicitly per category — silence on a category is not equivalent to "nothing to declare for that category". Per skill-creator's obligation-vs-exhortation discipline, admission of specific skipped work is preferred over a blanket "everything ran fine".

---

## Inviolable Rules

1. **Read the profile fresh every invocation.** Never embed profile content in this file or any reference file. Profile data is a living document and snapshots go stale silently — the moment this skill caches "user has a 2:2 from York" or "user's flagship project is Nyquestro," the cached fact will eventually diverge from reality. The profile is at `profile/`; read it every time.
2. **Never fabricate experience or factual answers.** Frame essay-gaps honestly with adjacent skills rather than claim capabilities the profile does not support. Omit factual keys whose source values are `TODO` or absent rather than invent plausible-sounding values. The user will interview on the basis of what this skill wrote.
3. **The five mandatory reads are preconditions, not suggestions.** If reads 1–4 are unavailable, skip the job with a stated reason. Read 5 has its own bootstrap (step 4a); a missing or partial `applications.md` does not block essay generation, only factual-key generation.
4. **`INSERT OR REPLACE` only for `application_packages`.** Do not modify any other table from this skill — not `jobs`, not `user_decisions`, not `companies`. Applying to a job is a separate user action triggered from the TUI.
5. **`profile/applications.md` is the durable factual-answer source.** Per-job overrides supplied during invocation are recorded in the package only — they never modify `applications.md`. The file's authority is per-user and per-invocation-stable; package-level overrides are per-job and per-invocation-only.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. Items that cannot be evidenced in the agent's output are either skipped and declared in step 7's skipped-fields list, or the skill has not finished.

- [ ] **Profile read fresh this invocation** — cite the tool call that read each file under `profile/`. Relying on earlier-session memory fails this item.
- [ ] **`cernio format` run at step 0** — the row-count summary appears in chat before step 1. If zero rows were touched, the "already clean" declaration is stated explicitly; silence fails this item.
- [ ] **`profile/applications.md` read this invocation** — cite the tool call that read the file. If the file did not exist and step 4a bootstrapped it, cite the Write tool call that created it.
- [ ] **Per-job quotation evidence** — for every job that received a package, the `why_interested` answer quotes at least one responsibility or stack element verbatim from the job's `raw_description`. The quotation is identifiable in the generated text.
- [ ] **Per-essay-answer named profile element** — every essay answer names a specific project, skill, or experience from the profile. The project name appears in the answer (e.g. "Nyquestro"), not a generic reference ("one of my projects").
- [ ] **`why_company` cites company-specific content** — the answer quotes or paraphrases the company's `what_they_do` field and names either the team/product area from the job description or a specific product the company ships. Generic "great company" phrasings fail this item.
- [ ] **No fabricated experience** — every skill or technology claimed in the essay answers maps to an entry in `profile/skills.md` or a demonstrated usage in a per-project file in `profile/projects/`. Gaps are framed with adjacent evidence, not filled with plausible-sounding claims.
- [ ] **Factual keys traceable to `applications.md`** — for every factual key in the package, the value in the JSON matches a `**Key:** value` line in `profile/applications.md` (or is a per-job override declared in step 7). Values not traceable to either source fail this item.
- [ ] **TODO-valued fields omitted, not invented** — every factual key whose source value was `TODO` or absent is omitted from the JSON package and appears in step 7's skipped-fields list for that job. Inventing a value to fill a `TODO` slot fails this item.
- [ ] **Essay answer length bounds** — each essay answer is between 2 and 4 paragraphs. Count before submitting.
- [ ] **Cover letter structure verified** — the cover letter has four identifiable parts: specific-role hook, technical fit with named evidence, why-this-company, short close. Each part is visible as its own paragraph or sentence cluster.
- [ ] **JSON syntactic validity** — the JSON parses (test with a JSON-parsing tool or language runtime). Syntactically-invalid JSON breaks the autofill pipeline silently.
- [ ] **JSON key contract** — every package contains the four essay keys (`why_interested`, `why_company`, `technical_project`, `cover_letter`), plus every factual key from step 4 whose `applications.md` value was non-`TODO`, plus any per-job overrides, plus any job-specific essay-set extensions named in the skipped-keys log. Missing essay keys fail autofill; missing factual keys are silently tolerated by the autofill but must be surfaced in step 7.
- [ ] **`INSERT OR REPLACE` used** — cite the actual SQL statement executed. `INSERT` without `OR REPLACE` risks erroring on rerun.
- [ ] **Only `application_packages` modified** — no writes to `jobs`, `user_decisions`, or `companies`. Cite the set of tables written to this invocation. (Step 4a's bootstrap write to `profile/applications.md` is a filesystem write, not a DB write, and is expected.)
- [ ] **Skipped-jobs list emitted** — step 7 report contains the explicit skipped-jobs list with per-job reasons, or an explicit "no jobs skipped" line. Absence of the list fails this item.
- [ ] **Skipped-factual-fields list emitted** — step 7 report enumerates every factual key omitted because `applications.md` had `TODO` or missing values, per job. If every factual key was filled for every job, that is stated explicitly.
- [ ] **Per-job overrides logged** — step 7 report lists every (job, key, value) override applied this invocation, or states "no per-job overrides this invocation" explicitly.
- [ ] **TUI reminder included** — the final report tells the user to press `p` on any job with the `●` indicator and reminds them that TODO fields remain blank in the autofill UI.

---

## Additive-Freedom Permission for Prescribed Lists

Per Inviolable Rule 9 of the skill-creator that authored / iterated this skill (every prescribed list must include explicit additive-freedom permission). The lists in this SKILL.md that are non-exhaustive and may be extended on a per-run basis:

- **The four essay JSON keys** (§3) are the minimum required essay surface. Per-job essay extensions named in step 3 are valid additions. The four listed keys remain mandatory; no extension may weaken, conditional-ise, or replace them.
- **The 13 factual JSON keys** (§4) are the minimum factual surface. Forms that ask a fact none of the 13 covers (e.g. security clearance) may add a new factual key sourced from a new field in `applications.md`. The 13 listed keys remain mandatory when their source values are non-`TODO`.
- **The `applications.md` schema fields** (§4a skeleton) are the minimum durable-fact fields. The user may add fields for their situation (security clearance, professional certifications, alternate contact details); the skill reads any `**Key:** value` line. The listed fields are the baseline.
- **The mandatory reads table** (§Mandatory Reads) is the minimum precondition set. Skills that need additional preconditions (per-job calendar availability, recruiter context from a CRM) may add reads; the five listed reads remain mandatory.
- **The five Inviolable Rules** are the current structural constraints. If a future iteration surfaces a new constraint warranting Rule 6 or higher, add it; existing rules remain inviolable.
- **The Quality Checklist items** are the current required obligations. New items may be added; existing items remain mandatory.

For all six lists above, additions must be **purely additive** — they may not weaken, conditional-ise, or escape-hatch any existing item. Document the addition in the per-run skipped-keys / overrides log of step 7 so future readers see the extension trail.
