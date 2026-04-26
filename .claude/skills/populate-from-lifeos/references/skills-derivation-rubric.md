# Skills Derivation Rubric

## Table of Contents

- [Purpose](#purpose)
- [What This Rubric Rejects](#what-this-rubric-rejects)
- [The Six Tables](#the-six-tables)
- [Per-Table Definitions and Boundaries](#per-table-definitions-and-boundaries)
- [The Four Proficiency Bands](#the-four-proficiency-bands)
- [The Five-Dimension Scoring Model](#the-five-dimension-scoring-model)
- [Mapping Dimensions to Bands](#mapping-dimensions-to-bands)
- [The Cross-Domain Transfer Lift](#the-cross-domain-transfer-lift)
- [Evidence Reading Protocol](#evidence-reading-protocol)
- [Output Schema for skills.md](#output-schema-for-skillsmd)
- [Anti-Puffing for the Skills File](#anti-puffing-for-the-skills-file)
- [Worked Calibration Examples](#worked-calibration-examples)

---

## Purpose

This is the contract for the single skills-derivation subagent dispatched in Phase 5. The orchestrator embeds this entire file verbatim in the dispatch prompt. The agent reads every per-project file in `cernio/profile/projects/`, applies the rubric below, and writes `cernio/profile/skills.md`.

The audience for `skills.md` is the same as for the per-project files: future grading agents (`grade-companies`, `grade-jobs`) and the `prepare-applications` skill. Generic skills entries (*"Familiar with Python"*) are useless for grading. Specific entries with calibrated proficiency and concrete evidence (*"Python — Comfortable. Used in NeuroDrive analytics pipeline (16-field tick capture, 25 episode aggregates, diagnostic Markdown report generation), credit-card fraud detection (XGBoost gradient boosting, precision/recall threshold tuning), and the merged-then-closed tinygrad ONNX LSTM operator implementation. Demonstrated outside framework usage in tinygrad operator work; framework breadth across PyTorch, TensorFlow, scikit-learn, XGBoost"*) are.

---

## What This Rubric Rejects

The user has named these as failure modes — the rubric exists specifically to avoid them:

| Failure mode | Why it fails |
|---|---|
| **Lines of code as proficiency proxy** | Bad code is long. Inefficient functions look like depth. A 500-line Rust file demonstrating one anti-pattern does not beat a 100-line file demonstrating ownership-driven design. |
| **Project count as proficiency proxy** | One deep project beats five shallow ones. Multiple projects can repeat the same technique without adding depth. |
| **Library count as proficiency proxy** | Library-free projects are sometimes deliberately library-free (NeuroDrive's no-ML-framework choice). Counting libraries used misreads ascetic projects as weakness. |
| **"Used in N projects" → automatic Comfortable** | Using a tool in N shallow contexts proves use, not competence. |
| **README pitch language** | The GitHub README is curation, not source. Skills derived from README claims are derived from marketing, not evidence. |

The rubric replaces these with multi-dimensional evidence reading per the dimensions enumerated below.

---

## The Six Tables

`skills.md` is structured as six distinct tables, each with its own scope. Categories do not overlap.

| Table | Scope | Examples |
|---|---|---|
| **1. Programming Languages** | Languages the user writes code in | Rust, Python, TypeScript, JavaScript, C++, Java, OCaml |
| **2. Frameworks** | Application-shaping scaffolds, application frameworks, web frameworks, runtimes that define program structure | Tauri, React, Tokio, Bevy, Ratatui, libGDX, LangChain, FastAPI |
| **3. Libraries** | Discrete libraries used for specific functionality, not full app frameworks | rusqlite, ONNX Runtime, num-bigint, ndarray, scikit-learn, XGBoost, DEAP, NEAT, chrono, serde, reqwest |
| **4. Engines and runtimes** | Domain-specific execution engines: ML inference engines, game engines, GPU runtimes, simulation runtimes | ONNX Runtime, wgpu, CUDA, PyTorch (as a runtime, not a library), TensorFlow (runtime), JAX |
| **5. Tools and platforms** | Developer tooling, infrastructure, observability, version control, package managers | Git, GitHub, SQLite (as data store), Docker (when present), CI systems, Wireshark, Nmap, Linux toolchain |
| **6. Concepts and domains** | Conceptual / domain expertise demonstrated in projects, separate from any specific tool | Lock-free concurrency, market microstructure, AMM mathematics, reinforcement learning, biological plasticity, compiler design, content-addressed storage, network protocols, cryptography, computer vision, semantic search, quantitative risk modelling, desktop application development, local-first architecture |

The boundary between Table 2 (Frameworks) and Table 3 (Libraries) is sometimes fuzzy. Heuristic: if the tool *shapes the program* (entry-point structure, threading model, lifecycle, plugin system), it is a Framework. If the tool *provides functionality the program calls into*, it is a Library. Tokio is a Framework (defines the async runtime structure). Reqwest is a Library (provides HTTP client functionality the program calls).

The boundary between Table 4 (Engines/Runtimes) and Tables 2/3 is similar. Engines are heavyweight execution environments with their own execution model — ONNX Runtime executes inference graphs; Bevy executes ECS systems; PyTorch executes tensor graphs. Frameworks shape the program's structure; engines execute on the program's behalf.

Table 6 (Concepts and Domains) is the most important table for grading fit. A grader assessing a role about lock-free trading systems can match against *"Lock-free concurrency"* and *"Market microstructure"* even if the user has not used the specific language the role requires. Concepts transfer across stacks; tools do not.

---

## Per-Table Definitions and Boundaries

### Table 1 — Programming Languages

Each row: a language + proficiency band + evidence summary.

**What counts as evidence:**
- Central to one or more projects (the project's primary language).
- Used substantively in a project (not just one config file or one small script).
- Specific techniques demonstrated (ownership patterns, FFI, trait-based design, async runtime usage, memory management, concurrency primitives).

**What does not count:**
- A `.gitattributes` mention.
- A single shell-out script.
- README claims without project evidence.

### Table 2 — Frameworks

Each row: a framework + proficiency band + evidence summary.

**What counts as evidence:**
- Used to structure one or more projects (Tauri shell for a desktop app, Tokio runtime for an async pipeline).
- Specific framework features used substantively (Tokio's spawn / select / mpsc; Bevy's plugin composition + system ordering sets; React's hooks + context).

**What does not count:**
- Listed as a dependency without project evidence of use.
- A framework demonstrated only by its hello-world equivalent.

### Table 3 — Libraries

Same standard as frameworks but for finer-grained functionality. A library used to perform one specific task in one project is Familiar, not Comfortable.

### Table 4 — Engines and Runtimes

Each row: an engine + proficiency band + evidence summary. The depth here is about understanding the engine's execution model, not just using its API. ONNX Runtime as a black box is Familiar. ONNX Runtime with custom operator implementation (the tinygrad LSTM PR) is Comfortable.

### Table 5 — Tools and Platforms

Each row: a tool + proficiency band + evidence summary. This table is typically the broadest because tools have the lowest depth threshold — daily Git use is Comfortable; never having used Docker is fine and means Docker simply does not appear.

### Table 6 — Concepts and Domains

Each row: a domain + proficiency band + evidence summary.

This is the most judgment-intensive table. The agent identifies concepts the projects *demonstrate*, not concepts the projects *mention*. A project that uses CLIP for image search demonstrates "computer vision" only if the project file's Architecture section shows the user understood the embedding model, the cosine similarity decisions, the tokeniser. A project that calls `from torchvision import` does not demonstrate computer vision per se — it demonstrates library use.

The concepts table contains at least one entry per major project domain demonstrated in `cernio/profile/projects/*.md`. Domains the per-project files evidence (systems, ML, finance, networking, compilers, desktop apps, local-first architecture, etc.) each get an entry; domains with no project evidence do not get entries. A Concepts table with fewer entries than the portfolio's distinct demonstrated domains is under-counted; the agent's evidence trace cites which projects evidence each domain.

---

## The Four Proficiency Bands

| Band | Definition |
|---|---|
| **Proficient** | Demonstrated substantive depth in multiple substantial contexts. The user can solve novel problems in this area, not just apply known patterns. Evidence: central to multiple flagship-or-active projects with non-trivial technique work, OR central to one project with research-level depth. |
| **Comfortable** | Working competence demonstrated in a substantial context. Can use this in production-like work without supervision. Evidence: central to one substantial project, OR substantively used across multiple projects, with the project files showing concrete technique work (not just "uses X"). |
| **Familiar** | Working knowledge demonstrated in a limited or single context. Could pick this up in a new project, but evidence of depth is bounded. Evidence: used in one project's scope with limited surface area, OR used substantively in an abandoned-early or paused project. |
| **Beginner** | Exposure-level. Coursework, single small project, or peripheral use. Use sparingly — Beginner-level entries crowd the file without much grading signal. Skip when the evidence is too thin even for Familiar. |

The bands are not numerical thresholds. They are evidence-anchored judgments. The dimensions below are how the agent reaches a judgment.

---

## The Five-Dimension Scoring Model

For each candidate entry (a language, framework, library, engine, tool, or concept), evaluate against five dimensions:

### 1. Depth of work demonstrated

Read the project files where the entry appears. In *each* project's "What is currently built", "Architecture", and "Demonstrated skills" sections, what specifically does the user do with this entry? Two examples:

- **Rust → NeuroDrive:** "8 Bevy subsystems implementing ECS architecture, fixed-timestep simulation, plugin composition, system ordering sets, world-space debug overlays, UI panels. ~393 KB across 67 source files. Custom fixed-flat-storage pattern after 43x performance regression discovery." High depth.
- **TypeScript → Personal Website:** "Particle simulation written from scratch rather than using a physics library." Moderate depth in one narrow context.

Higher depth → higher band.

### 2. Conceptual complexity of what was built

Easy projects in a language do not demonstrate the language. Hard projects do. A todo-list app in Rust does not demonstrate Rust expertise. A lock-free order book in Rust does. The conceptual difficulty of what was built directly shapes proficiency.

Heuristics for "hard":
- Concurrency-correctness work (lock-free, wait-free, atomic primitives).
- Memory-management work (custom allocators, FFI lifetimes, zero-copy).
- Algorithm-design work (custom RL implementation, custom tokeniser, hand-rolled SVG charting beyond library defaults).
- Domain-specific complexity (Uniswap V3 sqrtPriceX96 decoding via BigUint arithmetic; price-time priority matching with saturating fill logic).
- Performance-engineering work (43x regression hunt, profile-driven optimisation, frame-budget management).

A project file's "Demonstrated skills" section that mentions any of these signals high conceptual complexity for the relevant tools/concepts.

### 3. Completion stage

Active and complete projects carry more weight than abandoned-early projects. An abandoned project where the user got past the type system but before the matching engine demonstrates work, but less than a completed project.

| Status | Weight in proficiency assessment |
|---|---|
| Active (in progress, recent activity) | Full weight |
| Complete (shipped, no more planned work) | Full weight |
| Paused (started, paused for known reason, may resume) | 75% weight |
| Dormant (no activity for 6+ months, no clear pause reason) | 50% weight |
| Abandoned (deliberately stopped, no resume planned) | 25% weight, but use in proficiency proportional to what was actually built before abandonment |

### 4. Cross-domain transfer

Same tool used across multiple problem domains demonstrates broader proficiency than the same tool used five times for the same kind of problem.

Example: Rust used in Cernio (TUI + pipeline + database), NeuroDrive (game engine + ML training), Aurix (desktop + DeFi RPC + financial math), Image Browser (Tauri + ML inference), Nyquestro (lock-free type system + matching engine) demonstrates *cross-domain* Rust proficiency. Same Rust LOC count concentrated in five trading systems demonstrates depth in one domain, not breadth.

The Concepts table benefits from cross-domain transfer the most — if a domain shows up across multiple unrelated projects, it is more strongly demonstrated than if it shows up in one.

### 5. Evidence specificity in the project files

The project file's content quality bounds how high the band can go. If the project file's Architecture section says *"uses Rust for performance"* with no specifics, the agent cannot give Rust a Proficient rating from that project alone. The bound: bands are limited by what the source evidence supports.

This dimension catches the case where a project might have substantive work but the LifeOS source has not captured the detail. In that case the agent records a lower band and notes the gap in the WIDND for the user to address (improve the LifeOS source for that project).

---

## Mapping Dimensions to Bands

The agent does not compute a numerical score across the five dimensions. The agent reads the evidence holistically and judges. Heuristic patterns:

| Pattern across the five dimensions | Likely band |
|---|---|
| High depth + high complexity + active/complete + cross-domain + specific evidence | Proficient |
| High depth + high complexity + active/complete + single domain + specific evidence | Proficient |
| Moderate depth + moderate complexity + active/complete + specific evidence | Comfortable |
| High depth + moderate complexity + paused/dormant + specific evidence | Comfortable |
| Moderate depth + moderate complexity + abandoned + specific evidence | Familiar |
| Limited depth + limited complexity + any status + specific evidence | Familiar |
| Limited depth + thin evidence in source files | Familiar (with WIDND note about source thinness) |
| Thinner than Familiar | Skip; do not include |

The mapping is illustrative. The agent applies judgment, not a lookup table.

---

## The Cross-Domain Transfer Lift

A specific case worth naming: when a tool appears in projects spanning ≥3 distinct domains (e.g. trading + ML + desktop apps + databases), it earns a one-band lift relative to its single-project depth assessment. The reason: a tool that survives translation across unrelated problem domains demonstrates broader fluency than the same tool concentrated in one domain.

Worked example: Rust appears in Image Browser (desktop ML inference), NeuroDrive (game engine + RL), Aurix (DeFi analytics), Nyquestro (HFT-style systems), Cernio (TUI + ATS pipeline + SQLite). That is 5 domains. Even if no individual project's depth alone would justify Proficient, the cross-domain breadth would lift Rust to Proficient. This is the user's own framing: "depth across domains" is the core signal, not LOC.

This lift applies primarily to Languages, Frameworks, and Concepts. It applies less to Libraries (which are usually domain-bounded by design). It does not apply to Tools (Git used everywhere does not demonstrate cross-domain Git proficiency in the same way).

---

## Evidence Reading Protocol

The agent reads every per-project file in `cernio/profile/projects/` end-to-end. No summarising shortcuts. The agent's response evidence block lists every project file consumed:

```markdown
## Evidence Block

| Project file | Lines | Verbatim last line |
|---|---|---|
| cernio/profile/projects/cernio.md | <N> | "<exact text>" |
| cernio/profile/projects/image-browser.md | <N> | "<exact text>" |
| ... | ... | ... |
```

Same Tier-3 anchor as the per-project agents — partial reads cannot produce verbatim last lines.

For each candidate skill entry, the agent's reasoning trace (in its return summary, not in skills.md itself) names which project files contributed and what evidence each contributed. This trace is what the orchestrator pastes into the Phase 8 summary so the user can audit any band assignment.

---

## Output Schema for skills.md

```markdown
# Skills

> Derived from project files in `profile/projects/`. Last synced: <YYYY-MM-DD>.
> Proficiency bands: Proficient | Comfortable | Familiar | Beginner.
> Per `populate-from-lifeos` rubric — evidence-anchored, not LOC-based.

---

## Programming Languages

| Language | Proficiency | Evidence (specific projects + what they demonstrate) |
|----------|-------------|------------------------------------------------------|
| <Language> | <Band> | <Project A: specific technique demonstrated. Project B: specific technique. Cross-domain note if applicable.> |

---

## Frameworks

| Framework | Proficiency | Evidence |
|-----------|-------------|----------|
| <Framework> | <Band> | <Specific use evidence per project> |

---

## Libraries

| Library | Proficiency | Evidence |
|---------|-------------|----------|
| <Library> | <Band> | <Specific use evidence per project> |

---

## Engines and Runtimes

| Engine | Proficiency | Evidence |
|--------|-------------|----------|
| <Engine> | <Band> | <Specific use evidence per project> |

---

## Tools and Platforms

| Tool | Proficiency | Evidence |
|------|-------------|----------|
| <Tool> | <Band> | <Specific use evidence per project> |

---

## Concepts and Domains

| Domain | Proficiency | Evidence |
|--------|-------------|----------|
| <Domain> | <Band> | <Specific demonstrated work per project> |

---

## Methodologies and Soft Skills

[A short prose section, drawn from project file patterns rather than from a separate LifeOS source. Examples of what to include: iterative milestone-driven development, test-driven validation for correctness-critical work, plain-text inspectable artefacts, cross-disciplinary synthesis. The examples are illustrative — derive from the actual project portfolio's patterns, not from this list.]
```

The Evidence column is mandatory and substantive. *"Used in NeuroDrive"* fails the standard. *"Central to NeuroDrive — handwritten PPO with clipped surrogate objective and GAE, asymmetric actor-critic (2×64 actor, 2×128 critic), 8-subsystem Bevy ECS architecture, 43x performance fix via flat row-major weight storage"* is the standard.

---

## Anti-Puffing for the Skills File

Embed verbatim in the dispatch prompt:

> *"Proficiency reflects what the projects demonstrate, not what the user has been exposed to. A language used in one abandoned-early project is Familiar at most. A library named in passing is not a skill. Calibrate against the dimensions in the rubric, not against intuition or generosity. When uncertain whether evidence supports a higher band, choose the lower band and note the uncertainty in the return summary so the orchestrator can flag it for user review."*

Concrete rules:

1. **No band higher than the source evidence supports.** The project files are the ceiling.
2. **No skill entry without project evidence.** If the user mentioned a language in an interview but never used it in a project, it does not appear in skills.md.
3. **Evidence column quotes specific project artefacts.** Names of architectures, techniques, performance numbers, design decisions — not abstractions.
4. **When two project files contradict (e.g. one says active, one says abandoned), trust the more recent file** and note the contradiction in the return summary.
5. **Do not invent skills the project files do not surface.** If no project file mentions cryptography, cryptography does not appear in the Concepts table even if a coursework module covered it.

---

## Worked Calibration Examples

These are illustrative judgments showing the rubric in action. The agent applies similar reasoning to each candidate entry.

**Example 1 — Rust (Languages table):**

- Depth: high (multiple projects with non-trivial technique work — lock-free design, async pipelines, FFI, memory management, performance hunting).
- Complexity: high (matching engine, custom PPO, ABI hand-encoding, lock-free order book, ECS subsystems).
- Status: mixed but mostly active or complete (Image Browser complete, Cernio active, NeuroDrive active, Aurix in progress, Nyquestro in progress).
- Cross-domain: very high (TUI + database + game engine + ML + DeFi + HFT-style systems + desktop apps = 5+ domains).
- Evidence: specific in every project file.

→ **Proficient**, with cross-domain lift confirming.

**Example 2 — C++ (Languages table):**

- Depth: limited (Chrona and Tectra both abandoned, Tectra never reached implementation depth, Chrona reached the content-addressed storage layer).
- Complexity: moderate-to-high in concept (version control internals, trading infrastructure design) but limited in execution.
- Status: both abandoned.
- Cross-domain: two domains (VCS, trading infrastructure) but both abandoned-early.
- Evidence: design ambition is detailed in LifeOS; implementation depth is bounded.

→ **Familiar**. The depth-status combination caps the band; design ambition is not implementation depth.

**Example 3 — Lock-free concurrency (Concepts table):**

- Depth: high in Nyquestro (designed for lock-free order book, currently has type system foundation).
- Complexity: very high (this is research-level systems work).
- Status: Nyquestro in progress.
- Cross-domain: appears in Nyquestro and conceptually in NeuroDrive (concurrent rollout buffer though not lock-free).
- Evidence: specific in Nyquestro's Architecture (intrusive linked list per price level, saturating fill logic, zero-allocation event frames).

→ **Comfortable**. The actual lock-free implementation is the foundation layer, not the matching engine yet, so the band cannot be Proficient until more is built. The Concepts table notes this honestly.

**Example 4 — Reinforcement learning (Concepts table):**

- Depth: very high in NeuroDrive (handwritten PPO, GAE, clipped surrogate, asymmetric actor-critic, reward engineering under entertainment constraints, 16-field tick analytics, 25-field episode analytics, 10-section diagnostic reports).
- Complexity: research-level (no framework, every learning rule from first principles, biological plasticity transition planned).
- Status: NeuroDrive active.
- Cross-domain: appears in AsteroidsAI as well (NEAT, GA, ES, GNN+SAC comparison).
- Evidence: extensively specific.

→ **Proficient**. This is the strongest evidence in the portfolio for any concept.

**Example 5 — Docker (Tools table):**

- No project file mentions Docker.

→ **Skip**. Do not include. The user's portfolio-gaps.md is the place to flag the absence; skills.md only lists what the projects demonstrate.

These examples show the rubric is judgment-driven, evidence-anchored, and willing to under-rate when evidence is bounded. That under-rating is a feature — it preserves the file's grading utility downstream.
