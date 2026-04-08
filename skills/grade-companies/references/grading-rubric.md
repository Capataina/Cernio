# Company Grading Rubric

A comprehensive rubric for evaluating whether a company is worth monitoring for jobs, and at what priority. The grade captures a multidimensional judgement about career value — not just "do they have relevant jobs" but "would working here advance the long-term trajectory."

---

## Table of Contents

1. [Evaluation Dimensions](#evaluation-dimensions)
2. [Grade Definitions](#grade-definitions)
3. [Worked Examples](#worked-examples)
4. [Boundary Cases](#boundary-cases)
5. [Companies That Do Not Make the Cut](#companies-that-do-not-make-the-cut)
6. [How Company Grade Interacts with Job Grade](#how-company-grade-interacts-with-job-grade)

---

## Evaluation Dimensions

### High weight

These dimensions drive the grade. A company that scores well on all three high-weight dimensions is almost certainly S-tier. A company that fails on two is almost certainly C-tier.

#### Engineering Reputation

**What to assess:** Is this company known for strong engineering? Do they invest in engineering as a discipline, not just as a cost centre?

**Why it matters:** Caner has no formal work experience. The name on the first job compensates for the gaps that a 2:2 and zero years of employment create. "I was at Cloudflare" signals competence to future employers in a way that "I was at RandomAgency Ltd" cannot. This premium is highest for the first job and decreases with each subsequent role — but for this evaluation, it is at its peak.

**Strong signals:**
- Active engineering blog with substantive technical posts (not marketing dressed as engineering)
- Open-source contributions or projects maintained by the company
- Engineers who speak at conferences, publish papers, or are individually well-known
- Positive Glassdoor/Blind reviews specifically about engineering culture (not just "good perks")
- Tech radar or public stance on technology choices
- Known for engineering-led product development rather than sales-led or consulting-led

**Weak signals:**
- "We use cutting-edge technology" on the careers page with no evidence
- Blog exists but is entirely marketing, product announcements, or SEO content
- No engineering presence on GitHub, no conference talks, no technical publications
- Glassdoor reviews mention "outdated tech stack", "no code review culture", or "move fast and break things without the fixing part"
- Engineering team is a support function for a sales/consulting business

**Nuance:** A small company with 3 brilliant engineers and no blog can have excellent engineering culture. A large company with a beautiful eng blog can have terrible day-to-day engineering. Use the blog and OSS presence as starting evidence, but triangulate with employee reviews, LinkedIn profiles of engineers (what do they work on? how long do they stay?), and the company's actual product (is it technically impressive?).

#### Technical Alignment

**What to assess:** Does the company's core work involve problems that match the profile? Systems engineering, infrastructure, low-latency, data pipelines, ML infrastructure, distributed systems, compilers, runtimes?

**Why it matters:** The portfolio is heavily systems-oriented: a lock-free matching engine, a Bevy RL environment with handwritten PPO, a DeFi analytics platform, a CLIP-powered image browser with FFI to ONNX Runtime. A company doing enterprise CRUD or WordPress themes would waste this alignment. The best companies are ones where the day-to-day engineering problems resemble what Caner already builds for fun — that is where the portfolio converts most effectively in interviews.

**Strong alignment:**
- Core product is infrastructure (CDN, database, message queue, orchestration, networking)
- Performance-critical systems (trading, real-time, low-latency, high-throughput)
- ML infrastructure (training platforms, inference serving, feature stores, model deployment)
- Developer tools (compilers, runtimes, build systems, debuggers, profilers)
- Rust, C++, or systems-level Python in production
- Problems that require thinking about memory, concurrency, latency, or throughput

**Moderate alignment:**
- Backend engineering that is technically deep but application-layer (payment processing, identity, fraud detection)
- Data engineering (pipelines, warehousing, ETL) — relevant but not the strongest match
- Security engineering — adjacent to systems, and the network security coursework provides a foundation
- Go or Java backend with systems-level concerns (garbage collection tuning, lock-free structures, protocol implementation)

**Weak alignment:**
- Primarily frontend or full-stack web development
- Enterprise software with no performance constraints
- Consulting/agency work where the tech stack changes per client
- Marketing technology, CRM, or business intelligence tools
- Mobile development
- Work described entirely in business terms with no technical substance visible

#### Growth Trajectory

**What to assess:** Is this company growing, stable, or declining? Are they hiring, expanding, or contracting?

**Why it matters:** A growing company is more likely to (a) actually hire entry-level engineers, (b) have budget for sponsorship when the time comes, (c) offer career progression as the company scales, and (d) still exist in 2 years. A declining company may rescind offers, freeze hiring, or shut down the specific team after joining.

**Growing signals:**
- Recent funding round (Series A or later — pre-seed is too early for this purpose)
- Active job listings across multiple engineering roles
- Headcount growth visible on LinkedIn (company size trending up over 12-24 months)
- Expanding into new markets, new products, or new offices
- Revenue growth mentioned in press or investor communications
- Recent product launches or major feature releases

**Stable signals:**
- Profitable and self-sustaining, not actively raising
- Consistent headcount, hiring to replace attrition
- Mature product with ongoing development
- No recent negative press or strategic pivots

**Declining signals:**
- Recent layoffs (check layoffs.fyi, news coverage)
- Careers page empty, removed, or showing only non-engineering roles
- No new funding in 3+ years for a company that should need it
- Key people leaving (CTO departure, engineering leadership churn)
- Acquisition rumours or "strategic review" language
- Product deprecated or superseded by competitors

### Medium weight

These dimensions influence the grade but rarely drive it alone. A company strong on high-weight dimensions does not get downgraded to C because of a medium-weight weakness. But between two companies that are equal on high-weight dimensions, medium-weight differences determine the tie.

#### Sponsorship Likelihood

**What to assess:** Can and will this company sponsor a Skilled Worker visa when the Graduate visa expires in August 2027?

**Why it matters:** This is not a soft preference — it is a hard constraint with a deadline. A company that cannot sponsor means leaving the UK or changing jobs within 2 years regardless of how good the role is. Companies that can and do sponsor are materially more valuable than those that theoretically could but have no track record.

**How to assess:**
- Check the [UK Sponsor Register](https://www.gov.uk/government/publications/register-of-licensed-sponsors-workers) — this is the definitive source. If they are listed, they have a licence.
- Company size matters: companies with 200+ employees almost always have the HR infrastructure for sponsorship. Companies with 10-30 employees may find the process burdensome.
- International hiring history: do they employ people from outside the UK/EU? Do job listings mention visa sponsorship?
- Sector patterns: finance, big tech, and defence/government contractors sponsor routinely. Small startups and agencies rarely do.

**The time factor:** Because the Graduate visa runs until August 2027, there is a ~1.5 year window where sponsorship is not needed. This means a company that is unlikely to sponsor is not automatically C-tier — it could still be a strong first job for 1-2 years, building CV signal before moving to a sponsor-capable employer. But it is a real cost, and the grade should reflect that the exit is forced.

#### Career Ceiling

**What to assess:** How far can an engineer progress at this company? Is there a path from entry-level to senior, staff, and principal?

**Why it matters:** The long-term income target is £500K+, which requires reaching Staff/Principal level at a company that pays at that level, or reaching senior at a top-tier company that pays senior engineers that much. A company with a flat org structure where everyone is "Software Engineer" forever limits trajectory even if the day-to-day work is interesting.

**High ceiling indicators:**
- Multiple defined engineering levels (Junior → Mid → Senior → Staff → Principal or equivalent)
- Evidence of internal promotion (engineers who joined junior and are now senior/staff)
- Compensation bands that reach into £150K+ for senior and £250K+ for staff (at the top tier)
- IC (individual contributor) track that is parallel to management, not subordinate to it

**Low ceiling indicators:**
- Flat org with no levels — "we don't believe in titles"
- Engineering team too small to have meaningful levels (5-person team will not have a Staff role)
- Compensation capped at market median with no top-of-band for exceptional performers
- Only path to seniority is management — no IC leadership track

#### Company Stability

**What to assess:** Will this company exist and be hiring 2 years from now?

**Why it matters:** Joining a company that folds within a year means another job search while burning Graduate visa time. Stability is not about being boring — it is about the company surviving long enough to deliver the career value that justified joining.

**Stable indicators:**
- Profitable or has clear path to profitability with sufficient runway
- Revenue-generating product in an established market
- Well-funded (Series B+ with reputable investors) if pre-profit
- Strong market position or defensible niche

**Unstable indicators:**
- Pre-revenue with less than 18 months runway
- Dependent on a single client or contract
- In a market that is consolidating (most competitors acquired or dead)
- Founder/CEO churn, board conflicts, strategic pivots every 6 months

### Low weight (tiebreakers)

These dimensions break ties between companies that are similar on higher-weight dimensions.

| Dimension | What to assess | Notes |
|-----------|---------------|-------|
| **CV signal** | Would this company name impress future employers? | Matters most for the first job. "Palantir" > "Form3" > "RandomStartup". |
| **Location** | London office? Hybrid policy? Commute? | London or Remote-UK is the baseline. Cambridge is acceptable. |
| **Culture fit** | Engineering-led? Small teams? Autonomy? | Strong preference for engineering-led cultures over sales-led or process-heavy. |
| **Compensation** | Do they pay at or above market? | Hard to assess from outside, but Levels.fyi, Glassdoor, and Blind give signals. |

---

## Grade Definitions

Company grades are gentler than job grades — they assess whether a company is worth monitoring, not whether to apply right now. A B-tier company might produce an S-tier role.

| Grade | Meaning | Profile |
|-------|---------|---------|
| **S** | Excellent company — high priority monitoring | Scores well on all three high-weight dimensions. Strong engineering reputation, clear technical alignment, growing. Medium-weight dimensions are at least decent. This is a company where getting any relevant role would be a career win. |
| **A** | Good company — regular monitoring | Strong on two of three high-weight dimensions, acceptable on the third. Or strong on all three but with a notable medium-weight weakness (uncertain sponsorship, flat org). Worth tracking actively — good roles will appear here. |
| **B** | Decent company — worth tracking | Has genuine technical alignment and at least one other high-weight strength, but with clear weaknesses. Maybe small and uncertain, maybe narrow domain, maybe weak on engineering reputation. Still worth having in the database because a good role here is better than no role. |
| **C** | Marginal — archived | Borderline relevance. The connection to the profile is tenuous, or the company's weaknesses outweigh its strengths for this specific candidate. Archived to prevent noise in job searches, but preserved for deduplication. |

### The gap between B and C

This is the most consequential boundary. B companies stay in the active search pool — their jobs get fetched, evaluated, and shown in the TUI. C companies are archived and invisible. Getting this boundary wrong in either direction has real costs:

- **False B (should be C):** The job search wastes time fetching and evaluating roles from a company that was never going to produce a relevant hit. Low cost per company, but it accumulates.
- **False C (should be B):** A company with a genuinely good role gets archived, and the role is never seen. This is a missed opportunity that cannot be recovered.

When in doubt between B and C, lean toward B. The cost of monitoring one extra company is low. The cost of missing a good role is high.

---

## Worked Examples

These examples demonstrate the full reasoning chain through every dimension. They are illustrative — each real company should be evaluated fresh based on current evidence, not pattern-matched to these examples.

### Example 1: FAANG — Cloudflare

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Exceptional.** One of the best engineering blogs in the industry. Major OSS projects (Workers, QUIC, Pingora). Engineers are individually well-known. Rust in production at scale. |
| Technical alignment | **Excellent.** Core product is network infrastructure — CDN, DNS, DDoS mitigation, edge compute. Performance-critical, systems-level, distributed. Directly matches the profile's systems engineering focus. |
| Growth trajectory | **Strong.** Publicly traded, consistent revenue growth, expanding product portfolio, actively hiring across engineering. |
| Sponsorship likelihood | **High.** Large company (3000+ employees), on the UK sponsor register, history of international hiring. London office. |
| Career ceiling | **High.** Clear IC progression to Staff and Principal. Compensation at senior levels is competitive with FAANG. |
| Stability | **High.** Profitable, public, diversified revenue. |

**Grade: S** — Exemplary across every dimension. Any relevant role here is worth pursuing aggressively.

### Example 2: Fintech Scaleup — Form3

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Good.** Known in the payments/fintech engineering community. Engineering blog exists but less prominent than tier-1. Go-heavy engineering team with public talks. |
| Technical alignment | **Good.** Payment infrastructure — technically deep, involves distributed systems, real-time processing, correctness-critical. Not Rust, but the problem domain (low-latency, high-reliability financial systems) aligns with the systems engineering profile. |
| Growth trajectory | **Good.** Series C (backed by Goldman Sachs and Lloyds), expanding, actively hiring. |
| Sponsorship likelihood | **High.** 200+ employees, on the sponsor register, international team. |
| Career ceiling | **Moderate-good.** Multiple engineering levels, but ceiling is lower than FAANG-scale companies. Comp tops out below the £500K long-term target. |
| Stability | **Good.** Well-funded, revenue-generating, strong backers. Not yet profitable but has sufficient runway. |

**Grade: A** — Strong across most dimensions. The weaker brand recognition and lower comp ceiling compared to S-tier companies keep it at A. A great role here would be a strong first job.

### Example 3: Infrastructure Startup — Coadjute

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Unknown/weak signal.** Small company, minimal public engineering presence. No engineering blog, no visible OSS. Hard to assess culture from outside. |
| Technical alignment | **Moderate.** Real-estate settlement infrastructure — involves some systems-level work (integration, real-time data flow) but the domain is niche and the engineering challenges may be more application-layer than systems-layer. |
| Growth trajectory | **Moderate.** Funded (Series A), but small team (30-50). Hiring, but at a scale where adding one entry-level engineer is a significant decision. |
| Sponsorship likelihood | **Low-moderate.** Small company, unlikely to have established sponsorship infrastructure. Might be willing for the right candidate but would be a first-time process. |
| Career ceiling | **Low.** Team too small for meaningful levels. Career progression would come from the company growing, which is uncertain. |
| Stability | **Moderate.** Funded but early-stage. Market (proptech/settlement) is real but niche. Survival is not guaranteed. |

**Grade: B** — Genuine technical work and the domain has depth, but weaknesses on reputation, sponsorship, and career ceiling accumulate. Worth monitoring because a well-scoped infrastructure role here could be excellent, but not a priority target.

### Example 4: Defence/Security — Palantir

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Strong.** Known for hiring exceptional engineers and paying well. Engineering-led product development. Technically demanding interview process signals engineering standards. Uses Rust and Java for core systems. |
| Technical alignment | **Excellent.** Core work is large-scale data infrastructure, distributed systems, and analytical platforms. Performance and correctness are first-class concerns. |
| Growth trajectory | **Strong.** Public company, growing revenue, expanding government and commercial contracts, actively hiring. |
| Sponsorship likelihood | **Complex.** Large company with HR infrastructure, on the sponsor register. However, many roles require security clearance (SC/DV), which Caner is not currently eligible for due to nationality and residency requirements. Need to filter for roles that do not require clearance. |
| Career ceiling | **High.** Clear engineering levels, competitive compensation, strong IC track. Palantir on a CV is a career accelerator. |
| Stability | **High.** Profitable, public, diversified across government and commercial. |

**Grade: S** — Despite the security clearance complication, the overall profile is exceptional. The clearance issue is a job-level filter (some roles require it, some do not), not a company-level disqualifier. The company is worth maximum monitoring, with clearance requirements checked per role.

### Example 5: Quant Firm — XTX Markets

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Exceptional.** Tech-first market maker — technology is the product, not a support function. Known for paying top-of-market and hiring the strongest engineers. Small team, every engineer works on critical systems. |
| Technical alignment | **Excellent.** Low-latency trading infrastructure, matching engines, data pipelines, ML for strategy — directly overlaps with Nyquestro's domain. Caner has literally built a matching engine in Rust. |
| Growth trajectory | **Strong.** Profitable, expanding, actively hiring. Does not raise external funding because it does not need to. |
| Sponsorship likelihood | **High.** International team, on the sponsor register, accustomed to hiring non-UK talent. |
| Career ceiling | **Very high.** Quant-tech compensation is the highest in the industry. Senior engineers earn well into the £500K+ range. IC-focused culture. |
| Stability | **High.** Massively profitable. Survival is not a concern. |

**Grade: S** — One of the strongest possible matches. Technical alignment is near-perfect (Nyquestro is a direct portfolio piece), compensation ceiling is the highest available, and the company is famously engineering-led. The only risk is that entry-level hiring is fiercely competitive.

### Example 6: AI Lab — Anthropic (or equivalent)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Exceptional.** Research lab with world-class engineers. Public contributions to the field, technically demanding hiring bar. |
| Technical alignment | **Good-excellent.** ML infrastructure, distributed training systems, inference serving — all systems-level problems. The ONNX Runtime work, tinygrad contribution, and NeuroDrive's handwritten PPO demonstrate relevant depth. Alignment is strongest on the infrastructure/systems side of AI, less on the research side. |
| Growth trajectory | **Strong.** Well-funded, rapidly expanding, high demand for engineering talent. |
| Sponsorship likelihood | **High.** US-headquartered but with UK/London presence, international hiring is routine. |
| Career ceiling | **Very high.** AI infrastructure engineering compensation is among the highest in the industry. Clear progression paths. |
| Stability | **Good.** Well-funded with strong backing. Market risk is low (AI is not going away). Dependency on continued funding is a minor concern for non-profitable labs. |

**Grade: S** — AI infrastructure is a strong match for the systems engineering profile, and the ML background (PPO, ONNX, tinygrad) provides domain credibility. The entry bar is high, but the payoff is exceptional.

### Example 7: Bespoke/Niche — Grafana Labs

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Strong.** Major OSS company — Grafana, Loki, Tempo, Mimir are widely used. Engineering culture is core to the company identity. Go-heavy but increasingly using Rust for performance-critical components. |
| Technical alignment | **Good.** Observability infrastructure — involves distributed systems, time-series databases, query engines, high-throughput data ingestion. Systems-level problems, even if the domain is observability rather than trading or ML. |
| Growth trajectory | **Strong.** Series D+, growing rapidly, large customer base, actively hiring. |
| Sponsorship likelihood | **High.** 500+ employees, global team, on the sponsor register. Remote-friendly culture with London presence. |
| Career ceiling | **Good-high.** Engineering-led company with clear levels. Comp is competitive but below top-tier quant/FAANG for senior roles. |
| Stability | **Good.** Revenue-generating, well-funded, strong market position in observability. |

**Grade: A** — Strong across the board, with genuine technical alignment (infrastructure, distributed systems, performance). The slightly lower comp ceiling and less direct domain overlap (observability vs. trading/ML infrastructure) compared to S-tier companies keep it at A. An excellent place for a first job — strong engineering culture, OSS-first, and the work is genuinely systems-level.

### Example 8: Trading Infrastructure — Adaptive Financial Consulting

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Good.** Known in the trading infrastructure community. Build Aeron (high-performance messaging) and related tooling. Technically strong team with conference presence. |
| Technical alignment | **Excellent.** Low-latency messaging, exchange connectivity, trading system infrastructure — directly in the domain of Nyquestro and Tectra. Java/C++ heavy, but the problems (lock-free data structures, protocol design, latency measurement) are the same ones Caner works on in Rust. |
| Growth trajectory | **Moderate.** Established company, steady rather than explosive growth. Consulting model means revenue is project-dependent. |
| Sponsorship likelihood | **Moderate.** Medium-sized company, likely on the sponsor register but consulting firms sometimes prefer candidates who do not need sponsorship. |
| Career ceiling | **Moderate.** Consulting company structure may limit IC progression compared to product companies. |
| Stability | **Good.** Established, profitable, niche market with consistent demand. |

**Grade: A** — Excellent technical alignment with the trading infrastructure portfolio, but the consulting model introduces career ceiling and growth trajectory concerns that keep it from S-tier. The domain overlap is strong enough that a good role here would directly build on existing project work.

---

## Boundary Cases

These examples demonstrate where the grade boundary lies and why a company falls on one side rather than the other.

### "Looks like an A but is actually a B" — Monzo

**Why it looks like A:** Well-known fintech brand, engineering blog, large engineering team, confirmed sponsor, growing. Good CV signal.

**Why it is B:** Technical alignment is moderate — Monzo's engineering challenges are primarily application-layer (mobile banking, microservices, product features) rather than systems-level infrastructure. The tech stack (Go, Kubernetes, cloud-native) is relevant but the day-to-day problems are not the kind of systems engineering the profile is strongest in. A backend role at Monzo is a fine job but would not leverage the portfolio's deepest strengths (low-latency, lock-free concurrency, Rust, ML infrastructure).

**The principle:** Brand recognition does not compensate for misaligned technical depth. An A-tier company needs strong technical alignment, not just strong reputation.

### "Looks like a B but is actually an A" — Small Rust Infrastructure Company

Imagine a 50-person company building database infrastructure in Rust, well-funded (Series B), with engineers from Google and Meta, an active engineering blog, and a clear IC ladder to Staff engineer.

**Why it looks like B:** Small, relatively unknown, limited CV signal, narrow domain.

**Why it is A:** Technical alignment is near-perfect (Rust, infrastructure, performance-critical). Engineering reputation is strong for its size (pedigree of engineers, technical blog). Growth trajectory is good (well-funded, hiring). Sponsorship is plausible (Series B with 50 employees typically has HR infrastructure). The only weakness is CV signal, which is a low-weight tiebreaker, not a high-weight driver.

**The principle:** Company size and brand are tiebreakers, not primary drivers. A small company doing exactly the right technical work with strong engineers can outrank a large company doing tangentially relevant work.

### "Looks like an S but is actually an A" — Large Consultancy with Tech Division

Consider a major consultancy (Accenture, Deloitte, McKinsey) with a dedicated technology division that does cloud infrastructure, data engineering, and digital transformation.

**Why it looks like S:** Huge brand, definite sponsor, massive scale, good CV signal, stable.

**Why it is A (or even B):** Engineering culture in a consultancy is fundamentally different from a product company. Engineering is a cost centre, not a profit centre. The tech stack changes per client engagement. Career progression is on a consulting ladder (Analyst → Consultant → Manager → Partner), not an engineering ladder (Junior → Senior → Staff → Principal). The work may be technically interesting on some engagements and mind-numbing on others, with limited control over assignment. Technical alignment is unpredictable.

**The principle:** Size, stability, and sponsorship do not override culture and alignment. A consultancy can be A-tier if the specific division genuinely does deep engineering work, but the consulting model itself is a structural drag on career ceiling and technical growth.

### "Looks like a C but is actually a B" — Pre-revenue Startup with Exceptional Technical Fit

A 15-person startup building a Rust-based distributed database, pre-revenue, seed-funded, with no sponsorship history.

**Why it looks like C:** Tiny, no revenue, sponsorship unlikely, no career ladder, survival uncertain.

**Why it is B:** The technical alignment is exceptional — Rust, distributed systems, database internals. The engineering team, while small, may be doing work that is directly portfolio-relevant. The sponsorship weakness is real but mitigated by the Graduate visa window (can work here for 1-1.5 years without sponsorship). If the company survives and grows, early employee signal is valuable. If it does not, the experience is still technically relevant.

**The principle:** A company with exceptional technical alignment gets the benefit of the doubt at the B/C boundary. The cost of missing a genuinely good role at a company like this is higher than the cost of monitoring one extra small company.

---

## Companies That Do Not Make the Cut

Some companies should never enter the database. These are not C-tier (marginal but preserved) — they are excluded entirely because tracking them has zero expected value.

| Reason | Examples | What to do |
|--------|----------|-----------|
| **Dead** | Website down, company dissolved, domain parked | Remove from potential.md with note "dead / dissolved" |
| **Acquired and absorbed** | No separate hiring, careers redirect to parent | Remove. If the parent company is interesting, track the parent instead |
| **No engineering team** | Purely sales, marketing, or business operations | Remove. "We're a tech company" on the website does not mean they have engineers |
| **Completely unrelated work** | Interior design firm, restaurant chain, law firm | Remove. Sometimes name collisions with tech companies cause false positives in discovery |
| **Too small to hire entry-level** | 3-5 person company with no funding | Remove. They are not going to hire someone who needs mentorship and onboarding |
| **Excluded sector** | Gambling, adtech, consumer crypto | Remove. These are hard exclusions in the preferences, not judgement calls |

The distinction between "C-tier and archived" and "excluded entirely" matters for the database. C-tier companies have a row with grade='C' and status='archived' — they prevent re-discovery. Excluded companies are not inserted at all, so they could theoretically be re-discovered, but the discovery skill should also recognise the same exclusion criteria.

---

## How Company Grade Interacts with Job Grade

Company grade sets a monitoring priority. Job grade determines whether to apply. They are independent assessments — a B company can have an S job, and an S company can have an F job.

| Scenario | Company Grade | Job Grade | Why it makes sense |
|----------|:------------:|:---------:|-------------------|
| Top company, perfect role | S | SS | The dream case. Apply immediately with maximum effort. |
| Top company, wrong role | S | F | Cloudflare hiring a marketing manager. Company is great, role is irrelevant. |
| Risky company, perfect role | B | S | 40-person Rust infrastructure startup with a founding engineer role. Company is uncertain but the role is exceptional for growth. |
| Good company, mediocre role | A | C | Strong fintech with a legacy Java maintenance role. Company is good but the role is a dead end. |
| Unknown company, strong technical role | B | A | Small observability company with a systems engineer role involving Rust and distributed tracing. Company lacks brand signal but the role itself is technically excellent. |

**The key insight:** Company grade determines whether jobs are fetched and shown to the user. Job grade determines whether to apply. A high company grade with no good roles wastes some search time but causes no harm. A low company grade that filters out a great role before it is ever seen is an unrecoverable loss. This is why the B/C boundary matters so much — it is the cutoff between "jobs are visible" and "jobs are invisible."
