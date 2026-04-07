# Cernio — Project Notes

> Design decisions, preferences, and lessons from sessions. Read this to understand *why* things are the way they are.

---

## Session: 2026-04-07 — Initial Setup and Vision Alignment

### Decision: Collaborative session model, not automated pipeline

The original README described Cernio as an automated daily pipeline (`cernio refresh`). This was revised in the first session. Cernio is a **collaborative tool** — the user launches Claude Code, they discuss what makes sense to do, and Claude orchestrates scripts and evaluates results as part of a back-and-forth conversation.

**Why:** The user wants to be involved in every decision. They tell Claude what they applied to, Claude suggests whether to discover new companies or fetch open positions, they decide together. The tool serves the user's judgment, not replaces it.

**Implication:** There is no single "run everything" command. Scripts are invoked dynamically based on what the session needs.

---

### Decision: Scripts handle volume, Claude handles judgment

Rust scripts are **parameterised tools** for combinatorial work — scanning hundreds of ATS boards for dozens of search terms, probing slug patterns, fetching job JSON. They are generic: no hardcoded addresses, no hardcoded search terms, no profile awareness.

Claude handles judgment — reading job descriptions, comparing against the structured profile, assessing fit, and explaining reasoning.

**Why:** A script can check 100 Greenhouse boards x 50 title patterns = 5000 combinations in seconds. Claude cannot do that economically. But Claude can read 50 resulting job descriptions and assess fit against a nuanced profile — something a keyword script cannot do well.

**Implication:** Scripts must be designed as generic, reusable tools with parameterised inputs. They should work for any ATS, any search terms, any set of companies. The intelligence lives in the conversation, not in the scripts.

---

### Decision: TUI is a real-time dashboard, not a standalone browser

The ratatui TUI shows live state as it happens — results appearing from script runs, evaluation status updating as Claude reads job descriptions (pending → evaluating → fit/no fit), user actions (watching, applied, rejected). It is not a tool the user launches independently to browse pre-computed results.

**Why:** The user wants to follow everything in real time — from company discovery to evaluation to export. Staring at a markdown file being modified is a poor experience. The TUI makes the collaborative process visible.

**Implication:** The TUI must watch a shared data store for changes and update reactively. The data contract between scripts, Claude, and the TUI is the most important interface to get right.

---

### Decision: Markdown export is user-triggered, not automatic

The "these are the jobs you should apply to" markdown file is only generated when the user confirms results in the TUI and explicitly triggers export (e.g. pressing a key). It is not produced automatically by a pipeline.

**Why:** The user reviews and confirms everything before it becomes an actionable list. Automatic export would produce lists the user hasn't vetted.

---

### Decision: Teaching mode removed from CLAUDE.md

The original CLAUDE.md supported teaching and implementation modes. Teaching mode was removed — this project will never need it. Sessions go straight to implementation work.

---

### Decision: README is editable

The original CLAUDE.md treated README.md as read-only ("suggest updates, do not edit directly"). This was changed — the README is a living document that should be updated as decisions change and hardcoded assumptions are replaced. The project is evolving and the README must evolve with it.

---

### Decision: Skills are conversational, not CLI-only

The user should be able to say "run a discovery" and Claude routes to the correct skill. No requirement to use CLI syntax like `claude --skill discover-companies`. The same applies to all project-specific skills.

---

### Decision: Profile folder read on every startup

The `profile/` folder contains the user's complete structured profile — everything that would be asked in an interview or job application. Claude reads it at the start of every session so it has full context about who the user is, what they're looking for, and what constraints apply.

**Why:** The profile is the source of truth for evaluation. Without it loaded, Claude cannot assess job fit accurately. It also means the user never has to re-explain their background.

---

### Decision: Main projects have readable context/ folders

Caner's four main projects — **NeuroDrive**, **Nyquestro**, **Aurix**, and **Image Browser** (PinterestStyleImageBrowser) — each have a `context/` folder maintained by the same skill system. When asked, Claude can fetch or read these to get up-to-date architectural and implementation details without Caner having to explain.

**Implication:** When evaluating job fit or updating the profile, Claude can ground assessments in actual code and architecture rather than relying solely on what's written in `projects.md`.

---

### Decision: Profile should be auto-updatable from repo links

Future capability: given a GitHub repo link, Claude (or a parameterised script) should be able to scrape the repo — README, `context/` folder, code inspection — and produce a complete `projects.md` entry with accurate tech stack, proficiency assessment, and relevant skill additions. Re-running it on an existing project updates the entry.

