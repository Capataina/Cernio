---
name: prepare-applications
description: "Generates personalised application packages for selected jobs (cover letter, 'why this role', 'why this company', technical-project answer, and other common application questions) grounded in the user's profile, the job description, and the fit assessment. Writes the answers as JSON into the application_packages SQLite table so the TUI autofill flow (press 'p') can prefill the form. Invoke when the user says 'prepare applications', 'prep these jobs', 'get applications ready', 'draft cover letters', 'batch apply', 'fill in answers for these jobs', or names a specific set of jobs they want ready to submit. Not for grading jobs (use grade-jobs), discovering companies (use discover-companies), resolving ATS slugs (use populate-db / resolve-portals), or auditing database health (use check-integrity). Use this skill whenever a user identifies jobs to apply to and wants personalised answers prepared, even if they do not name the skill explicitly."
---

# Prepare Applications

For each selected job, generate a JSON package of tailored answers to common application questions and write it to the `application_packages` SQLite table. The TUI's `p` keypress reads that package and uses Chrome CDP to prefill the application form. This skill produces the judgment half of autofill (the cover letter, the "why this company" response, the technical-project narrative, plus the durable factual answers) and hands the mechanical form-filling to the autofill pipeline.

The package is not a template. Every essay answer references specific projects, skills, and experiences from the profile by name, ties them to what the company actually does and what this role actually demands, and is honest about gaps. Every factual answer cites a specific value sourced from the live state of `profile/` files. Generic answers waste the slot; a recruiter spots them at a glance.

---

## Mandatory Reads Before Generating Any Answer

These reads are non-negotiable preconditions. Answers produced without them are discarded.

| # | What to read | Evidence that it was read |
|---|---|---|
| 1 | **Every file in `profile/`**, all files, no exceptions | You can cite the specific project from `profile/projects/` that grounds the technical-project answer, and the specific skill from `profile/skills.md` that the role asks for |
| 2 | **`profile/application-voice.md`** specifically (it is in `profile/`, but called out here because it shapes how essays are written) | You can name the alignment-density category for this role and justify the project-count choice for each essay against §1 of the voice file |
| 3 | **The job's `raw_description`** from the DB | You can quote two concrete responsibilities or stack elements from the description in the "why this role" answer |
| 4 | **The job's `fit_assessment`** from the DB | Your answer builds on the fit assessment's analysis rather than restating it; the assessment is internal reasoning, the answer is external-facing |
| 5 | **The company's `what_they_do` and `grade_reasoning`** from the DB | The "why this company" answer references what the company actually builds, not generic praise |

If any of reads 1, 3, 4, or 5 cannot be performed (DB unreachable, profile directory missing), the answer for the affected job cannot be written. State the skip explicitly and stop rather than produce a generic filler.

Profile data is not cached in this skill or in any reference file. It is read fresh from `profile/` on every invocation because the profile is a living document and yesterday's snapshot may not match today's reality.

---

## Workflow

### 0. Run `cernio format` before reading any job description

Step 1's SELECT pulls `raw_description` and `fit_assessment`, and every answer in step 3 is built directly from those strings. Raw HTML in either field (`<p>`, `<strong>`, `&amp;`, `&nbsp;`) leaks into cover letters and "why this role" answers, quoting markup instead of prose; stale HTML in `fit_assessment` degrades the "why company" and technical-project answers the same way. `cernio format` is idempotent and fast; running it at the top of every invocation guarantees the package is built from clean plaintext, not markup fragments.

```bash
cernio format
```

Paste the row-count summary into the chat response. If zero rows were touched, say so explicitly. This is required evidence that cleanliness is verified this run, not assumed.

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

The `NOT IN` clauses are load-bearing: re-preparing a job that already has a package overwrites existing work, and re-preparing applied jobs wastes effort.

### 2. Read the profile in full

Read every file in `profile/`. The full inventory rotates as the profile evolves; the files load-bearing for application generation are typically:

