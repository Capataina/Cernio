# Cernio

> A local-first, collaborative job discovery and curation engine written in Rust. Built to solve the real bottleneck in a serious technical job search: finding the right roles at the right companies, evaluated against your actual profile, sourced from a personally curated universe of employers, and reviewed together in a real-time terminal dashboard.

---

## Why Cernio exists

Job search tools fall into two camps and both are broken in different ways.

The first camp is the mass-market aggregator. LinkedIn, Indeed, Glassdoor, Reed. They optimise for volume and engagement, not for signal. They surface jobs from companies that pay to be surfaced, they bury the listings that matter under hundreds that do not, and their filters are too coarse to encode what you actually care about. A backend systems engineer interested in payments infrastructure at a London scaleup gets the same firehose as someone looking for a customer success role at a CRM startup. The tool does not know the difference and cannot be made to.

The second camp is the curated tracker. TrueUp, Otta, Wellfound. Better signal, smaller universe, but the curation is somebody else's. Their editorial choices about what counts as a "hot" company shape what you see, and those choices may or may not match what you actually want. They also tend to lock the most useful filters behind a subscription, charge for export, or obscure the underlying data so you cannot build on top of it.

Both camps share a deeper problem: they treat your CV and your preferences as an afterthought. You upload a PDF, the system extracts a few keywords, and matching becomes a fuzzy search over those keywords against a job description. There is no structured representation of your skills, your projects, your visa situation, your location constraints, your seniority, your tech stack preferences, your dealbreakers, or your soft preferences. The tool does not know that you write Rust, that you are interested in payment rails but not consumer crypto, that you need visa sponsorship, that you live in London, that you would consider remote-UK roles, that you have shipped a real distributed systems project, or that you have explicitly opted out of consulting and customer-facing engineering. It cannot match against any of that because it never asked.

Cernio takes the opposite approach. Your structured profile is the source of truth — not a parsed CV, but separate files covering education, experience, projects, skills, preferences, and everything else that matters in an application. A curated universe of companies, built from authoritative public lists and refreshed on demand, is the search space. Parameterised scripts handle the volume work: scanning hundreds of ATS boards for matching roles in seconds. Claude reads the actual job descriptions, compares them against your profile, and evaluates fit with full reasoning. A real-time terminal dashboard shows you everything as it happens — results arriving, evaluations in progress, fit assessments completing. You review, confirm, and decide. When you're satisfied, you export a clean markdown report of the jobs worth applying to.

The goal is not to automate applications. The goal is to ensure that during each session, the small number of jobs you should actually consider are surfaced, evaluated, and sitting in front of you — with full reasoning about why each one fits or doesn't. Everything downstream of that is left to you, because the bottleneck is not application throughput. The bottleneck is finding the right things to apply to in the first place.

---

## How Cernio works

Cernio is a collaborative tool, not an automated pipeline. You launch a session with Claude, discuss what makes sense to do, and work through the job search together. A typical session looks like this:

1. **Session starts** — Claude reads your profile, the company universe, and recent state. You discuss what's changed: new applications, updated preferences, new companies to track.

2. **Profile update** (when your projects or skills have changed) — Give Claude a GitHub repo link and it scrapes the repo — README, context folder, code — and produces or updates your profile entries automatically. New skills, updated proficiency levels, and technical highlights are captured without manual editing.

3. **Discovery** (when the universe needs expanding) — Claude scrapes authoritative public lists for new companies, deduplicates against the existing universe, and writes new entries for resolution.

4. **Resolution** (when new companies need ATS identification) — A Rust script probes known ATS URL patterns deterministically. Claude handles the fuzzy fallback for companies that don't match standard patterns.

5. **Job search** (the core loop) — You and Claude decide what to search for. Claude builds the parameters, invokes a Rust script that scans hundreds of ATS boards for matching terms in seconds, and results flow into the data store.

6. **Evaluation** — Claude reads each job description, compares it against your structured profile, and writes fit assessments. The TUI shows this in real time: each row transitions from *pending* to *evaluating* to *strong fit* / *no fit*.

7. **Review and export** — You review the evaluated results in the TUI. Mark jobs as watching, applied, or rejected. When satisfied, press a key to export a clean markdown report.

8. **Session wrap-up** — You tell Claude what you applied to, what changed, and Claude updates state and suggests what to focus on next time.

---

## Architecture

Cernio is structured as three layers communicating through a shared data store:

