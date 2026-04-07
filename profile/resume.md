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
  \item Implemented the ONNX \texttt{LSTM} operator in \texttt{tinygrad/nn/onnx.py}, adding support for forward, reverse, and bidirectional modes, optional inputs including initial hidden and cell states, and a shared \texttt{\_apply\_rnn\_activation} helper covering all RNN activation types required by the ONNX spec; submitted as \href{https://github.com/tinygrad/tinygrad/pull/15453}{PR \#15453}.
  \item Added comprehensive regression tests in \texttt{test/external/external\_test\_onnx\_ops.py} covering all LSTM configurations, verifying numerical correctness against ONNX Runtime across forward, reverse, bidirectional, and stateful variants.
\end{osscontrib}

%-----------------------------------------------------------------------
% PROJECTS
%-----------------------------------------------------------------------
\section{Projects}

\begin{project}{Image Browser: Local-First Pinterest-Style Image Manager}{https://github.com/Capataina/PinterestStyleImageBrowser}{Rust, Tauri, React, TypeScript, SQLite, ONNX}
  \item Built local-first desktop application for browsing large image collections with infinite scroll masonry layout, manual tagging, CLIP-powered visual similarity search, and natural language semantic search, architected across a Rust backend, Tauri IPC layer, React frontend, SQLite persistence, and ONNX Runtime inference pipeline running entirely offline.
  \item Designed as a production-grade privacy-first tool demonstrating end-to-end desktop product delivery, local ML inference integration, and full-stack systems architecture across frontend, backend, and database layers.
\end{project}

\begin{project}{Aurix: Local-First DeFi Analytics Platform}{https://github.com/Capataina/Aurix}{Rust, Tauri, React, TypeScript, SQLite}
  \item Building local-first Tauri desktop application for real-time Ethereum DeFi analytics, covering concurrent cross-DEX arbitrage detection, Uniswap V3 LP backtesting with tick mathematics, on-chain wallet tracking, gas pattern analysis, and token correlation risk modelling, all on-device via free public RPC endpoints with no paid services required.
  \item Designed as a full-stack DeFi intelligence platform demonstrating concurrent systems engineering, deep understanding of Ethereum AMM mechanics and market microstructure, quantitative risk modelling, and end-to-end product delivery across systems, financial, and web3 engineering domains.
\end{project}

\begin{project}{NeuroDrive: Brain-Inspired Continual Learning System}{https://github.com/Capataina/NeuroDrive}{Rust, Bevy}
  \item Building brain-inspired autonomous agent that learns to drive in a custom 2D racing environment using Hebbian plasticity, STDP eligibility traces, and dopamine-modulated weight updates from first principles, with no backpropagation or ML frameworks, architected around a sparse neural graph with plastic hidden topology and fixed sensorimotor interfaces.
  \item Designed as a research-grade platform for studying biologically plausible continual learning, grounded in a fully deterministic and instrumented simulation environment that validates task learnability before biological plasticity mechanisms are introduced.
\end{project}

\begin{project}{Nyquestro: High-Performance Order Matching Engine}{https://github.com/Capataina/Nyquestro}{Rust}
  \item Building a full-stack exchange simulation in safe Rust implementing a lock-free order book with price-time priority matching, a binary UDP wire protocol with fixed-width frames and checksum validation, a real-time risk layer covering fat-finger protection, per-session position bounds, and rolling VaR circuit breaking, and a market-making strategy agent that reconstructs the live book, computes order flow imbalance signals, and manages inventory-aware quote placement and adverse selection exposure.
  \item Designed as a systems and quantitative engineering research platform demonstrating low-latency concurrent Rust, market microstructure mechanics from both the exchange and participant perspective, and rigorous latency profiling across p50--p99.9 distributions with hardware performance counter instrumentation.
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
        \item Built CNN models for image classification and facial recognition, MLP for predictive analytics, and RNN for cryptographic attack simulation, demonstrating proficiency across supervised learning architectures.
        \item Implemented genetic algorithms for optimisation tasks using DEAP, exploring evolutionary approaches.
        \item Ran analyses Wireshark, Nmap, and tcpdump to identify DDOS attack patterns and proposed infrastructure upgrades.
        \item Led development of a simulation game in Java using libGDX as lead developer in an 8-person team, coordinating architecture, feature implementation, and automated cross-platform testing pipelines.
    \end{itemize}
  }

%-----------------------------------------------------------------------
% SKILLS
%-----------------------------------------------------------------------
\section{Skills}

\begin{itemize}
  \item \skillEntry{Languages}{Rust, Python, C++, TypeScript, JavaScript, Java}
  \item \skillEntry{Systems Programming}{Lock-Free Data Structures, Multithreading, Memory Safety, Low-Latency Optimisation}
  \item \skillEntry{AI/ML}{PyTorch, TensorFlow, ONNX Runtime, NEAT, DEAP, XGBoost, scikit-learn}
  \item \skillEntry{Desktop \& Full-Stack}{Tauri, React, SQLite, Node.js}
  \item \skillEntry{Blockchain \& DeFi}{Ethereum RPC, DeFi Protocol Mechanics, AMM Mathematics, Quantitative Risk Modelling}
  \item \skillEntry{Market Microstructure}{Order Book Mechanics, Market-Making, Binary Wire Protocols, Exchange Systems}
  \item \skillEntry{Mathematics}{Linear Algebra, Calculus, Probability, Optimisation Theory}
\end{itemize}

\end{document}