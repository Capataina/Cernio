# Company Grading Rubric

A comprehensive rubric for evaluating whether a company is worth monitoring for jobs, and at what priority. The grade captures a multidimensional judgement about career value — not just "do they have relevant jobs" but "would working here advance the long-term trajectory."

**Important:** This rubric references the candidate's profile throughout. All profile facts must come from reading the files in `profile/` — never from hardcoded values in this document. When the rubric says "the candidate's visa situation" or "the portfolio," it means: read `visa.md`, read `projects.md`, etc.

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

**Why it matters:** Read `experience.md` to understand the candidate's work history. If formal experience is limited, the name on the first/next employer compensates for credential gaps in a way that an unknown company name cannot. This premium is highest for the first job and decreases with each subsequent role — but the current state of `experience.md` determines how much weight to place here.

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

**What to assess:** Does the company's core work involve problems that match the candidate's profile? Read `projects.md` and `skills.md` to understand the candidate's technical centre of gravity, then assess how well this company's engineering problems overlap.

**Why it matters:** The portfolio is the candidate's primary evidence of capability. A company whose day-to-day engineering problems resemble what the candidate already builds is a company where the portfolio converts most effectively in interviews and where the daily work builds on existing strength. A company doing work entirely outside the candidate's domain wastes this alignment.

**How to assess alignment:** Map the technologies, domains, and problem types from `projects.md` and `skills.md` against the company's engineering work:

- **Strong alignment:** Core product involves the same problem types, languages, or domains the candidate's projects demonstrate (e.g., if the portfolio shows low-latency systems work, a trading infrastructure company is strongly aligned)
- **Moderate alignment:** Problem types are adjacent — technically deep but in a different domain (e.g., payment processing, identity, fraud detection, security engineering)
- **Weak alignment:** Primarily application-layer work, no performance constraints, consulting/agency work, marketing technology, mobile development, or work described entirely in business terms with no technical substance

#### Growth Trajectory

**What to assess:** Is this company growing, stable, or declining? Are they hiring, expanding, or contracting?

**Why it matters:** Read `visa.md` for the candidate's visa timeline. A growing company is more likely to (a) actually hire at the candidate's level, (b) have budget for sponsorship when the time comes, (c) offer career progression as the company scales, and (d) still exist when the visa timeline demands it. A declining company may rescind offers, freeze hiring, or shut down the specific team after joining.

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

**What to assess:** Can and will this company sponsor the required visa when the candidate's current right-to-work expires? Read `visa.md` for the exact visa type, expiry date, and what sponsorship is needed.

**Why it matters:** This is not a soft preference — it is a hard constraint with a deadline. A company that cannot sponsor means leaving the UK or changing jobs within the visa window regardless of how good the role is. Companies that can and do sponsor are materially more valuable than those that theoretically could but have no track record.

