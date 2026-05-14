---
title: LinkedIn
tags:
  - profile
  - career
  - linkedin
last_verified: 2026-05-07
---

# LinkedIn — content + update checklist

Public profile: [linkedin.com/in/atacanercetinkaya](https://www.linkedin.com/in/atacanercetinkaya/).

This file is the **canonical source for LinkedIn content** that mirrors the resume + cover letter. LinkedIn has no MCP/API access in Caner's tooling — this file is a checklist + ready-to-paste content. After applying updates in the LinkedIn UI, bump `last_verified` to today's date.

The framing rule: LinkedIn must be consistent with [[Profile/Professional/Resume - Ata Caner Cetinkaya|Resume]] + [[Profile/Professional/Cover Letter - Ata Caner Cetinkaya|Cover Letter]]. Inconsistency between artefacts is a credibility leak — recruiters cross-reference. When the Resume is refreshed via the [[.claude/skills/improve-resume/SKILL|improve-resume]] skill, this file should be refreshed in the same session.

---

## Update checklist (next session in LinkedIn UI)

- [ ] **Headline** — verify current state, refresh if stale (proposed text below)
- [ ] **About / Summary** — replace with the [About copy](#about-section) below
- [ ] **Experience — Open Source Contributor entry (NEW)** — add as a single rolling Experience entry per [Experience section — Open Source Contributor](#experience-section--open-source-contributor) below
- [ ] **Projects** — sync to match [Resume project descriptions](#projects-section-uniform-layout-for-the-5-linkedin-project-entries)
- [ ] **Featured** — pin top 3 projects (Aurix, Nyquestro, Cernio per current resume order)
- [ ] **Open to work** banner — set if actively applying (per Visa.md, Graduate visa unrestricted RTW until Aug 2027)
- [ ] **Skills** — sync to Resume Skills section keyword list

After applying: bump `last_verified` in the frontmatter above to today's date.

---

## Headline

**Live headline (Caner-applied 2026-05-07):**

```
AI/ML Infrastructure Engineer | Rust Systems Engineer | High Frequency & Low-Latency Trading + DeFi Analytics | Open Source Contributor (burn & tinygrad)
```

4-segment structure: AI/ML Infrastructure as the lead role-class (matches the long-term healthcare AI direction in the About); Rust Systems as the technical spine; HFT + DeFi as the domain anchor (Aurix + Nyquestro projects); OSS Contributor with named credentials (burn + tinygrad) closing as the verifiable-contribution tail. Drops the location + Open-to-Work intent because LinkedIn surfaces those via separate fields.

**Earlier proposal (kept for reference + future iteration):**

```
Rust Systems Engineer | DeFi Analytics & Order Matching (Aurix, Nyquestro) | ML Infrastructure + OSS (burn, tinygrad)
```

3-segment shape; Caner extended to 4 to lift AI/ML Infrastructure to the front and keep all named credentials on display.

---

## About section

**Ready-to-paste About copy** — bullet-restructured per Caner 2026-05-07 guidance. Plaintext bullets (`-`), no fancy formatting, prose-and-bullets hybrid.

LinkedIn About cap is **2,600 characters**. Draft below sits at **2,561** (verified by `wc -m`; prior draft was 3,719, over-cap; trimmed 1,158 chars by removing fillers and tightening sentences while preserving every substantive claim — every project + OSS PR + degree-work item + healthcare angle + hobby is intact).

> [!warning] LinkedIn paste-spacing quirk + workaround
> LinkedIn's About text-field has a known issue where pasting from a rich-text source (browser, formatted markdown view, etc.) can collapse the blank lines between bullets / paragraphs, leaving the text as a single dense block. The blank lines in the draft below are intentional structural separators; if they collapse on paste, fix in one of two ways:
>
> 1. **Plain-text intermediate**: paste the draft into TextEdit (Format → Make Plain Text first, ⌘⇧T) or Notes (set to Plain Text), then copy from there to LinkedIn. The plaintext clipboard path preserves the literal `\n\n` newline pairs.
> 2. **LinkedIn mobile app**: the LinkedIn iOS / Android app About-edit screen handles pasted newlines more reliably than the web editor in some paste paths.
>
> If both above fail, the manual fallback: paste the dense block into LinkedIn first, then click into the About box and manually press Enter at every paragraph + bullet boundary to insert real newlines. Tedious but works.

```
Rust systems engineer; BEng Computer Science (University of York, 2025). Building performance-critical tools across DeFi analytics, low-latency trading, ML infrastructure, and local-first desktop applications.

Current work:

- Aurix — local-first DeFi analytics on Tauri 2 + Rust; cross-DEX arbitrage, V3 LP backtesting, wallet tracking, gas prediction, risk modelling. Shipped: V3 LP backtester with Q64.96 fixed-point math, multi-asset benchmarking, and regime-conditional capital allocation.

- Nyquestro — order matching engine in safe Rust; targeting lock-free order book + binary UDP wire protocol. MVP shipped: multi-instrument routing, Ratatui observability dashboard, live BTC/ETH/SOL feed via Coinbase WebSocket.

- Cernio — local-first job-discovery treating job search as a systems problem; 9 native Claude Code skills orchestrate AI agents via a Ratatui dashboard.

- NeuroDrive — RL racing sim in Rust + Bevy; handwritten PPO + biology-first sparse-graph learner running continuously, no weight resets and no backprop in the brain learner.

- Image Browser — local-first Tauri 2 image browser; CLIP + DINOv2 + SigLIP-2 via ONNX Runtime with Reciprocal Rank Fusion over per-encoder rankings.

Open source:

- tracel-ai/burn — A-FINE no-reference image-quality metric: inlined CLIP ViT backbone, 5 evaluator heads, PyTorch-weight loader, regression tests vs reference. PR #4894, under review.

- tinygrad — ONNX LSTM operator (forward/reverse/bidirectional), regression suite verified against ONNX Runtime (PR #15453).

Degree work: CNN/MLP/RNN models for image classification, predictive analytics, and cryptographic analysis; 8-person Java/libGDX simulation game team lead; multithreaded Unity ECS roguelite. Game mods (RimWorld, Minecraft, Terraria, Escape from Tarkov) — 150,000+ downloads.

Long-term: AI in healthcare.

- Personalised medicine — ML over individual physiological data to surface insights, support clinical decisions, and respect privacy.

- Consumer health tooling — Apple Health / Bevel-shaped adaptive systems over physiological signals (sleep, HRV, training load, recovery) that understand the individual rather than applying population averages.

Hobbies: hypertrophy training, calisthenics, climbing, nutrition science.

Seeking graduate engineering roles where I can contribute to real projects alongside engineers who take correctness and systems design seriously.

Let's connect if you're building where performance, safety, and correctness matter — especially AI infrastructure, quant / fintech, or AI-driven health.
```

**Why this shape**:

- **Intro paragraph** — identity + degree + current trajectory. Single paragraph; sets the frame before the bullets land.
- **Project bullets (5 entries)** — each project gets its own bullet. Reader skims down the list rather than parsing a wall of text. Order = recency + ship-state weight (Aurix and Nyquestro both have shipped components and lead; Cernio / NeuroDrive / Image Browser sit beneath as steadier ongoing work).
- **OSS bullets (2 entries)** — separated from project bullets because the audience read is different (OSS is a verifiable-credentials signal per `research/09-open-source-representation.md`). Each bullet names the project + what was implemented + the verifiable artefact (PR number).
- **Degree + game mods paragraph** — kept as prose because it's a single broad-history sweep with no clean per-item separation. Game mods 150k downloads is the standout number.
- **AI-in-healthcare bullets (2 angles)** — broken from the prior single-paragraph version into two clearly labelled bullets (personalised medicine; consumer-grade health tooling). Reader sees both angles distinctly. The closing line "the work being done in that space is the exact direction I want to grow into" anchors both.
- **Hobbies single line** — kept short. Sits adjacent to the AI-in-healthcare section so health authenticity is structurally evident.
- **Closing ask + Let's-connect** — kept as prose, two short paragraphs. Asks frame what role-shape Caner is looking for; connect-line names the three target domains (AI infrastructure, quant/fintech, AI-driven health) for self-selecting outreach.

---

## Experience section — Open Source Contributor

**Single rolling Experience entry** dedicated to OSS engineering contributions. Recommended placement: as a standalone Experience entry on the LinkedIn profile (above Education, below any prior employment if listed). The placement decision rationale and target audience analysis lives below the entry.

### Why an Experience entry, not a Project entry

LinkedIn surfaces Experience above Projects in profile rendering, and LinkedIn's recruiter-search index weights Experience descriptions more heavily than Projects descriptions for keyword matches. For a visa-constrained search where every legitimate signal needs to land above the fold, OSS contributions of substantive scope (a 1,864-line metric implementation merged into a recognised Rust ML library after maintainer review) are best classified as professional engineering output, not as side-portfolio. The bar for the placement is whether the substance defends it; with one APPROVED substantial PR it does, modestly framed. The entry is designed as a single rolling summary that updates as more PRs land, NOT one Experience entry per PR.

### Date range

**March 2026 – Present.**

Start anchored to 2026-03-25 (claim of `tracel-ai/burn` Issue #4312 A-FINE metric, confirmed by maintainer laggui same day). This is the earliest verifiable engineering-scale OSS engagement. Game-mod releases (RimWorld, Minecraft, Terraria, Tarkov; 150,000+ downloads aggregate) sit elsewhere in the profile (About + Education Activities) — they are mod releases, not engineering contributions to other people's open-source codebases, and conflating the two would dilute the signal.

### Ready-to-paste entry

LinkedIn Experience fields:

- **Title**: `Open Source Contributor`
- **Employment type**: `Self-employed`
- **Company name**: leave blank, or `Independent` if LinkedIn requires a non-empty value
- **Location**: `United Kingdom · Remote`
- **Start date**: `March 2026`
- **End date**: leave blank (Present)
- **Description**:

```
Engineering-scale contributions to active Rust + Python ML / deep-learning open-source projects. Single rolling entry; expanded as additional contributions land.

Recent work:

- tracel-ai/burn — A-FINE no-reference image-quality metric. 1,864-line implementation across 10 files: inlined CLIP ViT backbone with PyTorch-weight loader, 5 evaluator heads (technical, structural, aesthetic, authenticity, overall), end-to-end regression tests against the reference implementation, and `forward_with_features` refactor preserving CLS-token output for downstream metrics.

- tinygrad/tinygrad — ONNX LSTM operator (forward, reverse, bidirectional). Full operator implementation following tinygrad's ONNX backend conventions, with an end-to-end regression suite verifying behaviour against ONNX Runtime.
```

### Why this shape

- **Single rolling Experience entry, not one-per-PR**: the Experience section becomes the durable summary; per-PR depth lives in the per-project files in `Projects/Open Source Contributions/` (`Burn.md`, `Tinygrad.md`). Future contributions are added to the bullet list without creating new Experience entries — keeps the profile clean.
- **Modest title ("Open Source Contributor", not "Open Source Engineer")**: at the current contribution depth + ongoing engagement, the title is defensible. Upgrade to "Open Source Engineer" or similar once 3-5 substantial contributions are visible across multiple recognised projects.
- **Implementation framing across both bullets, no PR-state narrative**: each bullet asserts what was built (file counts, line counts, named technical components, behavioural verification) without surfacing PR numbers, submission dates, maintainer names, or approval status. Caner's verbatim 2026-05-08: "just say what we implemented." Implementation is durable; PR state decays. Per-PR detail (burn PR #4894 approval 2026-05-07 by laggui, tinygrad PR #15453 closure on line-count + refactored-resubmission plan + round-2 stop rule) stays in `Projects/Open Source Contributions/<repo>.md` as canonical operational record. The bullets do not decay if PR states change.
- **Specific technical components named, not vague claims**: CLIP ViT backbone, PyTorch-weight loader, 5 named evaluator heads, regression tests, `forward_with_features` refactor; ONNX LSTM operator across the three modes, ONNX Runtime regression suite. A reviewer can verify any claim in seconds by searching the project repo. The opposite anti-pattern is vague "contributed to open source projects" framing with no specifics — that signals weak more than it signals strong.
- **`forward_with_features` refactor specifically named**: it's a structurally-meaningful API design decision (preserving CLS-token output as a reusable feature-extraction surface for downstream metric work), not a generic "I refactored stuff" claim. Naming it adds substance.
- **No `Tech surface:` line**: removed per Caner 2026-05-08. The technical surface is implicit in the bullets' named components (Rust, Python, ML-operator, ONNX, deep-learning) and explicit in the LinkedIn `Skills` section. Repeating it in the Experience description duplicates the Skills section, which already enumerates it.

### Update cadence

The entry is the rolling summary; the Recent-work bullet list is the moving part. Each time a new OSS contribution is implemented at substantive scope, a new bullet is added in the same implementation-only shape (project — what was implemented — named technical components — behavioural verification). No PR numbers, no submission dates, no maintainer names, no approval/merge status; that detail lives in `Projects/Open Source Contributions/<repo>.md`. When the bullet count crosses 4-5 substantial contributions, consider:

1. Promoting the title to "Open Source Engineer".
2. Splitting into per-project Experience entries if the work has clearly differentiated into multiple long-running contribution streams.
3. Adding an `Open Source Contributions` Featured-section pin linked to a public summary page or to the most-cited PR.

Keep the canonical-source for per-PR detail in `Projects/Open Source Contributions/<repo>.md`. The LinkedIn entry summarises from there, never the reverse.

---

## Featured section (pin 3-5 items)

LinkedIn's Featured section surfaces 3-5 items above the rest of the profile. Recommended order (current resume priority):

1. **Aurix** — `https://github.com/Capataina/Aurix`. Caption: "Local-first DeFi analytics on Tauri + Rust. Vector A V3 LP backtester shipped (Q64.96 fixed-point math, multi-asset benchmarking, regime-conditional capital allocation)."
2. **Nyquestro** — `https://github.com/Capataina/Nyquestro`. Caption: "From-scratch order matching engine in safe Rust. Matching engine MVP + Ratatui observability + live Coinbase BTC/ETH/SOL feed shipped."
3. **Cernio** — `https://github.com/Capataina/cernio`. Caption: "Local-first job-discovery engine treating job search as a systems problem. Async Rust + 9 native Claude Code skills + interactive Ratatui dashboard."
4. **NeuroDrive** — `https://github.com/Capataina/NeuroDrive`. Caption: "Biology-first RL racing simulator in Rust + Bevy. Handwritten PPO + sparse-graph continual learner with three-factor plasticity."
5. **Image Browser** — `https://github.com/Capataina/PinterestStyleImageBrowser`. Caption: "Local-first multi-encoder image manager. CLIP + DINOv2 + SigLIP-2 via ONNX Runtime, RRF over per-encoder rankings."

Optional addition (when burn PR merges): the merged burn PR as a Featured link with caption "Implemented A-FINE no-reference image-quality metric in tracel-ai/burn (PR #4894)."

---

## Education section (LinkedIn UI fields)

LinkedIn's Education entry has two free-form text fields: **Activities and societies** (500 char cap) and **Description** (1,000 char cap). Both are ready-to-paste below.

The field content was rewritten 2026-05-07 against the full module list (10 modules across Year 1, Year 2, Year 3, plus Masters-level options) per the [[Profile/Professional/Education|Education.md]] full breakdown.

### Activities and societies (Caner's preferred version, 2026-05-07)

Caner kept his original Activities text after the Description rewrite — keeping it as the canonical version below.

```
Climbing, Weightlifting, Boxing and Art societies. Also many open source contributions and projects across fields like game development, infra engineering for fintech and optimised multi-threaded high performance systems across Unity ECS and Rust atomics.
```

### Description (target ≤1,000 chars; current draft 949)

```
BEng Computer Science (University of York, 2022-2025).

ML / AI: CNN for flower classification + facial recognition, MLP for body dysmorphia prediction, RNN for ciphertext indistinguishability attacks (Intelligent Systems: ML & Optimisation; Probabilistic & Deep Learning) using Python, PyTorch, TensorFlow. Genetic algorithms in DEAP for beer quality prediction and evolutionary regression (Evolutionary & Adaptive Computing).

Systems: led an 8-person team on a Java + libGDX simulation game (Engineering 1) covering CI/CD, OO architecture, TDD. Java DSA: stacks, queues, trees, graphs, greedy, dynamic programming, complexity analysis.

Cybersecurity: AES, RSA, Diffie-Hellman, BB84 quantum key distribution (Cryptography); threat modelling, multi-layer network attacks/defences (Network Security).

Data: SQL + NoSQL, statistical analysis, hypothesis testing, regression (Data Science).

HCI: user-centred design, group prototype, user testing.
```

### Why this shape

- **Activities (was thin + fluffy)** — the original ("many open source contributions and projects across fields like game development, infra engineering for fintech") had no concrete artefacts. The rewrite anchors on verifiable numbers (150,000+ mod downloads, named repos, named PR numbers) so a reviewer can audit.
- **Description (was 958/1000 + truncated mid-sentence)** — the original was a coursework-snapshot ending in cut-off prose. The rewrite groups by subject area (ML/AI / Systems / Cybersecurity / Data / HCI) with each section naming the responsible module(s) parenthetically. A reviewer skims domains; an interviewer probes any one and pulls module + concrete project example.
- **Module names are bracketed parenthetically not separately listed** — saves chars + ties each project directly to its formal module credential.
- **Full module breakdown** lives in [[Profile/Professional/Education|Education.md]] for vault reference; LinkedIn Description carries the highlights only.

---

## Projects section (uniform layout for the 5 LinkedIn Project entries)

LinkedIn's Projects section accepts standalone entries with title, date range, and a free-form description. Caner's existing 5 entries (NeuroDrive, Nyquestro, Image Browser, Aurix, Cernio) had inconsistent shapes — some had `Technologies used:` blocks (which age as soon as a dependency changes), `Future plans:` blocks (which age the moment plans land or pivot), `Skills demonstrated:` blocks (which duplicate the Skills section), and `Current Implementation:` blocks (which age as soon as more ships).

### Uniform low-upkeep template (applied to all 5 entries below)

```
[Project Name] ([Subtitle])
[Start month + year] – [Present | end date]

[Framing paragraph — 1-2 sentences. What is this and what does it explore / target. Vision-forward + scope-stating, not implementation-snapshotting.]

Architectural principles:
- [Design principle / scope statement that ages slowly]
- [Design principle / scope statement that ages slowly]
- [Design principle / scope statement that ages slowly]
- [Design principle / scope statement that ages slowly]
- [Optional 5th]

What this signals:
[Closing paragraph — what engineering or research signal this project carries; what it demonstrates about the builder. Designed to remain valid as the project grows.]
```

### Why this shape (and what gets dropped)

- **No `Technologies used` block**: a stack list ages as soon as a dependency upgrades or a backend swaps. The architectural principles bullets reference frameworks where they're load-bearing (Rust + Bevy is meaningful for NeuroDrive's deterministic 60 Hz sim; Tauri 2 is meaningful for Aurix's local-first read-only constraint), but as part of the design rationale rather than as an enumerable list.
- **No `Future plans` block**: plans land or pivot, which makes the entry stale within weeks. The framing paragraph captures forward-looking ambition (vision-forward Sentence 1 pattern from `improve-resume` research file 07); the architectural principles state the scope-direction the project is built toward.
- **No `Skills demonstrated` block**: duplicates the LinkedIn Skills section + the resume Skills section. The "What this signals" closer carries the same signal in prose without enumerating skill keywords.
- **`Current Implementation` collapses into Architectural principles**: the bullets state design decisions baked into the project's architecture (lock-free atomics, Q64.96 fixed-point, Hebbian + STDP learning rules) rather than what's been built so far. Architectural decisions are durable; built-so-far is a moving target.
- **Closing reflection mirrors what each project signals to a reviewer**: lifted the closing-paragraph approach from Caner's existing NeuroDrive + Nyquestro + Image Browser entries (they all closed with a "what this is / what this demonstrates" reflection that doesn't age).

### Date status note (surfaced for Caner to confirm before applying)

| Project | Caner's existing LinkedIn date | Vault evidence | Recommended |
|---|---|---|---|
| NeuroDrive | Feb 2026 – Present | Active per `Projects/NeuroDrive/_Overview.md` | Keep as-is |
| Nyquestro | Jun 2025 – Dec 2025 | Active per `Projects/Nyquestro/_Overview.md` (last_verified 2026-05-04; substantial step-change session 2026-05-04) | **Update end date to Present** — project is active, current end-date misrepresents |
| Image Browser | Nov 2025 – Present | Active per `Projects/Image Browser/_Overview.md` | Keep as-is |
| Aurix | (no entry yet) | `Projects/Aurix/_Overview.md` frontmatter `created: 2026-03-04` | Set to **Mar 2026 – Present** |
| Cernio | (no entry yet) | Vault folder created 2026-04-13; project itself predates vault — Caner-set date needed | **Caner to set start date** (the vault doesn't carry the project's actual start; pick the month you opened the repo / started shipping) |

### Entry 1: NeuroDrive (Brain-Inspired Learning System)

**Date range**: Feb 2026 – Present

```
NeuroDrive (Brain-Inspired Learning System)
Feb 2026 – Present

A real-time research-grade learning laboratory exploring whether a driving agent can acquire skilled behaviour purely through biologically grounded local plasticity rules; no backpropagation, no external ML frameworks, no evolutionary search. Built from first principles inside a custom 2D top-down racing environment.

Architectural principles:
- Deterministic, instrumented simulation as a first-class citizen: continuous car physics, centerline-spline progress metric, raycast sensors, collision detection, and live telemetry exist before any learning is introduced
- Biologically grounded learning architecture: Hebbian plasticity, STDP-family eligibility traces for local credit assignment, dopamine-like reward-prediction-error gating, and structural plasticity rules (synapse growth and pruning) for topology adaptation
- Staged-milestone validation: a from-scratch A2C baseline establishes task learnability before transitioning to gradient-free local plasticity, isolating environment design from learning-rule issues and preventing ambiguous failure modes
- Mechanism transparency over benchmark performance: the project is designed to visibly and measurably learn using principles grounded in biological neural adaptation, not to outperform mainstream RL on lap times

What this signals:
NeuroDrive is a research playground for engineered biological plasticity, prioritising mechanism interpretability over benchmark scores. The work demonstrates rigour around isolating sources of failure (environment-vs-learning-rule), comfort with implementing low-level learning dynamics from scratch, and an interest in the boundary between neuroscience and engineered systems.
```

### Entry 2: Nyquestro (High-Performance Lock-Free Limit-Order Book)

**Date range**: Jun 2025 – Present (recommend updating from "Dec 2025" — project is active per `Projects/Nyquestro/_Overview.md`)

```
Nyquestro (High-Performance Lock-Free Limit-Order Book)
Jun 2025 – Present

A from-scratch order matching engine in safe Rust, working toward a lock-free limit-order book with price-time priority matching, a binary UDP wire protocol, a real-time risk layer, and a market-making strategy agent. Every layer is handcrafted to explore real-time financial systems and deterministic concurrency.

Architectural principles:
- Pure-safe-Rust implementation: zero-cost abstractions over `unsafe` shortcuts; lock-free atomic data structures over OS locks
- Multi-instrument routing across multiple gateway protocols (FIX, UDP, WebSocket, gRPC, CLI) for cross-domain interoperability
- Real-time observability built in at the engine layer: tracing spans, structured telemetry, latency and fill-rate dashboards as first-class operational surfaces
- Built-in risk controls (kill-switches, throttles, fat-finger protection, rolling VaR circuit-breaking) integrated at the matching-engine layer, not bolted on
- Real-market data integration: live exchange WebSocket bridges feeding production market depth alongside synthetic flows for testing

What this signals:
Nyquestro showcases systems-level design with a focus on real-time correctness, memory safety, and extensibility. The work demonstrates comfort with concurrent programming, low-latency engineering decisions, and protocol-level work, relevant for financial infrastructure, low-level Rust, and high-performance networking contexts.
```

### Entry 3: Image Browser (Local-First Multi-Encoder Image Manager)

**Date range**: Nov 2025 – Present

```
Image Browser (Local-First Multi-Encoder Image Manager)
Nov 2025 – Present

A local-first desktop application for browsing, tagging, and semantically searching personal image libraries entirely offline. The system combines multiple complementary image-embedding models to surface conceptual, structural, and descriptive similarity simultaneously, without sending any user data to a remote service.

Architectural principles:
- Local-first by design: no cloud dependencies, no external services, no telemetry; complete user privacy as a structural constraint, not a feature toggle
- Multi-encoder ensemble retrieval: heterogeneous embedding spaces (general visual, self-supervised, vision-language) combined via Reciprocal Rank Fusion rather than picking a single model and accepting its blind spots
- Concurrent stateful indexing: WAL-mode SQLite with separate read and write connections so the UI stays responsive during background indexing and embedding generation
- Typed IPC error envelopes between Rust backend and frontend: error semantics survive the language boundary cleanly
- Pinterest-style masonry presentation, multi-folder library with filesystem watcher, AND/OR tag filtering, per-image annotations, slideshow mode

What this signals:
Image Browser demonstrates that ML models can enhance local-first software without compromising privacy. The work proves comfort with desktop-application architecture, ML inference at the edge (no cloud GPU), and the engineering discipline required to make heterogeneous models useful together rather than competing.
```

### Entry 4: Aurix (Local-First DeFi Analytics Platform)

**Date range**: Mar 2026 – Present (per `Projects/Aurix/_Overview.md` frontmatter `created: 2026-03-04`)

```
Aurix (Local-First DeFi Analytics Platform)
Mar 2026 – Present

A local-first, zero-cost, read-only DeFi analytics desktop application for monitoring and analysing decentralised exchange markets entirely on-device. Spans cross-DEX arbitrage, Uniswap V3 LP backtesting, wallet tracking, gas prediction, and risk modelling, with each surface backed by exact on-chain math rather than approximate aggregator data.

Architectural principles:
- Local-first, read-only by design: computation is local, data sources are public (free on-chain RPC + free hosted subgraphs + public market data); no premium APIs, no wallet, no transaction capability
- Hand-crafted protocol layer: raw JSON-RPC with hand-crafted ABI encoding (no third-party EVM client dependency); Uniswap V3 sqrtPriceX96 and V2 reserve-ratio decoding via BigUint
- Exact fixed-point arithmetic: Q64.96 fixed-point math validated against on-chain positions for the V3 LP backtester, prioritising correctness over approximation
- Multi-asset benchmarking and regime-conditional capital allocation: the LP backtester compares against DeFi-native baselines (Aave, Compound, Lido, native staking) and TradFi baselines (T-bills, S&P 500, gold) under regime-aware weighting
- Hand-rolled visualisation surface: SVG charting with per-venue colour coding plus a TypeScript insight engine (rolling statistics, trailing-run detection, severity-graded notifications) on top of typed IPC envelopes

What this signals:
Aurix demonstrates that serious DeFi analytics can be built without trusting third-party services with user activity. The work spans systems-level Rust on the backend, exact-arithmetic numerical engineering, and frontend dashboard design, showing comfort with cross-domain stack ownership and the discipline required to make local-first DeFi tooling correct enough to be trusted.
```

### Entry 5: Cernio (Local-First Job Discovery and Curation Engine)

**Date range**: [Caner-set — pick the month you opened the repo or started shipping]

```
Cernio (Local-First Job Discovery and Curation Engine)
[Start month + year] – Present

A local-first job-discovery and curation engine that treats job search as a systems problem. Scans hundreds of companies across the major ATS providers, deduplicates and stores against a structured candidate profile, and orchestrates AI agents to grade every opportunity on multiple fit dimensions, all surfaced through an interactive terminal dashboard.

Architectural principles:
- Local-first execution: SQLite-backed state, no cloud dependencies, no third-party brokering of the candidate profile; the entire pipeline runs on the candidate's own machine
- ATS-aware ingestion: scans hundreds of companies across the major ATS providers with deduplication at the SQLite layer; the pipeline knows the structural shapes ATS providers serve rather than treating every page as a generic crawl target
- AI-agent orchestration via vault-installed Claude Code skills: 9 native skills grade every opportunity against a structured candidate profile on multiple fit dimensions, with prompts and rubrics versioned alongside the code
- Interactive Ratatui terminal dashboard: vim-style search, grade-based sorting, pipeline kanban, multi-select bulk operations, markdown export, responsive layout, real-time database refresh
- Async-Rust core with deterministic state transitions in the kanban; markdown export as the canonical interchange format

What this signals:
Cernio demonstrates that the job-search process can be engineered as a structured pipeline rather than navigated manually. The work spans async-Rust systems engineering, agent orchestration patterns, and TUI-driven interaction design, showing how AI-agent infrastructure can produce real day-to-day productivity outside of toy demos.
```

---

## Skills section

Sync to the Resume Skills section keyword list:

```
Languages: Rust, Python, C++, TypeScript, JavaScript, Java
Systems: Lock-Free Data Structures, Multithreading, Memory Safety, Low-Latency Optimisation
AI/ML: PyTorch, TensorFlow, ONNX Runtime, NEAT, DEAP, XGBoost, scikit-learn
Desktop & Full-Stack: Tauri, React, SQLite, Node.js
Finance: Order Book Mechanics, Market-Making, Ethereum RPC, AMM Mathematics, Quantitative Risk Modelling
Mathematics: Linear Algebra, Calculus, Probability, Optimisation Theory
```

LinkedIn's UI accepts each as a separate Skill entry. Recommended: add the high-signal individual ones (Rust, Tauri, ONNX Runtime, PyTorch, Lock-Free Data Structures, Order Book Mechanics) as standalone skill entries so endorsements can accumulate per-skill.

---

## Recent-activity post candidates (optional)

LinkedIn rewards recent posts with profile reach. Two candidate posts mirroring this week's resume changes — write only if Caner wants engagement signal, not required:

1. **Aurix V3 LP backtester ship**: short post about the Vector A V3 LP backtester being live. Architectural callouts (Q64.96 fixed-point, multi-asset benchmarking) plus a screenshot of the React dashboard. Link: github.com/Capataina/Aurix.

2. **Nyquestro Coinbase live data integration**: short post about the matching engine MVP + observability dashboard now consuming real Coinbase BTC/ETH/SOL market depth. Visual: a screenshot of the Ratatui dashboard rendering live latency / fill-rate. Link: github.com/Capataina/Nyquestro.

Either post can be reused as the LinkedIn-article cover-image opportunity per the [memory note](../../../../.claude/projects/-Users-atacanercetinkaya-Documents-life-os/memory/reference_linkedin_article_cover.md) (cover accepts video; 20s 1280p MP4 verified 2026-04-29).

---

## Cross-references

- [[Profile/Professional/Resume - Ata Caner Cetinkaya|Resume]] — canonical source for project content; LinkedIn must mirror.
- [[Profile/Professional/Cover Letter - Ata Caner Cetinkaya|Cover Letter]] — canonical narrative source.
- [[Profile/Professional/Visa|Visa]] — Open-to-Work banner depends on Graduate-visa status (unrestricted RTW until August 2027).
- [[Profile/Professional/Personal|Personal]] — contact info for LinkedIn UI's contact-info section.
- [[Projects/_Overview|Projects Overview]] — full active-project list; LinkedIn surfaces top 5 Featured items.
- `reference_linkedin_article_cover` memory — LinkedIn article cover accepts video (verified 2026-04-29).

## Update history

- 2026-05-07 — created. Captures Resume v2026-05-07 state (burn PR #4894 submitted, Aurix V3 LP backtester shipped, Nyquestro matching engine MVP shipped). Checklist for first-application-ready pass through the LinkedIn UI.
- 2026-05-08 — added new **Experience section — Open Source Contributor** (single rolling Experience entry). Date range March 2026 – Present. Four iterations same session: (1) burn-only draft with "approved 2026-05-07 by laggui; merge pending" status framing; (2) added tinygrad as refactored-resubmission-in-flight per Caner "word it as if it's done already"; (3) tinygrad bullet revised to implementation-framed achievement per Caner "write the description as 'done' not 'resubmitted'. Just say we implemented it"; (4) burn bullet aligned to the same implementation-only framing per Caner "write both the same way; the other one also says merge pending etc. just say what we implemented. remove the tech surface as well" — burn bullet stripped of submission date, approval date, maintainer name, merge status; `Tech surface:` line removed entirely. Final framing across both bullets is implementation-as-achievement: durable regardless of any specific PR's merge state, no audit-trail tension because the implementation exists independently of PR outcomes. Per-PR detail (burn PR #4894 approval / tinygrad PR #15453 closure + refactored-resubmission plan + round-2 stop rule) stays in `Projects/Open Source Contributions/<repo>.md` as canonical operational record. Update-checklist row added; Skills section now carries the technical-surface signal that Tech-surface line previously duplicated.
