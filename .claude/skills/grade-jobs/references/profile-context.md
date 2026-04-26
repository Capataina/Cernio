# Profile Context for Job Evaluation

**This file does not contain the candidate's profile. It tells you how to read the profile and what to extract for job grading.**

The candidate's profile is maintained across files in `profile/`. You must read ALL of them before grading any job. Every file. Do not skip any — missing context leads to grading errors.

---

## Table of Contents

1. [Mandatory: Read every file in profile/](#mandatory-read-every-file-in-profile)
2. [What to build from the profile files](#what-to-build-from-the-profile-files)
3. [Tech stack evaluation framework](#tech-stack-evaluation-framework)
4. [What makes a job genuinely exciting vs. merely acceptable](#what-makes-a-job-genuinely-exciting-vs-merely-acceptable)
5. [Profile elements to reference in fit assessments](#profile-elements-to-reference-in-fit-assessments)

---

## Mandatory: Read every file in profile/

| File | What to extract for job grading |
|------|--------------------------------|
| `profile/projects/` (per-project files + `index.md`) | **Critical.** The project portfolio is the primary evidence of engineering capability. Read `profile/projects/index.md` for the inventory, then read every per-project file. For each project, extract: what it demonstrates, what technologies it uses, what domain knowledge it proves, and its **status** from the file's frontmatter (active/complete/paused/dormant/abandoned). Status determines evidence weight — active and complete projects are primary evidence, paused/dormant projects are secondary, abandoned projects are background context only. This is what you match against job requirements. |
| `profile/projects/open-source-contributions.md` | OSS contributions and community involvement record. Strengthens narrative for mission-driven roles and demonstrates engagement with the broader ecosystem. |
| `profile/skills.md` | **Critical.** Technical skills inventory across six tables (Programming Languages, Frameworks, Libraries, Engines and Runtimes, Tools and Platforms, Concepts and Domains) with four bands (Proficient/Comfortable/Familiar/Beginner). Use this to assess stack alignment — does the candidate know the languages, frameworks, and tools the job requires? The band is the proficiency signal. |
| `profile/experience.md` | Formal work experience. Determines seniority assessment — how many years of professional experience exist, and what that means for roles requiring specific tenure. |
| `profile/education.md` | Degree details. Some roles filter on degree classification or institution. Know the candidate's credentials to assess whether HR screening filters will pass or reject. |
| `profile/visa.md` | **Critical.** Visa type, expiry date, right-to-work status. Determines sponsorship assessment for every job. Know the exact timeline. |
| `profile/preferences.toml` | **Critical.** Hard constraints (excluded sectors, excluded role types, location requirements) and soft preferences. Any job that violates a hard constraint is an automatic F. |
| `profile/portfolio-gaps.md` | Known skill gaps. If a job requires a skill listed here as a gap, note it — but also check whether the gap has been addressed since the file was last updated. Update this file with new patterns discovered during grading. |
| `profile/personal.md` | Location, nationality, personal constraints. Affects commute assessment, relocation feasibility, and security clearance eligibility. |
| `profile/resume.md` | The structured CV. Understand what the candidate presents on paper — this is what hiring managers and recruiters see first. |
| `profile/cover-letter.md` | Application narrative. Understand how the candidate positions their strengths — useful for assessing whether a compelling application narrative exists for a given role. |
| `profile/interests.md` | Domain interests. Jobs in domains the candidate finds genuinely motivating score higher on the "exciting vs. acceptable" spectrum. |
| `profile/certifications.md` | Certifications held. Some roles require or prefer specific credentials. |
| `profile/languages.md` | Spoken languages. Relevant for roles with language requirements or international teams. |
| `profile/military.md` | Military status. Relevant for security clearance eligibility assessment. |

---

## What to build from the profile files

### Technical identity

Synthesise from the per-project files in `profile/projects/`, `skills.md`, and `experience.md`:

- **Primary technical strengths** — The candidate's core competencies, ordered by depth. What are they strongest at? What evidence supports each strength? Build a table using **active or complete projects as primary evidence**:

  | Strength | Evidence (from projects/experience) | Project status | What it means for job fit |
  |----------|-------------------------------------|----------------|---------------------------|
  | (filled from profile) | (specific projects/roles) | active/complete/paused/dormant/abandoned | (which job types this maps to) |

- **Secondary strengths** — Skills the candidate has but that are not the primary focus. Comfortable, not expert. Paused or dormant projects often demonstrate these.

- **Known weaknesses / portfolio gaps** — Read `portfolio-gaps.md` and cross-reference with the per-project files in `profile/projects/` and `skills.md`. If a role requires one of these as a hard prerequisite, the candidate cannot credibly claim the skill. If it is a "nice to have," note the gap but do not force a grade reduction.

**Status weighting determines evidence depth.** Each per-project file has a `status` frontmatter field (active, complete, paused, dormant, abandoned). Active and complete projects are the candidate's strongest evidence of capability — primary citations. Paused and dormant projects are secondary evidence — solid work but not currently invested in. Abandoned projects show breadth of interest but are background context only, not primary evidence. When a role requirement matches only an abandoned project, the candidate's evidence for that requirement is weak.

### Career targets and what they mean for grading

Synthesise from `preferences.toml`, `visa.md`, `experience.md`, and `interests.md`:

- **Long-term target** — What is the candidate's career trajectory aim? What compensation level? What role type? Grade career ceiling by asking "does this role's domain produce engineers at that level at 10-15 years?" rather than "is this a nice first job?"

- **Immediate goal** — What is the candidate looking for right now? Read `visa.md` for timeline context — the visa situation determines urgency and how to weight immediate achievability vs. long-term trajectory.

### Seniority constraints

Synthesise from `experience.md`, `education.md`, and the per-project files in `profile/projects/`:

- **What the profile supports** — Based on formal experience (years, roles) and project portfolio depth, what seniority levels are achievable? What is a stretch? What is not achievable?

- **How to assess seniority from descriptions** — Ignore the title. Read the description for:
  1. Years stated — hard requirement or preference?
  2. Scope of responsibility — "own a component" vs. "own the architecture of the platform"
  3. Expectations of others' work — "mentor junior engineers," "lead design reviews" presupposes experience
  4. Production expectations — "incident management experience," "on-call leadership" presupposes operational maturity

### Visa and sponsorship assessment

Read `visa.md` to build a sponsorship assessment framework:

- What is the current visa status and expiry date?
- What happens after expiry — what type of sponsorship is needed?
- What is the window during which sponsorship is not needed?
- How does this timeline affect the value of companies that can vs. cannot sponsor?

Apply this framework to every job:
- Large companies with established programmes — assume sponsorship is viable unless negative signals exist
- Mid-size companies — check sponsor register, note uncertainty if unclear
- Small startups — sponsorship is uncertain; note the risk but do not force an F if the experience value within the visa window justifies the role
- Short-term contracts — sponsorship matters less; the experience is the value

---

## Tech stack evaluation framework

Build this from `skills.md` and the per-project files in `profile/projects/`:

### Strongly aligned stacks
Technologies the candidate has deep, demonstrated experience with. Application would be strong.

### Well-aligned stacks
Technologies the candidate does not use directly but can make a compelling case for (similar paradigm, transferable skills, quick to learn given existing expertise).

### Weakly aligned but not dealbreakers
Technologies with no direct experience but where language-learning or tool-learning is not a fundamental barrier.

### Dealbreakers only if the stack IS the role
Technologies or platforms that represent a fundamental mismatch with the candidate's direction. Only a problem if the entire role revolves around them.

---

## What makes a job genuinely exciting vs. merely acceptable

### Exciting (pushes toward S/SS)

Synthesise from the per-project files in `profile/projects/`, `interests.md`, and `preferences.toml`:
- The role involves building something from scratch or substantially from first principles
- The domain has a direct connection to an existing project in the portfolio
- The technical challenge is real — not theoretical complexity but actual hard problems at scale
- The team includes engineers whose work you can find and respect
- The candidate's primary language is in the production stack
- The role involves building infrastructure that other engineers depend on

### Acceptable (appropriate for A/B)

- Well-known company, reasonable work, good learning environment, but the specific role is not deeply aligned with profile strengths
- The tech stack is standard and the domain is reasonable but not a personal interest
- The company has good engineering culture but the role is more operational than creative
- The career ceiling is solid but not exceptional

### Warning signs (pushes toward C/F)

Read `preferences.toml` for explicit dealbreakers, then also watch for:
- Description focuses on processes, meetings, stakeholder management rather than writing code
- Tech work is primarily integration — connecting third-party services rather than building systems
- Role is framed around a single tool or framework
- Company's "engineering" is configuring vendor products rather than building technology
- Significant customer-facing responsibilities (check `preferences.toml` for whether this is a hard exclusion)

---

## Profile elements to reference in fit assessments

When writing fit assessments, you must connect jobs to the profile concretely. Here is a checklist of profile elements to reference:

**For every job (minimum):**
- At least one project from `profile/projects/` that demonstrates relevant capability, named explicitly — preferably an active or complete project. If the best match is an abandoned project, acknowledge that the evidence is weaker.
- The candidate's proficiency level for the job's required technologies from `skills.md`
- Seniority assessment grounded in `experience.md` and the per-project files in `profile/projects/`

**For SS/S jobs (mandatory):**
- All active and complete projects from `profile/projects/` that are relevant, with explanation of what each demonstrates. Paused or dormant projects should be included if directly relevant; abandoned projects should only appear if they add something the active/complete projects don't cover.
- Full technology overlap analysis from `skills.md`
- Gaps identified from `portfolio-gaps.md` with severity assessment
- Visa timeline from `visa.md` and sponsorship viability
- Career trajectory fit against targets in `preferences.toml`
- At least one specific element from the candidate's background that makes them a standout applicant

**For A/B jobs:**
- Primary project alignment (at least one named project)
- Key gap or reason it's not S-tier
- Whether it addresses any gaps from `portfolio-gaps.md`

**For C/F jobs:**
- The specific dealbreaker, referencing the relevant profile file (e.g., "hard seniority mismatch per experience.md" or "excluded sector per preferences.toml")
