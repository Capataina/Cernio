# Profile Context for Company Evaluation

A focused distillation of Caner's profile for company grading. This file contains everything the grading agent needs to evaluate companies against the candidate — without reading 14 separate profile files.

---

## Table of Contents

1. [Technical Background](#technical-background)
2. [Career Targets](#career-targets)
3. [Visa Situation](#visa-situation)
4. [Sector Preferences and Dealbreakers](#sector-preferences-and-dealbreakers)
5. [What Matters Most for Company Evaluation](#what-matters-most-for-company-evaluation)

---

## Technical Background

### The short version

Caner is a recent Computer Science graduate (BEng, University of York, 2:2, July 2025) with no formal work experience but a substantial self-built project portfolio in Rust systems engineering, ML infrastructure, and quantitative finance. Turkish national, living in London.

### The portfolio

The portfolio is the primary evidence of engineering capability. It replaces the work experience section that a conventional candidate would have.

| Project | What it demonstrates | Key technologies |
|---------|---------------------|-----------------|
| **Nyquestro** | Lock-free order matching engine with price-time priority, real-time risk layer, binary UDP protocol, market-making strategy agent, HDR latency benchmarking. This is a from-scratch implementation of the most latency-sensitive software in finance. | Rust, lock-free CAS, slab allocator, UDP/FIX, atomic operations |
| **NeuroDrive** | Handwritten PPO reinforcement learning in a Bevy simulation. 8 Bevy subsystems, 43-dimensional observation space, custom reward engineering, amortised training within frame budget. No ML framework dependencies. | Rust, Bevy ECS, PPO from scratch, analytics pipeline |
| **Aurix** | DeFi analytics platform — cross-DEX arbitrage, Uniswap V3 LP backtesting with tick mathematics, wallet position decoding, gas pattern analysis, portfolio risk modelling (VaR, rolling volatility). | Rust, Tauri, React, SQLite, concurrent HTTP |
| **Image Browser** | Local-first desktop app with CLIP-powered semantic search. ONNX Runtime integration via FFI, cosine similarity over 512-dim embeddings, thumbnail pipeline, zero-cloud architecture. | Rust, Tauri, React, ONNX Runtime, CLIP, SQLite |
| **Cernio** | This project. Collaborative job discovery engine with ATS scrapers, Claude Code skills, and a Ratatui TUI. | Rust, Tokio, Reqwest, Ratatui, SQLite |
| **Vynapse** | Hybrid deep learning and neuroevolution engine. Gradient-based training, NEAT topology evolution, DEAP-style population evolution, static graph execution. All in safe Rust. | Rust, autodiff, evolutionary algorithms |

**Additional projects (lower priority but demonstrate breadth):** Xyntra (kernel-fusion compiler pass in Rust), Tectra (C++ trading infrastructure stack), Zyphos (network protocol laboratory in Rust), Chrona (Git internals in C++), Consilium (multi-LLM debate platform in Python/LangChain).

**Open-source contribution:** Implemented an ONNX LSTM operator in tinygrad (Python). PR was closed due to tinygrad's redesign, not quality — the implementation passed all ONNX conformance tests.

### Technical strengths

The profile's technical centre of gravity, in order of strength:

1. **Rust systems programming** — Primary language across all major projects. Comfortable with async (Tokio), lock-free concurrency, ownership-driven design, FFI, ECS architecture (Bevy), and zero-allocation hot paths.
2. **Low-latency and performance engineering** — Nyquestro's matching engine is built around latency measurement (HDR histograms, hardware performance counters). NeuroDrive amortises PPO training to maintain 60 FPS. Performance is treated as a design constraint, not an afterthought.
3. **ML and numerical computing** — Handwritten PPO, ONNX operator implementation, CLIP inference pipeline, DeFi mathematics (Uniswap V3 tick math, VaR). Comfortable with PyTorch, TensorFlow, scikit-learn, XGBoost.
4. **Local-first architecture** — Three production-style projects (Image Browser, Aurix, Cernio) built on the principle of local-first, privacy-by-construction, no-cloud design.
5. **Quantitative finance** — Order book mechanics, market-making, order flow imbalance, adverse selection, LP backtesting, risk modelling. Built directly, not studied abstractly.

### What the profile lacks

Understanding the gaps is as important as understanding the strengths for company evaluation:

- **No formal work experience.** Zero years in a professional engineering team. No code review experience at scale, no production incident experience, no experience with large existing codebases.
- **2:2 degree classification.** Below the typical filter threshold (2:1) for graduate programmes at prestigious companies. This closes some doors at the HR screening stage.
- **No DevOps/cloud experience.** No AWS/GCP/Azure, no Kubernetes, no Terraform, no CI/CD beyond basic GitHub Actions. This is a portfolio gap for many infrastructure roles.
- **No formal security clearance.** Not currently eligible for SC/DV due to nationality and residency requirements. This excludes some defence/government roles.

---

## Career Targets

### Immediate target (0-2 years)

Entry-level or junior systems/infrastructure engineering role in London or Remote-UK. The ideal first job involves:

- Systems-level problems (not application-layer CRUD)
- Rust, C++, or systems-level Python/Go
- Performance-critical or correctness-critical work
- A team that values strong builders over credentials
- A company name that strengthens the CV for the next move

### Long-term target (5-15 years)

Senior/Principal systems engineer at a company paying £500K+ total compensation. This is achievable through the Staff+ IC track at top-tier tech companies, quant firms, or AI labs. It requires:

- Building a career trajectory through increasingly technical roles
- Accumulating company names that signal engineering competence
- Deepening expertise in systems engineering specifically (not broadening into management or generalist roles)

### Seniority range

- **Target:** Entry-level, graduate, junior, new grad
- **Acceptable stretch:** Junior+ or roles that say "senior" but are accessible to strong candidates without years of formal experience (some companies use "senior" loosely)
- **Out of reach:** Roles requiring 3+ years of professional experience, Staff/Principal roles, management roles

---

## Visa Situation

| Fact | Detail |
|------|--------|
| Current visa | UK Graduate visa |
| Right to work | Unrestricted in the UK until August 2027 |
| Expiry | August 2027 |
| After expiry | Requires Skilled Worker visa sponsored by a UK employer |
| Nationality | Turkish (single nationality, no dual citizenship) |
| Security clearance | None held. Not currently eligible for SC/DV. |

### Strategic implications for company evaluation

The visa creates a two-phase career calculus:

**Phase 1 (now through August 2027):** Any UK employer can hire Caner with zero immigration friction — no sponsorship paperwork, no additional cost, no legal risk. This makes Caner cheaper and lower-friction than a candidate who needs immediate sponsorship, which is a genuine competitive advantage at companies that are hesitant about sponsorship.

**Phase 2 (August 2027 onwards):** Sponsorship becomes mandatory. A company that cannot or will not sponsor means leaving the UK or changing jobs. The ideal scenario is joining a company during Phase 1 that can sponsor during Phase 2 — proving value as an employee before the sponsorship conversation.

**What this means for grading:** Companies that are on the UK Sponsor Register and have a track record of sponsoring are more valuable than those that do not, even though sponsorship is not immediately needed. The 1.5-year Phase 1 window mitigates the risk somewhat — a company that is unlikely to sponsor is not automatically bad if the role provides strong CV signal for the next move. But all else being equal, a sponsor-capable company is worth more.

---

## Sector Preferences and Dealbreakers

### Preferred sectors (in rough priority order)

1. Trading systems and exchange infrastructure
2. AI/ML infrastructure and tooling
3. Fintech infrastructure (payments, settlement, risk)
4. Systems software (databases, compilers, runtimes)
5. Developer tools and platforms
6. Infrastructure (cloud, networking, CDN, edge)
7. Security engineering

### Hard exclusions

These sectors are excluded from the search entirely — not evaluated, not graded, not tracked:

- **Gambling** — Ethical objection
- **Adtech** — No interest in optimising ad delivery
- **Consumer crypto** — Distinct from DeFi infrastructure; "web3 social" and "NFT marketplace" type companies

### Not excluded but unenthusiastic

These are not hard exclusions but produce lower grades unless the specific role is technically compelling:

- Pure frontend/mobile companies
- Enterprise SaaS with no systems-level concerns
- Agencies and consultancies (variable work quality, lower career ceiling)
- Healthcare IT (regulation-heavy, often legacy stacks)

---

## What Matters Most for Company Evaluation

When evaluating a company for this specific candidate, the factors that matter most are:

### 1. Engineering culture and reputation

The profile's main weakness is the absence of employer names. A company with strong engineering reputation partially compensates — "I got through Palantir's interview process" and "I worked at Cloudflare" signal competence that a blank work history cannot. Companies known for rigorous hiring bars and strong engineering culture provide the most career signal per year of employment.

### 2. Technical alignment with the portfolio

The portfolio is the profile's main strength. Companies whose day-to-day engineering problems resemble what Caner already builds — lock-free systems, ML infrastructure, data-intensive pipelines, performance-critical services — are companies where the portfolio converts most effectively in interviews and where the daily work builds on existing strength.

### 3. Sponsorship capability

Not immediately required, but a hard constraint within 1.5 years of employment. Companies that cannot sponsor limit tenure to the Graduate visa window, which means the first job becomes a stepping stone rather than a career foundation. This is acceptable but costly — every job change burns time, momentum, and the advantage of internal promotion.

### 4. Career trajectory ceiling

The long-term target is £500K+ at Staff/Principal. Companies with clear IC progression, competitive compensation, and a culture that promotes engineers based on impact rather than tenure are worth more than companies with flat structures or management-only advancement tracks.

### 5. Growth and stability

A company that folds or freezes hiring within a year wastes precious Graduate visa time. Growth is also a proxy for entry-level hiring appetite — expanding companies are more willing to invest in junior talent than stable companies that prefer to hire experienced replacements.
