# Projects

## Project 1

- **Name**: Image Browser
- **URL**: https://github.com/Capataina/PinterestStyleImageBrowser
- **Type**: Personal
- **Tech stack**: Rust, Tauri, React, TypeScript, SQLite, ONNX Runtime, CLIP
- **Status**: Completed
- **Summary**: Local-first desktop application for browsing and searching large image collections, with Pinterest-style masonry layout, manual tagging, CLIP-powered visual similarity search, and natural language semantic search, all running entirely on the user's machine with no cloud, no accounts, and no external services.
- **Your role**: Solo. Designed and built every layer end to end.
- **Technical highlights**: Treats the user's local image library as a first-class collection: thumbnailed, indexed, tagged, and searchable both by text labels and by semantic meaning. Architecture is a Tauri desktop application with a React frontend (masonry grid, search bar, tag UI, slideshow, similarity results), a Rust backend handling folder scanning, thumbnail generation, tag management, search logic, CLIP embedding generation, and a cosine similarity engine, an SQLite store for image metadata, tags, and embeddings, and an ONNX Runtime layer running CLIP image and text encoders locally. Every image is encoded into a 512-dimensional embedding stored in SQLite and loaded into memory at startup; cosine similarity over the full library powers both the visual similarity feature and the semantic search bar. Embedding generation, indexing, and inference all run locally with no GPU required and no Python in the pipeline. Designed around privacy by construction: original images are never modified or uploaded, and all derived data lives in a local SQLite database alongside on-disk thumbnail caches.

## Project 2

- **Name**: Aurix
- **URL**: https://github.com/Capataina/Aurix
- **Type**: Personal
- **Tech stack**: Rust, Tauri, React, TypeScript, SQLite
- **Status**: In Progress
- **Summary**: Local-first DeFi analytics platform for Ethereum, covering real-time cross-DEX arbitrage detection, Uniswap V3 LP backtesting, on-chain position tracking, gas pattern analysis and prediction, and quantitative portfolio risk modelling, running entirely on the user's machine against free public RPC endpoints.
- **Your role**: Solo. Designs and builds the full stack across systems, financial, and web3 layers.
- **Technical highlights**: Sits between hosted dashboards (which require wallet connections and send data to third parties) and raw blockchain explorers (which give no analytical layer). Five distinct analytical surfaces: an arbitrage scanner that fetches token prices concurrently across multiple DEXes and surfaces gas-adjusted profit opportunities; a Uniswap V3 LP analyser implementing tick mathematics, fixed-point arithmetic, and historical backtesting with fee income, impermanent loss, and net return tracking against a hold-only baseline; a wallet and position tracker that decodes active DeFi positions for any Ethereum address from public on-chain data without a private key or wallet connection; a gas price monitor with hour-of-day and day-of-week pattern analysis, heatmap visualisation, and timing recommendations from a model trained on local history; and a token correlation and risk dashboard computing rolling pairwise correlations, rolling annualised volatility, historical-simulation Value-at-Risk, and stress tests against historical drawdown periods. Architected as a Tauri desktop application with a React dashboard, a concurrent Rust backend coordinating data fetching, price comparison, financial calculation, and risk modelling, and local storage for opportunity logs, price history, position snapshots, and gas data. Read-only by construction: never submits transactions, only observes and analyses. Built explicitly to require no wallet, no ETH, no paid services, and no API keys.

## Project 3

