# grade-companies

**Read ALL files in `profile/` before grading. Every file. Do not skip any — missing context leads to grading errors.** The profile is a living system that changes as the candidate's portfolio grows. Never rely on embedded snapshots or cached assumptions about the profile.

Grades ungraded companies in the Cernio database against the user's structured profile. Use when asked to "grade companies", "evaluate companies", "rate the universe", "score the database", "assess companies", "grade ungraded", "update company grades", or when a populate-db run has added companies without grades. Not for grading individual jobs (that's the job grading skill), discovering new companies (that's the discovery skill), or populating the database from potential.md (that's populate-db).

---

## Mandatory reads — do not proceed without completing these

**STOP. Before grading any company, you MUST read these files in full:**

1. **Every file in `profile/`** — all 15 files, no exceptions. Missing even one file (e.g., skipping `visa.md`) leads to grading errors on critical dimensions like sponsorship.
2. **`references/grading-rubric.md`** — the complete rubric with evaluation dimensions, worked examples, and boundary cases. This is HOW you grade.
3. **`references/profile-context.md`** — tells you what to extract from each profile file and how to synthesise it for company evaluation. This bridges the profile to the rubric.

**When delegating grading to subagents:** embed the FULL TEXT of both reference files and all relevant profile data in each agent's prompt. Subagents cannot read files from the repo.

**Do not begin evaluating any company until all mandatory reads are complete.**

---

## Why company grading exists

Company grading answers one question: **is this company worth monitoring for jobs?**

The Cernio pipeline works in stages: discovery finds companies, population resolves their ATS endpoints, and job search fetches open roles. Grading sits between population and job search as a quality gate. Without it, every company gets equal search priority, and the user drowns in noise from companies that were never going to produce relevant roles.

But grading serves a deeper purpose than filtering. The candidate's specific profile — their technical strengths, credential gaps, visa constraints, and career targets — creates a strategic landscape where company choice matters more than it would for a conventional candidate. Read the profile files to understand this landscape:

- **Engineering reputation is weighted high** because read `experience.md` — if formal work experience is limited, company signal on a CV partially compensates for the absence of prior employer names.
- **Sponsorship capability is weighted high** because read `visa.md` — there may be a hard deadline after which sponsorship becomes mandatory. A company that cannot or will not sponsor becomes a career dead-end after that date.
- **Career ceiling is weighted high** because read `preferences.toml` — the long-term target determines what "high ceiling" means. A company where everyone stays at the same level forever limits trajectory regardless of entry-level experience quality.
- **Technical alignment is weighted high** because read `projects.md` and `skills.md` — the portfolio has specific technical strengths. A company doing work outside those strengths wastes the strongest part of the profile.

The grade captures this multidimensional judgement in a single letter so that downstream processes (job search, TUI display, cleanup) can act on it without re-evaluating the company every time.

---

## How grading works

### 1. Load context

First, read ALL files in `profile/` — every single one. See `references/profile-context.md` for what to extract from each file and how to synthesise it for company evaluation.

Then read `references/grading-rubric.md` for the full rubric with evaluation dimensions, worked examples, and boundary cases.

Both are essential. The profile files tell you what matters for this specific candidate. The rubric tells you how to apply those priorities systematically. The profile-context reference tells you how to bridge the two.

### 2. Query the database

Find companies that need grading:

```sql
SELECT id, name, website, what_they_do, status, location, sector_tags, why_relevant
FROM companies
WHERE grade IS NULL
  AND status != 'archived';
```

This returns companies that have been populated (resolved or bespoke) but never graded, plus any potential-status companies that entered the database without a grade. Archived companies are excluded — they were already evaluated and set aside.

### 3. Research each company

For each ungraded company, build enough understanding to evaluate it across every rubric dimension. What "enough" means varies by company:

