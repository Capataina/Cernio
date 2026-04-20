# Location Evaluation Rubric

> **Purpose:** a reasoning framework for evaluating whether a given city-and-country pair is a good destination for Caner's career, visa situation, lifestyle preferences, and long-term trajectory. This is not a scoring formula. It's a list of factors grouped by importance, to be used as the scaffolding for reasoning-based evaluation.
>
> **What this rubric is for:** personal geographic evaluation for a 24-year-old Turkish software engineer with a 2027 UK visa deadline, a Rust/systems/quant/AI-infrastructure portfolio, and specific lifestyle preferences recorded elsewhere in the profile. It is **not** a generic "best cities in the world" ranking.
>
> **What this rubric is not:** an arithmetic formula. There is no "if lifestyle fit is X and career is Y then grade is Z" rule. The rubric is a thinking tool — the evaluator weighs factors across the three tiers and reaches a reasoned verdict, consistent with the rest of the grading system which is explicitly reasoning-based (see `context/notes/grading-rubric.md`).

---

## How to use this rubric

### Evaluate city + country as a paired unit

Some factors are determined at the country level (visa, tax, national political direction, legal stability, PR pathway), some at the city level (firm density, urban aesthetic, local safety, café culture, nightlife), and some are hybrid. The evaluator produces a verdict for a specific city-country pair — e.g. "Munich, Germany" — not for a city or country alone. Same country, different cities can diverge wildly: Frankfurt ≠ Berlin ≠ Munich, NYC ≠ SF ≠ Austin, Dubai ≠ Sharjah.

When evaluating multiple cities in the same country, walk through the country-level factors once and the city-level factors separately for each city.

### Trajectory is first-class, not an afterthought

Every factor gets both a **current state** read and a **trajectory** read across three horizons:

- **Short (1–3 years):** announced projects, pending policy changes, visible pipeline, confirmed near-term developments
- **Medium (5–7 years):** demographic trends, economic direction, political trajectory, ongoing regeneration projects
- **Long (10–15 years):** structural shifts, climate, geopolitical realignment, generational change

A city that is 10/10 now but declining to 6/10 over five years is worse than a city at 7/10 now rising to 9/10. Vector matters as much as scalar.

**Example of why trajectory matters:** UK nightlife current state is mid-tier, but the trajectory is upward (DCMS Night Time Economy Commissioner, Agent of Change principle protecting venues from new-resident noise complaints, active licensing reform). German nightlife current state is higher than the UK in absolute terms (Berlin especially), but the trajectory is downward (Clubsterben, venue closures, festival cancellations, gentrification pressure). Same current quality, opposite vectors — the evaluation should reflect the vector.

### Weight tiers are about importance, not arithmetic

- **Tier 1 — dominant factors**: can make or break the entire case. If a city fails badly on a Tier 1 factor, the other tiers rarely recover it.
- **Tier 2 — meaningful factors**: shift the verdict but don't override Tier 1. A city that is mixed on Tier 2 is usually still viable if Tier 1 is strong.
- **Tier 3 — fine-tuning factors**: tiebreakers when two city-country pairs score similarly on Tier 1 and 2.

There is no numeric aggregation. The evaluator reasons across all three tiers and produces a verdict that weighs everything in proportion to its tier.

---

## Tier 1 — Dominant factors (deal-makers and deal-breakers)

### 1. Visa accessibility for a Turkish national at entry level

**Level:** country
**Why Tier 1:** without this, nothing else matters. If a Turkish fresh CS graduate cannot realistically land a work visa, the city is not a real option regardless of how good the firms are.

**What "good" looks like:**
- Visa path explicitly designed for skilled migrants (Blue Card, Tier 2/Skilled Worker equivalents)
- Salary threshold achievable by graduate roles
- Processing time measured in weeks, not months
- Dependents allowed
- Path to permanent residency is clear

**What "bad" looks like:**
- Lottery-based systems (US H-1B has sub-30% acceptance)
- Points-based systems biased against non-local education or non-local work experience
- Quotas that get filled early in the year
- Employer must prove no local candidate exists
- "Skilled migrant" pathway exists on paper but is bureaucratically slow or arbitrary in practice

