# Skill Architecture Decisions

How Cernio's skills are structured, invoked, and relate to the upstream framework.

---

## Project-specific skills live in this repo

Cernio's skills (discovery, population, profile scraping) are specific to this project and live in `skills/` within the repo. They are tightly coupled to Cernio's data model, filesystem layout, and workflow.

The upstream `agent-skills` repo (`Capataina/agent-skills`) contains universal, reusable skills (upkeep-context, code-health-audit, etc.) plus the principal-engineer personality template and 95KB of research on writing effective agent skills. Cernio's `CLAUDE.md` was derived from the principal-engineer personality but has been customised.

**Why:** Project-specific skills don't generalise. Keeping them here means they evolve with the project and don't pollute the universal skill library.

---

## Conversational invocation

Skills are invoked conversationally. The user says "run a discovery" or "scrape Nyquestro" and Claude routes to the correct skill. No CLI syntax or slash-command notation required.

**Why:** The user shouldn't need to remember exact skill names. Intent-based routing is faster and more natural.

---

## Skills designed following research principles

The `AgentCreationResearch/writing-skills.md` document (40KB) guided skill design. Key principles applied:

- **Positive framing** — tell what to do, not what not to do
- **Explain why** — every instruction includes reasoning so agents generalise
- **Structure fixed, content free** — workflow structure is locked; how the agent reasons within it is judgment
- **Quality checklist at the end** — recency anchor for important requirements
- **No arbitrary limits** — principles over numeric constraints
- **Progressive disclosure** — SKILL.md is the core; reference files load on demand

---

## Teaching mode removed

The original CLAUDE.md supported teaching and implementation modes. Teaching mode was removed — this project will never need it. Sessions go straight to implementation.

---

## README is editable

The original CLAUDE.md treated README.md as read-only. This was changed — the README is a living document updated as decisions change and hardcoded assumptions are replaced.

---

## Mandatory-read protocol (added session 3, 2026-04-08)

All 8 skills now enforce a mandatory-read block at the top of their SKILL.md. This block requires reading:
1. The skill's SKILL.md itself
2. Every file in the skill's `references/` directory
3. All files in `profile/`

This was added after discovering that agents executing skills (especially discovery and grading) were skipping reference files, producing shallow and generic results. The search-strategies reference for discovery and the grading rubrics for evaluation are what make these skills valuable — without them, agents default to generic output.

CLAUDE.md now enforces this globally via three sections: Skill Execution Protocol, Subagent Context Requirements, and Grade and Fit Assessment Quality Standard.

**Why:** The first discovery run's agents that didn't use web search produced obvious company lists. Grading runs without the full rubric produced shallow "good company, decent tech" reasoning. The mandatory-read protocol prevents both failure modes.