```
┌─────────────────────────────────────────────────────────────────┐
│                   Conversational Session                         │
│                   (You + Claude Code)                            │
│                                                                  │
│  • Decide what to do: discover, fetch, evaluate, export         │
│  • Claude orchestrates scripts, evaluates results, advises      │
│  • You make all application decisions                           │
└──────────┬──────────────────────────────────┬───────────────────┘
           │ invokes                          │ writes evaluations
           ▼                                  ▼
┌─────────────────────────┐    ┌──────────────────────────────────┐
│    Rust Scripts          │    │         Shared Data Store        │
│    (parameterised tools) │    │                                  │
│                          │───►│  • Raw results from scripts     │
│  • search: scan ATS      │    │  • Evaluation status per result │
│    boards for terms      │    │  • User decisions (applied,     │
│  • resolve: probe slugs  │    │    watching, rejected)          │
│  • export: generate md   │    │  • Company universe             │
│                          │    │  • Profile data                 │
└──────────────────────────┘    └──────────────┬──────────────────┘
                                               │ watches
                                               ▼
                                ┌──────────────────────────────────┐
                                │         Ratatui TUI               │
                                │         (live dashboard)          │
                                │                                   │
                                │  • Results appear in real time    │
                                │  • Status: pending → evaluating   │
                                │    → fit / no fit                 │
                                │  • User confirms, marks, exports  │
                                └──────────────────────────────────┘
```

### Why this split

**Scripts handle combinatorial volume.** Scanning 200 Greenhouse boards for 50 title patterns means 10,000 checks. A Rust script does this in seconds. An LLM cannot do it economically or quickly.

**Claude handles judgment.** The script returns 80 matches. Claude reads each job description, compares it against a structured profile with dozens of dimensions (tech stack, seniority, visa, location, remote policy, sector, dealbreakers), and explains why each one fits or doesn't. A keyword script cannot do this well.

**The TUI makes it visible.** Instead of waiting for a pipeline to finish and reading a static markdown file, the user watches the process unfold — results arriving, evaluations completing, fit assessments updating in real time. The collaborative process is the product.

**The data store is the contract.** Scripts write results, Claude writes evaluations, the TUI reads and displays, user actions write back. Every layer is independently testable, and every artefact is inspectable.

### The AI layer

Claude Code skills handle slow, fuzzy, infrequent work that requires reasoning:

| Skill | Purpose |
|-------|---------|
| `discover-companies` | Fetch public list sources, parse tables, deduplicate against universe |
| `resolve-portals` | Web search fallback when deterministic slug resolution fails |
| `enrich-company` | Pull funding, stage, headcount when missing |

Skills are invoked conversationally. The user says "run a discovery" and Claude routes to the right skill — no CLI syntax required.

### The Rust scripts

Scripts are generic, stateless, and parameterised. They take inputs, produce outputs, and exit. They do not hardcode addresses, search terms, or profile data. The intelligence lives in the conversation that decides what inputs to feed them.

### The TUI

The TUI is a live dashboard for the collaborative session, not a standalone application. It watches the shared data store and reflects state changes in real time:

- Results from script runs appear as rows
- Each row shows status: pending, evaluating, strong fit, weak fit, no fit
- The user can mark jobs directly: watching, applied, rejected
- A keypress triggers markdown export of the current view

---

## What Cernio focuses on

Cernio targets five capabilities, each independently useful:

- **Profile management** — a structured personal profile covering education, experience, projects, skills, visa status, and preferences, auto-updatable from GitHub repo links by scraping READMEs, context folders, and code to produce accurate entries without manual editing
- **Company universe construction** — discovering UK and remote-UK technology employers from authoritative public list sources, deduplicated and tagged by sector, stage, and geography
- **ATS portal resolution** — deterministic and agent-assisted resolution of each company to its applicant tracking system slug, with graceful fallback for companies on custom portals
- **Job search and evaluation** — parameterised scripts scan ATS boards at volume, Claude evaluates results against the structured profile, the TUI shows everything in real time
- **Curation and export** — user reviews evaluated results in the TUI, confirms decisions, and exports clean markdown reports on demand

Each capability is independently completable. A working universe and resolver alone is already more useful than any existing tool for the niche of "show me which UK tech companies use which ATS and link me directly to their boards."

---

## Design Principles

