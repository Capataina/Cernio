# Job Grading Rubric

> The complete evaluation framework for grading jobs. Every job is assessed across these dimensions, with the full reasoning chain visible. Applied consistently to every role across every company.

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
| **S** | Strong application candidate | One dimension is merely good rather than excellent. Perhaps the company is well-regarded but not tier-1 (Monzo vs Google), or the scope is slightly narrower than ideal, or the tech stack is adjacent rather than core. The role is still clearly worth pursuing and would be a strong career move. |
| **A** | Worth applying to | Two dimensions are notably weaker. The role is good but has clear gaps — maybe a strong company but the role itself is narrow, or technically deep work at a company with limited brand recognition. Still a net positive career move, but not the kind of opportunity that demands dropping everything. |
| **B** | Backup / worth watching | The role is acceptable but uninspiring on several fronts. Decent work, learning, or brand, but not enough of any to make it a priority. Apply if the pipeline is thin. These roles serve as practice applications and fallbacks. |
| **C** | Only if nothing better exists | The role is technically achievable but offers limited career value. Narrow scope, weak company signal, poor growth environment, or domain with low transferability. A job, not a career move. The main risk is opportunity cost — time spent here is time not spent building toward the long-term target. |
| **F** | Do not apply | A dealbreaker is present. The seniority is unachievable (hard 5+ years, staff-level scope), the domain is a career dead-end, the role is non-engineering, or a hard constraint from preferences.toml is violated. No amount of strength in other dimensions compensates. |

### The SS threshold

SS is not "very good." SS means: if you were offered this role tomorrow, you would accept without hesitation and feel confident it was the right move for the next 2-3 years of your career. The bar is deliberately high because SS roles get the most attention and the most detailed assessment. Inflating SS dilutes the signal the user relies on.

### The S-A boundary

This is the most consequential grading decision. S means "apply with energy and a tailored application." A means "apply if time permits." The difference is whether the role has a realistic path to being the user's best option, or whether it's merely a good option in a lineup of good options.

Ask: "If I had three S-tier roles and this role, would I still apply to this one?" If yes, it's genuinely S. If you'd deprioritise it, it's A.

### The F threshold

F is not "bad." F is "do not waste time." The most common F reasons:
- Hard seniority mismatch (description explicitly requires 5+ years of professional experience, senior/staff-level scope and expectations)
- Non-engineering role disguised by title (Solutions Engineer that's really pre-sales, Developer Advocate that's really marketing)
- Domain dealbreaker (gambling, adtech, consumer crypto)
- Role type dealbreaker (consulting, customer-facing, management, support engineering)
- Security clearance requirement that cannot be met (SC/DV for Turkish national)

---

## Evaluation Dimensions

### Critical dimensions

These are non-negotiable. A role that fails either critical dimension is an F regardless of how strong the other dimensions are.

#### Career ceiling

**What to assess:** Does this role's domain lead to high-income, high-impact positions at 10-15 years of experience? What do senior/staff/principal engineers in this domain earn? What kinds of companies hire for these skills?

**Why it's critical:** The first job sets the trajectory. A role in a high-ceiling domain (systems engineering, AI/ML infrastructure, trading systems, platform engineering, distributed systems) creates compounding returns — each year of experience makes the next role easier to get and better compensated. A role in a low-ceiling domain (IT support, narrow QA, operational work with no engineering progression) caps growth early. At entry level, the domain matters more than the specific role because you're choosing a trajectory, not just a job.

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
- "DevOps" can be high-ceiling (building internal platforms, infrastructure automation at scale) or low-ceiling (manual deployment, ticket-driven ops). The description tells you which.
- "Full-stack" at a small company often means "you do everything including the CSS" which fragments focus. At a large company it can mean genuine breadth across meaningful systems.

#### Seniority match

**What to assess:** Can this candidate realistically get hired for this role? Not "is the title entry-level" but "does the description's actual expectations match someone with a strong project portfolio but no formal work experience?"

