---
name: discover-companies
description: "Expands the Cernio company universe via parallel subagents (one per sector + one non-obvious-sources agent) finding companies through creative web search — VC portfolios, OSS affiliations, conference sponsors, engineering blogs, community hiring threads, funding announcements, sector registries. Each agent reads the profile, receives the existing universe for dedup, writes finds to `companies/discovery-{territory}.md`; orchestrator imports via `cernio import`. Obvious searches find obvious companies; value is indirect-signal discovery. Invoke on 'run a discovery', 'find me companies', 'expand the universe', 'discover companies in [sector]', 'explore [sector]', 'what companies should I be looking at', 'add more companies'. Not for resolving ATS (populate-db / resolve-portals), searching known-company jobs (search-jobs), grading companies (grade-companies), or evaluating jobs (grade-jobs). Use when the universe needs fresh additions."
---

# Discover Companies

Expands the Cernio company universe. Mass-market aggregators surface the same obvious names to everyone; this skill finds the 60-person startup doing brilliant infrastructure work, the company that just raised a Series B and is hiring its first engineers in a profile-aligned niche, the firm whose engineering blog describes exactly the kind of work the profile's flagship projects demonstrate. The universe is the foundation of every downstream step — the quality of companies tracked determines the quality of jobs found, evaluations written, and applications prepared.

Discovery runs parallel subagents by territory. One agent per sector (territories derived from the profile on this invocation, not a hardcoded list) plus one agent dedicated to non-obvious sources (source-type territory: VC portfolios, OSS contributors, conference sponsors, engineering blog rolls). Each agent reads the profile, receives the existing universe for deduplication, performs real web searches, and writes its finds to its own file. The orchestrator imports the files and the DB's unique website-URL constraint handles dedup.

---

## Mandatory Reads Before Discovery

| # | What | Evidence |
|---|---|---|
| 1 | **Every file in `profile/`** | Territory design was derived from the profile's actual skills / interests / preferences this invocation, not from a hardcoded sector list |
| 2 | **`references/search-strategies.md`** | Each dispatched subagent prompt embeds the full file verbatim — agents cannot read the skill's references and produce generic "top companies" lists without it |
| 3 | **The existing universe from `state/cernio.db` and `companies/potential.md`** | The discovery run produces no re-discoveries; every subagent received the known-companies list and uses it as dedup filter |

The profile is not cached in this skill or in any subagent prompt template. Every invocation reads `profile/` fresh — the territory design and relevance judgments depend on the current profile state.

---

## Workflow

### 1. Read the profile and the existing universe

Read every file in `profile/`. Build the lens through which discovery is filtered: skills, domain interests, sector preferences, geographic constraints, visa status, career direction. A company is worth discovering only if the profile's specific skills and interests would be genuinely valued there — not if the company is impressive in the abstract.

Query the existing universe:

```sql
SELECT name, website FROM companies;
```

Combine with `companies/potential.md` entries (discovered but not yet resolved). Website URL is the stable dedup key — company names vary across sources but domains are usually consistent. Pass this full list to every subagent as a skip filter.

### 2. Design the search territories from the profile

Territories are designed fresh each invocation from what the profile actually says. There is no fixed sector list.

- Profile emphasises distributed systems + data infrastructure → territories may be "distributed data platforms", "low-latency infrastructure", "storage and database engines".
- Profile emphasises biotech + computational chemistry → territories become "molecular simulation", "drug discovery tooling", "structural bioinformatics".
- Profile has breadth → more agents, narrower territories each.
- Profile is focused on one area → fewer agents with deeper mandates.

**The non-obvious-sources agent is always dispatched.** Its territory is defined by source type, not sector. It explores OSS contributor affiliations in profile-relevant repositories, conference sponsor lists for events aligned with the profile's domains, engineering blog rolls, community discussion threads, ecosystem-specific job boards. These sources catch companies that have no press coverage and appear on no curated lists — exactly the class of discovery generic tools miss.

