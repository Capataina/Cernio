# Cernio — Architecture

> Last updated: 2026-04-07 — pre-implementation, based on README and initial design session.

---

## What Cernio Is

A local-first, collaborative job discovery and curation engine. The user and Claude work together in a conversational session to find, evaluate, and curate job opportunities from a personally built universe of UK and remote-UK technology employers.

---

## System Shape

Cernio has three layers that communicate through a shared data store:

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
│                          │───►│  SQLite or JSONL (TBD)           │
│  • search: scan ATS      │    │  • raw results from scripts     │
│    boards for terms      │    │  • evaluation status per result  │
│  • fetch: pull job JSON  │    │  • user decisions (applied,      │
│  • resolve: probe slugs  │    │    watching, rejected)           │
│  • export: generate md   │    │  • company universe              │
│                          │    │  • profile data                  │
└──────────────────────────┘    └──────────────┬──────────────────┘
                                               │ watches
                                               ▼
                                ┌──────────────────────────────────┐
                                │         Ratatui TUI               │
                                │         (live dashboard)          │
                                │                                   │
                                │  • Shows all results in real time │
                                │  • Status per row: pending,       │
                                │    evaluating, fit, no fit        │
                                │  • User confirms/rejects from UI  │
                                │  • Export to markdown on keypress  │
                                └──────────────────────────────────┘
```

### Layer responsibilities

| Layer | Does | Does not |
|-------|------|----------|
| **Conversation** (Claude + user) | Orchestrates scripts, evaluates job descriptions against profile, recommends actions, discusses strategy | Submit applications, make decisions without user input |
| **Rust scripts** | High-volume parameterised work: scan hundreds of ATS boards, probe slug patterns, fetch job JSON, generate exports | Make judgments, know about the profile, decide what to search for |
| **TUI** | Real-time display of results and evaluation status, user confirmation of decisions, markdown export trigger | Run scripts, evaluate jobs, modify data independently |
| **Data store** | Contract between all layers — scripts write results, Claude writes evaluations, TUI reads and displays, user actions write back | Contain business logic |

### The key insight

Scripts handle **combinatorial volume** (hundreds of boards x dozens of search terms = thousands of checks in seconds). Claude handles **judgment** (read the actual job description, compare against structured profile, assess fit). The user handles **decisions** (what to apply to, what to skip, what preferences to change). The TUI makes all of this **visible in real time**.

---

## AI Layer (Claude Code Skills)

Claude Code skills handle slow, fuzzy, infrequent work that requires reasoning:

| Skill | Purpose |
|-------|---------|
| `discover-companies` | Fetch public list sources, parse tables, deduplicate against universe |
| `resolve-portals` | Web search fallback when deterministic slug resolution fails |
| `enrich-company` | Pull funding, stage, headcount when missing |

Skills are invoked conversationally — the user says "run a discovery" and Claude routes to the right skill.

---

## Rust Scripts (Parameterised Tools)

Scripts are generic, stateless, and parameterised. They take inputs, produce outputs, and exit. They do not hardcode addresses, search terms, or profile data.

**Planned scripts** (specifics decided at implementation time):

| Script | Inputs | Output |
|--------|--------|--------|
| `profile-scrape` | GitHub repo URL | Structured repo data (README, context/, file tree, code samples) for Claude to produce profile entries |
| `search` | ATS slugs file, search terms, ATS provider | Matching jobs as structured results |
| `resolve` | Company names, ATS URL patterns | Slug matches with verification status |
| `export` | Result set, format options | Markdown file with tables and links |

---

## Data Flow

```
1. Session starts
   └─► Claude reads profile/, context/, README.md
   └─► Claude and user discuss what to do

2. Profile update (when projects or skills have changed)
   └─► User gives Claude a GitHub repo link
   └─► Script scrapes repo: README, context/, code, file tree
   └─► Claude produces/updates projects.md entry, skills.md additions
   └─► Profile stays current without manual editing

3. Discovery (when needed)
   └─► Claude skill scrapes public lists
   └─► New companies written to companies/potential.md

4. Resolution (when needed)
   └─► Rust script probes ATS slug patterns (deterministic)
   └─► Claude skill handles failures (fuzzy fallback)
   └─► Companies land in companies/{ats}.md or companies/bespoke.md

5. Job search (collaborative decision)
   └─► User and Claude decide what to search for
   └─► Claude builds params, invokes Rust search script
   └─► Raw results written to data store
   └─► TUI shows results appearing in real time

6. Evaluation (Claude)
   └─► Claude reads each job description
   └─► Compares against profile preferences
   └─► Writes evaluation status to data store
   └─► TUI updates rows: evaluating → fit/no fit

7. User review
   └─► User reviews evaluated results in TUI
   └─► Marks jobs as watching, applied, rejected
   └─► Presses export key → markdown generated

8. Session wrap-up
   └─► User tells Claude what they applied to
   └─► Claude updates state, suggests next actions
```

---

## Filesystem Layout

```
cernio/
├── src/                    # Rust source code
├── profile/                # Structured personal profile (read every startup)
│   ├── education.md
│   ├── experience.md
│   ├── projects.md
│   ├── skills.md
│   ├── preferences.toml    # Hard and soft job preferences
│   └── ...                 # Additional profile files
├── companies/              # Company universe
│   ├── potential.md        # Newly discovered, unresolved
│   ├── greenhouse.md       # Resolved to Greenhouse
│   ├── ashby.md            # Resolved to Ashby
│   ├── lever.md            # Resolved to Lever
│   ├── workable.md         # Resolved to Workable
│   └── bespoke.md          # No supported ATS, careers URL preserved
├── state/                  # User decisions and application tracking
│   ├── db.sqlite           # Hot-path data store (TBD)
│   ├── applied.toml
│   ├── watching.toml
│   └── rejected.toml
├── exports/                # Generated markdown reports
├── context/                # Project memory (for Claude sessions)
├── learning/               # Educational material (optional)
├── skills/                 # Claude Code skill definitions
└── Cargo.toml
```

---

## Technology Stack

| Component | Choice | Reason |
|-----------|--------|--------|
| Core language | Rust | Async networking, structured parsing, TUI ecosystem, portfolio value |
| Async runtime | Tokio | Standard for Rust async |
| HTTP | Reqwest | Simple JSON over HTTPS |
| Serialisation | Serde | JSON, TOML, JSONL |
| TUI | Ratatui | Modern, maintained, strong ecosystem |
| Cache/data store | SQLite (rusqlite or sqlx) | Hot-path queries, real-time TUI updates |
| AI layer | Claude Code skills | Fuzzy reasoning, conversational orchestration |

---

## Current State

**Pre-implementation, project setup complete.** The repository contains:
- `Cargo.toml` — bare package definition, no dependencies yet
- `src/main.rs` — hello world
- `CLAUDE.md` — session configuration (tuned: no teaching mode, editable README, conversational skills, profile on startup)
- `README.md` — project vision and milestones (reflects collaborative model)
- `context/` — architecture and session notes capturing all design decisions
- `profile/` — 12 structured files, fully populated with the user's background, skills, projects, preferences, and constraints

No production code, no scripts, no company universe yet. Milestone 1 (skeleton and profile schema) is substantially complete. Next: Milestone 2 (profile auto-update from repos) or Milestone 3 (universe construction).
