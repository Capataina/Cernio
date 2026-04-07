# Company Grading Rubric

> How to evaluate and grade companies during the populate-db process. More lenient than job grading — a B-tier company might have an S-tier role. But dead, dying, or irrelevant companies shouldn't enter the database.

---

## Evaluation Dimensions

### High weight

| Dimension | What to assess | Good signal | Bad signal |
|-----------|---------------|-------------|------------|
| **Engineering reputation** | Is this company known for good engineering? Do they have an engineering blog? Open source contributions? Do engineers speak positively about working there? | Active engineering blog, open source projects, talks at conferences, positive Glassdoor engineering reviews | No engineering presence online, negative reviews about tech culture, "we use spreadsheets for everything" |
| **Technical alignment** | Does their core work involve problems that match the profile? Systems, infrastructure, ML, data, low-latency? | Core product involves systems engineering, data infrastructure, ML pipelines, distributed systems | Primarily a marketing/sales/consulting company that happens to have a website |
| **Growth trajectory** | Growing, stable, or declining? Recent funding? Hiring actively? | Recent funding round, active job listings, headcount growth, expanding into new markets | Layoffs, no new funding in years, careers page empty or removed, shrinking headcount |

### Medium weight

| Dimension | What to assess | Good signal | Bad signal |
|-----------|---------------|-------------|------------|
| **Sponsorship likelihood** | Do they have a Skilled Worker sponsor licence? Size and history suggest they could sponsor? | Listed on UK sponsor register, large company with HR infrastructure, history of hiring international talent | Tiny company with no HR, explicitly "must have existing right to work" everywhere |
| **Career ceiling** | Can you grow here? Is there a path from entry-level to senior and beyond? | Multiple engineering levels, clear progression, engineers who've grown from junior to senior internally | Flat org with no progression, everyone is "engineer" forever, or too small to have levels |
| **Company stability** | Will they exist in 2 years? Are they profitable or well-funded enough to survive? | Profitable, well-funded (runway), established revenue, strong market position | Pre-seed with no revenue, burning cash with no path to profitability, in a dying market |

### Low weight (tiebreakers)

| Dimension | What to assess |
|-----------|---------------|
| **Company signal on CV** | Would having this company name on a CV impress future employers? |
| **Location convenience** | London office? Good hybrid policy? |
| **Culture fit** | Engineering-led? Small teams? Autonomy? |

---

## Grading Scale

Company grades are gentler than job grades — they assess whether a company is worth having in the database and monitoring, not whether to apply right now.

| Grade | Meaning | When to assign | Example |
|-------|---------|----------------|---------|
| **S** | Excellent company — high priority for job monitoring | Strong engineering reputation + clear technical alignment + growing + likely to sponsor + career progression | Palantir, Cloudflare, Stripe, XTX Markets |
| **A** | Good company — monitor regularly | Good on most dimensions, one or two weaknesses. Solid engineering work, decent growth. | Mid-stage fintech with good tech team but less brand recognition |
| **B** | Decent company — worth tracking | Has relevant engineering work but weaker on some dimensions. Maybe small, maybe uncertain growth, maybe narrow domain. | 30-person startup doing interesting Rust work, but pre-revenue with unclear runway |
| **C** | Marginal — keep in DB but low priority | Borderline relevance. Maybe the work is tangential, or the company is struggling, or sponsorship is unlikely. Still better than not knowing about them. | Small consultancy that occasionally works on relevant infra projects |

**Companies that don't make the cut (removed from potential.md, never enter DB):**
- Dead website / company shut down
- Acquired and fully absorbed (no separate hiring)
- No engineering team or engineering work
- Work is completely unrelated to the profile
- So small that hiring an entry-level engineer is implausible

---

## How grading interacts with job grading

Company grade sets a baseline but doesn't cap job grade:

| Company | Company Grade | Job Grade | Makes sense? |
|---------|-------------|-----------|--------------|
| Palantir | S | SS (New Grad Infra) | Yes — great company, perfect role |
| Small Rust startup | B | S (founding engineer, Rust, systems) | Yes — company is risky but the role is exceptional for growth |
| Big bank | A | C (maintaining legacy Java) | Yes — good company, dead-end role |
| Dead startup | Removed | N/A | Never reaches job search |

The company grade tells you "should we monitor this company for jobs?" The job grade tells you "should we apply to this specific role?"

---

## Presenting company grades

When processing a batch from potential.md, present results grouped by outcome:

```
## Added to database

### S-tier
- Cloudflare (S) — Rust in production, strong engineering blog, growing, known sponsor
- XTX Markets (S) — Tech-first market maker, London HQ, bespoke low-latency infrastructure

### A-tier
- Form3 (A) — Good payment infrastructure engineering, backed by Goldman/Lloyds, less brand recognition

### B-tier
- Coadjute (B) — Interesting settlement infrastructure, small team, early stage

## Removed (not added)
- Vypercore — Website down, appears inactive
- Argent — Pivoted to consumer product, engineering team unclear
```
