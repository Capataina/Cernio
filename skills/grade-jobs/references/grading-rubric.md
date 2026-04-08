# Job Grading Rubric

> The complete evaluation framework for grading jobs. Every job is assessed across these dimensions, with the full reasoning chain visible. Applied consistently to every role across every company.

**Important:** This rubric references the candidate's profile throughout. All profile facts must come from reading the files in `profile/` — never from hardcoded values in this document. When the rubric says "the candidate's portfolio" or "the visa timeline," it means: read `projects.md`, read `visa.md`, etc.

## Table of Contents

1. [Grade Scale](#grade-scale)
2. [Evaluation Dimensions](#evaluation-dimensions)
3. [Dimension Interaction Effects](#dimension-interaction-effects)
4. [Worked Examples](#worked-examples)
5. [Boundary Cases](#boundary-cases)
6. [Common Misjudgments](#common-misjudgments)

---

## Grade Scale

Six grades from SS (exceptional) to F (reject). The key to accurate grading is understanding what separates each grade from the one above it.

| Grade | Meaning | What separates it from the grade above |
|-------|---------|----------------------------------------|
| **SS** | Apply immediately, prioritise above all others | Nothing — this is the top. Every critical and high-weight dimension is strong. The role is achievable, the ceiling is high, the company opens doors, the work is technically deep, and sponsorship is viable. These are rare. Expect 1-3 per hundred jobs. |
| **S** | Strong application candidate | One dimension is merely good rather than excellent. Perhaps the company is well-regarded but not tier-1, or the scope is slightly narrower than ideal, or the tech stack is adjacent rather than core. The role is still clearly worth pursuing and would be a strong career move. |
| **A** | Worth applying to | Two dimensions are notably weaker. The role is good but has clear gaps — maybe a strong company but the role itself is narrow, or technically deep work at a company with limited brand recognition. Still a net positive career move, but not the kind of opportunity that demands dropping everything. |
| **B** | Backup / worth watching | The role is acceptable but uninspiring on several fronts. Decent work, learning, or brand, but not enough of any to make it a priority. Apply if the pipeline is thin. These roles serve as practice applications and fallbacks. |
| **C** | Only if nothing better exists | The role is technically achievable but offers limited career value. Narrow scope, weak company signal, poor growth environment, or domain with low transferability. A job, not a career move. The main risk is opportunity cost — time spent here is time not spent building toward the long-term target (read `preferences.toml`). |
| **F** | Do not apply | A dealbreaker is present. The seniority is unachievable (read `experience.md` to determine), the domain is a career dead-end, the role is non-engineering, or a hard constraint from `preferences.toml` is violated. No amount of strength in other dimensions compensates. |

### The SS threshold

SS is not "very good." SS means: if the candidate were offered this role tomorrow, they would accept without hesitation and feel confident it was the right move for the next 2-3 years. The bar is deliberately high because SS roles get the most attention and the most detailed assessment. Inflating SS dilutes the signal the user relies on.

### The S-A boundary

This is the most consequential grading decision. S means "apply with energy and a tailored application." A means "apply if time permits." The difference is whether the role has a realistic path to being the candidate's best option, or whether it's merely a good option in a lineup of good options.

Ask: "If the candidate had three S-tier roles and this role, would they still apply to this one?" If yes, it's genuinely S. If they'd deprioritise it, it's A.

### The F threshold

F is not "bad." F is "do not waste time." The most common F reasons:
- Hard seniority mismatch (description explicitly requires 5+ years of professional experience, senior/staff-level scope)
- Non-engineering role disguised by title (Solutions Engineer that's really pre-sales, Developer Advocate that's really marketing)
- Domain dealbreaker (check `preferences.toml` for excluded sectors)
- Role type dealbreaker (check `preferences.toml` for excluded role types)
- Security clearance requirement that cannot be met (check `personal.md` and `military.md` for nationality and clearance eligibility)

---

## Evaluation Dimensions

### Critical dimensions

These are non-negotiable. A role that fails either critical dimension is an F regardless of how strong the other dimensions are.

#### Career ceiling

**What to assess:** Does this role's domain lead to high-income, high-impact positions at 10-15 years of experience? Read `preferences.toml` for the candidate's long-term targets, then assess whether this domain can deliver them.

**Why it's critical:** The first job sets the trajectory. A role in a high-ceiling domain creates compounding returns — each year of experience makes the next role easier to get and better compensated. A role in a low-ceiling domain caps growth early. At entry level, the domain matters more than the specific role because you're choosing a trajectory, not just a job.

**Good signal:**
- Systems engineering, infrastructure, platform engineering
- Trading systems, exchange infrastructure, market data
- AI/ML infrastructure, ML tooling, model serving
- Distributed systems, databases, compilers, runtime engineering
- Developer tools, open-source infrastructure
- The role description mentions scale, performance, reliability, or correctness as primary concerns

**Bad signal:**
- IT support, helpdesk, desktop support
- QA-only roles with no path to SDET or engineering
- Purely operational roles (monitoring dashboards, ticket triage) with no engineering component
- Narrow product work where the "engineering" is configuring a third-party tool
- Roles where the primary output is documentation, process, or coordination rather than code

**Edge cases:**
- "Data Engineer" can be high-ceiling (building data infrastructure at scale) or low-ceiling (writing SQL reports and maintaining Airflow DAGs). Read the description to determine which.
- "DevOps" can be high-ceiling (building internal platforms, infrastructure automation at scale) or low-ceiling (manual deployment, ticket-driven ops).
- "Full-stack" at a small company often means "you do everything including the CSS" which fragments focus. At a large company it can mean genuine breadth across meaningful systems.

#### Seniority match

**What to assess:** Can this candidate realistically get hired for this role? Read `experience.md` for formal work history, `projects.md` for demonstrated capability, and `education.md` for credentials. Assess whether the description's actual expectations match the candidate's level.

**Why it's critical:** A perfect role the candidate cannot get is worthless. Conversely, a role with a "Senior" title but expectations that match the candidate's actual level is genuinely valuable — many companies use title inflation. The assessment must be based on the description's requirements, not the title.

**Achievable signals:**
- "New grad", "graduate programme", "entry-level", "junior", "early career"
- No years of experience stated, with scope that matches project-level experience
- "0-2 years", "1+ years" (strong projects can substitute)
- Intern-to-hire conversion programmes
- "Strong academic background with relevant projects" or similar language valuing demonstrated ability over tenure

**Stretch but possible:**
- "2-3 years" with a strong portfolio narrative (projects demonstrate equivalent experience)
- "Mid-level" where the actual listed responsibilities align with what the profile shows
- Roles that list extensive requirements but also say "or equivalent demonstrated ability"

**Not achievable:**
- "5+ years of professional experience" as a hard requirement
- Staff/principal-level scope: owning org-wide architecture, mentoring multiple teams, setting technical direction for a department
- Expectations of production incident management, on-call leadership, or system ownership that presupposes years of operational experience
- "Led teams of X engineers" as a requirement

**The "Senior" trap:** Many companies, especially in the UK, use "Senior" for roles that are 2-3 years out of university. Read the description. If the actual expectations are "build features, participate in design reviews, own a component," that may be achievable despite the title. If the expectations are "drive architectural decisions across the org, mentor junior engineers, own system reliability," it is not.

---

### High-weight dimensions

These strongly influence the grade but cannot force an F on their own. A role that's weak on one high-weight dimension might still be an A; weak on two and it drops to B.

#### Skill breadth

**What to assess:** Will this role expose the candidate to multiple technical layers (backend, infrastructure, data, some frontend) or lock them into one narrow slice? Breadth early in a career creates more options later.

**Why it matters:** Read `experience.md` — at the candidate's current career stage, they may not yet know which specialisation will be the best long-term bet. A role that touches multiple infrastructure layers creates several possible specialisation paths. A role maintaining a single microservice creates one.

**Good signal:** "Full-stack infrastructure," multiple team rotations, cross-functional work, exposure to different parts of the stack, graduate rotation programmes.

**Bad signal:** "You will exclusively maintain this one microservice," single-tool focus, no mention of cross-team work, siloed within one narrow layer.

#### Company signal

**What to assess:** Does having this company on the CV open doors? Read `experience.md` — for a first/early job, brand recognition matters disproportionately.

**Why it matters:** The first job is the hardest to get and the most important for trajectory. A strong company name gets the CV past initial screening at other strong companies. This premium decreases for subsequent jobs as the work itself becomes the signal, but for early career it is significant.

**Strong signal:** Companies that engineers know and respect — tier-1 tech companies, well-funded scale-ups with strong engineering reputations, top quant firms, leading AI labs.

**Moderate signal:** Large companies with solid but not exceptional engineering brands. Smaller companies with strong domain reputations.

**Weak signal:** Unknown companies with no engineering reputation, body shops, agencies, outsourcing firms.

#### Technical depth

**What to assess:** Will the candidate work on genuinely hard problems? Performance-critical code, distributed systems, scale, algorithmic complexity? Read `projects.md` — the candidate's portfolio demonstrates what kind of technical depth they are capable of and interested in.

**Why it matters:** Technical depth builds the kind of experience that compounds. Working on systems-level problems teaches skills that transfer across companies and domains. Working on application-layer CRUD teaches framework conventions.

**Good signal:** Scale metrics, performance requirements, distributed systems, real-time processing, compiler/runtime work, ML training infrastructure, custom storage engines.

**Bad signal:** CRUD applications, config management as the primary work, glue code, no mention of scale or technical challenge.

#### Sponsorship viability

**What to assess:** Can and will this company sponsor the required visa when the candidate's current right-to-work expires? Read `visa.md` for the exact timeline.

**Why it matters:** The visa creates a hard deadline. After expiry, the candidate needs sponsored employment to remain in the UK. A brilliant role at a company that cannot sponsor is limited to the remaining visa window.

**Strong signal:**
- Large company with established sponsor licence
- "We sponsor visas" in the job description
- Established graduate programme
- Known track record of sponsoring international hires

**Moderate signal:**
- Mid-size company that probably has a sponsor licence but doesn't mention it
- Company in a sector that commonly sponsors

**Weak signal:**
- Small startup with no visible sponsor licence
- "Must have existing right to work in the UK"

**Important nuance:** Read `visa.md` — the candidate may currently have unrestricted right to work. If so, sponsorship is not needed now. A company that says "must have right to work" is not excluding the candidate today. When uncertain, do not penalise heavily, but note the risk.

---

### Medium-weight dimensions

These are tiebreakers and quality signals. They rarely change a grade by more than one step.

#### Domain transferability

**What to assess:** Are the skills built in this role useful across multiple companies and sectors, or are they specific to this company's proprietary systems?

**Good signal:** Infrastructure, platform engineering, database systems, cloud-native tooling, open-source technologies, standard protocols.

**Bad signal:** Proprietary internal DSL, hyper-specific product knowledge, vendor-locked tools with no external market.

#### Growth environment

**What to assess:** Will the candidate learn fast? Are there strong engineers to learn from? Is there a mentorship culture?

**Good signal:** Dedicated mentor, small team of strong engineers, open-source contributions, engineering blog, code review culture, engineering ladder with clear progression.

**Bad signal:** No mentorship mentioned, very large teams where a new hire would be invisible, no engineering culture signals.

#### Tech stack relevance

**What to assess:** Are the technologies used broadly valuable in the market, and do they overlap with the candidate's existing strengths? Read `skills.md` for current proficiency.

**Good signal:** Technologies with broad market demand that overlap with the candidate's primary skills (check `skills.md`).

**Moderate signal:** Technologies with broad market demand but no direct overlap.

**Bad signal:** Proprietary frameworks, legacy-only stacks with no modernisation path.

**Profile-specific bonus:** If the candidate's primary language (from `skills.md`) is in the production stack, that's a strong positive signal.

---

## Dimension Interaction Effects

Dimensions do not operate independently. Some combinations amplify; others cancel.

### Amplifying combinations

- **High career ceiling + strong company signal:** The best possible foundation. A systems role at a tier-1 company sets up the entire career trajectory. This combination justifies S even if other dimensions are moderate.
- **Technical depth + skill breadth:** Deep work across multiple layers is the ideal learning environment. This combination distinguishes SS from S.
- **Strong company signal + good growth environment:** Learning from excellent people at a company whose name opens doors. Each reinforces the other's value.

### Cancelling combinations

- **High career ceiling but no seniority match:** The role is perfect — for someone with more experience. Grade F regardless of how attractive the domain is.
- **Strong company signal but narrow scope:** Working at a great company but if the role is exclusively narrow operational work, the company brand is partially wasted.
- **Great tech stack but low career ceiling:** Building interesting things in a great language at a company going nowhere. The language experience transfers, but the domain doesn't. This pulls toward B.

### The sponsorship modifier

Sponsorship viability acts as a multiplier on the overall grade, not a standalone dimension. Read `visa.md` for the timeline, then apply:
- Clear sponsorship path: no modification.
- Uncertain sponsorship: drops the grade by half a step.
- Likely no sponsorship: drops the grade by a full step.
- Explicit "no sponsorship ever": if the role otherwise requires staying beyond the visa expiry, drop by two steps or to F.

For short-term contract roles or roles where the experience value within the visa window justifies the role regardless, sponsorship matters less.

---

## Worked Examples

**Note:** These examples demonstrate the reasoning framework. When grading real jobs, substitute the candidate's actual profile data from `profile/` files. References to specific projects, skills, and credentials below are illustrative — always verify against current profile files.

### Example 1: Graduate Software Engineer, Infrastructure @ Tier-1 Infrastructure Company

**Title parse:** "Graduate" = explicitly entry-level. "Infrastructure" = high-ceiling domain. Tier-1 company = top engineering brand.

**Description signals:** Building and maintaining edge network infrastructure. Systems handling millions of requests per second. Multiple systems languages. Team rotation in the first year. Dedicated mentor. Graduate cohort with structured onboarding.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | Infrastructure at internet scale. This domain leads to principal-level infrastructure roles. |
| Seniority match | Perfect | Explicitly a graduate programme. No years required. |
| Skill breadth | Excellent | Rotation across multiple infrastructure domains in year one. |
| Company signal | Excellent | Tier-1 engineering brand. |
| Technical depth | Excellent | Millions of requests/sec, performance-critical code, multiple low-level languages. |
| Sponsorship | Strong | Large company with established sponsor licence and graduate programme. |
| Domain transferability | Excellent | Infrastructure and distributed systems skills transfer everywhere. |
| Growth environment | Excellent | Dedicated mentor, cohort programme, strong engineering culture. |
| Tech stack | Assess against `skills.md` | Check whether the listed languages overlap with the candidate's primary skills. |

**Grade: SS.** Every dimension is strong. This is the archetype of what SS looks like.

---

### Example 2: Software Engineer, New Grad — Production Infrastructure @ Strong Brand

**Description signals:** Owning reliability and performance of production systems. Kubernetes, cloud infrastructure, monitoring and alerting. On-call rotation. Mentorship from senior engineers.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Strong | Production infrastructure is a high-ceiling domain, though more ops-flavoured than pure systems. |
| Seniority match | Perfect | Explicitly new grad. |
| Skill breadth | Good | Infrastructure + monitoring + reliability. |
| Company signal | Excellent | Top-tier brand for systems work. |
| Technical depth | Good | Production-scale systems, but more operational than pure infrastructure. |
| Sponsorship | Strong | Large company, established grad programme. |
| Tech stack | Assess against `skills.md` | Check overlap with candidate's primary languages. |

**Grade: S.** Strong on nearly every front. The more ops-flavoured work keeps it from SS.

---

### Example 3: Backend Software Engineer — Payments @ Tier-1 Fintech

**Description signals:** Payment processing systems. High availability, low latency. 2-4 years of experience preferred. Distributed systems, API design, database modelling.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | Payments infrastructure at scale. |
| Seniority match | Stretch | "2-4 years preferred" — not a hard requirement. Check `projects.md` for relevant domain work that could substitute for years of experience. |
| Company signal | Excellent | Tier-1 engineering brand. |
| Technical depth | Excellent | Low-latency, high-availability distributed systems at scale. |
| Sponsorship | Strong | Large company with global workforce. |
| Tech stack | Assess against `skills.md` | Check overlap. |

**Grade: A.** Would be S except for the seniority stretch. Worth applying with a tailored narrative connecting relevant portfolio projects to the domain.

---

### Example 4: ML Engineer — Model Serving Infrastructure

**Description signals:** Building and scaling model serving infrastructure. Low-latency inference, model optimisation. 1-3 years of experience. ML frameworks and model deployment.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | ML infrastructure is one of the highest-growth domains. |
| Seniority match | Good | "1-3 years" is a stretch. Check `projects.md` for ML-adjacent work (inference pipelines, ML framework contributions). |
| Technical depth | Excellent | Low-latency inference, model optimisation. Check `projects.md` for directly relevant experience. |
| Tech stack | Assess against `skills.md` | Check for overlap with ML tooling experience. |

**Grade: S.** Direct domain overlap with relevant portfolio work (if present) is a differentiator.

---

### Example 5: Quantitative Developer — Low-Latency Trading @ Top Quant Firm

**Description signals:** Trading systems. Sub-microsecond latency requirements. Mathematics background. 0-2 years, graduate-level entry.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Exceptional | Quant development at a top firm. Among the highest-compensated roles in technology. |
| Seniority match | Good | Explicitly graduate-level entry. |
| Technical depth | Exceptional | Sub-microsecond latency. Among the most demanding performance engineering in the world. |
| Company signal | Excellent | Top-tier name in quantitative finance. |
| Domain transferability | Moderate | Core skills transfer, domain knowledge is more specialised. Check `projects.md` for trading systems work. |

**Grade: SS.** Graduate-level entry at a top quant firm. If the candidate's portfolio includes matching engine or trading systems work, the alignment is near-perfect.

---

### Example 6: Site Reliability Engineer @ Well-Known Tech Company

**Description signals:** Streaming infrastructure reliability. Kubernetes, cloud, Terraform, monitoring. On-call rotation. Incident management. 1-3 years preferred.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Good | SRE at scale is solid but trends toward operational work. |
| Seniority match | Stretch | "1-3 years preferred." SRE roles also expect operational maturity the candidate may lack (check `experience.md`). |
| Technical depth | Moderate | Reliability engineering is important but the description emphasises operational tooling over building systems. |
| Tech stack | Assess against `skills.md` | Heavy on managed cloud services. Check alignment. |

**Grade: B.** The combination of seniority stretch, operational focus, and managed-cloud-heavy stack makes it a backup option.

---

### Example 7: Compiler Engineer — LLVM @ Major Hardware Company

**Description signals:** LLVM backends. Code generation, optimisation passes. PhD or equivalent research experience preferred.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | Compiler engineering is one of the highest-ceiling specialisations. |
| Seniority match | Stretch-to-difficult | "PhD or equivalent research experience preferred" is a high bar. Check `projects.md` for compiler-related work. |
| Technical depth | Exceptional | LLVM internals, code generation, optimisation passes. |
| Skill breadth | Narrow | Compiler engineering is deep but narrow. |

**Grade: A.** Exceptional career ceiling and technical depth, but the seniority match is a real concern. Worth a focused application highlighting any compiler-adjacent work from the portfolio.

---

### Example 8: Platform Engineer — Internal Developer Tools @ Consumer Fintech

**Description signals:** Internal platforms for developer productivity. CI/CD, deployment tooling, service mesh, observability. Entry-level considered.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Good | Platform engineering is a strong career path. Internal tooling is slightly lower ceiling than external infrastructure. |
| Seniority match | Good | Explicitly accessible to entry-level. |
| Technical depth | Moderate | Internal tooling is useful but often less technically demanding than product infrastructure. |
| Company signal | Good | Well-known in UK tech. Not tier-1 globally. |
| Tech stack | Assess against `skills.md` and `portfolio-gaps.md` | May address known portfolio gaps in CI/CD and containerisation. |

**Grade: A.** Solid role with explicit entry-level accessibility. Interesting side benefit: may directly address known portfolio gaps.

---

## Boundary Cases

These are jobs that look like one grade but are actually another. They test whether the grading is based on surface signals or genuine analysis.

### Boundary 1: Looks like A, actually S

**Role:** "Software Engineer — Data Infrastructure" at a company you've barely heard of.

**Why it looks like A:** Unknown company, generic title, no obvious prestige.

**Why it's actually S:** The description reveals: building a distributed query engine in the candidate's primary language (check `skills.md`), sub-second query times over petabyte-scale data, the team is 8 engineers from top firms, well-funded, and they explicitly sponsor visas. Company signal is one dimension; it should not override six strong dimensions.

**Lesson:** Do not let company name recognition dominate the grade. Read the description.

### Boundary 2: Looks like S, actually B

**Role:** "Graduate Software Engineer" at a tier-1 company.

**Why it looks like S:** Tier-1 brand. Graduate. Software Engineer.

**Why it's actually B:** The description reveals the team is internal IT systems — building ticketing tools and device management dashboards. The work is CRUD applications, the team is 50+ people, and there's no mention of scale or technical challenge. The brand is valuable but the actual work has a low career ceiling and minimal technical depth.

**Lesson:** Company brand does not automatically make a role good. A low-ceiling role at a great company is still a low-ceiling role.

### Boundary 3: Looks like C, actually S

**Role:** "Junior Developer" at a 15-person company with a plain website and unclear product.

**Why it looks like C:** Unknown company, generic title, small team.

**Why it's actually S:** The company is building a custom database engine in the candidate's primary language (check `skills.md`). The team includes engineers from top firms. The tech is deep (custom storage engine, lock-free data structures, SIMD-optimised queries), and they sponsor visas. Small team means direct mentorship from senior engineers.

**Lesson:** Small company + plain title can hide extraordinary opportunities. When the company is unknown, the description carries all the weight.

### Boundary 4: Looks like B, actually F

**Role:** "Software Engineer" at a well-funded startup. Decent salary, modern stack, 100-person company.

**Why it looks like B:** Funded startup, modern stack, reasonable size.

**Why it's actually F:** The description reveals the role is "Solutions Engineer" relabelled — 60% customer calls, integration support, custom API adapters. This is customer-facing support engineering disguised by the title — check `preferences.toml` for whether this is a hard exclusion.

**Lesson:** Titles lie. The description's actual day-to-day activities determine the grade.

### Boundary 5: Looks like F, actually A

**Role:** "Senior Backend Engineer" at a strong fintech. The "Senior" title looks like an immediate seniority mismatch.

**Why it looks like F:** "Senior" in the title.

**Why it's actually A:** The description lists no years of experience requirement. The "Senior" title maps to a level system where actual expectations are mid-level at most companies. Check `projects.md` for relevant domain experience that supports the application. The company is a strong brand, sponsors visas, and the domain has excellent career ceiling.

**Lesson:** "Senior" does not automatically mean unachievable. Always read the actual requirements. Many UK companies use "Senior" for their second engineering level.

---

## Common Misjudgments

These are patterns of grading error to actively avoid.

**Over-weighting tech stack match.** The candidate's primary language in the listing is a positive signal, but a role using a different language in an excellent domain with great career ceiling is worth more than a primary-language role in a dead-end domain. Tech stacks are learned in months; domains take years to build expertise in.

**Under-weighting career ceiling for entry-level roles.** At 10-15 years of experience, the domain matters more than anything else. A QA automation role at a great company has a lower career ceiling than a systems engineering role at a well-funded startup. The domain trajectory is the most important factor at entry level because you're choosing a path, not just a job.

**Assuming "no sponsorship mention" means "won't sponsor."** Many companies sponsor but don't mention it in job descriptions. Large companies with international workforces almost always sponsor. Only penalise sponsorship when there are active negative signals at a small company with no visible sponsor licence.

**Grade inflation from enthusiasm.** A role that hits personal interest buttons (check `interests.md`) is exciting but must still be graded on the full rubric. Enthusiasm is a signal that the application will be strong — it doesn't change the grade. An unachievable dream role is still F.

**Grade deflation from unfamiliarity.** A role in a domain you haven't encountered before deserves research, not dismissal. If you don't know the ceiling of a domain, look it up before grading. Don't default to B because the domain is unfamiliar.