**Why:** Manual profile maintenance is friction. The projects are the source of truth for what Caner can do. Reading them directly produces more accurate and detailed entries than manual writing.

**Implication:** A GitHub scraping script is a natural early addition to Cernio's toolkit — parameterised, takes a repo URL, dumps structured output that Claude can reason over.

---

### Note: agent-skills repo is the upstream skill framework

Caner authored the skill system that Cernio's CLAUDE.md and skills are derived from. The repo `Capataina/agent-skills` contains the full skill library (feature pipeline, refactor pipeline, test pipeline, maintenance skills), the principal-engineer personality template, and 95KB of research on writing effective agent skills. The `upkeep-context`, `upkeep-learning`, `project-research`, and `code-health-audit` skills referenced in Cernio's CLAUDE.md all originate there.

---

### Decision: Project-specific skills live in this repo, not in agent-skills

The `agent-skills` repo contains universal, reusable skills (upkeep-context, code-health-audit, etc.). Cernio's skills — discovery, resolution, enrichment, profile scraping — are specific to this project and live in `skills/` within this repo.

**Why:** These skills are tightly coupled to Cernio's data model, filesystem layout, and workflow. They don't generalise to other projects. Keeping them here means they evolve with the project and don't pollute the universal skill library.

**Open questions:** Whether skills should share universal scripts or each have their own, how scripts and skills relate, and exact skill boundaries are all deferred — decisions will be made as skills are designed.

---

### Decision: Claude acts as career coach, not just job finder

Cernio's scope extends beyond discovering and evaluating jobs. Claude should actively track patterns across job descriptions to identify what's strong and what's missing in the profile. If 50 jobs all mention Kubernetes and the profile has zero Kubernetes experience, that's actionable intelligence — not just a filter miss.

**Why:** The better the profile fits the market, the more jobs pass evaluation. Improving the profile is a force multiplier on everything else Cernio does. Entry-level candidates especially benefit because small additions (a certification, an open source contribution, a new tool in an existing project) can open entire categories of roles.

**Implication:** `profile/portfolio-gaps.md` tracks strengths, gaps, and closure opportunities. Claude updates it as a byproduct of job evaluation — every batch of evaluated jobs produces pattern observations. Gap closure recommendations should be concrete and prioritised: "add a Dockerfile to Nyquestro" beats "learn Docker."

---

### Decision: Everything is dynamic and evolves over time

The profile, the resume, the preferences, the company universe — all of it changes as projects grow, new contributions land, new skills are acquired, and market patterns shift. Nothing is static. The system should treat every artefact as a living document that may need updating at any session.

**Implication:** When Claude reads the profile at startup, it should note anything that looks stale or inconsistent with what it knows from scraping repos or evaluating jobs. Proactive suggestions to update are welcome.

---

### Note: Resume uses four main projects

The current resume (LaTeX, stored as `profile/resume.md`) leads with the tinygrad open source contribution, then four projects: Image Browser, Aurix, NeuroDrive, and Nyquestro. Other projects in `projects.md` exist but are incomplete or lower priority. The resume is the public-facing artefact; `projects.md` is the full inventory.

---

### Decision: Profile entries describe what's built, not what's planned