- **Name**: NeuroDrive
- **URL**: https://github.com/Capataina/NeuroDrive
- **Type**: Personal / research-style
- **Tech stack**: Rust, Bevy 0.18, Serde
- **Status**: In Progress — deterministic environment and PPO baseline complete, biological plasticity transition pending
- **Summary**: Brain-inspired continual learning system built around a custom 2D top-down racing environment in Rust and Bevy. The long-term goal is replacing gradient-based learning with biological plasticity rules (Hebbian, STDP, dopamine-modulated updates). Currently at baseline validation — proving the environment and observation contract are learnable with a handwritten PPO before transitioning to biological mechanisms.
- **Your role**: Solo. Designs the simulation environment, the RL training infrastructure, the analytics pipeline, and every learning rule from first principles.
- **Technical highlights**: Eight distinct Bevy subsystems (~250KB of Rust) implementing a complete RL training environment from scratch with no ML framework dependencies. The environment is a deterministic 60 Hz 2D racing simulation with a multi-car vectorised trainer (8 cars, configurable, shared rollout buffer, random centreline spawn positions). The observation space is 43-dimensional: raycast distance sensors, split forward/lateral velocity, speed delta, 12-point lookahead with heading deltas and curvatures spanning 30–650 units, and previous actions. The brain is a handwritten PPO implementation with clipped surrogate objective, multi-epoch updates amortised across ticks to stay within frame budget, and an asymmetric architecture (actor 2×64, critic 2×128). Reward engineering is constrained by an entertainment-first design principle — the simulation must be entertaining to watch, which rules out crash penalties, survival bonuses, and anything that incentivises safe play; velocity-projection reward along the centreline tangent makes cars go fast while the learning challenge is surviving corners. A comprehensive analytics pipeline captures 16 tick-level fields and 25 episode-level aggregates including crash classification, and exports 10-section diagnostic Markdown reports with auto-generated takeaways. Feature-gated profiling infrastructure measures per-frame timing with auto-export. Debug tooling includes a live HUD, world-space overlays for sensors and geometry, and a live leaderboard ranking cars by cumulative progress. Performance engineering: discovered and fixed a 43x performance regression from `Vec<Vec<f32>>` weight storage by switching to flat contiguous row-major storage; amortised PPO training across ticks to maintain smooth 60 Hz rendering with 8 concurrent cars. The project is deliberately staged: deterministic sandbox first, then PPO baseline validation (current stage), then biological plasticity replaces gradients, then ablations, then spiking dynamics with STDP traces, then structural growth and pruning. Excludes PyTorch, TensorFlow, JAX, and any external ML dependency — the entire learning machinery is hand-built in safe Rust.

## Project 4

- **Name**: Nyquestro
- **URL**: https://github.com/Capataina/Nyquestro
- **Type**: Personal
- **Tech stack**: Rust
- **Status**: In Progress
- **Summary**: A from-scratch implementation in safe Rust of the most latency-sensitive piece of software in finance: a correct, lock-free order matching engine with price-time priority, a real-time risk layer, a binary UDP wire protocol, and a market-making strategy agent, instrumented end to end with a rigorous latency benchmarking harness.
- **Your role**: Solo. Designs the matching engine core, the risk guard, the binary protocol, the strategy agent, and the benchmarking harness.
- **Technical highlights**: Composed of independently testable layers connected through narrow, replaceable interfaces. The matching engine core maintains bid and ask sides as atomic price-level buckets, each an intrusive FIFO list managed without locks, supporting price-time priority matching, partial fills, and atomic cancellation. The hot path avoids heap allocation, lock contention, and cache misses by design: lock-free CAS operations replace mutexes (which are a tail-latency cliff under contention), and a slab allocator pre-allocates a fixed pool of order node slots with a lock-free free-list, eliminating allocator churn from the latency profile. The risk guard sits between the gateway and the matching engine, enforcing fat-finger protection, per-session position bounds with real-time inventory and unrealised PnL tracking, rolling Value-at-Risk circuit breaking, and per-session order rate limits, with an immutable append-only event journal for post-trade audit and replay. The order gateway speaks a custom binary UDP protocol using fixed-width little-endian frames with versioning, length prefixing, and checksums, mapping directly to in-memory representations via a single memcpy with no parsing or allocation, alongside a FIX TCP acceptor for compatibility. A market data publisher emits depth snapshots and incremental book updates. The strategy agent is a fully external participant connecting through the same protocol any client would use: it reconstructs the live book from the depth feed, computes spread, mid-price, and order flow imbalance signals, runs two-sided market-making logic with quote skew under directional pressure, tracks inventory and mark-to-market PnL atomically on every fill, applies inventory-aware quote pricing to nudge positions back toward flat, and detects adverse selection by monitoring fill rate and direction. The benchmarking harness produces HDR latency histograms across p50, p99, p99.9, and p99.99 per workload profile, with hardware performance counter integration surfacing L3 cache miss rate, branch misprediction rate, and instructions per cycle alongside latency. The whole core is constrained to safe Rust: where unsafe would normally be required for performance, carefully designed safe abstractions compile to equivalent machine code, and the compiler verifies the absence of data races at compile time.

