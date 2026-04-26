---
title: Cover Letter
tags:
  - profile
  - career
---

\documentclass[12pt,a4paper]{article}
\usepackage[margin=1.5cm]{geometry}
\usepackage{parskip}
\usepackage{helvet}
\renewcommand{\familydefault}{\sfdefault}
\usepackage{hyperref}

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

During university, I took on the role of lead engineer in an eight-person team developing a simulation game in Java using the LibGDX framework. I was responsible for the overall architecture, set up CI/CD pipelines with GitHub Actions, and implemented automated testing with JUnit across cross-platform test matrices. That experience taught me how to think about systems that need to work reliably for other people, not just on your own machine, and how to communicate clearly in a technical context where everyone has a different mental model of the codebase.

Since graduating, I have been developing a portfolio of self-directed systems projects that each explore a different domain in depth. \textbf{Cernio} is a job discovery engine built in async Rust that scans 408 companies across 7 ATS provider APIs, orchestrates parallel AI agents to evaluate 900+ jobs against a structured candidate profile, and presents everything in a 4-view interactive terminal dashboard built with Ratatui — with search, filtering, a pipeline kanban, responsive layout, and real-time SQLite-backed state. \textbf{NeuroDrive} is a from-scratch reinforcement learning system in Rust and Bevy: 393 KB across 67 source files implementing a deterministic multi-car racing simulation, a handwritten PPO agent with a 43-dimensional observation space, and a comprehensive analytics pipeline — all without any ML framework dependency. \textbf{Image Browser} is a desktop application that runs CLIP image and text encoders entirely offline via ONNX Runtime to power visual similarity and cross-modal semantic search across 50+ languages, with a pure-Rust WordPiece tokenizer and three cosine similarity modes. \textbf{Aurix} is a DeFi analytics platform targeting five analytical surfaces — cross-DEX arbitrage, Uniswap V3 LP backtesting, wallet tracking, gas prediction, and risk modelling — currently implementing the arbitrage scanner with raw JSON-RPC, hand-crafted ABI encoding, and Uniswap V3 Q96 fixed-point price decoding. \textbf{Nyquestro} is an order matching engine in safe Rust, working toward a lock-free order book with price-time priority, a binary UDP wire protocol, a real-time risk layer, and a market-making strategy agent — currently building the foundational type layer with fixed-point price arithmetic, zero-allocation event frames, and thorough integration tests.

These projects were not built to fill a CV. Each one started because I wanted to understand something deeply, and the only way I know how to do that is to build it. Working across systems programming, AI infrastructure, financial engineering, and ML tooling has given me a broad foundation, but more importantly, it has taught me how to move between domains.

Longer term, I am drawn to the intersection of AI and health. Strength training and nutrition science are central to how I live, and I find the engineering problems in that space genuinely compelling: building systems that can reason meaningfully over personal health data, surface useful insights from physiological signals, and do so in a way that respects privacy and works reliably over time. The kind of work being done in that space is the exact kind I want to grow into.

I am looking for a role where I can contribute to real projects, work alongside engineers who take correctness and systems design seriously, and keep building towards the direction I'm headed.

Thank you for your time,\\
Ata Caner Cetinkaya

\end{document}
