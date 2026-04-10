# Portfolio Gap Analysis

> A living document tracking what the job market consistently asks for vs what the profile currently offers. Updated as Claude evaluates jobs and spots patterns.

---

## Current Strengths

- **From-scratch implementation depth**: Every major project builds core functionality from first principles (matching engine, PPO, CLIP inference pipeline, DeFi analytics). This is rare at entry level and directly demonstrates understanding rather than framework usage.
- **Observability and instrumentation**: NeuroDrive has a comprehensive analytics pipeline (16 tick fields, 25 episode aggregates, diagnostic markdown reports, profiling infrastructure). Nyquestro has HDR latency histograms with hardware perf counters. This demonstrates production-minded engineering.
- **Performance engineering**: Flat storage 43x improvement in NeuroDrive, lock-free structures in Nyquestro, frame budget management with amortised training. Concrete evidence of profiling-driven optimisation.
- **Multi-domain fluency**: Systems, ML, finance, and desktop application development across a single portfolio. Unusual breadth at entry level.

## Known Gaps

- **No CI/CD in any project**: No GitHub Actions, no automated testing pipelines, no deployment workflows visible in any repo. This is one of the most commonly asked-about skills at entry level.
- **No containerisation**: No Dockerfiles, no container-based development or deployment. Docker/Kubernetes appear in a large percentage of backend and infrastructure roles.
- **No cloud experience**: No AWS, GCP, or Azure usage. Many entry-level roles expect at least basic cloud familiarity.
- **No testing visible in project entries**: NeuroDrive's `cargo test` passes per architecture.md, but testing isn't highlighted in any profile entry. Test-driven development is a common interview topic.
- **No collaborative development evidence**: All projects are solo. No evidence of code review, PR workflows, or team development beyond the university team project.

## Gap Closure Opportunities

- **Add GitHub Actions CI to NeuroDrive or Nyquestro**: A `cargo test` + `cargo clippy` pipeline takes 30 minutes to set up and immediately addresses the CI/CD gap. Nyquestro is the best candidate since it's the most engineering-focused project.
- **Add a Dockerfile to one project**: Even a simple dev container for Nyquestro or NeuroDrive demonstrates containerisation familiarity. Low effort, high signal.
- **Highlight existing tests in profile entries**: If tests exist (NeuroDrive passes `cargo test`), make them visible in the technical highlights. Don't undersell what already exists.
- **Land a merged open source contribution**: The tinygrad PR was closed for line-count reasons. A merged contribution to any well-known project would be a significant signal. Bevy, ratatui, or reqwest are all projects already in the stack.

## Geographic Patterns

