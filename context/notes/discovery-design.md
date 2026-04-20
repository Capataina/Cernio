# Discovery Design Decisions

Discovery is Cernio's most complex and most important skill. The companies it finds determine the quality of everything downstream.

---

## Broad discovery, filtering later

Discovery finds companies that do work aligned with the profile. It does not eliminate companies for visa status, size, funding stage, or any other hard constraint — that filtering happens at the job evaluation stage.

A 10-person startup with no public sponsor licence is still worth discovering if they do interesting systems work. Companies can make exceptions on sponsorship, and "can sponsor" is not always publicly disclosed.

**Why:** Premature filtering at discovery time means missing opportunities. Discovery casts wide; the funnel narrows at job search and evaluation.

---

## Creative, not formulaic

Searching "best UK fintech 2026" catches obvious names everyone knows. The real value is finding the 60-person company doing brilliant ML infrastructure work with zero press coverage.

Creative sources include:
- GitHub orgs building in Rust/ML/infrastructure
- Contributors to specific Rust crates (tokio, axum, reqwest)
- Niche conference sponsors (QCon London, RustConf, EuroRust)
- HN "Who's hiring" threads with insider context
- Engineering blog posts about topics matching the profile
- LinkedIn alumni patterns — where do people at interesting companies come from?
- Non-obvious sector matches (healthcare AI, climate tech, biotech infrastructure)

The creativity in *where to look* and *how to look sideways* is the skill's entire value. Full search strategies documented in `.claude/skills/discover-companies/references/search-strategies.md`.

---

## Heavy parallelisation

A single agent doing 50+ web searches sequentially is too slow and context-limited. Discovery uses a team of parallel agents, each exploring a different sector or source type:

- An orchestrator reads the profile, divides the search space by sector/source, dispatches agents
- Each agent explores its assigned territory independently
- Agents return structured results; the orchestrator deduplicates against the existing universe and merges

Typical sectors: AI/ML infrastructure, fintech/payments, trading systems/quant, healthcare AI/biotech, developer tools/compilers/databases, non-obvious sources (GitHub orgs, conference sponsors, engineering blogs).

**Lesson from first run (2026-04-07):** Agents that don't use web search tools default to answering from training knowledge, producing lists of well-known companies everyone already knows. The systems/infra agent that *did* use web searches found non-obvious companies (Flux Computing, Vypercore, lowRISC via CompilerJobs and LLVM Discourse). Every discovery agent must be explicitly instructed to use WebSearch and WebFetch — this is the single biggest quality lever for discovery results.

---

## Separate from resolution and enrichment

Discovery answers "which companies should we be watching?" It outputs company name, website, what they do, why they're relevant, and where the agent found them. It does not resolve ATS portals or gather detailed enrichment data.

Enrichment data can be noted at discovery time if readily available, but gathering it is not discovery's responsibility. Resolution happens via the `populate-db` skill.

---

## TrueUp as a reference model

TrueUp (trueup.io) is a tech job meta-aggregator that enriches company profiles with growth signals (funding, headcount, Glassdoor/Blind sentiment, layoff data, hiring velocity). Tracks 460K+ jobs across 51K+ company profiles. Publishes a weekly "Hot 200" ranking.

Cernio's discovery is a personalised version of TrueUp's company intelligence layer — profile-aware from the start, conversational, and focused on finding companies TrueUp would miss. We don't replicate their automated data pipeline; we compensate with creative search strategies and human-in-the-loop judgment.
