---
title: Resume
tags:
  - profile
  - career
last_verified: 2026-05-13
---

\documentclass[9pt,a4paper]{extarticle}

%-----------------------------------------------------------------------
% PACKAGES AND CUSTOMIZATION
%-----------------------------------------------------------------------
\usepackage[margin=0.25in]{geometry}
\usepackage{parskip}
\setlength{\parskip}{0.02em}
\usepackage[compact]{titlesec}
\usepackage{enumitem}
\usepackage[T1]{fontenc}
\usepackage{mathpazo}
\usepackage{textcomp}
\usepackage{xcolor}
\usepackage{ragged2e}
\usepackage{microtype}

% Global spacing adjustments
\setlist[itemize]{topsep=0.01em, itemsep=0.02em, parsep=0em, leftmargin=*, rightmargin=0pt, after=\vspace{0.05em}}

% Load hyperref last
\usepackage{hyperref}
\hypersetup{hidelinks}

% Section spacing
\titleformat{\section}{\color{black}\bfseries\Large}{}{0em}{}[\titlerule]
\titlespacing{\section}{0pt}{0.02em}{0.1em}

%-----------------------------------------------------------------------
% CUSTOM COMMANDS / ENVIRONMENTS
%-----------------------------------------------------------------------

% Education Entry
\newcommand{\educationEntry}[5]{%
  \noindent{\fontsize{12}{12}\selectfont\textbf{#1}} \hfill \textit{#2}\par
  \noindent\textit{#3} \hfill \textit{#4}\par
  \if\relax\detokenize{#5}\relax
  \else
    #5
  \fi
}

% Project Environment — project name itself links to repo
\newenvironment{project}[3]{%
  \noindent
  {\fontsize{11}{11}\selectfont\textbf{\href{#2}{#1}}} \hfill \textit{#3}\par
  \vspace{0.05em}
  \begin{itemize}
}{%
  \end{itemize}
}

% Open Source Contribution Environment — name itself links to repo
\newenvironment{osscontrib}[3]{%
  \noindent
  {\fontsize{11}{11}\selectfont\textbf{\href{#2}{#1}}} \hfill \textit{#3}\par
  \vspace{0.05em}
  \begin{itemize}
}{%
  \end{itemize}
}

% Skill Entry
\newcommand{\skillEntry}[2]{%
  \textbf{#1:} #2 \par
}

\begin{document}
\pagenumbering{gobble}

%-----------------------------------------------------------------------
% NAME AND CONTACT INFORMATION
%-----------------------------------------------------------------------
\begin{center}
    {\LARGE \textbf{Ata Caner Cetinkaya}}\\[0.4em]
    \normalsize
    London, UK \textbf{\,|\,} +44 7391 904514 \textbf{\,|\,} \href{mailto:atacanercetinkaya@gmail.com}{atacanercetinkaya@gmail.com}\\[0.15em]
    \href{https://www.linkedin.com/in/atacanercetinkaya}{linkedin.com/in/atacanercetinkaya} \textbf{\,|\,} \href{https://github.com/Capataina}{github.com/Capataina}
\end{center}

%-----------------------------------------------------------------------
% OPEN SOURCE CONTRIBUTIONS
%-----------------------------------------------------------------------
\section{Open Source Contributions}

\begin{osscontrib}{burn, Rust Deep Learning Framework}{https://github.com/tracel-ai/burn}{Rust}
  \item Merged the A-FINE full-reference image-quality metric (\href{https://github.com/tracel-ai/burn/pull/4894}{PR \#4894}, \href{https://github.com/tracel-ai/burn/issues/4312}{issue \#4312}, arXiv 2503.11221) into burn-train as my first upstream tracel-ai/burn merge; 1,864 additions across 10 files, inlined CLIP ViT-B/32 backbone with custom fused-QKV attention, PyTorch-weight loader, and parity tests against the PyIQA reference.
  \item Scoping the fold4d / col2im ONNX operator (\href{https://github.com/tracel-ai/burn/issues/4519}{issue \#4519}) as the inverse of unfold4d; default-impl trade-off (scatter-add vs identity-weight \texttt{conv\_transpose2d}) surfaced with maintainer.
\end{osscontrib}

\begin{osscontrib}{tinygrad, Deep Learning Framework}{https://github.com/tinygrad/tinygrad}{Python}
  \item Implementing the ONNX LSTM operator (\href{https://github.com/tinygrad/tinygrad/pull/16119}{PR \#16119}) for Silero VAD parity at +21 tokenised lines; forward direction with default activations, explicit \texttt{NotImplementedError} raises for seven unsupported inputs, two LSTM tests enabled in the ONNX suite. Re-attempt of \href{https://github.com/tinygrad/tinygrad/pull/15453}{PR \#15453} (closed on tinygrad's line-budget policy) at the narrower scope the maintainer's review surfaced.
\end{osscontrib}

\begin{osscontrib}{alloy, Modern Rust Ethereum Library}{https://github.com/alloy-rs/alloy}{Rust}
  \item Picked up the JSON-RPC \texttt{Response} deserialisation recursion-limit fix in \texttt{alloy-json-rpc} (\href{https://github.com/alloy-rs/alloy/issues/1156}{issue \#1156}); roughly a 30-LOC patch routing the user-side \texttt{unbounded\_depth} feature through to the buried deserialiser.
\end{osscontrib}

%-----------------------------------------------------------------------
% PROJECTS
%-----------------------------------------------------------------------
\section{Projects}

\begin{project}{Cernio: Local-First Job Discovery and Curation Engine}{https://github.com/Capataina/cernio}{Rust, Tokio, Ratatui, SQLite}
  \item Built a local-first job-discovery and curation engine that scans hundreds of companies across the major ATS providers, deduplicates against SQLite, and orchestrates AI agents grading every opportunity against a structured candidate profile on multiple fit dimensions through 9 native Claude Code skills.
  \item Built an interactive Ratatui terminal dashboard with vim-style search, grade-based sorting, pipeline kanban, multi-select bulk operations, markdown export, responsive layout, and real-time database refresh.
\end{project}

\begin{project}{NeuroDrive: Biology-First RL Driving Simulator}{https://github.com/Capataina/NeuroDrive}{Rust, Bevy}
  \item Built a reinforcement-learning environment in Rust + Bevy with no external ML framework, deterministic 60 Hz multi-car simulation, custom observation space, and a handwritten PPO with clipped surrogate, asymmetric actor-critic, and AdamW. Cars learn end-to-end; fixed a 43x regression by flattening nested-Vec weights to row-major.
  \item Shipped a biologically-inspired learner running alongside PPO: a sparse graph network with three-factor plasticity, eligibility traces, reward neuromodulation, homeostatic regulation, and continual-backprop structural growth, a single agent that learns continuously across its entire lifetime, with no weight resets and no backpropagation.
\end{project}

\begin{project}{Image Browser: Multi-Encoder Local-First Image Manager}{https://github.com/Capataina/PinterestStyleImageBrowser}{Rust, Tauri, React, ONNX}
  \item Built a desktop app for browsing, tagging, and semantically searching personal image libraries offline. Three image-embedding models (CLIP, DINOv2, SigLIP-2) run via ONNX Runtime; per-encoder rankings are combined with Reciprocal Rank Fusion to surface conceptual, structural, and descriptive similarity.
  \item Built in Tauri 2 + React 19: Pinterest-style masonry, multi-folder library with filesystem watcher and orphan detection, AND/OR tag filtering, per-image annotations, typed IPC error envelopes, and SQLite in WAL mode with separate read/write connections so the UI stays responsive during indexing.
\end{project}

\begin{project}{Tessarix: Local-First Interactive Learning Substrate}{https://github.com/Capataina/Tessarix}{Tauri 2, React 19, MDX, Rust}
  \item Building a local-first desktop substrate that teaches abstract technical concepts (image quality, linear algebra, ML) through narrative MDX lessons fused with embedded interactive widgets and LLM-integrated assessment surfaces.
  \item Shipped the M1 substrate, 9 MDX lessons (first being A-FINE, cross-referencing burn PR \#4894), 53 reusable widgets, and three LLM-integrated assessment features; three-pillar architecture (Teach, Quiz, Interview) over shared content and question bank.
\end{project}

\begin{project}{Aurix: Local-First DeFi Analytics Platform}{https://github.com/Capataina/Aurix}{Rust, Tauri, React, TypeScript}
  \item Building Aurix, an on-device Ethereum analytics platform on Tauri 2 + React 19 + Rust spanning cross-DEX arbitrage, V3 LP backtesting, wallet tracking, gas prediction, and risk modelling; the arbitrage scanner reads raw JSON-RPC with hand-crafted ABI encoding (no ethers-rs), decoding Uniswap V3 sqrtPriceX96 and V2 reserve ratios via BigUint.
  \item Shipped the V3 LP backtester with Q64.96 fixed-point math, multi-asset benchmarking against DeFi-native and TradFi baselines, and regime-conditional capital allocation; built a React dashboard with hand-rolled SVG charting and a TypeScript insight engine computing rolling statistics, trailing run detection, and severity-graded notifications.
\end{project}

\begin{project}{Nyquestro: High-Performance Order Matching Engine}{https://github.com/Capataina/Nyquestro}{Rust}
  \item Building a from-scratch exchange simulation in safe Rust targeting a lock-free order book with price-time priority matching, a binary UDP wire protocol, a real-time risk layer (fat-finger protection, rolling VaR circuit breaking), and a market-making agent with order-flow-imbalance signals and inventory-aware quote placement.
  \item Shipped the matching engine MVP with multi-instrument routing, a Ratatui observability dashboard rendering live latency and fill-rate infographics, and a Coinbase Advanced Trade WebSocket bridge feeding real BTC-USD / ETH-USD / SOL-USD market depth.
\end{project}

%-----------------------------------------------------------------------
% EDUCATION
%-----------------------------------------------------------------------
\section{Education}

\educationEntry
  {University of York}
  {York, UK}
  {Bachelor of Engineering (BEng) in Computer Science}
  {September 2022 -- June 2025}
  {
    \begin{itemize}
        \item Dissertation \textit{Neuronika}: React + TypeScript SPA with Mistral Small 3.1 24B driving adaptive multi-layered tag generation across three retrieval surfaces; 4-participant pilot vs Obsidian showed 355\% average information-retrieval speedup.
        \item Lead developer on an 8-person Java/libGDX simulation team: owned architecture, features, and CI testing.
        \item CNN, MLP, and RNN coursework models across image classification, predictive analytics, and cryptographic attack simulation.
    \end{itemize}
  }

%-----------------------------------------------------------------------
% SKILLS
%-----------------------------------------------------------------------
\section{Skills}

\begin{itemize}
  \item \skillEntry{Languages}{Rust, Python, C++, TypeScript, JavaScript, Java}
  \item \skillEntry{Systems}{Lock-Free Data Structures, Multithreading, Memory Safety, Low-Latency Optimisation}
  \item \skillEntry{AI/ML}{PyTorch, TensorFlow, ONNX Runtime, NEAT, DEAP, XGBoost, scikit-learn}
  \item \skillEntry{Desktop \& Full-Stack}{Tauri, React, SQLite, Node.js}
  \item \skillEntry{Finance}{Order Book Mechanics, Market-Making, Ethereum RPC, AMM Mathematics, Quantitative Risk Modelling}
  \item \skillEntry{Mathematics}{Linear Algebra, Calculus, Probability, Optimisation Theory}
\end{itemize}

\end{document}
