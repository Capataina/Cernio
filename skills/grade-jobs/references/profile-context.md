# Profile Context for Job Evaluation

> A focused distillation of the candidate's profile, optimised for making fast, accurate grading decisions. Read this before grading any jobs.

---

## Technical identity

The candidate is a systems-oriented engineer whose projects demonstrate a consistent pattern: taking complex domains (market microstructure, DeFi mathematics, reinforcement learning, compiler fusion, CLIP inference) and building the core machinery from first principles in Rust, without leaning on frameworks. This is the defining characteristic of the profile — not "knows Rust" but "builds complex systems from scratch and instruments them rigorously."

### Primary technical strengths

| Strength | Evidence | What it means for job fit |
|----------|----------|---------------------------|
| **Rust systems programming** | Primary language across 7+ projects. Lock-free data structures, async (tokio), FFI, ECS architecture, custom binary protocols. | Strong fit for any role listing Rust. Also strong for C++ roles where the low-level thinking transfers. Weaker match for roles where Rust is incidental. |
| **Lock-free concurrency and low-latency design** | Nyquestro: lock-free CAS operations, slab allocator, HDR latency histograms with hardware perf counters. | Directly relevant to HFT, trading systems, database internals, real-time infrastructure. This is a rare skill at entry level and a genuine differentiator. |
| **Reinforcement learning from scratch** | NeuroDrive: handwritten PPO with clipped surrogate, GAE, asymmetric actor-critic, no ML framework dependency. | Demonstrates ML fundamentals beyond framework usage. Relevant for ML infrastructure roles and research-adjacent positions. |
| **Local ML inference and ONNX** | Image Browser: CLIP inference via ONNX Runtime FFI. tinygrad: LSTM operator contribution. Xyntra: kernel fusion compiler. | Direct relevance for ML serving, model optimisation, and inference infrastructure roles. The ONNX experience is a specific differentiator. |
| **Financial domain knowledge** | Nyquestro: order book mechanics, price-time priority matching, market-making, adverse selection. Aurix: Uniswap V3 tick mathematics, DeFi analytics, VaR, risk modelling. | Unusual combination of financial domain knowledge and engineering skill. Directly relevant for fintech, trading systems, exchange infrastructure. |
| **Observability and instrumentation** | NeuroDrive: 16 tick fields, 25 episode aggregates, diagnostic reports. Nyquestro: HDR histograms, hardware perf counters. | Demonstrates production-minded engineering. Relevant for SRE, platform engineering, and any role that values measurement. |

### Secondary strengths

- **Python for ML and data work:** PyTorch, TensorFlow, scikit-learn, XGBoost, DEAP. Comfortable, not expert.
- **TypeScript/React for frontends:** Tauri desktop applications. Comfortable with frontend layer but it's not the focus.
- **SQLite as a local data layer:** Used across three projects. Understanding of when embedded databases are the right tool.
- **Desktop application architecture:** Tauri applications with Rust backends. Unusual specialisation that's relevant for desktop tooling roles.

### Known weaknesses (portfolio gaps)

These are skills the market frequently asks for that the profile currently lacks. If a role requires one of these as a hard prerequisite, the candidate cannot credibly claim the skill. If it's a "nice to have," the gap is noted but doesn't force a grade reduction.

| Gap | Severity for job fit | Status |
|-----|---------------------|--------|
| **No CI/CD experience** | Moderate — asked for at most entry-level roles but learnable in a day | No GitHub Actions in any project |
| **No containerisation** | Moderate — Docker/Kubernetes appear in many backend and infrastructure roles | No Dockerfiles in any project |
| **No cloud experience** | Moderate to high — AWS/GCP/Azure appear in the majority of infrastructure roles | All projects are local-first by design |
| **No collaborative development evidence** | Low to moderate — all projects are solo, no PR workflows, code review, or team contribution visible | University team project (Java/libGDX) was lead developer but isn't in a public repo |
| **No Go experience** | Low — Go is common in platform engineering and infrastructure but not a hard requirement when Rust/C++ are present | Go is learnable quickly given the existing systems background |

---

## Career targets and what they mean for grading

### The north star

The candidate's long-term target is reaching 500K+ annual compensation over a career, specifically in systems engineering, trading systems, AI/ML infrastructure, or platform engineering at companies where engineering is the core competency.

**What this means for grading:** Grade career ceiling by asking "does this domain produce 500K+ engineers at 10-15 years?" rather than "is this a nice first job?" The domains that reach this level are: trading systems (quant developer, HFT engineer), ML infrastructure (staff ML platform engineer), systems/infrastructure (staff/principal engineer at FAANG), database/compiler/runtime engineering (specialised but extremely well-compensated), and some fintech infrastructure roles (principal engineer at Stripe, Wise, or similar).

### The immediate goal

First full-time engineering role, ideally starting in 2026. The Graduate visa provides unrestricted right to work until August 2027, making the candidate inexpensive and low-friction to hire for the next year.