## Project 5

- **Name**: Cernio
- **URL**: https://github.com/Capataina/Cernio
- **Type**: Personal
- **Tech stack**: Rust, Tokio, Reqwest, Serde, Ratatui, SQLite, Claude Code skills
- **Status**: In Progress
- **Summary**: Local-first, collaborative job discovery and curation engine built to solve the actual bottleneck in a serious technical job search: finding the right roles at the right companies, evaluated against a structured personal profile, sourced from a personally curated company universe, and reviewed together with an LLM agent in a real-time terminal dashboard.
- **Your role**: Solo. Designs the architecture, the Rust core, the data store contract, the TUI, and the Claude Code skill layer.
- **Technical highlights**: Frames the job search problem honestly: mass-market aggregators (LinkedIn, Indeed) optimise for volume and have filters too coarse to encode what matters, while curated trackers (TrueUp, Otta, Wellfound) impose someone else's editorial choices; both treat the CV as an afterthought and reduce matching to keyword search over extracted text. Cernio replaces this with a structured profile (separate files for education, experience, projects, skills, preferences, visa, languages, and more) as the source of truth, a curated universe of UK and remote-UK technology employers built from authoritative public list sources, and parameterised Rust scripts that scan hundreds of ATS boards in seconds for matching roles. Architected as three layers communicating through a shared data store: generic stateless Rust scripts handle combinatorial volume work (scanning 200 Greenhouse boards for 50 title patterns means 10,000 checks, which a script does in seconds and an LLM cannot do economically); Claude reads each job description, compares it against the structured profile across dozens of dimensions, and writes fit assessments with full reasoning; a Ratatui terminal dashboard watches the data store and shows the entire collaborative process unfolding in real time, with each row transitioning from pending to evaluating to strong fit or no fit. Claude Code skills handle the slow, fuzzy, infrequent work that requires reasoning: discovering new companies from public sources, resolving company names to ATS slugs when deterministic patterns fail, and enriching missing metadata. Sessions are conversational: the user and the agent decide together what to search for, the agent orchestrates scripts and writes evaluations, the TUI makes everything visible, and the user reviews, marks results as watching/applied/rejected, and exports clean markdown reports on confirmation. Explicitly never submits applications, sends emails, or contacts recruiters; every external action is performed by the user. Local-first, no cloud dependencies, no hosted database, no telemetry, no account, no subscription. Plain text and inspectable artefacts everywhere: markdown for company files, TOML for preferences, JSONL for job data, SQLite for hot-path queries.

## Project 6