> This section tracks which cities and countries fit the profile best, combining **market fit** (what Caner can do well) with **visa accessibility** (what he can actually get hired into) with **lifestyle fit** (where he'd actually be happy — see `lifestyle-preferences.md`). Updated as geographic research and targeted grading surfaces new information.

### Tier SS — top targets beyond London

| City | Market fit | Visa | Lifestyle | Notes |
|---|---|---|---|---|
| **London (current)** | SS | Graduate visa until 08/2027, then Skilled Worker | A (Kings Cross / Nine Elms / Paddington; weak for Croydon-tier areas) | Primary track. Stripe intern, Graphcore, HRT/Jane Street grad programmes, Google KGX1 (aesthetic gold standard). |
| **Dublin** | S | Critical Skills Employment Permit is genuinely friendly to fresh CS grads; €32k threshold; Stamp 4 after 2 years | B+ (good urban energy, weaker than London on scale, fine on café culture, fine on safety) | Stripe HQ, Susquehanna Dublin office, Fenergo. The most mechanically achievable non-UK move for this profile. |

### Tier S — strong fit but with caveats

| City | Market fit | Visa | Lifestyle | Notes |
|---|---|---|---|---|
| **Berlin** | A | EU Blue Card, ~€45k threshold, 21-33 months to PR, A2/B1 German is an asset | Good (Berlin's social scene is notably larger than the rest of Germany — worth re-evaluating if the city had been filed as uniformly restrained) | Trade Republic, N26, Solana Labs, Wayve. Worth re-visiting as a candidate destination. |
| **Paris** | A | Talent Passport "Skilled Employee", €43k threshold, 4-year permit | C+ (aesthetic is historic-not-futuristic; Parisian café culture is hostile to laptop work) | **Hugging Face Paris is a near-perfect technical match** — the only Paris role where the career upside outweighs the lifestyle mismatch. Mistral AI, Datadog Paris as secondary targets. |
| **Munich / Frankfurt** | B+ | Blue Card same as Berlin | C (historic-conservative; good gyms, weak nightlife, religion more visible than Berlin) | Apple Munich, Cerebras Munich, Citadel Frankfurt. Graduate-accessible at some firms but lifestyle friction is real. |

### Tier A — feasible but harder

| City | Market fit | Visa | Lifestyle | Notes |
|---|---|---|---|---|
| **Toronto** | A | Express Entry CRS scoring is structurally biased against fresh non-Canadian CS grads with no Canadian education, no family ties, no LMIA-backed offer. Realistic CRS without ties: 410-450; typical cutoffs: 470-500. PNP routes exist but require provincial connections. **Harder than initially described.** | A+ (aesthetic gold standard on high-rise density; Shopify/1Password culture fit) | Would be SS if the visa worked. Treat as a stretch goal. |
| **Sydney / Melbourne** | B+ | Skilled Independent 189 (points-based), or employer-sponsored 482. Easier than US, harder than EU Blue Card. | A- (good aesthetic on Sydney CBD, good nightlife scene, serious gyms, safe) | Atlassian, Canva, IMC Sydney, Optiver Sydney. Timezone distance from European collaborators is the main friction. |
| **Singapore** | S | Employment Pass is senior-skewed; fresh grad path is narrow. | A (modern, safe, serious gyms, strong tech adoption) but C (religious environment is moderate; nightlife is legally restricted; secular mostly) | **Optiver / IMC / Jane Street / Tower / HRT / Jump all have Singapore offices.** The realistic path is to land at Optiver Amsterdam (no — he's rejected Amsterdam) or IMC London and transfer in after 1-2 years. Direct fresh-grad applications are possible but selective. |
| **Dubai / Abu Dhabi** | B | Golden Visa for tech workers, relatively straightforward | B+ on aesthetic (modern, futuristic, gold-standard high-rises) but D on religious environment (Gulf religious backdrop is a hard friction for an agnostic atheist) | Growing crypto/fintech scene. Tax advantages significant. Worth grading individual roles but not a strategic priority given cultural fit friction. |

### Tier B — structurally hard visa or lifestyle mismatch

| City | Why flagged |
|---|---|
| **SF / Seattle / NYC** | Aesthetically perfect (Seattle in particular), firms are a perfect match, but **H-1B lottery sub-30%** for non-US-grads is the dominant constraint. Apply anyway as stretch — especially to firms with internal immigration teams (Anthropic, Citadel, HRT, Jane Street, Stripe, Databricks) — but do not build the plan around it. |
| **Zürich** | Swiss non-EU quota is ~8,500 permits/year for the entire country, employer must prove no EU candidate exists. Realistic only 2-3 years into career with seniority leverage. |
| **Amsterdam** | **Explicitly rejected by Caner** on aesthetic (historic low-rise), body-scale (Dutch height average), and lifestyle mismatch. Do not recommend despite Optiver/IMC/Flow Traders being top-tier technical matches. Lesson: overrode the aesthetic fit when I first recommended it — don't repeat. |
| **Tokyo / Hong Kong** | Visa is workable (HSP for Japan; various for HK) but the gap between international-firm English and daily-life Japanese/Cantonese is significant. Tokyo-based HFT offices (Optiver, Jane Street, HRT, IMC, Tower) hire non-Japanese speakers but the broader lifestyle is language-dependent. Worth considering as a post-experience transfer destination, not as a first move. |

### Known blockers

- **US security clearance** roles — Turkish nationality excludes. Affects defence, intelligence-adjacent, some trading infra.
- **UK SC/DV clearance** roles — same exclusion. Affects Helsing, parts of Palantir UK, parts of BAE, Faculty AI's Defence wing.
- **Canadian Express Entry without ties** — see Toronto row above. The points deficit without Canadian education/work/family is structural, not fixable with better applications.
- **Switzerland non-EU quota** — see Zürich row above. Not practically addressable for an entry-level candidate.

### Geographic closure opportunities

These are concrete steps that would unlock whole geographies rather than individual jobs:

1. **Push German from A2/B1 to B2** over the next 12-18 months. Unlocks a meaningful slice of Berlin, Munich, Frankfurt, Vienna, and Zürich roles that currently require German — without abandoning the English-first strategy. Low cost, high optionality.
2. **Target a UK-based role at a firm with a strong international transfer programme** (Optiver London, IMC London, Citadel London, Jane Street London, Stripe London). This transforms Singapore/Tokyo/New York from "direct-apply lottery" into "internal transfer after 1-2 years," which is dramatically more realistic.
3. **Build a C++ or OCaml presence in the portfolio** (already tracked in the skill-gaps section above). This has the side-effect of making the profile competitive at Jane Street (OCaml) and the German industrial software scene (C++ heavy) — unlocking cities as a secondary benefit.
4. **Actively avoid building a portfolio that requires security clearance to use** — e.g. don't take an open-source contribution or side project in a domain that requires clearance-equivalent vetting for professional work (classified ML, defence embedded, certain crypto). The current portfolio is clean on this.

---

## Patterns from Job Evaluations

### Batch: QRT (74 jobs), Rerun (3), Schonfeld (2), SingleStore (7), Squarepoint (39) — 2026-04-09

**The quant fund experience wall is real.** QRT listed 74 jobs, not a single one was graduate-level. Every role required 2-10+ years of professional experience. Squarepoint, by contrast, offers explicit graduate/junior pipelines (Graduate SWE, Desk Quant Analyst, Junior QR). This pattern suggests: when targeting quant funds, prioritise firms with structured graduate programmes over firms that only hire experienced professionals.

**C++ proficiency is the single biggest gap for trading systems roles.** QRT's most exciting roles (Low Latency Developer, Market Access Developer, HFT Quant Developer, Core Trading Technology) all require C++ as a hard requirement. Caner's C++ is self-rated "Familiar" — Nyquestro and Tectra demonstrate the concepts (lock-free design, exchange protocols, latency benchmarking) but in Rust, not C++. The Rust-to-C++ translation is real but undemonstrated. **Closure opportunity: build a small C++ component or contribute to a C++ open-source trading/systems project to demonstrate direct C++ proficiency, not just conceptual transfer from Rust.**

**Rust is vanishingly rare in quant finance.** Across 113 quant fund jobs (QRT + Squarepoint), Rust appeared exactly once as an acceptable language (Squarepoint HPC Python role, 1675). The quant finance stack is overwhelmingly C++ and Python. Rerun is the exception — a Rust-native company where Caner's portfolio is a perfect match. **Key insight: Caner's Rust expertise is a massive differentiator at Rust-native companies (Rerun, infrastructure startups) but provides only indirect evidence at traditional quant funds.**

**KDB/q appears as a specialised requirement.** Squarepoint has dedicated KDB quant developer roles. This is a niche but well-compensated specialisation in quant finance. Not recommended as a gap to close given the narrow applicability.

**DevOps/platform roles are abundant but misaligned.** Both QRT and Squarepoint have many platform specialist, SRE, and DevOps roles. These are achievable (especially graduate-level ones at Squarepoint) but diverge from Caner's core interests. The DevOps gap (no CI/CD, no Docker, no cloud) matters more as a general employability signal than as a career direction.

**Graduate programmes at quant funds are the golden ticket.** Squarepoint's Graduate Software Developer (1652, graded SS) offers C++, Python, Frontend, and Quant Dev tracks — multi-track programmes at top quant funds are exceptionally rare and valuable. These roles combine CV signal, sponsorship capability, financial domain exposure, and career trajectory in one package. **Priority: apply to every graduate SWE programme at a top-tier quant fund.**

**Confirmed strengths from this batch:**
- Nyquestro's domain (matching engine, exchange protocols, market data, lock-free concurrency) maps directly to the most sought-after quant fund roles — the gap is seniority and language (C++ vs Rust), not domain knowledge
- Multi-language proficiency (Rust + Python + TypeScript) is a genuine differentiator for graduate roles that offer track selection
- Financial domain knowledge from Aurix + Nyquestro is a standout signal for quant fund graduate programmes — most CS graduates have zero finance exposure

### Batch: Anthropic (47), Adaptive FC (2), Databento (4), Gensyn (1), HRT (33), Jane Street (38) — 2026-04-09

**125 jobs graded. Breakdown: 4 SS, 6 S, 9 A, 16 B, 16 C, 74 F.**

**Anthropic's London roles almost all require significant experience.** Of 47 Anthropic jobs, only the AI Safety Fellow (fellowship, no experience required) graded S or above. Every software engineering role requires 4-10+ years. Every research engineer role expects "industry experience in machine learning research" or advanced degrees (MS/PhD). The London engineering positions are exclusively mid-to-senior. **Key insight: Anthropic's entry point is through the Fellowship programme, not through direct engineering hires.**

**HRT and Jane Street graduate pipelines are the top priority.** HRT's 2026 Grad SWE (ID 1089, SS) and Research Engineer (ID 1085, SS) are explicitly graduate-accessible. Jane Street's Software Engineer roles (IDs 1152/1153, SS) and Production Engineer (ID 1144, S) do not state years requirements. These represent the highest-signal, achievable opportunities in the entire batch.

**OCaml is a growing gap.** Jane Street uses OCaml as its primary language. 8 Jane Street roles graded A or above (SWE, Production Engineer, ML Performance Engineer, PL Engineer, Linux Engineer, Data Engineer, ML Engineer, Tools & Compilers R&D) all involve OCaml. While Jane Street teaches OCaml to new hires, demonstrating some OCaml familiarity would strengthen applications. **Closure opportunity: work through Jane Street's publicly available OCaml exercises or contribute to an OCaml open-source project. Even a small OCaml project would signal genuine interest in the firm's primary language.**

**Compiler/PL engineering is a confirmed high-value gap.** Jane Street's Programming Language Engineer (ID 1145, S) and Tools & Compilers R&D (ID 1154, S) are both S-tier but the profile lacks any compiler project. This was already identified as a gap but this batch confirms it is directly blocking access to some of the most aligned and prestigious roles. **Closure opportunity: build a simple interpreter or compiler (even a toy language) in Rust or OCaml. This would address the gap while demonstrating systems-level thinking in a directly relevant domain.**

**ML at production scale is the boundary between A and S at top firms.** Anthropic, HRT, and Jane Street all have ML roles that would be S/SS if the profile demonstrated production-scale ML experience. NeuroDrive shows ML from scratch but not at the scale these firms operate (petabytes of data, thousands of GPUs). The gap is not ML knowledge — it's ML at scale. **Closure opportunity: training a model on a larger dataset using cloud GPUs (even a one-time experiment on Lambda Labs or Vast.ai) would demonstrate scale awareness.**

**Location filters eliminated ~40% of otherwise interesting roles.** 33 Anthropic roles were F purely due to US-only locations. Several HRT roles (Junior Quant Latency Engineer, AI Tools SWE, Treasury Infrastructure SWE, Distributed Systems SWE) would have been S/A if London-available. **Key insight: the best Anthropic and HRT roles are disproportionately US-located. The London job market at these firms is significantly narrower.**

**Confirmed strengths from this batch:**
- **Rust + systems engineering is a genuine differentiator** — Nyquestro's lock-free matching engine is directly cited in every SS/S assessment as primary evidence. This combination is rare among graduates and immediately distinguishes the profile from web-application-heavy competitors
- **From-scratch ML (NeuroDrive) provides a credible research engineering narrative** — the Anthropic Safety Fellowship (S), HRT Research Engineer (SS), and Jane Street ML Performance Engineer (S) all value empirical ML capability demonstrated through building, not just using frameworks
- **Financial domain knowledge from Aurix + Nyquestro continues to be a standout signal** — HRT and Jane Street roles are explicitly in the trading domain, and the profile's finance exposure is a genuine edge over typical CS graduates
- **The portfolio substitutes credibly for 1-2 years of professional experience** — HRT's C++ SWE (1+ years, graded A), HRT Python SWE (New Grad welcome, graded S), and multiple Jane Street roles accept the portfolio as evidence despite no formal work history

### Batch: Squarepoint (21), Stellar (1), Stripe (27), Synthesia (21), Thought Machine (8), Tower (7), Virtu (1), Waymo (13), Wayve (9), Winton (5), WorldQuant (8), Zopa (5) — 2026-04-09

**125 jobs graded. Breakdown: 1 SS, 1 S, 9 A, 14 B, 6 C, 94 F (75% F).**

**The F rate is extreme because most of these companies only post experienced roles.** Squarepoint posted 21 jobs — only 3 were accessible (up to 4 years or no explicit requirement). Synthesia posted 21 roles — every single one was Senior, Tech Lead, or a non-engineering role. Waymo posted 13 — all Senior/PhD or US-only. This confirms the pattern from earlier batches: most A-tier companies have thin entry-level pipelines. The graduate programme is the exception, not the rule.

**Stripe's intern programme is the standout opportunity.** The Stripe Software Engineer Intern in London (ID 1744, SS) is the only SS in this batch. Stripe's intern programme offers real projects shipping to production, dedicated mentorship, and a conversion pathway — at one of the most prestigious engineering brands in fintech. The other Stripe UK engineering roles (Backend/Payouts 1727, Full Stack/Expansion 1737, SWE/Money Movement 1745) all graded A but explicitly state "intern, new grad, staff should use dedicated postings," suggesting mid-level expectations. **Priority: apply to the Stripe intern programme immediately; consider the mid-level postings as stretch applications where the portfolio's depth may differentiate.**

**Thought Machine has the best accessible infrastructure role.** Software Engineer (Infrastructure) at Thought Machine (ID 1817, S) has no Senior prefix and focuses on tooling/automation for cloud-native banking infrastructure. This is the intersection of infrastructure engineering and financial technology — high ceiling, strong company signal ($2.7B, FT fastest-growing), London. The non-Senior Back End Engineer (1814, A) and Full Stack Engineer (1815, A) are also accessible. **Key insight: Thought Machine's hiring appears genuinely open to junior engineers for non-Senior roles, unlike Synthesia where even "Software Engineer" titles carry Tech Lead expectations.**

**WorldQuant values intellectual horsepower over years.** The Quantitative Developer (1979, A) and AI Software Developer (1976, A) roles at WorldQuant don't state explicit years requirements — the culture description emphasises hiring "the best and brightest" and "intellectual horsepower first and foremost." The from-scratch portfolio (Nyquestro's matching engine, NeuroDrive's PPO) directly demonstrates this. **Key insight: WorldQuant may be more accessible than typical quant funds because they explicitly value raw ability.**

**Cloud/DevOps gap confirmed as the #1 employability barrier.** Roles at Winton (Cloud Engineer), Zopa (Platform Engineer), and across Thought Machine's infrastructure roles all require AWS, Kubernetes, Docker, Terraform. This gap now appears in 3 out of 3 grading batches. The Cloud Engineer role at Winton (1909, B) was downgraded specifically because of the cloud skill gap despite strong domain alignment. **Urgency: the CI/CD + Docker + cloud gap closure recommendations from earlier batches should be actioned before the next application round.**

**Trading firm roles bifurcate sharply: either highly achievable or completely unachievable.** Tower Research Capital posted 7 roles — 2 were B-grade quant dev roles (C++ 1828, Python 1829) with uncertain seniority, 1 was a C-grade operations engineer, and 4 were F (senior/trader/support). Squarepoint had a similar pattern: most roles need 4-6+ years, but the "up to 4 years" roles (Python SWE 1698, Frontend 1693, KDB+/Q 1696) are genuinely entry-friendly. **Key insight: at HFT/quant firms, the achievable roles cluster around graduate programmes and "up to X years" postings — everything else is a wall.**

**Confirmed strengths from this batch:**
- **Financial domain knowledge continues to be the strongest differentiator** — Nyquestro and Aurix are cited as primary evidence in every A-or-above assessment at quant/fintech firms (Squarepoint, WorldQuant, Tower, Stripe Money Movement)
- **Stripe's intern programme explicitly values the kind of work Caner produces** — the programme description mentions "systems design," "rebuilding statistics aggregation service," and "building service discovery systems" — all directly aligned with portfolio projects
- **Thought Machine's accessible roles confirm that fintech infrastructure is a viable career entry** — non-Senior engineering roles at well-funded fintechs are achievable with the portfolio's depth

### Batch: Amber Group (1), Codat (6), Copper.co (3), Cryptio (6), Dojo (16), Ebury (26), Electric Twin (5), Elliptic (16), Evervault (4), Faculty AI (42) — 2026-04-09

**125 jobs graded. Breakdown: 0 SS, 0 S, 1 A, 3 B, 11 C, 110 F (88% F).**

**The F rate is the highest of any batch because these are exclusively C-grade companies posting almost entirely Senior/Lead roles.** Of 125 jobs, 110 were F — overwhelmingly due to seniority mismatch (Senior/Lead titles with explicit years requirements) or non-engineering roles (sales, analyst, design, HR, consulting). Only 15 roles had any potential accessibility, and most of those had other blockers.

**C-grade companies rarely have entry-level pipelines.** Unlike S/A-tier companies (Palantir, Jane Street, HRT) that run structured graduate programmes, C-grade companies almost exclusively hire experienced engineers. Dojo posted 16 roles — 15 were Senior/Lead. Ebury posted 26 roles — the two non-Senior SWE roles (661, 662) explicitly required "3-4 years experience." Faculty AI posted 42 roles — every SWE role required 4+ years or Senior/Lead seniority, and the non-Senior ML roles required SC clearance. **Key insight: searching C-grade companies for entry-level roles has extremely low yield. The min_company_grade filter in preferences.toml could be raised to B for future searches without meaningful opportunity loss.**

**Evervault London (728) is the standout: Rust + payments infrastructure, "All Levels."** Product Engineer on the Cards team using Rust and Node.js, building high-throughput encryption proxies and payments orchestration. This is one of the rare C-company roles that genuinely deserves attention — Rust in the production stack, explicitly open to all levels, and backed by Index Ventures/Sequoia/Kleiner Perkins. The "All Levels" framing is significant because it removes the seniority wall that blocks every other role in this batch.

**UK Security Clearance (SC/DV) is an absolute blocker at Faculty AI.** 8 Faculty AI roles in Defence and National Security required SC or DV eligibility. Caner is categorically ineligible as a Turkish national per visa.md. This is a permanent blocker, not a gap that can be closed. Faculty's non-Defence roles are still available but the consulting model is borderline on the preferences.toml consulting exclusion.

**Faculty AI is fundamentally an AI consultancy, not a product company.** All Faculty roles involve "delivering bespoke AI solutions for clients" — embedding engineers with external organisations. This client-delivery model is borderline on the consulting exclusion per preferences.toml. Future company grading should consider whether Faculty's consulting model warrants a downgrade or sector exclusion note.

**Confirmed strengths from this batch:**
- **Rust remains a genuine differentiator** — the Evervault Product Engineer role (728, A) specifically uses Rust in production. Cryptio's Senior Rust Engineer (444, C) also uses Rust. These are the only roles in 125 that mention Rust, confirming its rarity and value.
- **DeFi/crypto domain knowledge from Aurix opens doors at institutional crypto companies** — Copper.co's Associate roles (432, 433) directly connect to Aurix's cross-DEX arbitrage and settlement knowledge, making Caner competitive for crypto infrastructure roles despite the JVM stack mismatch.

**New gap confirmed:**
- **JVM languages (Scala/Kotlin/Java) are common in fintech infrastructure.** Copper.co (432, 433) requires Scala/Java. Elliptic's Data Engineer (704) uses Scala/Spark. This appeared in earlier batches (Squarepoint KDB/q roles) and continues to surface. Caner has Java familiarity from university but no Scala/Kotlin. Not recommended as a gap to close — the Rust identity is more valuable than JVM breadth.

### Batch: Spotify (5), Stability AI (8), Starburst (2), Teya (45), Tide (6), TrueLayer (2), Unlikely AI (2), Voleon (36), Wincent (1), Wintermute (8), Allica Bank (10) — 2026-04-09

**125 jobs graded. Breakdown: 0 SS, 0 S, 0 A, 5 B, 4 C, 116 F (93% F).**

**This batch is overwhelmingly non-engineering roles at B/C-tier companies.** The extreme F rate (93%) is driven by three factors: (1) Teya posted 45 jobs, ~40 of which are Business Development/Sales roles outside London; (2) Voleon posted 36 jobs, ~30 of which are US/Canada-only duplicates; (3) Tide posted 6 roles, all customer-facing sales. These companies' ATS systems export every role, and while the search pipeline correctly included them, the vast majority are non-engineering.

**Wintermute is the most interesting company in this batch for engineering alignment.** Three C++ Quant Developer roles (1902/1903/1904) and one Python Core Developer role (1905) are all London, sponsor, and deeply aligned with Nyquestro and Aurix domain knowledge. All graded B because of the C++ proficiency gap and seniority uncertainty (minimum 2 years C++ required). **The C++ gap continues to be the primary barrier to the most exciting trading systems roles.** This is the third consecutive batch confirming this pattern.

**TrueLayer Senior SWE [Rust] (1838) is the only Rust-in-production fintech role in this batch.** TrueLayer uses Rust natively for their open banking payments platform — perfect technical alignment. Graded B because of "Senior" title and implied professional experience expectations. **Key insight: TrueLayer is worth watching for junior/graduate Rust roles. If they open entry-level positions, those would be S-tier given the Rust + fintech + London + sponsor combination.**

**Voleon Research Intern requires PhD — no entry path for BEng holders.** Both London Research Intern postings (1844/1863) require "Ph.D. degree expected Dec 2026 or 2027" and "key contributions to multiple top-tier publications." Despite exceptional domain alignment (AI/ML for finance), the academic credentials wall is absolute.

**No new portfolio gaps emerged from this batch.** The batch contained no SS/S/A roles, so no new technology or domain requirements were surfaced. Existing gaps (C++ proficiency, cloud/DevOps, CI/CD) were confirmed as barriers in Wintermute's C++ roles but no new gap patterns appeared.

**Confirmed strengths from this batch:**
- **Rust + fintech alignment continues to be validated** — TrueLayer's Rust-native stack confirms Rust expertise is valued in fintech payments
- **Trading domain knowledge from Nyquestro/Aurix opens doors at crypto trading firms** — Wintermute's roles are directly aligned but blocked by C++ proficiency gap
- **The portfolio's financial domain depth differentiates even at B-tier firms** — every B-grade assessment cited Nyquestro or Aurix as primary evidence

### Batch: Fireblocks (8), FluidStack (9), Form3 (5), GSR Markets (5), Genesis Global (1), GitLab (60), Graphcore (37) — 2026-04-09

**125 jobs graded. Breakdown: 0 SS, 1 S, 10 A, 13 B, 9 C, 92 F (74% F).**

**GitLab posted 60 roles — zero were accessible.** Every role was F: engineering roles are Senior/Intermediate (requiring professional Ruby/Go experience), most are location-restricted to India/North America/APAC, and over half are non-engineering (sales, HR, legal, design, solutions architecture). No graduate pipeline exists.

**Graphcore is the standout — a genuine graduate pipeline.** Of 37 roles, 17 are explicitly graduate/intern level across SWE, ML, firmware, silicon, and research.

**Graphcore ML Kernels & Runtime (999, S) is the top opportunity.** Graduate SWE writing performance-critical code mapping ML operations to custom hardware, optimising execution graphs, building runtime infrastructure. Maps directly to Xyntra (kernel-fusion compiler) and preferences.toml positive signals (compilers, runtime-engineering). Career ceiling is exceptional. Bristol only.

**GSR Markets Quantitative Developer (853, A) is a rare Rust trading role.** Low-latency trading system in Rust. One year Rust experience required. Nyquestro is a near-perfect portfolio match.

**C++ remains the primary language gap for AI hardware roles.** Every Graphcore graduate SWE role requires C/C++. **Closure opportunity: a small C++ project (lock-free queue, ONNX operator) would strengthen all Graphcore and quant fund applications.**

**LLVM/MLIR compiler infrastructure is a high-value gap.** Graphcore Triton (1002, A) mentions MLIR/LLVM. Confirms the compiler gap extends to AI hardware. **Closure opportunity: contribute to LLVM/MLIR subproject.**

**Bristol is a viable secondary location for AI hardware.** 11 of 17 accessible Graphcore roles are Bristol-only. Only 3 have London options.

**Confirmed strengths:**
- Rust differentiates at crypto trading firms — GSR Quant Developer (853) uses Rust for low-latency trading
- Xyntra provides evidence for ML runtime/compiler roles — cited in S-grade for Graphcore ML Kernels (999)
- NeuroDrive from-scratch ML differentiates — relevant to Graphcore Applied AI (993), Research roles (1018, 1021)
- tinygrad ONNX LSTM contribution provides ML framework internals evidence — relevant to Graphcore PyTorch (1001) and Triton (1002)

### Batch: PostHog (1), Proton (20), Quantum Motion (5), Recraft (3), Riverlane (13), Snowflake (37), Speechmatics (17), Spotify (29) — 2026-04-09

**125 jobs graded. Breakdown: 0 SS, 2 S, 4 A, 10 B, 14 C, 95 F (76% F).**

**Snowflake's entire UK pipeline is Solution Engineering/Sales — zero product engineering roles.** Of 37 Snowflake jobs, every single one was either a Solution Engineer (customer-facing pre-sales, excluded by preferences.toml), Sales Development Representative, Account Executive, or Architect. Not a single product engineering role appeared in the UK. Snowflake's engineering roles are likely US-headquartered. **Key insight: Snowflake should be reconsidered at the next company grading pass — a B-grade company with zero accessible engineering roles in the UK provides no value in the pipeline.**

**Speechmatics FutureVoices is the standout opportunity: a genuine graduate programme at an AI company.** Two postings (1596 Cambridge, 1597 London) for a 12-month graduate programme with mentorship from senior engineers. Requirements explicitly state "mindset is the number one thing" — no years of experience required. This is the only graduate programme in the entire batch. Graded S because the programme structure, mentorship, and AI domain are strong, though Speechmatics is B-tier and speech technology is not a core interest domain.

**Proton's Rust SWE (1400) is the most technically aligned non-graduate role.** Description explicitly accepts "complex Rust pet projects" as equivalent to professional experience — Caner's 5 flagship Rust projects massively exceed this bar. The role involves rebuilding Proton Mail Desktop App in Rust, connecting to Image Browser's Tauri/Rust desktop experience. However, the mentoring expectations suggest mid-level, and Proton's top-1% hiring bar is demanding. Graded A.

**B-tier companies post almost exclusively Senior roles — the entry-level drought continues.** Proton: 20 roles, 12 were Senior/Lead, 5 were iOS/Android (platform mismatch), only 3 had potential accessibility (1394 Payments Backend, 1400 Rust SWE, 1411 Linux SWE). Riverlane: 13 roles, 8 were Senior, 3 were quantum physics research (domain mismatch), only 1 accessible (1514 Compiler Engineer). Spotify: 29 roles, 14 were Senior, 5 were non-engineering (designers, data scientists), only the Data Engineer I (1623) had entry-level framing. **This confirms the pattern from every previous batch: the entry-level pipeline at most companies is razor-thin.**

**Spotify has no graduate/entry-level engineering programme visible in the UK.** 29 London roles, zero explicitly graduate/entry-level. Every backend engineer role requires "experienced" engineers. The non-Senior roles (Backend Engineer - Data Platform 1617, Personalization 1618, Platform DevEx 1619) all carry "experienced" language or require Spark/K8s/Java expertise. **Key insight: Spotify's UK engineering hiring appears mid-level only. Watch for seasonal intern/graduate programme postings.**

**Riverlane's Compiler Engineer (1514) validates the compiler gap as actionable.** The role explicitly says "You could have a background in computer science, maths or physics" — extremely open requirements for a compiler role. No years stated. LLVM MLIR-based quantum compiler work. This is the first genuinely accessible compiler engineering role in any batch. Graded A because B-tier company and Cambridge location, but the portfolio-gaps.md recommendation to build a compiler project would directly strengthen applications to roles like this.

**Confirmed strengths from this batch:**
- **Rust proficiency opens unique doors at privacy/security companies** — Proton's Rust SWE (1400) explicitly accepts project portfolios, and Rust is rare enough that Caner's 5 flagship Rust projects are a genuine differentiator vs other applicants
- **Desktop application experience from Image Browser transfers directly** — both Proton Rust SWE (1400) and Linux SWE (1411) involve desktop development, directly connecting to Image Browser's Tauri architecture
- **From-scratch engineering mindset aligns with graduate programme values** — Speechmatics FutureVoices (1596/1597) explicitly values "curious, ambitious, eager to grow" which maps to the self-directed portfolio approach

**No new portfolio gaps emerged.** Existing gaps (cloud/DevOps, C++ proficiency, CI/CD) were confirmed as barriers across multiple roles but no new technology or domain requirements surfaced beyond what previous batches identified.

### Batch: Graphcore (14), Helsing (19), Heron Data (3), Isomorphic Labs (13), Lemurian Labs (6), Linear (1), Marqeta (3), Nethermind (1), Nscale (24), Olix (12), PQShield (3), Paddle (5), Parity (5), Plumerai (2), Polar Signals (3), PolyAI (7), PostHog (4) — 2026-04-09

**125 jobs graded. Breakdown: 0 SS, 0 S, 1 A, 13 B, 16 C, 95 F (76% F).**

**B-grade companies overwhelmingly post Senior/Lead roles with no entry-level pathway.** This is the fourth consecutive batch confirming that non-S/A companies have vanishingly thin graduate pipelines. Of 125 jobs, 95 were F — primarily due to seniority mismatch, location mismatch (Bristol, Munich, US/Canada only), non-engineering roles, or excluded role types.

**Defence AI requires security clearance Caner cannot obtain.** Helsing's AI Research Engineer roles all require MSc (education.md: BEng) and the defence context creates security clearance barriers per visa.md (Turkish national, not eligible for SC/DV). Permanent structural blocker.

**Hardware/optical chip startups have deep software needs but only at senior level.** Olix Compiler Engineer (1281) is perfectly aligned technically — ML compiler backend for custom hardware — but requires 5+ years C/C++. Graphcore ML Kernels (1038) similarly aligned but Bristol location. **Gap closure: entry into compiler/runtime engineering requires targeting explicit graduate programmes at larger firms (ARM, Google, Apple).**

**PQShield internship (1299, A) is the sole accessible opportunity.** 6-month post-quantum cryptography internship, London/Oxford/Remote. Niche but high-value domain.

**MSc/PhD requirements are the second most common blocker after seniority.** Helsing (MSc for Research Engineer), Isomorphic Labs (PhD for Research Scientist), Plumerai (likely PhD/MSc for Research Engineer) all gate research roles behind advanced degrees.

**Confirmed strengths:**
- Rust ecosystem alignment (Parity Core Developer 1336 uses Rust as primary language)
- From-scratch ML (NeuroDrive cited for Helsing RL 1046, Plumerai 1358, Nscale 1272)
- Financial domain from Aurix/Nyquestro remains portfolio-portable across non-finance companies

**No new portfolio gaps identified.** Existing gaps (cloud/DevOps, C++ proficiency, MSc/PhD, CI/CD) confirmed but no new requirements surfaced.