- A well-known company (Cloudflare, Stripe, Palantir) needs minimal research — you likely know enough from training data, supplemented by checking their current state (still hiring? still independent? any recent changes?).
- A lesser-known company needs real investigation. Visit their website, check their engineering blog, look for open-source contributions, read recent news, check the UK sponsor register, assess their careers page.
- A startup with minimal web presence needs careful triangulation. Crunchbase for funding, LinkedIn for headcount signals, Companies House for registration details, GitHub for engineering activity.

The goal is confident evaluation, not exhaustive research. When you have enough signal to place a company in a grade tier with clear reasoning, move on. When signal is weak and ambiguous, say so in the reasoning — a grade with acknowledged uncertainty is more honest than a confident grade based on thin evidence.

### 4. Assign grades

Apply the rubric from `references/grading-rubric.md`. Each company gets:

- **Grade**: S, A, B, or C
- **Grade reasoning**: A concise paragraph explaining which dimensions drove the grade, what the company's strengths and weaknesses are, and any notable uncertainties. This reasoning is stored in the database and read by both the user and future agents — make it genuinely informative.
- **Why relevant** (update if the existing value is thin or outdated): What makes this company relevant to the profile specifically.

The grade reasoning must include **specific, verifiable evidence** across these categories:

**Profile connection (mandatory):**
- Name specific projects from `profile/projects.md` that align with the company's work (e.g., "Nyquestro's lock-free matching engine maps directly to their exchange infrastructure")
- Name specific technologies from `profile/skills.md` the company uses (e.g., "Production Rust shop — the candidate's primary language")
- Reference the candidate's career targets from `preferences.toml` where relevant

**Company evidence (include what you can verify — absence of evidence is not evidence of absence):**
- Recent funding, growth signals, or hiring activity IF you can find it — but do not penalise a company simply because you couldn't find public funding data. Private companies and profitable bootstrapped companies may have no public funding information.
- Engineering reputation signals: OSS contributions, engineering blog, conference talks, known engineers
- Sponsorship evidence: UK sponsor register check, international team composition, job listing mentions
- Number of open engineering roles IF visible on their careers page or ATS

**Boundary clarity (mandatory):**
- Explicitly state why this grade and not the adjacent one (e.g., "A rather than S because engineering reputation is strong but technical alignment is moderate — their core work is application-layer payments, not systems infrastructure")

**Important:** Grade based on evidence you can actually find and verify. If you cannot find funding information, hiring numbers, or growth metrics for a company, say so honestly in the reasoning rather than guessing. A company with no public funding data might be bootstrapped and profitable. A company with no visible open roles might hire through referrals. Do not downgrade companies solely because information is unavailable — downgrade only when you find actual negative signals.

An example of **unacceptable** grade reasoning:
> "Good company, sponsors visas, relevant tech."

An example of **acceptable** grade reasoning:
> "S-tier. Core product is a distributed time-series database built in Rust — direct overlap with the candidate's Nyquestro (lock-free matching engine) and systems programming depth. Active OSS presence (12 public repos, regular commits). Engineering blog with substantive posts on storage engine internals. Confirmed Skilled Worker sponsor (licence active on gov.uk register). Series B ($45M, 2025) — actively growing, 8 open engineering roles on Greenhouse. Clear IC ladder visible on Levels.fyi. Technical alignment is near-perfect; the only weakness is relatively low brand recognition outside the database community, which limits CV signal compared to tier-1 names."

### 5. C-tier companies stay active

C-tier companies are NOT automatically archived. They remain in the active search pool because job grading handles quality filtering — a C-tier company might still have one genuinely good role that would be graded A or B on its own merits. The cost of searching a few extra companies is low (extra grading time), while the cost of missing a good role at a "marginal" company is an unrecoverable loss.

Companies should only be archived manually when there is a hard reason (excluded sector, dissolved company, no engineering team) — not just because the overall grade is C.

**Do NOT set `status = 'archived'` when grading C-tier companies.** Leave their status unchanged. The SQL for C-tier is the same as S/A/B — just the grade value differs.

