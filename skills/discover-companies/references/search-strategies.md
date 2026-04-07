# Search Strategies for Company Discovery

> How to find companies that generic aggregators miss. Every agent should read this before beginning their search.

---

## The core problem

Obvious searches produce obvious results. Searching "best fintech companies UK" returns the same list every recruiter and job board already has — Revolut, Wise, Monzo, Starling. These companies are worth knowing about, but they're not where discovery adds value. The value is in finding companies the user would never have encountered through normal job searching.

This requires looking in places where companies leave traces of their work without explicitly advertising themselves as employers.

---

## Strategy 1: Curated lists and rankings

The most straightforward source. Useful for building the baseline, but should not be the only strategy.

**Where to look:**
- VC portfolio pages — Index Ventures, Balderton, Accel, Sequoia, Notion Capital, LocalGlobe, Octopus Ventures, Hoxton Ventures, Atomico. These list every company they've backed with brief descriptions and often sector tags.
- Beauhurst ranked lists — top UK companies by sector (fintech, SaaS, AI, healthtech, etc.)
- Sifted — European startup database and reporting
- Tech Nation reports — UK tech ecosystem analysis
- TrueUp Hot 200 — weekly-updated ranking of fastest-growing tech companies
- Otta / Welcome to the Jungle — curated tech companies with active job listings
- YC company directory — every Y Combinator company, filterable by sector and batch
- Dealroom — European startup and venture capital data

**How to use well:** Don't just grab names. Read the portfolio descriptions and filter for companies whose work genuinely connects to the profile. A VC portfolio page might list 80 companies, but only 5-10 will do work where systems programming, ML infrastructure, or financial engineering skills are valued.

---

## Strategy 2: Open source and GitHub ecosystem

Companies that build in the open leave traces in the ecosystem. These traces are high-signal because they reveal what the company actually works on, not just what their marketing says.

**Where to look:**
- GitHub organisation pages of companies in relevant sectors — look at their public repos, what languages they use, what they're actively maintaining
- Contributors to key crates and packages: `tokio`, `axum`, `reqwest`, `serde`, `bevy`, `ratatui`, `tch-rs`, `candle`, `ort` (ONNX Runtime Rust bindings), `polars`, `arrow-rs`. Companies often pay engineers to contribute to dependencies they rely on. The contributor's GitHub profile or commit email reveals the employer.
- GitHub Trending — filtered by Rust, Python, C++ to find active projects and the companies behind them
- CNCF landscape — companies contributing to cloud-native infrastructure
- Linux Foundation, Apache Foundation member and contributor lists
- `awesome-rust`, `awesome-ml`, `awesome-fintech` lists — curated collections often include companies alongside tools

**How to use well:** Don't just look at who owns the repo. Look at who's *contributing*. A company that has 3 engineers contributing to `tokio` is building serious async Rust infrastructure even if their company name never appears on a "top startups" list.

---

## Strategy 3: Conference and event sponsors

Companies that sponsor or speak at technical conferences are actively investing in the technologies and domains they sponsor. Sponsor lists are public and often include company descriptions.

