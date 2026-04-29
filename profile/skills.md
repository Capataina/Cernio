# Skills

> Derived from project files in `profile/projects/`. Last synced: 2026-04-29.
> Proficiency bands: Proficient | Comfortable | Familiar | Beginner.
> Per `populate-from-lifeos` rubric — evidence-anchored, not LOC-based.

---

## Programming Languages

| Language | Proficiency | Evidence (specific projects + what they demonstrate) |
|----------|-------------|------------------------------------------------------|
| **Rust** | Proficient | Cross-domain across 8 projects: Cernio (TUI + 6 ATS fetchers + 6 pipeline scripts + 325-test suite + lib+bin split), Image Browser (multi-encoder ML inference via ort, dual-connection SQLite, 26 Tauri commands), NeuroDrive (Bevy ECS + handwritten PPO + biologically-inspired learning, 80 files, 660KB, 133 tests), Aurix (Tauri 2 backend + Uniswap V3 sqrtPriceX96 via num-bigint), Nyquestro (allocation-free primitives + Copy-based event frames + 11-variant error taxonomy), Vynapse (4-crate workspace + trait-based modular architecture + 120 inline tests, zero `unsafe`), Xyntra (typed IR + 4-category error taxonomy), Zyphos (raw-TCP HTTP/1.1 with thread-per-connection + panic recovery). Cross-domain transfer lift confirmed (TUI + database + game engine + ML inference + DeFi + HFT-style systems + desktop apps + networking = 8+ domains). |
| **Python** | Comfortable | AsteroidsAI (NumPy-only GA / CMA-ES / NEAT, PyTorch + torch_geometric SAC, ~141 files, ~806KB), Consilium (LangChain provider adapters, Textual TUI, structured-state debate emission, ~106KB), tinygrad ONNX LSTM operator implementation (forward + reverse + bidirectional, 11 RNN activations, regression tests against ONNX Runtime). Cross-domain: ML/RL + LLM orchestration + ONNX framework contribution. |
| **C++** | Familiar | Chrona (C++20, `std::filesystem`, `std::optional`, enum classes, CMake + Catch2 v3.5.0 — VCS scaffold, ~21KB source paused), Tectra (C++20 trading-infra scaffold with `Clock` interface + `RealClock` + `VirtualClock` + nanosecond `Timestamp`, `-Werror` from day one). Both projects paused at scaffold stage; design ambition documented but execution depth bounded. |
| **TypeScript** | Comfortable | Image Browser (33 frontend files, React 19 + TanStack Query + framer-motion + ApiError discriminated union mirrored from Rust), Aurix (~1000 lines in `src/features/arbitrage/`, insights.ts 14.9KB analytical core, MarketChart 13.4KB). Two substantial Tauri-frontend projects with cross-runtime serde contracts. |

---

## Frameworks

| Framework | Proficiency | Evidence |
|-----------|-------------|----------|
| **Tauri 2** | Proficient | Image Browser (26 invoke handlers, ApiError typed wire format, `manage()` state, native folder picker, CLI plugin, dialog plugin), Aurix (single IPC command, camelCase serde contract). Two desktop apps with cross-runtime ApiError discriminated union. |
| **React 19** | Comfortable | Image Browser (33 TS files, hooks + TanStack Query staleTime:Infinity + framer-motion masonry tilt + multi-section settings drawer + perf overlay), Aurix (functional components + hooks; 100-sample rolling window). |
| **Bevy** | Comfortable | NeuroDrive (8-subsystem ECS architecture, fixed-timestep simulation at 60Hz, plugin composition, system ordering sets, world-space debug overlays, UI panels, F4-cycled trainer layouts). Single but deep project; ECS understood at architecture level. |
| **Tokio** | Comfortable | Cernio (async pipeline scripts: resolve, search, clean, check; Semaphore for parallel-fetch concurrency control), Aurix (1Hz polling loop with macros + rt-multi-thread). Multiple substantive contexts. |
| **Ratatui** | Comfortable | Cernio (5-view TUI, 26 source files, mouse support, responsive layout, semantic colour palette, modular `app/handler/views/widgets/` split, 2s SQLite poll). Single deep project. |
| **LangChain** | Familiar | Consilium (`langchain_ollama` + `langchain_google_genai` provider adapters, per-slot sampling profiles, structured-state JSON between rounds). One project, paused at Milestone 3. |
| **Textual** | Familiar | Consilium (minimal compose-run-result TUI flow). One project, paused. |
| **Vite** | Familiar | Image Browser (Vite 7 + vite-plugin-pages file-based routing), Aurix (Vite 7 build tool). Build-tool integration, not deep customisation. |

