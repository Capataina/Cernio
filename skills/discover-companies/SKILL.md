# discover-companies

Discovers companies that do work aligned with the user's profile and adds them to the company universe. Use when the user wants to expand their company universe, says "run a discovery", "find me companies", "let's explore [sector]", "what companies should I be looking at", or when the universe needs broadening in a specific direction. Not for resolving ATS portals, searching job listings, or evaluating individual roles.

---

## Mandatory reads — do not proceed without completing these

**STOP. Before executing any part of this skill, you MUST read these files in full:**

1. **Every file in `profile/`** — all of them, no exceptions. The profile determines what to search for, which sectors matter, and how to assess relevance.
2. **`references/search-strategies.md`** — the complete guide to creative, non-obvious company discovery. This file is what makes discovery produce valuable results instead of generic top-10 lists.

**When dispatching discovery agents:** You MUST embed the FULL TEXT of `references/search-strategies.md` directly in each agent's prompt. Agents cannot read files from the repo — if you don't include it, they work without it, and they will produce generic, obvious results. This is the single most common failure mode of this skill.

Also embed in each agent's prompt:
- The relevant slice of the profile (skills, domains, interests that matter for their sector)
- The complete list of already-known companies (from DB + potential.md) for deduplication
- Explicit instruction to use WebSearch and WebFetch tools — agents must perform real web searches

**Do not proceed to agent dispatch until all mandatory reads are complete.**

---

## Why this skill exists

The company universe is the foundation of everything downstream — job search, evaluation, and application tracking are only as good as the companies in the system. Mass-market aggregators surface the same obvious names to everyone. The value of discovery is finding the companies that generic tools miss: the 60-person startup doing brilliant infrastructure work, the company that just raised Series B and is hiring its first engineers in a niche the profile cares about, the firm whose work maps perfectly onto the profile but would never appear on a "top companies" list.

Discovery is the most important skill in Cernio. The companies it finds determine the quality of every subsequent step.

---

## How discovery works

### 1. Read the profile and existing universe

Before searching, understand what to look for and what is already known.

**Read every file in `profile/`** to build a complete picture of the user — skills, domains, interests, location preferences, visa constraints, experience level, and career direction. The profile determines which sectors to explore, which technologies matter, which geographies to target, and how to assess relevance. A company is worth discovering if the user's skills and interests would be genuinely valued there — not just if the company is impressive in the abstract.

This step is non-negotiable. The profile is the lens through which all discovery is filtered. Without reading it, agents cannot distinguish relevant companies from irrelevant ones, and the skill collapses into generic "top companies" lists that add no value.

**Read the existing universe from the database and `companies/potential.md`.** Query all company names and website URLs from `state/cernio.db` (these are resolved or bespoke companies already in the system), plus any entries still in `companies/potential.md` (discovered but not yet resolved). Discovery should not re-discover any of these. Website URL is the stable dedup key — company names vary across sources but the domain is usually consistent. Pass this list to every agent so they can skip known companies.

### 2. Design the search space from the profile

The profile determines the search territories. Read the user's skills, domain interests, sector preferences, and career direction, then divide the search space into independent territories that cover the profile's surface area.

The orchestrator designs this division fresh each time based on what the profile actually says — there is no fixed list of sectors. If the profile emphasises distributed systems and data infrastructure, those become territories. If it emphasises biotech and computational chemistry, those become territories instead. If the profile has breadth across several domains, create more agents. If it is focused on one area, create fewer agents with deeper mandates.

One agent should always be dedicated to non-obvious, indirect sources — the kind of places where companies leave traces of their work without advertising as employers. This agent's territory is defined by source type rather than sector, and the sources it explores should be chosen based on the technologies and domains in the profile.

**Each agent gets:**
- The relevant slice of the profile (skills and domains that matter for their sector)
- The list of already-known companies (to avoid duplicates)
- The search strategies reference file for guidance on discovery approaches
- A clear territory: which sector or source type they own
- **Explicit instruction to use WebSearch and WebFetch tools.** Agents must perform real web searches, not answer from training knowledge. Discovery from memory produces "top companies everyone already knows" — the well-known names that every job board already surfaces. The skill's value comes from finding companies through live web searches: VC portfolio pages, conference sponsor lists, contributor profiles, engineering blog posts, community threads, curated lists. An agent that does not search the web is not doing discovery.

**Each agent writes to its own file.** Every agent must write its discoveries to a separate file named `companies/discovery-{territory}.md` (e.g., `companies/discovery-trading.md`, `companies/discovery-ai-ml.md`, `companies/discovery-non-obvious.md`). Agents must NOT all write to `companies/potential.md` — that causes merge conflicts and makes it impossible to see which agent found what.

**Each agent also returns:**
- A brief summary of how many companies were found
- Notes on which sources were most productive and which searches yielded unexpected finds

### 3. Import and deduplicate

After all agents complete, the orchestrator imports each file separately using `cernio import companies/discovery-{territory}.md`. The database handles deduplication automatically via its unique constraint on company website URLs — there is no need for manual deduplication. If two agents find the same company, the second import is silently skipped.

### 4. Review

