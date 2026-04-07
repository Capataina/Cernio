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

### Note: Profile preferences are intentionally flexible

Hard filters like `exclude_sectors` and `tech_must_have` are kept loose because Caner is an entry-level engineer and wants to explore options rather than prematurely narrow the search. This is a deliberate choice, not an oversight.