- **Name**: Vynapse
- **URL**: https://github.com/Capataina/Vynapse
- **Type**: Personal / research-style
- **Tech stack**: Rust
- **Status**: In Progress
- **Summary**: A Rust-native deep learning and neuroevolution engine built as a hybrid learning runtime, unifying gradient-based learning and evolutionary optimisation within a single execution and graph infrastructure.
- **Your role**: Solo.
- **Technical highlights**: Bridges the optimisation paradigms of PyTorch, TensorFlow, DEAP, and NEAT inside one modular system, with every training mode operating on the same core tensor and graph runtime so networks can be evolved, fine-tuned, and deployed interchangeably. Supports SGD-style training with reverse-mode autodiff, NEAT-style topology-evolving neural networks with speciation and compatibility distance, DEAP-style population-based weight evolution, and static graph execution with forward/backward scheduling. Hybrid optimisation modes can combine evolution and gradient descent in a single training loop (Lamarckian or Baldwinian). Architected around trait-based modularity: genomes, fitness functions, selection strategies, genetic operators, loss functions, and activations are all implemented as interchangeable trait implementations. Built entirely in safe parallel Rust with no Python bindings and no unsafe blocks.

## Project 7

- **Name**: Consilium
- **URL**: https://github.com/Capataina/Consilium
- **Type**: Personal
- **Tech stack**: Python, LangChain, MCP
- **Status**: Abandoned (not enough interest but working demo available)
- **Summary**: A multi-LLM debate and knowledge synthesis platform that orchestrates structured multi-round debates between heterogeneous LLMs (Claude, GPT, Gemini, and local Ollama models) and synthesises their perspectives into a single structured knowledge artefact.
- **Your role**: Solo.
- **Technical highlights**: Built on the observation that every major AI model has different training data, different objectives, and different blind spots, and that comparing answers across providers is more valuable than relying on any single one if there is a structured way to compare, contrast, and synthesise them. The orchestrator runs structured debate rounds in which each agent responds independently without seeing others' raw output, a dedicated summariser agent compresses each round's output into a running summary to manage context and enable unlimited debate length, and convergence and divergence detection identifies which positions models agree on and which they persistently disagree on across rounds. Built on LangChain for provider-agnostic LLM orchestration so commercial and local models participate on equal footing, and on MCP for shared tool access so all agents regardless of provider can call the same tools (web search, calculator, citation fetcher) through a unified interface. Every debate produces a final synthesis document capturing arguments made, consensus reached, unresolved disagreements, and metadata, alongside the full transcript and a convergence report.

## Project 8

- **Name**: Zyphos
- **URL**: https://github.com/Capataina/Zyphos
- **Type**: Personal
- **Tech stack**: Rust
- **Status**: In Progress (not enough interest)
- **Summary**: A network protocol laboratory built through hands-on HTTP server implementation, designed to internalise sockets, protocols, and network programming while applying performance patterns from compilers, trading systems, and distributed infrastructure where they naturally fit.
- **Your role**: Solo.
- **Technical highlights**: Bottom-up: raw sockets and system calls first, then TCP state machines and connection handling, then HTTP/1.0 parsing and routing, then thread pools, memory pools, zero-copy buffers, HTTP/1.1 connection pooling, and epoll/kqueue event loops, then HTTP/2 and beyond. Cross-domain technique application is deliberate: SIMD techniques from compiler engineering applied to protocol parsing, lock-free structures from HFT applied to connection handling, arena allocation and zero-copy buffers from systems work, NUMA awareness and CPU pinning from low-latency infrastructure, and time and ordering primitives (NTP, PTP, vector clocks) from distributed systems. Measurement obsessed throughout: syscall counts, cache misses, packet rates, and latency percentiles tracked at every milestone. Security worked through by implementation: deliberately triggering slowloris, SYN floods, and request smuggling, then fixing them.

## Project 9

- **Name**: Chrona
- **URL**: https://github.com/Capataina/Chrona
- **Type**: Personal
- **Tech stack**: C++
- **Status**: In Progress (not enough interest)
- **Summary**: A Git-inspired, local-first version control core built in modern C++ to internalise how real version control works under the hood: content-addressed storage, immutable snapshots, commit graphs, staging and index semantics, and diffs.
- **Your role**: Solo.
- **Technical highlights**: Built around the deliberate distinction that Git is the version control system (the local engine and data model) while GitHub and GitLab are hosting platforms layered on top, and that most developers learn the commands without ever understanding the engine. Targets the inner engine specifically: a content-addressed object database providing integrity and deduplication, snapshot-based versioning with trees, blobs, and commits as immutable objects, a commit DAG representing history as a graph rather than a linear list, an index/staging model separating working tree from staged from committed state, and diffs as a derived view rather than the primary storage truth. Networking, history rewriting, packfiles, and enterprise extras are explicitly excluded to keep the focus on the core data model. Prioritises clarity, correctness, deterministic encoding, stable hashing, and testable invariants before performance and ergonomics.