After import, present the combined results to the user: how many companies each agent found, how many were new vs duplicates, and any notable finds worth highlighting.

---

## Output format

Each discovered company should capture:

```markdown
### [Company Name]
- **Website**: [URL]
- **Location**: [HQ city/country]
- **What they do**: [1-2 sentences — specific about their actual product/service, not generic sector labels]
- **Why relevant**: [Specific connection to the profile — name the projects, technologies, or domains from `profile/` that align. "Interesting tech company" is not acceptable. "Core product involves lock-free data structures and low-latency networking — directly overlaps with Nyquestro's matching engine architecture and NeuroDrive's real-time simulation" is the standard.]
- **Source**: [Where the agent found them — specific and verifiable: a named portfolio page, a dated thread, a specific repository or contributor profile]
- **Discovered**: [Date]
```

The "why relevant" field matters. A company without a clear connection to the profile should not be in the universe. The reasoning also helps later when evaluating jobs — it provides context for why the company was worth tracking.

**Quality standard for "Why relevant":** Every entry must name at least one specific element from the profile (a project, a skill, a domain interest) that connects to the company's work. Generic relevance statements like "does interesting engineering" or "relevant to systems engineering" are not acceptable — they add no information. The connection must be concrete enough that someone reading it understands exactly why this company matters for THIS specific candidate.

---

## Agent dispatch guidelines

**Critical: each agent must receive the full content of `references/search-strategies.md` embedded in their prompt.** This is not optional. The search strategies reference teaches agents HOW to discover companies creatively — through VC portfolios, open source signals, conference sponsors, engineering blogs, funding announcements, and indirect traces. Without it, agents default to obvious web searches that produce the same results as every job board. The reference file is what makes this skill valuable.

**Use standard subagents, not worktree-isolated ones.** Discovery agents are doing web research and writing to a shared result set. They do not modify repo files independently — they return structured results to the orchestrator.

**Each agent should be self-sufficient.** Give it everything it needs in the prompt: the relevant profile slice, the existing company list, the search strategies, and its territory. The agent should not need to read additional files or ask for clarification.

**Encourage depth over breadth within each territory.** Each agent should explore thoroughly within its territory — following leads, expanding from one good find to its competitors and partners, digging into indirect sources — rather than skimming the surface of everything.

**The non-obvious sources agent is critical.** This agent looks in places the sector-specific agents will not: open source contributor affiliations, conference sponsor lists, engineering blog rolls, community discussion threads, ecosystem-specific job boards. The specific sources it explores should be derived from the profile's technologies and domains. These indirect signals catch companies that have no press coverage and appear on no curated lists.

**Encourage creative sourcing.** Agents should invent search strategies beyond what the reference file suggests. The reference file teaches discovery approaches — it is not an exhaustive list of sources. An agent that finds a company through a method nobody anticipated is doing the best work.

---

## Search strategy

The `references/search-strategies.md` file contains guidance on discovery approaches — how to look sideways, how to use indirect signals, and how to expand from one good find to many. Every agent should read it before beginning their search.

The core principle: **obvious searches find obvious companies.** Searching "[sector] companies [location] [year]" returns the same list every aggregator already has. The skill's value comes from creative, indirect discovery — finding companies through the traces they leave in the ecosystem rather than through top-10 lists.

Agents should treat the reference file as a starting point for their thinking, not a constraint on their methods. The best discoveries come from sources and search strategies the skill author never anticipated.

---

## Iterative use

Discovery is designed to be run repeatedly across sessions. Each run should:

- Check the existing universe first and skip known companies
- Focus on areas the user wants to expand, or on sectors the profile emphasises that have not been explored yet
- Pick up where previous runs left off — if the last run covered one sector thoroughly, this run can explore others
- Surface genuinely new finds, not repackage what is already known

Over time, the universe grows from dozens to hundreds of companies, each with a clear reason for being there.

---

## What discovery is not

Discovery finds companies. It does not:

- Resolve ATS portals or find job board URLs — that is the resolution step
- Search for specific job listings — that is the search scripts
- Evaluate whether a specific role fits the profile — that is the evaluation step
- Filter companies by visa sponsorship capability — sponsorship status is not always public, companies can make exceptions, and filtering at discovery time loses opportunities

Discovery is broad by design. The funnel narrows at every subsequent step.

---

## Quality checklist

Before presenting results, verify:

- [ ] Every agent read `profile/` before searching — discoveries are grounded in the actual profile, not generic sector assumptions
- [ ] The existing universe was queried from `state/cernio.db` and `companies/potential.md` before agents began, and no already-known company appears in the results
- [ ] Every discovered company has a specific, concrete "why relevant" that connects to something in the profile — not just "they're a tech company"
- [ ] The source is specific enough to be verifiable — a named page, dated thread, or identifiable repository, not just "web search"
- [ ] The results include non-obvious finds, not just the well-known names everyone already knows about
- [ ] Each agent explored beyond the first page of search results — depth over breadth
- [ ] The "what they do" field describes their actual product or service, not just their sector label
- [ ] Companies span a range of sizes and stages — not exclusively large established firms or exclusively tiny startups
- [ ] Search territories were derived from the profile, not from a hardcoded list of sectors
