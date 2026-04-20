# Scraping Methodology

> How to extract the information that matters from a GitHub repository. The goal is a profile entry that accurately represents what the project demonstrates — technically specific, evidence-grounded, and honest about depth and status.

---

## Table of Contents

- [Information sources, in priority order](#information-sources-in-priority-order) — context → README → manifests → API → source; priority explained
- [Assessing technical depth](#assessing-technical-depth) — what to look for, what NOT to highlight, the depth test
- [Assessing project status](#assessing-project-status) — status levels with evidence signals; the "not enough interest" nuance
- [Multi-repo scraping](#multi-repo-scraping) — iteration pattern and the skip-if-accurate rule
- [Cross-referencing across profile files](#cross-referencing-across-profile-files) — which profile file each finding updates

---

## Information sources, in priority order

### 1. Context folder (richest source)

If the repo has a `context/` folder (maintained by the upkeep-context skill or manually), this is the single richest source. Read these files first:

| File | What it gives you |
|------|-------------------|
| `context/architecture.md` | Full system shape, module structure, dependency direction, current implementation state, what's built vs planned |
| `context/notes.md` | Index of design decisions, lessons learned, rationale behind choices |
| `context/notes/*.md` | Individual design decision files — why specific approaches were chosen or rejected |
| `context/systems/*.md` | Deep-dive into specific subsystems — boundaries, interfaces, implementation details |

A well-maintained `context/` folder can give you everything needed for a profile entry without reading any source code. If it exists and is current, start here and only go deeper if gaps remain.

### 2. README.md

The project's public face. Extract:
- Project summary and purpose
- Architecture overview (if present)
- Feature list
- Technology choices and rationale
- Current status and roadmap

**Caveat:** READMEs can be aspirational. Cross-reference claims against dependency manifests and source code. If the README says "supports distributed mode" but no distributed code exists, the profile entry should not claim it.

### 3. Dependency manifests (ground truth for tech stack)

These tell you what the code actually uses, regardless of what the README claims:

| File | Language/Framework |
|------|-------------------|
| `Cargo.toml` | Rust — crates, features, edition |
| `package.json` | JavaScript/TypeScript — dependencies, devDependencies |
| `pyproject.toml` / `requirements.txt` / `setup.py` | Python — packages |
| `go.mod` | Go — modules |
| `pom.xml` / `build.gradle` | Java — dependencies |

Read the dependency list end to end. Each dependency tells you something:
- `tokio` → async Rust
- `ratatui` → terminal UI
- `rusqlite` → SQLite integration
- `reqwest` → HTTP client
- `serde` → serialisation
- `bevy` → game engine / ECS

When the README and the dependency manifest disagree, trust the manifest.

### 4. GitHub API metadata

Use WebFetch on the GitHub API to get:
- `https://api.github.com/repos/{owner}/{repo}` — description, language, topics, stars, forks, created_at, pushed_at, size
- `https://api.github.com/repos/{owner}/{repo}/contents/` — top-level directory listing
- `https://api.github.com/repos/{owner}/{repo}/commits?per_page=5` — recent commit activity

Key signals:
- **`pushed_at`** — when was the last commit? If months ago, the project may be paused or abandoned.
- **`language`** — primary language by bytes of code
- **`topics`** — author-applied tags (often useful for domain classification)
- **`size`** — rough indicator of project scale (in KB)

### 5. Source code (targeted, not exhaustive)

Only read source files when the above sources leave gaps. Target:
- **Entry point** (`main.rs`, `main.py`, `index.ts`) — overall structure
- **Module declarations** (`lib.rs`, `mod.rs`) — architecture shape
- **Core modules** — the files that implement the project's central logic
- **Test files** — what's tested reveals what's important

The goal is targeted assessment. Read 3-5 key files, not the entire codebase. You're writing a profile entry, not doing a code review.

---

## Assessing technical depth

The **technical highlights** field in projects.md is the most important output of a scrape. It's what differentiates "I built a web app" from "I built a lock-free order book with slab allocation and HDR latency histograms."

### What to look for

**Architecture decisions:**
- How is the system structured? What are the layers/modules and how do they communicate?
- Are there clear module boundaries or is everything in one file?
- Is there a deliberate separation of concerns?

**Interesting problems solved:**
- Performance engineering (lock-free structures, zero-copy, SIMD, cache-aware design)
- Correctness constraints (ordering guarantees, atomicity, determinism)
- Scale handling (concurrent processing, batching, efficient data structures)
- Domain-specific challenges (financial mathematics, ML inference, protocol parsing)

**Engineering sophistication signals:**
- Custom data structures or algorithms (not just using library defaults)
- Measurement and instrumentation (benchmarks, latency histograms, profiling)
- Error handling strategy (Result types, error classification, recovery)
- Testing approach (unit tests, integration tests, property tests, golden tests)

**What the code does that's non-trivial:**
- Implements something from scratch that most people use a library for
- Handles a constraint that makes the problem harder (real-time budget, memory limit, no-unsafe, no-dependency)
- Connects multiple complex domains (ML + systems, finance + infrastructure)

### What NOT to highlight

- Standard library usage or basic framework patterns
- "Uses React for the frontend" (unless the frontend is itself technically interesting)
- Generic descriptions of the project type ("a web application", "a CLI tool")
- Features that are planned but not implemented
- Aspirational performance claims without measurement

### The depth test

For each technical highlight, ask: "Could this sentence describe hundreds of other projects, or is it specific to this one?"

- **Generic (bad):** "Uses async Rust for concurrent operations"
- **Specific (good):** "Amortises PPO training across ticks to maintain smooth 60 Hz rendering with 8 concurrent cars, discovered and fixed a 43x performance regression from nested Vec weight storage by switching to flat contiguous row-major layout"

The specific version tells you something about the engineer. The generic version tells you nothing.

---

## Assessing project status

Status must reflect the repo's actual state, not aspirational README claims.

| Status | Evidence |
|--------|----------|
| **Completed** | Feature-complete, README describes working features, recent commits are polish/fixes not new features, or no recent commits because it's done |
| **In Progress** | Active commits within the last 2-3 months, features being added, roadmap items in progress |
| **In Progress (not enough interest)** | Has substantial code but commits have slowed or stopped. The user started it, built something real, but shifted focus. Not abandoned — just deprioritised |
| **Paused** | Last commit 3-6 months ago, clear unfinished work, but the codebase is functional |
| **Abandoned** | Last commit 6+ months ago, unfinished, no recent activity |

**Important:** "In Progress (not enough interest)" is a legitimate status in this project. The user has several exploratory projects that have real code but aren't actively being developed. Don't inflate these to "In Progress" or deflate them to "Abandoned" — use the honest middle ground.

---

## Multi-repo scraping

When the user says "scrape all my repos" or "update everything":

1. Read `profile/projects.md` to get the current project inventory
2. For each project with a URL, check the repo's `pushed_at` date against the profile entry's apparent freshness
3. Prioritise repos where the profile entry seems stale (project has had significant commits since the entry was written)
4. Skip repos where the profile entry is already accurate and current
5. Report which projects were updated, which were skipped, and why

Don't re-scrape a project whose entry already captures the current state of the code. That wastes time and risks introducing unnecessary rewrites.

---

## Cross-referencing across profile files

A scrape can affect multiple profile files. After scraping, check for consistency:

| If you found... | Update... |
|-----------------|-----------|
| New technology not in skills.md | `profile/skills.md` — add with honest proficiency level |
| Existing skill used more deeply | `profile/skills.md` — consider upgrading proficiency level |
| Project covers a known gap | `profile/portfolio-gaps.md` — note that the gap is partially or fully addressed |
| Technology the market asks for | `profile/portfolio-gaps.md` — add as a strength or note closure opportunity |
| Project demonstrates a domain | `profile/skills.md` — domains section, with appropriate depth level |