**Trajectory considerations:**
- Is the country tightening or loosening immigration policy?
- Are there political movements actively campaigning for restrictions?
- Are there announced reforms in the pipeline?

### 2. Target firm density in the candidate's sectors

**Level:** city
**Why Tier 1:** career trajectory is capped by the number of firms in your actual target sectors that exist in the city. Few firms means few chances, slow career, capped upside. This is the dominant career factor at the entry-level stage.

**What the target sectors are for this profile:**
- Quantitative trading / HFT firms
- Modern fintech infrastructure (payments, trading systems, settlement)
- AI infrastructure and frontier ML companies
- Systems engineering (compilers, runtimes, databases, distributed systems)
- Rust-in-production companies
- Modern developer tooling

**What "good" looks like:**
- Double-digit count of target-sector firms with London/NYC-level engineering depth
- Multiple graduate programmes or entry-level pathways
- Active hiring markets where you could switch jobs without leaving the city
- Network density — other engineers in similar firms to learn from

**What "bad" looks like:**
- Can count target firms on one hand
- All firms are satellite offices of foreign HQs, not primary engineering centres
- No graduate programmes — experienced hires only
- Specialist sector with nowhere to move laterally

**Trajectory considerations:**
- Are firms moving in or out of the city?
- What does the startup pipeline look like — young firms that will be mature in 5 years?
- Tax, regulatory, or political trends affecting firm location decisions

### 3. Urban aesthetic match

**Level:** city
**Why Tier 1:** Caner has explicit, consistent, and repeated preferences about urban aesthetic. These are not cosmetic — they are where he spends every day for years. Cities that fail this grind him down in ways that compound over time. See `profile/lifestyle-preferences.md` for the detailed anchor points.

**What "good" looks like:**
- Mixed-scale urban fabric: tall towers combined with human-scale streets and smaller buildings in the same district
- Integrated greenery that is part of the streetscape rather than sectioned off into token parks
- Recent regeneration — districts designed and built in the last 15–20 years
- Walkability as part of daily life (take laptop to café, walk home from dinner, evening decompression walks)
- A futuristic energy — the sense that the city is visibly building forward

**What "bad" looks like:**
- Low-rise historic districts with no modernity (the aesthetic opposite of his preference)
- Pure skyscraper canyons with no mixed-scale or integrated green (visually modern but aesthetically wrong)
- Car-dependent sprawl with no walkability
- Cities that feel frozen — preserving the past rather than building the future

**Trajectory considerations:**
- What regeneration projects are under way or announced?
- Is the city building upward or maintaining the existing fabric?
- New districts being planned?

### 4. Safety and civic order

**Level:** city, with a country-level floor
**Why Tier 1:** unsafe streets simultaneously invalidate every other lifestyle preference. No evening decompression walks. No late-night café work. No 10pm gym sessions. No weekend exploration. No solo dinner out. Safety is a precondition for the whole personal-infrastructure stack, not a tertiary axis among equals.

**What "good" looks like:**
- Safe to walk home from a venue at midnight without elevated alertness
- Low violent crime, low aggressive property crime, low street disorder
- Clean, maintained public realm
- Civic order that makes the city feel dignified rather than chaotic

**What "bad" looks like:**
- Areas with visible disorder, aggressive begging, drug use, or known avoidance zones
- High street crime, frequent petty theft, pickpocketing endemic
- Urban decay — cities where the walkable infrastructure has collapsed

**Trajectory considerations:**
- Is the crime rate rising or falling?
- Are specific neighbourhoods becoming less or more safe?
- Policing and enforcement policy direction

### 5. Political and legal stability (10–15 year outlook)

**Level:** country
**Why Tier 1:** this is a long-term relocation, not a gap year. A country that is currently excellent but institutionally fragile over a decade is a worse bet than one that is currently mediocre but institutionally solid. Turkey's political trajectory is part of why the candidate is leaving Turkey in the first place — do not repeat the mistake by moving somewhere with comparable long-term political risk.