**How to assess:**
- Check the [UK Sponsor Register](https://www.gov.uk/government/publications/register-of-licensed-sponsors-workers) — this is the definitive source. If they are listed, they have a licence.
- Company size matters: companies with 200+ employees almost always have the HR infrastructure for sponsorship. Companies with 10-30 employees may find the process burdensome.
- International hiring history: do they employ people from outside the UK/EU? Do job listings mention visa sponsorship?
- Sector patterns: finance, big tech, and defence/government contractors sponsor routinely. Small startups and agencies rarely do.

**The time factor:** Read `visa.md` for the remaining visa window. During that window, sponsorship is not needed — the candidate has unrestricted right to work. This means a company that is unlikely to sponsor is not automatically C-tier — it could still be a strong first job within the visa window, building CV signal before moving to a sponsor-capable employer. But it is a real cost, and the grade should reflect that the exit is forced.

#### Career Ceiling

**What to assess:** How far can an engineer progress at this company? Is there a path from entry-level to senior, staff, and principal?

**Why it matters:** Read `preferences.toml` for the candidate's long-term income and career targets. A company with a flat org structure where everyone is "Software Engineer" forever limits trajectory even if the day-to-day work is interesting.

**High ceiling indicators:**
- Multiple defined engineering levels (Junior → Mid → Senior → Staff → Principal or equivalent)
- Evidence of internal promotion (engineers who joined junior and are now senior/staff)
- Compensation bands that reach into the candidate's long-term target range
- IC (individual contributor) track that is parallel to management, not subordinate to it

**Low ceiling indicators:**
- Flat org with no levels — "we don't believe in titles"
- Engineering team too small to have meaningful levels (5-person team will not have a Staff role)
- Compensation capped at market median with no top-of-band for exceptional performers
- Only path to seniority is management — no IC leadership track

#### Company Stability

**What to assess:** Will this company exist and be hiring when the candidate needs it to?

**Why it matters:** Read `visa.md` — joining a company that folds within the visa window means another job search while burning precious visa time. Stability is not about being boring — it is about the company surviving long enough to deliver the career value that justified joining.

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
| **CV signal** | Would this company name impress future employers? | Matters most for a first job (read `experience.md` to determine). |
| **Location** | Office location? Hybrid policy? Commute? | Read `preferences.toml` for location requirements. |
| **Culture fit** | Engineering-led? Small teams? Autonomy? | Read `preferences.toml` and `interests.md` for culture preferences. |
| **Compensation** | Do they pay at or above market? | Hard to assess from outside, but Levels.fyi, Glassdoor, and Blind give signals. |

---

## Grade Definitions

Company grades are gentler than job grades — they assess whether a company is worth monitoring, not whether to apply right now. A B-tier company might produce an S-tier role.

**Grades should be conservative.** S is reserved for companies where the alignment is genuinely exceptional across almost every dimension. Most good companies are A-tier. Most decent companies are B-tier. S should feel rare — if you're grading 50 companies and 15 come out as S, your bar is too low.

| Grade | Meaning | Profile | Rough frequency |
|-------|---------|---------|-----------------|
| **S** | Exceptional — career-defining | Scores well on ALL three high-weight dimensions AND at least two medium-weight. Strong engineering reputation, clear technical alignment with the candidate's specific projects, actively growing, likely to sponsor, high career ceiling. Getting any relevant role here would be a career win. These are companies where the candidate's portfolio would genuinely impress interviewers. | ~10% of the universe. If more than 15% are S, re-examine. |
| **A** | Strong — actively monitor | Strong on two of three high-weight dimensions, acceptable on the third. Or strong on all three but with a notable medium-weight weakness. These are companies the candidate should actively pursue — they produce genuinely good roles. | ~25-30% |
| **B** | Decent — worth tracking | Has genuine relevance and at least one clear strength, but with notable weaknesses. Maybe the tech stack doesn't match, maybe the company is small, maybe the domain is adjacent rather than core. Still worth having in the database because a good role here beats no role. | ~35-40% |
| **C** | Marginal — archived | The connection to the candidate's profile is genuinely tenuous, OR the company has hard negatives (excluded sector, no engineering team, dissolved). Archived to prevent noise, preserved for dedup. | ~20-25% |

### Anti-inflation rules

These rules prevent the grade drift that occurs when every "good company" becomes S:

1. **Tech stack mismatch alone does NOT make a company C.** A large, well-funded company with strong engineering reputation and guaranteed sponsorship is at minimum B, even if they use Java instead of Rust. The tech stack is a medium-weight dimension, not a dealbreaker. Monzo (Go), Stripe (Ruby/Go), Bloomberg (C++) are all valuable employers regardless of language.

2. **Large UK employers with guaranteed sponsorship get a floor of B.** Companies like Monzo, Revolut, Starling, Checkout.com, Wise, and Adyen are massive employers with established engineering teams, proven sponsorship track records, and strong CV signal. They should never be C-tier unless they fall into an explicitly excluded sector. Even if their technical alignment is moderate, the sponsorship guarantee, brand recognition, and career ceiling make them worth monitoring.

3. **S requires genuine exceptionality, not just "good across the board."** To earn S, a company must have either (a) near-perfect technical alignment where the candidate's specific projects map directly to the company's core work, OR (b) an engineering reputation so strong that getting hired there would be transformative for the candidate's career regardless of the specific role. "Good company, grows, sponsors" is A-tier, not S.

4. **The candidate's career stage matters.** With no formal work experience, CV signal from the first employer is disproportionately important. A company with a strong engineering name (even if the tech stack isn't Rust) is more valuable than a tiny Rust startup with no brand recognition. First-job companies should be graded with this lens.

