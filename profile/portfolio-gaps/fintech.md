---
title: Portfolio Gaps — Fintech / Payments
lane: fintech
last_updated: 2026-06-01
seed_source: grade-jobs Phase 3 (187 graded jobs)
pool_size: 187
lane_distribution: {S: 4, A: 28, B: 26, C: 25, F: 104}
---

# Portfolio Gaps — Fintech / Payments lane

The fintech lane is one of the broadest, most-sponsor-friendly, and most CV-defining lanes Caner targets. Pool size is 187 graded jobs across UK-domiciled payments-infrastructure, neobanks, FX, lending, capital-markets, treasury and crypto-finance employers. The realistic-landable S/A pool is 32 of 187 (~17%): wide-funnel grad programmes (Revolut Rev-celerator, Vocalink Launch, FINBOURNE), portfolio-readable IC2/IC4 platform roles at Plaid / Wise / GoCardless / Checkout.com / Starling, and the cluster of mid-band AI-native fintech roles at 9fin / Ebury / Hyperexponential / Quantifi.

The lane's failure-mode signature is well-defined: of 104 F-grades, the dominant patterns are **non-engineering analyst roles** (Credit Analyst, Risk Analyst, Financial Crime Analyst, Treasury Specialist — function-mismatch per `career-goals.md` role-truth-at-hire), **III/Senior seniority floors at neobanks** (Monzo Backend Engineer III, Wise Senior bands), **non-UK locations** (Numerix Remote-India), and **vendor-specialist roles** (Checkout.com Kyriba Specialist — vendor-configuration off-axis). The C-band is mostly platform-specialist / iOS / SOC roles where Q3b's career-axis reading bites despite a strong company brand.

## Open gaps

The gaps below are extracted from recurring JD requirements across the 187-job pool that the current `profile/` cannot evidence. Each is named with the specific roles that surfaced it.

| Gap | Frequency in pool | Surfaced at | Severity |
|-----|-------------------|-------------|----------|
| **PCI-DSS familiarity / cardholder-data-environment exposure** | Recurring at Checkout.com, Stripe, Modulr, Paddle | Software Engineer I (Checkout) cluster, all Stripe roles | Medium — interview-screen liability, not Q1-blocking |
| **Real-money production tenure (chargebacks, settlement, reconciliation)** | Universal across payments-platform JDs | Plaid IC4, GoCardless SDE-III, Wise Recurring Payin, Modulr | High — the single most-cited differentiator between mid-band and senior bands |
| **KYC / AML systems implementation (sanctions screening, transaction monitoring)** | High across neobank + payments-infra JDs | ClearBank Fraud Models, Ebury AI Agent Platform (financial crime), Modulr Screening, Starling KYC tooling | Medium — adjacent via Cernio's deterministic-pipeline shape but no first-hand anchor |
| **Double-entry ledger / accounting-grade transactional integrity** | Universal at neobanks and payments processors | Monzo (Cassandra-backed ledger), Starling (custom in-house ledger), Wise (multi-currency ledger), Modulr | High — the canonical fintech interview question. No portfolio anchor; closest is Cernio's `INSERT OR IGNORE` dedup, which is not ledger-shaped |
| **OpenBanking specifics — PSD2, SCA, FAPI, AIS/PIS scopes** | High at Plaid, GoCardless, ClearBank, TrueLayer-adjacent | Plaid Europe IC4 (explicitly), GoCardless Payment Intelligence | Medium — regulatory context, self-teachable in a weekend |
| **Card-network internals — authorisation flow, interchange, scheme rails (Visa / Mastercard / Amex)** | Recurring at Checkout.com, Stripe, Modulr, Vocalink | All card-processor roles in the pool | Medium — opaque domain; the lack is interview-visible |
| **Payment-rail integration depth — Faster Payments, SEPA SCT/Inst, ACH, Bacs, SWIFT** | Universal across UK-rails JDs | Vocalink Launch Grad (Faster Payments / Bacs explicit), ClearBank SWIFT Infrastructure, Wise Send-for-Banks | High — Vocalink and ClearBank specifically gate on this domain knowledge |
| **Compliance-engineering and regulatory-control implementation** | Recurring at regulated entities | ClearBank Platform / Models, Starling GRC, Modulr Compliance | Low-medium — adjacent via systems-discipline, but no first-hand anchor |
| **Fraud-detection ML — graph / behavioural / sequence models on transactions** | Recurring | Starling SWE (ML Projects), GoCardless Payment Intelligence (Success+ / Protect+), Ebury AI Agent Platform | Medium — NeuroDrive PPO is wrong-domain; Aurix anomaly-aware is closer but not transactional |
| **Java / Kotlin / Spring Boot fluency** | Universal at Revolut, Wise, Vocalink, Starling, ClearBank | Revolut Backend Java grad, Wise Recurring Payin, Vocalink Launch | Medium — concept-fit substitutes per Q3 decomposition rule, but the Wise / Revolut backend tracks explicitly name Java |
| **Go fluency** | High at Monzo, GoCardless, Paddle, Teya | Paddle Go SWE, Monzo Backend, GoCardless SDE-III | Low — learnable in weeks; the rubric treats this as tiebreaker, not a Q3-blocker |
| **C++ / vectorised analytics for quant-dev roles** | Cluster at Quantifi, FINBOURNE, Numerix, Hyperexponential | Quantifi Quant Developer | Medium for that sub-cluster only |
| **Cloud / Kubernetes / IaC depth at production scale** | Recurring at ClearBank, Starling, Modulr | ClearBank Platform Engineer (Network), Starling Cloud Security | Medium — already flagged in legacy gaps, still uncovered |
| **AWS / GCP service-level fluency (Lambda, EKS, Aurora, DynamoDB, Pub/Sub)** | Universal background | Most platform-engineering roles | Low — taught in weeks once on the job; not Q1-blocking |