**What "good" looks like:**
- Robust institutions (judiciary, civil service, central bank independence)
- Peaceful transfers of power
- Stable immigration policy not weaponised by changing governments
- Low risk of sudden authoritarian or xenophobic turn
- Legal protections for foreign nationals that do not depend on executive discretion

**What "bad" looks like:**
- Polarised politics with institutional capture risk
- Anti-immigration parties with meaningful electoral support
- Rule of law under sustained pressure
- Economic or currency instability that could devalue savings
- Weaponised immigration policy (e.g. Brexit-style sudden status changes)

**Trajectory considerations:**
- Where is the political centre moving?
- What do the upcoming 5–10 years of elections look like?
- Is the country trending toward or away from openness?

---

## Tier 2 — Meaningful factors (shift the verdict without overriding Tier 1)

### 6. Nightlife and active social scene

**Level:** city, with country-level influence
**What "good" looks like:** an active, energetic evening and weekend social scene. Live music, bars, clubs, a restaurant scene with atmosphere, festivals. The kind of city where going out is a real, visible part of daily life rather than everyone going home at 7pm.

**Trajectory notably matters here** — the UK nightlife revival vs German Clubsterben example is canonical.

### 7. Salary × cost-of-living ratio

**Level:** city
**What "good" looks like:** effective take-home after rent, food, transport, and realistic lifestyle expenses leaves meaningful disposable income. A high nominal salary that is consumed entirely by housing costs (Dublin, Amsterdam, SF) is worse than a moderate nominal salary in an affordable city.

### 8. Tax regime for high earners

**Level:** country
**Why it matters:** the stated career goal is £500k+ income trajectory. Over 20 years, the gap between a 0% tax jurisdiction and a 50% tax jurisdiction is compound-level life-changing. This is a meaningful factor even though it is not Tier 1.

**Anchor rates (as of mid-2020s):**
- UK: 45% top marginal
- Germany: 45–47.5%
- Netherlands: 49.5% + 30% ruling for skilled migrants
- US: 37–50% depending on state (no state income tax in TX, FL, WA, etc.)
- Switzerland: 22–40% depending on canton
- Singapore: 22%
- UAE: 0%
- Ireland: 40% plus USC and PRSI

### 9. Secular public culture

**Level:** hybrid (country + city)
**What "good" looks like:** religion is not in daily public life in a way that creates friction for an agnostic atheist. This does not mean the country must be irreligious — it means the public realm does not broadcast religious markers in an intrusive way.

### 10. Café culture and laptop-workable working environments

**Level:** city
**Why it matters:** working from cafés, coworking spaces, and libraries is core to how Caner operates day-to-day (see `profile/interests.md`). A city where café culture is small, closed, or hostile to laptop work removes a significant operating layer.

### 11. Path to permanent residency duration

**Level:** country
**Why it matters:** given the August 2027 UK visa deadline, the speed at which a new country converts to permanent residency changes whether a move solves the problem or just defers it. A 10-year PR path is much worse than a 2–3 year path.

### 12. Access to frontier technology as a user

**Level:** city
**What "good" looks like:** the city is a launch market for cutting-edge consumer technology — Waymo-style autonomous vehicles, latest payment infrastructure, modern healthcare technology, rapid adoption of new hardware. Caner values experiencing this tech, not just building it. A city where the frontier arrives years late fails this factor regardless of how good the office is.

### 13. Gym and fitness infrastructure

**Level:** city
**Why it matters:** strength training and martial arts are core hobbies. A city with serious premium gym options (Third Space-class environments) is meaningfully better than one where the best available gym is a basic chain.

### 14. English accessibility in daily life

**Level:** hybrid (country and city)
**Why it matters:** English is the candidate's strongest language after Turkish. German is conversational A2/B1. Everything else is zero. A city where English is sufficient for daily life (not just work) is meaningfully easier than one where local-language fluency is assumed.

### 15. Climate tolerance