## Project 10

- **Name**: Xyntra
- **URL**: https://github.com/Capataina/Xyntra
- **Type**: Personal / research-style
- **Tech stack**: Rust
- **Status**: In Progress (not enough interest)
- **Summary**: An automatic kernel-fusion compiler pass written entirely in safe Rust that ingests ONNX and TorchScript graphs, pattern-matches common op-chains, and emits a single fused GPU kernel through `wgpu` (cross-platform WGSL) or optional CUDA PTX.
- **Your role**: Solo.
- **Technical highlights**: Explores graph rewriting, GPU occupancy modelling, and autotuned code generation while keeping the entire pipeline 100% unsafe-free. Built around a type-safe IR with explicit error classification, a modular crate layout (`xyntra-core`, `xyntra-cli`, `xyntra-ir`), an `egg`-based e-graph integration for rewrite rules and saturation, a declarative fusion DSL for expressing patterns like `matmul → gelu → dropout`, scheduling heuristics with a cost model for fusion candidates, fusion legality checks for shape, dtype, and broadcast guards, WGSL and CUDA PTX backends with shared-memory tiling and vectorisation, and an autotuning harness using Bayesian optimisation over tile sizes. Correctness is enforced through golden unit tests comparing fused versus unfused outputs, gradient checks, edge-case libraries for broadcast, dynamic shapes, and odd strides, and configurable numerical tolerance thresholds. Observability includes structured tracing, kernel timeline JSON dumps, GPU occupancy analysis covering register and SM utilisation, HDR latency histograms, and roofline modelling.

## Project 11

- **Name**: Tectra
- **URL**: https://github.com/Capataina/Tectra
- **Type**: Personal
- **Tech stack**: C++, FlatBuffers, Protobuf, Prometheus
- **Status**: In Progress (not enough interest)
- **Summary**: A modern C++ trading-infrastructure stack combining a low-latency market-data feed handler, pre-trade risk service, firm-wide kill switch, deterministic replay engine, strategy execution framework, backtesting engine, and signal research toolkit into one cohesive production-style system.
- **Your role**: Solo.
- **Technical highlights**: Focuses on the invisible infrastructure every serious trading firm relies on, plus the strategy execution layer that sits on top of it. The feed handler decodes ITCH/OUCH-like streams, manages sequencing and recovery, builds L2 books, and publishes a unified internal schema. The pre-trade risk service performs microsecond-level rule checks for price bands, size and notional limits, credit caps, and per-venue throttles, with hot-reloadable limits and full auditability. The kill-switch and circuit-breaker layer provides automatic and manual triggers that freeze or slow order flow, cancel open orders, or isolate venues with sub-millisecond propagation. The deterministic replay engine uses append-only checksummed journals with time-virtualised playback and golden-run diffing for post-incident analysis. The strategy execution framework is a plugin architecture with signal generation, order management, position tracking, and PnL calculation; the backtesting engine performs time-virtualised historical replay with simulated fills, slippage models, and transaction cost analysis; the signal library covers technical indicators, statistical arbitrage, mean reversion, momentum, and ML feature extraction. Built with C++20, lock-free shared-memory rings, FlatBuffers/Protobuf schemas for contract-first messaging, Prometheus metrics with per-strategy PnL, Sharpe ratios, and drawdown tracking, and tamper-evident logging with append-only journals carrying checksums and Merkle roots. Dual-plane architecture separates the binary fast path for low-latency data from the structured control plane for operators and metrics.

