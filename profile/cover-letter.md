---
title: Cover Letter
tags:
  - profile
  - career
---

\documentclass[9pt,a4paper]{extarticle}
\usepackage[top=0.7cm,bottom=0.7cm,left=1.2cm,right=1.2cm]{geometry}
\usepackage{parskip}
\setlength{\parskip}{0.25em}
\usepackage{enumitem}
\setlist[itemize]{topsep=0pt, itemsep=0.1em, parsep=0pt, leftmargin=1.1em}
\usepackage[T1]{fontenc}
\usepackage{mathpazo}
\usepackage{hyperref}
\hypersetup{hidelinks}

\begin{document}
\pagestyle{empty}

\begin{flushright}
Ata Caner Cetinkaya \\
atacanercetinkaya@gmail.com \\
\href{https://www.linkedin.com/in/atacanercetinkaya}{linkedin.com/in/atacanercetinkaya} \textbar\
\href{https://github.com/Capataina}{github.com/Capataina} \\
\end{flushright}

Hello,

I am a recent Computer Science graduate based in London with a strong interest in systems engineering, AI infrastructure, and performance-critical software. I enjoy working at the layer where design decisions have real consequences: where the wrong abstraction costs you latency, the wrong data structure breaks under concurrency, or the wrong architecture makes the whole thing impossible to reason about. That is the kind of engineering I find genuinely interesting, and it is what I have been building towards.

During university, I took on the role of lead engineer in an eight-person team developing a simulation game in Java using the LibGDX framework. I was responsible for the overall architecture, set up CI/CD pipelines with GitHub Actions, and implemented automated testing with JUnit across cross-platform test matrices. That experience taught me how to think about systems that need to work reliably for other people, not just on your own machine, and how to communicate clearly in a technical context where everyone has a different mental model of the codebase. My final-year dissertation was \textbf{Neuronika}, an AI-driven personal knowledge management system: Mistral Small 3.1 24B drives adaptive multi-layered tag generation feeding three retrieval surfaces (dynamic folder hierarchy, vis.js interactive graph, Fuse.js semantic search). A 4-participant controlled pilot against Obsidian showed a 355\% average information-retrieval speedup across find-by-title, find-by-context, and group-retrieval tasks; the underlying PKM/IR thesis is what LifeOS now operationalises with a substantially richer intelligence layer.

Since graduating, I have been developing a portfolio of self-directed systems projects that each explore a different domain in depth.

\begin{itemize}
\item \textbf{Cernio} treats job search as a systems problem: an async-Rust engine scanning 456 companies across 6 ATS providers (plus a bespoke Eightfold integration), orchestrating parallel AI agents to evaluate 1,184+ jobs against a structured candidate profile through 9 native Claude Code skills, and surfacing everything in a 5-view interactive Ratatui dashboard with vim-style search, grade-based sorting, a pipeline kanban, responsive layout, and real-time SQLite-backed state.
\item \textbf{NeuroDrive} pairs a from-scratch reinforcement-learning racing simulator in Rust and Bevy with a biology-first learner that runs continuously across its lifetime: a deterministic multi-car simulation drives a handwritten PPO baseline (43-dim observation, custom AdamW, asymmetric actor-critic), and alongside it a sparse-graph network with three-factor plasticity, eligibility traces, reward neuromodulation, homeostasis, and continual-backprop structural growth, no weight resets and no backpropagation in the brain learner.
\item \textbf{Image Browser} is a desktop application for browsing, tagging, and semantically searching personal image libraries entirely offline. Three image-embedding models (CLIP, DINOv2, SigLIP-2) run via ONNX Runtime; per-encoder rankings are combined with Reciprocal Rank Fusion (Cormack 2009) to surface conceptual, structural, and descriptive similarity at once across multi-folder libraries with a filesystem watcher and live-progress async indexing.
\item \textbf{Aurix} is a DeFi analytics platform targeting five analytical surfaces: cross-DEX arbitrage, Uniswap V3 LP backtesting, wallet tracking, gas prediction, and risk modelling. The Vector A V3 LP backtester is shipped, with Q64.96 fixed-point math, multi-asset benchmarking against DeFi-native and TradFi baselines, and regime-conditional capital allocation; the cross-DEX arbitrage scanner uses raw JSON-RPC with hand-crafted ABI encoding (no ethers-rs).
\item \textbf{Nyquestro} is an order matching engine in safe Rust, working toward a lock-free order book with price-time priority, a binary UDP wire protocol, a real-time risk layer, and a market-making strategy agent. The matching engine MVP shipped with multi-instrument routing, a Ratatui observability dashboard rendering live latency and fill-rate infographics, and a Coinbase Advanced Trade WebSocket bridge feeding real BTC-USD / ETH-USD / SOL-USD market depth, on top of fixed-point cents pricing, nanosecond timestamps, and zero-allocation Copy event frames.
\item \textbf{Tessarix} is a local-first interactive learning substrate that teaches abstract technical concepts through narrative MDX lessons fused with embedded interactive widgets, replacing prose-only explanation with manipulable visualisations of the dimensions a wall of text strips out. The M1 substrate plus 9 MDX lessons (the first being A-FINE, cross-referencing the burn PR \#4894 I merged the same week), 53 reusable widgets, and three LLM-integrated assessment features (wrong-answer threads, tiered hints, right-pane chatbot) shipped across 48 hours via parallel-agent execution; three-pillar architecture (Teach, Quiz, Interview) over shared content and question bank, Tauri 2 + React 19 + MDX with a Rust backend.
\end{itemize}

Beyond my own work, I am also contributing to the open-source Rust ML and AI-infrastructure ecosystem.

\begin{itemize}
\item \textbf{tracel-ai/burn} (Rust deep learning framework): A-FINE full-reference image-quality metric merged 2026-05-11 (\href{https://github.com/tracel-ai/burn/pull/4894}{PR \#4894}, 1,864 additions across 10 files, inlined CLIP ViT-B/32 backbone with custom fused-QKV attention and PyTorch-weight loader with five non-obvious load-correctness fixes), my first upstream merge to tracel-ai/burn; in parallel, scoping the fold4d / col2im ONNX operator (\href{https://github.com/tracel-ai/burn/issues/4519}{issue \#4519}) as the inverse of unfold4d.
\item \textbf{tinygrad/tinygrad}: ONNX LSTM operator (\href{https://github.com/tinygrad/tinygrad/pull/16119}{PR \#16119}) in review at +21 tokenised lines for Silero VAD parity; forward direction with default activations and explicit \texttt{NotImplementedError} raises for seven unsupported inputs. Re-attempt of \href{https://github.com/tinygrad/tinygrad/pull/15453}{PR \#15453} (closed on tinygrad's line-budget policy) at the narrower scope the maintainer's review surfaced.
\item \textbf{alloy-rs/alloy} (modern Rust Ethereum library): picked up the JSON-RPC \texttt{Response} deserialisation recursion-limit fix in \texttt{alloy-json-rpc}.
\end{itemize}

These projects were not built to fill a CV. Each one started because I wanted to understand something deeply, and the only way I know how to do that is to build it. Working across systems programming, AI infrastructure, financial engineering, and ML tooling has given me a broad foundation, but more importantly, it has taught me how to move between domains.

Longer term, I am drawn to the intersection of AI and health. Strength training and nutrition science are central to how I live, and I find the engineering problems in that space genuinely compelling: building systems that can reason meaningfully over personal health data, surface useful insights from physiological signals, and do so in a way that respects privacy and works reliably over time. The kind of work being done in that space is the exact kind I want to grow into.

I am looking for a role where I can contribute to real projects, work alongside engineers who take correctness and systems design seriously, and keep building towards the direction I'm headed.

Thank you for your time,\\
Ata Caner Cetinkaya

\end{document}
