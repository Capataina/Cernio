# Profile Context for Company Evaluation

**This file does not contain the candidate's profile. It tells you how to read the profile and what to extract for company grading.**

The candidate's profile is a living system maintained across multiple files in `profile/`. You must read ALL of them before grading any company. Hardcoded snapshots go stale silently — always read the source files for current data.

---

## Mandatory: Read every file in profile/

Read **all** of the following files at the start of every grading session. Do not skip any — missing context leads to grading errors.

| File | What to extract for company grading |
|------|-------------------------------------|
| `profile/personal.md` | Location, nationality, personal constraints that affect where and how the candidate can work. |
| `profile/visa.md` | **Critical.** Current visa type, expiry date, right-to-work status, sponsorship requirements and timeline. This determines how to weight sponsorship capability for every company. |
| `profile/education.md` | Degree classification, university, graduation date. Degree classification affects whether HR screening filters at prestigious companies will pass or reject the candidate. |
| `profile/experience.md` | Formal work experience (or lack thereof). The presence or absence of professional experience fundamentally changes what company attributes matter most — e.g., if there is no work history, engineering reputation and CV signal from the first employer become disproportionately important. |
| `profile/projects.md` | **Critical.** The project portfolio is the primary evidence of engineering capability. Extract: languages used, domains covered, technical depth demonstrated, and what kinds of company problems the portfolio aligns with. This is what determines technical alignment scoring. |
| `profile/skills.md` | Technical skills inventory — languages, frameworks, tools, domains. Use this to assess stack alignment with each company's engineering work. |
| `profile/preferences.toml` | **Critical.** Hard constraints (excluded sectors, dealbreakers, location requirements) and soft preferences (preferred sectors, role types, work arrangements). Any company in an excluded sector gets rejected outright — do not grade it. |
| `profile/portfolio-gaps.md` | Known gaps in the profile — skills the market asks for that the candidate currently lacks. Use this to assess whether a company's stack requirements hit a gap vs. a strength. |
| `profile/resume.md` | The structured CV. Cross-reference with other files for a complete picture of what the candidate presents to employers. |
| `profile/cover-letter.md` | Application narrative and positioning strategy. Understand how the candidate frames their strengths — this affects which company types the candidate can most credibly approach. |
| `profile/interests.md` | Domain interests and intellectual curiosities. These influence which sectors and problem domains the candidate finds genuinely motivating vs. merely acceptable. |
| `profile/certifications.md` | Professional certifications held. Relevant for companies that value or require specific credentials. |
| `profile/languages.md` | Spoken languages. Relevant for companies with international teams or specific language requirements. |
| `profile/military.md` | Military service status. May be relevant for defence-sector companies or security clearance eligibility. |
| `profile/volunteering.md` | Volunteering and community involvement. Minor factor but can signal cultural alignment with mission-driven companies. |

---

## What to build from the profile files

After reading all profile files, you should have a mental model of:

### 1. Technical identity

Synthesise from `projects.md`, `skills.md`, and `experience.md`:
- What is the candidate's primary language and technical domain?
- What kinds of engineering problems does the portfolio demonstrate competence in?
- What is the depth vs. breadth balance?
- Where does the portfolio convert most effectively — i.e., which company types would see this work and immediately recognise its relevance?

### 2. Career targets and constraints

Synthesise from `preferences.toml`, `visa.md`, `education.md`, and `experience.md`:
- What seniority level is the candidate targeting and what is achievable?
- What is the visa timeline and when does sponsorship become a hard requirement?
- What sectors are excluded? What sectors are preferred?
- What location constraints exist?
- What is the long-term career trajectory the candidate is optimising for?

### 3. Strategic position

Synthesise from all files:
- What are the candidate's strongest differentiators compared to a typical applicant at their level?
- What are the biggest weaknesses or gaps that will affect how companies evaluate them?
- What does the candidate need most from their first/next employer — brand signal, technical depth, sponsorship, mentorship, or something else?

---

## What matters most for company evaluation

The following evaluation priorities are derived from the profile. Read the profile files to understand the specific values — the framework below tells you how to weight them.

### 1. Engineering reputation

**Why it matters:** Read `experience.md` to understand the candidate's work history. If formal experience is limited, the name on the first/next employer compensates for gaps that credentials alone cannot fill. A company with strong engineering reputation provides career signal per year of employment. The less conventional the candidate's background, the more this dimension matters.

**How the profile informs this:** The gap between the candidate's demonstrated technical ability (from `projects.md`) and their formal credentials (from `education.md` and `experience.md`) determines how much weight to place on engineering reputation. A large gap means reputation matters enormously.

### 2. Technical alignment with the portfolio

**Why it matters:** Read `projects.md` and `skills.md` to understand what the candidate builds. Companies whose day-to-day engineering problems resemble the candidate's project work are companies where the portfolio converts most effectively in interviews and where daily work builds on existing strength.

**How the profile informs this:** Map the technologies, domains, and problem types from the projects to each company's engineering work. Direct overlap (same language, same domain, same problem type) is the strongest signal.

### 3. Sponsorship capability

**Why it matters:** Read `visa.md` to understand the candidate's current right-to-work status, visa expiry date, and what happens after expiry. This creates a timeline: during the current visa period, any employer can hire with zero friction. After expiry, sponsorship becomes mandatory. Companies that can sponsor are more valuable than those that cannot, even before sponsorship is needed.

**How the profile informs this:** The visa expiry date determines urgency. The gap between now and expiry determines how much risk a non-sponsoring company represents. A company that is unlikely to sponsor is not automatically bad if the role provides strong CV signal for a move to a sponsoring employer before the visa expires.

### 4. Career trajectory ceiling

**Why it matters:** Read `preferences.toml` and `interests.md` to understand long-term compensation and career targets. Companies with clear IC progression, competitive compensation, and promotion-by-impact cultures are worth more than companies with flat structures or management-only advancement tracks.

**How the profile informs this:** The candidate's stated long-term targets determine what "high ceiling" means concretely. Check whether the company's domain and structure can plausibly deliver that trajectory.

### 5. Growth and stability

**Why it matters:** Read `visa.md` again. A company that folds or freezes hiring during the visa window wastes precious time. Growth is also a proxy for entry-level hiring appetite — expanding companies are more willing to invest in candidates who need development than stable companies that prefer experienced replacements.

**How the profile informs this:** The visa timeline creates a minimum useful tenure. A company must survive and remain a viable employer for at least that long to deliver its career value.

---

## Sector preferences and dealbreakers

Read `preferences.toml` for the authoritative list of:
- **Hard exclusions** — sectors that are excluded entirely, not evaluated, not graded, not tracked
- **Preferred sectors** — in priority order
- **Unenthusiastic but not excluded** — sectors that produce lower grades unless the specific role is technically compelling

Do not hardcode these here. The preferences file is the source of truth and may be updated as the candidate's priorities evolve.