## Project 12

- **Name**: Neuronika
- **URL**: https://github.com/Capataina/Neuronika
- **Type**: Personal
- **Tech stack**: Python
- **Status**: Completed
- **Summary**: AI-powered personal knowledge management tool that combines embedding-based retrieval with a semantic graph visualisation of the user's notes and concepts.
- **Your role**: Solo.
- **Technical highlights**: Built around the idea that personal notes are most useful when surfaced by meaning rather than by filename or folder. Combines vector embedding retrieval with a graph representation of the relationships between notes, enabling both semantic search and visual exploration of how ideas connect.

## Project 13

- **Name**: AsteroidsAI
- **URL**: https://github.com/Capataina/AsteroidsAI
- **Type**: Personal benchmarking project
- **Tech stack**: Python, NEAT, DEAP, PyTorch
- **Status**: Completed
- **Summary**: A benchmarking platform that compares multiple optimisation paradigms (NEAT, genetic algorithms, evolution strategies, and a GNN-based reinforcement learning agent using SAC) on a single Asteroids-style environment under controlled conditions.
- **Your role**: Solo.
- **Technical highlights**: Designed to put evolutionary and gradient-based reinforcement learning approaches side by side under the same task, the same observation space, and the same reward structure, making the differences between paradigms empirically visible rather than theoretical. Implements NEAT for topology-evolving networks, classical genetic algorithms via DEAP for population-based weight search, evolution strategies as a gradient-free baseline, and a GNN policy trained with Soft Actor-Critic as the deep RL representative.

## Project 14

- **Name**: Credit Card Fraud Detection
- **URL**: <!-- TODO: confirm public link if any -->
- **Type**: Personal / academic-style
- **Tech stack**: Python, scikit-learn, XGBoost
- **Status**: Completed
- **Summary**: A fraud detection model trained on a heavily class-imbalanced credit card transaction dataset, achieving 94% precision through gradient boosting and careful evaluation methodology.
- **Your role**: Solo.
- **Technical highlights**: Treats class imbalance as the central problem rather than an afterthought: the dataset contains a tiny fraction of fraudulent transactions, so naive accuracy is meaningless and precision/recall trade-offs dominate every modelling decision. Uses gradient boosting via XGBoost with careful threshold tuning and evaluation against precision, recall, and ROC-AUC rather than accuracy alone.

## Project 15

- **Name**: Personal Website
- **URL**: <!-- TODO: confirm if currently deployed -->
- **Type**: Personal
- **Tech stack**: TypeScript, JavaScript
- **Status**: Abandoned (not enough interest)
- **Summary**: A personal website with a custom particle physics simulation as the centrepiece interactive element.
- **Your role**: Solo.
- **Technical highlights**: Particle simulation written from scratch rather than using a physics library, integrating directly with the page as the primary visual identity of the site.

## Project 16

- **Name**: Game Modding Portfolio
- **URL**: <!-- Steam Workshop / Nexus / CurseForge profiles -->
- **Type**: Hobbyist software, hosted on public mod platforms
- **Tech stack**: C#, XML, game-specific scripting and modding APIs across RimWorld, Minecraft, Terraria, and Escape from Tarkov
- **Status**: Completed
- **Summary**: A portfolio of 18+ released mods across four games totalling over 150,000 combined downloads.
- **Your role**: Solo creator on every mod. Design, implementation, debugging, end-user support, and updates.
- **Technical highlights**: The original outlet for the same drive that produces the current systems projects: reverse-engineering of game internals to extend behaviour, working within strict compatibility constraints across game versions and other mods, shipping software to a real user base, and handling bug reports and feature requests from end users. Considered formative engineering experience, with the same loop (read foreign code, find the extension points, build something coherent on top) that underpins more recent infrastructure work.