---

## Libraries

| Library | Proficiency | Evidence |
|---------|-------------|----------|
| **ort (ONNX Runtime Rust)** | Comfortable | Image Browser (3 image encoders + 2 text encoders, shared M2-tuned `Session` builder Level3 / intra=4 / inter=1, CPU on macOS, CUDA on non-macOS with CPU fallback). Single but deep — multi-encoder lifecycle + parallel encode. |
| **rusqlite (SQLite, WAL)** | Comfortable | Cernio (5 tables, 6 migrations, 29 inline schema tests), Image Browser (5 tables + dual connection writer/reader, 4 idempotent migrations, embedding BLOB via bytemuck::cast_slice replacing 3 unsafe blocks, foreign keys + busy_timeout + manual checkpoint). Two projects with substantive WAL + migration discipline. |
| **HF tokenizers** | Familiar | Image Browser (BPE for CLIP, SentencePiece for SigLIP-2, uniform `tokenizer.json` interface across both). Single project, but cross-tokeniser portability proved. |
| **num-bigint** | Familiar | Aurix (Uniswap V3 sqrtPriceX96 Q64.96 fixed-point decode). Single project, narrow context but mathematically demanding. |
| **PyTorch** | Familiar | AsteroidsAI (SAC twin critics + actor network), tinygrad LSTM operator regression tests against PyTorch reference. Two contexts; framework as runtime, not deep autodiff customisation. |
| **torch_geometric** | Familiar | AsteroidsAI (GATv2Conv backbone for GNN-SAC). Single context, narrow. |
| **NumPy** | Familiar | AsteroidsAI (NumPy-only GA + CMA-ES + NEAT — three evolutionary methods, no torch dependency). Substantial in one project; no cross-domain breadth. |
| **reqwest + rustls-tls** | Comfortable | Cernio (shared HTTP client with retry helpers, per-request exponential backoff in `common.rs::get_with_retry`), Aurix (rustls-tls for cross-platform Tauri builds without OpenSSL dependency). Two projects; rustls discipline shared. |
| **Serde** | Comfortable | Cernio (JSON for ATS responses, TOML for `preferences.toml`), Image Browser (ApiError discriminated union with `#[serde(tag="kind", content="details")]`), Aurix (`rename_all="camelCase"` cross-runtime contract). Cross-domain serialisation use including discriminated unions. |
| **chromiumoxide** | Familiar | Cernio (Chrome CDP autofill — scaffolded but currently broken on React forms; CDP architecture understood). Single project, broken state, depth bounded. |
| **chrono** | Familiar | Nyquestro (`Ts` nanosecond timestamp wrapper), Zyphos (RFC 1123 date header). Two narrow contexts. |
| **thiserror** | Familiar | Aurix, Nyquestro (declarative error enum). Standard Rust idiom. |
| **fast_image_resize** | Familiar | Image Browser (NEON-optimised Lanczos3 thumbnail downsample). Single context. |
| **bytemuck** | Familiar | Image Browser (zero-copy float-array → byte-slice for embedding BLOB; replaced 3 `unsafe` blocks). Specific use, narrow but correct. |
| **notify-debouncer-mini** | Familiar | Image Browser (filesystem watcher with 5s debounce, single-flight coalescing). Single context. |
| **rayon** | Familiar | Image Browser (data-parallel thumbnail generation). Single context. |
| **assert_cmd / proptest / tempfile** | Familiar | Cernio (16 CLI integration tests via `assert_cmd::Command::cargo_bin("cernio")` + `CERNIO_DB_PATH` env var). Substantive test infrastructure but in one project. |

---

## Engines and Runtimes

| Engine | Proficiency | Evidence |
|--------|-------------|----------|
| **ONNX Runtime** | Comfortable | Image Browser (3 image encoder graphs + 2 text encoder graphs, shared session builder, CPU + CUDA paths). tinygrad ONNX LSTM contribution: implementation tested against ONNX Runtime as ground truth (regression tests in `external_test_onnx_ops.py`). Cross-context use spanning Rust runtime consumption + framework-internal operator implementation. |
| **Bevy ECS** | Comfortable | NeuroDrive (8 subsystems, plugin composition, system ordering sets, fixed-timestep simulation, world-space debug overlays). Architecture-depth understanding of ECS execution model. |
| **PyTorch (as runtime)** | Familiar | AsteroidsAI (SAC twin critics + actor network execute on PyTorch tensor graph). Framework consumption, not deep graph manipulation. |
| **SQLite (as data store)** | Comfortable | Cernio + Image Browser. WAL mode, migrations, manual checkpoint between batches, dual connections (Image Browser), idempotent migration framework (both). |