| File | What it grounds |
|---|---|
| `profile/projects/` (per-project files plus index) | The strongest material for `technical_project` and the technical middle of the cover letter |
| `profile/skills.md` | What the candidate can honestly claim |
| `profile/experience.md` | Work-history context, current employment status, notice-period reasoning |
| `profile/education.md` | Graduation year, degree class, institution |
| `profile/visa.md` | Visa status, sponsorship requirements, work-authorisation answers |
| `profile/preferences.toml` | Location preferences, remote stance, hard/soft filters |
| `profile/lifestyle-preferences.md` | City criteria that inform relocation phrasing |
| `profile/application-voice.md` | The texture layer that shapes every essay's length, density, tone, and hooks |
| `profile/portfolio-gaps.md` | Named gaps the essays must not overclaim against |
| `profile/cover-letter.md` | Reusable cover-letter scaffolding when present |
| `profile/personal.md`, `profile/linkedin.md`, `profile/resume.md` | Contact details and biographical anchors |

The agent picks which files weigh most for each role, and reads everything else to know what the profile does *not* contain. Read every file every time.

### 3. Generate the essay answer set per job

Write essay-style answers to this canonical question set. The four keys are the essay half of the contract with the autofill provider module in `src/autofill/`; the factual half is in step 4.

| JSON key | Question | Answer shape |
|----------|----------|--------------|
| `why_interested` | "Why are you interested in this role?" | Tie concrete responsibilities from the job description to specific profile projects or skills. Quote at least one responsibility or stack element verbatim. Name the projects. |
| `why_company` | "Why do you want to work at [Company]?" | Reference what the company actually builds (from `what_they_do`), the team or product area if the role specifies one, and the alignment with the candidate's trajectory. No "great company" filler. |
| `technical_project` | "Tell us about a technical project you've worked on." | Describe one or more projects (the right count is contextual; see §1 of `application-voice.md`). Problem, approach, specific techniques, measurable outcome. |
| `cover_letter` | Cover letter body | Specific-role hook, technical middle with named evidence, why-this-company, brief close. Project count, paragraph count, and length are contextual (see §1 and §8 of `application-voice.md`). No "Dear Hiring Manager" boilerplate. |

`application-voice.md` is the texture layer that shapes how each of these answers is written: how many projects to lead with (§1), what makes prose land (§2), narrative hooks to consider (§5), how to pick what to lead with (§6), how to frame gaps (§7), cover-letter rhythm (§8). Consult it per essay; do not treat its observations as defaults to apply mechanically.

The four essay keys are the minimum required set. If a specific job's form asks a fifth essay question the four keys do not cover (e.g. "Describe a time you handled an ambiguous technical decision"), add a new key with a descriptive name and write the answer at the same standard.

### 4. Generate the factual answer set per job

Write factual answers to the canonical question set below. Each value is sourced from the live state of the indicated `profile/` file; the phrasing patterns in §4 of `application-voice.md` describe the *shape* of strong answers, not the *values*. The values come from `profile/`.