## Confirmed strengths

The lane has a rich evidence base in the profile. The strengths below are cited by lane-relevant fit assessments across the 187-job pool and represent the dimensions where Caner is a stand-out applicant rather than a baseline applicant.

| Strength | Anchor projects (verbatim from `profile/projects/`) | How the lane uses it |
|----------|-----------------------------------------------------|----------------------|
| **Multi-vendor / multi-provider API normalisation under shared retry + dedup** | `cernio.md` (active, ~14k LOC Rust, 6 ATS provider trait fetchers with `common.rs` retry + slug normalisation across ~10–20 conventions probed in parallel) | Mirrors the structural shape of bank-aggregation work at Plaid (12,000 institutions), payment-rail aggregation at GoCardless, and provider-fanout at Modulr / TrueLayer. Cited verbatim in the Plaid IC4 S-grade. |
| **Deterministic transactional engines with byte-deterministic output** | `nyquestro.md` (active, deterministic LOB matching engine in safe Rust, price-time-priority, microstructure inspection surface) | Direct concept-fit for ledger engines, settlement engines, and any system where reproducibility is a regulatory requirement. The closest portfolio anchor to "double-entry ledger" reasoning. |
| **Production-grade quant-finance math from primary literature** | `aurix.md` (active, clean-room Uniswap V3 Q64.96 maths on `BigUint`, per-direction rounding via `mul_div_round_up`, LVR from Milionis-Moallemi-Roughgarden, 30+ unit tests including bit-exact Solidity reference matches) | Cited verbatim at B2C2 + Quantifi + FINBOURNE — distinguishes Caner from typical CS-grad applicants in quant-adjacent fintech roles. |
| **SQLite WAL discipline at production scale** | `cernio.md` (WAL mode for read-while-write, 6 idempotent migrations, manual table-rebuild for CHECK constraint changes), `image-browser.md` (writer-Mutex + reader-pool topology closing a 22s freeze), `aurix.md` (60-second WAL checkpoint task) | The transactional-integrity proxy. Wise Database Platform IC2 explicitly cited this evidence chain as compensating for the missing PostgreSQL depth. |
| **Idempotency-at-storage-layer thinking** | `cernio.md` (`INSERT OR IGNORE` on UNIQUE constraints as dedup mechanism), `aurix.md` (StrictMode discipline, second-mount cache-hit) | The canonical fintech reliability pattern. Surfaces in Plaid, GoCardless, Wise fit assessments. |
| **TypeScript + React 19 production frontends with strict-mode and TanStack Query** | `image-browser.md` (33 .ts/.tsx with TanStack Query 5 + ApiError discriminated-union), `aurix.md` (45 .tsx + 18 .ts ~9k LoC), `tessarix.md` (79 .tsx + 20 .ts with React 19 lazy/Suspense) | Covers the full-stack and frontend tracks at Revolut, Wise, Checkout.com, Ebury. Three active-status anchors satisfy the rubric's concept-fit citation specificity bar. |
| **LLM-systems engineering with structured-output discipline** | `consilium.md` (dormant, multi-LLM debate orchestration, structured 8-key JSON state, peer-output isolation, provider-agnostic factory + thin-adapter pattern), `tessarix.md` (active, three LLM commands with SSE streaming via futures-util::StreamExt, JSON-schema mode for tiered hints) | Carries the AI-native fintech sub-cluster: 9fin, Ebury AI Agent Platform, Hyperexponential. Caner is over-anchored for this growing surface. |
| **Rust + Python + TypeScript polyglot at depth** | `skills.md` Concepts and Domains | The Stripe / Adyen / Wise / Plaid stack is exactly this polyglot shape. The lane rewards breadth that holds together. |
| **BEng with finance-adjacent maths modules** | `education.md` (Probabilistic & Deep Learning, Evolutionary & Adaptive Computing) | Soft signal for Hyperexponential, Quantifi, FINBOURNE actuarial / quant routes. |

