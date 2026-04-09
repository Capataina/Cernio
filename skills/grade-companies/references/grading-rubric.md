# Company Grading Rubric

A rubric for evaluating whether a company is worth monitoring for jobs, and at what priority. Company grading is a thinking exercise, not a checklist — the grade should emerge from careful reasoning about what this company means for the candidate's career, not from mechanically scoring dimensions.

**Important:** This rubric references the candidate's profile throughout. All profile facts must come from reading the files in `profile/` — never from hardcoded values in this document.

---

## Table of Contents

1. [How to Grade a Company](#how-to-grade-a-company)
2. [The Core Questions](#the-core-questions)
3. [Analytical Dimensions](#analytical-dimensions)
4. [Grade Definitions](#grade-definitions)
5. [Cross-referencing and Relative Grading](#cross-referencing-and-relative-grading)
6. [Career-Stage Context](#career-stage-context)
7. [Common Grading Errors](#common-grading-errors)
8. [Worked Examples](#worked-examples)
9. [Evidence Standards](#evidence-standards)

---

## How to Grade a Company

Grading happens in four steps. Do not skip any.

**Step 1: Answer the core questions.** These force you to think about what actually matters — not in the abstract, but for this specific candidate at this specific career stage. Write 2-3 sentences per question. This is where the real reasoning happens.

**Step 2: Evaluate against the analytical dimensions.** The dimensions add precision and catch things the questions might miss. A company might feel right but have a hidden sponsorship problem, or feel wrong but actually have excellent technical depth you didn't consider.

**Step 3: Cross-reference questions and dimensions.** Do they agree? If the questions say "I'd be proud to work here, they can hire me, I'd grow" but the dimensions say "C tier because tech stack mismatch" — the dimensions are wrong, not the questions. If the dimensions say "strong on everything" but the questions reveal "the work wouldn't actually be engaging" — that matters too. When they conflict, reason through why and decide which signal is more reliable.

**Step 4: Relative comparison.** Compare this company against others at the same grade and adjacent grades. Does it genuinely belong where you've put it? Would the candidate really be indifferent between all the companies at this tier?

The grade reasoning written to the database should reflect this process — not just a conclusion, but the thinking that led to it.

---

## The Core Questions

For every company, answer these four questions. These are not optional warm-up exercises — they ARE the evaluation. The dimensions in the next section provide analytical support, but these questions are where the grade is determined.

### 1. Would the candidate be proud to say they work here?

This is the brand, reputation, and signal question. It captures something the dimension framework misses: the real-world social and professional value of having this company on your CV.

Think about it concretely: if the candidate got a job here and told friends, family, former classmates, and future interviewers, would they be impressed? Would it open doors? Would it be a story worth telling?

This is not about prestige snobbery — it's about the practical reality that early-career engineers are judged partly by where they work. Read `experience.md` — with no formal work history, the first employer name carries enormous weight.

A company doesn't need to be a household name to score well here. "I work at a company building the UK's payment infrastructure" is impressive even if the company name isn't famous. But "I work at Apple" is impressive without any further explanation.

### 2. Could they actually hire the candidate and keep them?

This is the practicality question. It doesn't matter how good a company is if they can't or won't hire someone at this career stage.

Consider:
- Do they hire graduates or entry-level engineers? Or is it senior-only?
- Can they sponsor a Skilled Worker visa when the Graduate visa expires? (Read `visa.md` for the timeline.)
- Are they in the UK, or do they have UK-based roles?
- Are they actually hiring, or is the careers page empty?

A company that checks every other box but can't sponsor is a ticking clock — the candidate would need to leave within the visa window. That's not necessarily a dealbreaker (great experience within the window can justify the move), but it's a real cost that should be reflected.

### 3. Would the candidate grow here?

This is the career trajectory question. Not just "is the company growing" (that's the dimension) but "would the candidate specifically become a better, more employable, higher-value engineer by working here?"

Consider:
- Would they work alongside strong engineers who push them?
- Would the problems be genuinely challenging or routine?
- Is there a path from junior to senior to staff?
- Would the experience compound — building skills and knowledge that make the next job even better?
- Or would they stagnate, learning one narrow thing with no upward trajectory?

A 2-year stint at a company with brilliant engineers and hard problems is worth more than 5 years at a company doing routine work, even if the routine company has a better name.

### 4. Is the work something the candidate would find engaging?

This is the motivation and alignment question. Not "does the tech stack match" (that's a dimension detail) but "would the candidate wake up interested in what they're building?"

Read `projects.md` and `interests.md` — what does the candidate actually choose to build when nobody is making them? That reveals genuine interests, not just skills on paper.

A company whose core problems resemble what the candidate builds for fun is a company where they'll thrive. A company whose work is technically impressive but in a domain the candidate finds boring is a company where they'll coast.

This doesn't mean every company needs to be in a "passion" domain. "I find this interesting enough and the other factors are excellent" is a perfectly valid answer. But "I would actively dread this work" is a signal that matters.

---

## Analytical Dimensions

These dimensions provide structure and precision. Use them to support and validate the reasoning from the core questions — not to override it.

### High weight

| Dimension | What to assess | Why it matters |
|-----------|---------------|---------------|
| **Engineering reputation** | Is this company known for strong engineering? Eng blog, OSS, conference talks, Glassdoor engineering reviews, known engineers. | Read `experience.md` — with limited formal experience, the first employer's name compensates for credential gaps. The higher the reputation, the more doors it opens. |
| **Technical alignment** | Does the company's core work involve problems that match the portfolio? Map `projects.md` and `skills.md` against their engineering work. | The portfolio is the primary evidence of capability. A company whose problems resemble the candidate's projects is a company where the portfolio converts in interviews. |
| **Growth trajectory** | Growing, stable, or declining? Funding, hiring activity, headcount trends, product launches. | Read `visa.md` — growth means they're more likely to hire at entry level, budget for sponsorship, and offer career progression. |

### Medium weight

| Dimension | What to assess | Why it matters |
|-----------|---------------|---------------|
| **Sponsorship capability** | Can and will they sponsor? Check the UK sponsor register, company size, international team, job listing mentions. | Read `visa.md` — mandatory from August 2027. Not a soft preference, a hard constraint with a deadline. |
| **Career ceiling** | Path from entry to senior/staff/principal? IC track parallel to management? Compensation bands reaching long-term targets? | Read `preferences.toml` — first job sets the trajectory. A flat org caps growth regardless of how interesting the work is. |
| **Company stability** | Will they exist when the candidate needs them to? Profitable, well-funded, strong market position? | Read `visa.md` — company folding during the visa window wastes precious time. |

### Low weight (tiebreakers)

| Dimension | What to assess |
|-----------|---------------|
| **Tech stack match** | Does the company use technologies from `skills.md`? This is a TIEBREAKER, not a driver. Languages are learned in months. |
| **CV signal** | Would the name impress future employers? Matters most for a first job. |
| **Location** | Office in London/Cambridge? Hybrid policy? |
| **Culture fit** | Engineering-led? Small teams? Autonomy? |

**Critical note on tech stack:** Tech stack match is deliberately low-weight. A company using Go, Java, C++, or Python is not penalised for not using Rust. What matters is whether the engineering *problems* align (distributed systems, low-latency, data infrastructure) — not whether they solve those problems in the same language. A graduate engineer switching languages takes weeks, not years.

---

## Grade Definitions

| Grade | What it means | How it should feel |
|-------|--------------|-------------------|
| **S** | Exceptional. Getting any relevant role here would be a career win. | "I would drop everything to apply here." The questions all have strong answers AND the dimensions confirm it. ~10% of the universe. |
| **A** | Strong. Worth actively pursuing. | "I'd be genuinely excited about an offer here." Most questions have strong answers with maybe one area of uncertainty. ~25-30%. |
| **B** | Decent. Worth tracking. | "I'd consider it — depends on the specific role." Some clear strengths but notable weaknesses. A good role here beats no role. ~35-40%. |
| **C** | Marginal. Low probability of relevant roles. | "Probably not, but I wouldn't delete it." Tenuous connection to the profile, or hard negatives present. ~20-25%. |

**S requires genuine exceptionality.** "Good company, grows, sponsors" is A-tier. S means either near-perfect technical alignment (the candidate's projects map directly to the company's core work) OR an engineering reputation so strong that getting hired there is transformative regardless of the specific role.

**C is for genuine marginality.** A company should only be C if you can articulate a clear reason why monitoring it is likely a waste — an excluded sector, no engineering team, no relevance to the profile at all. "Their stack is different" is not sufficient for C. "They're a marketing agency with no backend engineers" is.

---

## Cross-referencing and Relative Grading

**This step is mandatory.** Do not write grades to the database without completing it.

### After grading a batch

1. **Compare within each tier.** Look at all companies you've put at the same grade. Do they genuinely belong together? Would it make sense to tell the candidate "these are all equally worth pursuing"? If a trillion-dollar tech company and a 10-person unfunded startup are both B, something is wrong.

2. **Compare across adjacent tiers.** For every company near a boundary, ask: "Is this genuinely less valuable than everything in the tier above?" If a B-tier company would clearly be a better career move than an A-tier company, adjust.

3. **The "which offer would you take" test.** For any two companies at different grades, imagine the candidate has a graduate SWE offer from both. Would they take the higher-graded one? If the answer consistently contradicts the grades, the grades are wrong.

4. **Sanity check.** Scan for obvious anomalies: a well-known employer at C, a tiny startup with no sponsorship at S. These aren't automatic corrections — but each one demands re-examination and explicit justification.

### When grading in parallel across agents

Each agent grades its batch independently, but the orchestrator MUST do a cross-batch comparison before writing to the database. Pull the top 5 and bottom 5 from each agent's output and verify they make sense relative to each other.

---

## Career-Stage Context

Read `experience.md` — the candidate has no formal work experience. This fundamentally changes the evaluation:

- **CV signal matters more than at any other career stage.** The first employer's name IS the professional credential. This doesn't mean every big company is S-tier, but it means brand recognition is a genuinely important factor, not a tiebreaker.
- **Sponsorship is not a nice-to-have.** Read `visa.md` — it becomes mandatory. Companies that solve this constraint are materially more valuable.
- **Tech stack is the least important factor.** A graduate engineer learns a new language in weeks. What matters is the class of problems, the quality of mentorship, and the career trajectory — not whether they write Rust or Go.
- **"Consumer product" is not "consumer-facing role."** Backend infrastructure at Spotify, Monzo, or Uber is systems engineering. The preference exclusion targets customer-facing roles (consulting, support, sales engineering), not companies whose product has end-users.

---

## Common Grading Errors

Every one of these has occurred in production and caused real damage:

**Grading a major employer as C because the tech stack doesn't match.** Monzo was graded C because it uses Go instead of Rust. This is wrong. A graduate SWE at Monzo with guaranteed sponsorship, strong CV signal, and deep backend infrastructure work is a strong career move regardless of language. Tech stack is a tiebreaker, not a primary driver.

**Treating all companies in a tier as equivalent.** Amazon and a 5-person unfunded startup cannot both be B. If they are, the grades are not calibrated. Use relative comparison to catch this.

**Over-indexing on domain alignment.** The preferred sectors in `preferences.toml` are soft preferences, not hard exclusions. A strong role at a healthcare AI company or a climate tech infrastructure company can be A-tier. Only the explicitly excluded sectors (gambling, adtech, consumer-crypto) justify downgrading on sector alone.

**Conflating "consumer product" with "consumer-facing role."** Spotify's backend is systems engineering. Uber's pricing engine is real-time infrastructure. The product being consumer-facing does not make the engineering role consumer-facing.

**Ignoring the practical reality of sponsorship.** A brilliant 10-person startup that can't sponsor is a job with a hard expiry date. That doesn't make it C — the experience within the visa window might be valuable — but the grade should honestly reflect this constraint rather than ignoring it.

---

## Worked Examples

These demonstrate the full reasoning process — questions first, dimensions second, grade as conclusion.

### Example: Cloudflare (→ S)

**Q1 — Proud to work here?** Yes, unambiguously. Cloudflare is one of the most respected engineering companies in the world. "I work at Cloudflare" opens every door.

**Q2 — Could they hire and keep?** Yes. Large company, London office, established graduate hiring, confirmed Skilled Worker sponsor. No barriers.

**Q3 — Would the candidate grow?** Yes. World-class engineers, extremely challenging problems (edge computing at global scale), clear IC progression to Principal. The engineering depth is exceptional.

**Q4 — Engaging work?** Yes. They rewrote their core proxy from NGINX to Rust — Rust in production at massive scale. CDN, edge compute, DNS, DDoS mitigation are performance-critical, systems-level, distributed problems. Direct alignment with Nyquestro and NeuroDrive's systems thinking.

**Dimensions confirm:** Engineering reputation exceptional, technical alignment excellent (Rust + infrastructure), growth strong (profitable, public, expanding), sponsorship guaranteed, career ceiling very high.

**Grade: S.** Every question has a strong answer. Dimensions confirm. This is the archetype of S.

### Example: Monzo (→ A)

**Q1 — Proud to work here?** Yes. Monzo is a household name in UK tech. Strong engineering brand with a well-known engineering blog and open culture. "I work at Monzo" carries real weight.

**Q2 — Could they hire and keep?** Yes. Large UK bank, guaranteed Skilled Worker sponsor, established graduate/entry hiring. No barriers.

**Q3 — Would the candidate grow?** Yes. Strong engineering culture, technically deep backend (Go, distributed systems, real-time financial processing at scale). Good mentorship signals. Clear engineering levels.

**Q4 — Engaging work?** Moderately. The backend infrastructure involves distributed systems and financial transaction processing — connecting to Nyquestro's domain. But it's consumer banking product infrastructure rather than pure systems/trading/compilers — the core domains the candidate is most passionate about. "Interesting enough, with excellent other factors" rather than "this is exactly what I want to build."

**Dimensions confirm:** Engineering reputation strong, technical alignment moderate (Go not Rust, banking not trading, but distributed systems at scale), growth strong, sponsorship guaranteed, career ceiling good.

**Grade: A.** Three strong question answers, one moderate. Strong across dimensions except tech stack (Go) and domain (banking vs trading). The brand signal, sponsorship, and engineering depth make this solidly A. Not S because the domain isn't a core passion — but absolutely worth pursuing.

### Example: Small Rust Startup with No Brand (→ B)

**Q1 — Proud to work here?** Mixed. Nobody outside the Rust community would know the name. But "I'm building a database engine in Rust" is a compelling story to tell, even without the company name carrying weight.

**Q2 — Could they hire and keep?** Uncertain. 15 employees, sponsorship capability unknown, may not have HR infrastructure for visa processing. Within the Graduate visa window they could hire, but what happens after August 2027?

**Q3 — Would the candidate grow?** Potentially excellent. Small team with strong engineers means direct mentorship and high-impact work. But no career ladder, no structured progression, and the company's survival is uncertain.

**Q4 — Engaging work?** Highly. Building a database engine in Rust — direct alignment with Nyquestro's systems work, Cernio's data layer, and Xyntra's compiler interests. The work itself is exactly what the candidate would choose to do for fun.

**Dimensions:** Technical alignment excellent (Rust + database), engineering reputation unknown, growth uncertain, sponsorship uncertain, career ceiling limited by size.

**Grade: B.** The work alignment is exceptional but the practical factors (brand, sponsorship, stability) create real risk. Worth monitoring — if a role appears that's genuinely compelling, the technical fit might justify accepting the practical trade-offs. But not A because the risks are too significant for a first job.

---

## Evidence Standards

Grade reasoning must be grounded in evidence you can actually find and verify.

| Signal | Where to find it | If you can't find it |
|--------|-----------------|---------------------|
| Engineering reputation | GitHub, eng blog, conference talks, Glassdoor | "No public engineering signals" — weak negative but not conclusive |
| Technical alignment | Website, product docs, job descriptions, eng blog | If unclear, note uncertainty |
| Growth trajectory | Crunchbase, press releases, LinkedIn headcount | "Growth data unavailable" — do NOT assume declining |
| Sponsorship | UK sponsor register, job listings, company size | Check register. Large companies usually sponsor even if not explicitly listed |
| Career ceiling | Levels.fyi, Glassdoor, LinkedIn engineer profiles | Infer from size and structure |

**Absence of evidence is not evidence of absence.** A company with no public funding data might be bootstrapped and profitable. Grade on what you CAN find. Positive evidence raises the grade. Negative evidence lowers it. Absence is neutral.