| JSON key | Source in `profile/` | Default phrasing pattern (from voice file §4) |
|---|---|---|
| `visa_status` | `profile/visa.md` (current visa name and expiry date) | Visa name plus expiry date in one phrase |
| `sponsorship_needed` | `profile/visa.md` (current visa status and future requirement) | "Not currently; [visa name] valid through [date]. Skilled Worker sponsorship would be required from [date] onwards" or equivalent |
| `us_work_authorization` | `profile/visa.md` (binary; "No" unless the profile says otherwise) | "Yes" / "No" |
| `how_heard` | Agent default ("LinkedIn" or "Company careers page"); override at invocation if the user actually heard via a referral or event | Match the form's option list |
| `start_date` | Voice file default ("As soon as possible") unless the user has flagged a specific constraint | Phrase or calendar date depending on the form's expected shape |
| `notice_period` | `profile/experience.md` (current employment status) | "N/A (no current employment)" when independent; the actual notice when employed |
| `current_employer` | `profile/experience.md` (current role description) | "Independent" or a fuller form depending on the field's tone |
| `graduation_year` | `profile/education.md` (most recent degree) | The year |
| `gpa` | `profile/education.md` (degree class plus institution) | Class plus degree plus institution in one phrase |
| `preferred_office` | `profile/preferences.toml` (`hard.locations`) and `profile/lifestyle-preferences.md` | Single city or short list matching the role's options |
| `interviewed_before` | Agent default ("No") unless the user has explicitly tracked a prior interview at this exact company | "No" / "Yes" |
| `relocation_open` | `profile/preferences.toml` and `profile/lifestyle-preferences.md` | Direct yes/no with a brief qualifier |
| `data_protection_consent` | Always "Yes" | Constant; the form's downstream logic depends on it |
| `salary_expectation` | "35000" (number-only, no currency symbol or thousands separator); a sensible figure anchored at £35k when the role's description signals a higher band; never priced too high | Most salary fields are optional and the autofill leaves them blank by default. When the form makes the field mandatory, a concrete figure beats a blank. Greenhouse salary fields often parse as plain integers; commas, currency symbols, or words break the parse |
| `salary_unit` | "gross annual" by default | Greenhouse forms with a salary breakdown ask for a unit selection (gross annual / gross monthly / net annual / net monthly). "Gross annual" is the standard UK convention |
| `salary_currency` | "GBP" by default; the role's home currency when explicitly stated | Greenhouse currency dropdowns expect ISO codes. Caner's roles are UK-based so GBP is the default; Swiss / EU / US roles get CHF / EUR / USD respectively |
| `right_to_work` | "Yes" while UK Graduate Visa is valid (through 7 August 2027); "No" thereafter unless the role is in a country where Caner has unrestricted work authorisation | Single-select Yes/No on most Greenhouse forms. Distinct from `sponsorship_needed`: this is "can I work in this location right now", that is "do I need sponsorship to take the role" |
| `years_of_rust` | A specific number of years (e.g. "2 years") sourced from how long Caner has been actively building Rust projects in his portfolio | Greenhouse uses input_text for this question; recruiters expect a single phrase. Honest count of project work, not inflated by tutorial / read-only time |
| `links` | One string combining LinkedIn, GitHub, and portfolio URLs, prefixed with their labels for human readability | Form fields like "Please share your LinkedIn / GitHub / Portfolio" expect one combined answer; splitting them across separate package keys forces the autofill to guess which URL goes in which slot |

Fields explicitly NOT in this skill's output:

- **Free-form text the profile cannot answer.** When a form asks something genuinely novel ("What's the most interesting bug you've debugged?") that no profile file covers and that is not one of the four essay keys, omit that key. The autofill UI surfaces the unfilled fields.