### The gap between B and C

This is the most consequential boundary. B companies stay in the active search pool — their jobs get fetched, evaluated, and shown in the TUI. C companies are archived and invisible. Getting this boundary wrong in either direction has real costs:

- **False B (should be C):** The job search wastes time fetching and evaluating roles from a company that was never going to produce a relevant hit. Low cost per company, but it accumulates.
- **False C (should be B):** A company with a genuinely good role gets archived, and the role is never seen. This is a missed opportunity that cannot be recovered.

**When in doubt between B and C, lean toward B.** The cost of monitoring one extra company is low. The cost of missing a good role is high. A company should only be C if you can articulate a clear reason why monitoring it would be a genuine waste — not just "their stack is different."

---

## Worked Examples

These examples demonstrate the full reasoning chain through every dimension. They are illustrative — each real company should be evaluated fresh based on current evidence, not pattern-matched to these examples.

**Note:** The examples below reference the candidate's profile generically. When grading real companies, use the actual profile data from `profile/` files.

### Example 1: Tier-1 Infrastructure Company (e.g. Cloudflare)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Exceptional.** One of the best engineering blogs in the industry. Major OSS projects. Engineers are individually well-known. Systems languages in production at scale. |
| Technical alignment | **Excellent.** Core product is network infrastructure — CDN, DNS, DDoS mitigation, edge compute. Performance-critical, systems-level, distributed. Assess alignment against the candidate's `projects.md`. |
| Growth trajectory | **Strong.** Publicly traded, consistent revenue growth, expanding product portfolio, actively hiring across engineering. |
| Sponsorship likelihood | **High.** Large company (3000+ employees), on the UK sponsor register, history of international hiring. |
| Career ceiling | **High.** Clear IC progression to Staff and Principal. Compensation at senior levels is competitive. |
| Stability | **High.** Profitable, public, diversified revenue. |

**Grade: S** — Exemplary across every dimension. Any relevant role here is worth pursuing aggressively.

### Example 2: Fintech Scaleup (e.g. Form3)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Good.** Known in the payments/fintech engineering community. Engineering blog exists but less prominent. |
| Technical alignment | **Good.** Payment infrastructure — technically deep, involves distributed systems, real-time processing, correctness-critical. Assess stack overlap against `skills.md`. |
| Growth trajectory | **Good.** Series C, expanding, actively hiring. |
| Sponsorship likelihood | **High.** 200+ employees, on the sponsor register, international team. |
| Career ceiling | **Moderate-good.** Multiple engineering levels, but ceiling may be lower than the candidate's long-term targets (check `preferences.toml`). |
| Stability | **Good.** Well-funded, revenue-generating, strong backers. |

**Grade: A** — Strong across most dimensions. Weaker brand recognition and potentially lower comp ceiling compared to S-tier keep it at A. A great role here would be a strong first job.

### Example 3: Small Infrastructure Startup (e.g. Coadjute)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Unknown/weak signal.** Small company, minimal public engineering presence. |
| Technical alignment | **Moderate.** Niche domain — involves some systems-level work but may be more application-layer. Assess against `projects.md`. |
| Growth trajectory | **Moderate.** Funded but small team. Hiring at a scale where adding one entry-level engineer is a significant decision. |
| Sponsorship likelihood | **Low-moderate.** Small company, unlikely to have established sponsorship infrastructure. |
| Career ceiling | **Low.** Team too small for meaningful levels. |
| Stability | **Moderate.** Funded but early-stage. |