## Closure prescriptions

Ordered by lane-impact per unit of effort.

1. **Build a double-entry ledger demo in Rust with idempotency keys and a Faster Payments simulator harness.** The single most-cited gap (real-money production tenure + ledger discipline + payment-rail integration). A 2-3 week project — `axum` API surface, SQLite WAL backing, idempotency-keyed transfers, simulated Faster Payments inbound/outbound with the published ISO 20022 message schema — collapses three top-cited gaps simultaneously. Naming it as a project in `profile/projects/` lets the existing concept-fit machinery in Cernio + Nyquestro + Aurix do the rest.

2. **Write a PSD2 / SCA / FAPI primer note in `learning/` or as a project README.** A weekend of regulatory reading (FCA PSD2 technical standards, Open Banking UK's API specifications, FAPI 1.0 Advanced). Closes the regulatory-context interview-screen gap at Plaid, GoCardless, ClearBank, TrueLayer in one pass. Self-evidence via published artefact.

3. **Apply systematically to the lane's S/A-tier wide-funnel grad pipelines.** The 32-job realistic-landable pool is dominated by structured programmes. The application cost is low (single CV + portfolio gallery), the conversion is the actual lane-volume question, not the fit question:
   - **SS / S band:** Plaid IC4 London, Revolut Rev-celerator (Backend Java, Python, Frontend, Internship), Vocalink Launch 2026.
   - **A band primary:** GoCardless SDE-III, Wise (Recurring Payin / Send for Banks / Database Platform), Checkout.com SWE I + AI CoE, Starling (Data Platform / ML Projects), Hyperexponential Junior Model Developer, FINBOURNE SWE, Quantifi Quant Dev, 9fin Backend / Full-Stack, Ebury AI Agent Platform / Enabling Teams, Lendable Python Engineer, Vocalink Intern, Thought Machine (Full-Stack / Infrastructure), Paddle Go SWE, ClearBank Platform Engineer, Rimes Data Engineer, Fireblocks Full-Stack, Teya Golang.

4. **Pick up Java basics to a Familiar+ band.** Two weeks with Spring Boot Tutorials and one toy CRUD service. The York libGDX project demonstrates JVM-comfort but Revolut Backend Java and Wise Recurring Payin both name Java explicitly. The cost-benefit is tiny.

5. **Surface a fraud-detection / transaction-anomaly mini-project.** Aurix's anomaly-aware regime detection is adjacent but not transactional. A small graph-based fraud detector on synthetic transaction data (a Kaggle-style dataset) becomes the anchor for Starling ML Projects, GoCardless Payment Intelligence (Success+ / Protect+), Ebury financial-crime AI agents.

6. **Do NOT chase Kyriba / Treasury / Murex / Calypso specialisms.** The C-band of the lane is dominated by vendor-specialist roles (Checkout Kyriba, Numerix BizOps) where the specialism actively pulls the career trajectory off-axis. The structural lesson from the C-grade cluster is that brand alone does not compensate for vendor-track-routing.

## Pinnacle anchors — what the candidate has and what's missing

| Company | Current strongest pull | What's missing | Realistic landing band |
|---------|------------------------|----------------|------------------------|
| **Monzo** | Brand + Go/Kotlin stack adjacency | Backend Engineer III is the only band currently posted — 3-5 year seniority floor. Caner's portfolio substitutes for ~1-2y, not 3-5y. Watch for Monzo's next graduate-engineer cycle reopening. | F at current bands; A/S when grad pipeline opens |
| **Starling Bank** | Concrete A-grades exist (Data Platform Engineer, SWE ML Projects). Engine-by-Starling banking platform is on-axis. | Java / Spring Boot fluency for the core engineering roles; UK-resident sponsorship is fine | A — apply to Data Platform + ML Projects now; B for Cloud Security / Database Reliability sub-cluster |
| **Wise** | A-grades on Recurring Payin, Send for Banks, Database Platform. London Liverpool Street office. | Java backend stack-gap (concept-fit substitutes per Q3 rule). Real-money payments tenure. | A — backend payments tracks are the primary fire |
| **Revolut** | S-grade grad programmes (Backend Java, Python). A-grades on Frontend / Mobile tracks. Rev-celerator is the wide-funnel structured pipeline. | Java for Backend Java track; iOS / Android for Mobile tracks (off-axis anyway). Application timing — Rev-celerator opens annually. | S on Backend Java, Python; A on Frontend; B on Mobile |
| **Stripe (UK)** | None viable currently — all Stripe roles in the pool grade F | Stripe's open UK roles are all senior bands (Backend Engineer Core Tech, Full Stack Link/Privy at mid-senior). Plus the Stripe roles in the pool include several non-engineering (Policy Development, Strategy & Operations, Fraud Patterns Analyst) | F at current postings; watch for Stripe's UK graduate / new-grad opening |
| **GoCardless** | A-grade SDE-III. €1.05B Mollie acquisition + Octopus £12B migration + Tom Blomfield founder. | Mid-band seniority floor at SDE-III is friction (1y professional + portfolio at the lower edge). SDE-II grad route would be cleaner. | A — fire SDE-III; B on SDE-II Frontend |
| **Plaid** | S-grade IC4 London (Europe team). 12,000 financial institutions + bank-aggregation infrastructure mirrors Cernio's architectural shape exactly — the strongest concept-fit anchor in the lane. | IC4 is mid-band (2-4y), realistic with friction. OpenBanking PSD2 / FAPI specifics — the regulatory primer is the closure. | S — top of the application queue |
| **Checkout.com** | A-grades on SWE I, AI CoE. London HQ + 10B daily transactions. | PCI-DSS exposure for the SWE I core-engineering role. Avoid Kyriba Specialist (C-grade, vendor-track routing). | A on SWE I / AI CoE; C on Kyriba; B on Analytics Engineer |
| **Modulr** | None — pool entries are screening-analyst function-mismatch (Financial Crime Screening Analyst graded C). Modulr engineering roles not in current pool. | The role-search for Modulr surfaced only operations roles. Either Modulr is hiring exclusively non-engineering currently, or `search-jobs` missed the engineering tracks. Worth a re-search pass. | Unknown — needs job-search re-run |

## Lane-internal calibration — 187-pool placement

The lane's distribution shape (S: 4, A: 28, B: 26, C: 25, F: 104) implies:

- **2.1% S-grade rate (4/187).** All four S-grades are wide-funnel structured pipelines at lane-pinnacle firms (Plaid IC4, Revolut Backend Java + Python, Vocalink Launch). This is the realistic top — not because there's a budget, but because the realistic-landable + portfolio-stand-out intersection is structurally narrow for a 1-year-experience portfolio candidate. A higher S-rate would indicate inflation, not opportunity.
- **15.0% A-grade rate (28/187).** This is the lane's primary application surface. A-grades are dominated by IC2/IC3-band roles at adjacent-pinnacle firms (Wise, GoCardless, Checkout.com, Starling, Thought Machine, FINBOURNE, 9fin, Ebury, Hyperexponential, Quantifi, Lendable, Paddle, Fireblocks). The portfolio's quant-math depth (Aurix) and multi-vendor API normalisation depth (Cernio) consistently rescue the grade when Q3 stack-fit is moderate.
- **13.9% B-grade rate (26/187).** Mostly backup-band applications — frontend specialisms at strong brands (Paddle Frontend, GoCardless SDE-II Frontend, Revolut Mobile), platform-narrow sub-specialisms (Starling Cloud Security, Wise Data Engineer, ClearBank SWIFT Infrastructure), and mid-tier brand mid-band roles (Atticus, Humaans, Moniepoint).
- **13.4% C-grade rate (25/187).** Vendor-specialist roles, iOS-specialist roles at strong brands, customer-facing integration analyst roles, and the prestige-trap pattern where strong brand + off-axis function bites.
- **55.6% F-grade rate (104/187).** Dominated by: non-engineering analyst functions (Credit Analyst, Risk Analyst, Treasury Analyst, Financial Crime Analyst), senior bands above the candidate's seniority floor (Monzo III, Stripe Senior, Wise Senior), non-UK locations (Numerix Remote-India, Stripe Canada), and Stripe's UK pool being predominantly senior or non-engineering at the current snapshot.

The lane's calibration confirms the realism semantic: a 17% realistic-landable rate is consistent with the candidate's specific constraints (1y professional + portfolio + UK Graduate visa + London preference), not a budget shortfall.

**Gaps:** PCI-DSS, real-money production tenure, double-entry ledger implementation, KYC/AML systems, OpenBanking specifics (PSD2/SCA/FAPI), card-network internals, payment-rail integration depth (Faster Payments / SEPA / ACH / Bacs / SWIFT), compliance-engineering, fraud-detection ML, Java/Kotlin/Spring Boot fluency for the Revolut / Wise / Vocalink backend tracks. **Strengths:** Cernio's multi-vendor API-normalisation architecture (cited verbatim at Plaid), Aurix's clean-room Q64.96 quant-math from primary literature, Nyquestro's deterministic-engine discipline, SQLite WAL transactional integrity, idempotency-at-storage-layer thinking, three-anchor TypeScript+React-19 production frontends, LLM-systems engineering for the AI-native fintech sub-cluster, polyglot Rust+Python+TypeScript at depth. **Key recommendation:** ship a 2-3 week double-entry ledger demo with idempotency-keyed transfers and a Faster Payments simulator harness — this single artefact collapses three top-cited gaps (real-money tenure, ledger discipline, payment-rail integration) and converts the existing Cernio/Nyquestro/Aurix concept-fit machinery into A/S-grade anchors at Wise, GoCardless, Plaid, Modulr, and Vocalink simultaneously.