### 6. Present results for review

Present all graded companies grouped by grade tier, with the reasoning visible. The user reviews and confirms before anything is written to the database.

Format:

```
## Grading Results

### S-tier
- **Cloudflare** — Exceptional infrastructure company. Core product (CDN, edge compute, Workers
  runtime) directly overlaps with the candidate's systems engineering focus. Production Rust
  usage aligns with primary language. Active OSS, substantive engineering blog. Confirmed
  Skilled Worker sponsor. Clear IC track to Principal. Recent Q1 earnings show continued growth.
  Why relevant: Edge infrastructure and performance-critical networking maps to Nyquestro's
  matching engine architecture and NeuroDrive's real-time distributed simulation. Rust in
  production stack.

### A-tier
- **Form3** — Solid payments infrastructure. Core work (Faster Payments, BACS, CHAPS processing)
  involves distributed systems, strict ordering, and fault tolerance — technically deep and
  adjacent to exchange infrastructure. Go-heavy but systems-oriented. Series C, 200+ employees,
  confirmed sponsor. Weaker on brand recognition outside fintech.
  Why relevant: Payment message routing and transaction ordering are structurally similar to
  order matching. The candidate's distributed systems experience from NeuroDrive transfers well.

### C-tier (will be archived)
- **SmallCo Ltd** — Marketing-focused consultancy. Engineering team of 3, no public repos,
  no engineering blog. Core work is CMS customisation, not systems engineering. Not on the
  sponsor register. No alignment with the candidate's infrastructure/systems focus.
  Why relevant: Originally discovered via fintech list but core work is application-layer
  marketing technology — no connection to the candidate's projects or technical interests.
```

### 7. Write to database

After user confirmation, execute the updates. **Use EXACTLY this SQL format — do not rename columns, do not add extra fields, do not change the syntax.**

```sql
-- EXACT format for ALL grades (S, A, B, C) — do not modify column names:
UPDATE companies SET grade = 'X', grade_reasoning = 'reasoning text here', why_relevant = 'updated relevance text', graded_at = datetime('now') WHERE id = N;
```

**Critical SQL rules for agents:**
- Column names are `grade`, `grade_reasoning`, `why_relevant`, `graded_at`. NOT `reasoning`, NOT `relevance`, NOT any other variation.
- Do NOT set `status = 'archived'` for any grade including C. C-tier companies stay active.
- Escape single quotes in text by doubling them: `it''s` not `it's`
- `graded_at` must be `datetime('now')`, not a hardcoded date string
- Every statement must end with a semicolon
- One UPDATE per line, no multi-line statements

---

## Regrading

Sometimes the user will ask to regrade specific companies — perhaps new information has emerged, or the rubric has evolved. When regrading:

- Query by name or ID rather than the `grade IS NULL` filter
- Show the previous grade and reasoning alongside the new evaluation
- Explain what changed and why the new grade differs (or confirm it if it does not)

---

## Quality checklist

Before presenting results to the user:

- [ ] All files in `profile/` were read before grading began
- [ ] Every graded company has a grade_reasoning that explains the evaluation across the high-weight dimensions (engineering reputation, technical alignment, growth trajectory) with specific evidence
- [ ] Grade reasoning distinguishes this company from adjacent tiers — it is clear why an A is not an S, or why a B is not a C
- [ ] C-tier companies are flagged for archival and the user understands what archival means
- [ ] Companies with weak or ambiguous signal have uncertainty acknowledged in the reasoning rather than false confidence
- [ ] The why_relevant field connects the company to the profile specifically — not generic praise, but what about this company matters for this candidate
- [ ] No company was graded without checking its current state — a company that was S-tier a year ago might have had layoffs, pivots, or acquisitions since
- [ ] Sponsorship assessment is grounded in evidence (sponsor register check, company size, international hiring history) rather than assumption
- [ ] Results are presented grouped by tier for easy review before any database writes happen