**Why it's critical:** A perfect role the candidate cannot get is worthless. Conversely, a role with a "Senior" title but expectations that a strong new grad can meet is genuinely valuable — many companies use title inflation. The assessment must be based on the description's requirements, not the title.

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

**Why it matters:** At year 0-3, the candidate doesn't yet know which specialisation will be the best long-term bet. A role that touches backend services, infrastructure automation, data pipelines, and observability creates three possible specialisation paths at year 3. A role maintaining a single microservice creates one.

**Good signal:** "Full-stack infrastructure," multiple team rotations, cross-functional work, exposure to different parts of the stack, "you'll work across our platform," graduate rotation programmes.

**Bad signal:** "You will exclusively maintain this one microservice," single-tool focus (just Terraform, just Ansible), no mention of cross-team work, siloed within one narrow layer.

#### Company signal

**What to assess:** Does having this company on the CV open doors? For a first job, brand recognition matters disproportionately.

**Why it matters:** The first job is the hardest to get and the most important for trajectory. "Software Engineer at Cloudflare" opens doors that "Software Engineer at RandomStartup Ltd" cannot. This premium decreases for second and third jobs as the work itself becomes the signal, but for job #1 it is significant. A strong company name gets the CV past initial screening at other strong companies.

**Strong signal:** Companies that engineers know and respect. Cloudflare, Stripe, Palantir, Jane Street, Google, Apple, Bloomberg, Spotify, Monzo, Revolut, Wise, Arm, DeepMind, Anthropic, Databricks. Well-funded scale-ups with strong engineering reputations (e.g. Vercel, Supabase, Grafana Labs).