**Where to look:**
- RustConf, EuroRust, Rust London — Rust-specific conferences
- QCon London — broad software engineering, attracts infrastructure and systems companies
- MLOps World, MLconf — ML infrastructure and tooling
- KubeCon Europe — cloud-native infrastructure (even if the user doesn't use k8s, companies there often need systems engineers)
- PyData London, EuroPython — Python ML ecosystem
- Fintech conferences: Money 20/20, Finovate
- Healthcare AI conferences: HIMSS, Health 2.0
- Meetup groups: Rust London, London Systems Meetup, London ML — check past sponsors and speakers

**How to use well:** Speaker company affiliations are especially valuable. An engineer giving a talk on "building a low-latency matching engine in Rust" at a conference is revealing exactly what their company works on. The talk abstract often has more technical detail than the company's website.

---

## Strategy 4: Job board threads and community discussions

"Who's hiring" threads surface companies that are actively looking, often with context that job boards strip away — what team, what stage, what the actual work involves.

**Where to look:**
- Hacker News monthly "Who is hiring?" threads — searchable at hn.algolia.com, filter by location (London, UK, remote) and technology (Rust, ML, systems)
- Reddit r/cscareerquestionsEU, r/UKJobs, r/rust (job threads), r/MachineLearning (job threads)
- Rust community job boards: This Week in Rust job listings, Rust Jobs
- Functional programming job boards (many systems-oriented companies post here)
- Blind — company discussions reveal engineering culture and hiring patterns
- levels.fyi — company compensation data reveals which companies are hiring and at what level

**How to use well:** The context in these threads is gold. A HN post saying "We're a 30-person team building ML infrastructure for drug discovery, looking for our first Rust engineer, London-based, will sponsor visas" contains more signal than any job board listing.

---

## Strategy 5: Funding and investment signals

Recently funded companies are the most likely to be actively hiring. Funding announcements often describe what the company does in more detail than their website.

**Where to look:**
- Crunchbase — search by sector, location, and funding round
- TechCrunch, Sifted, The Information — funding round announcements
- UK government investment announcements — particularly in AI, healthtech, fintech
- "Recently funded" filters on TrueUp, Otta, Wellfound
- VC firm blogs and Twitter/X — they announce portfolio companies and investments

**How to use well:** Focus on the round description, not just the amount. A company that raised £20M "to build next-generation ML infrastructure for financial markets" is more relevant than one that raised £100M "to expand our consumer social app." The what matters more than the how much.

---

## Strategy 6: Engineering blogs and technical content

Companies that publish engineering blogs are revealing their technical stack, their problems, and their culture. This is the highest-fidelity signal for assessing whether a company does work the profile aligns with.

**Where to look:**
- Search for engineering blog posts about specific technologies: "Rust in production", "building a matching engine", "ML inference at scale", "local-first architecture"
- Company engineering blogs directly: many mid-size tech companies maintain active blogs (search "[company name] engineering blog")
- Medium engineering publications
- dev.to, Hashnode — individual engineers writing about their work (their employer is in their bio)
- InfoQ, The Morning Paper (Adrian Colyer), Papers We Love — companies referenced in technical discussions

**How to use well:** An engineering blog post titled "How we migrated our trading system from C++ to Rust" tells you: the company has a trading system, they use Rust, they're growing enough to justify a migration, and they value engineering enough to write about it. That's a discovery.

---

## Strategy 7: Sector-specific deep dives

For each sector in the profile, there are domain-specific sources that general tech lists don't cover.

**Fintech / payments / banking infrastructure:**
- FCA register — every regulated financial company in the UK
- Innovate Finance member directory
- Open Banking ecosystem participants
- Payment processor partner pages (Stripe, Adyen, Checkout.com partner ecosystems)

**AI / ML infrastructure:**
- AI company trackers: State of AI Report companies, CB Insights AI 100
- ML tooling landscape maps (MLOps ecosystem diagrams)
- Papers with Code — look at which companies are publishing research
- Hugging Face organisation pages — companies with active ML presence

**Healthcare AI / biotech / computational biology:**
- BioTech companies on the London Stock Exchange
- Wellcome Trust funded companies and research groups
- UK BioIndustry Association members
- Isomorphic Labs, BenevolentAI, Recursion, and their competitive landscape
- NHS Digital and NHS-adjacent tech companies

**Trading systems / quant / market infrastructure:**
- Firms listed on major exchange membership pages (LSE, ICE, CME)
- Proprietary trading firms: many are private and don't advertise, but lists exist (QuantNet, Wilmott forums)
- Market data and infrastructure providers
- FIX Protocol member companies

**Developer tools / compilers / databases / infrastructure:**
- CNCF member companies
- Database company landscape (search "database landscape 2026")
- Developer tool funding rounds
- Companies contributing to LLVM, GCC, V8, or language runtimes

---

## Strategy 8: The "who else" expansion

Once you find one good company, ask: who are their competitors, their partners, their customers, and where do their alumni go?

- Search "[company name] competitors" or "[company name] alternatives"
- Check LinkedIn for "People also viewed" on company pages
- Look at the company's partner/integration pages
- Search for the company on Glassdoor — the "Compare" feature shows related companies
- Look at where the company's engineers previously worked (LinkedIn alumni patterns)

This is the most effective strategy for finding non-obvious companies. One good discovery leads to 5-10 more through proximity.

---

## Combining strategies

No single strategy covers the full landscape. The most productive approach combines:

1. **Curated lists** for the baseline — the obvious names that should be in any universe
2. **Open source and GitHub** for companies building in the specific technologies the profile uses
3. **Funding signals** for companies that are actively growing and likely hiring
4. **Engineering blogs and conference talks** for companies doing the exact technical work the profile aligns with
5. **Community threads** for companies hiring right now with real context
6. **The "who else" expansion** for turning each good find into a cluster of related companies
7. **Sector-specific sources** for depth in the domains that matter most

The first three strategies build breadth. The last four build depth. Both matter.
