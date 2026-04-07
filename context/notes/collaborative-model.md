# Collaborative Session Model

Cernio is a collaborative tool, not an automated pipeline. Every action happens in a conversational session where the user and Claude decide together what to do.

---

## No automated pipeline

The original README described Cernio as a daily automated pipeline (`cernio refresh`). This was revised in the first design session. There is no single "run everything" command. Scripts and skills are invoked dynamically based on what the session needs.

**Why:** The user wants to be involved in every decision — what to discover, what to search for, what to apply to. The tool serves the user's judgment, not replaces it.

---

## Scripts for volume, Claude for judgment

Rust scripts are parameterised tools for combinatorial work — scanning hundreds of ATS boards for dozens of search terms, probing slug patterns, fetching job JSON. They are generic: no hardcoded addresses, no hardcoded search terms, no profile awareness.

Claude handles judgment — reading job descriptions, comparing against the structured profile, assessing fit, and explaining reasoning.

**Why:** A script can check 100 Greenhouse boards × 50 title patterns = 5,000 combinations in seconds. Claude cannot do that economically. But Claude can read 50 resulting job descriptions and assess fit against a nuanced profile — something a keyword script cannot do well.

**Implication:** Scripts must be designed as generic, reusable tools with parameterised inputs. They should work for any ATS, any search terms, any set of companies. The intelligence lives in the conversation, not in the scripts.

---

## TUI as a real-time dashboard

The ratatui TUI shows live state as it happens — results appearing from script runs, evaluation status updating as Claude reads job descriptions (pending → evaluating → fit/no fit), user actions (watching, applied, rejected). It is not a standalone browser for pre-computed results.

The TUI covers the full company lifecycle — potential companies awaiting research, resolved companies with ATS slugs, bespoke companies with careers links, and the full universe sortable and filterable. When Claude researches a company, the TUI row updates in real time.

**Why:** The user wants to follow everything in real time — from company discovery through evaluation to export. The TUI is the window into everything Cernio is doing.

---

## Markdown export on user confirmation only

Markdown reports are generated only when the user confirms results in the TUI and explicitly triggers export (e.g. pressing a key). Nothing is produced automatically.

**Why:** The user reviews and confirms everything before it becomes an actionable list.