**Level:** city
**What "good" looks like:** a climate the candidate can actually live in. Preferences: cold is fine, snow welcome, thunderstorms enjoyed. Heat is tolerable with heavy AC infrastructure but not without. Grey drizzle (London, Dublin, Seattle) is fine.

**Climate change consideration:** over 10–15 years, some currently-tolerable climates may become hostile (Gulf heat, South Asian humidity, Mediterranean summer extremes). Flag this where relevant.

---

## Tier 3 — Fine-tuning factors (tiebreakers within a tier)

### 16. Integration quality and xenophobia baseline

**Level:** country and city
How the local population actually treats long-term migrants rather than how the government policy reads on paper. Some countries are welcoming to migrants; some make you feel like a permanent outsider even after 20 years.

### 17. Housing market depth

**Level:** city
Can you actually find a flat, or is it a blood sport? Dublin, Amsterdam, parts of SF have become rental blood sports. London is manageable with effort. Berlin is still manageable but tightening. Tokyo is easy relative to demand.

### 18. International airport connectivity

**Level:** city
Number of direct flights, hub quality, frequency of service to key destinations (including Istanbul for family visits even though the candidate states this is not a factor for grading).

### 19. Food and cuisine diversity

**Level:** city
Availability of diverse cuisine for a candidate who cooks actively and enjoys dining out. Turkish cuisine accessibility is a plus but not a requirement.

### 20. Healthcare system quality and accessibility

**Level:** country
Matters more over 10–15 years than it does at 24. Still worth flagging because long-term relocations should consider this.

### 21. Time zone overlap for international collaboration

**Level:** city
Matters if working with teams across Europe, Americas, or Asia. Not a driver but a tiebreaker.

### 22. Currency stability over 10–15 years

**Level:** country
A country whose currency is likely to devalue significantly over 10–15 years is a worse savings destination than one with a stable currency. Relevant for long-term wealth building.

---

## Mechanical constraints the evaluator must factor in

These are hard facts about the candidate, not preferences:

- **Turkish national, no dual citizenship.** Cannot currently obtain UK SC/DV clearance, US clearance, French DGSE-adjacent clearance, or equivalents. Excludes clearance-gated employers. Some destinations treat Turkish nationals differently under bilateral agreements.
- **UK Graduate visa expires August 2027.** After that, Skilled Worker sponsorship or an alternative visa pathway is required to continue in the UK. The clock is the dominant forcing function on the geographic strategy.
- **Zero years professional work history.** The candidate substitutes a strong project portfolio (5 flagship Rust projects plus open source contribution), but roles requiring "3+ years of experience" are mechanically out of reach. This matters because many countries' skilled migrant schemes require experience thresholds.
- **BEng Computer Science, 2:2 classification, University of York.** The 2:2 grade is a filter at some firms and programmes — relevant where a 2:1 floor is enforced.
- **Languages: Turkish native, English fluent, German A2/B1.** Everything else is zero.

---

## What the evaluator should not do

- **Do not produce an arithmetic formula.** No "Tier 1 = 0.6, Tier 2 = 0.3, Tier 3 = 0.1" weightings. Reasoning-based evaluation throughout.
- **Do not exclude cities preemptively.** Every city and country should be evaluated on its merits relative to the rubric, even ones that seem obviously wrong. If a city genuinely fails, the evaluation should say why. The value of running this analysis is the reasoning, not the conclusion.
- **Do not treat lifestyle factors as overriding.** Career ceiling and visa access dominate. Lifestyle fit modulates but does not override.
- **Do not pattern-match to stereotypes.** Research actual current 2026 state of each city. Do not rely on dated impressions — "Berlin is poor but sexy" is a decade out of date; "London is unsafe" is wrong for most of Central London; "Dubai is hot and tax-free" is a surface read that misses the frontier-tech adoption and career infrastructure questions.
- **Do not defer to prior conclusions in the profile files.** The profile files contain analysis from previous sessions — tier rankings, "rejected" verdicts, and framings. Treat these as noise and reach your own verdicts independently from the raw data.
