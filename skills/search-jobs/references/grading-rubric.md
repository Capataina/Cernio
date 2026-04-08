# Job Grading Rubric

> How to evaluate and grade every job that passes triage. Applied consistently to every role across every company.

---

## Evaluation Dimensions

Every job is assessed across these dimensions. Not every dimension has equal weight — career ceiling and seniority match are non-negotiable, while tech stack relevance is a tiebreaker.

### Critical (must be acceptable or the role is an F)

| Dimension | What to assess | Good signal | Bad signal |
|-----------|---------------|-------------|------------|
| **Career ceiling** | Does this role/domain lead to £500K+ income over a career? What do people in this domain earn at 10-15 years experience? | Systems engineering, AI/ML infrastructure, trading systems, platform engineering, distributed systems | IT support, QA-only, narrow specialisation with no upward path, operational roles with no engineering progression |
| **Seniority match** | Is this role actually achievable given the profile? A perfect role you can't get hired for is worthless. | New grad, entry-level, junior, no years stated with reasonable expectations, internship | Hard "5+ years required", scope that clearly demands production experience, senior/staff expectations |

### High weight

| Dimension | What to assess | Good signal | Bad signal |
|-----------|---------------|-------------|------------|
| **Skill breadth** | Does the role expose you to multiple layers (backend + infra + data + some frontend) or lock you into one narrow thing? Breadth early in career = more options later. | "Full-stack infrastructure", multiple team rotations, cross-functional work, exposure to different parts of the stack | "You will exclusively maintain this one microservice", single-tool focus, no cross-team collaboration |
| **Company signal** | Does having this company on the CV open doors? First jobs matter disproportionately for brand recognition. | Palantir, Spotify, Google, Stripe, well-known scale-ups, companies engineers respect | Unknown company with no engineering reputation, body shops, agencies, companies nobody in tech has heard of |
| **Technical depth** | Will you work on genuinely hard problems? Performance-critical code, distributed systems, scale? | "Millions of transactions", "hundreds of services", "low-latency", distributed systems, data infrastructure | CRUD apps, config management, glue code, no mention of scale or technical challenge |
| **Sponsorship viability** | Can and will they sponsor a Skilled Worker visa when needed? Check `profile/visa.md` for current visa status and sponsorship timeline. | Large company with active sponsor licence, established graduate programme, "we sponsor" in description | Small startup with no sponsor licence, "must have existing right to work", no mention of sponsorship |

### Medium weight

| Dimension | What to assess | Good signal | Bad signal |
|-----------|---------------|-------------|------------|
| **Domain transferability** | Are the skills you build here useful elsewhere, or are they company-specific? | Infrastructure, platform engineering, database systems — transferable everywhere | Proprietary internal DSL nobody else uses, hyper-specific product knowledge that doesn't transfer |
| **Growth environment** | Will you learn fast? Strong engineers, mentorship, code review culture? You learn fastest from people better than you. | Dedicated mentor, small team of strong engineers, open source contributions, engineering blog, "we pair program" | No mentorship mentioned, very large teams where you'd be invisible, no engineering culture signals |
| **Tech stack relevance** | Are the technologies used in production broadly valuable? | Rust, Go, Java, Python, C++, Kubernetes, Kafka, PostgreSQL, distributed systems tooling | Proprietary framework nobody else uses, legacy-only stack (COBOL, ancient Java), no modern tooling |

---

## Grading Scale

| Grade | Meaning | When to assign |
|-------|---------|----------------|
| **SS** | Apply immediately, prioritise above all others | Achievable seniority + high career ceiling + strong company signal + broad skill exposure + sponsors visas + technically deep. Everything lines up. |
| **S** | Strong application candidate | Hits most dimensions well. Maybe one gap — great role at a lesser-known company, or strong company but slightly narrow scope. Still very much worth pursuing. |
| **A** | Worth applying to | Good on several dimensions, 1-2 notable weaknesses. The opportunity is still valuable despite gaps. |
| **B** | Backup / worth watching | Decent role but either too narrow, weak company signal, uncertain sponsorship, or domain doesn't transfer well. Apply if nothing better is available. |
| **C** | Only if nothing better exists | Technically achievable but limited ceiling, narrow scope, or poor growth environment. A job, but not a career move. |
| **F** | Don't apply | Wrong seniority (hard requirement mismatch), dead-end domain, dealbreaker present, or fundamentally misaligned with career goals. |

---

## How to present grades

When evaluating a batch of jobs, present every triaged role with its grade and a one-line justification. Group by grade for scannability:

```
## SS
- Software Engineer, New Grad - Infrastructure @ Palantir
  New grad, tier-1 brand, infrastructure breadth across 4 tracks, known sponsor

## S
- Software Engineer, New Grad - Production Infra @ Palantir
  Same brand, genuine new grad, more ops-flavoured but strong growth environment

## A
- Backend Software Engineer - Infrastructure @ Palantir
  Rust in stack, perfect domain, but reads as mid-level — reach application

## F
- Account Executive - Backstage @ Palantir
  Sales role, not engineering
```

For roles graded S or above, always fetch the full description and provide detailed evaluation. For A and B, fetch and evaluate if the user wants deeper assessment. For C and F, the one-line justification is sufficient.