**What this means for grading:** Roles that are achievable now and set up the trajectory matter more than roles that are perfect but require 3 years of experience. A "good enough" role at the right company beats a "perfect" role the candidate can't get. But don't over-index on achievability to the point of accepting dead-end roles — the whole point is setting up the trajectory.

---

## Seniority constraints

### What the profile supports

The candidate has no formal work experience but an unusually deep project portfolio for entry level. The projects demonstrate skills that most 1-2 year professionals don't have (lock-free data structures, from-scratch RL, custom matching engines). This creates an unusual seniority profile:

- **Clearly achievable:** New grad, graduate programme, entry-level, junior, intern-to-hire
- **Stretch but credible:** "1-2 years" where the description emphasises projects and ability over tenure, "mid-level" at companies with title deflation
- **Unlikely but worth trying if the role is exceptional:** "2-4 years preferred" with "or equivalent" language, "Senior" at companies known for title deflation (some UK companies use Senior for their 2nd level)
- **Not achievable:** Hard "5+ years," staff/principal scope, expectations of production incident management or team leadership

### How to assess seniority from descriptions

Ignore the title. Read the description for:
1. **Years stated:** Is it a hard requirement ("must have 5+ years") or a preference ("ideally 2+ years")?
2. **Scope of responsibility:** "Own a component" is entry-friendly. "Own the architecture of the payments platform" is not.
3. **Expectations of others' work:** "Mentor junior engineers," "lead technical design reviews" presupposes experience.
4. **Production expectations:** "Production incident management experience" and "on-call leadership" presuppose operational maturity.

---

## Visa timeline and sponsorship assessment

| Period | Status | Implication for grading |
|--------|--------|------------------------|
| Now through August 2027 | Graduate visa, unrestricted right to work | No sponsorship needed. The candidate is low-friction to hire. |
| August 2027 onwards | Skilled Worker visa required | The employer must hold a valid sponsor licence and be willing to sponsor. |

### How to factor sponsorship into grades

- **For roles at large companies with established graduate programmes:** Assume sponsorship is viable unless there are negative signals. These companies routinely sponsor.
- **For roles at mid-size companies:** Check if the company appears on the UK sponsor register (the agent can look this up). Note uncertainty if unclear.
- **For roles at small startups:** Sponsorship is uncertain. If the company has fewer than 50 employees and no visible sponsor licence, note the risk. This doesn't force an F — a year of great experience at a startup that can't sponsor is still valuable if the candidate can move to a sponsoring employer before August 2027.
- **For short-term contracts or roles explicitly limited to 1 year:** Sponsorship matters less. The experience is the value, not the long-term position.

---

## Tech stack evaluation

### Strongly aligned stacks

These stacks match the profile directly and the application would be strong:
- Rust (primary language)
- C++ (familiar, systems-level thinking transfers directly)
- Python for ML/data (comfortable, production-ready for ML work)
- ONNX Runtime, model serving infrastructure
- Low-latency systems, performance-critical code
- SQLite, embedded databases

### Well-aligned stacks

These don't overlap directly but the candidate can make a compelling case:
- Go (systems language, quick to learn for someone who knows Rust)
- Java (familiar from university, common in backend and trading systems)
- Kubernetes, Docker (portfolio gap but conceptually understood)
- PostgreSQL, distributed databases (SQL experience from SQLite, distributed systems concepts from projects)

### Weakly aligned but not dealbreakers

- TypeScript/Node.js backend (has TypeScript experience but backend Node.js is not the profile's strength)
- .NET / C# (game modding experience in C# but no backend .NET work)
- Ruby, PHP, Scala (no direct experience but language-learning is not a barrier for a polyglot)

### Dealbreakers only if the stack IS the role

- WordPress, Drupal, or CMS-focused work
- Low-code/no-code platforms as the primary tool
- Legacy COBOL or mainframe systems

---

## What makes a job genuinely exciting vs merely acceptable

### Exciting (pushes toward S/SS)

- The role involves building something from scratch or substantially from first principles
- The domain has a direct connection to an existing project (trading systems, ML inference, distributed systems, performance engineering)
- The technical challenge is real — not theoretical complexity but actual hard problems at scale
- The team includes engineers whose work you can find and respect (open-source contributions, papers, talks)
- Rust is in the production stack (rare and highly valued)
- The role title and description suggest you'd be building infrastructure that other engineers depend on

### Acceptable (appropriate for A/B)

- Well-known company, reasonable work, good learning environment, but the specific role isn't deeply aligned with profile strengths
- The tech stack is standard (Java/Go/Python) and the domain is reasonable but not a personal interest
- The company has good engineering culture but the role is more operational than creative
- The career ceiling is solid but not exceptional (backend engineer at a SaaS company)

### Warning signs (pushes toward C/F)

- The description is mostly about processes, meetings, stakeholder management, and "driving alignment" rather than writing code
- The tech work is primarily integration — connecting third-party services rather than building systems
- The role is framed around a single tool or framework (just Terraform, just Ansible, just monitoring)
- The company's "engineering" is configuring vendor products rather than building technology
- The description mentions "customer-facing" responsibilities as a significant part of the role
