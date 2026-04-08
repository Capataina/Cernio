# Cernio

A local-first job discovery, evaluation, and career preparation engine built in Rust and powered by Claude Code. Cernio builds a structured profile of who you are, discovers companies you'd never find on a job board, scans their open roles in seconds, evaluates every listing against your actual skills and goals, and presents everything in a real-time terminal dashboard. You make every decision — the system handles the volume work and the nuanced evaluation that neither scripts nor job boards can do alone.

---

## Table of Contents

1. [The Problem](#the-problem)
2. [How Cernio Thinks About Job Search](#how-cernio-thinks-about-job-search)
3. [The Pipeline](#the-pipeline)
   - [Your Profile](#1-your-profile)
   - [Company Discovery](#2-company-discovery)
   - [Portal Resolution](#3-portal-resolution)
   - [Job Search](#4-job-search)
   - [Evaluation](#5-evaluation)
   - [Curation and Decisions](#6-curation-and-decisions)
4. [The Terminal Dashboard](#the-terminal-dashboard)
5. [The Skill System](#the-skill-system)
6. [Design Philosophy](#design-philosophy)
7. [What Makes This Different](#what-makes-this-different)
8. [Where This Is Going](#where-this-is-going)
9. [Architecture](#architecture)
10. [Technology](#technology)
11. [Current State](#current-state)

---

## The Problem

Every job search tool available today makes the same trade-off: you get either volume or relevance, never both.

Aggregators like LinkedIn and Indeed show you thousands of listings, but their filters can't encode what you actually care about. You can filter by "Software Engineer" in "London," but you can't say "I write Rust, I'm interested in infrastructure and trading systems, I need visa sponsorship in 18 months, I don't want consulting or customer-facing roles, and I have a matching engine project that makes me a strong fit for exchange infrastructure." The tool doesn't know any of that, and it can't be made to.

Curated platforms like Otta and Wellfound are better on signal, but the curation is someone else's. Their editorial decisions about what counts as a "hot" company shape what you see. And the most useful features tend to be behind a paywall.

Both approaches share a deeper issue: they treat your background as an afterthought. You upload a PDF, the system extracts a few keywords, and matching becomes a fuzzy keyword search. There's no structured representation of your projects, your visa constraints, your career trajectory, your technical depth, or your dealbreakers. The tool can't distinguish between a systems engineer interested in payment infrastructure and someone looking for a customer success role — because it never had the information to try.

Cernio starts from the other end. Your structured profile is the foundation. Everything else — which companies to watch, which jobs to surface, how to evaluate fit — flows from a detailed understanding of who you are, what you've built, and where you're heading.

---

## How Cernio Thinks About Job Search

Cernio is a collaborative tool, not an automated pipeline. There's no "run everything" button. You work with Claude in a conversational session: you discuss what needs doing, Claude orchestrates the scripts and skills, and you make every decision. A typical session might look like:

> "Let's expand the universe — run a discovery focused on AI infrastructure companies."
> "Search all S-tier companies for new roles."
> "Grade the pending jobs, start with the graduate roles."
> "I applied to Cloudflare and Palantir yesterday — update the pipeline."

The system is built around a simple split: **scripts handle volume, Claude handles judgment.** Scanning 200 job boards for 50 search terms means 10,000 HTTP requests — a Rust script does that in seconds. But reading 80 resulting job descriptions and evaluating each one against a nuanced profile with dozens of dimensions? That requires reasoning, not pattern matching. Each layer does what it's best at.

Everything happens locally. No cloud, no accounts, no telemetry, no subscriptions. Your profile data, your company universe, your evaluations — all of it lives on your machine.

---

## The Pipeline

Cernio's pipeline has six stages. Each one is independently useful and builds on the ones before it.

### 1. Your Profile

The profile is the foundation of everything downstream. It's not a parsed CV — it's a structured collection of files covering every dimension that matters in a job application:

```
profile/
├── personal.md          # Location, nationality, contact
├── education.md         # Degrees, modules, classification
├── experience.md        # Work history
├── projects.md          # Full project portfolio (technical highlights, stack, status)
├── skills.md            # Languages, frameworks, tools, domains with proficiency levels
├── preferences.toml     # Search filters, location patterns, excluded sectors, career targets
├── visa.md              # Visa type, expiry, sponsorship requirements
├── portfolio-gaps.md    # Market-identified gaps and closure opportunities
├── resume.md            # LaTeX resume source
├── cover-letter.md      # Application narrative
├── interests.md         # Domain interests
├── certifications.md    # Professional certifications
├── languages.md         # Spoken languages
├── military.md          # Service status (relevant for clearance)
└── volunteering.md      # Community involvement
```

The profile is a living system. It changes as you build new projects, gain new skills, and shift your preferences. When that happens, the grades and evaluations downstream are considered potentially stale — the system knows to flag them for re-evaluation rather than silently showing outdated assessments.

#### Automated profile updates from GitHub

One of Cernio's most useful features is the `profile-scrape` skill. Give Claude a GitHub repo link and it reads the repository — the context folder, README, dependency manifests, source code — and produces or updates your profile entries automatically. New skills are captured with honest proficiency levels based on what the code actually demonstrates. Technical highlights are extracted from the architecture, not from aspirational README descriptions.

This means your profile stays current as your projects evolve. No manual editing, no self-reported skill levels. The code is the source of truth.

#### Career coaching and gap analysis

As Cernio evaluates jobs across hundreds of companies, patterns emerge. If every strong-fit infrastructure role mentions Kubernetes and your profile doesn't have it, that's a concrete gap worth closing. The system tracks these patterns in `portfolio-gaps.md` with specific recommendations — not "learn Docker" but "add a Dockerfile and CI pipeline to your matching engine project."

The profile isn't just a data source for matching. It's an evolving career document that gets sharper with every session.

---

### 2. Company Discovery

The company universe is the search space for everything downstream. The quality of the companies you track determines the quality of the jobs you find.

Discovery is handled by the `discover-companies` skill, which dispatches parallel AI agents to search for companies across different sectors and source types. Each agent explores its territory using real web searches — not training knowledge — looking for companies through the traces they leave in the ecosystem:

- **VC portfolio pages** for funded companies in target sectors
- **Open source contributor affiliations** for companies building with specific technologies
- **Conference sponsor lists** for companies investing in relevant domains
- **Engineering blog posts** for companies solving problems that match the profile
- **Community hiring threads** for companies with insider context about team and culture
- **Funding announcements** for companies that are growing and likely hiring
- **Sector-specific registries** (FCA register, Companies House, industry associations)

The key principle: **obvious searches find obvious companies.** Searching "best UK fintech 2025" returns the same list every job board already has. The skill's value comes from creative, indirect discovery — finding the 60-person company doing brilliant infrastructure work that has zero press coverage and would never appear on a "top companies" list.

Each discovered company captures what they do, why they're relevant to the profile specifically, and where the agent found them. The "why relevant" field matters — it connects the company to specific projects, technologies, and interests from the profile, not just generic sector labels.

After discovery, companies land in the database where they're graded on a four-tier scale (S/A/B/C) based on engineering reputation, technical alignment with the profile, growth trajectory, sponsorship capability, and career ceiling. C-tier companies are archived — preserved for deduplication but excluded from job searches. Everything above C stays in the active universe.

---

### 3. Portal Resolution

To search a company for jobs, Cernio needs to know which applicant tracking system they use and what their specific URL slug is. This is portal resolution.

The `cernio resolve` script handles the mechanical first pass. It generates slug candidates from the company name (e.g., "XTX Markets" → `xtxmarkets`, `xtx-markets`, `xtx`) and probes each one against every supported ATS provider:

| Provider | How it's checked |
|----------|-----------------|
| Greenhouse | `boards-api.greenhouse.io/v1/boards/{slug}/jobs` |
| Lever | `api.lever.co/v0/postings/{slug}` |
| Ashby | `api.ashbyhq.com/posting-api/job-board/{slug}` |
| Workable | `apply.workable.com/api/v1/widget/accounts/{slug}` |
| SmartRecruiters | `api.smartrecruiters.com/v1/companies/{slug}/postings` |
| Workday | `{company}.wd{N}.myworkdayjobs.com/wday/cxs/{company}/{site}/jobs` |

The script probes all providers per company and records every hit — because companies sometimes use multiple ATS platforms (engineering on Greenhouse, corporate on Workday). The one with the most jobs is marked as primary.

Companies that fail mechanical resolution get picked up by the `resolve-portals` AI skill, which uses web search to find the careers page and extract the correct ATS URL from links, iframes, and redirects. Companies on unsupported ATS providers (iCIMS, Taleo, Personio, Pinpoint HQ, etc.) are marked as "bespoke" with their careers URL preserved — they can't be searched automatically, but the information isn't lost.

A few hard-won lessons from production: SmartRecruiters returns HTTP 200 for completely fabricated slugs (only trust `totalFound > 0`). Companies migrate ATS providers and leave residual boards behind. Slug guessing is unreliable — XTX Markets uses `xtxmarketstechnologies` (their legal entity name). The careers page is always the answer.

---

### 4. Job Search

This is the volume step. The `cernio search` script fetches every open role from every resolved company above the configured grade threshold, applies mechanical filters, deduplicates against the database, and inserts the survivors.

The filter chain:

```
Raw jobs from ATS APIs
  → Location filter (London/UK/Remote patterns, configurable per ATS provider)
  → Exclusion filter (Principal, Director, VP, Intern — title keywords that are categorically wrong)
  → Inclusion filter (Engineer, Infrastructure, ML, Systems — at least one must match)
  → Deduplication (skip jobs already in the database by URL)
  → Insert into SQLite as pending evaluation
```

Every filter is configurable in `profile/preferences.toml` with inline documentation. The bias at every stage is toward inclusion — a false positive costs 30 seconds of AI evaluation time, but a false negative means a perfect job is silently lost forever.

"Senior" is deliberately not in the exclusion list. Many UK companies use "Senior" for their second engineering level. The AI evaluates seniority from the actual job description, not the title.

At target scale, the flow looks like this:

```
120 resolved S/A/B companies
  → ~9,000 raw jobs fetched
  → ~1,500 after location filter
  → ~1,200 after exclusion filter
  → ~600 after inclusion filter
  → ~500 new jobs after dedup
```

The script handles bespoke companies separately — for S and A-tier bespoke companies, Claude manually searches their careers pages and external job aggregators (LinkedIn, Indeed, Glassdoor) to ensure no opportunities are missed from companies outside the automated pipeline.

---

### 5. Evaluation

This is where Cernio's AI layer adds the most value. The `grade-jobs` skill reads every pending job description and evaluates it against the full profile across multiple dimensions:

**Critical dimensions** (either of these can force an F regardless of everything else):
- **Career ceiling** — does this domain lead to high-impact, high-income positions at 10-15 years? Systems engineering, trading infrastructure, ML infra = high ceiling. IT support, narrow QA = low ceiling.
- **Seniority match** — can you actually get hired? Ignore the title, read the actual requirements. "Senior" at many UK companies means 2-3 years out of university.

**High-weight dimensions:**
- **Skill breadth** — exposure to multiple layers vs locked into one narrow thing
- **Company signal** — does this name open doors on a CV? Matters most for a first job.
- **Technical depth** — genuinely hard problems or CRUD applications?
- **Sponsorship viability** — can and will they sponsor when the visa timeline demands it?

**Medium-weight dimensions:**
- Domain transferability, growth environment, tech stack relevance

Each job gets a grade from SS (apply immediately) to F (don't apply), with a fit assessment that explains the reasoning. The fit assessment isn't generic — it references specific projects from the profile, specific technologies, specific gaps, the visa timeline, and career trajectory targets. A reader should understand exactly why this role fits or doesn't fit for this specific candidate.

Grading is parallelised across multiple agents, each handling a batch of jobs grouped by company. The queue is prioritised by signal — graduate roles at S-tier companies get graded before generic titles at B-tier companies, so the strongest opportunities surface first.

The grading system uses six tiers:

| Grade | Meaning | What happens |
|-------|---------|-------------|
| **SS** | Exceptional fit — apply immediately | Full multi-paragraph assessment with application narrative |
| **S** | Strong fit — prioritise | Detailed assessment with strengths and gaps |
| **A** | Worth applying | Key alignment and notable weaknesses |
| **B** | Backup option | Brief assessment |
| **C** | Only if nothing better | One-line reason |
| **F** | Don't apply | Specific dealbreaker cited |

---

### 6. Curation and Decisions

After evaluation, everything lands in the terminal dashboard for review. You browse the results, mark jobs as watching, applied, or rejected, and export clean markdown reports when you're ready.

The pipeline view shows your application funnel as a kanban board: Watching → Applied → Interview. Cards are coloured by grade so you can see the quality of your pipeline at a glance.

The system tracks decisions over time. Applied jobs stay visible regardless of age. Watched jobs are your shortlist. Rejected jobs are hidden but preserved. The `cernio clean` script periodically removes F and C-graded jobs and archives low-value companies to keep the signal-to-noise ratio high.

---

## The Terminal Dashboard

The TUI is a real-time, interactive terminal dashboard built with Ratatui. It's designed to feel like lazygit or btop — responsive, keyboard-driven with full mouse support, and visually dense without being cluttered.

### Four views

| View | What it shows |
|------|--------------|
| **Dashboard** | Universe stats, grade distributions, ATS coverage, session summary, scrollable list of all top-tier jobs |
| **Companies** | Company table with grade, status, job count, ATS provider. Detail panel shows description, grade reasoning, relevance, and full job list |
| **Jobs** | Job table with grade, title, company, location, decision status. Detail panel shows full description, fit assessment, and URL |
| **Pipeline** | Kanban board — three columns (Watching / Applied / Interview) with grade-coloured cards |

### Key features

- **Search and filter** — press `/` for vim-style instant filtering across title, company, and location
- **Sort** — cycle through grade, company, date, and location ordering
- **Grade override** — correct AI grading errors directly from the TUI
- **Multi-select** — Ctrl+click and Shift+click for bulk operations (watch, apply, reject)
- **Export** — press `e` to export the current view to markdown
- **Archive toggle** — show or hide archived items
- **Mouse support** — scroll any pane, click any row, click tabs to switch views
- **Responsive layout** — adapts from side-by-side to stacked to compact based on terminal width
- **Session summary** — displays a natural-language summary of the current state, generated by Claude before TUI launch
- **Full job descriptions** — complete descriptions with HTML cleanup in the detail pane, not just truncated previews

The TUI auto-refreshes from SQLite every 2 seconds, so it stays in sync with concurrent script runs and Claude sessions. Every colour uses the terminal's ANSI palette — the TUI inherits your terminal theme (Catppuccin, Dracula, Nord, whatever you use) rather than imposing its own.

---

## The Skill System

Cernio uses Claude Code skills for work that requires judgment — the slow, fuzzy, infrequent tasks that scripts can't handle. Skills are structured AI workflows with mandatory reference files that ensure consistent, high-quality output.

| Skill | What it does |
|-------|-------------|
| `profile-scrape` | Scrapes GitHub repos and updates profile entries with evidence-based technical highlights |
| `discover-companies` | Dispatches parallel agents to find companies through creative web searches |
| `populate-db` | Orchestrates the pipeline from discovery to resolution — validates companies, runs the resolver, handles AI fallback |
| `resolve-portals` | AI fallback for companies that fail mechanical ATS resolution |
| `grade-companies` | Evaluates companies against the profile across engineering reputation, technical alignment, growth, sponsorship, and career ceiling |
| `grade-jobs` | Evaluates jobs against the profile with structured fit assessments, parallelised across agents |
| `check-integrity` | Audits database health — detects stale grades, shallow assessments, missing data, profile-driven staleness |
| `search-jobs` | Orchestrates the search pipeline — runs the script, reviews results, handles bespoke companies |

Every skill has a mandatory-read protocol: before execution, Claude must read the skill's SKILL.md, all reference files in the skill's `references/` directory, and every file in `profile/`. When delegating to parallel agents, the full content of every reference file is embedded in each agent's prompt — because agents can't read files from the repo, and a grading agent without the rubric produces useless output.

Skills are invoked conversationally. You say "run a discovery" or "grade the pending jobs" and Claude routes to the right skill. No CLI syntax required.

---

## Design Philosophy

These aren't just bullet points — they're the principles that shaped every decision in the project.

### Collaborative, not automated

Every action in Cernio happens in a conversational session. You and Claude discuss what to do, Claude orchestrates the tools, and you make every decision. There is no `cernio refresh` that runs the whole pipeline daily.

This is a deliberate choice, not a limitation. Job search involves judgment at every step — which companies to track, which sectors to explore, which roles to pursue, when to apply, how to prioritise. Automating these decisions means either making them badly or making them rigidly. The collaborative model keeps human judgment in the loop where it matters while automating the parts that are genuinely mechanical.

### Local-first

Every component runs on your machine. No cloud backend, no hosted database, no telemetry, no accounts, no subscriptions, no running costs. Your profile data, your company evaluations, your application decisions — all of it stays local.

This isn't just a privacy stance. It means you own your data completely. You can inspect every artefact, back up with a file copy, and build on top of it however you want. There's no vendor lock-in because there's no vendor.

### Scripts for volume, Claude for judgment

This split is the architectural foundation. Scanning 200 Greenhouse boards for 50 title patterns means 10,000 HTTP checks — a Rust script does that in seconds. Reading 80 resulting job descriptions and evaluating each against a nuanced profile with dozens of dimensions — that requires reasoning a script fundamentally cannot do.

Neither layer tries to do the other's job. Scripts never make judgment calls. Claude never does mechanical volume work. The boundary is clean and each side is optimised for what it's best at.

### The living system

Nothing in Cernio is permanent. Grades change when your profile changes — a company that was C-tier because you lacked Kubernetes experience becomes B-tier when you add a Kubernetes project. Job evaluations shift when your preferences change. The system is designed to detect and surface this staleness rather than silently showing outdated assessments.

Skills read the profile fresh at runtime rather than embedding snapshots. Archived entries are preserved for deduplication and reversibility, never deleted. The check-integrity skill detects when profile changes have invalidated existing grades and recommends targeted re-evaluation.

### Discovery is wide, filtering is sharp

The discovery layer captures everything that meets a low bar — a company doing technically interesting work is worth knowing about regardless of size, funding stage, or sponsorship track record. The funnel narrows at each subsequent stage: company grading filters the universe, search filters by location and title, evaluation filters by fit. Premature filtering at discovery time means missing opportunities. False negatives are the enemy at every stage.

### Human in the loop for everything external

Cernio never submits an application, sends an email, or contacts a recruiter. Every action that touches the outside world is performed by you. The system surfaces, evaluates, and organises — you decide and act. This is a hard constraint, not a temporary limitation. Even the future autofill feature (see [Where This Is Going](#where-this-is-going)) pre-fills forms but still requires you to review and submit.

---

## What Makes This Different

There are other projects that try to automate job search. Here's what Cernio does that they don't.

### A structured, living profile — not a parsed CV

Most tools extract keywords from a PDF and call it a profile. Cernio maintains 15 structured files covering education, experience, projects, skills, preferences, visa status, and more. The profile is auto-updatable from GitHub repos — give Claude a link and it reads your actual code to assess what you've built, what technologies you use, and what depth of engineering work you've demonstrated. No self-reported skill levels, no aspirational descriptions. The code is the evidence.

### Creative company discovery — not aggregator scraping

Instead of scraping Indeed or LinkedIn, Cernio dispatches parallel AI agents that search the web creatively — VC portfolios, open source contributor affiliations, conference sponsors, engineering blog posts, community hiring threads, sector-specific registries. Each agent explores a different territory and returns companies you would never have found through a job board. The first discovery run found 73 companies, many of which had no presence on any mainstream platform.

### Profile-grounded evaluation — not keyword matching

Every job is evaluated against the full structured profile across career ceiling, seniority fit, technical depth, skill breadth, company signal, sponsorship viability, domain transferability, and growth environment. The fit assessment references specific projects, specific technologies, specific gaps, and the visa timeline. It's not "good match" — it's "your matching engine project demonstrates the lock-free systems thinking this role demands, and Cloudflare is a confirmed sponsor with a graduate programme that addresses your visa timeline."

### Transparent reasoning — not opaque scores

Every grade comes with its full reasoning chain. You can read exactly why a company is S-tier and not A-tier, why a job is SS and not S, what the strengths and gaps are. Nothing is a black box. If you disagree with a grade, you can override it directly from the TUI.

### A real terminal dashboard — not a spreadsheet

The TUI is a four-view interactive dashboard with mouse support, search, filtering, sorting, a pipeline kanban, and responsive layout. It's designed to be as polished as tools like lazygit or btop — not a table dump in the terminal.

### Active career coaching — not passive matching

As the system evaluates hundreds of jobs across dozens of companies, it tracks patterns. If the market consistently asks for something your profile lacks, that's captured as a concrete gap with a specific closure recommendation. The profile gets sharper with every session.

---

## Where This Is Going

Cernio's current pipeline handles finding and evaluating jobs. The roadmap extends into preparing for and applying to them.

### Interview preparation engine

The `interview-prep` skill will take the SS, S, and A-tier jobs from the database (applied or not) alongside the portfolio gaps, and generate a personalised learning curriculum.

**Study materials:** Deep-dive concept files on algorithms, data structures, system design patterns, and domain knowledge drawn directly from what your target jobs ask for. Not generic LeetCode — materials tailored to the specific roles you're pursuing.

**LeetCode-style practice, but personalised:**
- Claude describes a problem derived from patterns in your target jobs
- You implement the solution in a language Claude chooses (Rust, Python, whatever the role uses)
- Claude provides unit tests — you run them to verify correctness
- Hints available on request, full algorithm teaching after completion

**Systems practice — the upgrade from LeetCode:**
- Multi-component system design problems with integration tests
- Build an order book, a matching engine, and a trade reporter — each component has unit tests, and integration tests verify they work together
- Practice building actual systems, not just isolated algorithms
- Entirely personalised to the companies and roles you're targeting

**Company-specific prep:**
- Research briefs on what the company builds, their tech stack, recent engineering blog posts
- Interview format expectations
- How to frame your projects in terms the company cares about

### Application autofill

When you press `o` to open a job link, the browser opens with application fields pre-filled from your profile. Name, email, education, work history — populated automatically. For platforms like Greenhouse that support URL parameters, this is straightforward.

The more interesting extension: **personalised cover letters and custom question responses.** Claude reads the job description, reads your profile, and drafts a cover letter that connects your specific projects to what the role actually asks for. Same for those "why do you want to work here" text boxes — genuine, specific answers grounded in your real background, not generic templates.

The critical constraint: Cernio still never submits anything. The form is pre-filled, the cover letter is drafted, but you review everything and click submit yourself. This keeps the human in the loop while eliminating the mechanical drudgery of re-typing the same information 50 times.

### Personalised resume generation

Instead of one resume for every application, generate a tailored version that emphasises the projects and skills most relevant to each specific role. A trading systems role gets your matching engine project front and centre. An ML infrastructure role leads with your neural network and inference work. Same candidate, same truth, different emphasis — which is exactly what good applicants do manually, just automated.

---

## Architecture

Cernio is three layers communicating through a shared SQLite data store:

```
┌─────────────────────────────────────────────────────────────────┐
│                   Conversational Session                         │
│                   (You + Claude Code)                            │
│                                                                  │
│  Decides what to do. Orchestrates scripts and skills.           │
│  Evaluates results. You make all application decisions.         │
└──────────┬──────────────────────────────────────────────────────┘
           │ invokes scripts          writes evaluations │
           ▼                                             ▼
┌──────────────────────┐         ┌────────────────────────────────┐
│    Rust Scripts       │────────►│        SQLite Database          │
│                       │         │                                 │
│  resolve: probe ATS   │         │  companies (lifecycle, grades)  │
│  search: scan boards  │         │  jobs (evaluations, grades)     │
│  clean: remove noise  │         │  user_decisions (watching,      │
│  check: verify health │         │    applied, rejected)           │
│  import: bulk ingest  │         │  company_portals (ATS slugs)    │
└───────────────────────┘         └──────────────┬────────────────┘
                                                 │ auto-refresh
                                                 ▼
                                  ┌────────────────────────────────┐
                                  │         Ratatui TUI              │
                                  │                                  │
                                  │  4 views: Dashboard, Companies,  │
                                  │  Jobs, Pipeline (kanban)         │
                                  │  Mouse, search, sort, export     │
                                  └──────────────────────────────────┘
```

The conversation layer sits at the top and drives everything. Scripts handle mechanical volume work downward. Claude writes evaluations into the database. The TUI watches the database and reflects changes in real time. User actions in the TUI write back through SQLite. No layer depends upward.

For the full architectural deep-dive — module structure, schema details, dependency direction, data flow — see `context/architecture.md`.

---

## Technology

| Component | Choice | Why |
|-----------|--------|-----|
| **Core** | Rust | Async networking, structured parsing, TUI ecosystem, and genuine engineering substance |
| **Async** | Tokio | Industry-standard async runtime |
| **HTTP** | Reqwest | Simple, reliable HTTP for ATS API calls |
| **Serialisation** | Serde | JSON (ATS responses), TOML (config), seamless |
| **Database** | SQLite via rusqlite | Single file, WAL mode for concurrent reads/writes, zero infrastructure |
| **TUI** | Ratatui 0.29 + Crossterm | Modern, maintained, rich widget ecosystem |
| **AI** | Claude Code skills | Conversational invocation, structured workflows with reference files |
| **Config** | TOML | `preferences.toml` — self-documenting, human-editable search filters and thresholds |

Six ATS fetchers: Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday. Companies on unsupported providers (iCIMS, Taleo, Personio, Pinpoint HQ, BambooHR, Jobvite) are marked bespoke with careers URL preserved.

---

## Current State

The core pipeline is fully operational. 79 companies in the database (59 resolved across 6 ATS providers), 684 jobs evaluated (14 SS, 53 S), 8 skills with comprehensive reference files, TUI v4 with 4 views and full interactivity.

### What's working

- Full pipeline: discover → resolve → search → grade → curate → export
- 6 ATS fetchers with parallel search across all resolved companies
- Company and job grading with profile-specific reasoning and evidence standards
- TUI with dashboard, company browser, job browser, and pipeline kanban
- Database maintenance: cleanup, integrity checks, staleness detection
- Profile auto-update from GitHub repos

### What's next

- Expanding the company universe with further discovery rounds
- Interview preparation engine (study materials, personalised practice problems)
- Application autofill (pre-filled forms, personalised cover letters)
- Personalised resume generation per application