When scraping repos, profile entries should lead with what the code actually demonstrates today. Future goals belong in the summary and project staging description, but technical highlights must be grounded in implemented reality. This avoids interview landmines ("tell me about your STDP implementation" when it doesn't exist yet) and actually sells the work better — concrete engineering is more compelling than aspirational architecture.

**Why:** The NeuroDrive scrape revealed the old entry described biological plasticity as if it was built, while the actual codebase — a handwritten PPO with entertainment-constrained reward engineering, comprehensive analytics, and performance work — was undersold. Leading with reality produced a stronger entry.

**How to apply:** Always distinguish "built and working" from "designed and planned." Both belong in the entry, but technical highlights come from code, not READMEs.

---

### Decision: Discovery is broad, filtering is later

Discovery finds companies that do work aligned with the profile. It does not eliminate companies for visa status, size, funding stage, or any other hard constraint — that filtering happens at the job evaluation stage. A 10-person startup with no public sponsor licence is still worth discovering if they do interesting systems work. Companies can make exceptions on sponsorship, and "can sponsor" is not always publicly disclosed.

**Why:** Premature filtering at discovery time means missing opportunities. A company that looks too small to sponsor might acquire sponsorship capability, or might have a relationship with a larger parent that handles it. Discovery casts wide; the funnel narrows at job search and evaluation.

---

### Decision: Discovery must be creative, not formulaic

Searching "best UK fintech 2026" catches obvious names everyone knows. The real value is finding the 60-person company doing brilliant ML infrastructure work with zero press coverage. This requires looking in unexpected places:

- GitHub orgs building in Rust/ML/infra
- Contributors to specific Rust crates
- Niche conference sponsors (QCon London, RustConf, etc.)
- HN "who's hiring" threads with insider context
- Engineering blog posts about topics matching the profile
- LinkedIn: where do people at interesting companies come *from*?
- Non-obvious sector matches (healthcare AI, climate tech, biotech infrastructure)

The creativity in *where to look* and *how to look sideways* is the skill's entire value. A dumb aggregator search is easy; finding what TrueUp would never surface is the goal.

---

### Decision: Discovery uses heavy parallelisation

A single agent doing 50+ web searches sequentially is too slow and context-limited. Discovery should use a team of parallel agents, each exploring a different sector or source type:

- An orchestrator reads the profile, divides the search space by sector/source, dispatches agents, and deduplicates results against the existing universe
- Each agent explores its assigned territory independently (AI/ML, fintech, healthcare, developer tools, trading systems, non-obvious sources like GitHub orgs and conference sponsors)
- Agents return structured results; the orchestrator merges and deduplicates

This is the most complex skill in the system. The quality of companies discovered determines the quality of everything downstream.

---

### Decision: Discovery is separate from resolution and enrichment

Discovery answers "which companies should we be watching?" — it outputs company name, website, what they do, why they're relevant, and where the agent found them. It does not resolve ATS portals or gather detailed enrichment data (funding rounds, headcount, Glassdoor ratings). Those are separate steps that happen after discovery.

Enrichment data (funding, growth signals, hiring velocity) can be noted at discovery time if readily available, but gathering it is not the discovery skill's responsibility.

---

### Reference: TrueUp as a model for discovery

TrueUp (trueup.io) is a tech job meta-aggregator that enriches company profiles with growth signals (funding, headcount trajectory, investor quality, Glassdoor/Blind sentiment, layoff data, hiring velocity). Key differentiator: company intelligence alongside job listings. Tracks 460K+ jobs across 51K+ company profiles. Publishes a weekly "Hot 200" ranking of fastest-growing tech companies.

Cernio's discovery is a personalised version of TrueUp's company intelligence layer — profile-aware from the start, conversational, and focused on finding companies TrueUp would miss (small teams, non-obvious sectors, indirect signals). We don't replicate their automated data pipeline; we compensate with creative search strategies and human-in-the-loop judgment.

---

### Decision: SQLite as the single source of truth for structured data

The company universe, job search results, evaluation status, and user decisions all live in a single SQLite database (`state/cernio.db`). Markdown files become human-readable views and exports, not the source of truth for structured data.

**Why SQLite over other options:**
- Single file, zero infrastructure — no Docker, no server process, no connection string. As local-first as it gets.
- 740 companies is trivial for SQLite (handles millions of rows without issue)
- Full SQL querying, indexing, and fast filtered reads — the TUI can query "all fintech companies sorted by discovery date" instantly
- Rust has mature support via `rusqlite` (sync) and `sqlx` (async)
- Backup is trivial: it's a file. Copy it before risky operations.
- The TUI reads it directly for real-time display with no file parsing or race conditions

**What lives in SQLite:** resolved company universe, ATS slugs and verification status, job search results, evaluation status and fit assessments, user decisions (watching/applied/rejected)

**What stays in markdown:** `profile/` (human-edited, read by Claude), `companies/potential.md` (initial landing zone from discovery before migration to SQLite), `exports/` (generated reports on demand)

**Schema grows organically:** companies are added in small batches as discovery runs across sessions, not bulk-loaded. The database evolves alongside the project.

**Safety:** Schema tracked in a migration file so the database can always be recreated. The `.db` file lives in `state/` (gitignored). Markdown exports serve as human-readable snapshots.

---

### Decision: TUI is a live dashboard for the entire universe, not just jobs

The TUI should have views for the full company lifecycle — potential companies awaiting research, resolved companies with ATS slugs, bespoke companies with careers links, and the full universe sortable and filterable. When Claude researches a company, the TUI row updates in real time (potential → researching → strong fit / weak fit → approved). Same pattern for job search and evaluation.

**Why:** The user wants to watch the entire process unfold — from discovery through research through job evaluation. The TUI is the window into everything Cernio is doing.

---

### Note: Profile preferences are intentionally flexible

Hard filters like `exclude_sectors` and `tech_must_have` are kept loose because Caner is an entry-level engineer and wants to explore options rather than prematurely narrow the search. This is a deliberate choice, not an oversight.