---

## Tools and Platforms

| Tool | Proficiency | Evidence |
|------|-------------|----------|
| **Git / GitHub** | Comfortable | Daily use across all 12 projects + OSS contributions; native Claude Code Skill tool integration via `.claude/skills/` (Cernio); `gh` CLI for cross-repo automation (the populate-from-lifeos skill itself). |
| **Cargo** | Comfortable | All 7 Rust projects; workspace design (Vynapse 4-crate workspace, Cernio lib+bin split with `CERNIO_DB_PATH` for test isolation), build profiles, integration tests via `tests/`. |
| **CMake** | Familiar | Chrona (FetchContent Catch2 v3.5.0, two-target build), Tectra (CMake 3.20 with `-Wall -Wextra -Wpedantic -Werror`). Two narrow contexts. |
| **Catch2** | Familiar | Chrona (v3.5.0 via FetchContent). One context. |
| **Vitest** | Familiar | Image Browser (62 frontend tests). |
| **Vite** | Familiar | Image Browser, Aurix. |
| **Claude Code skills** | Proficient | Cernio (9 project-local skills with mandatory-read protocols, evidence-anchored quality gates, obligation-anchored framing, what-I-did-not-do declarations, Tier-3 quality checklists), the populate-from-lifeos skill itself (this very file's source). Skill-engineering depth demonstrated through iteration: 4 grading-rubric iterations driven by production failures, skill-creator-audited migration of all 9 skills. |
| **Obsidian / LifeOS** | Comfortable | The LifeOS vault that this `profile/` syncs from — a custom structured personal-information system with cross-repo profile distribution feeding downstream career tools. Externalised-state discipline applied at vault scale. |

---

## Concepts and Domains

| Domain | Proficiency | Evidence |
|--------|-------------|----------|
| **Reinforcement learning** | Proficient | NeuroDrive (handwritten PPO with clipped surrogate objective, GAE, asymmetric actor-critic 2×64 actor + 2×128 critic, AdamW, PopArt critic target scaling, dual GEMM backend, target-KL stop, observation normalisation; M4 ~21× frame-time improvement; biologically-inspired learner with three-factor plasticity, eligibility traces, raw-reward modulator, homeostasis, continual-backprop structural plasticity). AsteroidsAI (Soft Actor-Critic with twin critics + GNN-encoded state via GATv2Conv; CMA-ES + GA + NEAT comparators on the same environment). Cross-method depth: PPO + biological plasticity + SAC + evolutionary methods all hand-rolled. |
| **Local-first architecture** | Proficient | Image Browser (offline ML inference, 2.5GB models managed locally, SQLite WAL, no cloud), Cernio (single SQLite file, no Docker, no server, no API keys), Aurix (free public RPC only, no paid APIs, never submits transactions). Three substantial projects all framed as local-first. |
| **Multi-encoder retrieval / late fusion** | Comfortable | Image Browser (Reciprocal Rank Fusion via Cormack 2009 k=60 over 3 image encoders for image-image and 2 text encoders for text-image; FusionIndexState lazy populate; rank-based fusion sidesteps cosine-distribution mismatch). Single but research-paper-anchored. |
| **Vision-language semantic search** | Comfortable | Image Browser (CLIP + SigLIP-2 text-image fusion, three encoder families running in parallel on indexing). |
| **DeFi market microstructure** | Familiar | Aurix (Uniswap V3 sqrtPriceX96 Q64.96 fixed-point decode via num-bigint, Uniswap V2 reserve-ratio decode, SushiSwap V2-fork compatibility, four-venue WETH/USDC arbitrage detection). Single project, paused. |
| **Lock-free / wait-free design** | Familiar | Nyquestro (allocation-free primitives, immutable Copy-based event frames, lock-free OrderBook design intent in Architecture documents — implementation foundation only). Design-level understanding; matching engine itself unbuilt. |
| **Market microstructure / matching engines** | Familiar | Nyquestro (limit order book design, price-time priority intent, fill mechanics, order lifecycle, hardening plan). Tectra (trading-infra design space — feed handler, pre-trade risk, kill switch, deterministic replay). Both at scaffold stage. |
| **Compiler IR design** | Familiar | Xyntra (typed `NodeID` / `TensorShape` / `OpKind` primitives, HashMap-backed node store, 1000-node stress test, 4-category error taxonomy, GPU-parameter validation). Single project, dormant at IR layer; downstream compiler stages absent. |
| **Network protocols / HTTP** | Familiar | Zyphos (raw-socket TCP listener, thread-per-connection with panic recovery, HTTP/1.1 request-line parsing, response serialisation with RFC 1123 date headers, exact-match + prefix-strip routing). Single dormant project at Milestone 5 of 30. |
| **Evolutionary algorithms / neuroevolution** | Comfortable | Vynapse (Rust evolutionary trainer with μ+λ replacement, tournament selection, Gaussian mutation, uniform crossover; trait-based modular architecture, 120 inline tests). AsteroidsAI (NumPy GA + diagonal CMA-ES with Pareto ranking + NEAT speciation/innovation tracking from scratch). Cross-language depth: Rust + Python implementations of similar paradigms. |
| **Local ML inference** | Comfortable | Image Browser (ONNX Runtime running 3 image + 2 text encoder graphs locally; ~2.5GB models managed via fail-soft per-file HuggingFace download; CPU on macOS, CUDA on non-macOS with fallback). Specific deep technique work (shared session builder, per-encoder parallelism, parallel-by-encoder indexing). |
| **Multi-LLM orchestration** | Familiar | Consilium (heterogeneous-model debate driver, structured-state 8-key JSON between rounds, fallback synthesis from raw turns, anti-evaluative prompt contract). Single project, paused. |
| **Conversational AI orchestration** | Comfortable | Cernio (scripts-handle-volume / AI-handles-judgment split; mandatory-read protocols; obligation-anchored quality gates; Living System Philosophy where skills never embed profile snapshots). Foundational architectural pattern across 9 skills. |
| **VCS internals** | Familiar | Chrona (content-addressed object storage design intent + commit DAG + immutable snapshots + staging + refs as design space; only repo-discovery walk-up + error model implemented). Design awareness, implementation depth bounded. |
| **Time virtualisation / deterministic replay** | Familiar | Tectra (`Clock` interface + `RealClock` + `VirtualClock` as foundational primitive for deterministic replay design). NeuroDrive (multi-layer determinism + reproducibility status documented). Two contexts at design level. |
| **Performance engineering** | Comfortable | NeuroDrive (M4 ~21× frame-time improvement via dual-GEMM backend on Accelerate AMX + batched actor; 60Hz target on M2 MacBook Air as deliberate constraint), Image Browser (partial-sort cosine 2.53× speedup, persistent `cosine_cache.bin` with mtime freshness, batch-checkpoint embedding writes replacing per-row autocommit that caused multi-second WAL stalls). Two substantive perf-engineering arcs. |
| **Skill ecosystem engineering** | Proficient | Cernio (9 skills with mandatory-read protocols, evidence-anchored quality gates, obligation-anchored framing, what-I-did-not-do declarations, Tier-3 quality checklists, 4 grading-rubric iterations driven by production failures). The populate-from-lifeos skill itself is meta-evidence — a skill that orchestrates parallel synthesis subagents under a strict evidence-block contract. |
| **Markdown synthesis from multi-source vault** | Comfortable | The LifeOS profile sync workflow itself; `populate-from-lifeos` skill orchestrating per-project synthesis under schema constraints. |
| **Audit-passed posture / code-health discipline** | Comfortable | Image Browser (28 code-health-audit findings, all shipped), Cernio (27-finding audit categorised + queued by batch dependency, plans live in `context/plans/code-health-audit/`). Two substantial code-health audit cycles. |

---

## Methodologies and Soft Skills

The portfolio's pattern signal:

- **Iterative milestone-driven development.** Cernio's 9 sessions, NeuroDrive's M1–M6 progression, Vynapse's plans-driven refactor pass. Plans-as-source-of-truth (`plans/NNN_topic.md` schema) recurs across LifeOS, Cernio, Vynapse, Chrona.
- **Test-driven validation for correctness-critical work.** Cernio session-9 added 306 tests retroactively and surfaced two silent data-loss bugs immediately. Image Browser ships 125 cargo lib + 62 vitest tests with audit-passed posture. Nyquestro has 854 lines of integration tests against ~49KB of source. Vynapse has 120 inline `#[test]` functions across 12 files.
- **Plain-text inspectable artefacts.** Cernio's preferences.toml + `context/` markdown architecture. NeuroDrive's 36 markdown context docs (939KB) + 60 learning files (504KB). Image Browser's profiling layer emits `report.md` with Stall Analysis. LifeOS as the meta-instance.
- **Cross-disciplinary synthesis.** The portfolio spans systems Rust (Cernio, Nyquestro, Tectra, Zyphos), ML inference (Image Browser, NeuroDrive), evolutionary methods (Vynapse, AsteroidsAI), DeFi (Aurix), networking (Zyphos), compilers (Xyntra), VCS internals (Chrona), LLM orchestration (Consilium). Multiple paradigms tackled in the same language (Rust evolutionary + Python evolutionary; Rust trading + C++ trading) demonstrating cross-language pattern transfer.
- **Anti-puffing discipline.** Every project's vault notes explicitly enumerate README-vs-reality discrepancies. The honesty about what is built vs intended is itself a portfolio signal — it lets future sessions trust the documentation without re-verifying every claim.
- **Documentation-first iteration.** Nyquestro has ~2.6:1 documentation-to-source ratio; Tectra has ~13:1. Hardening plans, design decisions, and reality-vs-README reconciliations exist before implementation rather than after.
- **Obligation-anchored over exhortation-anchored skill design.** The principal-engineering decision (Cernio session 9, cross-domain) of replacing vague "be thorough" framing with verifiable obligations — produce artefact X, emit section Y, quote the last line of each reference — is itself a transferable methodology demonstrated through skill iteration.

---

## Evidence Block

| Project file | Lines | Verbatim last line |
|---|---|---|
| profile/projects/cernio.md | 195 | "\| Projects/Cernio/Work/Profile Populate Skill.md \| 197 \| \"- LifeOS commit `cf14e1d` — Phase 1 landing commit\" \|" |
| profile/projects/image-browser.md | 210 | "\| Projects/Image Browser/Systems/Watcher.md \| 83 \| \"- `Capataina/PinterestStyleImageBrowser/context/systems/watcher.md` — full implementation notes\" \|" |
| profile/projects/aurix.md | 150 | "\| Projects/Aurix/Work/Tab 2 Timeboost MEV Analytics.md \| 74 \| \"#aurix #work #defi #timeboost #mev #sequencer\" \|" |
| profile/projects/neurodrive.md | 189 | "\| Projects/NeuroDrive/Work/Performance.md \| 30 \| \"#neurodrive #work #performance\" \|" |
| profile/projects/nyquestro.md | 147 | "\| Projects/Nyquestro/Work/V2 Distributed Extension.md \| 67 \| \"#nyquestro #work #distributed-systems #consensus\" \|" |
| profile/projects/vynapse.md | 149 | "\| Projects/Vynapse/Systems/Traits Layer.md \| 185 \| \"- Stubs that will need new trait work: [[Vynapse/Gaps#Traits that need extension]]\" \|" |
| profile/projects/asteroidsai.md | 152 | "\| Projects/AsteroidsAI/Systems/State Encoders.md \| 219 \| \"- [[AsteroidsAI/Gaps]] — encoder drift, schema versioning, VectorEncoder dead code\" \|" |
| profile/projects/consilium.md | 148 | "\| Projects/Consilium/Systems/Transcripts.md \| 165 \| \"#project/consilium #domain/persistence #domain/output\" \|" |
| profile/projects/chrona.md | 137 | "\| Projects/Chrona/Systems/Repo Discovery.md \| 129 \| \"- [[Chrona/Gaps]] — the `exists` vs `is_directory` gap and the commented-out tests\" \|" |
| profile/projects/xyntra.md | 148 | "\| Projects/Xyntra/Systems/Validation.md \| 112 \| \"`#project/xyntra` `#validation` `#scaffold` `#todo` `#next-work`\" \|" |
| profile/projects/zyphos.md | 132 | "\| Projects/Zyphos/Systems/Testing.md \| 113 \| \"- [[Roadmap]] — test priorities in the next session\" \|" |
| profile/projects/tectra.md | 127 | "\| Projects/Tectra/Systems/Logging.md \| 84 \| \"- [[Tectra/Roadmap]] — the README's Milestone 1 (Foundations) is where logging gets wired\" \|" |
| profile/projects/open-source-contributions.md | 189 | "\| Projects/Open Source Contributions/Tinygrad.md \| 130 \| \"- [[Projects/Open Source Contributions/Burn\|Burn]] — sister contribution notes\" \|" |
