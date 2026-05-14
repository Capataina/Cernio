---
title: Education
tags:
  - profile
  - education
last_verified: 2026-05-09
---

# Education

## Degree 1

- **Institution**: University of York
- **Degree type**: BEng
- **Subject / Major**: Computer Science
- **Minor / Specialisation**: None
- **Start date**: September 2022
- **End date**: July 2025
- **Grade / Classification**: 2:2
- **Dissertation / Thesis topic**: Personal knowledge management and information retrieval systems — literature review identifying gaps in current PKM/IR approaches, with Neuronika built as the practical project that addresses those gaps. See Final-year dissertation + project section below.

## Modules (10 total across Year 1, Year 2, Year 3 + Masters-level options)

### Year 1 (Certificate level)

- **Software 1: Foundations of Programming for Computer Science** (`COM00015C`, 20 credits) — Python procedural programming foundations: data structures, debugging, unit testing, file handling (CSV/JSON), professional software engineering practice. Single online exam.
- **Software 2: Object-Oriented Data Structures & Algorithms** (`COM00016C`, 20 credits) — Java OO data structures (stacks, queues, trees, graphs); algorithm complexity analysis with formal notation and inductive proof; greedy algorithms and dynamic programming; test-driven development. Two online exams (50% / 50%).
- **Human-Computer Interaction 1: Introduction to User-Centred Design** (`COM00018C`, 20 credits) — User-centred design methodologies, psychological principles in interactive systems, group prototype + user testing exercise, ethical considerations for human-participant studies. 70% groupwork + 30% exam.

### Year 2 (Intermediate level)

- **DATA: Introduction to Data Science** (`COM00028I`, S1, 20 credits) — Data collection, cleaning, manipulation; statistical analysis (correlation, inferential statistics, regression); relational databases + SQL + NoSQL paradigms; Python statistical testing; data visualisation; data ethics. 100% coursework.
- **Engineering 1: Software & Systems Engineering** (`COM00019I`, S2, 20 credits) — Requirements elicitation, software architecture, OO design, testing methodologies, CI/CD, risk management, team collaboration, third-party integration, cybersecurity. **8-person Java/libGDX simulation game team project** (lead developer; ownership of architecture, features, CI testing). 35% groupwork + 65% major group project.
- **Intelligent Systems: Machine Learning & Optimisation** (`COM00026I`, S2, 20 credits) — Linear regression, neural networks, linear algebra, continuous optimisation; gradient descent, backpropagation, overfitting and regularisation. Python + PyTorch. 30% essay/coursework + 70% online exam.

### Year 3 (Honours level)

- **Cryptography** (`MAT00080H`, 20 credits) — Classical ciphers; AES block cipher; public-key cryptography (RSA, Diffie-Hellman); BB84 quantum key distribution. Closed exam. (Maths-school module taken as a Computer Science option.)
- **Network Security** (`COM00056H`, 20 credits) — Threat modelling, security policies, multi-layer network attacks and defences, CIA principles (confidentiality / integrity / availability), security analysis, policy implementation. 100% coursework.

### Masters-level options (taken in Year 3)

- **Evolutionary & Adaptive Computing** (`COM00177M`, 20 credits) — Biologically-inspired computing: evolutionary algorithms in Python via DEAP, agent-based modelling with evolutionary adaptation, multi-agent system performance analysis. **Genetic algorithms applied to beer quality prediction and evolutionary regression.** 100% coursework.
- **Intelligent Systems: Probabilistic & Deep Learning** (`COM00184M`, 20 credits) — Optimisation, Bayesian methods, kernel techniques, gradient descent variants, deep learning architectures, decision trees, neural networks; large-scale data handling. Python + PyTorch / TensorFlow. 100% coursework.

### Final-year dissertation + project

The final-year work had two arms — a written dissertation and a practical project that demonstrated the dissertation's thesis.

**Dissertation** — *Neuronika: AI-Driven Intelligent Note Taking and Organisation System* (Version 1.1.3, 25 April 2025). Supervisor: Dr Tommy Yuan. Substantial written work on personal knowledge management and information retrieval systems. Reviewed the existing PKM/IR literature (Jones 2007, Barreau 1995, Whitham 2017, Bergman 2014, Civan 2008, Golder 2006, Diao 2009, Jarrahi 2022, Sweller 1988 cognitive load), surveyed nine existing PKM tools across folder-based / tag-based / block-based / graph-based / hybrid paradigms, ran a structured user survey on note-taking habits and organisation preferences, and argued that AI-driven adaptive tag generation can serve as an organisational backbone bridging the rigidity of folder hierarchies and the labour cost of manual tagging.

**Practical project — Neuronika** — AI-driven note-taking and PKM application built as the dissertation's practical answer. React + TypeScript + Vite single-page app, fully client-side (no backend, no persistent storage in the prototype — privacy by construction). Live at [neuronika.vercel.app](https://neuronika.vercel.app); repo at [github.com/Capataina/Neuronika](https://github.com/Capataina/Neuronika).

