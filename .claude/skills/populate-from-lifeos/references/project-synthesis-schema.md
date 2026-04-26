# Per-Project File Synthesis Schema

## Table of Contents

- [Purpose and Audience](#purpose-and-audience)
- [The Read-Everything Obligation](#the-read-everything-obligation)
- [The Evidence Block (Tier-3 Anchor)](#the-evidence-block-tier-3-anchor)
- [Per-Project File Structure](#per-project-file-structure)
- [Section Intent Descriptions](#section-intent-descriptions)
- [Anti-Puffing Rules](#anti-puffing-rules)
- [Depth Standards](#depth-standards)
- [Worked Example — Skeleton vs Comprehensive](#worked-example--skeleton-vs-comprehensive)
- [Output Contract Returned to Orchestrator](#output-contract-returned-to-orchestrator)

---

## Purpose and Audience

This file is the contract for the per-project synthesis subagents dispatched in Phase 3. The orchestrator embeds this entire file verbatim in every per-project agent's dispatch prompt — the agent has no other source for the schema, evidence requirements, or anti-puffing principle.

The audience for the synthesised file (`cernio/profile/projects/<name>.md`) is *future grading agents* — `grade-companies` and `grade-jobs` read these files when assessing whether a Cernio user-evidence claim is supported by what the project demonstrates. Generic content forces those graders to fall back on the README pitch language, which defeats the entire LifeOS-as-canonical-source design. Specific content lets graders cite the project file directly: *"role requires lock-free concurrency; Caner's Nyquestro project file states 'lock-free intrusive doubly-linked list per price level' in §Architecture"*.

The ceiling: a future grader reading only the project file (no LifeOS access, no README) should be able to assess fit against any role with no guesswork.

---

## The Read-Everything Obligation

The agent reads **every file** in `LifeOS/Projects/<Name>/`, including subfolders, recursively. No filename-based triage. No "the Overview is enough". The LifeOS folder structure follows the Option C schema (Overview, Architecture, Systems, Decisions, Gaps, Roadmap, plus Work/), and every file contributes — Decisions.md captures the *why* behind technology choices that Architecture.md only states; Work/ captures in-flight scope that Roadmap.md does not yet codify; Gaps.md surfaces honest limitations that the README would never name.

**Read protocol:**

1. List the folder: `gh api repos/Capataina/LifeOS/contents/Projects/<URL-encoded-name> --jq '[.[] | {name: .name, type: .type, path: .path}]'`. Note the `type` for each entry (file vs dir).
2. For every entry of type `file` ending in `.md`, fetch and base64-decode the content.
3. For every entry of type `dir`, recurse: list, fetch every `.md` file inside.
4. Track every read in the evidence block (see next section).

The folder may contain non-markdown files (images, PDFs, attachments). Skip these silently — they are out of scope for synthesis.

If a file cannot be fetched (404, API error, permission issue), record it in the evidence block with the error. Do not silently drop it. Do not retry indefinitely (one retry permitted; second failure is recorded as unreadable).

---

## The Evidence Block (Tier-3 Anchor)

The agent's response to the orchestrator MUST include an evidence block listing every LifeOS file consumed. Format:

```markdown
## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| Projects/<Name>/Overview.md | 87 | "<exact text of the last line>" |
| Projects/<Name>/Architecture.md | 142 | "<exact text>" |
| Projects/<Name>/Systems/Storage.md | 56 | "<exact text>" |
| Projects/<Name>/Decisions.md | 38 | "<exact text>" |
| ... | ... | ... |
| Projects/<Name>/Work/Q2-roadmap.md | UNREADABLE | "API returned 404" |
```

The verbatim last line is the Tier-3 evidence anchor (per the skill-creator's research/19-verification-gates.md). A partial read of a file cannot produce its last line. An agent that paraphrases ("end of architecture section") or substitutes ("(content omitted for brevity)") fails the evidence requirement and the synthesis is treated as incomplete.

The orchestrator validates: every entry of type `file` listed in the folder enumeration appears in the evidence block, with either a verbatim last line or an explicit `UNREADABLE` marker with reason. Any silent omission triggers a re-dispatch of that agent with the gap named.

---

## Per-Project File Structure

```markdown
---
name: <Project Name as on the README>
status: <active | dormant | paused | abandoned>
source_repo: <GitHub URL from the README, or null for OSS-only entries>
lifeos_folder: Projects/<Name>
last_synced: <YYYY-MM-DD>
sources_read: <integer count of LifeOS files consumed>
---

# <Project Name>

## One-line summary

[A single sentence capturing what the project is, written for a senior engineer skimming a portfolio. Not the README pitch — the actual technical positioning. "Lock-free limit order book matching engine in safe Rust exploring wait-free concurrent data structures" beats "high-performance trading system".]

## What it is

[3-5 sentences. The fuller framing of the project's purpose, scope, and ambition. Drawn from LifeOS Overview.md primarily. Distinguish what the project *is designed to do* from what it *currently demonstrates* — those differ for in-progress work and the distinction matters for grading fit.]

## Architecture

[Comprehensive. Major components, their responsibilities, dependency direction, key abstractions. Every component named here is sourced from either LifeOS Architecture.md or Systems/<component>.md — components without LifeOS evidence are not synthesised into this section. Use diagrams (ASCII trees, dependency graphs) where they clarify shape. This is the section graders will cite when assessing whether the user has built what a role requires.]

## Subsystems and components

[Per-subsystem breakdown when the project has multiple. One subsection per subsystem — name, responsibility, technologies, current state, key files. Skip this section entirely if the project is single-component. Drawn from LifeOS Systems/.]

## Technologies and concepts demonstrated

[Concrete, evidence-anchored, structured by category. The skills-derivation agent (Phase 5) reads this section to feed skills.md, so its quality directly shapes the user's profile fingerprint.]

### Languages
- **<Language>** — <how it is used in this project, with specifics: which files, which subsystems, what depth of usage>

### Frameworks and libraries
- **<Library>** — <what it provides; whether it is core or peripheral to the project>

### Runtimes / engines / platforms
- **<Engine>** — <e.g. "Bevy 0.18 ECS for the simulation loop, with custom plugin composition across 8 subsystems">

### Tools
- **<Tool>** — <build tooling, profilers, debuggers, infra>

### Domains and concepts
- **<Domain>** — <e.g. "lock-free concurrency: implements wait-free intrusive doubly-linked list per price level"; "PPO with clipped surrogate objective and GAE">

## Key technical decisions

[Drawn from LifeOS Decisions.md. Per major decision: what was chosen, what was rejected, why. Connect each decision to the architecture section so the *why* sits next to the *what*. Decisions like "no ML framework dependency" or "FFI to ONNX Runtime instead of pure-Rust inference" carry portfolio signal — surface them.]

## What is currently built

[Distinct from "What it is". This section is the *honest current state* — what code actually exists and works today, not what the design aspires to. For in-progress projects, this is the most important section: the design ambition can be massive while the implemented scope is narrow, and grading honesty depends on the distinction. Cite specific subsystems, file counts, line counts where available from LifeOS Overview's scale numbers.]

## Current state

[Status (matches frontmatter), last meaningful activity, what is currently in flight (from LifeOS Work/). One paragraph max — this is the temporal context, not a roadmap.]

## Gaps and known limitations

[Drawn from LifeOS Gaps.md, filtered to what is career-relevant. Honest gaps strengthen the profile rather than weakening it — they show technical maturity. Frame each gap as a fact, not as an apology.]

## Direction (in-flight, not wishlist)

[From LifeOS Roadmap.md, but ONLY items that are actively being worked on or have a concrete near-term plan. Aspirational backlog items belong in LifeOS, not in the per-project profile file. The distinction: *"currently implementing the matching engine core after completing the type system"* belongs here; *"someday will add a market-making strategy"* does not.]

## Demonstrated skills

[The deliberate signal-amplification section. What this specific project proves the user can do. Specific, not generic. *"Implements a complete RL training environment from scratch with no ML framework dependency, including handwritten PPO with clipped surrogate objective"* is a demonstrated skill. *"Familiar with machine learning"* is not.]

---

## Evidence Block

| Path | Lines | Verbatim last line |
|---|---|---|
| <every file in the LifeOS folder, with line count and verbatim last line> |
```

---

## Section Intent Descriptions

(Repeated here so the per-section bracket placeholders above are self-explanatory when the agent fills them in.)

| Section | Intent | Anti-pattern |
|---|---|---|
| One-line summary | Technical positioning sentence | README pitch sentence |
| What it is | Fuller scope and ambition | Pitch language |
| Architecture | Components, dependencies, abstractions | Vague "modular design" |
| Subsystems | Per-subsystem detail | Skipped section claiming "subsystems are described in Architecture" |
| Technologies and concepts | Categorised inventory with depth-of-use per item | Flat list of names |
| Key technical decisions | Choices made, alternatives rejected, why | Restating what was chosen without the why |
| What is currently built | Honest implemented scope | Restating the design ambition |
| Current state | Temporal context, in-flight items | Roadmap items |
| Gaps and known limitations | Honest current limitations | Apologetic framing or omission |
| Direction | Active in-flight work | Aspirational wishlist |
| Demonstrated skills | Specific provable capabilities | Generic skill labels |

---

## Anti-Puffing Rules

The single most important constraint on synthesis. Embed the following imperative verbatim in every per-project agent prompt:

> *"Describe what the project demonstrates from its LifeOS folder content. Do not interpolate from the GitHub README's pitch language. Do not invent technologies, integrations, or scale numbers not present in the LifeOS source. If a section the schema asks for has no source evidence, state 'no source evidence in LifeOS' for that section rather than fabricating content."*

Concrete rules derived from the principle:

1. **No technology mentions without LifeOS evidence.** If LifeOS Architecture.md does not name `wgpu`, the per-project file does not name `wgpu` even if the README does. The skill is downstream of LifeOS, not of the README.
2. **Scale numbers come from LifeOS Overview.md or git inspection.** Lines of code, file counts, test counts, commit counts — only reproduce numbers LifeOS has captured (typically in Overview.md frontmatter or a Scale section). Do not invent numbers from the README's pitch.
3. **Status is from LifeOS, not from inference.** The frontmatter `status` field reflects what LifeOS Overview.md states. If LifeOS says "paused" and the README's pitch language reads as "active", the file says "paused".
4. **Aspirational language is not skill evidence.** *"Will support distributed deployment"* is roadmap, not demonstrated skill. *"Implements the type system, matching engine core pending"* is honest current scope.
5. **No section is silently dropped.** If LifeOS has no Decisions.md, the Key Technical Decisions section in the per-project file says *"no decisions captured in LifeOS source"* explicitly.

---

## Depth Standards

The per-project file's depth is **proportionate to the project's depth in LifeOS**. A project with extensive LifeOS documentation produces a long, dense per-project file. A project with a single Overview.md produces a shorter, sparser one. Padding a sparse project to match a deeper one is anti-puffing in disguise.

Heuristic: the per-project file should consume between 30% and 80% of the total content in the LifeOS source folder, depending on how much LifeOS material is signal vs scaffolding. A 5KB Overview-only project might produce a 2-3KB per-project file. A 50KB multi-system flagship might produce a 20-30KB file. The Architecture and Technologies sections are typically the heaviest.

What a future grader needs from this file (the depth target):

- Enough technology specificity that they can match against any role's stack requirements.
- Enough decision-level reasoning that they can assess whether the user has thought through the design space, not just used libraries.
- Enough current-state honesty that they can tell what is built from what is planned.
- Enough domain-concept signal that they can map the project to roles outside the user's primary stack (e.g. mapping a Rust trading engine to a C++ HFT role on transferable concepts).

---

## Worked Example — Skeleton vs Comprehensive

For an imaginary `Projects/Echo/` containing `Overview.md`, `Architecture.md`, `Decisions.md`, `Gaps.md`:

**Skeleton (insufficient):**

```markdown
## Architecture

Echo is a TUI-based note manager built in Rust. It uses a SQLite database for storage.
```

**Comprehensive (target):**

```markdown
## Architecture

Echo is a single-binary Rust TUI built on Ratatui 0.29 with three subsystems communicating through a shared SQLite store:

- **Editor** (`src/editor/`) — buffer model with rope-based text storage (xi-rope), undo tree, vim-keybinding state machine, syntax highlighting via tree-sitter parsers compiled into the binary at build time.
- **Index** (`src/index/`) — SQLite FTS5 wrapper providing full-text search over notes, with custom tokenisation for code blocks and YAML frontmatter. Index updates are debounced (500ms) and run off-main-thread via tokio::spawn_blocking.
- **Sync** (`src/sync/`) — optional git-backed sync layer. When enabled, commits the notes directory after every save with a generated commit message; pulls before any read operation that crosses session boundaries. The sync layer is feature-gated; default builds exclude it.

Dependency direction: editor and index both read/write the SQLite store; sync wraps the store with a pre/post-write hook. Neither editor nor index depend on sync, so the feature gate cleanly removes the entire layer when disabled.
```

The comprehensive version is the target. It is graspable to a senior engineer in one read; it surfaces specific technologies (Ratatui 0.29, xi-rope, tree-sitter, SQLite FTS5, tokio); it explains dependency direction; it names the feature-gate boundary. A grader assessing fit against a TUI role can cite specifics; against a database role, the FTS5 detail; against an editor-tooling role, the rope structure.

The skeleton version is what comes out of *"summarise the project"* prompts. It is what this skill is designed *not* to produce.

---

## Output Contract Returned to Orchestrator

When the per-project agent completes, it returns to the orchestrator a structured response with three parts:

1. **The output file path written:** `cernio/profile/projects/<lowercase-dashes>.md`.
2. **The evidence block** in the format described above — every LifeOS file consumed, with line count and verbatim last line.
3. **A one-line summary of any anomalies** — files that could not be read, sections of the schema that had no LifeOS source evidence, or any decision the agent had to make under uncertainty (e.g. *"Status set to 'paused' inferred from Overview.md last-modified date 4 months ago; LifeOS source did not state status explicitly"*).

The orchestrator records all three in the run ledger that feeds Phase 8's `sync-summary.md`. Anomalies become WIDND entries; missing evidence-block rows trigger re-dispatch.
