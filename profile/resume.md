\documentclass[9.5pt,a4paper]{article}

%-----------------------------------------------------------------------
% PACKAGES AND CUSTOMIZATION
%-----------------------------------------------------------------------
\usepackage[margin=0.3in]{geometry}
\usepackage{parskip}
\setlength{\parskip}{0.15em}
\usepackage[compact]{titlesec}
\usepackage{enumitem}
\usepackage{newpxtext}
\usepackage{newpxmath}
\usepackage[T1]{fontenc}
\usepackage{textcomp}
\usepackage{fontawesome5}
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

% Project Environment
\newenvironment{project}[3]{%
  \noindent
  {\fontsize{12}{12}\selectfont\textbf{#1}} \href{#2}{[ \faGithub\ ]} \hfill \textit{#3}\par
  \vspace{0.1em}
  \begin{itemize}
}{%
  \end{itemize}
}

% Open Source Contribution Environment
\newenvironment{osscontrib}[3]{%
  \noindent
  {\fontsize{12}{12}\selectfont\textbf{#1}} \href{#2}{[ \faCodeBranch\ ]} \hfill \textit{#3}\par
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
    Address: London, UK \\[0.15em]
    \begin{tabular}{r c l}
    Phone: +44 7391 904514 & \textbf{|} & Email: \href{mailto:atacanercetinkaya@gmail.com}{atacanercetinkaya@gmail.com}\\
    LinkedIn: \href{https://www.linkedin.com/in/atacanercetinkaya}{linkedin.com/in/atacanercetinkaya} & \textbf{|} &
    GitHub: \href{https://github.com/Capataina}{github.com/Capataina}
    \end{tabular}
\end{center}

%-----------------------------------------------------------------------
% OPEN SOURCE CONTRIBUTIONS
%-----------------------------------------------------------------------
\section{Open Source Contributions}

\begin{osscontrib}{tinygrad — Deep Learning Framework}{https://github.com/tinygrad/tinygrad}{Python}
  \item Implemented the ONNX \texttt{LSTM} operator supporting forward, reverse, and bidirectional modes with optional initial states, a shared activation helper covering all RNN activation types per the ONNX spec, and a full regression test suite verified against ONNX Runtime; submitted as \href{https://github.com/tinygrad/tinygrad/pull/15453}{PR \#15453}.
\end{osscontrib}

%-----------------------------------------------------------------------
% PROJECTS
%-----------------------------------------------------------------------
\section{Projects}

\begin{project}{Cernio: AI-Powered Job Discovery Engine}{https://github.com/Capataina/cernio}{Rust, Tokio, Ratatui, SQLite}
  \item Built async Rust pipeline scanning 408 companies across 7 ATS provider APIs, applying configurable filter chains (location, exclusion, inclusion), deduplicating against SQLite in WAL mode, and orchestrating parallel AI agents to evaluate 900+ jobs with multi-dimensional fit assessments grounded in a structured candidate profile.
  \item Built 4-view interactive Ratatui terminal dashboard with vim-style search, grade-based sorting, pipeline kanban, multi-select bulk operations, markdown export, responsive layout across three breakpoints, and real-time auto-refresh from SQLite.
\end{project}

\begin{project}{NeuroDrive: RL Training Environment \& Agent}{https://github.com/Capataina/NeuroDrive}{Rust, Bevy}
  \item Built complete RL training environment from scratch across 8 Bevy subsystems (393 KB, 67 source files) with no ML framework dependencies: deterministic 60 Hz multi-car simulation, 43-dimensional observation space, and handwritten PPO with clipped surrogate objective, asymmetric actor-critic, orthogonal init, and AdamW optimiser.
  \item Built analytics pipeline capturing 16 tick-level fields and 25 episode-level aggregates with crash classification and automated diagnostic reports. Feature-gated profiling across all 17 FixedUpdate systems. Fixed a 43x regression by switching from nested Vec to flat contiguous row-major weight storage.
\end{project}

\begin{project}{Image Browser: Local-First Semantic Image Manager}{https://github.com/Capataina/PinterestStyleImageBrowser}{Rust, Tauri, React, ONNX}
  \item Built CLIP-powered image search running entirely offline: images encoded to 512-dimensional vectors via ONNX Runtime (CUDA fallback), and a pure-Rust WordPiece tokenizer encoding text queries through a multilingual CLIP model into the same vector space for cross-modal semantic search. Three cosine similarity modes: diversity-sampled, strict-ranked, and Pinterest-style tiered.
  \item Built Tauri 2 desktop app with Pinterest-style masonry layout, manual tagging, batch CLIP embedding on startup, thumbnail pipeline with on-disk caching, and SQLite persistence for metadata, tags, and embeddings.
\end{project}

\begin{project}{Aurix: Local-First DeFi Analytics Platform}{https://github.com/Capataina/Aurix}{Rust, Tauri, React, TypeScript}
  \item Building on-device Ethereum analytics platform targeting cross-DEX arbitrage, Uniswap V3 LP backtesting, wallet tracking, gas prediction, and risk modelling. Currently implements the arbitrage scanner using raw JSON-RPC with hand-crafted ABI encoding (no ethers-rs), decoding Uniswap V3 sqrtPriceX96 via BigUint arithmetic and V2 reserve ratios across 4 venues concurrently.
  \item Built React dashboard with hand-rolled SVG charting (4 analytical modes, per-venue colour coding) and a TypeScript insight engine computing rolling statistics, trailing run detection, and severity-graded notifications.
\end{project}

\begin{project}{Nyquestro: High-Performance Order Matching Engine}{https://github.com/Capataina/Nyquestro}{Rust}
  \item Building a from-scratch exchange simulation in safe Rust targeting a lock-free order book with price-time priority matching, a binary UDP wire protocol, a real-time risk layer (fat-finger protection, rolling VaR circuit breaking), and a market-making strategy agent with order flow imbalance signals and inventory-aware quote placement.
  \item Currently implementing the foundational type layer: fixed-point price representation in cents, nanosecond timestamps, order state machine with saturating fill logic, zero-allocation Copy event frames, structured error taxonomy, and \textasciitilde50 integration tests.
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
        \item Built CNN, MLP, and RNN models for image classification, predictive analytics, and cryptographic attack simulation, demonstrating proficiency across supervised learning architectures.
        \item Led development of a simulation game in Java/libGDX as lead developer in an 8-person team, coordinating architecture, feature implementation, and automated cross-platform testing pipelines.
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