**Grade: B** — Genuine technical work but weaknesses on reputation, sponsorship, and career ceiling accumulate. Worth monitoring because a well-scoped infrastructure role here could be excellent, but not a priority target.

### Example 4: Defence/Security Company with Clearance Complexity (e.g. Palantir)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Strong.** Known for hiring exceptional engineers and paying well. Engineering-led product development. |
| Technical alignment | **Excellent.** Large-scale data infrastructure, distributed systems, analytical platforms. Assess against `projects.md`. |
| Growth trajectory | **Strong.** Public company, growing revenue, actively hiring. |
| Sponsorship likelihood | **Complex.** Large company with HR infrastructure, on the sponsor register. However, many roles require security clearance — check `personal.md` and `military.md` for clearance eligibility based on nationality and residency. Need to filter for roles that do not require clearance. |
| Career ceiling | **High.** Clear engineering levels, competitive compensation, strong IC track. |
| Stability | **High.** Profitable, public, diversified. |

**Grade: S** — Despite the security clearance complication, the overall profile is exceptional. Clearance is a job-level filter (some roles require it, some do not), not a company-level disqualifier.

### Example 5: Top Quant Firm (e.g. XTX Markets)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Exceptional.** Tech-first market maker — technology is the product. Known for paying top-of-market and hiring the strongest engineers. |
| Technical alignment | **Excellent.** Low-latency trading infrastructure, matching engines, data pipelines, ML for strategy. Assess against `projects.md` — if the portfolio includes trading systems or matching engine work, this is a near-perfect match. |
| Growth trajectory | **Strong.** Profitable, expanding, actively hiring. |
| Sponsorship likelihood | **High.** International team, on the sponsor register. |
| Career ceiling | **Very high.** Quant-tech compensation is among the highest in the industry. Check against long-term targets in `preferences.toml`. |
| Stability | **High.** Massively profitable. |

**Grade: S** — One of the strongest possible matches if the candidate's portfolio includes relevant domain work. The only risk is that entry-level hiring is fiercely competitive.

### Example 6: AI Lab (e.g. Anthropic or equivalent)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Exceptional.** Research lab with world-class engineers. |
| Technical alignment | **Good-excellent.** ML infrastructure, distributed training systems, inference serving — all systems-level problems. Check `projects.md` for ML-adjacent work (RL implementations, ONNX experience, ML framework contributions). |
| Growth trajectory | **Strong.** Well-funded, rapidly expanding. |
| Sponsorship likelihood | **High.** International hiring is routine. |
| Career ceiling | **Very high.** AI infrastructure engineering compensation is among the highest. |
| Stability | **Good.** Well-funded. Market risk is low. |

**Grade: S** — AI infrastructure is a strong match for a systems engineering profile with ML depth. Entry bar is high but payoff is exceptional.

### Example 7: OSS-First Infrastructure Company (e.g. Grafana Labs)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Strong.** Major OSS company. Engineering culture is core to company identity. |
| Technical alignment | **Good.** Observability infrastructure — distributed systems, time-series databases, query engines, high-throughput data ingestion. Assess against `projects.md` and `skills.md`. |
| Growth trajectory | **Strong.** Well-funded, growing rapidly. |
| Sponsorship likelihood | **High.** Large global team, on the sponsor register. |
| Career ceiling | **Good-high.** Engineering-led company with clear levels. Comp may be slightly below top-tier firms. |
| Stability | **Good.** Revenue-generating, well-funded. |

**Grade: A** — Strong across the board, with genuine technical alignment. Slightly lower comp ceiling and less direct domain overlap compared to S-tier keep it at A. Excellent first-job environment.

### Example 8: Trading Infrastructure Consultancy (e.g. Adaptive Financial Consulting)