Salary expectations note: `salary_expectation` is included in the package by default at £35,000. The autofill module is responsible for deciding whether to fill the form's salary field with this value (when the field is mandatory) or leave it blank (when the field is optional and the user prefers to negotiate later). When the role's description signals a posted salary band, the agent may pick a sensible figure within that band anchored at £35k as the personal floor, never priced too high (automated screens often filter out figures above a band's midpoint).

The 13 factual keys listed above are the minimum durable-fact surface. If a specific job's form asks a 14th factual question that all 13 keys collectively do not cover (e.g. a security-cleared role asks "Do you hold an active SC clearance?"), add a new key with a descriptive name and source its value from the appropriate profile file (or omit it if the profile file does not contain that information).

**When a factual value cannot be sourced from `profile/`** (the relevant file exists but lacks the information, or the information is explicitly TBD): omit that key from the package and list the field name in the per-job skipped-fields entry in step 7. The autofill leaves the form field empty; the user fills it manually. Do not invent a value.

**Per-job overrides.** Most factual values are durable across applications. A handful genuinely vary per-job (`preferred_office` for a multi-region role, `start_date` if the user has flagged a specific constraint). When the user supplies an override during the invocation ("for the Tower job, set preferred_office=Paris"), record it in the JSON package for that one job and note the override in the skipped-fields/overrides log of step 7. Per-job overrides do not modify any `profile/` file; overrides are per-package adjustments only.

### 5. Apply per-job overrides supplied at invocation

When the user names a job-specific override for a factual key during invocation, record the override in the JSON for that one job. Do not propagate the override to other jobs in the batch. Per-job overrides do not modify any `profile/` file; the profile remains the durable source, overrides are per-package adjustments only.

### 6. Write the package to the database

`application_packages.answers` is the **only** place a generated package lives. Do not write a copy to `state/`, `profile/`, or anywhere else in the repo. Re-preparing a job is an in-place update of the DB row; the same `INSERT OR REPLACE` overwrites the previous version.

```sql
INSERT OR REPLACE INTO application_packages (job_id, answers, created_at)
VALUES (?1, ?2, datetime('now'));
```

`answers` is a JSON object containing the four essay keys (step 3), the factual keys with values successfully sourced from `profile/` (step 4), any per-job overrides (step 5), and any job-specific essay-set extensions. `INSERT OR REPLACE` makes the operation idempotent: rerunning the skill on the same job updates the package rather than erroring.

**Concrete invocation pattern via the Bash tool** (the agent typically runs the skill from the project root):

```bash
# 1. Write the JSON to a /tmp path (ephemeral, deleted in step 3).
#    /tmp/ is the right home because the file's only purpose is to
#    survive the sqlite3 invocation; it is not durable state.
Write tool → /tmp/cernio-pkg-<job_id>.json

# 2. Insert into the DB, casting to TEXT.
sqlite3 state/cernio.db "INSERT OR REPLACE INTO application_packages (job_id, answers, created_at) VALUES (<job_id>, CAST(readfile('/tmp/cernio-pkg-<job_id>.json') AS TEXT), datetime('now'))"

# 3. Delete the temp file.
rm /tmp/cernio-pkg-<job_id>.json
```

> [!warning] BLOB-vs-TEXT trap with `readfile()`
> `readfile('path.json')` returns a BLOB, and SQLite's type affinity will store BLOB in a TEXT-declared column without complaint. The TUI autofill then rejects the BLOB and silently treats the package as missing. **Always wrap `readfile()` in `CAST(... AS TEXT)`** so the row is stored with TEXT affinity. Verify post-insert with `sqlite3 state/cernio.db "SELECT typeof(answers) FROM application_packages WHERE job_id = <id>"` — must return `text`, not `blob`.

> [!important] No persistent package files on disk
> Do not write the package JSON to `state/<slug>-<id>-package.json`, `profile/applications/<id>.json`, or any other persistent path. The DB row is the canonical location; on-disk copies create a drift surface ("is the file or the DB authoritative?") and accumulate in the repo. The only acceptable on-disk artefact is the ephemeral `/tmp/` file from the pattern above, which is deleted before the run finishes.

### 7. Report the outcome

Tell the user:

- **How many packages were created**, and the job IDs affected.
- **Which jobs are now ready** (they show the yellow `●` indicator in the TUI).
- **Which jobs were skipped and why**: one row per skipped job with the specific reason (already packaged, already applied, description missing, fit assessment missing, company context missing, or an answer failed a generation-standard bar and could not be salvaged). If nothing was skipped, say so explicitly.
- **Skipped factual fields per job**: for every job whose package was written, the list of factual keys omitted because the source profile file lacked the information. If every factual key was filled for every job, say so explicitly.
- **Per-job overrides applied**: the (job ID, key, override value) triples. If none, say so explicitly.
- **TUI reminder**: press `p` on any job with `●` to open Chrome with the form prefilled. Skipped fields remain blank in the autofill UI; the user fills them manually before submission.

---

## The Answer Generation Standard

Every essay answer meets these bars. An answer that fails any bar is rewritten or the skill admits it could not meet the bar rather than ship generic filler.

**Specific, not generic.**
Weak: "I'm passionate about technology and excited about this opportunity."
Strong: an answer that names a specific project from `profile/projects/`, a specific responsibility from the job description, and a specific architectural decision or measurable outcome.

**Profile-grounded.**
Every essay answer references at least one specific element from the profile by name: a project, a technology, a concrete experience. The reader should be able to tell this answer could only have been written by this specific candidate.

**Company-aware.**
The `why_company` answer cites what the company actually builds, their stack where known, the specific team or product area named in the job posting. The company's `what_they_do` field is the anchor.

**Honest.**
Do not fabricate experience. If the role asks for a technology the profile does not contain, do not claim it. Frame adjacent experience instead, using the gap-framing reasoning in §7 of `application-voice.md`.

**Voice-aligned.**
Every essay's length, density, project-count, paragraph structure, hook choice, and prose texture is decided by reading the role and profile against `application-voice.md`, not by following a fixed template. There is no fixed paragraph count; there is no fixed project count per essay; there is no fixed word count. The voice file's reasoning shapes the call per essay.

Factual answers are not subject to the essay bars. They are values sourced from `profile/` files (or per-job overrides). The standard for factual answers is "traceable to the source file" or "explicitly overridden by the user at invocation".

---

## Parallelisation

For batches of 5 or more jobs, dispatch parallel subagents (one per 2-3 jobs). The main agent is the orchestrator; subagents generate the JSON and return SQL INSERT statements, which the orchestrator collects and executes.

Each subagent prompt embeds every item below. Subagents run in isolated contexts and cannot read the profile, the skill directory, or the database themselves.

- **The full content of every file in `profile/`** verbatim, not summarised. Includes `profile/application-voice.md` so subagents have the texture-layer reasoning, and includes every factual-key source file (`visa.md`, `experience.md`, `education.md`, `preferences.toml`, `lifestyle-preferences.md`) verbatim.
- The full `raw_description`, `fit_assessment`, `grade`, and company context for each assigned job.
- The Answer Generation Standard from this skill file, reproduced verbatim.
- The four essay JSON keys (`why_interested`, `why_company`, `technical_project`, `cover_letter`) reproduced verbatim with their answer-shape descriptions.
- The 13 factual JSON keys with their source-file mapping, reproduced verbatim from step 4.
- The missing-value rule (omit the key, list in skipped-fields) reproduced verbatim, so subagents cannot silently invent values.
- The mandatory-reads table reproduced verbatim, so the subagent cannot silently skip the profile / job-description / fit-assessment / company-context preconditions.
- Any per-job overrides supplied by the user for jobs in the subagent's batch.
- Explicit instruction to output the SQL INSERT statements directly (with the full JSON inline in the values), not a narrative summary.

The failure mode this defends against is paraphrased-context subagents generating answers that match a summary but not the actual profile or voice-file content. These ship with subtle drift (wrong project names, claimed skills the profile does not support, factual answers that diverge from the durable source, prose that violates the voice file's texture guidance) that only surfaces at interview.

---

## Declare What Was Skipped

Close every run of this skill with a "What I Did Not Do" section in the step 7 report covering:

- **Jobs left without a package**: each with the specific blocker (missing description, missing fit_assessment, missing company context, generation-standard failure that could not be salvaged, already-applied / already-packaged).
- **Factual keys omitted because the source profile file lacked the information**: per job, the list of keys not in the package, the source file consulted, and the specific information the user needs to add for the next run.
- **Essay-set extensions added beyond the four canonical keys**: per job that received a new essay key, the key name and the form question that drove it.
- **Per-job overrides applied**: per job, the (key, value) pairs the user explicitly overrode for this job.

If a category genuinely has nothing to declare, state so explicitly per category. Silence on a category is not equivalent to "nothing to declare for that category". Admission of specific skipped work is preferred over a blanket "everything ran fine".

---

## Inviolable Rules

1. **Read the profile fresh every invocation.** Never embed profile content in this file or any reference file. Profile data is a living document and snapshots go stale silently. The moment this skill caches "user has a 2:2 from York" or "user's flagship project is Nyquestro," the cached fact will eventually diverge from reality.
2. **Never fabricate experience or factual answers.** Frame essay-gaps honestly with adjacent skills (using the §7 gap-framing reasoning in `application-voice.md`) rather than claim capabilities the profile does not support. Omit factual keys whose source profile file lacks the information rather than invent plausible-sounding values.
3. **The mandatory reads are preconditions, not suggestions.** If reads 1, 3, 4, or 5 are unavailable, skip the job with a stated reason. Read 2 (`application-voice.md`) is a subset of read 1 (`every file in profile/`); both are mandatory.
4. **`INSERT OR REPLACE` only for `application_packages`.** Do not modify any other table from this skill (not `jobs`, not `user_decisions`, not `companies`). Applying to a job is a separate user action triggered from the TUI.
5. **The profile is the durable factual source.** Per-job overrides supplied during invocation are recorded in the package only; they never modify any `profile/` file. The profile's authority is durable and cross-session; package-level overrides are per-job and per-invocation-only.
6. **`application-voice.md` is reasoning, not rules.** Do not strip away guidance from the voice file by quoting it as fixed prescriptions. Its observations are inputs the agent weighs per essay; the right call may sit outside any single observation in the file.

---

## Quality Checklist

Each item is an obligation with concrete evidence, not a subjective self-rating. Items that cannot be evidenced are either skipped and declared in step 7's skipped-fields list, or the skill has not finished.

- [ ] **Profile read fresh this invocation**: cite the tool call that read each file under `profile/`. Relying on earlier-session memory fails this item.
- [ ] **`profile/application-voice.md` read this invocation**: cite the tool call. The post-run report names which voice-file sections informed the most material decisions for this batch.
- [ ] **`cernio format` run at step 0**: the row-count summary appears in chat before step 1. If zero rows were touched, the "already clean" declaration is stated explicitly. Silence fails this item.
- [ ] **Per-job quotation evidence**: for every job that received a package, the `why_interested` answer quotes at least one responsibility or stack element verbatim from the job's `raw_description`. The quotation is identifiable in the generated text.
- [ ] **Per-essay-answer named profile element**: every essay answer names a specific project, skill, or experience from the profile. The project name appears in the answer (e.g. "Nyquestro" or whatever is currently in `profile/projects/`), not a generic reference ("one of my projects").
- [ ] **`why_company` cites company-specific content**: the answer quotes or paraphrases the company's `what_they_do` field and names either the team/product area from the job description or a specific product the company ships. Generic "great company" phrasings fail this item.
- [ ] **No fabricated experience**: every skill or technology claimed in the essay answers maps to an entry in `profile/skills.md` or a demonstrated usage in a per-project file in `profile/projects/`. Gaps are framed with adjacent evidence per §7 of the voice file, not filled with plausible-sounding claims.
- [ ] **Factual keys traceable to source files**: for every factual key in the package, the value matches information in the indicated `profile/` file (per the table in step 4), or is a per-job override declared in step 7. Values not traceable to either source fail this item.
- [ ] **Voice-aligned essays**: the project count, paragraph count, length, and texture of each essay was decided per the voice file's reasoning. The post-run report names the alignment-density category for this role (rich / single-bullseye / loose / perfect-fit) and the project-count choice it drove for each essay.
- [ ] **No em-dashes in essay prose**: per §2.1 of the voice file. Em-dashes are permitted inside verbatim quotations from the job description, which carry the company's own voice. Em-dashes elsewhere in essay prose fail this item.
- [ ] **No generic enthusiasm phrases**: "passionate about", "excited to", "thrilled" do not appear in essay prose per §2.2 of the voice file.
- [ ] **JSON syntactic validity**: the JSON parses (test with a JSON-parsing tool or language runtime). Syntactically-invalid JSON breaks the autofill pipeline silently.
- [ ] **JSON key contract**: every package contains the four essay keys (`why_interested`, `why_company`, `technical_project`, `cover_letter`), plus every factual key from step 4 whose source profile file contained the information, plus any per-job overrides, plus any job-specific essay-set extensions. Missing essay keys fail autofill; missing factual keys are silently tolerated by the autofill but must be surfaced in step 7.
- [ ] **`INSERT OR REPLACE` used**: cite the actual SQL statement executed. `INSERT` without `OR REPLACE` risks erroring on rerun.
- [ ] **Package column is TEXT, not BLOB**: when inserting via the `sqlite3` CLI, `readfile()` is wrapped in `CAST(... AS TEXT)`. `sqlite3 ... "SELECT typeof(answers) FROM application_packages WHERE job_id = ?"` returns `text`.
- [ ] **No persistent package JSON on disk**: the package lives only in `application_packages.answers`. The `/tmp/cernio-pkg-<job_id>.json` ephemeral file is deleted before run completion. `ls state/*.json profile/applications/*.json 2>/dev/null` returns nothing (or only files that pre-existed and are unrelated to packages).
- [ ] **Only `application_packages` modified**: no writes to `jobs`, `user_decisions`, or `companies`. Cite the set of tables written to this invocation.
- [ ] **Skipped-jobs list emitted**: step 7 report contains the explicit skipped-jobs list with per-job reasons, or an explicit "no jobs skipped" line. Absence of the list fails this item.
- [ ] **Skipped-factual-fields list emitted**: step 7 report enumerates every factual key omitted because the source profile file lacked the information, per job. If every factual key was filled for every job, that is stated explicitly.
- [ ] **Per-job overrides logged**: step 7 report lists every (job, key, value) override applied this invocation, or states "no per-job overrides this invocation" explicitly.
- [ ] **TUI reminder included**: the final report tells the user to press `p` on any job with the `●` indicator and reminds them that any skipped fields remain blank in the autofill UI.

---

## Additive-Freedom Permission for Prescribed Lists

The lists in this SKILL.md that are non-exhaustive and may be extended on a per-run basis:

- **The four essay JSON keys** (§3) are the minimum required essay surface. Per-job essay extensions named in step 3 are valid additions. The four listed keys remain mandatory; no extension may weaken, conditional-ise, or replace them.
- **The 13 factual JSON keys** (§4) are the minimum factual surface. Forms that ask a fact none of the 13 covers (e.g. security clearance) may add a new factual key sourced from the appropriate `profile/` file. The 13 listed keys remain mandatory when their source profile files contain the information.
- **The source-file mapping** (§4 table) is the minimum factual sourcing map. The user may add new fields to existing `profile/` files (or new files entirely) when their situation requires it (security clearance, professional certifications, alternate contact details); the skill reads from the live profile state.
- **The mandatory reads table** (§Mandatory Reads) is the minimum precondition set. Skills that need additional preconditions (per-job calendar availability, recruiter context from a CRM) may add reads; the five listed reads remain mandatory.
- **The Inviolable Rules** are the current structural constraints. If a future iteration surfaces a new constraint warranting Rule 7 or higher, add it; existing rules remain inviolable.
- **The Quality Checklist items** are the current required obligations. New items may be added; existing items remain mandatory.

For all lists above, additions must be **purely additive**: they may not weaken, conditional-ise, or escape-hatch any existing item. Document the addition in the per-run skipped-keys / overrides log of step 7 so future readers see the extension trail.
