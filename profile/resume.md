---
title: Resume
tags:
  - profile
  - career
last_verified: 2026-04-28
---

\documentclass[9.5pt,a4paper]{article}

%-----------------------------------------------------------------------
% PACKAGES AND CUSTOMIZATION
%-----------------------------------------------------------------------
\usepackage[margin=0.3in]{geometry}
\usepackage{parskip}
\setlength{\parskip}{0.05em}
\usepackage[compact]{titlesec}
\usepackage{enumitem}
\usepackage[T1]{fontenc}
\usepackage{mathpazo}
\usepackage{textcomp}
\usepackage{xcolor}
\usepackage{ragged2e}
\usepackage{microtype}

% Global spacing adjustments
\setlist[itemize]{topsep=0.01em, itemsep=0.03em, parsep=0em, leftmargin=*, rightmargin=0pt, after=\vspace{0.1em}}

% Load hyperref last
\usepackage{hyperref}
\hypersetup{hidelinks}

% Section spacing
\titleformat{\section}{\color{black}\bfseries\LARGE}{}{0em}{}[\titlerule]
\titlespacing{\section}{0pt}{0.05em}{0.2em}

%-----------------------------------------------------------------------
% CUSTOM COMMANDS / ENVIRONMENTS
%-----------------------------------------------------------------------

% Education Entry
\newcommand{\educationEntry}[5]{%
  \noindent{\fontsize{13}{13}\selectfont\textbf{#1}} \hfill \textit{#2}\par
  \noindent\textit{#3} \hfill \textit{#4}\par
  \if\relax\detokenize{#5}\relax
  \else
    #5
  \fi
}

% Project Environment — project name itself links to repo
\newenvironment{project}[3]{%
  \noindent
  {\fontsize{12}{12}\selectfont\textbf{\href{#2}{#1}}} \hfill \textit{#3}\par
  \vspace{0.1em}
  \begin{itemize}
}{%
  \end{itemize}
}

% Open Source Contribution Environment — name itself links to repo
\newenvironment{osscontrib}[3]{%
  \noindent
  {\fontsize{12}{12}\selectfont\textbf{\href{#2}{#1}}} \hfill \textit{#3}\par
  \vspace{0.1em}
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

\begin{osscontrib}{tinygrad, Deep Learning Framework}{https://github.com/tinygrad/tinygrad}{Python}
  \item Implemented the ONNX LSTM operator supporting forward, reverse, and bidirectional modes with optional initial states, a shared activation helper covering all RNN activation types per the ONNX spec, and a full regression suite verified against ONNX Runtime; submitted as \href{https://github.com/tinygrad/tinygrad/pull/15453}{PR \#15453}.
\end{osscontrib}

\begin{osscontrib}{burn, Rust Deep Learning Framework}{https://github.com/tracel-ai/burn}{Rust}
  \item Claimed and currently implementing the A-FINE no-reference image-quality metric (\href{https://github.com/tracel-ai/burn/issues/4312}{issue \#4312}, arXiv 2503.11221) into burn-train with an inlined CLIP ViT backbone, five evaluator heads, PyTorch-weight loader, and reference-output regression tests, following maintainer-confirmed precedent.
\end{osscontrib}

%-----------------------------------------------------------------------
% PROJECTS
%-----------------------------------------------------------------------
\section{Projects}

\begin{project}{Cernio: Local-First Job Discovery and Curation Engine}{https://github.com/Capataina/cernio}{Rust, Tokio, Ratatui, SQLite}
  \item Built a local-first job-discovery and curation engine that treats job search as a systems problem, scanning hundreds of companies across the major ATS providers, deduplicating against SQLite, and orchestrating AI agents grading every opportunity against a structured candidate profile on multiple fit dimensions through 9 native Claude Code skills installed at .claude/skills/.
  \item Built an interactive Ratatui terminal dashboard with vim-style search, grade-based sorting, pipeline kanban, multi-select bulk operations, markdown export, responsive layout, and real-time database refresh.
\end{project}

\begin{project}{NeuroDrive: Biology-First RL Driving Simulator}{https://github.com/Capataina/NeuroDrive}{Rust, Bevy}
  \item Built a reinforcement-learning environment in Rust + Bevy with no external ML framework, deterministic 60 Hz multi-car simulation, custom observation space, and a handwritten PPO with clipped surrogate, asymmetric actor-critic, and AdamW. Cars learn end-to-end; fixed a 43x regression by flattening nested-Vec weights to row-major.
  \item Shipped a biologically-inspired learner running alongside PPO: a sparse graph network with three-factor plasticity, eligibility traces, reward neuromodulation, homeostatic regulation, and continual-backprop structural growth, a single agent that learns continuously across its entire lifetime, with no weight resets and no backpropagation.
\end{project}

\begin{project}{Image Browser: Multi-Encoder Local-First Image Manager}{https://github.com/Capataina/PinterestStyleImageBrowser}{Rust, Tauri, React, ONNX}
  \item Built a desktop app for browsing, tagging, and semantically searching personal image libraries entirely offline. Three image-embedding models (CLIP, DINOv2, SigLIP-2) run via ONNX Runtime; per-encoder rankings are combined with Reciprocal Rank Fusion (Cormack 2009) to surface conceptual, structural, and descriptive similarity at once.
  \item Built in Tauri 2 + React 19: Pinterest-style masonry, multi-folder library with filesystem watcher and orphan detection, AND/OR tag filtering, per-image annotations, typed IPC error envelopes, and SQLite in WAL mode with separate read/write connections so the UI stays responsive during indexing.
\end{project}

\begin{project}{Aurix: Local-First DeFi Analytics Platform}{https://github.com/Capataina/Aurix}{Rust, Tauri, React, TypeScript}
  \item Building an on-device Ethereum analytics app targeting cross-DEX arbitrage, Uniswap V3 LP backtesting, wallet tracking, gas prediction, and risk modelling. The arbitrage scanner uses raw JSON-RPC with hand-crafted ABI encoding (no ethers-rs), decoding Uniswap V3 sqrtPriceX96 and V2 reserve ratios via BigUint across multiple DEXs.
  \item Built a React dashboard with hand-rolled SVG charting (per-venue colour coding, multiple analytical modes) and a TypeScript insight engine computing rolling statistics, trailing run detection, and severity-graded notifications.
\end{project}

\begin{project}{Nyquestro: High-Performance Order Matching Engine}{https://github.com/Capataina/Nyquestro}{Rust}
  \item Building a from-scratch exchange simulation in safe Rust targeting a lock-free order book with price-time priority matching, a binary UDP wire protocol, a real-time risk layer (fat-finger protection, rolling VaR circuit breaking), and a market-making agent with order-flow-imbalance signals and inventory-aware quote placement.
  \item Currently implementing the foundational type layer: fixed-point cents pricing, nanosecond timestamps, an order state machine with saturating fill logic, zero-allocation Copy event frames, and a structured error taxonomy.
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
        \item Built CNN, MLP, and RNN models for image classification, predictive analytics, and cryptographic attack simulation.
        \item Led an 8-person team developing a Java/libGDX simulation game; owned architecture, features, and CI testing.
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