| Dimension | Assessment |
|-----------|-----------|
| Engineering reputation | **Good.** Known in the trading infrastructure community. Technically strong team. |
| Technical alignment | **Excellent.** Low-latency messaging, exchange connectivity, trading system infrastructure. Assess against `projects.md` for relevant domain work. |
| Growth trajectory | **Moderate.** Established company, steady growth. Consulting model means revenue is project-dependent. |
| Sponsorship likelihood | **Moderate.** Medium-sized company, consulting firms sometimes prefer candidates without sponsorship needs. |
| Career ceiling | **Moderate.** Consulting company structure may limit IC progression compared to product companies. |
| Stability | **Good.** Established, profitable. |

**Grade: A** — Excellent technical alignment, but consulting model introduces career ceiling and growth trajectory concerns.

---

## Boundary Cases

These examples demonstrate where the grade boundary lies and why a company falls on one side rather than the other.

### "Looks like an A but is actually a B" — Consumer Fintech

**Why it looks like A:** Well-known fintech brand, engineering blog, large engineering team, confirmed sponsor, growing. Good CV signal.

**Why it is B:** Technical alignment is moderate — engineering challenges are primarily application-layer (mobile banking, microservices, product features) rather than systems-level infrastructure. The tech stack is relevant but the day-to-day problems are not the kind of work the candidate's portfolio demonstrates deepest strength in (check `projects.md`).

**The principle:** Brand recognition does not compensate for misaligned technical depth. An A-tier company needs strong technical alignment, not just strong reputation.

### "Looks like a B but is actually an A" — Small Company with Perfect Technical Fit

Imagine a 50-person company building infrastructure in the candidate's primary language (check `skills.md`), well-funded, with engineers from top firms, an active engineering blog, and a clear IC ladder.

**Why it looks like B:** Small, relatively unknown, limited CV signal.

**Why it is A:** Technical alignment is near-perfect. Engineering reputation is strong for its size. Growth trajectory is good. The only weakness is CV signal, which is a low-weight tiebreaker, not a high-weight driver.

**The principle:** Company size and brand are tiebreakers, not primary drivers. A small company doing exactly the right technical work with strong engineers can outrank a large company doing tangentially relevant work.

### "Looks like an S but is actually an A" — Large Consultancy with Tech Division

**Why it looks like S:** Huge brand, definite sponsor, massive scale, good CV signal, stable.

**Why it is A (or even B):** Engineering culture in a consultancy is fundamentally different from a product company. Engineering is a cost centre, not a profit centre. Career progression is on a consulting ladder, not an engineering ladder. The work may be technically interesting on some engagements and mind-numbing on others.

**The principle:** Size, stability, and sponsorship do not override culture and alignment.

### "Looks like a C but is actually a B" — Pre-revenue Startup with Exceptional Technical Fit

A tiny startup building in the candidate's primary language and domain (check `projects.md` and `skills.md`), pre-revenue, seed-funded, with no sponsorship history.

**Why it looks like C:** Tiny, no revenue, sponsorship unlikely, no career ladder, survival uncertain.

**Why it is B:** Technical alignment is exceptional. The sponsorship weakness is real but mitigated by the current visa window (check `visa.md` — if there is time remaining on the current visa, the candidate can work here without sponsorship during that period). If the company survives, early employee signal is valuable.

**The principle:** A company with exceptional technical alignment gets the benefit of the doubt at the B/C boundary.

---

## Companies That Do Not Make the Cut

Some companies should never enter the database. These are not C-tier (marginal but preserved) — they are excluded entirely because tracking them has zero expected value.

| Reason | Examples | What to do |
|--------|----------|-----------|
| **Dead** | Website down, company dissolved, domain parked | Remove from potential.md with note "dead / dissolved" |
| **Acquired and absorbed** | No separate hiring, careers redirect to parent | Remove. If the parent company is interesting, track the parent instead |
| **No engineering team** | Purely sales, marketing, or business operations | Remove. "We're a tech company" on the website does not mean they have engineers |
| **Completely unrelated work** | Interior design firm, restaurant chain, law firm | Remove. Sometimes name collisions with tech companies cause false positives in discovery |
| **Too small to hire at candidate's level** | 3-5 person company with no funding | Remove. They are not going to hire someone who needs mentorship and onboarding |
| **Excluded sector** | Check `preferences.toml` for hard exclusions | Remove. These are hard exclusions, not judgement calls |

