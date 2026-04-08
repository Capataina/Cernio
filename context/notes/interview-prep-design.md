# Interview Prep Design

How the interview-prep skill will work — a personalised, rigorous learning and practice system built from the user's actual job targets and portfolio gaps.

---

## Core idea

Take the `upkeep-learning` skill from the agent-skills framework (which teaches repo-wide implementation) and adapt it for interview preparation. Instead of learning a codebase, it teaches the user the skills, algorithms, and domain knowledge needed for their top-tier job targets.

---

## Inputs

The skill reads three things to build a curriculum:

1. **SS, S, and A-tier jobs from the database** — both applied and not-yet-applied. These represent the roles the user is actively targeting. The skill extracts: required technologies, domain knowledge, algorithmic expectations, system design patterns, and company-specific themes.

2. **`profile/portfolio-gaps.md`** — known gaps between what the market asks for and what the profile currently demonstrates. These are the highest-priority learning targets.

3. **`profile/` (full)** — what the user already knows, so the curriculum doesn't waste time on things they're already strong in.

---

## What it produces

A `learning/` directory (same structure as `upkeep-learning` outputs) containing:

### Concept files
Deep-dive explanations of algorithms, data structures, system design patterns, and domain knowledge relevant to the target jobs. Written to teach, not just reference — worked examples, intuition, complexity analysis, trade-offs.

### Learning paths
Ordered sequences through the concept files, grouped by theme (e.g., "distributed systems fundamentals", "low-latency data structures", "graph algorithms for trading systems"). Paths are personalised — if the user already knows hash maps but not B-trees, the path starts at B-trees.

### LeetCode-style problems (the key differentiator)

Unlike LeetCode, these are 100% personalised to the user's job targets:

**Unit-test problems (algorithm practice):**
- Claude describes a problem derived from patterns seen in the target jobs
- The user implements the solution in a language Claude chooses (Rust, Python, etc.)
- Claude provides unit tests — the user runs them to verify correctness
- Claude offers hints on request, teaches the algorithm after completion

**Integration-test problems (systems practice):**
- Claude designs a multi-component system (e.g., "build a simple order book with price-time priority, a matching engine, and a trade reporter")
- Each component has its own unit tests
- Integration tests verify the components work together
- This is the upgrade over LeetCode — practice building systems, not just algorithms

**Company-specific problems:**
- Problems designed around the actual engineering challenges of target companies
- E.g., for a trading systems role: implement a lock-free queue, build a simple FIX parser, design a market data fanout system
- For an infrastructure role: implement a connection pool, build a rate limiter, design a service discovery mechanism

### Study documents
Company-specific research briefs for interview preparation:
- What the company builds, their tech stack, recent engineering blog posts
- Common interview formats and what to expect
- How to frame the user's projects in terms the company cares about

---

## Invocation

Conversational: "let's prep for interviews", "create a learning plan", "I have an interview at Cloudflare", "practice algorithms".

The skill should also be invocable with a specific focus: "prep me for trading systems interviews" or "create problems around distributed consensus".

---

## Relationship to upkeep-learning

The `upkeep-learning` skill maintains a `learning/` archive for understanding a project's codebase. The `interview-prep` skill maintains a `learning/` archive for understanding the skills needed for target jobs. Same structural pattern (concept files, learning paths, exercises), different content source (job descriptions and portfolio gaps instead of source code).

The skill does NOT need background-knowledge files (those are for understanding a repo's domain). It does need concept files, learning paths, and exercises — the three most directly useful components.

---

## Future integration

- The TUI could show interview prep status alongside job pipeline (how many concepts studied, problems solved)
- Portfolio gap closure could be tracked — when a learning path is completed, the gap narrows
- Mock interview sessions could be a conversational flow: Claude asks questions, user answers, Claude evaluates
