# grade-companies

Grades ungraded companies in the Cernio database against the user's structured profile. Use when asked to "grade companies", "evaluate companies", "rate the universe", "score the database", "assess companies", "grade ungraded", "update company grades", or when a populate-db run has added companies without grades. Not for grading individual jobs (that's the job grading skill), discovering new companies (that's the discovery skill), or populating the database from potential.md (that's populate-db).

---

## Why company grading exists

Company grading answers one question: **is this company worth monitoring for jobs?**

The Cernio pipeline works in stages: discovery finds companies, population resolves their ATS endpoints, and job search fetches open roles. Grading sits between population and job search as a quality gate. Without it, every company gets equal search priority, and the user drowns in noise from companies that were never going to produce relevant roles.

But grading serves a deeper purpose than filtering. Caner is a strong technical builder with no formal work experience, a 2:2 from York, and a Graduate visa that expires in August 2027. These facts create a strategic landscape where company choice matters more than it would for a candidate with a conventional CV:

- **Engineering reputation is weighted high** because company signal on a CV partially compensates for the absence of prior employer names. "Palantir" or "Cloudflare" on a first job opens doors that credentials alone cannot.
- **Sponsorship capability is weighted high** because there is a hard deadline. A company that cannot or will not sponsor is a career dead-end after August 2027, no matter how interesting the work.
- **Career ceiling is weighted high** because the long-term target is senior/principal systems engineering at £500K+ compensation. A company where everyone stays at the same level forever limits trajectory regardless of the entry-level experience quality.
- **Technical alignment is weighted high** because the portfolio is Rust-heavy systems engineering. A company doing enterprise Java or WordPress development wastes the strongest part of the profile.

The grade captures this multidimensional judgement in a single letter so that downstream processes (job search, TUI display, cleanup) can act on it without re-evaluating the company every time.

---

## How grading works

### 1. Load context

Read `references/profile-context.md` for the distilled profile used in evaluation. Read `references/grading-rubric.md` for the full rubric with evaluation dimensions, worked examples, and boundary cases.

Both files are essential. The profile context tells you what matters for this specific candidate. The rubric tells you how to apply those priorities systematically.

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

The grade reasoning should be written as if explaining to a smart colleague why you put this company in this tier and not the adjacent one. "Good company" is not reasoning. "Strong engineering reputation (active OSS, eng blog, conference presence), core product is infrastructure-level (CDN, DNS, edge compute), confirmed Skilled Worker sponsor, clear IC progression to Staff+, but entry-level hiring is competitive and may prefer candidates with more conventional credentials" is reasoning.

### 5. Handle C-tier archival

Companies graded C get their status set to `archived`. This means:
- They are excluded from job searches (the search script skips archived companies)
- They disappear from the TUI's active view
- They remain in the database for deduplication (discovery will not re-discover them)
- They preserve their grade and reasoning (the evaluation is not lost)

Archival is not deletion. A C-tier company that was properly evaluated and found marginal is more valuable than a gap in the database — it prevents wasted effort in future discovery runs.

### 6. Present results for review

Present all graded companies grouped by grade tier, with the reasoning visible. The user reviews and confirms before anything is written to the database.

Format:

```
## Grading Results

### S-tier
- **Cloudflare** — Strong engineering reputation (active OSS, eng blog, Rust in production),
  core infrastructure product, confirmed sponsor, clear Staff+ IC track.
  Why relevant: CDN/edge infrastructure aligns with systems engineering focus;
  known for hiring strong builders regardless of credential gaps.

### A-tier
- **Form3** — Solid payments infrastructure engineering, Go-heavy but systems-oriented,
  backed by Goldman/Lloyds, confirmed sponsor. Weaker on brand recognition.
  Why relevant: Payment infrastructure is technically deep; good first-job signal.

### B-tier
- **Coadjute** — Interesting settlement infrastructure, small team (30-50),
  early stage but funded. Sponsorship uncertain at this size.
  Why relevant: Niche infrastructure work, Rust-adjacent problem space.

### C-tier (will be archived)
- **SmallCo Ltd** — Marketing-focused consultancy, engineering team of 3,
  no evidence of sponsorship capability, work is primarily application-layer.
  Why relevant: Originally discovered via fintech list but core work is not infrastructure.
```

### 7. Write to database

After user confirmation, execute the updates:

```sql
-- For S, A, B tier companies:
UPDATE companies
SET grade = 'X',
    grade_reasoning = 'reasoning text here',
    graded_at = datetime('now'),
    why_relevant = 'updated relevance text'
WHERE id = N;

-- For C tier companies (grade + archive):
UPDATE companies
SET grade = 'C',
    grade_reasoning = 'reasoning text here',
    graded_at = datetime('now'),
    why_relevant = 'updated relevance text',
    status = 'archived'
WHERE id = N;
```

---

## Regrading

Sometimes the user will ask to regrade specific companies — perhaps new information has emerged, or the rubric has evolved. When regrading:

- Query by name or ID rather than the `grade IS NULL` filter
- Show the previous grade and reasoning alongside the new evaluation
- Explain what changed and why the new grade differs (or confirm it if it does not)

---

## Quality checklist

Before presenting results to the user:

- [ ] Every graded company has a grade_reasoning that explains the evaluation across the high-weight dimensions (engineering reputation, technical alignment, growth trajectory) with specific evidence
- [ ] Grade reasoning distinguishes this company from adjacent tiers — it is clear why an A is not an S, or why a B is not a C
- [ ] C-tier companies are flagged for archival and the user understands what archival means
- [ ] Companies with weak or ambiguous signal have uncertainty acknowledged in the reasoning rather than false confidence
- [ ] The why_relevant field connects the company to the profile specifically — not generic praise, but what about this company matters for this candidate
- [ ] No company was graded without checking its current state — a company that was S-tier a year ago might have had layoffs, pivots, or acquisitions since
- [ ] Sponsorship assessment is grounded in evidence (sponsor register check, company size, international hiring history) rather than assumption
- [ ] Results are presented grouped by tier for easy review before any database writes happen