The distinction between "C-tier and archived" and "excluded entirely" matters for the database. C-tier companies have a row with grade='C' and status='archived' — they prevent re-discovery. Excluded companies are not inserted at all.

---

## How Company Grade Interacts with Job Grade

Company grade sets a monitoring priority. Job grade determines whether to apply. They are independent assessments — a B company can have an S job, and an S company can have an F job.

| Scenario | Company Grade | Job Grade | Why it makes sense |
|----------|:------------:|:---------:|-------------------|
| Top company, perfect role | S | SS | The dream case. Apply immediately with maximum effort. |
| Top company, wrong role | S | F | Great company hiring for a role outside the candidate's profile. Company is great, role is irrelevant. |
| Risky company, perfect role | B | S | Small startup with a founding engineer role perfectly matching the portfolio. Company is uncertain but the role is exceptional for growth. |
| Good company, mediocre role | A | C | Strong company with a legacy maintenance role. Company is good but the role is a dead end. |
| Unknown company, strong technical role | B | A | Small company with a systems engineer role matching the candidate's skills. Company lacks brand signal but the role itself is technically excellent. |

**The key insight:** Company grade determines whether jobs are fetched and shown to the user. Job grade determines whether to apply. A high company grade with no good roles wastes some search time but causes no harm. A low company grade that filters out a great role before it is ever seen is an unrecoverable loss. This is why the B/C boundary matters so much — it is the cutoff between "jobs are visible" and "jobs are invisible."

---

## Evidence Standards

Grade reasoning must be grounded in evidence you can actually find and verify. Different types of evidence have different reliability and availability.

### What to look for (and where)

| Signal | Where to find it | Reliability | If you can't find it |
|--------|-----------------|-------------|---------------------|
| Engineering reputation | GitHub repos, engineering blog, conference talks, Glassdoor engineering reviews | High | Note "no public engineering signals found" — this is a weak negative signal but not conclusive |
| Technical alignment | Company website, product docs, job descriptions, engineering blog posts | High | If the company's actual work is unclear from public sources, note the uncertainty |
| Growth trajectory | Crunchbase, press releases, LinkedIn headcount, careers page job count | Moderate | Many private/profitable companies have no public growth data. Do NOT assume declining — say "growth data unavailable" |
| Sponsorship | UK sponsor register (gov.uk), job listing text, company size | High (register), Moderate (inference) | Check the register. If not listed, note it but consider company size — large companies often sponsor even without being listed for the specific entity name |
| Career ceiling | Levels.fyi, Glassdoor reviews, LinkedIn profiles of engineers, job ladder in descriptions | Moderate | If progression data is unavailable, infer from company size and structure |
| Hiring activity | Careers page, ATS job count, LinkedIn Jobs | Moderate | "No visible open roles" is not the same as "not hiring" — many companies hire through referrals |

### The golden rule

**Absence of evidence is not evidence of absence.** A company with no public funding data might be bootstrapped and profitable. A company with no engineering blog might have excellent engineering culture internally. A company with no visible open roles might hire exclusively through referrals.

Grade based on what you CAN find:
- Positive evidence raises the grade
- Negative evidence (layoffs, dissolved on Companies House, "we don't sponsor" in listings) lowers the grade
- Absence of evidence is neutral — note it honestly but do not treat it as a negative signal

The exception: if you cannot find enough evidence to form ANY confident assessment, say so. A grade with acknowledged thin evidence ("B — limited public information; graded on technical alignment alone, which appears moderate based on product description") is more honest than a confident grade based on nothing.
