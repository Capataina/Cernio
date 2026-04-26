# Prioritisation Guide

> How to order the pending queue so the most promising jobs get graded first. The user should see actionable results early, not after the full backlog is cleared.

---

## Table of Contents

1. [Why prioritisation matters](#why-prioritisation-matters)
2. [The compound signal](#the-compound-signal)
3. [Worked example: ordering 50 pending jobs](#worked-example-ordering-50-pending-jobs)
4. [The SQL query with prioritisation](#the-sql-query-with-prioritisation)
5. [Adapting priority across sessions](#adapting-priority-across-sessions)

---

## Why prioritisation matters

Grading happens in batches of ~30 jobs per session. If 142 jobs are pending, the user sees results from the first batch while 112 remain ungraded. If the first batch is randomly selected, the user might see 25 B/C/F grades and 5 scattered A grades — no clear action to take, and the SS-tier role buried in batch 4 remains invisible.

If the first batch prioritises the highest-signal jobs, the user sees the strongest opportunities immediately and can start preparing applications while later batches process the tail. The first batch should contain the jobs most likely to be SS or S.

**Important interaction with grading:** Because prioritisation deliberately puts the best jobs first, every batch is skewed toward high-quality roles. This means within-batch relative grading would systematically deflate grades. The grading rubric addresses this with calibration-anchored grading — agents compare each job against real examples from the existing database at each grade tier, not against other jobs in the batch. See the grading rubric's "Cross-referencing and Relative Grading" section for the full mechanism.

---

## The compound signal

Priority is a product of three factors:

```
priority = company_grade x title_promise x role_type_alignment
```

### Factor 1: Company grade

The company grade is already in the database. It reflects the company's engineering reputation, sponsorship likelihood, and overall relevance to the profile.

| Company grade | Priority weight | Reasoning |
|---------------|----------------|-----------|
| S | Highest | These companies were judged as excellent fits. Any role from an S-company has strong baseline signal. |
| A | High | Good companies. Most of their roles will be worth evaluating carefully. |
| B | Moderate | Decent companies. Some roles will be good, many will be marginal. |
| NULL / ungraded | Low | Unknown signal. Grade the role on its own merits but don't prioritise it in the queue. |

### Factor 2: Title promise

Title keywords carry signal about whether the role is likely to be a strong match. These aren't filters (everything in the queue already passed filtering) — they're priority boosters.

**High-promise keywords** (boost priority):

| Keyword | Why it boosts |
|---------|---------------|
| Graduate, New Grad, Grad | Explicitly entry-level. Seniority match is guaranteed. Removes the biggest source of F grades. |
| Entry, Early Career | Same as above. |
| Junior | Likely achievable. Low seniority risk. |
| Infrastructure, Infra | High-ceiling domain directly aligned with profile. |
| Systems, System | Core alignment with profile identity. |
| Platform | Infrastructure-adjacent, high ceiling. |
| Rust | Primary language. Rare in job listings, strong signal when present. |
| Low-latency, Performance | Direct alignment with performance-critical projects in profile. Check `profile/projects/` for relevant work. |
| Trading, Quant, Quantitative | Domain alignment with financial/trading projects in profile. Check `profile/projects/`. |
| ML, Machine Learning, AI | Domain alignment with ML/AI projects in profile. Check `profile/projects/` and `skills.md`. |
| Compiler, Runtime | Direct alignment with compiler/runtime projects in profile. Check `profile/projects/`. |
| Backend, Back-end | Core engineering work, likely aligned. |

**Moderate-promise keywords** (neutral priority):

| Keyword | Why it's neutral |
|---------|------------------|
| Software Engineer (alone) | Too generic to boost. Could be anything from systems to CRUD. |
| Developer, Dev | Same as above. |
| Full-stack, Fullstack | Mixed signal — might be backend-heavy or might be mostly frontend. |
| Cloud, DevOps | Somewhat aligned but often operational rather than systems. |
| Data | Ambiguous — could be data infrastructure (good) or data analytics (less aligned). |
| SRE, Reliability | Solid but more operational than systems. |
| Security, Cyber | Interesting domain but less profile alignment. |

**Low-promise keywords** (lower priority, but do not exclude):

| Keyword | Why it's deprioritised |
|---------|------------------------|
| Senior, Lead | Higher seniority risk. Many will be F, but some are grading-deflated and worth checking. |
| Analyst | Often non-engineering. Sometimes data engineering. Worth grading but not prioritising. |
| Research | Could be excellent (research engineer) or misaligned (pure research with no engineering). |

### Factor 3: Role type alignment

Beyond individual keywords, the combination of title terms signals the role type:

| Role type pattern | Priority | Reasoning |
|-------------------|----------|-----------|
| [Entry-level keyword] + [Systems/Infra keyword] + [S-company] | Top priority | Trifecta: achievable, high-ceiling, strong company. |
| [Entry-level keyword] + [Any engineering] + [S-company] | High | Achievable at a top company. Even if the specific role isn't perfect, the company brand is valuable. |
| [Systems/Infra keyword] + [A-company] | High | Good domain at a good company. Seniority needs checking but worth prioritising. |
| [Trading/Quant keyword] + [Any company grade] | High | Domain is so well-aligned with the profile that even at a lesser-known company it's worth early evaluation. |
| [Rust/Compiler/Runtime keyword] + [Any company grade] | High | Rare and highly aligned. These roles are uncommon enough that they deserve immediate attention. |
| [Generic engineering] + [B-company] | Moderate | Needs evaluation but less likely to be S-tier. |
| [Senior/Lead keyword] + [Any company] | Lower | Likely F on seniority but some will be achievable. Grade after higher-priority jobs. |

---

## Worked example: ordering 50 pending jobs

Given 50 ungraded jobs, here's how to select and order the first 15 for grading:

### The queue

```
 1. Software Engineer, Infrastructure @ Cloudflare (S-company)
 2. Senior Backend Engineer @ Revolut (A-company)
 3. Software Engineer @ TechStartup Ltd (B-company)
 4. Graduate Software Engineer @ Palantir (S-company)
 5. Data Analyst @ Monzo (A-company)
 6. ML Engineer, Model Serving @ Databricks (A-company)
 7. Platform Engineer @ Wise (A-company)
 8. Junior Developer @ UnknownCorp (B-company)
 9. Senior Systems Engineer @ Bloomberg (S-company)
10. Software Engineer, Rust — Trading Systems @ small quant firm (B-company)
11. Backend Engineer @ Deliveroo (A-company)
12. DevOps Engineer @ mid-size SaaS (B-company)
13. Graduate Engineer, Compilers @ Arm (S-company)
14. Software Engineer @ another B-company (B-company)
15. Junior Infrastructure Engineer @ Grafana Labs (A-company)
16. Senior Software Engineer @ Google (S-company)
17. Research Engineer @ DeepMind (S-company)
18. Full Stack Developer @ agency (B-company)
19. Quantitative Developer @ XTX Markets (S-company)
20. SRE @ Spotify (A-company)
... (30 more, mix of grades and titles)
```

### Prioritised first 15

| Priority | Job | Reasoning |
|----------|-----|-----------|
| 1 | #4: Graduate SWE @ Palantir | S-company + explicit new grad + engineering. Guaranteed seniority match at a top company. |
| 2 | #13: Graduate Engineer, Compilers @ Arm | S-company + explicit grad + compiler domain (check `profile/projects/` for compiler work). |
| 3 | #19: Quantitative Developer @ XTX Markets | S-company + quant/trading domain (check `profile/projects/` for trading projects). Even without "grad" keyword, the domain fit is exceptional. |
| 4 | #1: SWE Infrastructure @ Cloudflare | S-company + infrastructure keyword. No grad indicator but infrastructure at Cloudflare is very likely to have entry pathways. |
| 5 | #10: SWE Rust, Trading @ small quant | Only B-company, but Rust + trading is a double-hit on the rarest, most aligned keywords. Worth immediate evaluation despite company grade. |
| 6 | #6: ML Engineer, Model Serving @ Databricks | A-company + ML + model serving (check `profile/projects/` for inference/ML work). |
| 7 | #15: Junior Infra Engineer @ Grafana Labs | A-company + junior + infrastructure. Triple positive signal. |
| 8 | #7: Platform Engineer @ Wise | A-company + platform + fintech domain (check `profile/projects/` for fintech alignment). |
| 9 | #17: Research Engineer @ DeepMind | S-company + research/ML. Might be unachievable (PhD often required) but S-company + ML alignment warrants early evaluation. |
| 10 | #9: Senior Systems Engineer @ Bloomberg | S-company + systems. "Senior" is a risk but Bloomberg's title levels may be accessible. Worth checking early because if achievable, this is S-tier. |
| 11 | #16: Senior SWE @ Google | S-company. "Senior at Google" is almost certainly unachievable, but it's fast to confirm and cross off. |
| 12 | #11: Backend Engineer @ Deliveroo | A-company + backend. Solid baseline. |
| 13 | #2: Senior Backend @ Revolut | A-company + fintech. "Senior" at Revolut might be level 2. Worth checking. |
| 14 | #20: SRE @ Spotify | A-company. SRE is less aligned but Spotify is a strong brand. |
| 15 | #5: Data Analyst @ Monzo | A-company, but "Data Analyst" is low-alignment. Included because A-company jobs should all be evaluated before B-company jobs. |

### What got deprioritised (batch 2+)

- #3, #8, #14: B-company with generic titles. No strong signal to warrant early grading.
- #12: B-company + DevOps. Operational lean, lower alignment.
- #18: B-company + agency + full-stack. Multiple negative signals. Grade last.

---

## The SQL query with prioritisation

The basic query gets all pending jobs. For prioritisation, the agent reads the results and re-orders them using the compound signal logic above. The database ordering by company grade gets the first approximation right:

```sql
SELECT j.id, j.title, j.url, j.location, j.raw_description, j.posted_date,
       c.name AS company_name, c.grade AS company_grade, c.what_they_do
FROM jobs j
JOIN companies c ON c.id = j.company_id
WHERE j.evaluation_status = 'pending'
   OR j.evaluation_status = 'evaluating'
ORDER BY
    CASE c.grade
        WHEN 'S' THEN 1
        WHEN 'A' THEN 2
        WHEN 'B' THEN 3
        ELSE 4
    END,
    j.id ASC
```

After fetching, the agent applies title-keyword boosting to reorder within each company-grade tier. This produces the final priority order for the batch.

---

## Adapting priority across sessions

In the first grading session, the queue is entirely ungraded. Prioritise as described above.

In subsequent sessions, the remaining queue is the tail — it has fewer S-company jobs and more B-company jobs with generic titles. This is expected. The prioritisation still applies (grade the best of what remains first), but the agent should set expectations: "The remaining 80 jobs are mostly B-company roles with generic titles. Expect more B/C/F grades and fewer S grades in this batch."

If new jobs enter the queue between sessions (from a fresh search-jobs run), they should be interleaved with the existing queue based on their compound signal, not appended to the end. A freshly discovered "Graduate SWE @ Stripe" should be graded before the 60 remaining B-company generic titles from the last session.