**Moderate signal:** Large companies with solid but not exceptional engineering brands. Smaller companies with strong domain reputations (e.g. a well-known quant firm that's not a household name). Companies where the engineering team has visible output (tech blog, open-source contributions, conference talks).

**Weak signal:** Unknown companies with no engineering reputation, body shops, agencies, outsourcing firms, companies where the engineering is cost-centre work.

#### Technical depth

**What to assess:** Will the candidate work on genuinely hard problems? Performance-critical code, distributed systems, scale, algorithmic complexity?

**Why it matters:** Technical depth builds the kind of experience that compounds. Working on low-latency systems teaches you to think about performance. Working on distributed systems teaches you to think about failure modes. Working on ML infrastructure teaches you to think about data pipelines and model serving. These skills transfer across companies and domains. Working on CRUD apps teaches you framework conventions.

**Good signal:** "Millions of transactions," "hundreds of services," "low-latency," "performance-critical," distributed systems, data infrastructure at scale, real-time processing, "sub-millisecond," compiler/runtime work, ML training infrastructure, custom storage engines.

**Bad signal:** CRUD applications, config management as the primary work, glue code between third-party services, no mention of scale or technical challenge, "you'll build landing pages and marketing tools."

#### Sponsorship viability

**What to assess:** Can and will this company sponsor a Skilled Worker visa when needed (from August 2027)?

**Why it matters:** The Graduate visa expires August 2027. At that point, the candidate needs a Skilled Worker visa to remain in the UK. A brilliant role at a company that cannot or will not sponsor is a 1-year job with a hard expiry. This dimension is a soft deadline — it doesn't need sponsorship today, but the company must be capable and willing when the time comes.

**Strong signal:**
- Large company with an established sponsor licence (check the UK government register)
- "We sponsor visas" in the job description
- Established graduate programme (companies that run grad programmes almost always sponsor)
- Known track record of sponsoring international hires

**Moderate signal:**
- Mid-size company that probably has a sponsor licence but doesn't mention it
- Company in a sector that commonly sponsors (finance, tech)

**Weak signal:**
- Small startup with no visible sponsor licence
- "Must have existing right to work in the UK" (though this may only apply to the immediate hire, not future sponsorship)
- Company in a sector that rarely sponsors

**Important nuance:** The candidate currently has unrestricted right to work (Graduate visa until August 2027). Sponsorship is not needed now. A company that says "must have right to work" is not excluding the candidate today — it might be signalling unwillingness to sponsor later, or it might just be standard legal language. When uncertain, do not penalise heavily, but note the risk.

---

### Medium-weight dimensions

These are tiebreakers and quality signals. They rarely change a grade by more than one step, but they separate an A from a high B or a low S from a high A.

#### Domain transferability

**What to assess:** Are the skills built in this role useful across multiple companies and sectors, or are they specific to this company's proprietary systems?

**Why it matters:** Transferable skills (infrastructure, distributed systems, database internals, ML pipelines) compound across your career. Company-specific skills (proprietary internal DSLs, custom frameworks nobody else uses) have value only at the current employer.

**Good signal:** Infrastructure, platform engineering, database systems, cloud-native tooling, open-source technologies, standard protocols.

**Bad signal:** Proprietary internal DSL, hyper-specific product knowledge, vendor-locked tools with no external market.

#### Growth environment

**What to assess:** Will the candidate learn fast? Are there strong engineers to learn from? Is there a mentorship culture, code review discipline, and investment in engineering quality?

**Why it matters:** At entry level, learning velocity is everything. The difference between working alongside senior engineers who review your code thoughtfully and working in a large team where you're invisible is enormous in the first two years. The quality of the people around you determines how fast you grow.

**Good signal:** Dedicated mentor, small team of strong engineers, open-source contributions, engineering blog, "we pair program," "weekly tech talks," engineering ladder with clear progression.

**Bad signal:** No mentorship mentioned, very large teams where a new grad would be invisible, no engineering culture signals, no evidence of investment in engineering quality.

#### Tech stack relevance

**What to assess:** Are the technologies used broadly valuable in the market, and do they overlap with the candidate's existing strengths?

**Why it matters:** Learning Rust, Go, Python, Kubernetes, or PostgreSQL in production has market value everywhere. Learning a proprietary framework has market value nowhere else. Additionally, stack overlap with existing skills means a shorter ramp-up and a stronger application.

**Good signal:** Rust, Go, Java, Python, C++, Kubernetes, Kafka, PostgreSQL, distributed systems tooling, modern observability stacks.

**Moderate signal:** TypeScript/Node.js backend (broadly valuable but less aligned with profile), .NET (large market but different ecosystem).

**Bad signal:** Proprietary framework nobody else uses, legacy-only stack (COBOL, ancient Java with no modernisation path), no modern tooling.

**Profile-specific bonus:** Rust in production is a strong positive signal — it's the candidate's primary language and a differentiator. C++ in performance-critical contexts is also well-aligned.

---

## Dimension Interaction Effects

Dimensions do not operate independently. Some combinations amplify; others cancel.

### Amplifying combinations

- **High career ceiling + strong company signal:** The best possible foundation. A systems role at a tier-1 company sets up the entire career trajectory. This combination justifies S even if other dimensions are moderate.
- **Technical depth + skill breadth:** Deep work across multiple layers is the ideal learning environment. This combination distinguishes SS from S.
- **Strong company signal + good growth environment:** Learning from excellent people at a company whose name opens doors. Each reinforces the other's value.

### Cancelling combinations

- **High career ceiling but no seniority match:** The role is perfect — for someone with 5 years of experience. Grade F regardless of how attractive the domain is.
- **Strong company signal but narrow scope:** Working at Google is valuable, but if the role is exclusively writing Terraform configs in a single team, the company brand is partially wasted. The compound value is lower than either dimension alone suggests.
- **Great tech stack but low career ceiling:** Building interesting things in Rust at a company going nowhere. The Rust experience transfers, but the domain doesn't. This pulls toward B — good for learning, limited for career trajectory.

### The sponsorship modifier

Sponsorship viability acts as a multiplier on the overall grade, not a standalone dimension:
- Clear sponsorship path: no modification.
- Uncertain sponsorship: drops the grade by half a step (e.g. S becomes low S or high A).
- Likely no sponsorship: drops the grade by a full step (S becomes A, A becomes B).
- Explicit "no sponsorship ever": if the role otherwise requires staying beyond August 2027, drop by two steps or to F if the role is clearly long-term.

For short-term contract roles or roles where 1 year of experience would be valuable regardless of staying, sponsorship matters less.

---

## Worked Examples

### Example 1: Graduate Software Engineer, Infrastructure @ Cloudflare

**Title parse:** "Graduate" = explicitly entry-level. "Infrastructure" = high-ceiling domain. "Cloudflare" = tier-1 engineering brand.

**Description signals:** Building and maintaining the edge network infrastructure. Working on systems that handle millions of requests per second. Languages: Go, Rust, C. Team rotation in the first year across edge compute, DNS, and DDoS mitigation. Dedicated mentor. Graduate cohort with structured onboarding.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | Infrastructure at internet scale. This domain leads to principal-level infrastructure roles at any major tech company. |
| Seniority match | Perfect | Explicitly a graduate programme. No years required. |
| Skill breadth | Excellent | Rotation across edge, DNS, and DDoS. Three distinct infrastructure domains in year one. |
| Company signal | Excellent | Cloudflare is a top-tier engineering brand. Every systems role will recognise this name. |
| Technical depth | Excellent | Millions of requests/sec, performance-critical code, multiple low-level languages. |
| Sponsorship | Strong | Large company with established sponsor licence, graduate programme implies sponsorship infrastructure. |
| Domain transferability | Excellent | Infrastructure, networking, and distributed systems skills transfer everywhere. |
| Growth environment | Excellent | Dedicated mentor, cohort programme, known for strong engineering culture. |
| Tech stack | Strong | Go, Rust, C — all high-value. Rust is the primary profile language. |

**Grade: SS.** Every dimension is strong. This is the archetype of what SS looks like.

---

### Example 2: Software Engineer, New Grad — Production Infrastructure @ Palantir

**Title parse:** "New Grad" = entry-level. "Production Infrastructure" = systems work. "Palantir" = strong brand, known for hard problems.

**Description signals:** Owning reliability and performance of production systems. Kubernetes, cloud infrastructure, monitoring and alerting. On-call rotation. Languages: Java, Go, Python. Mentorship from senior engineers.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Strong | Production infrastructure is a high-ceiling domain, though more ops-flavoured than pure systems. SRE/platform career path is strong. |
| Seniority match | Perfect | Explicitly new grad. |
| Skill breadth | Good | Infrastructure + monitoring + reliability. Less breadth than a role touching application code too, but solid. |
| Company signal | Excellent | Palantir is a top-tier brand for systems work. |
| Technical depth | Good | Production-scale systems, but more operational than the pure infrastructure role. Kubernetes and monitoring are deep but different. |
| Sponsorship | Strong | Large company, established grad programme. |
| Domain transferability | Strong | SRE and production infrastructure skills transfer broadly. |
| Growth environment | Strong | Palantir mentorship is well-regarded. |
| Tech stack | Moderate | Java, Go, Python. No Rust, but all are broadly valuable. |

**Grade: S.** Strong on nearly every front. The more ops-flavoured nature of the work and the lack of Rust keep it from SS, but this is clearly worth a strong application.

---

### Example 3: Backend Software Engineer — Payments @ Stripe

**Title parse:** "Backend" = aligned. "Payments" = fintech infrastructure. "Stripe" = tier-1 company.

**Description signals:** Designing and building payment processing systems. High availability, low latency. Ruby, Java, Go. 2-4 years of experience preferred. Distributed systems, API design, database modelling.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | Payments infrastructure at scale. This domain leads to principal engineer roles at any fintech or large tech company. |
| Seniority match | Stretch | "2-4 years preferred" is not a hard requirement, and "preferred" language suggests flexibility. The profile's project depth (Nyquestro, Aurix) tells a compelling payments/fintech story. Achievable but needs a strong application. |
| Skill breadth | Good | Distributed systems, API design, database work. Backend-focused rather than full-stack, but within backend the scope is broad. |
| Company signal | Excellent | Stripe is a top-tier engineering brand globally. |
| Technical depth | Excellent | Low-latency, high-availability, distributed systems at massive scale. |
| Sponsorship | Strong | Large company with global workforce and established sponsorship. |
| Domain transferability | Excellent | Payments and fintech infrastructure skills are in high demand everywhere. |
| Growth environment | Excellent | Stripe's engineering culture is well-documented and highly regarded. |
| Tech stack | Moderate | Ruby is less aligned (profile has no Ruby), but Java and Go are broadly valuable. |

**Grade: A.** Would be S except for the seniority stretch. "2-4 years preferred" is not a hard wall, but it will require a particularly strong application that positions the project portfolio as equivalent experience. Worth applying with a tailored narrative connecting Nyquestro and Aurix directly to payments infrastructure.

---

### Example 4: ML Engineer — Model Serving Infrastructure @ Databricks

**Title parse:** "ML Engineer" = domain-aligned. "Model Serving Infrastructure" = infrastructure, not pure ML. "Databricks" = strong company in the space.

**Description signals:** Building and scaling model serving infrastructure. Low-latency inference, model optimisation, ONNX Runtime, TensorRT. Python, C++. 1-3 years of experience. Familiarity with ML frameworks and model deployment.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | ML infrastructure is one of the highest-growth domains. Model serving is specifically high-value — it's the intersection of systems and ML. |
| Seniority match | Good | "1-3 years" is a stretch but the profile has direct ONNX Runtime experience (Image Browser, tinygrad contribution). Compelling narrative. |
| Skill breadth | Moderate | ML infrastructure is deep but narrow. Less breadth than a general infrastructure role. |
| Company signal | Strong | Databricks is well-known in the ML/data space. Strong but not quite universal brand recognition. |
| Technical depth | Excellent | Low-latency inference, model optimisation, ONNX Runtime. Directly builds on profile strengths. |
| Sponsorship | Strong | Large company, global presence. |
| Domain transferability | Strong | Model serving skills are in demand at every company doing ML. |
| Growth environment | Strong | Databricks invests heavily in engineering. |
| Tech stack | Strong | Python, C++ are in the profile. ONNX Runtime is a direct hit. |

**Grade: S.** The ONNX Runtime overlap with the profile is a differentiator. The seniority stretch is real but the specific domain experience (tinygrad LSTM operator, Image Browser CLIP pipeline) provides a compelling story that most 1-year professionals wouldn't have.

---

### Example 5: Quantitative Developer — Low-Latency Trading @ XTX Markets

**Title parse:** "Quantitative Developer" = quant + engineering. "Low-Latency Trading" = HFT. "XTX Markets" = top quant firm.

**Description signals:** Developing and optimising trading systems. Sub-microsecond latency requirements. C++, Python. Strong mathematics background. Working closely with researchers. 0-2 years, graduate-level entry.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Exceptional | Quant development at a top firm. This domain leads to some of the highest-compensated roles in technology. Career ceiling is well above the 500K target. |
| Seniority match | Good | "0-2 years, graduate-level entry." Explicitly accessible. |
| Skill breadth | Moderate | Deep but narrow — trading systems are a specialised domain. However, the skills (low-latency, performance engineering, C++) transfer to any performance-critical systems role. |
| Company signal | Excellent | XTX is a top-tier name in quantitative finance. Universally respected in systems and quant circles. |
| Technical depth | Exceptional | Sub-microsecond latency. This is among the most demanding performance engineering in the world. |
| Sponsorship | Strong | Large, established firm with international hiring. |
| Domain transferability | Moderate | Core skills (low-latency, C++, performance) transfer. Domain knowledge (market microstructure) is more specialised but Nyquestro demonstrates exactly this. |
| Growth environment | Strong | Working with world-class researchers and engineers. Small, elite team. |
| Tech stack | Good | C++, Python. C++ is in the profile. No Rust, but the low-latency principles from Nyquestro directly apply. |

**Grade: SS.** Graduate-level entry at a top quant firm, directly aligned with Nyquestro's domain (order matching, market microstructure, low-latency engineering). The profile tells an exceptionally compelling story for this role.

---

### Example 6: Site Reliability Engineer @ Spotify

**Title parse:** "SRE" = infrastructure-adjacent. "Spotify" = strong brand.

**Description signals:** Ensuring reliability of streaming infrastructure. Kubernetes, GCP, Terraform, monitoring. On-call rotation. Java, Python. Incident management. 1-3 years preferred.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Good | SRE at scale is a solid career path, but it trends toward operational work over time. The ceiling is lower than pure systems or infrastructure engineering unless the role evolves into platform engineering. |
| Seniority match | Stretch | "1-3 years preferred" is a stretch. SRE roles also expect operational maturity (incident management, on-call experience) that the profile lacks. |
| Skill breadth | Good | Kubernetes, cloud, monitoring, incident management. Broad within the ops domain. |
| Company signal | Strong | Spotify is a well-known engineering brand. |
| Technical depth | Moderate | Reliability engineering is important work but the description emphasises operational tooling (Terraform, monitoring) over building systems. |
| Sponsorship | Strong | Large company, known for international hiring. |
| Domain transferability | Good | SRE skills transfer across companies. Kubernetes and cloud skills are broadly valuable. |
| Growth environment | Good | Spotify has strong engineering culture. |
| Tech stack | Moderate | Java, Python, Kubernetes, GCP. No Rust or C++, heavy on managed cloud services. |

**Grade: B.** The SRE role at Spotify is reputable but the combination of seniority stretch, operational rather than systems focus, and managed-cloud-heavy stack makes it a backup option. Worth applying if the pipeline is thin, but not a priority.

---

### Example 7: Compiler Engineer — LLVM @ Arm

**Title parse:** "Compiler Engineer" = deep systems. "LLVM" = industry-standard compiler infrastructure. "Arm" = major processor company.

**Description signals:** Working on LLVM backends for Arm architecture. Code generation, optimisation passes, machine instruction scheduling. C++. PhD or equivalent research experience preferred, or strong MSc/BSc with demonstrated compiler knowledge. Contributing to open-source LLVM.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Excellent | Compiler engineering is one of the highest-ceiling specialisations in software. Compiler engineers are rare and extremely well-compensated. |
| Seniority match | Stretch-to-difficult | "PhD or equivalent research experience preferred" is a high bar. Xyntra (kernel fusion compiler) shows some compiler familiarity, but this is a stretch. |
| Skill breadth | Narrow | Compiler engineering is deep but narrow. The skills transfer to a small set of roles (compilers, runtime engineering, language design). |
| Company signal | Strong | Arm is a major name in hardware and systems engineering. |
| Technical depth | Exceptional | LLVM internals, code generation, optimisation passes. Among the deepest technical work available. |
| Sponsorship | Strong | Large, established company with global presence. |
| Domain transferability | Moderate | Compiler skills transfer to a small but extremely high-value set of roles. |
| Growth environment | Strong | Working with LLVM experts, contributing to open source. |
| Tech stack | Moderate | C++ is in the profile. The domain (compiler internals) is adjacent to Xyntra's scope. |

**Grade: A.** The career ceiling and technical depth are exceptional, but the seniority match is a real concern — "PhD preferred" often means they genuinely expect research-level compiler experience. Worth a focused application that highlights Xyntra's compiler work and tinygrad's ONNX operator contribution, but should be realistic about the odds.

---

### Example 8: Platform Engineer — Internal Developer Tools @ Monzo

**Title parse:** "Platform Engineer" = infrastructure. "Internal Developer Tools" = tooling. "Monzo" = well-known fintech.

**Description signals:** Building internal platforms for developer productivity. CI/CD pipelines, deployment tooling, service mesh, observability. Go, Kubernetes, AWS. Entry-level considered, strong graduates welcome.

**Dimension assessment:**
| Dimension | Rating | Reasoning |
|-----------|--------|-----------|
| Career ceiling | Good | Platform engineering is a strong career path. Internal developer tooling is slightly lower ceiling than external infrastructure (less brand-visible output) but the skills are identical. |
| Seniority match | Good | "Entry-level considered, strong graduates welcome." Explicitly accessible. |
| Skill breadth | Good | CI/CD, deployment, service mesh, observability. Decent breadth within the platform domain. |
| Company signal | Good | Monzo is well-known in UK tech and fintech. Not tier-1 globally but strong in the UK market. |
| Technical depth | Moderate | Internal tooling is useful work but often less technically demanding than product infrastructure. Service mesh and observability can be deep, but CI/CD pipeline work is more operational. |
| Sponsorship | Moderate | Monzo sponsors but has had hiring fluctuations. Less certain than larger firms. |
| Domain transferability | Good | Platform engineering and CI/CD skills transfer well. Go and Kubernetes are broadly valuable. |
| Growth environment | Good | Monzo has a visible engineering culture (blog, open-source). |
| Tech stack | Moderate | Go, Kubernetes, AWS. No Rust overlap, but Go is broadly valuable. Addresses the CI/CD portfolio gap. |

**Grade: A.** Solid role at a good company with explicit entry-level accessibility. Falls short of S because the work is more operational than systems-deep, and the company signal is UK-strong but not globally tier-1. Interesting side benefit: directly addresses the CI/CD and containerisation portfolio gaps.

---

## Boundary Cases

These are jobs that look like one grade but are actually another. They test whether the grading is based on surface signals or genuine analysis.

### Boundary 1: Looks like A, actually S

**Role:** "Software Engineer — Data Infrastructure" at a company you've barely heard of.

**Why it looks like A:** Unknown company, generic title, no obvious prestige.

**Why it's actually S:** The description reveals: building a distributed query engine in Rust, sub-second query times over petabyte-scale data, the team is 8 engineers (3 from Google, 2 from Databricks), Series B funded by Sequoia, and they explicitly sponsor visas. The company signal is weak but every other dimension is excellent, and the small team of ex-FAANG engineers means the growth environment is exceptional. Company signal is one dimension; it should not override six strong dimensions.

**Lesson:** Do not let company name recognition dominate the grade. Read the description. The strength of the team, the technical depth, and the stack alignment can more than compensate for limited brand recognition.

### Boundary 2: Looks like S, actually B

**Role:** "Graduate Software Engineer" at Google.

**Why it looks like S:** Google. Graduate. Software Engineer. Tier-1 brand, entry-level, engineering role.

**Why it's actually B:** The description reveals the team is Google's internal IT systems — building ticketing tools and device management dashboards. The work is CRUD applications in Java, the team is 50+ people, and there's no mention of scale, performance, or technical challenge. The Google name is valuable, but the actual work has a low career ceiling and minimal technical depth. After two years building internal IT tools, the CV says "Google" but the skills say "CRUD."

**Lesson:** Company brand does not automatically make a role good. The brand is one dimension. A low-ceiling role at Google is still a low-ceiling role — the brand helps with the next job search but the missing skill development hurts more than the brand helps.

### Boundary 3: Looks like C, actually S

**Role:** "Junior Developer" at a 15-person company with a plain website, no engineering blog, and a product you have to read about twice to understand.

**Why it looks like C:** Unknown company, generic title, small team, unclear product.

**Why it's actually S:** The company is building a custom database engine for time-series financial data in Rust. The "Junior Developer" title is because they don't do title inflation. The 15-person company has 10 engineers, 6 of whom previously worked at Bloomberg, Citadel, or Two Sigma. The tech is deep (custom storage engine, lock-free data structures, SIMD-optimised queries), the domain ceiling is exceptional (database engineering), and they explicitly sponsor visas. The small team means direct mentorship from senior engineers who built production systems at major firms.

**Lesson:** Small company + plain title can hide extraordinary opportunities. When the company is unknown, the description carries all the weight. Read it thoroughly and look for signals of technical depth, team quality, and domain value.

### Boundary 4: Looks like B, actually F

**Role:** "Software Engineer" at a well-funded Series C startup in London. Decent salary, modern stack (TypeScript, React, Node.js, AWS), 100-person company.

**Why it looks like B:** Funded startup, modern stack, reasonable size, London. A perfectly acceptable fallback.

**Why it's actually F:** The description reveals the role is "Solutions Engineer" relabelled — 60% of the time is spent on customer calls, integration support, and writing custom API adapters for specific client requests. The engineering work is glue code connecting the company's API to each customer's systems. This is customer-facing support engineering disguised by the "Software Engineer" title, which is a hard exclusion in preferences.toml.

**Lesson:** Titles lie. "Software Engineer" at a startup can mean anything from building distributed systems to doing customer support calls in a code editor. The description's actual day-to-day activities determine the grade, not the title.

### Boundary 5: Looks like F, actually A

**Role:** "Senior Backend Engineer" at Wise. The "Senior" title looks like an immediate seniority mismatch.

**Why it looks like F:** "Senior" in the title. No entry-level signal.

**Why it's actually A:** The description lists requirements as "Strong programming skills in one or more of: Java, Kotlin, Go" and "Experience building and operating distributed systems." No years of experience mentioned. The "Senior" title at Wise maps to their level system where the actual expectations are mid-level at most companies. The profile has distributed systems experience (Nyquestro) and fintech domain knowledge (Aurix). Wise is a strong fintech brand, sponsors visas, and the domain (international payments infrastructure) has an excellent career ceiling. This is a reach application, but the fit on domain and technical alignment is strong enough to warrant it.

**Lesson:** "Senior" does not automatically mean unachievable. Always read the actual requirements. Many UK companies use "Senior" for their second engineering level, which corresponds to 1-3 years at other companies.

---

## Common Misjudgments

These are patterns of grading error to actively avoid.

**Over-weighting tech stack match.** Rust in the listing is a positive signal, but a role using Java in an excellent domain with great career ceiling is worth more than a Rust role in a dead-end domain. Tech stacks are learned in months; domains take years to build expertise in.

**Under-weighting career ceiling for entry-level roles.** At 10-15 years of experience, the domain matters more than anything else. A QA automation role in Python at Google has a lower career ceiling than a systems engineering role in C++ at a well-funded startup. The domain trajectory is the most important factor at entry level because you're choosing a path, not just a job.

**Assuming "no sponsorship mention" means "won't sponsor."** Many companies sponsor but don't mention it in job descriptions. Large companies with international workforces almost always sponsor. Only penalise sponsorship when there are active negative signals ("must have permanent right to work") at a small company with no visible sponsor licence.

**Grade inflation from enthusiasm.** A role that hits your personal interest buttons (Rust, low-latency, matching engines) is exciting but must still be graded on the full rubric. Enthusiasm is a signal that the application will be strong — it doesn't change the grade. An unachievable dream role is still F.

**Grade deflation from unfamiliarity.** A role in a domain you haven't encountered before (e.g. "Observability Platform Engineer") deserves research, not dismissal. If you don't know the ceiling of a domain, look it up before grading. Don't default to B because the domain is unfamiliar.