Stack and components:
- **Note editor**: react-markdown + remark-gfm + react-syntax-highlighter + remark-math + rehype-katex (full markdown including code blocks, callouts, tables, LaTeX maths).
- **Layout**: React-Grid-Layout for an infinitely-scrollable masonry board with resizable / draggable notes.
- **AI tag generation**: Mistral Small 3.1 24B via OpenRouter (chosen for 96K context window + zero per-token cost + strong instruction-following); multi-layered tags ranging from generalised ("Fitness") to specific ("Tendon Health"); adaptive learning prioritises pre-existing tags for similar contexts to prevent tag fragmentation.
- **Three information-retrieval surfaces**: dynamic folder hierarchy generated from tags (allowing notes to appear in multiple folders simultaneously), interactive visual graph (vis.js, tag → tag → note tree), typo-tolerant context-aware semantic search (Fuse.js + bidirectional synonym dictionary as the client-side stand-in for a vector database).

Pilot study (4 participants, controlled comparison vs Obsidian, three retrieval tasks):

| Task | Time saved (Neuronika vs Obsidian) |
|---|---|
| Find a note by exact title | **300%** |
| Find a note by remembered context only | **433%** |
| Find all notes on a topic (group retrieval) | **339%** |
| **Average across all tasks** | **355%** |

The semantic context-aware search bar was the most-requested feature in the pre-experiment user survey (98% of respondents) and the most-used feature during the experiment; all 4 participants used it for all 3 tasks. Post-experiment Likert-scale survey on AI-generated tag accuracy showed strong alignment between AI tags and how participants would have labelled notes themselves. Ethics approval covered informed consent, data destruction post-grading, and anonymisation; no PII retained.

Conceptual lineage: the dissertation's central claim — that PKM systems work best when an intelligence layer (in 2025: an LLM doing adaptive tagging; today: Claude doing graph reasoning) supplements rather than replaces the user's own thinking — is the same thesis [[Projects/LifeOS/_Overview|LifeOS]] now operationalises with a substantially richer intelligence layer and a vault-as-substrate architecture. Neuronika was the proof-of-concept; LifeOS is the production version.

## Coursework projects mapped to modules

| Project | Module(s) | Trajectory to current portfolio |
|---|---|---|
| CNN for flower classification + facial recognition | Intelligent Systems: ML & Optimisation; Probabilistic & Deep Learning | Foundational for [[Projects/Image Browser/_Overview\|Image Browser]]'s 3-encoder ONNX pipeline (CLIP + DINOv2 + SigLIP-2) and [[Projects/NeuroDrive/_Overview\|NeuroDrive]]'s handwritten PPO. |
| MLP for body dysmorphia prediction | Probabilistic & Deep Learning | Predictive analytics across structured features. |
| RNN for ciphertext indistinguishability attacks | Cryptography + Probabilistic & Deep Learning | Cross-module application of sequence models to cryptanalysis. |
| Genetic algorithms via DEAP for beer quality prediction; evolutionary regression | Evolutionary & Adaptive Computing | Lineage to [[Projects/NeuroDrive/_Overview\|NeuroDrive]]'s biology-first sparse-graph learner with three-factor plasticity. |
| 8-person Java + libGDX simulation game (lead developer) | Engineering 1: Software & Systems Engineering | Team CI/CD + OO architecture; precursor to full-cycle ownership in [[Projects/Cernio/_Overview\|Cernio]], [[Projects/Nyquestro/_Overview\|Nyquestro]], and [[Projects/Aurix/_Overview\|Aurix]]. |
| Network traffic analysis (Wireshark, Nmap, tcpdump) | Network Security | Threat-modelling foundations applicable to [[Projects/Aurix/_Overview\|Aurix]]'s raw JSON-RPC + hand-crafted ABI encoding work. |
| Neuronika — AI-powered PKM application + dissertation on PKM and information retrieval | Year 3 dissertation + practical project | Substantial written dissertation reviewing the PKM and IR literature, identifying gaps, and arguing a specific design direction; Neuronika (TypeScript + React + Vite) built as the practical project addressing those gaps. Same problem domain as [[Projects/LifeOS/_Overview\|LifeOS]] — early thinking on the PKM space that LifeOS now approaches with Claude as the intelligence layer. |

## Notes

Coursework spanned the full systems-engineering arc: programming foundations (Python + Java), software engineering practice (CI/CD + OO + TDD), data structures and algorithms with complexity analysis, statistical and data-science methods, machine learning and deep learning (PyTorch + TensorFlow), evolutionary computation (DEAP), cryptography, network security, and human-centred design. All modules graded 100% coursework or coursework-heavy unless noted; the 8-person Engineering 1 project is the standout team-delivery experience.

See [[Projects/_Overview|Projects Overview]] for the post-degree portfolio and [[Profile/Professional/LinkedIn|LinkedIn]] § Education for the LinkedIn-UI-ready Description and Activities/societies fields.
