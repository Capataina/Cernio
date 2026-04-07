# Skills

## Programming Languages

| Language   | Proficiency | Notes |
|------------|-------------|-------|
| Rust       | Proficient  | Primary language. Used across all current major projects: Image Browser, Aurix, NeuroDrive, Nyquestro, Cernio. Comfortable with async (tokio), lock-free concurrency, ownership-driven design, and FFI to ONNX Runtime. |
| Python     | Proficient  | ML, data work, scripting, and open source contribution (tinygrad). Familiar with NumPy, pandas, PyTorch, TensorFlow, scikit-learn, XGBoost, DEAP. |
| TypeScript | Comfortable | Tauri frontends in React for Image Browser and Aurix. |
| JavaScript | Comfortable | Frontend work alongside TypeScript. |
| Java       | Familiar    | Lead developer on University of York 8-person team simulation game using libGDX. |
| C++        | Familiar    | Used for systems exploration and version-control internals work (Chrona). |

## Frameworks and Libraries

| Framework / Library | Proficiency | Notes |
|---------------------|-------------|-------|
| Tauri               | Proficient  | Desktop application shell for Image Browser and Aurix. |
| React               | Comfortable | Frontend layer for Tauri desktop applications. |
| Tokio               | Comfortable | Async runtime across Rust projects. |
| Bevy                | Comfortable | 8-subsystem application in NeuroDrive: ECS architecture, fixed-timestep simulation, plugin composition, system ordering sets, world-space debug overlays, UI panels. |
| Ratatui             | Comfortable | Terminal UI for Cernio's live dashboard. |
| ONNX Runtime        | Comfortable | Local CLIP inference in Image Browser; LSTM operator contribution to tinygrad. |
| PyTorch             | Comfortable | ML coursework and experimentation. |
| TensorFlow          | Comfortable | Google ML Crash Course and earlier ML work. |
| scikit-learn        | Comfortable | Classical ML, used in credit card fraud detection project. |
| XGBoost             | Comfortable | Gradient boosting for tabular tasks. |
| DEAP                | Comfortable | Evolutionary algorithms during university coursework and AsteroidsAI. |
| NEAT                | Comfortable | Neuroevolution baseline in AsteroidsAI. |
| libGDX              | Familiar    | Java game framework, used as lead developer in university team project. |

## Tools and Platforms

| Tool / Platform | Proficiency | Notes |
|-----------------|-------------|-------|
| Git / GitHub    | Proficient  | Daily use across all projects; account `Capataina`. |
| SQLite          | Comfortable | Local persistence for Image Browser, Aurix, Cernio. |
| Linux           | Comfortable | HackTheBox Linux Fundamentals; daily-driver familiarity. |
| Claude Code     | Comfortable | Used as primary AI development environment. |
| Codex CLI       | Comfortable | Secondary AI development tool. |
| MCP             | Comfortable | DeepLearning.AI / Anthropic course; understanding of multi-agent and tool-routing patterns. |
| Wireshark       | Familiar    | University network analysis coursework. |
| Nmap            | Familiar    | Network reconnaissance during coursework. |
| tcpdump         | Familiar    | Packet capture during coursework. |

## Domains and Concepts

| Domain                            | Depth       | Notes |
|-----------------------------------|-------------|-------|
| Systems programming               | Comfortable | Lock-free data structures, multithreading, memory safety, low-latency design. |
| Market microstructure             | Comfortable | Order book mechanics, price-time priority matching, market-making, order flow imbalance, adverse selection — built directly in Nyquestro. |
| DeFi and AMM mathematics          | Comfortable | Uniswap V3 tick mathematics, LP backtesting, cross-DEX arbitrage, on-chain analytics — built in Aurix. |
| Quantitative risk modelling       | Comfortable | VaR, rolling volatility, fat-finger protection, position bounds — implemented in Aurix and Nyquestro. |
| Local-first software architecture | Comfortable | Three production projects built on the principle: Image Browser, Aurix, Cernio. |
| Local ML inference                | Comfortable | CLIP via ONNX Runtime in Image Browser; FFI integration end to end. |
| Reinforcement learning            | Comfortable | Handwritten PPO from scratch in NeuroDrive: clipped surrogate objective, GAE, multi-epoch amortised updates, asymmetric actor-critic architecture, reward engineering under entertainment constraints. No framework dependency. |
| Biologically plausible learning   | Familiar    | Research-level understanding of Hebbian plasticity, STDP, dopamine-modulated reinforcement, sparse plastic graphs — NeuroDrive's biological phase is designed but not yet implemented. |
| Compiler / runtime concepts       | Familiar    | ONNX operator implementation in tinygrad; early-stage exploration in Xyntra (graph and kernel-fusion compiler). |
| Network analysis and security     | Familiar    | University coursework using Wireshark, Nmap, tcpdump for DDoS pattern identification. |
| Linear algebra and optimisation   | Comfortable | Underpins ML, AMM mathematics, and risk modelling work. |

## Soft Skills

- Self-directed learning — entire current project portfolio is self-initiated and self-driven.
- Technical writing — maintains detailed README files and project documentation as a default working practice.
- Cross-disciplinary synthesis — willing to learn the domain (market microstructure, AMM mechanics, biological learning) before building, rather than treating it as an afterthought.
- Honesty under pushback — pushes back precisely and accepts correction; documents project status truthfully (e.g. tinygrad PR closed, not merged).
- Lead developer experience — coordinated architecture, feature implementation, and testing pipelines for an 8-person team game project at university.

## Methodologies

- Iterative milestone-driven development with explicit exit criteria per milestone.
- Test-driven validation for correctness-critical work (regression tests against ONNX Runtime in tinygrad PR; deterministic instrumentation in NeuroDrive).
- Plain-text, inspectable artefacts as a default (markdown, TOML, JSONL, SQLite over opaque formats).