- **Collaborative, not automated**: every action happens in a conversational session. Claude orchestrates and evaluates, the user decides. There is no "run everything" command.
- **Local-first**: every component runs on your machine. No cloud dependencies, no hosted database, no telemetry, no account, no subscription.
- **Scripts for volume, Claude for judgment**: parameterised Rust scripts handle the combinatorial work that would be impractical for an LLM. Claude handles the nuanced evaluation that keyword matching cannot do well.
- **Live visibility**: the TUI shows the entire process in real time — discovery, resolution, search results, evaluation status. No waiting for a pipeline to finish.
- **Structured profile, not parsed CV**: your profile lives as separate files covering everything that matters in a job application. The tool reads structured data, not extracted keywords from a PDF.
- **Export on confirmation**: markdown reports are generated only when the user explicitly triggers them after reviewing results. Nothing is produced automatically.
- **Discovery is wide, filtering is sharp**: the discovery layer captures everything that meets a low bar. Evaluation is where opinions live.
- **Graceful degradation**: companies on unsupported ATS or custom portals land in a bespoke tier with a direct link, not in a dropped queue.
- **Human in the loop for everything external**: Cernio never submits an application, sends an email, or contacts a recruiter. Every action that touches the outside world is performed by you.
- **Plain text everywhere**: markdown for company files, TOML for preferences, JSONL for job data, SQLite for hot-path queries. Nothing is opaque.
- **Generic scripts, not hardcoded pipelines**: scripts take parameters and work for any ATS, any search terms, any set of companies. No hardcoded addresses or terms.

---

## Roadmap

- [ ] Milestone 1: Project Skeleton and Profile Schema
- [ ] Milestone 2: Profile Auto-Update from Repos
- [ ] Milestone 3: Universe Construction
- [ ] Milestone 4: Portal Resolution
- [ ] Milestone 5: Parameterised Search Scripts
- [ ] Milestone 6: Data Store and Real-Time TUI
- [ ] Milestone 7: Job Evaluation Pipeline
- [ ] Milestone 8: Curation, Export, and Bespoke Tier

Milestones 1 through 8 constitute the core project. Each is independently shippable and adds value without depending on the next being polished.

---

## Technology choices

**Fixed choices:**
- **Rust** for the core — async networking, structured parsing, TUI ecosystem, and genuine engineering substance.
- **Tokio** for async runtime.
- **Reqwest** for HTTP — the public ATS APIs are simple JSON over HTTPS.
- **Serde** for serialisation across JSON, TOML, and JSONL.
- **Ratatui** for the terminal UI — modern, maintained, strong ecosystem.
- **SQLite** (via rusqlite or sqlx) for the shared data store — real-time TUI updates require something faster than parsing JSONL on every change.
- **Claude Code skills** as the AI layer — conversationally invoked, communicating with the Rust core through the shared data store and filesystem.

**Deferred until needed:**
- Exact ATS providers beyond the initial four (Greenhouse, Ashby, Lever, Workable).
- Whether to add an LLM-based extraction pass at ingest time vs curated keyword lists.
- Exact data store schema — decided when the TUI's real-time update requirements are clear.
- Bespoke tier enrichment format.

---

## What this project is not

**It is not an auto-applier.** No version of Cernio submits an application, sends an email, or contacts a recruiter. Every action that touches the outside world is performed by a human after review.

**It is not an automated pipeline.** There is no `cernio refresh` command that runs everything daily. Every action happens in a collaborative session where the user and Claude decide together what to do.

**It is not a generic aggregator.** The universe is curated to the user's geography, sector interests, and constraints. Quality over quantity.

**It is not an LLM-scoring tool.** Job matching uses structured comparison against a structured profile. There is no opaque "AI fit rating."

**It is not a hosted service.** Everything runs locally. No account, no signup, no subscription, no telemetry.

---

## Why the name

Cernio is from the Latin *cernere* — to sift, to distinguish, to discern, to decide. It is the root of words like "discern", "concern", "certain", and "discrete". The verb describes the act of separating signal from noise, which is exactly what the project does at every layer: separating relevant companies from the wider market, separating interesting jobs from every open role, separating jobs worth applying to from jobs that merely match keywords. The project is a sieve. The name is the action.

---

## Summary

Cernio is a local-first, collaborative job discovery and curation engine built in Rust with a Claude Code agent layer for the parts that genuinely benefit from reasoning. You and Claude work together in a conversational session: parameterised Rust scripts handle the volume work of scanning hundreds of ATS boards, Claude evaluates results against your structured profile, and a real-time terminal dashboard shows the entire process as it unfolds. You review, confirm, and export — no automation without your explicit approval, no cloud dependencies, no running costs.
