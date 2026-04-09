# Projects

## Project 1

- **Name**: Image Browser
- **URL**: https://github.com/Capataina/PinterestStyleImageBrowser
- **Type**: Personal
- **Tier**: Flagship
- **Tech stack**: Rust, Tauri 2, React 19, TypeScript, SQLite (rusqlite), ONNX Runtime (ort), CLIP ViT-B/32, ndarray
- **Status**: Completed
- **Summary**: Local-first desktop application for browsing and searching large image collections, with Pinterest-style masonry layout, manual tagging, CLIP-powered visual similarity search, and natural language semantic search, all running entirely on the user's machine with no cloud, no accounts, and no external services.
- **Your role**: Solo. Designed and built every layer end to end.
- **Technical highlights**: Tauri 2 desktop application with a React 19 frontend and a Rust backend. Every image is preprocessed (224×224 resize, ImageNet normalisation, CHW channel layout) and encoded through a CLIP ViT-B/32 model via ONNX Runtime, producing 512-dimensional embeddings stored as BLOBs in SQLite. Batch encoding (32 images per batch) runs on startup for missing embeddings, with CUDA acceleration and automatic CPU fallback. A pure-Rust WordPiece tokenizer reads a HuggingFace tokenizer.json file to tokenize text queries, which are then encoded through a multilingual CLIP text model (clip-ViT-B-32-multilingual-v1) into the same 512-dimensional vector space, enabling cross-modal semantic search across 50+ languages. The text encoder is lazy-loaded on first query to avoid startup cost. A three-mode cosine similarity engine supports standard mode (top-20% diversity sampling), sorted mode (strict ranking for semantic text search), and tiered mode (Pinterest-style progressive sampling from 7 similarity tiers). The frontend renders a masonry grid with manual tagging (inline creation via # syntax in the search bar), visual similarity browsing, and semantic text search. Thumbnail generation preserves aspect ratios to 400×400 max with on-disk caching. Database migration handles schema evolution for existing databases. All ML inference runs locally — no network calls, no API keys, no cloud services. Original images are never modified; thumbnails and embeddings are derived artefacts stored locally.

## Project 2

- **Name**: Aurix
- **URL**: https://github.com/Capataina/Aurix
- **Type**: Personal
- **Tier**: Notable
- **Tech stack**: Rust, Tauri 2, React 19, TypeScript, Tokio, Reqwest, num-bigint
- **Status**: In Progress (early stage — arbitrage scanner partially complete, Tabs 2-5 not started)
- **Summary**: Local-first DeFi analytics platform for Ethereum built with Tauri 2. Currently implements a cross-DEX arbitrage scanner that fetches WETH/USDC prices from four venues using raw JSON-RPC with hand-crafted ABI encoding, displays them in a React dashboard with hand-rolled SVG charting, and computes gas-adjusted profit spreads with a derived insight engine. LP backtesting, wallet tracking, gas prediction, and risk modelling are planned but not yet implemented.
- **Your role**: Solo. Designs and builds the full stack across systems, financial, and web3 layers.
- **Technical highlights**: Interacts with Ethereum at the byte level rather than through a high-level SDK: ABI calldata is constructed by hand (function selectors, address padding, reserve decoding) using hex encoding, with no ethers-rs or alloy dependency. Uniswap V3 price fetching decodes sqrtPriceX96 from slot0() calls using num-bigint arbitrary-precision arithmetic, correctly handling the decimal shift between USDC (6 decimals) and WETH (18 decimals). Uniswap V2 and SushiSwap pricing resolves pool addresses via factory contracts, reads token ordering, and derives spot prices from raw reserve ratios with correct decimal normalisation. All four venue price fetches plus gas price run concurrently via tokio::join!. The React frontend renders a multi-series time-series chart entirely in hand-rolled SVG (~280 lines) with four switchable analytical modes (raw prices, deviation %, spread, gas-adjusted), dynamic domain scaling, grid lines, per-venue colour coding, and event markers. A ~350-line TypeScript insight engine computes rolling statistics (median, spread baselines, trailing run detection, per-venue deviation) and generates severity-graded insight cards and market event notifications from in-memory session history. Currently hard-coded to WETH/USDC on Ethereum mainnet. No persistence layer — all history is session-only in React state. Read-only by construction: never submits transactions.

## Project 3

- **Name**: NeuroDrive
- **URL**: https://github.com/Capataina/NeuroDrive
- **Type**: Personal / research-style
- **Tier**: Flagship
- **Tech stack**: Rust, Bevy 0.18, Serde
- **Status**: In Progress — deterministic environment and PPO baseline complete, biological plasticity transition pending
- **Summary**: Brain-inspired continual learning system built around a custom 2D top-down racing environment in Rust and Bevy. The long-term goal is replacing gradient-based learning with biological plasticity rules (Hebbian, STDP, dopamine-modulated updates). Currently at baseline validation — proving the environment and observation contract are learnable with a handwritten PPO before transitioning to biological mechanisms.
- **Your role**: Solo. Designs the simulation environment, the RL training infrastructure, the analytics pipeline, and every learning rule from first principles.
- **Technical highlights**: Eight distinct Bevy subsystems (~393KB of Rust across 67 source files) implementing a complete RL training environment from scratch with no ML framework dependencies. The environment is a deterministic 60 Hz 2D racing simulation with a multi-car vectorised trainer (8 cars, configurable, shared rollout buffer, random centreline spawn positions). The observation space is 43-dimensional: raycast distance sensors, split forward/lateral velocity, speed delta, 12-point lookahead with heading deltas and curvatures spanning 30–650 units, and previous actions. The brain is a handwritten PPO implementation with clipped surrogate objective, multi-epoch updates amortised across ticks to stay within frame budget, and an asymmetric architecture (actor 2×64, critic 2×128). Reward engineering is constrained by an entertainment-first design principle — the simulation must be entertaining to watch, which rules out crash penalties, survival bonuses, and anything that incentivises safe play; velocity-projection reward along the centreline tangent makes cars go fast while the learning challenge is surviving corners. A comprehensive analytics pipeline captures 16 tick-level fields and 25 episode-level aggregates including crash classification, and exports 10-section diagnostic Markdown reports with auto-generated takeaways. Feature-gated profiling infrastructure measures per-frame timing with auto-export. Debug tooling includes a live HUD, world-space overlays for sensors and geometry, and a live leaderboard ranking cars by cumulative progress. Performance engineering: discovered and fixed a 43x performance regression from `Vec<Vec<f32>>` weight storage by switching to flat contiguous row-major storage; amortised PPO training across ticks to maintain smooth 60 Hz rendering with 8 concurrent cars. The project is deliberately staged: deterministic sandbox first, then PPO baseline validation (current stage), then biological plasticity replaces gradients, then ablations, then spiking dynamics with STDP traces, then structural growth and pruning. Excludes PyTorch, TensorFlow, JAX, and any external ML dependency — the entire learning machinery is hand-built in safe Rust.

## Project 4

- **Name**: Nyquestro
- **URL**: https://github.com/Capataina/Nyquestro
- **Type**: Personal
- **Tier**: Notable
- **Tech stack**: Rust, chrono, thiserror
- **Status**: In Progress (early stage — type foundation complete, matching engine not yet implemented)
- **Summary**: An order matching engine project in safe Rust that has implemented its foundational type system — strongly-typed domain primitives, order state transitions, zero-allocation event frames, and a structured error taxonomy — with thorough integration tests. The matching engine core (order book, price-time priority matching, fill emission) is the next implementation milestone.
- **Your role**: Solo. Designs the type system, event model, error taxonomy, and matching engine architecture.
- **Technical highlights**: Domain primitives are strongly typed newtypes with validation: `Px` uses fixed-point integer representation in cents to avoid floating-point arithmetic on the hot path, `Ts` wraps nanosecond-precision timestamps with conversion and comparison methods, and `OrderID` enforces non-zero validity at construction. The `Order` struct models per-order state transitions through Open, PartiallyFilled, and FullyFilled states, with a fill method that applies saturating quantity subtraction and automatic status derivation. Three event frame types (FillEvent, QuoteEvent, OrderEvent) are implemented as Copy structs with no heap allocation, designed to flow through the future matching pipeline without cloning overhead. OrderEvent models the full lifecycle (New, Cancelled, Rejected) with typed rejection reasons. A structured error taxonomy with NyquestroError (11 variants) and ErrorSeverity classification distinguishes recoverable input validation failures from fatal state corruption. A PriceLevel container groups orders at a single price point. Integration tests (~50 tests across two files) exercise primitives, events, copy semantics, boundary conditions, and lifecycle scenarios. The matching engine module exists as a directory structure but the order book implementation is not yet written — the architecture documentation tracks this as the next milestone. Constrained to safe Rust with no unsafe blocks.

## Project 5

- **Name**: Cernio
- **URL**: https://github.com/Capataina/Cernio
- **Type**: Personal
- **Tier**: Flagship
- **Tech stack**: Rust, Tokio, Reqwest, Serde, Ratatui, SQLite (rusqlite), Crossterm, Claude Code skills
- **Status**: In Progress
- **Summary**: A local-first job discovery, evaluation, and career preparation engine that builds a structured profile of who you are, discovers companies through creative AI-driven web search, scans their job boards in seconds, evaluates every listing against your profile with full reasoning, and presents everything in a real-time interactive terminal dashboard. 408 companies tracked (287 resolved across 7 ATS providers, 121 bespoke), 937 jobs evaluated, 9 AI skills with comprehensive reference files.
- **Your role**: Solo. Designs and builds the full system: Rust pipeline scripts, SQLite data layer, Ratatui TUI, Claude Code skill architecture, and the structured profile system.
- **Technical highlights**: Architected as three layers communicating through a shared SQLite data store in WAL mode. The Rust pipeline layer consists of six parameterised CLI scripts: `resolve` probes slug candidates against 6 ATS providers (Greenhouse, Lever, Ashby, Workable, SmartRecruiters, Workday) in parallel using Tokio tasks with per-provider rate limiting; `search` fetches all open jobs from resolved companies and applies a configurable three-stage filter chain (location patterns per ATS provider, title exclusion keywords, title inclusion keywords) before deduplicating against existing database entries by URL; `clean` removes low-grade jobs and archives marginal companies while preserving user decisions and high-value matches; `check` produces a structured integrity report covering ATS slug re-verification, stale entry detection, completeness analysis, dead URL probing, duplicate detection, and profile-change-driven staleness; `import` bulk-ingests companies from external discovery sources with auto-clearing of the landing zone after successful import; and `format` converts raw HTML and entity-encoded job descriptions to clean plaintext, running idempotently on TUI startup. The TUI is a four-view interactive terminal dashboard built with Ratatui 0.29 and Crossterm: a dynamic stats dashboard with session summaries and scrollable top-tier job lists, a company browser with grade distribution bars and full job listings per company, a job browser with complete HTML-cleaned descriptions and profile-grounded fit assessments in the detail pane, and a pipeline kanban view with three columns (Watching/Applied/Interview) showing grade-coloured cards. Supports vim-style search with instant filtering across title, company, and location; four sort modes cycling through grade, company, date, and location; inline grade override via popup picker; multi-select with Ctrl+click and Shift+click for bulk operations; markdown export of the current view; and an archive visibility toggle. Mouse support handles viewport scrolling (3 lines per tick targeting whichever pane the cursor is over), click-to-select on table rows, and click-on-tab for view switching. A responsive layout system with a `distribute()` function dynamically sizes blocks based on content, switching between side-by-side master/detail (100+ columns), stacked layout (<100), and compact list-only mode (<80). The event loop drains up to 20 events per frame to prevent input buffering lag from trackpad momentum scrolling. Widget refactoring extracts shared rendering into reusable modules: proportional grade bars, HTML description cleanup with entity decoding and block-element-to-newline conversion, relative date formatting, and ephemeral toast notifications. The AI layer consists of 9 Claude Code skills, each with a mandatory-read protocol requiring the agent to read the skill's SKILL.md, all reference files in the skill's `references/` directory, and every file in `profile/` before execution. Skills are structured around comprehensive reference files: the grading skills include detailed rubrics with evaluation dimensions, worked examples across different company and role types, boundary cases showing where grade lines fall, evidence standards distinguishing verifiable signals from absent information, and profile-context guides explaining how to synthesise 15 profile files into evaluation criteria. The discovery skill dispatches parallel agents with the full text of a search-strategies reference embedded in each agent's prompt, teaching creative discovery through VC portfolios, open source contributor affiliations, conference sponsors, engineering blogs, and indirect ecosystem traces. The check-integrity skill includes remediation procedures for every issue type it can detect and quality-standards examples showing acceptable vs unacceptable grade reasoning. All evaluation output — company grade reasoning, job fit assessments, relevance statements — must reference specific profile elements by name (projects, technologies, visa timeline, career targets), with generic reasoning explicitly banned. A structured profile system across 15 files (education, experience, projects, skills, preferences, visa, portfolio gaps, and more) serves as the source of truth for all evaluation; the profile-scrape skill auto-updates entries from GitHub repositories by reading context folders, dependency manifests, and source code, with proficiency levels grounded in what the code actually demonstrates rather than self-reported claims. Portfolio gap tracking runs after every grading batch, identifying technologies and domain knowledge the market consistently asks for that the profile lacks, feeding a career coaching loop with specific closure recommendations. The database schema uses 5 tables across 6 migrations with 18 tests, supporting company lifecycle management (potential → resolved/bespoke → archived), multi-portal tracking via a company_portals junction table, job evaluation lifecycle (pending → evaluating → graded), and user decision history. All filter thresholds, search keywords, location patterns, cleanup rules, and grade boundaries are configurable through a self-documenting TOML preferences file. Local-first by construction: no cloud dependencies, no hosted database, no telemetry, no accounts, no subscriptions.

## Project 6

- **Name**: Vynapse
- **URL**: https://github.com/Capataina/Vynapse
- **Type**: Personal / research-style
- **Tier**: Notable
- **Tech stack**: Rust
- **Status**: In Progress (low ROI — compelling concept but too large to develop proportionately)
- **Summary**: A Rust-native deep learning and neuroevolution engine built as a hybrid learning runtime, unifying gradient-based learning and evolutionary optimisation within a single execution and graph infrastructure.
- **Your role**: Solo.
- **Technical highlights**: Bridges the optimisation paradigms of PyTorch, TensorFlow, DEAP, and NEAT inside one modular system, with every training mode operating on the same core tensor and graph runtime so networks can be evolved, fine-tuned, and deployed interchangeably. Supports SGD-style training with reverse-mode autodiff, NEAT-style topology-evolving neural networks with speciation and compatibility distance, DEAP-style population-based weight evolution, and static graph execution with forward/backward scheduling. Hybrid optimisation modes can combine evolution and gradient descent in a single training loop (Lamarckian or Baldwinian). Architected around trait-based modularity: genomes, fitness functions, selection strategies, genetic operators, loss functions, and activations are all implemented as interchangeable trait implementations. Built entirely in safe parallel Rust with no Python bindings and no unsafe blocks.

## Project 7

- **Name**: Consilium
- **URL**: https://github.com/Capataina/Consilium
- **Type**: Personal
- **Tier**: Minor
- **Tech stack**: Python, LangChain, MCP
- **Status**: Abandoned
- **Summary**: A multi-LLM debate and knowledge synthesis platform that orchestrates structured multi-round debates between heterogeneous LLMs (Claude, GPT, Gemini, and local Ollama models) and synthesises their perspectives into a single structured knowledge artefact.
- **Your role**: Solo.
- **Technical highlights**: Built on the observation that every major AI model has different training data, different objectives, and different blind spots, and that comparing answers across providers is more valuable than relying on any single one if there is a structured way to compare, contrast, and synthesise them. The orchestrator runs structured debate rounds in which each agent responds independently without seeing others' raw output, a dedicated summariser agent compresses each round's output into a running summary to manage context and enable unlimited debate length, and convergence and divergence detection identifies which positions models agree on and which they persistently disagree on across rounds. Built on LangChain for provider-agnostic LLM orchestration so commercial and local models participate on equal footing, and on MCP for shared tool access so all agents regardless of provider can call the same tools (web search, calculator, citation fetcher) through a unified interface. Every debate produces a final synthesis document capturing arguments made, consensus reached, unresolved disagreements, and metadata, alongside the full transcript and a convergence report.

## Project 8

- **Name**: Zyphos
- **URL**: https://github.com/Capataina/Zyphos
- **Type**: Personal
- **Tier**: Minor
- **Tech stack**: Rust
- **Status**: Abandoned
- **Summary**: A network protocol laboratory built through hands-on HTTP server implementation, designed to internalise sockets, protocols, and network programming while applying performance patterns from compilers, trading systems, and distributed infrastructure where they naturally fit.
- **Your role**: Solo.
- **Technical highlights**: Bottom-up: raw sockets and system calls first, then TCP state machines and connection handling, then HTTP/1.0 parsing and routing, then thread pools, memory pools, zero-copy buffers, HTTP/1.1 connection pooling, and epoll/kqueue event loops, then HTTP/2 and beyond. Cross-domain technique application is deliberate: SIMD techniques from compiler engineering applied to protocol parsing, lock-free structures from HFT applied to connection handling, arena allocation and zero-copy buffers from systems work, NUMA awareness and CPU pinning from low-latency infrastructure, and time and ordering primitives (NTP, PTP, vector clocks) from distributed systems. Measurement obsessed throughout: syscall counts, cache misses, packet rates, and latency percentiles tracked at every milestone. Security worked through by implementation: deliberately triggering slowloris, SYN floods, and request smuggling, then fixing them.

## Project 9

- **Name**: Chrona
- **URL**: https://github.com/Capataina/Chrona
- **Type**: Personal
- **Tier**: Minor
- **Tech stack**: C++
- **Status**: Abandoned
- **Summary**: A Git-inspired, local-first version control core built in modern C++ to internalise how real version control works under the hood: content-addressed storage, immutable snapshots, commit graphs, staging and index semantics, and diffs.
- **Your role**: Solo.
- **Technical highlights**: Built around the deliberate distinction that Git is the version control system (the local engine and data model) while GitHub and GitLab are hosting platforms layered on top, and that most developers learn the commands without ever understanding the engine. Targets the inner engine specifically: a content-addressed object database providing integrity and deduplication, snapshot-based versioning with trees, blobs, and commits as immutable objects, a commit DAG representing history as a graph rather than a linear list, an index/staging model separating working tree from staged from committed state, and diffs as a derived view rather than the primary storage truth. Networking, history rewriting, packfiles, and enterprise extras are explicitly excluded to keep the focus on the core data model. Prioritises clarity, correctness, deterministic encoding, stable hashing, and testable invariants before performance and ergonomics.

## Project 10

- **Name**: Xyntra
- **URL**: https://github.com/Capataina/Xyntra
- **Type**: Personal / research-style
- **Tier**: Notable
- **Tech stack**: Rust
- **Status**: Abandoned
- **Summary**: An automatic kernel-fusion compiler pass written entirely in safe Rust that ingests ONNX and TorchScript graphs, pattern-matches common op-chains, and emits a single fused GPU kernel through `wgpu` (cross-platform WGSL) or optional CUDA PTX.
- **Your role**: Solo.
- **Technical highlights**: Explores graph rewriting, GPU occupancy modelling, and autotuned code generation while keeping the entire pipeline 100% unsafe-free. Built around a type-safe IR with explicit error classification, a modular crate layout (`xyntra-core`, `xyntra-cli`, `xyntra-ir`), an `egg`-based e-graph integration for rewrite rules and saturation, a declarative fusion DSL for expressing patterns like `matmul → gelu → dropout`, scheduling heuristics with a cost model for fusion candidates, fusion legality checks for shape, dtype, and broadcast guards, WGSL and CUDA PTX backends with shared-memory tiling and vectorisation, and an autotuning harness using Bayesian optimisation over tile sizes. Correctness is enforced through golden unit tests comparing fused versus unfused outputs, gradient checks, edge-case libraries for broadcast, dynamic shapes, and odd strides, and configurable numerical tolerance thresholds. Observability includes structured tracing, kernel timeline JSON dumps, GPU occupancy analysis covering register and SM utilisation, HDR latency histograms, and roofline modelling.

## Project 11

- **Name**: Tectra
- **URL**: https://github.com/Capataina/Tectra
- **Type**: Personal
- **Tier**: Notable
- **Tech stack**: C++, FlatBuffers, Protobuf, Prometheus
- **Status**: Abandoned
- **Summary**: A modern C++ trading-infrastructure stack combining a low-latency market-data feed handler, pre-trade risk service, firm-wide kill switch, deterministic replay engine, strategy execution framework, backtesting engine, and signal research toolkit into one cohesive production-style system.
- **Your role**: Solo.
- **Technical highlights**: Focuses on the invisible infrastructure every serious trading firm relies on, plus the strategy execution layer that sits on top of it. The feed handler decodes ITCH/OUCH-like streams, manages sequencing and recovery, builds L2 books, and publishes a unified internal schema. The pre-trade risk service performs microsecond-level rule checks for price bands, size and notional limits, credit caps, and per-venue throttles, with hot-reloadable limits and full auditability. The kill-switch and circuit-breaker layer provides automatic and manual triggers that freeze or slow order flow, cancel open orders, or isolate venues with sub-millisecond propagation. The deterministic replay engine uses append-only checksummed journals with time-virtualised playback and golden-run diffing for post-incident analysis. The strategy execution framework is a plugin architecture with signal generation, order management, position tracking, and PnL calculation; the backtesting engine performs time-virtualised historical replay with simulated fills, slippage models, and transaction cost analysis; the signal library covers technical indicators, statistical arbitrage, mean reversion, momentum, and ML feature extraction. Built with C++20, lock-free shared-memory rings, FlatBuffers/Protobuf schemas for contract-first messaging, Prometheus metrics with per-strategy PnL, Sharpe ratios, and drawdown tracking, and tamper-evident logging with append-only journals carrying checksums and Merkle roots. Dual-plane architecture separates the binary fast path for low-latency data from the structured control plane for operators and metrics.

## Project 12

- **Name**: Neuronika
- **URL**: https://github.com/Capataina/Neuronika
- **Type**: Personal
- **Tier**: Minor
- **Tech stack**: Python
- **Status**: Completed
- **Summary**: AI-powered personal knowledge management tool that combines embedding-based retrieval with a semantic graph visualisation of the user's notes and concepts.
- **Your role**: Solo.
- **Technical highlights**: Built around the idea that personal notes are most useful when surfaced by meaning rather than by filename or folder. Combines vector embedding retrieval with a graph representation of the relationships between notes, enabling both semantic search and visual exploration of how ideas connect.

## Project 13

- **Name**: AsteroidsAI
- **URL**: https://github.com/Capataina/AsteroidsAI
- **Type**: Personal benchmarking project
- **Tier**: Notable
- **Tech stack**: Python, NEAT, DEAP, PyTorch
- **Status**: Completed
- **Summary**: A benchmarking platform that compares multiple optimisation paradigms (NEAT, genetic algorithms, evolution strategies, and a GNN-based reinforcement learning agent using SAC) on a single Asteroids-style environment under controlled conditions.
- **Your role**: Solo.
- **Technical highlights**: Designed to put evolutionary and gradient-based reinforcement learning approaches side by side under the same task, the same observation space, and the same reward structure, making the differences between paradigms empirically visible rather than theoretical. Implements NEAT for topology-evolving networks, classical genetic algorithms via DEAP for population-based weight search, evolution strategies as a gradient-free baseline, and a GNN policy trained with Soft Actor-Critic as the deep RL representative.

## Project 14

- **Name**: Credit Card Fraud Detection
- **URL**: <!-- TODO: confirm public link if any -->
- **Type**: Personal / academic-style
- **Tier**: Minor
- **Tech stack**: Python, scikit-learn, XGBoost
- **Status**: Completed
- **Summary**: A fraud detection model trained on a heavily class-imbalanced credit card transaction dataset, achieving 94% precision through gradient boosting and careful evaluation methodology.
- **Your role**: Solo.
- **Technical highlights**: Treats class imbalance as the central problem rather than an afterthought: the dataset contains a tiny fraction of fraudulent transactions, so naive accuracy is meaningless and precision/recall trade-offs dominate every modelling decision. Uses gradient boosting via XGBoost with careful threshold tuning and evaluation against precision, recall, and ROC-AUC rather than accuracy alone.

## Project 15

- **Name**: Personal Website
- **URL**: <!-- TODO: confirm if currently deployed -->
- **Type**: Personal
- **Tier**: Minor
- **Tech stack**: TypeScript, JavaScript
- **Status**: Abandoned
- **Summary**: A personal website with a custom particle physics simulation as the centrepiece interactive element.
- **Your role**: Solo.
- **Technical highlights**: Particle simulation written from scratch rather than using a physics library, integrating directly with the page as the primary visual identity of the site.

## Project 16

- **Name**: Game Modding Portfolio
- **URL**: <!-- Steam Workshop / Nexus / CurseForge profiles -->
- **Type**: Hobbyist software, hosted on public mod platforms
- **Tier**: Notable
- **Tech stack**: C#, XML, game-specific scripting and modding APIs across RimWorld, Minecraft, Terraria, and Escape from Tarkov
- **Status**: Completed (historical — no longer actively modding)
- **Summary**: A portfolio of 18+ released mods across four games totalling over 150,000 combined downloads.
- **Your role**: Solo creator on every mod. Design, implementation, debugging, end-user support, and updates.
- **Technical highlights**: The original outlet for the same drive that produces the current systems projects: reverse-engineering of game internals to extend behaviour, working within strict compatibility constraints across game versions and other mods, shipping software to a real user base, and handling bug reports and feature requests from end users. Considered formative engineering experience, with the same loop (read foreign code, find the extension points, build something coherent on top) that underpins more recent infrastructure work.