### 3. Dispatch subagents in parallel

Each subagent prompt embeds verbatim:

- The **full text of `references/search-strategies.md`** — agents cannot read the skill's references; without this file they fall back on obvious searches.
- The **relevant slice of the profile** (skills, domains, interests that matter for the agent's territory) with a note that the full `profile/` was already read by the orchestrator.
- The **complete existing-universe list** (name + website) so the agent skips known companies.
- The **agent's specific territory** (sector name OR "non-obvious sources" with starting-source list derived from the profile).
- **Explicit instruction to use `WebSearch` and `WebFetch`** — agents must perform real web searches, not answer from training knowledge. Discovery from memory produces the well-known names every job board already surfaces. Real web searches produce the indirect-signal companies that make this skill valuable.
- **Output-file obligation:** write discoveries to `companies/discovery-{territory}.md` (e.g. `companies/discovery-trading.md`, `companies/discovery-non-obvious.md`). Agents must **not** all write to `companies/potential.md` — that produces merge conflicts and loses per-agent provenance.

Use standard subagents (not worktree-isolated). Discovery is web research + independent file writes — shared working-directory state is not a risk.

### 4. Per-company output format

Each discovered company captures:

```markdown
### [Company Name]
- **Website**: [URL]
- **Location**: [HQ city / country]
- **What they do**: [1-2 sentences — specific product/service, not a generic sector label]
- **Why relevant**: [Specific connection to the profile — name the projects, technologies, or domains from `profile/` that align. Generic phrases fail this standard.]
- **Source**: [Where the agent found them — named portfolio page, dated thread, specific repository, contributor profile URL]
- **Discovered**: [Date]
```

**"Why relevant" standard.** Every entry names at least one specific element from the profile (a project, a skill, a domain interest) that connects to the company's work. Unacceptable: *"interesting tech company"*, *"does relevant engineering"*, *"systems engineering focus"*. Acceptable: *"Core product involves lock-free data structures and low-latency networking — directly overlaps with Nyquestro's matching engine architecture and NeuroDrive's real-time simulation."*

The source field is specific enough to be verifiable. Not *"web search"* — *"Founders Fund portfolio page, 'infrastructure' category, visited 2026-04-20"* or *"Rust Foundation member list"* or *"ICFP 2024 gold sponsor"*.

### 5. Import the files

After all subagents complete:

```bash
cargo run -- import companies/discovery-{territory}.md
```

Run once per discovery file. The DB's unique-constraint on `website` handles dedup automatically — a second import of the same URL is silently skipped.

### 6. Present results for review

Tell the user:

- How many companies each agent found, itemised by territory.
- How many were new vs duplicates per territory (reported by `cernio import`).
- Notable finds worth highlighting — particularly strong profile alignments or surprising sources (name the source explicitly).
- Which sources were most productive (signal for the next discovery run).

### 7. Declare what was skipped

Close the run with a "What I did not do" section covering: territories that were designed but not dispatched (with reason); subagents that returned empty or near-empty results (what they searched, why the well was dry); sources the orchestrator considered but did not include in the subagent starting-source lists; companies surfaced by subagents but rejected before import (with the reason — duplicate, not-profile-aligned, insufficient evidence, dead website). If nothing was skipped or deferred, say so explicitly — silence is the Claude abstention pattern.

---

## Reference Loading

**Mandatory-core — read at skill invocation every time:**

- `references/search-strategies.md` — how to find companies generic aggregators miss: 8 strategies (VC portfolios, OSS signals, conference sponsors, hiring threads, funding announcements, engineering blogs, sector deep-dives, "who else" expansion) plus guidance on inventing new strategies and combining them.

---

## Inviolable Rules

1. **Every subagent prompt embeds the full text of `references/search-strategies.md`, verbatim.** Without it, agents produce generic "top companies" lists that add no value over an Indeed search. This is the single most common failure mode of this skill.
2. **Profile is read fresh every invocation.** Territory design depends on current profile state. No cached snapshots.
3. **Each subagent writes to its own `companies/discovery-{territory}.md` file.** No shared writes to `companies/potential.md` — provenance and merge-conflict cost matter.
4. **The existing universe is queried and passed to every subagent** as a dedup filter. No re-discoveries.
5. **Subagents use `WebSearch` / `WebFetch` tools** — explicit instruction in every dispatch prompt. Discovery from training-data memory fails the skill's purpose.
6. **Territories are derived from the profile on this invocation**, not from a hardcoded sector list.
7. **The non-obvious-sources agent is always dispatched.** It is the agent whose source choices most often surface genuinely new companies.
8. **Every discovered company's "why relevant" names at least one specific profile element by name.** Generic relevance fails the standard.

---

## Iterative Use

Discovery runs repeatedly across sessions. Each run:

- Queries the current universe first and skips known companies
- Focuses on areas the user asked to expand, or on profile-emphasised sectors not yet explored
- Picks up where previous runs left off — if the last run covered one sector end-to-end, this run explores others
- Surfaces genuinely new finds, not repackaged known content

Over time, the universe grows to hundreds of companies, each with a profile-grounded reason for being there.

---

## What Discovery Is Not

- Resolving ATS portals / finding job-board URLs — that is populate-db / resolve-portals
- Searching for specific job listings — that is search-jobs
- Evaluating whether a specific role fits — that is grade-jobs
- Filtering companies by visa sponsorship capability — sponsorship status is not always public, companies make exceptions, filtering at discovery loses opportunities

Discovery is broad by design. The funnel narrows at every subsequent step.

---

## Quality Checklist

Each item is an obligation with a concrete evidence slot, not a subjective self-rating. An item that cannot be evidenced in the agent's output is either unmet and surfaced under step 7 "What I did not do," or the skill has not finished.

- [ ] **Profile read fresh this invocation** — cite the tool call that read each file in `profile/`.
- [ ] **Existing universe queried** — cite the SQL query run and the row count returned; the set of `(name, website)` passed to subagents is identifiable in the transcript.
- [ ] **Territory derivation shown** — the session transcript names the specific profile elements (skill, domain interest, sector preference) that produced each territory. "Designed from profile" without citation fails this item.
- [ ] **Non-obvious-sources agent dispatched with explicit source list** — the agent's starting-source list is reproduced in the dispatch prompt and is derived from specific profile technologies / domains (not a generic "OSS contributors" placeholder).
- [ ] **Every subagent prompt embeds `references/search-strategies.md` verbatim** — the prompt contents for every dispatched subagent are visible in the transcript; full file is present, not paraphrased.
- [ ] **Every subagent's prompt embeds the deduplication list** — the list of known `(name, website)` pairs appears in the prompt; subagents cannot re-discover companies already in the universe.
- [ ] **Each subagent's output cites WebSearch queries used** — per-company rows cite the actual query text or the fetched URL, not "via web search". Per-agent aggregate: at least one non-trivial WebSearch query per 3-5 companies claimed.
- [ ] **Per-company "why relevant" names a specific profile element** — a project, a skill, or a domain interest by name (e.g. "Nyquestro", not "my trading-system work"). Generic relevance fails this item.
- [ ] **Per-company "source" field is verifiable** — a named page URL with date, a specific repository URL, an identifiable contributor profile URL, or a dated thread URL. "Web search" or "Google" fails this item.
- [ ] **Per-company "what they do" describes product/service specifically** — names the product, the target user, and the technical surface. Sector-label answers ("fintech company") fail this item.
- [ ] **Each subagent's discoveries landed in its own `companies/discovery-{territory}.md` file** — verify by listing the new files after the run; no shared writes to `companies/potential.md`.
- [ ] **`cernio import` run once per discovery file** — cite the import commands executed and the per-file dedup counts reported by each.
- [ ] **Step 7 "What I did not do" declaration emitted** — names specific skipped territories, empty-result subagents, or rejected companies, or explicitly states "no deferrals or skipped territories".
