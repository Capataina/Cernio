# discover-companies

Discovers companies that do work aligned with the user's profile and adds them to the company universe. Use when the user wants to expand their company universe, says "run a discovery", "find me companies", "let's explore [sector]", "what companies should I be looking at", or when the universe needs broadening in a specific direction. Not for resolving ATS portals, searching job listings, or evaluating individual roles.

---

## Why this skill exists

The company universe is the foundation of everything downstream — job search, evaluation, and application tracking are only as good as the companies in the system. Mass-market aggregators surface the same obvious names to everyone. The value of discovery is finding the companies that generic tools miss: the 60-person startup doing brilliant ML infrastructure work, the fintech that just raised Series B and is hiring its first systems engineers, the healthcare AI company whose work maps perfectly onto the profile but would never appear on a "top tech companies" list.

Discovery is the most important skill in Cernio. The companies it finds determine the quality of every subsequent step.

---

## How discovery works

### 1. Read the profile and existing universe

Before searching, understand what to look for and what's already known.

**Read `profile/`** to understand the user's skills, domains, interests, and preferences. The profile determines which sectors to explore and how to assess relevance. A company is worth discovering if the user's skills and interests would be genuinely valued there — not just if the company is impressive in the abstract.

**Read the existing universe from the database and `companies/potential.md`.** Query all company names and website URLs from `state/cernio.db` (these are resolved or bespoke companies already in the system), plus any entries still in `companies/potential.md` (discovered but not yet resolved). Discovery should not re-discover any of these. Website URL is the stable dedup key — company names vary across sources but the domain is usually consistent. Pass this list to every agent so they can skip known companies.

### 2. Divide the search space and dispatch parallel agents

Discovery is too broad for a single agent to handle well. The orchestrator should divide the search space into independent territories and dispatch parallel agents, each exploring a different slice.

**Divide by sector and source type.** The exact division depends on the profile and the user's current priorities. A typical split might look like:

```
Orchestrator
  ├── Agent: AI / ML infrastructure / MLOps
  ├── Agent: Fintech / payments / banking infrastructure
  ├── Agent: Trading systems / quant / exchange infrastructure
  ├── Agent: Healthcare AI / biotech / computational biology
  ├── Agent: Developer tools / compilers / databases / infrastructure software
  ├── Agent: Non-obvious sources (GitHub orgs, conference sponsors,
  │          engineering blogs, HN/Reddit threads, Rust ecosystem)
  └── ... additional agents as the search space demands
```

The user may also request targeted discovery: "find me healthcare AI companies in London" or "who's building Rust infrastructure tooling?" In that case, the scope is narrower and fewer agents may be needed.

**Each agent gets:**
- The relevant slice of the profile (skills and domains that matter for their sector)
- The list of already-known companies (to avoid duplicates)
- The search strategies reference file for guidance on where to look
- A clear territory: which sector or source type they own
- **Explicit instruction to use WebSearch and WebFetch tools.** Agents must perform real web searches, not answer from training knowledge. Discovery from memory produces "top companies everyone already knows" — the well-known names that every job board already surfaces. The skill's value comes from finding companies through live web searches: VC portfolio pages, conference sponsor lists, GitHub contributor profiles, engineering blog posts, HN threads, curated lists. An agent that doesn't search the web is not doing discovery.

**Each agent returns:**
- A list of discovered companies in the output format defined below
- Brief notes on which sources were most productive and which searches yielded unexpected finds

### 3. Deduplicate and merge

When agents return, the orchestrator:
- Deduplicates across agents (multiple agents may independently find the same company)
- Deduplicates against the existing universe
- Merges results into a single list
- Presents the combined findings to the user for review

### 4. Write to the universe

After the user reviews the discoveries (they may want to discuss some, remove others, or ask for more detail), write accepted companies to `companies/potential.md`. These are now in the system and will be picked up by the resolution step later.

---

## Output format

Each discovered company should capture:

```markdown
### [Company Name]
- **Website**: [URL]
- **Location**: [HQ city/country]
- **What they do**: [1-2 sentences — specific about their actual product/service, not generic sector labels]
- **Why relevant**: [Why this company fits the profile — which skills, domains, or interests align]
- **Source**: [Where the agent found them — specific: "Sequoia portfolio page", "HN Who's Hiring March 2026", "contributor to the `tokio` crate"]
- **Discovered**: [Date]
```

The "why relevant" field matters. A company without a clear connection to the profile shouldn't be in the universe. The reasoning also helps later when evaluating jobs — it provides context for why the company was worth tracking.

---

## Agent dispatch guidelines

**Use standard subagents, not worktree-isolated ones.** Discovery agents are doing web research and writing to a shared result set. They don't modify repo files independently — they return structured results to the orchestrator.

**Each agent should be self-sufficient.** Give it everything it needs in the prompt: the relevant profile slice, the existing company list, the search strategies, and its territory. The agent should not need to read additional files or ask for clarification.

**Scope each agent to produce 10-30 companies.** This is a guideline, not a hard limit — an agent exploring a rich sector might find 40, while one exploring a niche might find 8. The point is that each agent should explore thoroughly within its territory rather than skimming the surface of everything.

**The non-obvious sources agent is critical.** This agent looks in places the sector-specific agents won't: GitHub organisation pages, open source contributor affiliations, conference sponsor lists, engineering blog rolls, job board threads, Rust/ML/systems community discussions. These indirect signals catch companies that have no press coverage and appear on no curated lists.

---

## Search strategy

The `references/search-strategies.md` file contains detailed guidance on where to look and how to look sideways. Every agent should read it before beginning their search.

The core principle: **obvious searches find obvious companies.** "Best fintech companies UK 2026" returns the same list every aggregator already has. The skill's value comes from creative, indirect discovery — finding companies through the traces they leave in the ecosystem rather than through top-10 lists.

---

## Iterative use

Discovery is designed to be run repeatedly across sessions. Each run should:

- Check the existing universe first and skip known companies
- Focus on areas the user wants to expand ("let's look at healthcare AI this time")
- Pick up where previous runs left off — if the last run covered fintech thoroughly, this run can explore other sectors
- Surface genuinely new finds, not repackage what's already known

Over time, the universe grows from dozens to hundreds of companies, each with a clear reason for being there.

---

## What discovery is not

Discovery finds companies. It does not:

- Resolve ATS portals or find job board URLs — that's the resolution step
- Search for specific job listings — that's the search scripts
- Evaluate whether a specific role fits the profile — that's the evaluation step
- Filter companies by visa sponsorship capability — sponsorship status is not always public, companies can make exceptions, and filtering at discovery time loses opportunities

Discovery is broad by design. The funnel narrows at every subsequent step.

---

## Quality checklist

Before presenting results, verify:

- [ ] Every discovered company has a specific, concrete "why relevant" — not just "they're a tech company"
- [ ] The source is specific enough to be verifiable ("Balderton portfolio page" not "web search")
- [ ] No company already in `companies/*.md` appears in the results
- [ ] The results include non-obvious finds, not just the well-known names everyone already knows about
- [ ] Each sector agent explored beyond the first page of search results — depth over breadth
- [ ] The "what they do" field describes their actual product or service, not just their sector label
- [ ] Companies span a range of sizes and stages — not exclusively large established firms or exclusively tiny startups
