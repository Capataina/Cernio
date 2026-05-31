# Fintech discovery — 2026-05-31

> Lane: fintech (dense, 131 existing). Hunt focus: embedded finance, treasury, mortgage tech, insurtech, climate fintech, B2B SaaS finance, A2A payments, wealthtech/pensions. Excludes consumer-crypto, adtech, gambling. All entries verified against `/tmp/cernio-universe.txt`.

---

### Round (Round Treasury)
- **Website**: https://www.roundtreasury.com
- **Location**: London (HQ)
- **What they do**: AI-powered finance-automation platform combining treasury, accounts-payable, and autonomous payroll for finance teams at venture-backed companies; clients include Cleo and PostHog. April 2026 $6M seed (Alstin Capital, Backed VC, Love Ventures).
- **Why relevant**: Treasury + AP automation is the same problem-shape as Cernio's pending-queue + reasoning-grading loop — structured workflow engine over financial events; Nyquestro's deterministic event-stream discipline (per-event `SubmitResult`, idempotent re-runs) maps onto agentic payment workflows; Round's planned "deepen integrations with banks" track aligns with Caner's Rust + multi-source orchestration experience from Aurix's 3-tier ingest fallback.
- **Source**: https://www.eu-startups.com/2026/04/londons-round-treasury-raises-e5-1-million-to-build-ai-powered-finance-automation-platform-for-modern-finance-teams/
- **Sponsor**: unknown (recently funded seed; check gov.uk register at hire time)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### TreasurySpring
- **Website**: https://treasuryspring.com
- **Location**: London (hybrid)
- **What they do**: Institutional cash-investment platform that lets corporate treasurers access fixed-term funds (FTFs) from banks, governments, money-market funds via a single portal — democratising what was historically OTC plumbing. Backed by Balderton Capital and LocalGlobe.
- **Why relevant**: Direct adjacency to Cernio's domain-natural composite-key storage discipline (TreasurySpring sits between corporates and fixed-income markets, exactly the "rules + cash flow" surface Aurix's M2.5 strategies grid models); Python backend role advertised; institutional-fintech is precisely the engineering function Caner's `bank-strats`-adjacent profile targets.
- **Source**: https://treasuryspring.com/careers
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Xelix
- **Website**: https://xelix.com
- **Location**: London (HQ, hybrid)
- **What they do**: Enterprise AP-controls SaaS — AI control-centre that integrates with ERPs to prevent overpayments/fraud, reconcile vendor statements, manage master vendor data. $160M Series B in July 2025 (Insight Partners). 27 open roles incl. Principal Tech Team Lead (Front End), Python Backend, Mid-Level AI Engineer.
- **Why relevant**: Real-time transaction-monitoring is the exact problem-shape as Nyquestro's `MetricsRegistry` per-op latency + `WindowedCounter` rolling counts; React 19 + Python stack aligns with Aurix's frontend + Rust-backend ergonomics; Series B + 27 open roles means active hiring volume for mid-level engineers.
- **Source**: https://careers.xelix.com/jobs
- **Sponsor**: unknown (Series B insurance: check at hire)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Tembo (Tembo Money)
- **Website**: https://www.tembomoney.com
- **Location**: London (Southwark)
- **What they do**: First-time-buyer mortgage fintech — proprietary savings platform plus a broker channel helping people get on/up the property ladder. £14M Series A in late 2024. 150+ employees, actively hiring Senior Ruby Engineers and an Engineering Lead.
- **Why relevant**: Aurix's M2.5 cartesian-grid backtester is structurally the same shape as Tembo's "savings plan + mortgage scenarios" calculator; Cernio's lane-based-relativity refactor parallels Tembo's customer-cohort modelling; engineering-lead role at first-time-buyer scale is the rare junior-track-with-leadership-runway role.
- **Source**: https://builtin.com/job/engineering-lead/2538330
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Generation Home (Gen H)
- **Website**: https://www.generationhome.com
- **Location**: London (Old Street, hybrid)
- **What they do**: Direct fintech mortgage lender (not just broker) — full underwriting platform for first-time buyers, with proprietary scoring beyond traditional affordability. 70+ employees, hybrid-friendly office.
- **Why relevant**: Direct-lender (not broker) means a real risk-engine + decisioning stack — closer to Aurix's verdict-prose synthesiser and Nyquestro's status-state-machine than to a broker UI; Old Street office matches London Tier-1 daily-commute preference; lender's underwriting layer is where Rust + numerical engineering matters most.
- **Source**: https://www.generationhome.com/careers
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Sprive
- **Website**: https://sprive.com
- **Location**: London (Cannon Place, 78 Cannon Street EC4N)
- **What they do**: Mortgage-overpayment automation — round-up debit-card spending into mortgage overpayments, cash-back routed to principal. £5.5M Series A April 2025 (Ascension). Same investor as Tembo.
- **Why relevant**: Mortgage-overpayment math is closed-form-finance (similar to Aurix's V3-concentrated IL closed forms); Open Banking + direct-debit choreography is the same idempotent CRUD discipline as Cernio's `INSERT OR IGNORE`; small Series A team means high engineering ownership per hire.
- **Source**: https://techcrunch.com/2025/04/28/uk-fintech-sprive-closes-7-3m-round-to-facilitate-mortgage-overpayments/
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Molo Finance
- **Website**: https://molofinance.com
- **Location**: London
- **What they do**: UK's first fully-digital mortgage lender — buy-to-let specialism, fully online underwriting from application to offer in minutes. Distinct from Habito/Trussle/Mojo (brokers) — Molo is the direct lender.
- **Why relevant**: Same direct-lender risk-engine appeal as Gen H but with longer track record + product depth in BTL (more complex underwriting); fully-online underwriting = real backend systems, not just UI.
- **Source**: https://molofinance.com/about-molo/careers-at-molo/
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Landbay
- **Website**: https://landbay.co.uk
- **Location**: London
- **What they do**: Buy-to-let mortgage lending platform — institutional and retail capital deployed against UK residential rental properties. Originally P2P, now mostly institutional; backed by Allianz GI.
- **Why relevant**: Capital-markets-side of mortgage tech (not consumer UX) — the work is matching investor capital to loan books, which is the same shape as Nyquestro's `Market::submit_limit` multi-instrument routing; quant-leaning engineering culture.
- **Source**: https://landbay.co.uk/careers
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### M:QUBE (mquba)
- **Website**: https://www.mqube.com
- **Location**: London
- **What they do**: Mortgage-as-a-platform — data + decisioning engine that delivers mortgage offers in minutes; sells the platform to lenders rather than going direct-to-consumer.
- **Why relevant**: B2B mortgage infrastructure (sell-the-pickaxe) is more durable than B2C mortgage; decisioning engine = real Rust/Python-grade numerical work; relatively small team means meaningful early-employee ownership.
- **Source**: https://www.cbinsights.com/company/mqube
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Token.io
- **Website**: https://token.io
- **Location**: London (HQ)
- **What they do**: A2A payment infrastructure provider — single API into 567M bank accounts across 20 countries (80%+ per-market reach). PSD2/Open-Banking-native, sells to PSPs and merchants. Direct competitor to TrueLayer/Yapily but more enterprise-focused.
- **Why relevant**: Aurix's `ArchiveSource` trait abstraction over multiple data sources is exactly the abstraction Token.io's "single API across 20 banking systems" requires; Open Banking PSD2 protocol depth pairs well with Caner's V3-port discipline (read the spec, port carefully); company-sponsored pension noted in careers signals mature ops.
- **Source**: https://token.io/careers
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Noda
- **Website**: https://noda.live
- **Location**: London (UK HQ; also Cyprus)
- **What they do**: Open Banking A2A payments for merchants and platforms — 1,650 banks across 28 countries; competes with Brite Payments and Trustly on European A2A. Real-time payment processing + financial-insights products.
- **Why relevant**: Real-time A2A is a payments-infra play that needs the same idempotency + replay-safety discipline as Cernio's storage layer; cross-border bank-API orchestration is the same problem-class as Aurix's 5-chain × 3-protocol × per-pool routing.
- **Source**: https://uk.linkedin.com/company/noda
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Finexer
- **Website**: https://finexer.com
- **Location**: London (UK-focused)
- **What they do**: UK-native Open Banking platform — 99% UK bank coverage, FCA-authorised AIS + PIS. Pitches itself as the lighter, faster, UK-tuned alternative to pan-European Plaid/TrueLayer/Tink. Startup, Standard, Enterprise public pricing.
- **Why relevant**: UK-first specialism = deeper integration per bank than the breadth-first incumbents; small early-stage team means rare opportunity to own a bank-integration vertical end-to-end; the engineering work is the same wire-format/decoder discipline as Nyquestro's Coinbase L2 bridge.
- **Source**: https://www.openbanking.org.uk/regulated-providers/finexer-ltd-2/
- **Sponsor**: unknown (small early-stage)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Fena (formerly FaizPay)
- **Website**: https://fena.co
- **Location**: London
- **What they do**: Open Banking payments for merchants — Pay-by-Bank checkout reducing card fees by up to 85%. FCA-authorised PISP. Listed by UKTN as one of the fastest-growing payments-infra firms 2026.
- **Why relevant**: Pay-by-Bank acceptance is the merchant-facing flip-side of Token.io/Noda's bank-side work — different surface, same underlying open-banking primitives; small team = real ownership per hire.
- **Source**: https://uk.linkedin.com/company/faizpay-fena
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Atlantic Money
- **Website**: https://atlantic.money
- **Location**: London (FCA-authorised payments institution since 2022)
- **What they do**: Flat-fee international remittance — €3 fixed fee, real-time mid-market rates, no per-transfer FX margin. Differentiates from Wise on pricing structure. Recently expanded to Canada + Australia, USA via bank-sponsor route.
- **Why relevant**: Lean payments-institution engineering culture — small team, FCA-regulated, direct competitor to Wise. The cross-border settlement plumbing is the kind of problem where Aurix's idempotent ingest discipline + Nyquestro's deterministic event-stream design pay off.
- **Source**: https://uk.linkedin.com/company/atlanticmoney
- **Sponsor**: unknown (small FCA-licensed firm)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Sixfold AI
- **Website**: https://www.sixfold.ai
- **Location**: London (UK + EU expansion office after Jan 2026 $30M Series B); US-HQ
- **What they do**: Generative-AI underwriting platform for insurers — feeds decision + outcome data into a continuously-refining "AI Underwriting Brain". Partners with Zurich, Generali, AXIS, Skyward Specialty, Mosaic. Expanding into UK + EU market 2026.
- **Why relevant**: Underwriting decisioning is the same closed-loop problem-shape as Cernio's grade-jobs relativity-pass (decision → outcome → re-calibrate); applied-ML on structured insurance data lines up with Caner's `ai-ml` lane interest.
- **Source**: https://www.sixfold.ai/about (Jan 2026 Series B announcement)
- **Sponsor**: unknown (recent UK entry)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Sweep
- **Website**: https://www.sweep.net
- **Location**: London office (also Paris HQ + Denver, CO)
- **What they do**: Enterprise climate-management platform — carbon accounting, supplier-emissions tracking, regulatory reporting, decarbonisation planning. Named Leader in Verdantix 2026 Green Quadrant. Founded 2020, well-funded (Series B+).
- **Why relevant**: ESG-finance reporting is mandated by UK SRS from FY-April-2026 onward — durable regulatory tailwind. Carbon-accounting math = clean numerical computation on supply-chain data, a domain shape close to Aurix's benchmark-asset computation.
- **Source**: https://www.sweep.net/blog/top-esg-software-for-uk-businesses-a-2026-guide
- **Sponsor**: unknown (verified UK office, multi-country employer)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Plan A
- **Website**: https://plana.earth
- **Location**: London office (Berlin HQ)
- **What they do**: Carbon-accounting + decarbonisation-planning platform for mid-large enterprises — TÜV-certified methodologies + advisory layer. Listed alongside Watershed and Sweep in 2026 carbon-software rankings.
- **Why relevant**: Mid-market carbon-accounting (not enterprise-only like Watershed) means engineering work skews toward repeatable platform rather than bespoke deployment; numerical work on emissions-factor matrices maps onto Aurix's strategy-grid numerical patterns.
- **Source**: https://www.sweep.net/blog/ultimate-guide-to-esg-software
- **Sponsor**: unknown (Berlin-HQ; check UK office sponsorship)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Algbra
- **Website**: https://www.algbra.com
- **Location**: London (HQ)
- **What they do**: Values-based / non-interest (Sharia-compliant) fintech platform — banking + investing products built for ethical-finance customers. Operates inside UK regulatory framework with tier-one banking partners.
- **Why relevant**: Sharia-compliant finance has tight rule-engine requirements (riba avoidance, asset-backed structures) — the work is exactly the kind of constraint-encoding Cernio's reasoning-based grading already does; specialist-bank engineering is rarer than generic neobank work.
- **Source**: https://www.algbra.com/careers/
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Updraft
- **Website**: https://www.updraft.com
- **Location**: London
- **What they do**: Credit-app for millennials — combines lending + credit-report + budgeting; help users break the credit-card cycle. Listed on Built In London as actively hiring AWS DevOps Engineer + Underwriting Team Lead.
- **Why relevant**: Underwriting + lending decisioning is the closed-loop decisioning problem-shape Caner has been working with in Cernio; AWS infra role provides cloud-credentials exposure he currently lacks (matches `portfolio-gaps.md` Docker/CI surfacing).
- **Source**: https://builtinlondon.uk/company/updraft/jobs
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Capchase
- **Website**: https://www.capchase.com
- **Location**: London office (also NYC + Madrid)
- **What they do**: Revenue-based finance + vendor financing for B2B SaaS — non-dilutive capital advanced against ARR, repaid as a percentage of revenue. $1B+ deployed. London is one of the company's major hubs.
- **Why relevant**: RBF underwriting requires real-time integration with billing/ERP systems + ML on revenue-stability features — the same multi-source ingest + decisioning pattern as Aurix's 3-tier fallback; B2B SaaS finance is durable category with structural growth.
- **Source**: https://www.capchase.com/careers
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Uncapped
- **Website**: https://weareuncapped.com
- **Location**: London (HQ)
- **What they do**: Revenue-based financing for online businesses — flat-fee advances against future revenue, repaid as percentage of sales. London-founded, raised £80M+ across multiple rounds. Direct UK competitor to Capchase.
- **Why relevant**: UK-native RBF play means London-centric eng team + UK-bank-rails work; engineering shape is the same revenue-stream-analysis + automated-decisioning loop. Smaller and earlier than Capchase = more individual ownership.
- **Source**: https://weareuncapped.com/careers
- **Sponsor**: unknown
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Tuum
- **Website**: https://tuum.com
- **Location**: London office (HQ Tallinn, Estonia)
- **What they do**: Cloud-native modular core-banking platform — direct competitor to Thought Machine and 10x Banking, with a more modular per-product (deposits / loans / cards / payments) sell. Powers multinational banks + fintechs.
- **Why relevant**: Core-banking engineering is precisely the systems-infra-meets-fintech intersection Caner targets — same depth as Thought Machine + 10x (both in DB) but earlier stage = more impact per engineer; modular architecture means cleaner subsystem ownership.
- **Source**: https://tuum.com/careers/
- **Sponsor**: unknown (verify UK entity)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### Mambu (UK)
- **Website**: https://mambu.com
- **Location**: London office (HQ Berlin/Amsterdam)
- **What they do**: SaaS cloud core-banking platform — 260+ banks/lenders/fintechs in 65 countries, customers incl. N26, ABN AMRO. Stronger in consumer + SME lending than Thought Machine; established but still aggressively hiring.
- **Why relevant**: Same core-banking lane as Tuum/Thought Machine/10x but with scale advantage and proven multi-tenant ops — engineering work is on the harder reliability/scale axis; multi-country employer with established sponsorship process likely.
- **Source**: https://mambu.com/en/careers
- **Sponsor**: unknown (multi-country tech employer)
- **Lane**: fintech
- **Discovered**: 2026-05-31

### BKN301
- **Website**: https://www.bkn301.com
- **Location**: London (UK BaaS Series B announced 2026)
- **What they do**: Banking-as-a-Service technology provider — BaaS Orchestrator platform letting fintechs spin up regulated banking products. £18.6M Series B in 2026 for global expansion + acquisitions.
- **Why relevant**: BaaS orchestration is multi-region multi-rail routing — the same engineering shape as Aurix's 5-chain × 3-protocol routing in `config/chains.rs`. Series B + acquisition-mode means real platform engineering opportunities, not just product tickets.
- **Source**: https://www.fintechfutures.com/venture-capital-funding/uk-baas-fintech-bkn301-lands-18.6m-series-b
- **Sponsor**: unknown (recently funded growth co.)
- **Lane**: fintech
- **Discovered**: 2026-05-31

---

## Notes on companies considered and skipped (dense-lane discipline)

- **Kriya (B2B paylater + lending)** — acquired by Allica Bank Oct 2025; Allica already in DB (line 22). Skip to avoid duplicate-org.
- **Round Treasury** — kept despite seed-stage because the AP/treasury surface is genuinely underrepresented in the existing 131-co universe.
- **Marshmallow, Penfold, Plum, Sokin, Pleo, Soldo, Spendesk, Wamo, Sling Money, Yaspa, Yonder, Vitesse, Volt, OpenPayd, BVNK, BCB, Modulr, ClearBank, Allica, Tide, Form3, ComplyAdvantage, Onfido, Hyperexponential, Featurespace, Ravelin, SEON, FintechOS, Personetics, Eigen, Behavox, Smarsh, TRM, Elliptic, Cryptio, Fonoa, Lenkie, Aptitude, Alfa, Hansen, Gresham, Brady, Toqio, Trustly, Ozone, Coadjute, Apexx, Nium, ZILO, Diesta, Statement (Tuza), Watershed Technology, Bud, Moneyhub, ClearScore, Codat, Primer, Paddle, Globacap, Cassini, Quantile, Beacon Platform, Genesis Global, Thought Machine, 10x Banking, FNZ, Finastra, BankiFi, Volante, Vega Investment, Banking Circle, Bank of London, Griffin, ZILO, Yapily, Cryptio, Carta, Ledgy, Quantifi, Web3 Labs, GoCardless, Marqeta, TrueLayer, Banking Circle** — all already in universe. Skipped.
- **Cuvva, Zego, Veygo, Flock** — Zego known unicorn; not in current DB but consumer/auto-insurance focus puts them lower priority than Sixfold (B2B AI) and Marshmallow's already in DB; deferred this round, candidates for next pass.
- **Habito, Trussle** — acquired (Monzo and OneDome resp.) — skip.

## Source quality

- Sponsor licence verification deliberately conservative — most entries marked `unknown` rather than `yes` because the gov.uk Skilled Worker register changes weekly and only `Kriya Finance Limited` had named worksponsor confirmation in this pass. Cernio's `populate-db` flow handles the careers-page verification at hire-time so `unknown` is the honest current-state, not a blocker.
- Every entry has a verifiable web source — no synthesis-only finds.
