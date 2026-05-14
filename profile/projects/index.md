---
title: Projects Index
last_synced: 2026-05-13
project_count: 14
sources: 14 per-project files derived from `Capataina/Capataina` README allow-list (Active + Other + Open Source sections)
---

# Projects Index

Navigation map for Cernio's per-project profile files. Each entry below points at a comprehensive file produced by `populate-from-lifeos` from the corresponding LifeOS `Projects/<Name>/` folder. Status field reflects the LifeOS source-of-truth, not inference.

> Active projects = ongoing build work. Paused = started, paused for known reason, may resume. Dormant = no activity 6+ months, no clear pause reason. Files in this folder are evidence-anchored and ceiling-bounded by what the LifeOS source supports — anti-puffing applies per the synthesis schema.

---

## Active

| Project | Source repo | Primary stack | Brief |
|---------|-------------|---------------|-------|
| [Cernio](cernio.md) | [Capataina/cernio](https://github.com/Capataina/cernio) | Rust + Ratatui + SQLite + Tokio | Local-first collaborative job discovery + curation engine combining a SQLite store, 6 ATS provider fetchers, a 5-view Ratatui TUI, and 9 native Claude Code skills into a system where scripts handle volume and AI handles judgment inside conversational sessions. |
| [Image Browser](image-browser.md) | [Capataina/PinterestStyleImageBrowser](https://github.com/Capataina/PinterestStyleImageBrowser) | Rust + Tauri 2 + React 19 + ONNX Runtime + SQLite WAL | Local-first desktop image library fusing three ONNX-Runtime image encoders (CLIP ViT-B/32 + DINOv2-Base + SigLIP-2 Base 256) via Reciprocal Rank Fusion (Cormack 2009, k=60), with a single-flight indexing pipeline, dual-connection SQLite WAL persistence, filesystem watcher, opt-in profiling, and zero post-launch network calls. |
| [Aurix](aurix.md) | [Capataina/Aurix](https://github.com/Capataina/Aurix) | Rust + Tauri 2 + React 19 + SQLite | Local-first read-only DeFi analytics desktop built around a clean-room Uniswap V3 LP backtester — Q64.96 tick mathematics, SQLite WAL with writer-thread + r2d2 reader pool, 3-tier free-data ingest fallback, full cartesian strategy grid, multi-asset benchmark comparison, and an adaptive-tercile vol-regime classifier feeding verdict-prose synthesis. |
| [NeuroDrive](neurodrive.md) | [Capataina/NeuroDrive](https://github.com/Capataina/NeuroDrive) | Rust + Bevy ECS | Brain-inspired continual-learning system built deliberately without backprop or ML frameworks — Hebbian plasticity, STDP eligibility traces, dopamine-modulated weight updates, structural plasticity inside a deterministic 2D racing environment running on Bevy ECS. M6 brain-inspired substrate code-shipped; SideBySide validation run gating M7+. |
| [Nyquestro](nyquestro.md) | [Capataina/Nyquestro](https://github.com/Capataina/Nyquestro) | Rust + Ratatui + Tokio | From-scratch deterministic limit-order matching engine in safe Rust with HDR-histogram latency telemetry, Ornstein-Uhlenbeck synthetic simulator plus Coinbase L2 WebSocket bridge, and a six-pane Ratatui observability dashboard. Single-threaded MVP; lock-free and binary-protocol work explicitly deferred per D2 correctness-before-performance. |
| [Tessarix](tessarix.md) | [Capataina/Tessarix](https://github.com/Capataina/Tessarix) | Rust + Tauri 2 + React + TypeScript + MDX | Local-first desktop application that teaches abstract technical concepts (image-quality metrics, linear algebra) through narrative MDX lessons fused with embedded interactive React widgets, LLM-integrated assessments (wrong-answer threads, JSON-schema tiered hints, streaming chatbot), three complexity tiers, and a typed telemetry pipeline — built as a personal pedagogy substrate, not a commercial product. |
| [Open Source Contributions](open-source-contributions.md) | [Capataina/OpenSourceContributions](https://github.com/Capataina/OpenSourceContributions) (umbrella, private) | Rust ecosystem (burn, tinygrad, alloy + 6 queued upstreams) | Managed engagement track across 9 vetted Rust and ML-infrastructure upstreams running out of a private umbrella repo with durable per-repo culture/conventions notes and a local `scout-issues` skill — active PRs into burn (one APPROVED, one draft) and tinygrad (one open), interest comment on alloy. |

## Paused

| Project | Source repo | Primary stack | Brief |
|---------|-------------|---------------|-------|
| [Vynapse](vynapse.md) | [Capataina/Vynapse](https://github.com/Capataina/Vynapse) | Rust | From-scratch evolutionary-algorithm trainer for fixed-topology neural networks, built as a 4-crate workspace with a trait-based modular architecture and a 10-milestone roadmap whose later paradigms (NEAT, SGD, autodiff, static graph, hybrid) are scaffolded as empty stub files. |
| [Chrona](chrona.md) | [Capataina/Chrona](https://github.com/Capataina/Chrona) | C++20 | Personal-learning attempt to reimplement Git's "inner engine" (content-addressed object store, trees, commit DAG, refs, index, diff) from first principles, currently parked at the CMake + CLI + error-model + repo-discovery scaffold stage with the VCS core entirely unbuilt. |

## Dormant

| Project | Source repo | Primary stack | Brief |
|---------|-------------|---------------|-------|
| [AsteroidsAI](asteroidsai.md) | [Capataina/Asteroids-AI](https://github.com/Capataina/Asteroids-AI) | Python + Arcade + NEAT + DEAP + PyTorch + TensorFlow | Comparative-ML research playground pitting four optimisation paradigms (GA, diagonal CMA-ES, NEAT, GNN-backed SAC) against the same Asteroids-style continuous-control environment through a shared evaluator, encoder family, reward preset, and analytics schema. |
| [Consilium](consilium.md) | [Capataina/Consilium](https://github.com/Capataina/Consilium) | Python 3.11+ + LangChain + Textual | A CLI/TUI that runs the same question through a heterogeneous roster of LLMs across multiple debate rounds, compressing each round's output into a strict 8-key JSON state snapshot fed forward in place of raw peer text, with a final thesis-style synthesis written to a Markdown transcript. |
| [Xyntra](xyntra.md) | [Capataina/xyntra](https://github.com/Capataina/xyntra) | Rust | Aspires to become an automatic kernel-fusion compiler for deep-learning graphs (ONNX/TorchScript → fused GPU kernel via WGSL or CUDA PTX). Currently implements only the foundational IR layer: NodeID, TensorShape, OpKind, Node, Graph, four-category error taxonomy, partially-wired config, and a validator scaffold whose public methods panic via `todo!()`. ~3-5% of README scope built. |
| [Zyphos](zyphos.md) | [Capataina/Zyphos](https://github.com/Capataina/Zyphos) | Rust (std-only + chrono) | Network-programming learning laboratory implementing an HTTP server from raw TCP up, std-only constraint throughout — no hyper, no axum, no tokio, no mio. 30-milestone ladder across 7 phases; currently the first three (M1, M3, M5) implemented as a thread-per-connection HTTP/1.1 echo server with three GET routes and `panic::catch_unwind` isolation. |
| [Tectra](tectra.md) | [Capataina/Tectra](https://github.com/Capataina/Tectra) | C++20 + CMake | Self-directed exploration into the invisible infrastructure of a trading firm in modern C++. Implemented scope: a `Clock` interface with `RealClock` and `VirtualClock`, a four-value `LogSeverity` enum, a strict CMake C++20 build with `-Werror`, and a `main.cpp` exercising the two clocks. 14-milestone plan with only the first foundational primitives in code. |

---

## Notes

- Each per-project file has its own **Evidence Block** at the bottom listing every LifeOS source file consumed during synthesis, with verbatim last line for partial-read detection.
- Status reflects LifeOS source-of-truth, not inference. When a project's status changes (resume from pause, archive into dormant, etc.), update the LifeOS folder's `_Overview.md` frontmatter and re-run `populate-from-lifeos` — never edit `cernio/profile/` files by hand for status changes.
- Skills derived from these project files live in `../skills.md`. The derivation rubric (`populate-from-lifeos/references/skills-derivation-rubric.md`) anchors proficiency bands against project evidence, not exposure.
